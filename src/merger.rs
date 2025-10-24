/// Configuration merger: CLI args > Env vars > Config file > Defaults
///
/// This module handles merging configuration from multiple sources:
/// 1. CLI arguments (highest priority)
/// 2. Environment variables
/// 3. Configuration file
/// 4. Built-in defaults (lowest priority)
use crate::cli::{ExecArgs, ServerArgs};
use crate::config::FabrikConfig;

/// Merged configuration for exec/daemon commands
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields will be used when implementing actual functionality
pub struct MergedExecConfig {
    pub cache_dir: String,
    pub max_cache_size: String,
    pub upstream: Vec<String>,
    pub upstream_timeout: String,
    pub jwt_token: Option<String>,
    pub http_port: u16,
    pub grpc_port: u16,
    pub s3_port: u16,
    pub build_systems: Vec<String>,
    pub write_through: bool,
    pub read_through: bool,
    pub offline: bool,
    pub log_level: String,
    pub metrics_port: u16,
}

/// Merged configuration for server command
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields will be used when implementing actual functionality
pub struct MergedServerConfig {
    pub cache_dir: String,
    pub max_cache_size: String,
    pub upstream: Vec<String>,
    pub s3_region: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
    pub http_bind: String,
    pub grpc_bind: String,
    pub s3_bind: String,
    pub fabrik_bind: String,
    pub jwt_public_key_file: Option<String>,
    pub jwt_public_key: Option<String>,
    pub jwt_jwks_url: Option<String>,
    pub jwt_key_refresh: String,
    pub jwt_required: bool,
    pub eviction_policy: String,
    pub default_ttl: String,
    pub write_through: bool,
    pub upstream_workers: u32,
    pub log_level: String,
    pub log_format: String,
    pub health_bind: String,
    pub health_enabled: bool,
    pub api_bind: String,
    pub metrics_enabled: bool,
    pub cache_query_api_enabled: bool,
    pub admin_api_enabled: bool,
    pub api_auth_required: bool,
    pub api_jwt_public_key_file: Option<String>,
    pub tracing_enabled: bool,
    pub tracing_endpoint: Option<String>,
    pub graceful_shutdown: String,
}

impl MergedExecConfig {
    /// Merge configuration from CLI args and config file
    /// Precedence: CLI > env (already handled by clap) > config file > defaults
    pub fn merge(args: &ExecArgs, file_config: Option<FabrikConfig>) -> Self {
        let file = file_config.unwrap_or_default();

        Self {
            cache_dir: args
                .config_cache_dir
                .clone()
                .unwrap_or_else(|| file.cache.dir.clone()),
            max_cache_size: args
                .config_max_cache_size
                .clone()
                .unwrap_or_else(|| file.cache.max_size.clone()),
            upstream: args
                .config_upstream
                .clone()
                .unwrap_or_else(|| file.upstream.iter().map(|u| u.url.clone()).collect()),
            upstream_timeout: args
                .config_upstream_timeout
                .clone()
                .unwrap_or_else(|| "30s".to_string()),
            jwt_token: args.config_jwt_token.clone().or_else(|| {
                args.config_jwt_token_file
                    .as_ref()
                    .and_then(|path| std::fs::read_to_string(path).ok())
            }),
            http_port: args.config_http_port.unwrap_or(0),
            grpc_port: args.config_grpc_port.unwrap_or(0),
            s3_port: args.config_s3_port.unwrap_or(0),
            build_systems: args
                .config_build_systems
                .clone()
                .unwrap_or_else(|| file.build_systems.enabled.clone()),
            write_through: args.config_write_through,
            read_through: args.config_read_through,
            offline: args.config_offline,
            log_level: args
                .config_log_level
                .clone()
                .unwrap_or_else(|| file.observability.log_level.clone()),
            metrics_port: args.config_metrics_port.unwrap_or(0),
        }
    }
}

