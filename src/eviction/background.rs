//! Background eviction task
//!
//! Runs eviction asynchronously in a background tokio task, avoiding
//! blocking `put()` operations. The task periodically checks cache size
//! and evicts objects according to the configured policy.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tracing::{debug, info, warn};

use super::{EvictionConfig, EvictionManager, EvictionPolicyType, LfuPolicy, LruPolicy, TtlPolicy};
use crate::eviction::policy::EvictionPolicy;
use crate::eviction::EvictionCandidate;

/// Trait for storage backends that support background eviction
pub trait EvictableStorage: Send + Sync + 'static {
    /// Get current cache size in bytes
    fn current_size(&self) -> anyhow::Result<u64>;

    /// Get all eviction candidates with their metadata
    fn get_eviction_candidates(&self) -> anyhow::Result<Vec<EvictionCandidate>>;

    /// Delete an object by ID
    fn delete_object(&self, id: &[u8]) -> anyhow::Result<()>;
}

/// Configuration for background eviction task
#[derive(Debug, Clone)]
pub struct BackgroundEvictionConfig {
    /// How often to check if eviction is needed
    pub check_interval: Duration,
    /// Eviction configuration (max_size, policy, etc.)
    pub eviction_config: EvictionConfig,
}

impl Default for BackgroundEvictionConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            eviction_config: EvictionConfig::default(),
        }
    }
}

impl BackgroundEvictionConfig {
    /// Create from eviction config with default check interval
    pub fn from_eviction_config(eviction_config: EvictionConfig) -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            eviction_config,
        }
    }

    /// Set the check interval
    #[allow(dead_code)]
    pub fn with_check_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }
}

/// Handle to control the background eviction task
pub struct BackgroundEvictionHandle {
    /// Signal to stop the background task
    shutdown: Arc<AtomicBool>,
    /// Notify to wake up the task for immediate eviction
    notify: Arc<Notify>,
    /// Join handle for the background task
    join_handle: Option<tokio::task::JoinHandle<()>>,
}

impl BackgroundEvictionHandle {
    /// Trigger an immediate eviction check (non-blocking)
    #[allow(dead_code)]
    pub fn trigger_eviction(&self) {
        self.notify.notify_one();
    }

    /// Stop the background eviction task
    pub async fn shutdown(mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.notify.notify_one();

        if let Some(handle) = self.join_handle.take() {
            // Wait for the task to finish with a timeout
            match tokio::time::timeout(Duration::from_secs(5), handle).await {
                Ok(Ok(())) => {
                    debug!("Background eviction task stopped");
                }
                Ok(Err(e)) => {
                    warn!("Background eviction task panicked: {}", e);
                }
                Err(_) => {
                    warn!("Background eviction task did not stop in time");
                }
            }
        }
    }

    /// Check if the background task is still running
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        !self.shutdown.load(Ordering::SeqCst)
    }
}

/// Spawn a background eviction task
///
/// Returns a handle that can be used to control the task.
pub fn spawn_background_eviction<S: EvictableStorage>(
    storage: Arc<S>,
    config: BackgroundEvictionConfig,
) -> BackgroundEvictionHandle {
    let shutdown = Arc::new(AtomicBool::new(false));
    let notify = Arc::new(Notify::new());

    let shutdown_clone = Arc::clone(&shutdown);
    let notify_clone = Arc::clone(&notify);

    // Log before moving config into the closure
    let check_interval = config.check_interval;

    let join_handle = tokio::spawn(async move {
        run_eviction_loop(storage, config, shutdown_clone, notify_clone).await;
    });

    info!(
        "Background eviction task started (interval: {:?})",
        check_interval
    );

    BackgroundEvictionHandle {
        shutdown,
        notify,
        join_handle: Some(join_handle),
    }
}

/// Main eviction loop
async fn run_eviction_loop<S: EvictableStorage>(
    storage: Arc<S>,
    config: BackgroundEvictionConfig,
    shutdown: Arc<AtomicBool>,
    notify: Arc<Notify>,
) {
    let eviction_manager = EvictionManager::new(config.eviction_config.clone());

    loop {
        // Wait for either the interval or a manual trigger
        tokio::select! {
            _ = tokio::time::sleep(config.check_interval) => {}
            _ = notify.notified() => {
                if shutdown.load(Ordering::SeqCst) {
                    debug!("Background eviction task received shutdown signal");
                    break;
                }
                debug!("Background eviction triggered manually");
            }
        }

        // Check shutdown again after waking
        if shutdown.load(Ordering::SeqCst) {
            break;
        }

        // Run eviction check
        if let Err(e) = run_eviction_cycle(&storage, &eviction_manager, &config.eviction_config) {
            warn!("Background eviction cycle failed: {}", e);
        }
    }

    info!("Background eviction task stopped");
}

