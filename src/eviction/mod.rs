//! Cache eviction module
//!
//! Provides eviction policies and management for the Fabrik cache:
//! - **LRU** (Least Recently Used): Evicts objects that haven't been accessed recently
//! - **LFU** (Least Frequently Used): Evicts objects with the lowest access count
//! - **TTL** (Time To Live): Evicts objects older than the configured TTL
//!
//! ## Architecture
//!
//! Eviction runs asynchronously in a background task:
//! - Periodically checks cache size (default: every 30 seconds)
//! - When cache exceeds `max_size`, evicts until 90% of max_size
//! - Non-blocking: `put()` operations are never delayed by eviction
//!
//! ## Configuration
//!
//! ```toml
//! [cache]
//! dir = ".fabrik/cache"
//! max_size = "5GB"
//! eviction_policy = "lfu"  # lru, lfu, or ttl
//! default_ttl = "7d"       # Used by TTL policy
//! ```

use anyhow::{Context, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, info};

mod background;
mod policy;

pub use background::{spawn_background_eviction, BackgroundEvictionConfig, EvictableStorage};

// Re-export for public API (may be used by consumers)
#[allow(unused_imports)]
pub use background::BackgroundEvictionHandle;
pub use policy::{EvictionCandidate, EvictionPolicy, LfuPolicy, LruPolicy, TtlPolicy};

/// Eviction statistics
#[derive(Debug, Default)]
pub struct EvictionStats {
    /// Total number of evictions performed
    pub evictions_total: AtomicU64,
    /// Total bytes evicted
    pub bytes_evicted: AtomicU64,
    /// Number of eviction runs
    pub eviction_runs: AtomicU64,
}

impl EvictionStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_eviction(&self, bytes: u64) {
        self.evictions_total.fetch_add(1, Ordering::Relaxed);
        self.bytes_evicted.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_run(&self) {
        self.eviction_runs.fetch_add(1, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub fn get_evictions_total(&self) -> u64 {
        self.evictions_total.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub fn get_bytes_evicted(&self) -> u64 {
        self.bytes_evicted.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub fn get_eviction_runs(&self) -> u64 {
        self.eviction_runs.load(Ordering::Relaxed)
    }
}

/// Eviction manager configuration
#[derive(Debug, Clone)]
pub struct EvictionConfig {
    /// Maximum cache size in bytes
    pub max_size_bytes: u64,
    /// Eviction policy type
    pub policy: EvictionPolicyType,
    /// Default TTL in seconds (for TTL policy)
    pub default_ttl_secs: u64,
    /// Target size after eviction (percentage of max_size)
    /// Default: 0.9 (evict until 90% of max_size)
    pub target_ratio: f64,
    /// Maximum objects to evict per run
    pub max_evictions_per_run: usize,
}

impl Default for EvictionConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 5 * 1024 * 1024 * 1024, // 5GB
            policy: EvictionPolicyType::Lfu,
            default_ttl_secs: 7 * 24 * 60 * 60, // 7 days
            target_ratio: 0.9,
            max_evictions_per_run: 1000,
        }
    }
}

impl EvictionConfig {
    /// Parse max_size string (e.g., "5GB", "100MB", "1TB") into bytes
    pub fn parse_size(size_str: &str) -> Result<u64> {
        let size_str = size_str.trim().to_uppercase();

        if let Some(num) = size_str.strip_suffix("TB") {
            let num: u64 = num.trim().parse().context("Invalid size number")?;
            Ok(num * 1024 * 1024 * 1024 * 1024)
        } else if let Some(num) = size_str.strip_suffix("GB") {
            let num: u64 = num.trim().parse().context("Invalid size number")?;
            Ok(num * 1024 * 1024 * 1024)
        } else if let Some(num) = size_str.strip_suffix("MB") {
            let num: u64 = num.trim().parse().context("Invalid size number")?;
            Ok(num * 1024 * 1024)
        } else if let Some(num) = size_str.strip_suffix("KB") {
            let num: u64 = num.trim().parse().context("Invalid size number")?;
            Ok(num * 1024)
        } else {
            // Assume bytes
            size_str.parse().context("Invalid size format")
        }
    }

    /// Parse TTL string (e.g., "7d", "24h", "30m") into seconds
    pub fn parse_ttl(ttl_str: &str) -> Result<u64> {
        let ttl_str = ttl_str.trim().to_lowercase();

        if let Some(num) = ttl_str.strip_suffix('d') {
            let num: u64 = num.trim().parse().context("Invalid TTL number")?;
            Ok(num * 24 * 60 * 60)
        } else if let Some(num) = ttl_str.strip_suffix('h') {
            let num: u64 = num.trim().parse().context("Invalid TTL number")?;
            Ok(num * 60 * 60)
        } else if let Some(num) = ttl_str.strip_suffix('m') {
            let num: u64 = num.trim().parse().context("Invalid TTL number")?;
            Ok(num * 60)
        } else if let Some(num) = ttl_str.strip_suffix('s') {
            let num: u64 = num.trim().parse().context("Invalid TTL number")?;
            Ok(num)
        } else {
            // Assume seconds
            ttl_str.parse().context("Invalid TTL format")
        }
    }

