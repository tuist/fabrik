use clap::{Parser, Subcommand};

/// Fabrik - Multi-layer build cache infrastructure
///
/// Fabrik provides transparent, high-performance caching for build systems
/// like Gradle, Bazel, Nx, and TurboRepo.
#[derive(Parser)]
#[command(name = "fabrik")]
#[command(author = "Tuist Team")]
#[command(version = "0.1.0")]
#[command(about = "Multi-layer build cache infrastructure", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Fabrik cache server
    Server {
        /// Storage backend to use (rocksdb, s3)
        #[arg(long, default_value = "rocksdb")]
        storage_backend: String,

        /// Path to RocksDB storage (if using rocksdb backend)
        #[arg(long)]
        rocksdb_path: Option<String>,

        /// Maximum cache size (e.g., "10GB", "1TB")
        #[arg(long)]
        max_cache_size: Option<String>,

        /// Upstream cache URL for fallback
        #[arg(long)]
        upstream_url: Option<String>,

        /// Path to JWT public key file
        #[arg(long)]
        jwt_public_key: Option<String>,

        /// HTTP server port
        #[arg(long, default_value = "8080")]
        http_port: u16,

        /// gRPC server port
        #[arg(long, default_value = "9090")]
        grpc_port: u16,

        /// Metrics endpoint port
        #[arg(long, default_value = "9091")]
        metrics_port: u16,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Server {
            storage_backend,
            rocksdb_path,
            max_cache_size,
            upstream_url,
            jwt_public_key,
            http_port,
            grpc_port,
            metrics_port,
        }) => {
            println!("Starting Fabrik server...");
            println!("  Storage backend: {}", storage_backend);
            if let Some(path) = rocksdb_path {
                println!("  RocksDB path: {}", path);
            }
            if let Some(size) = max_cache_size {
                println!("  Max cache size: {}", size);
            }
            if let Some(url) = upstream_url {
                println!("  Upstream URL: {}", url);
            }
            if let Some(key) = jwt_public_key {
                println!("  JWT public key: {}", key);
            }
            println!("  HTTP port: {}", http_port);
            println!("  gRPC port: {}", grpc_port);
            println!("  Metrics port: {}", metrics_port);
            println!("\nServer starting... (not yet implemented)");
        }
        None => {
            println!("Fabrik - Multi-layer build cache infrastructure");
            println!("\nUse 'fabrik --help' to see available commands");
        }
    }
}
