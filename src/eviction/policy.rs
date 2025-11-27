//! Eviction policy implementations
//!
//! Each policy defines how candidates are sorted for eviction:
//! - **LRU**: Sort by `accessed_at` (oldest first)
//! - **LFU**: Sort by `access_count` (lowest first)
//! - **TTL**: Sort by `created_at`, only select expired objects

use std::time::{SystemTime, UNIX_EPOCH};

/// Candidate for eviction with all metadata needed for policy decisions
#[derive(Debug, Clone)]
pub struct EvictionCandidate {
    /// Object ID (hash)
    pub id: Vec<u8>,
    /// Object size in bytes
    pub size: u64,
    /// Last access timestamp (Unix seconds)
    pub accessed_at: i64,
    /// Total access count
    pub access_count: u64,
    /// Creation timestamp (Unix seconds)
    pub created_at: i64,
}

/// Trait for eviction policy implementations
pub trait EvictionPolicy: Send + Sync {
    /// Sort candidates by eviction priority (first = most likely to evict)
    fn sort_candidates(&self, candidates: &mut [EvictionCandidate]);

    /// Filter candidates that should be considered for eviction
    /// Default: consider all candidates
    fn filter_candidates(&self, candidates: &[EvictionCandidate]) -> Vec<EvictionCandidate> {
        candidates.to_vec()
    }
}

/// LRU (Least Recently Used) eviction policy
///
/// Evicts objects that haven't been accessed recently.
/// Good for workloads with temporal locality.
#[derive(Debug, Default)]
pub struct LruPolicy;

impl EvictionPolicy for LruPolicy {
    fn sort_candidates(&self, candidates: &mut [EvictionCandidate]) {
        // Sort by accessed_at ascending (oldest first = evict first)
        candidates.sort_by(|a, b| a.accessed_at.cmp(&b.accessed_at));
    }
}

/// LFU (Least Frequently Used) eviction policy
///
/// Evicts objects with the lowest access count.
/// Good for workloads where frequently accessed objects should stay.
#[derive(Debug, Default)]
pub struct LfuPolicy;

impl EvictionPolicy for LfuPolicy {
    fn sort_candidates(&self, candidates: &mut [EvictionCandidate]) {
        // Sort by access_count ascending (lowest first = evict first)
        // Tie-breaker: older objects evicted first
        candidates.sort_by(|a, b| {
            a.access_count
                .cmp(&b.access_count)
                .then_with(|| a.accessed_at.cmp(&b.accessed_at))
        });
    }
}

/// TTL (Time To Live) eviction policy
///
/// Evicts objects older than the configured TTL.
/// Objects that haven't expired are never evicted (unless forced).
#[derive(Debug)]
pub struct TtlPolicy {
    /// TTL in seconds
    ttl_secs: u64,
}

impl TtlPolicy {
    pub fn new(ttl_secs: u64) -> Self {
        Self { ttl_secs }
    }

    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    fn is_expired(&self, created_at: i64) -> bool {
        let now = Self::current_timestamp();
        let age = now - created_at;
        age > self.ttl_secs as i64
    }
}

impl EvictionPolicy for TtlPolicy {
    fn sort_candidates(&self, candidates: &mut [EvictionCandidate]) {
        // Sort by created_at ascending (oldest first = most likely expired)
        candidates.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    }

    fn filter_candidates(&self, candidates: &[EvictionCandidate]) -> Vec<EvictionCandidate> {
        // Only consider expired objects
        candidates
            .iter()
            .filter(|c| self.is_expired(c.created_at))
            .cloned()
            .collect()
    }
}

