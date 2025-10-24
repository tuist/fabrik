use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Complete Fabrik configuration (loaded from TOML file)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FabrikConfig {
    #[serde(default)]
    pub cache: CacheConfig,

    #[serde(default)]
    pub upstream: Vec<UpstreamConfig>,

    #[serde(default)]
    pub auth: AuthConfig,

    #[serde(default)]
    pub build_systems: BuildSystemsConfig,

    #[serde(default)]
    pub fabrik: FabrikProtocolConfig,

    #[serde(default)]
    pub observability: ObservabilityConfig,

    #[serde(default)]
    pub runtime: RuntimeConfig,
}

/// Local cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache directory path
    pub dir: String,

    /// Maximum cache size (e.g., "5GB", "100GB")
    pub max_size: String,

    /// Eviction policy: lru, lfu, ttl
    #[serde(default = "default_eviction_policy")]
    pub eviction_policy: String,

    /// Default TTL for cached objects
    #[serde(default = "default_ttl")]
    pub default_ttl: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            dir: ".fabrik/cache".to_string(),
            max_size: "5GB".to_string(),
            eviction_policy: default_eviction_policy(),
            default_ttl: default_ttl(),
        }
    }
}

/// Upstream configuration (can be Fabrik instance or storage backend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    /// Upstream URL (https://, s3://, gcs://, etc.)
    pub url: String,

    /// Request timeout
    #[serde(default = "default_upstream_timeout")]
    pub timeout: String,

    /// Read-only upstream (never write)
    #[serde(default)]
    pub read_only: bool,

    /// Permanent storage (never evict)
    #[serde(default)]
    pub permanent: bool,

    /// Write-through (write immediately vs. async)
    #[serde(default = "default_true")]
    pub write_through: bool,

    // S3-specific fields
    #[serde(default)]
    pub region: Option<String>,

    #[serde(default)]
    pub endpoint: Option<String>,

    #[serde(default)]
    pub access_key: Option<String>,

    #[serde(default)]
    pub secret_key: Option<String>,

    #[serde(default = "default_workers")]
    pub workers: u32,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    /// Path to JWT public key file (PEM format)
    pub public_key_file: Option<String>,

    /// Inline JWT public key (PEM format)
    pub public_key: Option<String>,

    /// JWKS endpoint URL
    pub jwks_url: Option<String>,

    /// Key refresh interval
    #[serde(default = "default_key_refresh_interval")]
    pub key_refresh_interval: String,

    /// Require authentication
    #[serde(default = "default_true")]
    pub required: bool,
}

/// Build system adapters configuration (Layer 1 only)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildSystemsConfig {
    /// Enabled build systems
    #[serde(default = "default_build_systems")]
    pub enabled: Vec<String>,

    /// Gradle adapter configuration
    #[serde(default)]
    pub gradle: Option<AdapterConfig>,

    /// Bazel adapter configuration
    #[serde(default)]
    pub bazel: Option<AdapterConfig>,

    /// Nx adapter configuration
    #[serde(default)]
    pub nx: Option<AdapterConfig>,

    /// TurboRepo adapter configuration
    #[serde(default)]
    pub turborepo: Option<AdapterConfig>,

    /// sccache adapter configuration
    #[serde(default)]
    pub sccache: Option<AdapterConfig>,
}

/// Per-adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    /// Bind address (e.g., "0.0.0.0:8080")
    #[serde(default)]
    pub bind: Option<String>,

    /// Port (0 = random)
    #[serde(default)]
    pub port: Option<u16>,

    /// Auto-configure environment variables
    #[serde(default = "default_true")]
    pub auto_configure: bool,
}

/// Fabrik protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabrikProtocolConfig {
    /// Enable Fabrik protocol server (Layer 2)
    #[serde(default)]
    pub enabled: bool,

    /// Bind address for Fabrik gRPC server
    #[serde(default = "default_fabrik_bind")]
    pub bind: String,
}

impl Default for FabrikProtocolConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind: default_fabrik_bind(),
        }
    }
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Log format (json, text)
    #[serde(default = "default_log_format")]
    pub log_format: String,

    /// Health check bind address
    #[serde(default = "default_health_bind")]
    pub health_bind: String,

    /// Enable health API
    #[serde(default = "default_true")]
    pub health_enabled: bool,

    /// API bind address (metrics + cache query + admin)
    #[serde(default = "default_api_bind")]
    pub api_bind: String,

    /// Enable Prometheus metrics API
    #[serde(default = "default_true")]
    pub metrics_enabled: bool,

    /// Enable cache query API (for Tuist Dashboard)
    #[serde(default = "default_true")]
    pub cache_query_api_enabled: bool,

    /// Enable admin API (management operations)
    #[serde(default)]
    pub admin_api_enabled: bool,

    /// Require authentication for APIs
    #[serde(default = "default_true")]
    pub api_auth_required: bool,

    /// JWT public key for API authentication
    pub api_jwt_public_key_file: Option<String>,

    /// Enable tracing
    #[serde(default)]
    pub tracing_enabled: bool,

    /// Tracing endpoint (OpenTelemetry)
    pub tracing_endpoint: Option<String>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            log_format: default_log_format(),
            health_bind: default_health_bind(),
            health_enabled: true,
            api_bind: default_api_bind(),
            metrics_enabled: true,
            cache_query_api_enabled: true,
            admin_api_enabled: false,
            api_auth_required: true,
            api_jwt_public_key_file: None,
            tracing_enabled: false,
            tracing_endpoint: None,
        }
    }
}

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Graceful shutdown timeout
    #[serde(default = "default_graceful_shutdown")]
    pub graceful_shutdown_timeout: String,

    /// Max concurrent requests
    #[serde(default = "default_max_concurrent_requests")]
    pub max_concurrent_requests: u32,

    /// Worker threads (0 = auto)
    #[serde(default)]
    pub worker_threads: u32,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            graceful_shutdown_timeout: default_graceful_shutdown(),
            max_concurrent_requests: default_max_concurrent_requests(),
            worker_threads: 0,
        }
    }
}

