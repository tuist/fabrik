use anyhow::{Context, Result};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, Stdio};
use std::sync::Arc;
use tokio::net::UnixListener;
use tracing::info;

use crate::cli::ExecArgs;
use crate::config::FabrikConfig;
use crate::merger::MergedExecConfig;
use crate::storage::FilesystemStorage;
use crate::xcode::proto::cas::casdb_service_server::CasdbServiceServer;
use crate::xcode::proto::keyvalue::key_value_db_server::KeyValueDbServer;
use crate::xcode::{CasService, KeyValueService};

#[tokio::main]
pub async fn run(args: ExecArgs) -> Result<()> {
    // Load config file if specified
    let file_config = if let Some(config_path) = &args.config {
        Some(FabrikConfig::from_file(config_path)?)
    } else {
        None
    };

    // Merge configuration
    let config = MergedExecConfig::merge(&args, file_config);

    info!("Starting Fabrik exec mode");
    info!("  Cache directory: {}", config.cache_dir);

    // Create temporary Unix socket in cache directory
    let socket_path = PathBuf::from(&config.cache_dir).join("fabrik.sock");

    // Remove socket if it already exists
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)
            .context("Failed to remove existing socket")?;
    }

    // Initialize storage
    info!("Initializing storage at {}", config.cache_dir);
    let storage = Arc::new(FilesystemStorage::new(&config.cache_dir)?);

    // Create gRPC services
    let cas_service = CasService::new(storage.clone());
    let keyvalue_service = KeyValueService::new(storage.clone());

    info!("Starting gRPC server on Unix socket: {}", socket_path.display());

    // Create Unix socket listener
    let uds = UnixListener::bind(&socket_path)
        .context("Failed to bind Unix socket")?;

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

    info!("Executing command with cache enabled");

    // Prepare command with environment variables
    let mut cmd = ProcessCommand::new(&args.command[0]);
    cmd.args(&args.command[1..]);

    // Set Xcode cache environment variables
    cmd.env("COMPILATION_CACHE_ENABLE_CACHING", "YES");
    cmd.env("COMPILATION_CACHE_ENABLE_PLUGIN", "YES");
    cmd.env("COMPILATION_CACHE_REMOTE_SERVICE_PATH", socket_path.to_str().unwrap());

    // Execute command
    let status = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute command")?;

    info!("Command completed with status: {}", status);

    // Shutdown server
    server_handle.abort();

    // Clean up socket
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    if !status.success() {
        anyhow::bail!("Command failed with exit code: {}", status.code().unwrap_or(-1));
    }

    Ok(())
}
