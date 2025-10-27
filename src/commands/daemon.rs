use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use crate::cli::DaemonArgs;
use crate::config::FabrikConfig;
use crate::http::HttpServer;
use crate::merger::MergedExecConfig;
use crate::storage;

pub async fn run(args: DaemonArgs) -> Result<()> {
    // Load config file if specified
    let file_config = if let Some(config_path) = &args.config {
        Some(FabrikConfig::from_file(config_path)?)
    } else {
        None
    };

    // Convert DaemonArgs to ExecArgs for merging (they share the same config fields)
    let exec_args = crate::cli::ExecArgs {
        config: args.config,
        config_cache_dir: args.config_cache_dir,
        config_max_cache_size: args.config_max_cache_size,
        config_upstream: args.config_upstream,
        config_upstream_timeout: args.config_upstream_timeout,
        config_jwt_token: args.config_jwt_token,
        config_jwt_token_file: args.config_jwt_token_file,
        config_http_port: args.config_http_port,
        config_grpc_port: args.config_grpc_port,
        config_s3_port: args.config_s3_port,
        config_build_systems: args.config_build_systems,
        config_write_through: args.config_write_through,
        config_read_through: args.config_read_through,
        config_offline: args.config_offline,
        config_log_level: args.config_log_level,
        config_metrics_port: args.config_metrics_port,
        export_env: false,
        env_prefix: String::new(),
        command: vec![],
    };

    let config = MergedExecConfig::merge(&exec_args, file_config);

    info!("Starting Fabrik daemon mode");
    info!("Configuration:");
    info!("  Cache directory: {}", config.cache_dir);
    info!("  Max cache size: {}", config.max_cache_size);
    info!("  Upstream: {:?}", config.upstream);
    info!("  HTTP port: {}", config.http_port);

    // Initialize storage backend
    let storage = storage::create_storage(&config.cache_dir)?;
    let storage = Arc::new(storage);

    // Start HTTP server (for Metro, Gradle, Nx, TurboRepo, etc.)
    let http_server = HttpServer::new(config.http_port, storage.clone());

    info!("Starting HTTP cache server on port {}", config.http_port);

    // Run server (blocks until shutdown signal)
    http_server.run().await?;

    Ok(())
}