// Default value functions
fn default_eviction_policy() -> String {
    "lfu".to_string()
}

fn default_ttl() -> String {
    "7d".to_string()
}

fn default_upstream_timeout() -> String {
    "30s".to_string()
}

fn default_workers() -> u32 {
    10
}

fn default_key_refresh_interval() -> String {
    "5m".to_string()
}

fn default_build_systems() -> Vec<String> {
    vec![
        "gradle".to_string(),
        "bazel".to_string(),
        "nx".to_string(),
        "turborepo".to_string(),
        "sccache".to_string(),
    ]
}

fn default_fabrik_bind() -> String {
    "0.0.0.0:7070".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "json".to_string()
}

fn default_health_bind() -> String {
    "0.0.0.0:8888".to_string()
}

fn default_api_bind() -> String {
    "0.0.0.0:9091".to_string()
}

fn default_graceful_shutdown() -> String {
    "30s".to_string()
}

fn default_max_concurrent_requests() -> u32 {
    10000
}

fn default_true() -> bool {
    true
}

impl FabrikConfig {
    /// Load configuration from TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: FabrikConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.as_ref().display()))?;

        Ok(config)
    }

    /// Generate example configuration as TOML string
    pub fn example_exec() -> String {
        let config = FabrikConfig {
            cache: CacheConfig {
                dir: ".fabrik/cache".to_string(),
                max_size: "5GB".to_string(),
                eviction_policy: "lru".to_string(),
                default_ttl: "7d".to_string(),
            },
            upstream: vec![UpstreamConfig {
                url: "grpc://cache.example.com:7070".to_string(),  // Fabrik protocol
                timeout: "30s".to_string(),
                read_only: false,
                permanent: false,
                write_through: true,
                region: None,
                endpoint: None,
                access_key: None,
                secret_key: None,
                workers: 10,
            }],
            build_systems: BuildSystemsConfig {
                enabled: vec!["gradle".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        toml::to_string_pretty(&config).unwrap()
    }

    pub fn example_server() -> String {
        let config = FabrikConfig {
            cache: CacheConfig {
                dir: "/data/fabrik/cache".to_string(),
                max_size: "100GB".to_string(),
                eviction_policy: "lfu".to_string(),
                default_ttl: "7d".to_string(),
            },
            upstream: vec![UpstreamConfig {
                url: "s3://tuist-build-cache/tenant-example/".to_string(),
                timeout: "60s".to_string(),
                read_only: false,
                permanent: true,
                write_through: true,
                region: Some("us-east-1".to_string()),
                endpoint: None,
                access_key: None,
                secret_key: None,
                workers: 20,
            }],
            auth: AuthConfig {
                public_key_file: Some("/etc/fabrik/jwt-public-key.pem".to_string()),
                public_key: None,
                jwks_url: None,
                key_refresh_interval: "5m".to_string(),
                required: true,
            },
            build_systems: BuildSystemsConfig {
                enabled: vec![],  // Layer 2 doesn't run build system adapters
                ..Default::default()
            },
            fabrik: FabrikProtocolConfig {
                enabled: true,  // Layer 2 runs Fabrik protocol server
                bind: "0.0.0.0:7070".to_string(),
            },
            ..Default::default()
        };

        toml::to_string_pretty(&config).unwrap()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate cache directory is set
        if self.cache.dir.is_empty() {
            anyhow::bail!("cache.dir must be set");
        }

        // Validate cache size format
        if !self.cache.max_size.ends_with("GB")
            && !self.cache.max_size.ends_with("MB")
            && !self.cache.max_size.ends_with("TB") {
            anyhow::bail!("cache.max_size must end with GB, MB, or TB");
        }

        // Validate eviction policy
        if !["lru", "lfu", "ttl"].contains(&self.cache.eviction_policy.as_str()) {
            anyhow::bail!("cache.eviction_policy must be one of: lru, lfu, ttl");
        }

        // Validate upstream URLs
        for upstream in &self.upstream {
            if !upstream.url.starts_with("http://")
                && !upstream.url.starts_with("https://")
                && !upstream.url.starts_with("s3://")
                && !upstream.url.starts_with("gcs://") {
                anyhow::bail!(
                    "upstream.url must start with http://, https://, s3://, or gcs://: {}",
                    upstream.url
                );
            }
        }

        // Validate build systems
        for build_system in &self.build_systems.enabled {
            if !["gradle", "bazel", "nx", "turborepo", "sccache"].contains(&build_system.as_str()) {
                anyhow::bail!(
                    "build_systems.enabled must contain only: gradle, bazel, nx, turborepo, sccache"
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FabrikConfig::default();
        assert_eq!(config.cache.dir, ".fabrik/cache");
        assert_eq!(config.cache.max_size, "5GB");
        assert_eq!(config.cache.eviction_policy, "lfu");
    }

    #[test]
    fn test_validate_config() {
        let config = FabrikConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_eviction_policy() {
        let mut config = FabrikConfig::default();
        config.cache.eviction_policy = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_upstream_url() {
        let mut config = FabrikConfig::default();
        config.upstream.push(UpstreamConfig {
            url: "invalid://url".to_string(),
            timeout: "30s".to_string(),
            read_only: false,
            permanent: false,
            write_through: true,
            region: None,
            endpoint: None,
            access_key: None,
            secret_key: None,
            workers: 10,
        });
        assert!(config.validate().is_err());
    }
}
