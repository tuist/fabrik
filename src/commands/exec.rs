use anyhow::{Context, Result};
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tracing::info;

use crate::bazel::proto::bytestream::byte_stream_server::ByteStreamServer;
use crate::bazel::proto::remote_execution::action_cache_server::ActionCacheServer;
use crate::bazel::proto::remote_execution::capabilities_server::CapabilitiesServer;
use crate::bazel::proto::remote_execution::content_addressable_storage_server::ContentAddressableStorageServer;
use crate::bazel::{
    BazelActionCacheService, BazelByteStreamService, BazelCapabilitiesService, BazelCasService,
};
use crate::cli::ExecArgs;
use crate::config::FabrikConfig;
use crate::config_discovery::{default_turbo_team, generate_turbo_token};
use crate::http::HttpServer;
use crate::merger::MergedExecConfig;
use crate::storage;
use tonic::transport::Server;

pub async fn run(args: ExecArgs) -> Result<()> {
    if args.command.is_empty() {
        anyhow::bail!("No command specified. Usage: fabrik exec -- <command>");
    }

    // Load config file if specified
    let file_config = if let Some(config_path) = &args.config {
        Some(FabrikConfig::from_file(config_path)?)
    } else {
        None
    };

    // Merge configuration
    let config = MergedExecConfig::merge(&args, file_config);

    info!("Starting Fabrik exec mode");
    info!("Configuration:");
    info!("  Cache directory: {}", config.cache_dir);
    info!("  Max cache size: {}", config.max_cache_size);
    info!("  Upstream: {:?}", config.upstream);

    // Initialize shared storage backend
    let storage = storage::create_storage(&config.cache_dir)?;
    let storage = Arc::new(storage);

    // Start HTTP server (for Metro, Gradle, Nx, TurboRepo)
    let http_storage = storage.clone();
    let (http_server, http_port, http_listener) =
        HttpServer::new_with_port_zero(http_storage).await?;

    info!("HTTP cache server bound to port {}", http_port);

    let http_handle =
        tokio::spawn(async move { http_server.run_with_listener(http_listener).await });

    // Start gRPC server (for Bazel)
    let grpc_storage = storage.clone();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let grpc_port = listener.local_addr()?.port();
    let addr: std::net::SocketAddr = format!("127.0.0.1:{}", grpc_port).parse().unwrap();
    drop(listener);

    info!("Starting gRPC cache server on port {}", grpc_port);

    let grpc_handle = tokio::spawn(async move {
        let action_cache = BazelActionCacheService::new(grpc_storage.clone());
        let cas = BazelCasService::new(grpc_storage.clone());
        let bytestream = BazelByteStreamService::new(grpc_storage.clone());
        let capabilities = BazelCapabilitiesService::new();

        info!("gRPC server listening on 127.0.0.1:{}", addr.port());

        Server::builder()
            .add_service(ActionCacheServer::new(action_cache))
            .add_service(ContentAddressableStorageServer::new(cas))
            .add_service(ByteStreamServer::new(bytestream))
            .add_service(CapabilitiesServer::new(capabilities))
            .serve(addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    });

    // Build environment variables
    let mut env_vars = std::collections::HashMap::new();

    if args.export_env {
        let prefix = &args.env_prefix;
        env_vars.insert(
            format!("{}HTTP_URL", prefix),
            format!("http://127.0.0.1:{}", http_port),
        );
        env_vars.insert(
            format!("{}GRPC_URL", prefix),
            format!("grpc://127.0.0.1:{}", grpc_port),
        );

        // TurboRepo-specific env vars
        env_vars.insert(
            "TURBO_API".to_string(),
            format!("http://127.0.0.1:{}", http_port),
        );

        // Auto-generate TURBO_TEAM if not already set
        // TurboRepo requires a team to enable remote caching
        if std::env::var("TURBO_TEAM").is_err() {
            env_vars.insert("TURBO_TEAM".to_string(), default_turbo_team().to_string());
            info!("Auto-generated TURBO_TEAM for local development");
        }

        // Auto-generate TURBO_TOKEN if not already set
        // This enables remote caching without requiring manual token configuration
        if std::env::var("TURBO_TOKEN").is_err() {
            env_vars.insert("TURBO_TOKEN".to_string(), generate_turbo_token());
            info!("Auto-generated TURBO_TOKEN for local development");
        }

        info!("Exported environment variables:");
        for (key, value) in &env_vars {
            info!("  {}={}", key, value);
        }
    }

    // Execute user command
    info!("Executing command: {}", args.command.join(" "));

    let mut cmd = Command::new(&args.command[0]);
    cmd.args(&args.command[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Set environment variables
    for (key, value) in &env_vars {
        cmd.env(key, value);
    }

    let status = cmd
        .status()
        .await
        .with_context(|| format!("Failed to execute command: {}", args.command[0]))?;

    info!("Command completed with status: {}", status);

    // Shutdown servers
    info!("Shutting down cache servers...");
    http_handle.abort();
    grpc_handle.abort();

    // Give them a moment to cleanup
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
