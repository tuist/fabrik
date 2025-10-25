use crate::cli::GradleArgs;
use crate::gradle::GradleHttpServer;
use crate::storage::{create_storage, default_cache_dir};
use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tracing::{debug, info};

/// Run gradle with Fabrik cache
pub async fn run_gradle(args: GradleArgs) -> Result<()> {
    info!("Starting Fabrik Gradle wrapper");

    // Create storage backend using XDG-compliant default directory
    let cache_dir = args
        .common
        .config_cache_dir
        .unwrap_or_else(|| default_cache_dir().display().to_string());
    let storage = Arc::new(create_storage(&cache_dir)?);

    // Determine port (0 = random)
    let port = args.port;
    let bind_addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;

    // Create Gradle HTTP server
    let gradle_server = GradleHttpServer::new(storage.clone());
    let app = gradle_server.router();

    // Bind the HTTP server ourselves to get the actual port
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .context("Failed to bind TCP listener")?;

    let actual_addr = listener
        .local_addr()
        .context("Failed to get local address")?;
    let cache_url = format!("http://{}/cache/", actual_addr);

    info!("Gradle Remote Cache bound to: {}", cache_url);

    // Start HTTP server with the bound listener
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_handle = tokio::spawn(async move {
        info!("Starting Gradle HTTP server");

        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
                debug!("Graceful shutdown initiated");
            })
            .await
    });

    // Wait for server to start accepting connections
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    info!("Gradle Remote Cache running at: {}", cache_url);

    // Build gradle command
    let mut gradle_cmd = if cfg!(target_os = "windows") {
        Command::new("gradlew.bat")
    } else {
        Command::new("./gradlew")
    };

    // Add all user-provided arguments
    for arg in &args.gradle_args {
        gradle_cmd.arg(arg);
    }

    // Inject build cache configuration via system properties
    // This sets the remote cache URL dynamically
    gradle_cmd.arg(format!(
        "-Dorg.gradle.caching.buildCache.remote.url={}",
        cache_url
    ));
    gradle_cmd.arg("-Dorg.gradle.caching.buildCache.remote.push=true");

    // Set up stdio
    gradle_cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    info!(
        "Executing: ./gradlew {} -Dorg.gradle.caching.buildCache.remote.url={}",
        args.gradle_args.join(" "),
        cache_url
    );

    // Run gradle and wait for completion
    let status = gradle_cmd
        .status()
        .await
        .context("Failed to execute gradle")?;

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

    info!("Gradle completed with status: {}", status);

    // Exit with same code as gradle
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
