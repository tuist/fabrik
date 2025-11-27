use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use crate::cli::ServerArgs;
use crate::eviction::EvictionConfig;
use crate::merger::MergedServerConfig;
use crate::storage::FilesystemStorage;
use crate::xcode::proto::cas::casdb_service_server::CasdbServiceServer;
use crate::xcode::proto::keyvalue::key_value_db_server::KeyValueDbServer;
use crate::xcode::{CasService, KeyValueService};

pub async fn run(args: ServerArgs) -> Result<()> {
    use crate::config_discovery::load_config_with_discovery;

    // Load config file with auto-discovery
    let file_config = load_config_with_discovery(args.config.as_deref())?;

    // Merge configuration
    let config = MergedServerConfig::merge(&args, file_config);

    info!("[fabrik] Starting server mode");
    info!("[fabrik] Configuration:");
    info!("[fabrik]   Cache directory: {}", config.cache_dir);
    info!("[fabrik]   Max cache size: {}", config.max_cache_size);
    info!("[fabrik]   Eviction policy: {}", config.eviction_policy);
    info!("[fabrik]   Default TTL: {}", config.default_ttl);
    info!("[fabrik]   Upstream: {:?}", config.upstream);
    info!("[fabrik]   gRPC bind: {}", config.grpc_bind);

    // Initialize eviction configuration
    let eviction_config = EvictionConfig::from_cache_config(
        &config.max_cache_size,
        &config.eviction_policy,
        &config.default_ttl,
    )?;

    // Initialize filesystem storage with eviction
    info!("[fabrik] Initializing storage at {}", config.cache_dir);
    let storage = Arc::new(FilesystemStorage::with_eviction(
        &config.cache_dir,
        Some(eviction_config),
    )?);

    // Create gRPC services
    let cas_service = CasService::new(storage.clone());
    let keyvalue_service = KeyValueService::new(storage.clone());

    // Parse gRPC bind address
    let addr = config
        .grpc_bind
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid gRPC bind address: {}", e))?;

    info!("Starting Xcode cache server on {}", addr);
    info!("  - CAS (Content-Addressable Storage) service");
    info!("  - KeyValue database service");

    // Start gRPC server
    tonic::transport::Server::builder()
        .add_service(CasdbServiceServer::new(cas_service))
        .add_service(KeyValueDbServer::new(keyvalue_service))
        .serve(addr)
        .await?;

    info!("Server shutdown complete");
    Ok(())
}
