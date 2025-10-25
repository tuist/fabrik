// This module is only available on Unix platforms (macOS, Linux) since it uses UnixListener

use anyhow::{Context, Result};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::process::Command;
use tracing::info;

use crate::cli::XcodebuildArgs;
use crate::config::FabrikConfig;
use crate::storage;
use crate::xcode::proto::cas::casdb_service_server::CasdbServiceServer;
use crate::xcode::proto::keyvalue::key_value_db_server::KeyValueDbServer;
use crate::xcode::{CasService, KeyValueService};

/// Merged configuration for xcodebuild command
#[allow(dead_code)]
struct MergedXcodebuildConfig {
    cache_dir: String,
}

impl MergedXcodebuildConfig {
    fn merge(args: &XcodebuildArgs, file_config: Option<FabrikConfig>) -> Self {
        let default_config = FabrikConfig::default();
        let config = file_config.unwrap_or(default_config);

        Self {
            cache_dir: args
                .common
                .config_cache_dir
                .clone()
                .unwrap_or_else(|| config.cache.dir.clone()),
        }
    }
}

pub async fn run(args: XcodebuildArgs) -> Result<()> {
    // Load config file if specified
    let file_config = if let Some(config_path) = &args.common.config {
        Some(FabrikConfig::from_file(config_path)?)
    } else {
        None
    };

    // Merge configuration
    let config = MergedXcodebuildConfig::merge(&args, file_config);

    info!("Starting Fabrik xcodebuild mode");
    info!("  Cache directory: {}", config.cache_dir);

    // Create temporary Unix socket in cache directory
    let socket_path = PathBuf::from(&config.cache_dir).join("fabrik.sock");

    // Remove socket if it already exists
    if socket_path.exists() {
        std::fs::remove_file(&socket_path).context("Failed to remove existing socket")?;
    }

    // Initialize storage
    info!("Initializing storage...");
    let storage = Arc::new(storage::create_storage(&config.cache_dir)?);

    // Create gRPC services
    let cas_service = CasService::new(storage.clone());
    let keyvalue_service = KeyValueService::new(storage.clone());

    info!(
        "Starting gRPC server on Unix socket: {}",
        socket_path.display()
    );

    // Create Unix socket listener
    let uds = UnixListener::bind(&socket_path).context("Failed to bind Unix socket")?;

    // Set socket permissions (readable/writable by owner)
    std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))?;

    // Spawn server in background task
    let server_handle = tokio::spawn(async move {
        let incoming = tokio_stream::wrappers::UnixListenerStream::new(uds);

        tonic::transport::Server::builder()
            .add_service(CasdbServiceServer::new(cas_service))
            .add_service(KeyValueDbServer::new(keyvalue_service))
            .serve_with_incoming(incoming)
            .await
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    info!("Executing xcodebuild with cache enabled");

    // Prepare xcodebuild command
    let mut cmd = Command::new("xcodebuild");
    cmd.args(&args.xcodebuild_args);

    // Append cache build settings at the end
    cmd.arg("COMPILATION_CACHE_ENABLE_CACHING=YES");
    cmd.arg("COMPILATION_CACHE_ENABLE_PLUGIN=YES");
    cmd.arg(format!(
        "COMPILATION_CACHE_REMOTE_SERVICE_PATH={}",
        socket_path.to_str().unwrap()
    ));

    info!(
        "Running: xcodebuild {} COMPILATION_CACHE_ENABLE_CACHING=YES COMPILATION_CACHE_ENABLE_PLUGIN=YES COMPILATION_CACHE_REMOTE_SERVICE_PATH={}",
        args.xcodebuild_args.join(" "),
        socket_path.display()
    );

    // Execute command and ensure cleanup happens no matter what
    let result = cmd
        .kill_on_drop(true)
        .status()
        .await
        .context("Failed to execute xcodebuild");

    // ALWAYS cleanup server and socket, regardless of success/failure
    info!("Shutting down cache server");
    server_handle.abort();

    // Clean up socket
    if socket_path.exists() {
        if let Err(e) = std::fs::remove_file(&socket_path) {
            // Log but don't fail on socket cleanup error
            tracing::warn!("Failed to remove socket: {}", e);
        }
    }

    // Now check the result and propagate errors
    let status = result?;
    info!("xcodebuild completed with status: {}", status);

    if !status.success() {
        anyhow::bail!(
            "xcodebuild failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}
