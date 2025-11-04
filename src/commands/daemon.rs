use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
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
    use crate::config_discovery::{hash_config, DaemonState};

    // Load config file if specified
    let file_config = if let Some(config_path) = &args.config {
        Some(FabrikConfig::from_file(config_path)?)
    } else {
        None
    };

    // If we have a config file, compute hash for daemon identification
    let daemon_state_info = if let Some(ref config_path_str) = args.config {
        let config_path = std::path::PathBuf::from(config_path_str);
        let config_hash = hash_config(&config_path)?;
        Some((config_hash, config_path))
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

    // Initialize shared storage backend
    let storage = storage::create_storage(&config.cache_dir)?;
    let storage = Arc::new(storage);

    // Start all servers in parallel
    let mut handles = vec![];
    let mut actual_http_port = 0u16;
    let mut actual_grpc_port = 0u16;

    // 1. HTTP server (for Metro, Gradle, Nx, TurboRepo, Xcode)
    // Always use port 0 for daemon mode to avoid conflicts
    if config.http_port != 0 {
        let http_storage = storage.clone();

        // Bind to port 0 to get an available port
        let (http_server, http_port, http_listener) =
            HttpServer::new_with_port_zero(http_storage).await?;

        actual_http_port = http_port;
        info!("HTTP cache server bound to port {}", actual_http_port);

        handles.push(tokio::spawn(async move {
            http_server.run_with_listener(http_listener).await
        }));
    }

    // 2. gRPC server (for Bazel, Fabrik protocol)
    // For gRPC, we bind to port 0 as well
    if config.grpc_port != 0 {
        let grpc_storage = storage.clone();

        // Bind to find an available port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        actual_grpc_port = listener.local_addr()?.port();

        // We need to convert TcpListener to the address for tonic
        // tonic doesn't support pre-bound listeners easily, so we'll use the port
        let addr = format!("127.0.0.1:{}", actual_grpc_port).parse().unwrap();

        // Drop the listener since tonic will bind again
        drop(listener);

        info!("Starting gRPC cache server on port {}", actual_grpc_port);

        handles.push(tokio::spawn(async move {
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

    // Save daemon state with actual bound ports BEFORE starting servers
    let state_opt = if let Some((config_hash, config_path)) = daemon_state_info {
        let state = DaemonState {
            config_hash,
            pid: std::process::id(),
            http_port: actual_http_port,
            grpc_port: actual_grpc_port,
            metrics_port: config.metrics_port,
            unix_socket: None, // TODO: Implement Unix socket server
            config_path,
        };

        if let Err(e) = state.save() {
            tracing::warn!("Failed to save daemon state: {}", e);
            None
        } else {
            info!("Daemon state saved with hash: {}", state.config_hash);
            info!("  HTTP port: {}", state.http_port);
            info!("  gRPC port: {}", state.grpc_port);
            Some(state)
        }
    } else {
        None
    };

    info!("Daemon started - waiting for shutdown signal");

    // Wait for shutdown signal (Ctrl+C or SIGTERM)
    #[cfg(unix)]
    {
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down gracefully...");
            }
            _ = async {
                use tokio::signal::unix::{signal, SignalKind};
                let mut sigterm = signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler");
                sigterm.recv().await
            } => {
                info!("Received SIGTERM, shutting down gracefully...");
            }
        }
    }

    #[cfg(not(unix))]
    {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        info!("Received Ctrl+C, shutting down gracefully...");
    }

    // Gracefully wait for all servers to finish with a timeout
    info!("Waiting for servers to shutdown...");
    let shutdown_timeout = tokio::time::Duration::from_secs(5);

    for handle in handles {
        match tokio::time::timeout(shutdown_timeout, handle).await {
            Ok(Ok(Ok(()))) => {}
            Ok(Ok(Err(e))) => {
                tracing::warn!("Server shutdown error: {}", e);
            }
            Ok(Err(e)) => {
                tracing::warn!("Server task error: {}", e);
            }
            Err(_) => {
                tracing::warn!("Server shutdown timeout");
            }
        }
    }

    // Cleanup daemon state
    if let Some(state) = state_opt {
        if let Err(e) = state.cleanup() {
            tracing::warn!("Failed to cleanup daemon state: {}", e);
        } else {
            info!("Daemon state cleaned up");
        }
    }

    info!("Daemon stopped");
    Ok(())
}
