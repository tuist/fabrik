use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::watch;
use tonic::transport::Server;
use tracing::info;

use crate::cli::ServerArgs;
use crate::eviction::{spawn_background_eviction, BackgroundEvictionConfig, EvictionConfig};
use crate::fabrik_protocol::proto::fabrik_cache_server::FabrikCacheServer;
use crate::fabrik_protocol::FabrikCacheService;
use crate::hot_reload::HotReloadManager;
use crate::merger::MergedServerConfig;
use crate::storage::FilesystemStorage;
use crate::xcode::proto::cas::casdb_service_server::CasdbServiceServer;
use crate::xcode::proto::keyvalue::key_value_db_server::KeyValueDbServer;
use crate::xcode::{CasService, KeyValueService};

pub async fn run(args: ServerArgs) -> Result<()> {
    use crate::config_discovery::{discover_config, load_config_with_discovery};

    // Load config file with auto-discovery and get the config path
    let file_config = load_config_with_discovery(args.config.as_deref())?;

    // Determine config file path for hot-reload
    let config_path: Option<PathBuf> = if let Some(ref path) = args.config {
        Some(PathBuf::from(path))
    } else {
        // Try to discover config path
        let current_dir = std::env::current_dir()?;
        discover_config(&current_dir)?
    };

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

    // Check server layer configuration
    let is_regional = full_config.is_regional();
    let regional_bind = full_config.regional_bind().to_string();

    if is_regional {
        info!("Server layer: regional (Layer 2) on {}", regional_bind);
    } else {
        info!("Server layer: local (Layer 1)");
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

    // Set up hot-reload manager if we have a config file
    let hot_reload = if let Some(ref path) = config_path {
        match HotReloadManager::new(full_config.clone(), path).await {
            Ok(manager) => {
                info!("Hot-reload enabled for {}", path.display());
                info!("  - File changes will auto-reload config");
                #[cfg(unix)]
                info!("  - Send SIGHUP to manually reload");
                Some(manager)
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to set up hot-reload: {}. Continuing without hot-reload.",
                    e
                );
                None
            }
        }
    } else {
        info!("Hot-reload disabled (no config file)");
        None
    };

    // Create shutdown signal channel
    let (shutdown_tx, _) = watch::channel(false);

    // Track server handles
    let mut handles = vec![];

    // Start regional cache server if layer = "regional" (Layer 2 mode)
    if is_regional {
        let regional_storage = storage.clone();
        let regional_addr = regional_bind
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid server bind address: {}", e))?;
        let mut shutdown_rx = shutdown_tx.subscribe();

        info!("Starting regional cache server on {}", regional_addr);
        info!("  - FabrikCache service (Exists, Get, Put, Delete, GetStats)");

        handles.push(tokio::spawn(async move {
            let fabrik_service = FabrikCacheService::new(regional_storage);

            Server::builder()
                .add_service(FabrikCacheServer::new(fabrik_service))
                .serve_with_shutdown(regional_addr, async move {
                    let _ = shutdown_rx.changed().await;
                })
                .await
                .map_err(|e| anyhow::anyhow!("Regional cache server error: {}", e))
        }));
    }

    // Start Xcode services on the legacy gRPC bind (for backward compatibility)
    // Only start if not running as regional server (to avoid port conflicts)
    if !is_regional {
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

    // Shutdown hot-reload manager
    if let Some(ref manager) = hot_reload {
        info!("Shutting down hot-reload manager...");
        manager.shutdown();
    }

    info!("Server shutdown complete");
    Ok(())
}
