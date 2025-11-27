use super::{Storage, StorageStats};
use crate::eviction::{EvictableStorage, EvictionCandidate, EvictionConfig, EvictionManager};
use anyhow::{Context, Result};
use crossbeam_channel::{bounded, Sender};
use rocksdb::{IteratorMode, Options, DB};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// RocksDB column families for metadata storage
///
/// Column families provide logical partitioning of data within RocksDB.
/// We use separate column families for different types of metadata:
/// - "default": Object metadata (size, timestamps, access count)
/// - "index_accessed": Secondary index for accessed_at (for LRU eviction)
/// - "index_access_count": Secondary index for access_count (for LFU eviction)
const CF_DEFAULT: &str = "default";
const CF_INDEX_ACCESSED: &str = "index_accessed";
const CF_INDEX_ACCESS_COUNT: &str = "index_access_count";

/// Metadata stored for each cached object in RocksDB
///
/// Format (binary encoding):
/// - size: u64 (8 bytes)
/// - created_at: i64 (8 bytes)
/// - accessed_at: i64 (8 bytes)
/// - access_count: u64 (8 bytes)
///
/// Total: 32 bytes per object
#[derive(Debug, Clone)]
struct ObjectMetadata {
    size: u64,
    created_at: i64,
    accessed_at: i64,
    access_count: u64,
}

impl ObjectMetadata {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(&self.size.to_le_bytes());
        bytes.extend_from_slice(&self.created_at.to_le_bytes());
        bytes.extend_from_slice(&self.accessed_at.to_le_bytes());
        bytes.extend_from_slice(&self.access_count.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            anyhow::bail!(
                "Invalid metadata size: expected 32 bytes, got {}",
                bytes.len()
            );
        }

        Ok(Self {
            size: u64::from_le_bytes(bytes[0..8].try_into()?),
            created_at: i64::from_le_bytes(bytes[8..16].try_into()?),
            accessed_at: i64::from_le_bytes(bytes[16..24].try_into()?),
            access_count: u64::from_le_bytes(bytes[24..32].try_into()?),
        })
    }
}

/// Message type for batched access tracking updates
#[derive(Debug, Clone)]
struct TouchMessage {
    id: Vec<u8>,
    timestamp: i64,
}

/// Filesystem-based storage with RocksDB metadata tracking
///
/// Layout:
/// - `.fabrik/cache/objects/ab/cd1234...` - Content-addressed blob storage (first 2 chars = subdir)
/// - `.fabrik/cache/metadata/` - RocksDB database for access tracking and eviction
///
/// Optimizations:
/// - RocksDB provides concurrent reads/writes out of the box
/// - Async batched access tracking (touch operations)
/// - Snappy compression for metadata
/// - Column families for efficient indexing (LRU/LFU eviction)
/// - Automatic eviction when cache exceeds max_size
#[derive(Clone)]
pub struct FilesystemStorage {
    objects_dir: PathBuf,
    db: Arc<DB>,
    touch_sender: Sender<TouchMessage>,
    worker_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    #[allow(dead_code)]
    eviction_manager: Option<Arc<EvictionManager>>,
}

impl FilesystemStorage {
    /// Create a new filesystem storage at the given cache directory
    ///
    /// Opens RocksDB database with column families for metadata tracking.
    /// Spawns a background worker for batched access tracking.
    #[allow(dead_code)]
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self> {
        Self::with_eviction(cache_dir, None)
    }

