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
use crate::eviction::EvictionConfig;
use crate::http::HttpServer;
use crate::merger::MergedExecConfig;
use crate::storage;
use tonic::transport::Server;

pub async fn run(args: DaemonArgs) -> Result<()> {
    use crate::config_discovery::{discover_config, hash_config, DaemonState};

    // Load config file with auto-discovery and track the path for daemon state
    let (file_config, config_path_opt) = if let Some(config_path_str) = &args.config {
        // Explicit path provided
        let path = std::path::PathBuf::from(config_path_str);
        (Some(FabrikConfig::from_file(&path)?), Some(path))
    } else {
        // Auto-discover by traversing up directory tree
        let current_dir = std::env::current_dir()?;
        if let Some(discovered_path) = discover_config(&current_dir)? {
            (
                Some(FabrikConfig::from_file(&discovered_path)?),
                Some(discovered_path),
            )
        } else {
            (None, None)
        }
    };

    // If we have a config file, compute hash for daemon identification
    let daemon_state_info = if let Some(ref config_path) = config_path_opt {
        let config_hash = hash_config(config_path)?;
        Some((config_hash, config_path.clone()))
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

    let config = MergedExecConfig::merge(&exec_args, file_config.clone());

    // Check if Unix socket is configured (for Xcode)
    let socket_path = file_config.as_ref().and_then(|fc| fc.daemon.socket.clone());

    info!("[fabrik] Starting daemon mode");
    info!("[fabrik] Configuration:");
    info!("[fabrik]   Cache directory: {}", config.cache_dir);
    info!("[fabrik]   Max cache size: {}", config.max_cache_size);
    info!("[fabrik]   Eviction policy: {}", config.eviction_policy);
    info!("[fabrik]   Default TTL: {}", config.default_ttl);
    info!("[fabrik]   Upstream: {:?}", config.upstream);

    if let Some(ref socket) = socket_path {
        info!("[fabrik]   Mode: Unix socket (Xcode)");
        info!("[fabrik]   Socket path: {}", socket);
    } else {
        info!("[fabrik]   Mode: TCP (HTTP + gRPC)");
    }

    // Initialize eviction configuration from merged config
    let eviction_config = EvictionConfig::from_cache_config(
        &config.max_cache_size,
        &config.eviction_policy,
        &config.default_ttl,
    )?;

    // Initialize shared storage backend with eviction
    let storage = storage::create_storage_with_eviction(&config.cache_dir, eviction_config)?;
    let storage = Arc::new(storage);

    // Initialize P2P manager if enabled
    let p2p_manager = if let Some(ref fc) = file_config {
        if fc.p2p.enabled {
            info!("[fabrik] P2P cache sharing is enabled");
            let p2p = crate::p2p::P2PManager::new(fc.p2p.clone()).await?;
            p2p.start().await?;
            info!("[fabrik] P2P services started successfully");
            Some(Arc::new(p2p))
        } else {
            None
        }
    } else {
        None
    };

    // Start servers based on mode
    let mut handles = vec![];
    let mut actual_http_port = 0u16;
    let mut actual_grpc_port = 0u16;
    let mut actual_socket_path: Option<std::path::PathBuf> = None;

    // Check if we should use Unix socket mode (for Xcode)
    #[cfg(unix)]
    if let Some(ref socket_path_str) = socket_path {
        // Unix socket mode: Create ONLY Unix socket gRPC server
        use crate::xcode::proto::cas::casdb_service_server::CasdbServiceServer;
        use crate::xcode::proto::keyvalue::key_value_db_server::KeyValueDbServer;
        use crate::xcode::{CasService, KeyValueService};

        // Resolve relative path to absolute (relative to config file directory)
        let socket_path = if let Some((_, ref config_path)) = daemon_state_info {
            let config_dir = config_path.parent().unwrap_or(std::path::Path::new("."));
            config_dir.join(socket_path_str)
        } else {
            std::path::PathBuf::from(socket_path_str)
        };

        // Remove stale socket file if it exists
        if socket_path.exists() {
            info!("Removing stale socket file: {}", socket_path.display());
            std::fs::remove_file(&socket_path)?;
        }

        info!(
            "Creating Unix socket server for Xcode at: {}",
            socket_path.display()
        );

        // Create Unix socket listener
        let unix_listener = tokio::net::UnixListener::bind(&socket_path)?;
        actual_socket_path = Some(socket_path.clone());

        // Create Xcode gRPC services
        let cas_service = CasService::new(storage.clone());
        let keyvalue_service = KeyValueService::new(storage.clone());

        info!("Unix socket server listening on {}", socket_path.display());

        // Start Unix socket gRPC server
        handles.push(tokio::spawn(async move {
            use tokio_stream::wrappers::UnixListenerStream;

            Server::builder()
                .add_service(CasdbServiceServer::new(cas_service))
                .add_service(KeyValueDbServer::new(keyvalue_service))
                .serve_with_incoming(UnixListenerStream::new(unix_listener))
                .await
                .map_err(|e| anyhow::anyhow!("Unix socket gRPC server error: {}", e))
        }));

        info!("Daemon running in Unix socket mode (Xcode)");
    }

    #[cfg(not(unix))]
    if socket_path.is_some() {
        anyhow::bail!(
            "Unix sockets are not supported on Windows. Remove [daemon] socket from config."
        );
    }

    #[cfg(not(unix))]
    let socket_configured = false;

    #[cfg(unix)]
    let socket_configured = socket_path.is_some();

    if !socket_configured {
        // TCP mode: Create HTTP + gRPC servers

        // 1. HTTP server (for Metro, Gradle, Nx, TurboRepo)
        // Always start HTTP server in TCP mode
        {
            let http_storage = storage.clone();

            // Bind to port 0 to get an available port (or use config port if specified)
            let (http_server, http_port, http_listener) =
                HttpServer::new_with_port_zero(http_storage).await?;

            actual_http_port = http_port;
            info!("HTTP cache server bound to port {}", actual_http_port);

            handles.push(tokio::spawn(async move {
                http_server.run_with_listener(http_listener).await
            }));
        }

        // 2. gRPC server (for Bazel, Fabrik protocol)
        // Always start gRPC server in daemon mode
        {
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
    } // End of TCP mode

    // Save daemon state with actual bound ports/socket BEFORE starting servers
    let state_opt = if let Some((config_hash, config_path)) = daemon_state_info {
        let state = DaemonState {
            config_hash,
            pid: std::process::id(),
            http_port: actual_http_port,
            grpc_port: actual_grpc_port,
            metrics_port: config.metrics_port,
            unix_socket: actual_socket_path,
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

    // Shutdown P2P services first
    if let Some(p2p) = p2p_manager {
        if let Err(e) = p2p.shutdown().await {
            tracing::warn!("Failed to shutdown P2P services: {}", e);
        }
    }

    // Abort all server tasks immediately
    // Note: In the future, we should implement graceful shutdown for axum and tonic servers
    info!("Shutting down servers...");
    for handle in handles {
        handle.abort();
    }

    // Give them a moment to cleanup
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Cleanup daemon state
    if let Some(state) = state_opt {
        // Remove Unix socket file if it exists
        if let Some(ref socket_path) = state.unix_socket {
            if socket_path.exists() {
                if let Err(e) = std::fs::remove_file(socket_path) {
                    tracing::warn!("Failed to remove socket file: {}", e);
                } else {
                    info!("Removed Unix socket: {}", socket_path.display());
                }
            }
        }

        if let Err(e) = state.cleanup() {
            tracing::warn!("Failed to cleanup daemon state: {}", e);
        } else {
            info!("Daemon state cleaned up");
        }
    }

    info!("Daemon stopped");
    Ok(())
}
