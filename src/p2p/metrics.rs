/// P2P metrics tracking
///
/// Tracks P2P cache performance metrics including:
/// - Hit/miss rates
/// - Latency distributions
/// - Bandwidth usage
/// - Peer performance
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// P2P metrics collector
pub struct P2PMetrics {
    // Request counts
    requests_total: Arc<AtomicU64>,
    hits_total: Arc<AtomicU64>,
    misses_total: Arc<AtomicU64>,

    // Latency tracking (in microseconds)
    latency_sum_us: Arc<AtomicU64>,
    latency_count: Arc<AtomicU64>,

    // Bandwidth (in bytes)
    bytes_downloaded: Arc<AtomicU64>,
    bytes_uploaded: Arc<AtomicU64>,

    // Consent tracking
    consent_requests: Arc<AtomicU64>,
    consent_approvals: Arc<AtomicU64>,
    consent_denials: Arc<AtomicU64>,
}

impl P2PMetrics {
    pub fn new() -> Self {
        Self {
            requests_total: Arc::new(AtomicU64::new(0)),
            hits_total: Arc::new(AtomicU64::new(0)),
            misses_total: Arc::new(AtomicU64::new(0)),
            latency_sum_us: Arc::new(AtomicU64::new(0)),
            latency_count: Arc::new(AtomicU64::new(0)),
            bytes_downloaded: Arc::new(AtomicU64::new(0)),
            bytes_uploaded: Arc::new(AtomicU64::new(0)),
            consent_requests: Arc::new(AtomicU64::new(0)),
            consent_approvals: Arc::new(AtomicU64::new(0)),
            consent_denials: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a P2P cache hit
    pub fn record_hit(&self, latency: Duration, bytes: u64) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.hits_total.fetch_add(1, Ordering::Relaxed);
        self.latency_sum_us
            .fetch_add(latency.as_micros() as u64, Ordering::Relaxed);
        self.latency_count.fetch_add(1, Ordering::Relaxed);
        self.bytes_downloaded.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record a P2P cache miss
    pub fn record_miss(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.misses_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record bytes uploaded to a peer
    pub fn record_upload(&self, bytes: u64) {
        self.bytes_uploaded.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record a consent request
    pub fn record_consent_request(&self, approved: bool) {
        self.consent_requests.fetch_add(1, Ordering::Relaxed);
        if approved {
            self.consent_approvals.fetch_add(1, Ordering::Relaxed);
        } else {
            self.consent_denials.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get total number of requests
    pub fn requests_total(&self) -> u64 {
        self.requests_total.load(Ordering::Relaxed)
    }

    /// Get total number of hits
    pub fn hits_total(&self) -> u64 {
        self.hits_total.load(Ordering::Relaxed)
    }

    /// Get total number of misses
    pub fn misses_total(&self) -> u64 {
        self.misses_total.load(Ordering::Relaxed)
    }

    /// Get cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.requests_total();
        if total == 0 {
            0.0
        } else {
            self.hits_total() as f64 / total as f64
        }
    }

    /// Get average latency in milliseconds
    pub fn average_latency_ms(&self) -> f64 {
        let count = self.latency_count.load(Ordering::Relaxed);
        if count == 0 {
            0.0
        } else {
            let sum_us = self.latency_sum_us.load(Ordering::Relaxed);
            (sum_us as f64 / count as f64) / 1000.0 // Convert microseconds to milliseconds
        }
    }

    /// Get total bytes downloaded
    pub fn bytes_downloaded(&self) -> u64 {
        self.bytes_downloaded.load(Ordering::Relaxed)
    }

    /// Get total bytes uploaded
    pub fn bytes_uploaded(&self) -> u64 {
        self.bytes_uploaded.load(Ordering::Relaxed)
    }

    /// Get consent statistics
    pub fn consent_stats(&self) -> ConsentStats {
        ConsentStats {
            requests: self.consent_requests.load(Ordering::Relaxed),
            approvals: self.consent_approvals.load(Ordering::Relaxed),
            denials: self.consent_denials.load(Ordering::Relaxed),
        }
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let hit_rate = self.hit_rate();
        let avg_latency = self.average_latency_ms();
        let consent = self.consent_stats();

        format!(
            r#"# HELP fabrik_p2p_requests_total Total number of P2P cache requests
# TYPE fabrik_p2p_requests_total counter
fabrik_p2p_requests_total {}

# HELP fabrik_p2p_hits_total Total number of P2P cache hits
# TYPE fabrik_p2p_hits_total counter
fabrik_p2p_hits_total {}

# HELP fabrik_p2p_misses_total Total number of P2P cache misses
# TYPE fabrik_p2p_misses_total counter
fabrik_p2p_misses_total {}

# HELP fabrik_p2p_hit_rate P2P cache hit rate (0.0 to 1.0)
# TYPE fabrik_p2p_hit_rate gauge
fabrik_p2p_hit_rate {:.4}

# HELP fabrik_p2p_latency_ms Average P2P latency in milliseconds
# TYPE fabrik_p2p_latency_ms gauge
fabrik_p2p_latency_ms {:.2}

# HELP fabrik_p2p_bytes_downloaded_total Total bytes downloaded from peers
# TYPE fabrik_p2p_bytes_downloaded_total counter
fabrik_p2p_bytes_downloaded_total {}

# HELP fabrik_p2p_bytes_uploaded_total Total bytes uploaded to peers
# TYPE fabrik_p2p_bytes_uploaded_total counter
fabrik_p2p_bytes_uploaded_total {}

# HELP fabrik_p2p_consent_requests_total Total consent requests
# TYPE fabrik_p2p_consent_requests_total counter
fabrik_p2p_consent_requests_total {}

# HELP fabrik_p2p_consent_approvals_total Total consent approvals
# TYPE fabrik_p2p_consent_approvals_total counter
fabrik_p2p_consent_approvals_total {}

# HELP fabrik_p2p_consent_denials_total Total consent denials
# TYPE fabrik_p2p_consent_denials_total counter
fabrik_p2p_consent_denials_total {}
"#,
            self.requests_total(),
            self.hits_total(),
            self.misses_total(),
            hit_rate,
            avg_latency,
            self.bytes_downloaded(),
            self.bytes_uploaded(),
            consent.requests,
            consent.approvals,
            consent.denials,
        )
    }
}

impl Default for P2PMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Consent statistics
#[derive(Debug, Clone)]
pub struct ConsentStats {
    pub requests: u64,
    pub approvals: u64,
    pub denials: u64,
}

impl ConsentStats {
    pub fn approval_rate(&self) -> f64 {
        if self.requests == 0 {
            0.0
        } else {
            self.approvals as f64 / self.requests as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_hit_rate() {
        let metrics = P2PMetrics::new();

        assert_eq!(metrics.hit_rate(), 0.0);

        metrics.record_hit(Duration::from_millis(5), 1024);
        metrics.record_hit(Duration::from_millis(3), 2048);
        metrics.record_miss();

        assert_eq!(metrics.requests_total(), 3);
        assert_eq!(metrics.hits_total(), 2);
        assert_eq!(metrics.misses_total(), 1);
        assert!((metrics.hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_metrics_latency() {
        let metrics = P2PMetrics::new();

        metrics.record_hit(Duration::from_millis(5), 100);
        metrics.record_hit(Duration::from_millis(3), 200);

        let avg = metrics.average_latency_ms();
        assert!((avg - 4.0).abs() < 0.1); // Average should be ~4ms
    }

    #[test]
    fn test_metrics_bandwidth() {
        let metrics = P2PMetrics::new();

        metrics.record_hit(Duration::from_millis(5), 1024);
        metrics.record_upload(2048);

        assert_eq!(metrics.bytes_downloaded(), 1024);
        assert_eq!(metrics.bytes_uploaded(), 2048);
    }

    #[test]
    fn test_consent_stats() {
        let metrics = P2PMetrics::new();

        metrics.record_consent_request(true);
        metrics.record_consent_request(true);
        metrics.record_consent_request(false);

        let stats = metrics.consent_stats();
        assert_eq!(stats.requests, 3);
        assert_eq!(stats.approvals, 2);
        assert_eq!(stats.denials, 1);
        assert!((stats.approval_rate() - 0.666).abs() < 0.01);
    }
}
