use crate::cli::NxArgs;
use crate::http::HttpServer;
use crate::storage::{create_storage, default_cache_dir};
use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tracing::{debug, info};

/// Run nx with Fabrik cache
pub async fn run_nx(args: NxArgs) -> Result<()> {
    info!("Starting Fabrik Nx wrapper");

    // Create storage backend using XDG-compliant default directory
    let cache_dir = args
        .common
        .config_cache_dir
        .unwrap_or_else(|| default_cache_dir().display().to_string());
    let storage = Arc::new(create_storage(&cache_dir)?);

    // Determine port (0 = random)
    let port = args.port;
    let bind_addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;

    // Bind TCP listener to get actual port
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .context("Failed to bind TCP listener")?;

    let actual_addr = listener
        .local_addr()
        .context("Failed to get local address")?;
    let cache_url = format!("http://{}", actual_addr);

    info!("Nx Remote Cache bound to: {}", cache_url);

    // Create shared HTTP server router (supports /v1/cache/{hash} for Nx)
    let http_server = HttpServer::new(actual_addr.port(), storage);
    let app = http_server.router();

    // Start HTTP server with the bound listener
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_handle = tokio::spawn(async move {
        info!("Starting Nx HTTP server");

        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
                debug!("Graceful shutdown initiated");
            })
            .await
    });

    // Wait for server to start accepting connections
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    info!("Nx Remote Cache running at: {}", cache_url);

    // Build nx command
    // Nx is typically run via npx or a local nx binary
    let nx_command = if cfg!(target_os = "windows") {
        "npx.cmd"
    } else {
        "npx"
    };

    let mut nx_cmd = Command::new(nx_command);
    nx_cmd.arg("nx");

    // Add all user-provided arguments
    for arg in &args.nx_args {
        nx_cmd.arg(arg);
    }


    // Set up stdio
    nx_cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    info!(
        "Executing: {} nx {} (via nx.json remote cache={})",
        nx_command,
        args.nx_args.join(" "),
        cache_url
    );

    // Run nx and wait for completion
    let status = nx_cmd.status().await.context("Failed to execute nx")?;

    // Shutdown server
    debug!("Sending shutdown signal to HTTP server");
    let _ = shutdown_tx.send(());

    // Wait for server to shut down with timeout
    match tokio::time::timeout(tokio::time::Duration::from_secs(5), server_handle).await {
        Ok(server_result) => {
            if let Err(e) = server_result {
                debug!("HTTP server task error: {:?}", e);
            }
        }
        Err(_) => {
            debug!("Timeout waiting for HTTP server shutdown");
        }
    }

    info!("Nx completed with status: {}", status);

    // Exit with same code as nx
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