/// Combined policy that uses TTL as primary filter, then falls back to LRU/LFU
///
/// This provides the best of both worlds:
/// 1. Always evict expired objects first (TTL)
/// 2. If still over limit, use LRU or LFU for non-expired objects
#[derive(Debug)]
#[allow(dead_code)]
pub struct TtlWithFallbackPolicy {
    ttl_secs: u64,
    fallback: FallbackPolicy,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum FallbackPolicy {
    Lru,
    Lfu,
}

#[allow(dead_code)]
impl TtlWithFallbackPolicy {
    pub fn new(ttl_secs: u64, fallback: FallbackPolicy) -> Self {
        Self { ttl_secs, fallback }
    }

    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    fn is_expired(&self, created_at: i64) -> bool {
        let now = Self::current_timestamp();
        let age = now - created_at;
        age > self.ttl_secs as i64
    }
}

impl EvictionPolicy for TtlWithFallbackPolicy {
    fn sort_candidates(&self, candidates: &mut [EvictionCandidate]) {
        // Sort with expired objects first, then by fallback policy
        candidates.sort_by(|a, b| {
            let a_expired = self.is_expired(a.created_at);
            let b_expired = self.is_expired(b.created_at);

            // Expired objects come first
            match (a_expired, b_expired) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (true, true) => {
                    // Both expired: sort by age (oldest first)
                    a.created_at.cmp(&b.created_at)
                }
                (false, false) => {
                    // Neither expired: use fallback policy
                    match self.fallback {
                        FallbackPolicy::Lru => a.accessed_at.cmp(&b.accessed_at),
                        FallbackPolicy::Lfu => a
                            .access_count
                            .cmp(&b.access_count)
                            .then_with(|| a.accessed_at.cmp(&b.accessed_at)),
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candidate(
        id: u8,
        accessed_at: i64,
        access_count: u64,
        created_at: i64,
    ) -> EvictionCandidate {
        EvictionCandidate {
            id: vec![id],
            size: 100,
            accessed_at,
            access_count,
            created_at,
        }
    }

    #[test]
    fn test_lru_policy() {
        let policy = LruPolicy;
        let mut candidates = vec![
            make_candidate(1, 1000, 5, 500),
            make_candidate(2, 500, 10, 400), // Oldest access
            make_candidate(3, 2000, 1, 600), // Newest access
        ];

        policy.sort_candidates(&mut candidates);

        assert_eq!(candidates[0].id, vec![2]); // Oldest access first
        assert_eq!(candidates[1].id, vec![1]);
        assert_eq!(candidates[2].id, vec![3]); // Newest access last
    }

    #[test]
    fn test_lfu_policy() {
        let policy = LfuPolicy;
        let mut candidates = vec![
            make_candidate(1, 1000, 5, 500),
            make_candidate(2, 500, 1, 400),   // Lowest count
            make_candidate(3, 2000, 10, 600), // Highest count
        ];

        policy.sort_candidates(&mut candidates);

        assert_eq!(candidates[0].id, vec![2]); // Lowest count first
        assert_eq!(candidates[1].id, vec![1]);
        assert_eq!(candidates[2].id, vec![3]); // Highest count last
    }

    #[test]
    fn test_ttl_policy_filter() {
        let now = TtlPolicy::current_timestamp();
        let policy = TtlPolicy::new(3600); // 1 hour TTL

        let candidates = vec![
            make_candidate(1, now - 100, 5, now - 100), // Fresh (100s old)
            make_candidate(2, now - 7200, 10, now - 7200), // Expired (2h old)
            make_candidate(3, now - 1800, 1, now - 1800), // Fresh (30m old)
        ];

        let filtered = policy.filter_candidates(&candidates);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, vec![2]); // Only expired object
    }

    #[test]
    fn test_ttl_with_fallback_lru() {
        let now = TtlPolicy::current_timestamp();
        let policy = TtlWithFallbackPolicy::new(3600, FallbackPolicy::Lru);

        let mut candidates = vec![
            make_candidate(1, now - 100, 5, now - 100), // Fresh, recent access
            make_candidate(2, now - 7200, 10, now - 7200), // Expired
            make_candidate(3, now - 500, 1, now - 500), // Fresh, older access
        ];

        policy.sort_candidates(&mut candidates);

        // Expired first, then by access time
        assert_eq!(candidates[0].id, vec![2]); // Expired
        assert_eq!(candidates[1].id, vec![3]); // Fresh, older access
        assert_eq!(candidates[2].id, vec![1]); // Fresh, recent access
    }
}