    /// Create config from cache config strings
    pub fn from_cache_config(
        max_size: &str,
        eviction_policy: &str,
        default_ttl: &str,
    ) -> Result<Self> {
        let max_size_bytes = Self::parse_size(max_size)?;
        let policy = eviction_policy.parse()?;
        let default_ttl_secs = Self::parse_ttl(default_ttl)?;

        Ok(Self {
            max_size_bytes,
            policy,
            default_ttl_secs,
            ..Default::default()
        })
    }

    /// Get target size in bytes (after eviction)
    pub fn target_size_bytes(&self) -> u64 {
        (self.max_size_bytes as f64 * self.target_ratio) as u64
    }
}

/// Eviction policy type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicyType {
    /// Least Recently Used
    Lru,
    /// Least Frequently Used
    Lfu,
    /// Time To Live (age-based)
    Ttl,
}

impl std::str::FromStr for EvictionPolicyType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "lru" => Ok(Self::Lru),
            "lfu" => Ok(Self::Lfu),
            "ttl" => Ok(Self::Ttl),
            _ => anyhow::bail!("Invalid eviction policy: {}. Must be lru, lfu, or ttl", s),
        }
    }
}

impl EvictionPolicyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lru => "lru",
            Self::Lfu => "lfu",
            Self::Ttl => "ttl",
        }
    }
}

/// Eviction manager
///
/// Manages cache eviction based on configured policy
pub struct EvictionManager {
    config: EvictionConfig,
    stats: Arc<EvictionStats>,
}

impl EvictionManager {
    pub fn new(config: EvictionConfig) -> Self {
        info!(
            "Eviction manager initialized: policy={}, max_size={}MB, target_ratio={}",
            config.policy.as_str(),
            config.max_size_bytes / (1024 * 1024),
            config.target_ratio
        );

        Self {
            config,
            stats: Arc::new(EvictionStats::new()),
        }
    }

    /// Get eviction statistics
    #[allow(dead_code)]
    pub fn stats(&self) -> Arc<EvictionStats> {
        Arc::clone(&self.stats)
    }

    /// Get configuration
    #[allow(dead_code)]
    pub fn config(&self) -> &EvictionConfig {
        &self.config
    }

    /// Check if eviction is needed based on current cache size
    pub fn needs_eviction(&self, current_size_bytes: u64) -> bool {
        current_size_bytes > self.config.max_size_bytes
    }

    /// Calculate how many bytes need to be evicted
    pub fn bytes_to_evict(&self, current_size_bytes: u64) -> u64 {
        if current_size_bytes <= self.config.target_size_bytes() {
            return 0;
        }
        current_size_bytes - self.config.target_size_bytes()
    }

    /// Select candidates for eviction using the configured policy
    ///
    /// Returns a list of (id, size) tuples to evict, ordered by eviction priority
    #[allow(dead_code)]
    pub fn select_candidates(
        &self,
        candidates: &[EvictionCandidate],
        bytes_to_evict: u64,
    ) -> Vec<EvictionCandidate> {
        let policy: Box<dyn EvictionPolicy> = match self.config.policy {
            EvictionPolicyType::Lru => Box::new(LruPolicy),
            EvictionPolicyType::Lfu => Box::new(LfuPolicy),
            EvictionPolicyType::Ttl => Box::new(TtlPolicy::new(self.config.default_ttl_secs)),
        };

        let mut sorted_candidates = candidates.to_vec();
        policy.sort_candidates(&mut sorted_candidates);

        // Select candidates until we've freed enough space
        let mut selected = Vec::new();
        let mut total_size = 0u64;

        for candidate in sorted_candidates {
            if total_size >= bytes_to_evict && !selected.is_empty() {
                break;
            }
            if selected.len() >= self.config.max_evictions_per_run {
                break;
            }

            total_size += candidate.size;
            selected.push(candidate);
        }

        debug!(
            "Selected {} candidates for eviction ({} bytes)",
            selected.len(),
            total_size
        );

        selected
    }

    /// Record an eviction
    pub fn record_eviction(&self, bytes: u64) {
        self.stats.record_eviction(bytes);
    }

    /// Record an eviction run
    pub fn record_run(&self) {
        self.stats.record_run();
    }

