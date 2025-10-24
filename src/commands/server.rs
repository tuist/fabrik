use anyhow::Result;
use tracing::info;

use crate::cli::ServerArgs;
use crate::config::FabrikConfig;
use crate::merger::MergedServerConfig;

pub fn run(args: ServerArgs) -> Result<()> {
    // Load config file if specified
    let file_config = if let Some(config_path) = &args.config {
        Some(FabrikConfig::from_file(config_path)?)
    } else {
        None
    };

    // Merge configuration
    let config = MergedServerConfig::merge(&args, file_config);

    info!("Starting Fabrik server mode");
    info!("Configuration:");
    info!("  Cache directory: {}", config.cache_dir);
    info!("  Max cache size: {}", config.max_cache_size);
    info!("  Upstream: {:?}", config.upstream);
    info!("  HTTP bind: {}", config.http_bind);
    info!("  gRPC bind: {}", config.grpc_bind);
    info!("  S3 bind: {}", config.s3_bind);

    // Noop: In real implementation, would:
    // 1. Initialize RocksDB cache at config.cache_dir
    // 2. Configure upstream connections (S3, other Fabrik instances)
    // 3. Load JWT public key for authentication
    // 4. Start HTTP server on config.http_bind
    // 5. Start gRPC server on config.grpc_bind
    // 6. Start S3-compatible API server on config.s3_bind
    // 7. Start health API server on config.health_bind (if enabled)
    // 8. Start API server on config.api_bind (metrics, cache query, admin APIs)
    // 9. Run until SIGTERM/SIGINT with graceful shutdown

    println!("\n[NOOP] Would start server with:");
    println!("  - Local cache at: {}", config.cache_dir);
    println!("  - Max size: {}", config.max_cache_size);
    println!("  - Eviction policy: {}", config.eviction_policy);
    println!("  - Default TTL: {}", config.default_ttl);

    if !config.upstream.is_empty() {
        println!("\n  Upstream layers:");
        for (i, upstream) in config.upstream.iter().enumerate() {
            println!("    {}. {}", i + 1, upstream);
            if upstream.starts_with("s3://") {
                if let Some(region) = &config.s3_region {
                    println!("       Region: {}", region);
                }
                if let Some(endpoint) = &config.s3_endpoint {
                    println!("       Endpoint: {}", endpoint);
                }
                println!("       Workers: {}", config.upstream_workers);
            }
        }
    }

    println!("\n  Network:");
    println!("    - Fabrik Protocol (gRPC): {}", config.fabrik_bind);
    println!(
        "    - Health API:             {} (enabled: {})",
        config.health_bind, config.health_enabled
    );
    println!("    - API Server:             {}", config.api_bind);
    println!("      - Metrics API:          {}", config.metrics_enabled);
    println!(
        "      - Cache Query API:      {}",
        config.cache_query_api_enabled
    );
    println!("      - Admin API:            {}", config.admin_api_enabled);
    println!("      - Auth required:        {}", config.api_auth_required);

    println!("\n  Authentication:");
    if let Some(key_file) = &config.jwt_public_key_file {
        println!("    - JWT public key file: {}", key_file);
    } else if let Some(_key) = &config.jwt_public_key {
        println!("    - JWT public key: <inline>");
    } else if let Some(jwks_url) = &config.jwt_jwks_url {
        println!("    - JWKS URL: {}", jwks_url);
    } else {
        println!("    - JWT: Not configured");
    }
    println!("    - Required: {}", config.jwt_required);
    println!("    - Key refresh: {}", config.jwt_key_refresh);

    println!("\n  Observability:");
    println!("    - Log level: {}", config.log_level);
    println!("    - Log format: {}", config.log_format);
    println!("    - Tracing: {}", config.tracing_enabled);
    if let Some(endpoint) = &config.tracing_endpoint {
        println!("    - Tracing endpoint: {}", endpoint);
    }

    println!("\n  Runtime:");
    println!("    - Graceful shutdown: {}", config.graceful_shutdown);
    println!("    - Write-through: {}", config.write_through);

    println!("\n[NOOP] Server would now run until terminated (Ctrl+C)");

    Ok(())
}
