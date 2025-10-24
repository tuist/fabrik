// API types are defined here for future implementation but not yet used
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Health API
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_seconds: u64,
    pub version: String,
}

// ============================================================================
// Cache Query API - Artifacts
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub build_system: Option<String>,
    pub content_type: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub hash: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub last_accessed: String,
    pub access_count: u64,
    pub metadata: ArtifactMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListArtifactsResponse {
    pub artifacts: Vec<Artifact>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Deserialize)]
pub struct ListArtifactsQuery {
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
    pub sort: Option<String>,
}

fn default_limit() -> u32 {
    100
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactDetailResponse {
    pub exists: bool,
    pub hash: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub last_accessed: String,
    pub access_count: u64,
    pub in_local_cache: bool,
    pub in_upstream: bool,
}

#[derive(Debug, Deserialize)]
pub struct SearchArtifactsQuery {
    pub query: String,
    pub build_system: Option<String>,
    pub min_size: Option<String>,
    pub max_size: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

// ============================================================================
// Cache Query API - Statistics
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_objects: u64,
    pub total_size_bytes: u64,
    pub size_by_type: HashMap<String, u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hit_ratio: f64,
    pub latency_p50_ms: u64,
    pub latency_p95_ms: u64,
    pub latency_p99_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BandwidthStats {
    pub upload_bytes_total: u64,
    pub download_bytes_total: u64,
    pub upload_bytes_last_hour: u64,
    pub download_bytes_last_hour: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpstreamStats {
    pub requests_total: u64,
    pub hits: u64,
    pub misses: u64,
    pub errors: u64,
    pub latency_avg_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvictionStats {
    pub total: u64,
    pub last_eviction: Option<String>,
    pub eviction_rate_per_hour: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsResponse {
    pub cache: CacheStats,
    pub performance: PerformanceStats,
    pub bandwidth: BandwidthStats,
    pub upstream: UpstreamStats,
    pub evictions: EvictionStats,
}

#[derive(Debug, Deserialize)]
pub struct TopArtifactsQuery {
    #[serde(default = "default_top_limit")]
    pub limit: u32,
    pub sort: Option<String>,
}

fn default_top_limit() -> u32 {
    50
}

// ============================================================================
// Admin API
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct EvictRequest {
    pub target_size_bytes: u64,
    pub strategy: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EvictResponse {
    pub success: bool,
    pub evicted_count: u64,
    pub evicted_bytes: u64,
    pub current_size_bytes: u64,
}

#[derive(Debug, Deserialize)]
pub struct ClearCacheRequest {
    pub confirm: bool,
}

#[derive(Debug, Serialize)]
pub struct ClearCacheResponse {
    pub success: bool,
    pub cleared_count: u64,
    pub cleared_bytes: u64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub max_cache_size: Option<String>,
    pub eviction_policy: Option<String>,
    pub default_ttl: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateConfigResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct GetConfigResponse {
    pub cache: CacheConfigResponse,
    pub upstream: Vec<UpstreamConfigResponse>,
}

#[derive(Debug, Serialize)]
pub struct CacheConfigResponse {
    pub dir: String,
    pub max_size: String,
    pub eviction_policy: String,
    pub default_ttl: String,
}

#[derive(Debug, Serialize)]
pub struct UpstreamConfigResponse {
    pub url: String,
    pub timeout: String,
    pub permanent: bool,
}

// ============================================================================
// Error Responses
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: u16,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>, code: u16) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            code,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("not_found", message, 404)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new("unauthorized", message, 401)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("bad_request", message, 400)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new("internal_error", message, 500)
    }
}
