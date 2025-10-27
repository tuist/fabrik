use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use crate::bazel::proto::bytestream::byte_stream_server::ByteStreamServer;
use crate::bazel::proto::remote_execution::action_cache_server::ActionCacheServer;
use crate::bazel::proto::remote_execution::capabilities_server::CapabilitiesServer;
use crate::bazel::proto::remote_execution::content_addressable_storage_server::ContentAddressableStorageServer;
use crate::bazel::{
    BazelActionCacheService, BazelByteStreamService, BazelCapabilitiesService, BazelCasService,
};
use crate::cli::DaemonArgs;
use crate::config::FabrikConfig;
use crate::http::HttpServer;
use crate::merger::MergedExecConfig;
use crate::storage;
use tonic::transport::Server;

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
    info!("  gRPC port: {}", config.grpc_port);
    info!("  S3 port: {}", config.s3_port);

    // Initialize shared storage backend
    let storage = storage::create_storage(&config.cache_dir)?;
    let storage = Arc::new(storage);

    // Start all servers in parallel
    let mut handles = vec![];

    // 1. HTTP server (for Metro, Gradle, Nx, TurboRepo)
    if config.http_port > 0 {
        info!("Starting HTTP cache server on port {}", config.http_port);
        let http_storage = storage.clone();
        let http_port = config.http_port;
        handles.push(tokio::spawn(async move {
            HttpServer::new(http_port, http_storage).run().await
        }));
    }

    // 2. gRPC server (for Bazel, Fabrik protocol)
    if config.grpc_port > 0 {
        info!("Starting gRPC cache server on port {}", config.grpc_port);
        let grpc_storage = storage.clone();
        let grpc_port = config.grpc_port;

        handles.push(tokio::spawn(async move {
            let addr = format!("0.0.0.0:{}", grpc_port).parse().unwrap();

            // Create Bazel gRPC services
            let action_cache = BazelActionCacheService::new(grpc_storage.clone());
            let cas = BazelCasService::new(grpc_storage.clone());
            let bytestream = BazelByteStreamService::new(grpc_storage.clone());
            let capabilities = BazelCapabilitiesService::new();

            info!("gRPC server listening on {}", addr);

            Server::builder()
                .add_service(CapabilitiesServer::new(capabilities))
                .add_service(ActionCacheServer::new(action_cache))
                .add_service(ContentAddressableStorageServer::new(cas))
                .add_service(ByteStreamServer::new(bytestream))
                .serve(addr)
                .await
                .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
        }));
    }

    // 3. S3-compatible server (for sccache, BuildKit) - TODO: implement
    // if config.s3_port > 0 {
    //     info!("Starting S3-compatible server on port {}", config.s3_port);
    //     // TODO: Implement S3 protocol server
    // }

    // 4. Gradle-specific HTTP server (different endpoints than generic HTTP)
    // Gradle uses /cache/ prefix, so we might need a separate router
    // For now, Gradle can use the generic HTTP server

    info!("Daemon started - press Ctrl+C to stop");

    // Wait for all servers (runs until ctrl-c or error)
    for handle in handles {
        handle.await??;
    }

    Ok(())
}