    /// Log eviction summary
    pub fn log_summary(&self, evicted_count: usize, evicted_bytes: u64, duration_ms: u64) {
        if evicted_count > 0 {
            info!(
                "Eviction complete: evicted {} objects ({} MB) in {}ms",
                evicted_count,
                evicted_bytes / (1024 * 1024),
                duration_ms
            );
        } else {
            debug!("Eviction check complete: no objects evicted");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(
            EvictionConfig::parse_size("5GB").unwrap(),
            5 * 1024 * 1024 * 1024
        );
        assert_eq!(
            EvictionConfig::parse_size("100MB").unwrap(),
            100 * 1024 * 1024
        );
        assert_eq!(
            EvictionConfig::parse_size("1TB").unwrap(),
            1024 * 1024 * 1024 * 1024
        );
        assert_eq!(EvictionConfig::parse_size("512KB").unwrap(), 512 * 1024);
        assert_eq!(EvictionConfig::parse_size("1024").unwrap(), 1024);
    }

    #[test]
    fn test_parse_ttl() {
        assert_eq!(EvictionConfig::parse_ttl("7d").unwrap(), 7 * 24 * 60 * 60);
        assert_eq!(EvictionConfig::parse_ttl("24h").unwrap(), 24 * 60 * 60);
        assert_eq!(EvictionConfig::parse_ttl("30m").unwrap(), 30 * 60);
        assert_eq!(EvictionConfig::parse_ttl("3600s").unwrap(), 3600);
        assert_eq!(EvictionConfig::parse_ttl("3600").unwrap(), 3600);
    }

    #[test]
    fn test_eviction_policy_type() {
        assert_eq!(
            "lru".parse::<EvictionPolicyType>().unwrap(),
            EvictionPolicyType::Lru
        );
        assert_eq!(
            "LFU".parse::<EvictionPolicyType>().unwrap(),
            EvictionPolicyType::Lfu
        );
        assert_eq!(
            "ttl".parse::<EvictionPolicyType>().unwrap(),
            EvictionPolicyType::Ttl
        );
        assert!("invalid".parse::<EvictionPolicyType>().is_err());
    }

    #[test]
    fn test_needs_eviction() {
        let config = EvictionConfig {
            max_size_bytes: 1000,
            ..Default::default()
        };
        let manager = EvictionManager::new(config);

        assert!(!manager.needs_eviction(500));
        assert!(!manager.needs_eviction(1000));
        assert!(manager.needs_eviction(1001));
    }

    #[test]
    fn test_bytes_to_evict() {
        let config = EvictionConfig {
            max_size_bytes: 1000,
            target_ratio: 0.9,
            ..Default::default()
        };
        let manager = EvictionManager::new(config);

        // Target is 900 bytes (90% of 1000)
        assert_eq!(manager.bytes_to_evict(900), 0);
        assert_eq!(manager.bytes_to_evict(1000), 100);
        assert_eq!(manager.bytes_to_evict(1200), 300);
    }

    #[test]
    fn test_select_candidates_lru() {
        let config = EvictionConfig {
            policy: EvictionPolicyType::Lru,
            max_evictions_per_run: 100,
            ..Default::default()
        };
        let manager = EvictionManager::new(config);

        let candidates = vec![
            EvictionCandidate {
                id: vec![1],
                size: 100,
                accessed_at: 1000,
                access_count: 5,
                created_at: 500,
            },
            EvictionCandidate {
                id: vec![2],
                size: 100,
                accessed_at: 500, // Older access - should be evicted first
                access_count: 10,
                created_at: 400,
            },
            EvictionCandidate {
                id: vec![3],
                size: 100,
                accessed_at: 2000,
                access_count: 1,
                created_at: 600,
            },
        ];

        let selected = manager.select_candidates(&candidates, 150);

        // LRU should select id=2 first (oldest access), then id=1
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].id, vec![2]);
        assert_eq!(selected[1].id, vec![1]);
    }

    #[test]
    fn test_select_candidates_lfu() {
        let config = EvictionConfig {
            policy: EvictionPolicyType::Lfu,
            max_evictions_per_run: 100,
            ..Default::default()
        };
        let manager = EvictionManager::new(config);

        let candidates = vec![
            EvictionCandidate {
                id: vec![1],
                size: 100,
                accessed_at: 1000,
                access_count: 5,
                created_at: 500,
            },
            EvictionCandidate {
                id: vec![2],
                size: 100,
                accessed_at: 500,
                access_count: 1, // Lowest access count - should be evicted first
                created_at: 400,
            },
            EvictionCandidate {
                id: vec![3],
                size: 100,
                accessed_at: 2000,
                access_count: 10,
                created_at: 600,
            },
        ];

        let selected = manager.select_candidates(&candidates, 150);

        // LFU should select id=2 first (lowest count), then id=1
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].id, vec![2]);
        assert_eq!(selected[1].id, vec![1]);
    }
}