/// Run a single eviction cycle
fn run_eviction_cycle<S: EvictableStorage>(
    storage: &Arc<S>,
    eviction_manager: &EvictionManager,
    config: &EvictionConfig,
) -> anyhow::Result<()> {
    let current_size = storage.current_size()?;

    if !eviction_manager.needs_eviction(current_size) {
        debug!(
            "Cache size {}MB is under limit {}MB, no eviction needed",
            current_size / (1024 * 1024),
            config.max_size_bytes / (1024 * 1024)
        );
        return Ok(());
    }

    let bytes_to_evict = eviction_manager.bytes_to_evict(current_size);
    info!(
        "Cache size {}MB exceeds limit {}MB, evicting {}MB",
        current_size / (1024 * 1024),
        config.max_size_bytes / (1024 * 1024),
        bytes_to_evict / (1024 * 1024)
    );

    let start = Instant::now();

    // Get all candidates
    let candidates = storage.get_eviction_candidates()?;

    // Select candidates to evict using the configured policy
    let policy: Box<dyn EvictionPolicy> = match config.policy {
        EvictionPolicyType::Lru => Box::new(LruPolicy),
        EvictionPolicyType::Lfu => Box::new(LfuPolicy),
        EvictionPolicyType::Ttl => Box::new(TtlPolicy::new(config.default_ttl_secs)),
    };

    let mut sorted_candidates = candidates;
    policy.sort_candidates(&mut sorted_candidates);

    // Select candidates until we've freed enough space
    let mut to_evict = Vec::new();
    let mut total_size = 0u64;

    for candidate in sorted_candidates {
        if total_size >= bytes_to_evict && !to_evict.is_empty() {
            break;
        }
        if to_evict.len() >= config.max_evictions_per_run {
            break;
        }

        total_size += candidate.size;
        to_evict.push(candidate);
    }

    // Evict selected objects
    let mut evicted_count = 0usize;
    let mut evicted_bytes = 0u64;

    for candidate in &to_evict {
        match storage.delete_object(&candidate.id) {
            Ok(()) => {
                evicted_count += 1;
                evicted_bytes += candidate.size;
                eviction_manager.record_eviction(candidate.size);
                debug!(
                    "Evicted object {} ({} bytes)",
                    hex::encode(&candidate.id),
                    candidate.size
                );
            }
            Err(e) => {
                warn!(
                    "Failed to evict object {}: {}",
                    hex::encode(&candidate.id),
                    e
                );
            }
        }
    }

    eviction_manager.record_run();
    let duration_ms = start.elapsed().as_millis() as u64;
    eviction_manager.log_summary(evicted_count, evicted_bytes, duration_ms);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Object metadata tuple: (size, accessed_at, access_count, created_at)
    type ObjectMetadata = (u64, i64, u64, i64);

    /// Mock storage for testing
    struct MockStorage {
        objects: Mutex<HashMap<Vec<u8>, ObjectMetadata>>,
        total_size: Mutex<u64>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                objects: Mutex::new(HashMap::new()),
                total_size: Mutex::new(0),
            }
        }

        fn add_object(&self, id: Vec<u8>, size: u64, accessed_at: i64, access_count: u64) {
            let mut objects = self.objects.lock().unwrap();
            let mut total = self.total_size.lock().unwrap();
            objects.insert(id, (size, accessed_at, access_count, 0));
            *total += size;
        }

        fn object_count(&self) -> usize {
            self.objects.lock().unwrap().len()
        }
    }

    impl EvictableStorage for MockStorage {
        fn current_size(&self) -> anyhow::Result<u64> {
            Ok(*self.total_size.lock().unwrap())
        }

        fn get_eviction_candidates(&self) -> anyhow::Result<Vec<EvictionCandidate>> {
            let objects = self.objects.lock().unwrap();
            Ok(objects
                .iter()
                .map(
                    |(id, (size, accessed_at, access_count, created_at))| EvictionCandidate {
                        id: id.clone(),
                        size: *size,
                        accessed_at: *accessed_at,
                        access_count: *access_count,
                        created_at: *created_at,
                    },
                )
                .collect())
        }

        fn delete_object(&self, id: &[u8]) -> anyhow::Result<()> {
            let mut objects = self.objects.lock().unwrap();
            let mut total = self.total_size.lock().unwrap();
            if let Some((size, _, _, _)) = objects.remove(id) {
                *total -= size;
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_background_eviction_triggers() {
        let storage = Arc::new(MockStorage::new());

        // Add objects totaling 1500 bytes
        storage.add_object(vec![1], 500, 100, 1);
        storage.add_object(vec![2], 500, 200, 2);
        storage.add_object(vec![3], 500, 300, 3);

        assert_eq!(storage.object_count(), 3);
        assert_eq!(storage.current_size().unwrap(), 1500);

        // Configure eviction at 1000 bytes max, target 90% = 900 bytes
        let config = BackgroundEvictionConfig {
            check_interval: Duration::from_millis(50),
            eviction_config: EvictionConfig {
                max_size_bytes: 1000,
                policy: EvictionPolicyType::Lru,
                target_ratio: 0.9,
                max_evictions_per_run: 100,
                ..Default::default()
            },
        };

        let handle = spawn_background_eviction(storage.clone(), config);

        // Wait for eviction to run
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should have evicted objects to get under 900 bytes
        // LRU will evict object 1 (oldest access) first, then object 2
        let size = storage.current_size().unwrap();
        assert!(size <= 900, "Expected size <= 900, got {}", size);

        handle.shutdown().await;
    }

    #[tokio::test]
    async fn test_background_eviction_manual_trigger() {
        let storage = Arc::new(MockStorage::new());

        // Add objects under the limit initially
        storage.add_object(vec![1], 100, 100, 1);

        let config = BackgroundEvictionConfig {
            check_interval: Duration::from_secs(60), // Long interval
            eviction_config: EvictionConfig {
                max_size_bytes: 500,
                policy: EvictionPolicyType::Lfu,
                target_ratio: 0.9,
                max_evictions_per_run: 100,
                ..Default::default()
            },
        };

        let handle = spawn_background_eviction(storage.clone(), config);

        // Add more objects to exceed limit
        storage.add_object(vec![2], 200, 200, 1);
        storage.add_object(vec![3], 300, 300, 5);

        assert_eq!(storage.current_size().unwrap(), 600);

        // Trigger manual eviction
        handle.trigger_eviction();

        // Wait for eviction to run
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should have evicted to get under 450 (90% of 500)
        let size = storage.current_size().unwrap();
        assert!(size <= 450, "Expected size <= 450, got {}", size);

        handle.shutdown().await;
    }

    #[tokio::test]
    async fn test_background_eviction_shutdown() {
        let storage = Arc::new(MockStorage::new());

        let config = BackgroundEvictionConfig {
            check_interval: Duration::from_millis(10),
            eviction_config: EvictionConfig::default(),
        };

        let handle = spawn_background_eviction(storage, config);

        assert!(handle.is_running());

        handle.shutdown().await;

        // Task should be stopped (handle consumed)
    }

    #[tokio::test]
    async fn test_background_eviction_no_eviction_needed() {
        let storage = Arc::new(MockStorage::new());

        // Add objects under the limit
        storage.add_object(vec![1], 100, 100, 1);
        storage.add_object(vec![2], 100, 200, 2);

        let config = BackgroundEvictionConfig {
            check_interval: Duration::from_millis(50),
            eviction_config: EvictionConfig {
                max_size_bytes: 1000,
                policy: EvictionPolicyType::Lfu,
                target_ratio: 0.9,
                max_evictions_per_run: 100,
                ..Default::default()
            },
        };

        let handle = spawn_background_eviction(storage.clone(), config);

        // Wait for a few cycles
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should not have evicted anything
        assert_eq!(storage.object_count(), 2);
        assert_eq!(storage.current_size().unwrap(), 200);

        handle.shutdown().await;
    }

    #[tokio::test]
    async fn test_background_eviction_lfu_policy() {
        let storage = Arc::new(MockStorage::new());

        // Add objects with different access counts
        storage.add_object(vec![1], 400, 100, 10); // High access count - keep
        storage.add_object(vec![2], 400, 200, 1); // Low access count - evict first
        storage.add_object(vec![3], 400, 300, 5); // Medium access count

        let config = BackgroundEvictionConfig {
            check_interval: Duration::from_millis(50),
            eviction_config: EvictionConfig {
                max_size_bytes: 1000,
                policy: EvictionPolicyType::Lfu,
                target_ratio: 0.9,
                max_evictions_per_run: 100,
                ..Default::default()
            },
        };

        let handle = spawn_background_eviction(storage.clone(), config);

        // Wait for eviction
        tokio::time::sleep(Duration::from_millis(100)).await;

        // LFU should evict object 2 first (lowest access count)
        {
            let objects = storage.objects.lock().unwrap();
            assert!(
                !objects.contains_key(&vec![2]),
                "Object 2 should have been evicted (lowest access count)"
            );
            assert!(
                objects.contains_key(&vec![1]),
                "Object 1 should NOT have been evicted (highest access count)"
            );
        } // Mutex guard is dropped here before await

        handle.shutdown().await;
    }
}
