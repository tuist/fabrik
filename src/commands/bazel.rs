use crate::bazel::proto::bytestream::byte_stream_server::ByteStreamServer;
use crate::bazel::proto::remote_execution::action_cache_server::ActionCacheServer;
use crate::bazel::proto::remote_execution::capabilities_server::CapabilitiesServer;
use crate::bazel::proto::remote_execution::content_addressable_storage_server::ContentAddressableStorageServer;
use crate::bazel::{BazelActionCacheService, BazelByteStreamService, BazelCapabilitiesService, BazelCasService};
use crate::cli::BazelArgs;
use crate::storage::create_storage;
use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tracing::{debug, info};

/// Run bazel with Fabrik cache
pub async fn run_bazel(args: BazelArgs) -> Result<()> {
    info!("Starting Fabrik Bazel wrapper");

    // Create storage backend
    let cache_dir = args
        .common
        .config_cache_dir
        .unwrap_or_else(|| "/tmp/fabrik-bazel-cache".to_string());
    let storage = Arc::new(create_storage(&cache_dir)?);

    // Determine port (0 = random)
    let port = args.port;
    let bind_addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;

    // Create services
    let action_cache_service = BazelActionCacheService::new(storage.clone());
    let cas_service = BazelCasService::new(storage.clone());
    let bytestream_service = BazelByteStreamService::new(storage.clone());
    let capabilities_service = BazelCapabilitiesService::new();

    // We need to bind the server ourselves to get the actual port
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .context("Failed to bind TCP listener")?;

    let actual_addr = listener.local_addr().context("Failed to get local address")?;
    let cache_url = format!("grpc://{}", actual_addr);

    info!("Bazel Remote Cache bound to: {}", cache_url);

    // Start gRPC server with the bound listener
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);

    let server_handle = tokio::spawn(async move {
        info!("Starting Bazel gRPC server");

        Server::builder()
            .add_service(CapabilitiesServer::new(capabilities_service))
            .add_service(ActionCacheServer::new(action_cache_service))
            .add_service(ContentAddressableStorageServer::new(cas_service))
            .serve_with_incoming_shutdown(incoming, async {
                shutdown_rx.await.ok();
                debug!("Graceful shutdown initiated");
            })
            .await
    });

    // Wait for server to start accepting connections
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    info!("Bazel Remote Cache running at: {}", cache_url);

    // Build bazel command with injected --remote_cache flag
    let mut bazel_cmd = Command::new("bazel");

    // Add all user-provided arguments
    for arg in &args.bazel_args {
        bazel_cmd.arg(arg);
    }

    // Inject --remote_cache flag
    bazel_cmd.arg(format!("--remote_cache={}", cache_url));

    // Set up stdio
    bazel_cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    info!("Executing: bazel {} --remote_cache={}", args.bazel_args.join(" "), cache_url);

    // Run bazel and wait for completion
    let status = bazel_cmd
        .status()
        .await
        .context("Failed to execute bazel")?;

    // Shutdown server
    debug!("Sending shutdown signal to gRPC server");
    let _ = shutdown_tx.send(());

    // Wait for server to shut down with timeout
    match tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        server_handle
    ).await {
        Ok(server_result) => {
            if let Err(e) = server_result {
                debug!("gRPC server task error: {:?}", e);
            }
        }
        Err(_) => {
            debug!("Timeout waiting for gRPC server shutdown");
        }
    }

    info!("Bazel completed with status: {}", status);

    // Exit with same code as bazel
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