    /// Create a new filesystem storage with eviction configuration
    ///
    /// When eviction config is provided, the storage will automatically
    /// evict objects when the cache exceeds `max_size`.
    pub fn with_eviction<P: AsRef<Path>>(
        cache_dir: P,
        eviction_config: Option<EvictionConfig>,
    ) -> Result<Self> {
        let cache_dir = cache_dir.as_ref();
        let objects_dir = cache_dir.join("objects");
        let db_path = cache_dir.join("metadata");

        // Create directories
        fs::create_dir_all(&objects_dir).context("Failed to create objects directory")?;

        // Configure RocksDB options
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        // Performance tuning
        opts.set_compression_type(rocksdb::DBCompressionType::Snappy);
        opts.increase_parallelism(num_cpus::get() as i32);
        opts.set_max_background_jobs(4);

        // Disable statistics to reduce overhead and potential shutdown issues
        opts.set_statistics_level(rocksdb::statistics::StatsLevel::DisableAll);

        // Write buffer settings for better write performance
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB
        opts.set_max_write_buffer_number(3);

        // Open database with column families
        let db = DB::open_cf(
            &opts,
            &db_path,
            vec![CF_DEFAULT, CF_INDEX_ACCESSED, CF_INDEX_ACCESS_COUNT],
        )
        .context("Failed to open RocksDB database")?;

        let db = Arc::new(db);

        // Create channel for async touch operations (buffered for batching)
        let (touch_sender, touch_receiver) = bounded::<TouchMessage>(1000);

        // Spawn background worker for batched access tracking
        let db_clone = Arc::clone(&db);
        let worker_handle = thread::spawn(move || {
            let mut batch = Vec::with_capacity(100);
            let batch_timeout = Duration::from_millis(100);

            loop {
                // Collect messages for up to 100ms or 100 items
                match touch_receiver.recv_timeout(batch_timeout) {
                    Ok(msg) => {
                        batch.push(msg);

                        // Drain the channel up to 100 items
                        while batch.len() < 100 {
                            match touch_receiver.try_recv() {
                                Ok(msg) => batch.push(msg),
                                Err(_) => break,
                            }
                        }

                        // Execute batch update
                        if let Err(e) = Self::batch_touch(&db_clone, &batch) {
                            debug!("Failed to batch update access tracking: {}", e);
                        }

                        batch.clear();
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                        // Flush any pending items on timeout
                        if !batch.is_empty() {
                            if let Err(e) = Self::batch_touch(&db_clone, &batch) {
                                debug!("Failed to batch update access tracking: {}", e);
                            }
                            batch.clear();
                        }
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                        // Channel closed, flush and exit
                        if !batch.is_empty() {
                            let _ = Self::batch_touch(&db_clone, &batch);
                        }
                        break;
                    }
                }
            }
        });

        // Create eviction manager if config provided
        let eviction_manager = eviction_config.map(|config| Arc::new(EvictionManager::new(config)));

        Ok(Self {
            objects_dir,
            db,
            touch_sender,
            worker_handle: Arc::new(Mutex::new(Some(worker_handle))),
            eviction_manager,
        })
    }

    /// Batch update access tracking for multiple objects
    fn batch_touch(db: &Arc<DB>, batch: &[TouchMessage]) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        // Use RocksDB write batch for atomic updates
        let mut write_batch = rocksdb::WriteBatch::default();

        for msg in batch {
            // Get existing metadata
            if let Some(existing_bytes) = db.get(&msg.id)? {
                if let Ok(mut metadata) = ObjectMetadata::from_bytes(&existing_bytes) {
                    // Update access tracking
                    metadata.accessed_at = msg.timestamp;
                    metadata.access_count += 1;

                    // Write updated metadata
                    write_batch.put(&msg.id, metadata.to_bytes());

                    // Update secondary indexes (for efficient LRU/LFU queries)
                    let cf_accessed = db
                        .cf_handle(CF_INDEX_ACCESSED)
                        .context("Failed to get CF_INDEX_ACCESSED handle")?;
                    let cf_access_count = db
                        .cf_handle(CF_INDEX_ACCESS_COUNT)
                        .context("Failed to get CF_INDEX_ACCESS_COUNT handle")?;

                    // Index key: timestamp + id (for range queries)
                    let mut accessed_key = msg.timestamp.to_le_bytes().to_vec();
                    accessed_key.extend_from_slice(&msg.id);

                    let mut access_count_key = metadata.access_count.to_le_bytes().to_vec();
                    access_count_key.extend_from_slice(&msg.id);

                    write_batch.put_cf(cf_accessed, accessed_key, b"");
                    write_batch.put_cf(cf_access_count, access_count_key, b"");
                }
            }
        }

