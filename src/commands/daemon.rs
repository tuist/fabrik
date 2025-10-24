use anyhow::Result;
use tracing::info;

use crate::cli::DaemonArgs;
use crate::config::FabrikConfig;
use crate::merger::MergedExecConfig;

pub fn run(args: DaemonArgs) -> Result<()> {
    // Load config file if specified
    let file_config = if let Some(config_path) = &args.config {
        Some(FabrikConfig::from_file(config_path)?)
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

    let config = MergedExecConfig::merge(&exec_args, file_config);

    info!("Starting Fabrik daemon mode");
    info!("Configuration:");
    info!("  Cache directory: {}", config.cache_dir);
    info!("  Max cache size: {}", config.max_cache_size);
    info!("  Upstream: {:?}", config.upstream);

    // Noop: In real implementation, would:
    // 1. Check if daemon already running (via pid file)
    // 2. Start local cache server (RocksDB)
    // 3. Start HTTP/gRPC/S3 servers
    // 4. Write PID file if requested
    // 5. Create Unix socket for IPC if requested
    // 6. Daemonize if --background flag set
    // 7. Run until SIGTERM/SIGINT

    println!("\n[NOOP] Would start daemon with:");
    println!("  - Local cache at: {}", config.cache_dir);
    println!("  - Max size: {}", config.max_cache_size);
    println!("  - HTTP port: {}", config.http_port);
    println!("  - gRPC port: {}", config.grpc_port);
    println!("  - S3 port: {}", config.s3_port);

    if let Some(pid_file) = &args.pid_file {
        println!("  - PID file: {}", pid_file);
    }

    if let Some(socket) = &args.socket {
        println!("  - Unix socket: {}", socket);
    }

    if args.background {
        println!("\n[NOOP] Would daemonize process");
    } else {
        println!("\n[NOOP] Would run in foreground (press Ctrl+C to stop)");
    }

    Ok(())
}