impl MergedServerConfig {
    /// Merge configuration from CLI args and config file
    /// Precedence: CLI > env (already handled by clap) > config file > defaults
    pub fn merge(args: &ServerArgs, file_config: Option<FabrikConfig>) -> Self {
        let file = file_config.unwrap_or_default();

        // Check AWS env vars as fallback for S3 credentials
        let s3_access_key = args
            .config_s3_access_key
            .clone()
            .or_else(|| std::env::var("AWS_ACCESS_KEY_ID").ok());

        let s3_secret_key = args
            .config_s3_secret_key
            .clone()
            .or_else(|| std::env::var("AWS_SECRET_ACCESS_KEY").ok());

        let s3_region = args
            .config_s3_region
            .clone()
            .or_else(|| std::env::var("AWS_REGION").ok());

        Self {
            cache_dir: args
                .config_cache_dir
                .clone()
                .unwrap_or_else(|| file.cache.dir.clone()),
            max_cache_size: args
                .config_max_cache_size
                .clone()
                .unwrap_or_else(|| file.cache.max_size.clone()),
            upstream: args
                .config_upstream
                .clone()
                .unwrap_or_else(|| file.upstream.iter().map(|u| u.url.clone()).collect()),
            s3_region,
            s3_endpoint: args.config_s3_endpoint.clone(),
            s3_access_key,
            s3_secret_key,
            http_bind: args.config_http_bind.clone(),
            grpc_bind: args.config_grpc_bind.clone(),
            s3_bind: args.config_s3_bind.clone(),
            fabrik_bind: args.config_fabrik_bind.clone(),
            jwt_public_key_file: args.config_jwt_public_key_file.clone(),
            jwt_public_key: args.config_jwt_public_key.clone(),
            jwt_jwks_url: args.config_jwt_jwks_url.clone(),
            jwt_key_refresh: args
                .config_jwt_key_refresh
                .clone()
                .unwrap_or_else(|| file.auth.key_refresh_interval.clone()),
            jwt_required: args.config_jwt_required.unwrap_or(file.auth.required),
            eviction_policy: args
                .config_eviction_policy
                .clone()
                .unwrap_or_else(|| file.cache.eviction_policy.clone()),
            default_ttl: args
                .config_default_ttl
                .clone()
                .unwrap_or_else(|| file.cache.default_ttl.clone()),
            write_through: args.config_write_through,
            upstream_workers: args.config_upstream_workers.unwrap_or(10),
            log_level: args
                .config_log_level
                .clone()
                .unwrap_or_else(|| file.observability.log_level.clone()),
            log_format: args
                .config_log_format
                .clone()
                .unwrap_or_else(|| file.observability.log_format.clone()),
            health_bind: args
                .config_health_bind
                .clone()
                .unwrap_or_else(|| file.observability.health_bind.clone()),
            health_enabled: args
                .config_health_enabled
                .unwrap_or(file.observability.health_enabled),
            api_bind: args
                .config_api_bind
                .clone()
                .unwrap_or_else(|| file.observability.api_bind.clone()),
            metrics_enabled: args
                .config_metrics_enabled
                .unwrap_or(file.observability.metrics_enabled),
            cache_query_api_enabled: args
                .config_cache_query_api_enabled
                .unwrap_or(file.observability.cache_query_api_enabled),
            admin_api_enabled: args
                .config_admin_api_enabled
                .unwrap_or(file.observability.admin_api_enabled),
            api_auth_required: args
                .config_api_auth_required
                .unwrap_or(file.observability.api_auth_required),
            api_jwt_public_key_file: args
                .config_api_jwt_public_key_file
                .clone()
                .or_else(|| file.observability.api_jwt_public_key_file.clone()),
            tracing_enabled: args
                .config_tracing_enabled
                .unwrap_or(file.observability.tracing_enabled),
            tracing_endpoint: args
                .config_tracing_endpoint
                .clone()
                .or_else(|| file.observability.tracing_endpoint.clone()),
            graceful_shutdown: args
                .config_graceful_shutdown
                .clone()
                .unwrap_or_else(|| file.runtime.graceful_shutdown_timeout.clone()),
        }
    }
}