        db.write(write_batch)
            .context("Failed to write batch update")?;
        debug!("Batched {} access tracking updates", batch.len());

        Ok(())
    }

    /// Convert blob ID to filesystem path
    /// Uses git-style sharding: first 2 hex chars as subdirectory
    fn id_to_path(&self, id: &[u8]) -> PathBuf {
        let hex_id = hex::encode(id);
        let (prefix, suffix) = hex_id.split_at(2);
        self.objects_dir.join(prefix).join(suffix)
    }

    /// Get current Unix timestamp
    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    /// Get all eviction candidates with their metadata
    ///
    /// Returns all objects in the cache with metadata needed for eviction decisions.
    /// Used by the eviction manager to select which objects to evict.
    pub fn get_eviction_candidates(&self) -> Result<Vec<EvictionCandidate>> {
        let mut candidates = Vec::new();
        let iter = self.db.iterator(IteratorMode::Start);

        for item in iter {
            let (key, value) = item?;
            if let Ok(metadata) = ObjectMetadata::from_bytes(&value) {
                candidates.push(EvictionCandidate {
                    id: key.to_vec(),
                    size: metadata.size,
                    accessed_at: metadata.accessed_at,
                    access_count: metadata.access_count,
                    created_at: metadata.created_at,
                });
            }
        }

        Ok(candidates)
    }

    /// Run eviction if needed
    ///
    /// Checks if the cache exceeds max_size and evicts objects according to the
    /// configured policy until the cache is under the target size.
    ///
    /// Returns the number of objects evicted and total bytes freed.
    #[allow(dead_code)]
    pub fn run_eviction_if_needed(&self) -> Result<(usize, u64)> {
        let Some(ref eviction_manager) = self.eviction_manager else {
            return Ok((0, 0));
        };

        let stats = self.stats()?;
        if !eviction_manager.needs_eviction(stats.total_bytes) {
            debug!(
                "Cache size {}MB is under limit {}MB, no eviction needed",
                stats.total_bytes / (1024 * 1024),
                eviction_manager.config().max_size_bytes / (1024 * 1024)
            );
            return Ok((0, 0));
        }

        let bytes_to_evict = eviction_manager.bytes_to_evict(stats.total_bytes);
        info!(
            "Cache size {}MB exceeds limit {}MB, evicting {}MB",
            stats.total_bytes / (1024 * 1024),
            eviction_manager.config().max_size_bytes / (1024 * 1024),
            bytes_to_evict / (1024 * 1024)
        );

        let start = Instant::now();

        // Get all candidates
        let candidates = self.get_eviction_candidates()?;

        // Select candidates to evict
        let to_evict = eviction_manager.select_candidates(&candidates, bytes_to_evict);

        // Evict selected objects
        let mut evicted_count = 0usize;
        let mut evicted_bytes = 0u64;

        for candidate in &to_evict {
            match self.delete(&candidate.id) {
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

        Ok((evicted_count, evicted_bytes))
    }

    /// Force eviction to free the specified amount of space
    ///
    /// Unlike `run_eviction_if_needed`, this will evict objects even if
    /// the cache is under max_size.
    #[allow(dead_code)]
    pub fn force_eviction(&self, bytes_to_free: u64) -> Result<(usize, u64)> {
        let Some(ref eviction_manager) = self.eviction_manager else {
            anyhow::bail!("Eviction manager not configured");
        };

        let start = Instant::now();
        info!(
            "Force eviction requested: freeing {}MB",
            bytes_to_free / (1024 * 1024)
        );

        // Get all candidates
        let candidates = self.get_eviction_candidates()?;

        // Select candidates to evict
        let to_evict = eviction_manager.select_candidates(&candidates, bytes_to_free);

        // Evict selected objects
        let mut evicted_count = 0usize;
        let mut evicted_bytes = 0u64;

        for candidate in &to_evict {
            match self.delete(&candidate.id) {
                Ok(()) => {
                    evicted_count += 1;
                    evicted_bytes += candidate.size;
                    eviction_manager.record_eviction(candidate.size);
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

        Ok((evicted_count, evicted_bytes))
    }

    /// Get eviction statistics
    #[allow(dead_code)]
    pub fn eviction_stats(&self) -> Option<(u64, u64, u64)> {
        self.eviction_manager.as_ref().map(|m| {
            let stats = m.stats();
            (
                stats.get_evictions_total(),
                stats.get_bytes_evicted(),
                stats.get_eviction_runs(),
            )
        })
    }

    /// Check if eviction is enabled
    #[allow(dead_code)]
    pub fn has_eviction(&self) -> bool {
        self.eviction_manager.is_some()
    }

    /// Get the objects directory path
    #[allow(dead_code)]
    pub fn objects_dir(&self) -> &Path {
        &self.objects_dir
    }
}

/// Implementation of EvictableStorage for background eviction
impl EvictableStorage for FilesystemStorage {
    fn current_size(&self) -> Result<u64> {
        let stats = self.stats()?;
        Ok(stats.total_bytes)
    }

    fn get_eviction_candidates(&self) -> Result<Vec<EvictionCandidate>> {
        // Call the existing method
        FilesystemStorage::get_eviction_candidates(self)
    }

    fn delete_object(&self, id: &[u8]) -> Result<()> {
        self.delete(id)
    }
}

impl Drop for FilesystemStorage {
    fn drop(&mut self) {
        // Step 1: Join the background worker thread to ensure it exits cleanly
        // Note: The channel will be closed automatically when all senders are dropped,
        // which happens when this struct is dropped (self.touch_sender is dropped).
        // We must join the thread BEFORE dropping self.touch_sender to avoid race conditions.
        if let Ok(mut handle_lock) = self.worker_handle.lock() {
            if let Some(handle) = handle_lock.take() {
                // Drop the touch_sender before joining to signal the thread to exit
                // Create a temporary scope to ensure sender is dropped
                {
                    // Move touch_sender out and drop it to close the channel
                    let _sender = std::mem::replace(
                        &mut self.touch_sender,
                        bounded(0).0, // Replace with a dummy closed channel
                    );
                    // _sender drops here, closing the original channel
                }

                // Now wait for the worker thread to finish
                let _ = handle.join();
            }
        }

        // Step 2: Flush any pending writes to ensure data consistency
        if let Err(e) = self.db.flush() {
            eprintln!("Warning: Failed to flush RocksDB on shutdown: {}", e);
        }

        // Step 3: Cancel all background work to ensure clean shutdown
        // This is critical on Linux to avoid pthread lock errors
        self.db.cancel_all_background_work(true);

        // Step 4: Give RocksDB background threads time to fully terminate
        // Reduced to 50ms since we now properly wait for the worker thread
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

impl Storage for FilesystemStorage {
    fn put(&self, id: &[u8], data: &[u8]) -> Result<()> {
        // Note: Eviction is handled by a background task (spawn_background_eviction)
        // to avoid blocking put() operations. The background task periodically
        // checks cache size and evicts objects according to the configured policy.

        let path = self.id_to_path(id);

        // Create parent directory
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create parent directory")?;
        }

        // Write data atomically (write to temp file, then rename)
        // Use PID + thread ID to avoid collisions in concurrent writes
        let temp_name = format!(
            "{}.tmp.{}.{:?}",
            path.file_name().unwrap().to_str().unwrap(),
            std::process::id(),
            thread::current().id()
        );
        let temp_path = path.parent().unwrap().join(temp_name);

        let mut file = fs::File::create(&temp_path).context("Failed to create temp file")?;
        file.write_all(data).context("Failed to write data")?;
        file.sync_all().context("Failed to sync file")?;
        fs::rename(&temp_path, &path).context("Failed to rename temp file")?;

        // Update metadata in RocksDB
        let now = Self::current_timestamp();
        let size = data.len() as u64;

        // Check if object already exists to preserve access_count
        let access_count = if let Some(existing_bytes) = self.db.get(id)? {
            ObjectMetadata::from_bytes(&existing_bytes)
                .map(|m| m.access_count)
                .unwrap_or(0)
        } else {
            0
        };

        let metadata = ObjectMetadata {
            size,
            created_at: now,
            accessed_at: now,
            access_count,
        };

        self.db
            .put(id, metadata.to_bytes())
            .context("Failed to update metadata")?;

        Ok(())
    }

    fn get(&self, id: &[u8]) -> Result<Option<Vec<u8>>> {
        let path = self.id_to_path(id);

        if !path.exists() {
            return Ok(None);
        }

        // Read data
        let data = fs::read(&path).context("Failed to read object")?;

        // Update access metadata asynchronously (non-blocking)
        self.touch(id)?;

        Ok(Some(data))
    }

    fn exists(&self, id: &[u8]) -> Result<bool> {
        let path = self.id_to_path(id);
        Ok(path.exists())
    }

    fn delete(&self, id: &[u8]) -> Result<()> {
        let path = self.id_to_path(id);

        // Delete file
        if path.exists() {
            fs::remove_file(&path).context("Failed to delete object")?;
        }

        // Delete metadata from RocksDB
        self.db.delete(id).context("Failed to delete metadata")?;

        Ok(())
    }

    fn size(&self, id: &[u8]) -> Result<Option<u64>> {
        if let Some(metadata_bytes) = self.db.get(id)? {
            let metadata = ObjectMetadata::from_bytes(&metadata_bytes)?;
            Ok(Some(metadata.size))
        } else {
            Ok(None)
        }
    }

    fn touch(&self, id: &[u8]) -> Result<()> {
        // Send to async batch worker (non-blocking)
        let msg = TouchMessage {
            id: id.to_vec(),
            timestamp: Self::current_timestamp(),
        };

        // Use try_send to avoid blocking if channel is full
        // If channel is full, we simply drop the update (acceptable trade-off for performance)
        self.touch_sender.try_send(msg).ok();

        Ok(())
    }

    fn list_ids(&self) -> Result<Vec<Vec<u8>>> {
        let mut ids = Vec::new();
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);

        for item in iter {
            let (key, _) = item?;
            ids.push(key.to_vec());
        }

        Ok(ids)
    }

    fn stats(&self) -> Result<StorageStats> {
        let mut total_objects = 0u64;
        let mut total_bytes = 0u64;

        let iter = self.db.iterator(rocksdb::IteratorMode::Start);

        for item in iter {
            let (_, value) = item?;
            if let Ok(metadata) = ObjectMetadata::from_bytes(&value) {
                total_objects += 1;
                total_bytes += metadata.size;
            }
        }

        Ok(StorageStats {
            total_objects,
            total_bytes,
            cache_dir: self.objects_dir.parent().unwrap().to_path_buf(),
        })
    }
}

/// Hash data using SHA256
#[allow(dead_code)]
pub fn hash_data(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_filesystem_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FilesystemStorage::new(temp_dir.path()).unwrap();

        // Test put and get
        let id = hash_data(b"hello");
        storage.put(&id, b"hello world").unwrap();

        let data = storage.get(&id).unwrap();
        assert_eq!(data, Some(b"hello world".to_vec()));

        // Test exists
        assert!(storage.exists(&id).unwrap());

        // Test size
        assert_eq!(storage.size(&id).unwrap(), Some(11));

        // Test stats
        let stats = storage.stats().unwrap();
        assert_eq!(stats.total_objects, 1);
        assert_eq!(stats.total_bytes, 11);

        // Test delete
        storage.delete(&id).unwrap();
        assert!(!storage.exists(&id).unwrap());
    }
}
