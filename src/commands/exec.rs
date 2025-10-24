use anyhow::Result;
use tracing::info;

use crate::cli::ExecArgs;
use crate::config::FabrikConfig;
use crate::merger::MergedExecConfig;

pub fn run(args: ExecArgs) -> Result<()> {
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
    info!("  HTTP port: {}", config.http_port);
    info!("  gRPC port: {}", config.grpc_port);
    info!("  S3 port: {}", config.s3_port);
    info!("  Build systems: {:?}", config.build_systems);
    info!("  Write-through: {}", config.write_through);
    info!("  Read-through: {}", config.read_through);
    info!("  Offline mode: {}", config.offline);
    info!("  Log level: {}", config.log_level);

    // Noop: In real implementation, would:
    // 1. Start local cache server (RocksDB)
    // 2. Configure upstream connections
    // 3. Start HTTP/gRPC/S3 servers on random ports
    // 4. Export environment variables if requested
    // 5. Execute the command: args.command
    // 6. Wait for command to complete
    // 7. Shutdown cache server

    println!("\n[NOOP] Would start cache server with:");
    println!("  - Local cache at: {}", config.cache_dir);
    println!("  - Max size: {}", config.max_cache_size);

    if !config.upstream.is_empty() {
        println!("  - Upstreams:");
        for (i, upstream) in config.upstream.iter().enumerate() {
            println!("    {}. {}", i + 1, upstream);
        }
    }

    if args.export_env {
        println!("\n[NOOP] Would export environment variables:");
        println!("  {}HTTP_URL=http://127.0.0.1:{}", args.env_prefix, config.http_port);
        println!("  {}GRPC_URL=grpc://127.0.0.1:{}", args.env_prefix, config.grpc_port);
        println!("  {}S3_ENDPOINT=http://127.0.0.1:{}", args.env_prefix, config.s3_port);
    }

    println!("\n[NOOP] Would execute command:");
    println!("  {}", args.command.join(" "));

    println!("\n[NOOP] After command completes, would shutdown cache server");

    Ok(())
}
