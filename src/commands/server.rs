use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::watch;
use tonic::transport::Server;
use tracing::info;

use crate::cli::ServerArgs;
use crate::eviction::{spawn_background_eviction, BackgroundEvictionConfig, EvictionConfig};
use crate::fabrik_protocol::proto::fabrik_cache_server::FabrikCacheServer;
use crate::fabrik_protocol::FabrikCacheService;
use crate::merger::MergedServerConfig;
use crate::storage::FilesystemStorage;
use crate::xcode::proto::cas::casdb_service_server::CasdbServiceServer;
use crate::xcode::proto::keyvalue::key_value_db_server::KeyValueDbServer;
use crate::xcode::{CasService, KeyValueService};

pub async fn run(args: ServerArgs) -> Result<()> {
    use crate::config_discovery::load_config_with_discovery;

    // Load config file with auto-discovery
    let file_config = load_config_with_discovery(args.config.as_deref())?;

    // Keep a copy of the full config for Fabrik protocol settings
    let full_config = file_config.clone().unwrap_or_default();

    // Merge configuration
    let config = MergedServerConfig::merge(&args, file_config);

    info!("Starting server mode");
    info!("Configuration:");
    info!("  Cache directory: {}", config.cache_dir);
    info!("  Max cache size: {}", config.max_cache_size);
    info!("  Eviction policy: {}", config.eviction_policy);
    info!("  Default TTL: {}", config.default_ttl);
    info!("  Upstream: {:?}", config.upstream);
    info!("  gRPC bind: {}", config.grpc_bind);

    // Check Fabrik protocol configuration
    let fabrik_enabled = full_config.fabrik.enabled;
    let fabrik_bind = full_config.fabrik.bind.clone();

    if fabrik_enabled {
        info!("Fabrik protocol: enabled on {}", fabrik_bind);
    } else {
        info!("Fabrik protocol: disabled");
    }

    // Initialize eviction configuration
    let eviction_config = EvictionConfig::from_cache_config(
        &config.max_cache_size,
        &config.eviction_policy,
        &config.default_ttl,
    )?;

    // Initialize filesystem storage with eviction
    info!("Initializing storage at {}", config.cache_dir);
    let storage = Arc::new(FilesystemStorage::with_eviction(
        &config.cache_dir,
        Some(eviction_config.clone()),
    )?);

    // Spawn background eviction task
    let eviction_handle = {
        let bg_config = BackgroundEvictionConfig::from_eviction_config(eviction_config);
        spawn_background_eviction(storage.clone(), bg_config)
    };
    info!("Background eviction task started");

    // Create shutdown signal channel
    let (shutdown_tx, _) = watch::channel(false);

    // Track server handles
    let mut handles = vec![];

    // Start Fabrik protocol server if enabled (Layer 2 mode)
    if fabrik_enabled {
        let fabrik_storage = storage.clone();
        let fabrik_addr = fabrik_bind
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid Fabrik protocol bind address: {}", e))?;
        let mut shutdown_rx = shutdown_tx.subscribe();

        info!("Starting Fabrik protocol server on {}", fabrik_addr);
        info!("  - FabrikCache service (Exists, Get, Put, Delete, GetStats)");

        handles.push(tokio::spawn(async move {
            let fabrik_service = FabrikCacheService::new(fabrik_storage);

            Server::builder()
                .add_service(FabrikCacheServer::new(fabrik_service))
                .serve_with_shutdown(fabrik_addr, async move {
                    let _ = shutdown_rx.changed().await;
                })
                .await
                .map_err(|e| anyhow::anyhow!("Fabrik protocol server error: {}", e))
        }));
    }

    // Start Xcode services on the legacy gRPC bind (for backward compatibility)
    // Only start if Fabrik protocol is disabled (to avoid port conflicts)
    if !fabrik_enabled {
        let xcode_storage = storage.clone();
        let mut shutdown_rx = shutdown_tx.subscribe();

        let addr = config
            .grpc_bind
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid gRPC bind address: {}", e))?;

        info!("Starting Xcode cache server on {}", addr);
        info!("  - CAS (Content-Addressable Storage) service");
        info!("  - KeyValue database service");

        handles.push(tokio::spawn(async move {
            let cas_service = CasService::new(xcode_storage.clone());
            let keyvalue_service = KeyValueService::new(xcode_storage);

            Server::builder()
                .add_service(CasdbServiceServer::new(cas_service))
                .add_service(KeyValueDbServer::new(keyvalue_service))
                .serve_with_shutdown(addr, async move {
                    let _ = shutdown_rx.changed().await;
                })
                .await
                .map_err(|e| anyhow::anyhow!("Xcode gRPC server error: {}", e))
        }));
    }

    info!("Server started - waiting for shutdown signal");

    // Wait for shutdown signal
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

    // Signal all servers to shutdown
    let _ = shutdown_tx.send(true);

    // Wait for all server tasks to complete
    for handle in handles {
        let _ = handle.await;
    }

    // Shutdown background eviction task
    info!("Shutting down background eviction task...");
    eviction_handle.shutdown().await;

    info!("Server shutdown complete");
    Ok(())
}
