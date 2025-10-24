use clap::{Parser, Subcommand};

/// Fabrik - Multi-layer build cache infrastructure
///
/// Fabrik provides transparent, high-performance caching for build systems
/// like Gradle, Bazel, Nx, and TurboRepo.
#[derive(Parser, Debug)]
#[command(name = "fabrik")]
#[command(author = "Tuist Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Multi-layer build cache infrastructure", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Common configuration arguments shared across commands
#[derive(Parser, Debug, Clone)]
pub struct CommonConfigArgs {
    /// Config file path
    #[arg(short = 'c', long, env = "FABRIK_CONFIG")]
    pub config: Option<String>,

    /// Local cache directory
    #[arg(long, env = "FABRIK_CONFIG_CACHE_DIR")]
    pub config_cache_dir: Option<String>,

    /// Max local cache size (e.g., "5GB", "500MB")
    #[arg(long, env = "FABRIK_CONFIG_MAX_CACHE_SIZE")]
    pub config_max_cache_size: Option<String>,

    /// Upstream cache URL(s), comma-separated
    #[arg(long, env = "FABRIK_CONFIG_UPSTREAM", value_delimiter = ',')]
    pub config_upstream: Option<Vec<String>>,

    /// Log level (trace|debug|info|warn|error)
    #[arg(long, env = "FABRIK_CONFIG_LOG_LEVEL")]
    pub config_log_level: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Wrap a command with ephemeral cache server (Layer 1)
    Exec(ExecArgs),

    /// Wrap xcodebuild with Fabrik cache enabled (Unix only)
    #[cfg(unix)]
    Xcodebuild(XcodebuildArgs),

    /// Run long-lived local cache daemon (Layer 1)
    Daemon(DaemonArgs),

    /// Run regional/cloud cache server (Layer 2)
    Server(Box<ServerArgs>),

    /// Configuration management utilities
    Config(ConfigArgs),

    /// Health check and diagnostics
    Health(HealthArgs),
}

#[derive(Parser, Debug)]
pub struct ExecArgs {
    /// Config file path
    #[arg(short = 'c', long, env = "FABRIK_CONFIG")]
    pub config: Option<String>,

    // CONFIG-BACKED OPTIONS (can be set in config file)
    /// Local cache directory
    #[arg(long, env = "FABRIK_CONFIG_CACHE_DIR")]
    pub config_cache_dir: Option<String>,

    /// Max local cache size (e.g., "5GB", "500MB")
    #[arg(long, env = "FABRIK_CONFIG_MAX_CACHE_SIZE")]
    pub config_max_cache_size: Option<String>,

    /// Upstream cache URL(s), comma-separated
    #[arg(long, env = "FABRIK_CONFIG_UPSTREAM", value_delimiter = ',')]
    pub config_upstream: Option<Vec<String>>,

    /// Upstream request timeout
    #[arg(long, env = "FABRIK_CONFIG_UPSTREAM_TIMEOUT")]
    pub config_upstream_timeout: Option<String>,

    /// JWT token for authentication
    #[arg(long, env = "FABRIK_CONFIG_JWT_TOKEN")]
    pub config_jwt_token: Option<String>,

    /// File containing JWT token
    #[arg(long, env = "FABRIK_CONFIG_JWT_TOKEN_FILE")]
    pub config_jwt_token_file: Option<String>,

    /// HTTP server port (0 = random)
    #[arg(long, env = "FABRIK_CONFIG_HTTP_PORT")]
    pub config_http_port: Option<u16>,

    /// gRPC server port (0 = random)
    #[arg(long, env = "FABRIK_CONFIG_GRPC_PORT")]
    pub config_grpc_port: Option<u16>,

    /// S3 API port (0 = random)
    #[arg(long, env = "FABRIK_CONFIG_S3_PORT")]
    pub config_s3_port: Option<u16>,

    /// Enabled build systems (gradle,bazel,nx,turborepo,sccache)
    #[arg(long, env = "FABRIK_CONFIG_BUILD_SYSTEMS", value_delimiter = ',')]
    pub config_build_systems: Option<Vec<String>>,

    /// Write to upstream immediately
    #[arg(long, env = "FABRIK_CONFIG_WRITE_THROUGH")]
    pub config_write_through: bool,

    /// Populate from upstream on miss
    #[arg(long, env = "FABRIK_CONFIG_READ_THROUGH")]
    pub config_read_through: bool,

    /// Disable upstream communication
    #[arg(long, env = "FABRIK_CONFIG_OFFLINE")]
    pub config_offline: bool,

    /// Log level (trace|debug|info|warn|error)
    #[arg(long, env = "FABRIK_CONFIG_LOG_LEVEL")]
    pub config_log_level: Option<String>,

    /// Prometheus metrics port (0 = disabled)
    #[arg(long, env = "FABRIK_CONFIG_METRICS_PORT")]
    pub config_metrics_port: Option<u16>,

    // RUNTIME-ONLY OPTIONS (not in config file)
    /// Export cache URLs as environment variables
    #[arg(long)]
    pub export_env: bool,

    /// Prefix for exported environment variables
    #[arg(long, default_value = "FABRIK_")]
    pub env_prefix: String,

    /// Command to execute
    #[arg(last = true, required = true)]
    pub command: Vec<String>,
}

#[cfg(unix)]
#[derive(Parser, Debug)]
pub struct XcodebuildArgs {
    #[command(flatten)]
    pub common: CommonConfigArgs,

    /// xcodebuild arguments (e.g., -project, -scheme, -destination, etc.)
    #[arg(last = true, required = true)]
    pub xcodebuild_args: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct DaemonArgs {
    /// Config file path
    #[arg(short = 'c', long, env = "FABRIK_CONFIG")]
    pub config: Option<String>,

    // Same config-backed options as ExecArgs
    #[arg(long, env = "FABRIK_CONFIG_CACHE_DIR")]
    pub config_cache_dir: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_MAX_CACHE_SIZE")]
    pub config_max_cache_size: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_UPSTREAM", value_delimiter = ',')]
    pub config_upstream: Option<Vec<String>>,

    #[arg(long, env = "FABRIK_CONFIG_UPSTREAM_TIMEOUT")]
    pub config_upstream_timeout: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_JWT_TOKEN")]
    pub config_jwt_token: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_JWT_TOKEN_FILE")]
    pub config_jwt_token_file: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_HTTP_PORT")]
    pub config_http_port: Option<u16>,

    #[arg(long, env = "FABRIK_CONFIG_GRPC_PORT")]
    pub config_grpc_port: Option<u16>,

    #[arg(long, env = "FABRIK_CONFIG_S3_PORT")]
    pub config_s3_port: Option<u16>,

    #[arg(long, env = "FABRIK_CONFIG_BUILD_SYSTEMS", value_delimiter = ',')]
    pub config_build_systems: Option<Vec<String>>,

    #[arg(long, env = "FABRIK_CONFIG_WRITE_THROUGH")]
    pub config_write_through: bool,

    #[arg(long, env = "FABRIK_CONFIG_READ_THROUGH")]
    pub config_read_through: bool,

    #[arg(long, env = "FABRIK_CONFIG_OFFLINE")]
    pub config_offline: bool,

    #[arg(long, env = "FABRIK_CONFIG_LOG_LEVEL")]
    pub config_log_level: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_METRICS_PORT")]
    pub config_metrics_port: Option<u16>,

    // Daemon-specific options
    /// Write PID to file
    #[arg(long)]
    pub pid_file: Option<String>,

    /// Run as background process
    #[arg(long)]
    pub background: bool,

    /// Unix socket for IPC
    #[arg(long)]
    pub socket: Option<String>,
}

#[derive(Parser, Debug)]
pub struct ServerArgs {
    /// Config file path
    #[arg(short = 'c', long, env = "FABRIK_CONFIG")]
    pub config: Option<String>,

    // LOCAL STORAGE
    #[arg(long, env = "FABRIK_CONFIG_CACHE_DIR")]
    pub config_cache_dir: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_MAX_CACHE_SIZE")]
    pub config_max_cache_size: Option<String>,

    // UPSTREAM
    #[arg(long, env = "FABRIK_CONFIG_UPSTREAM", value_delimiter = ',')]
    pub config_upstream: Option<Vec<String>>,

    // S3 CREDENTIALS (for s3:// upstreams)
    #[arg(long, env = "FABRIK_CONFIG_S3_REGION")]
    pub config_s3_region: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_S3_ENDPOINT")]
    pub config_s3_endpoint: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_S3_ACCESS_KEY")]
    pub config_s3_access_key: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_S3_SECRET_KEY")]
    pub config_s3_secret_key: Option<String>,

    // NETWORK BINDINGS
    #[arg(long, env = "FABRIK_CONFIG_HTTP_BIND", default_value = "0.0.0.0:8080")]
    pub config_http_bind: String,

    #[arg(long, env = "FABRIK_CONFIG_GRPC_BIND", default_value = "0.0.0.0:9090")]
    pub config_grpc_bind: String,

    #[arg(long, env = "FABRIK_CONFIG_S3_BIND", default_value = "0.0.0.0:9000")]
    pub config_s3_bind: String,

    /// Fabrik protocol server bind address (gRPC)
    #[arg(
        long,
        env = "FABRIK_CONFIG_FABRIK_BIND",
        default_value = "0.0.0.0:7070"
    )]
    pub config_fabrik_bind: String,

    // AUTHENTICATION
    #[arg(long, env = "FABRIK_CONFIG_JWT_PUBLIC_KEY_FILE")]
    pub config_jwt_public_key_file: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_JWT_PUBLIC_KEY")]
    pub config_jwt_public_key: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_JWT_JWKS_URL")]
    pub config_jwt_jwks_url: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_JWT_KEY_REFRESH")]
    pub config_jwt_key_refresh: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_JWT_REQUIRED")]
    pub config_jwt_required: Option<bool>,

    // CACHE BEHAVIOR
    #[arg(long, env = "FABRIK_CONFIG_EVICTION_POLICY")]
    pub config_eviction_policy: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_DEFAULT_TTL")]
    pub config_default_ttl: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_WRITE_THROUGH")]
    pub config_write_through: bool,

    #[arg(long, env = "FABRIK_CONFIG_UPSTREAM_WORKERS")]
    pub config_upstream_workers: Option<u32>,

    // OBSERVABILITY
    #[arg(long, env = "FABRIK_CONFIG_LOG_LEVEL")]
    pub config_log_level: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_LOG_FORMAT")]
    pub config_log_format: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_HEALTH_BIND")]
    pub config_health_bind: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_HEALTH_ENABLED")]
    pub config_health_enabled: Option<bool>,

    #[arg(long, env = "FABRIK_CONFIG_API_BIND")]
    pub config_api_bind: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_METRICS_ENABLED")]
    pub config_metrics_enabled: Option<bool>,

    #[arg(long, env = "FABRIK_CONFIG_CACHE_QUERY_API_ENABLED")]
    pub config_cache_query_api_enabled: Option<bool>,

    #[arg(long, env = "FABRIK_CONFIG_ADMIN_API_ENABLED")]
    pub config_admin_api_enabled: Option<bool>,

    #[arg(long, env = "FABRIK_CONFIG_API_AUTH_REQUIRED")]
    pub config_api_auth_required: Option<bool>,

    #[arg(long, env = "FABRIK_CONFIG_API_JWT_PUBLIC_KEY_FILE")]
    pub config_api_jwt_public_key_file: Option<String>,

    #[arg(long, env = "FABRIK_CONFIG_TRACING_ENABLED")]
    pub config_tracing_enabled: Option<bool>,

    #[arg(long, env = "FABRIK_CONFIG_TRACING_ENDPOINT")]
    pub config_tracing_endpoint: Option<String>,

    // HIGH AVAILABILITY
    #[arg(long, env = "FABRIK_CONFIG_GRACEFUL_SHUTDOWN")]
    pub config_graceful_shutdown: Option<String>,
}

#[derive(Parser, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Validate configuration file
    Validate {
        /// Path to config file
        path: String,
    },
    /// Generate example config file
    Generate {
        /// Template type (exec, daemon, server)
        #[arg(long, default_value = "server")]
        template: String,
    },
    /// Show effective configuration (merged from all sources)
    Show {
        /// Config file path
        #[arg(short = 'c', long)]
        config: Option<String>,
    },
}

#[derive(Parser, Debug)]
pub struct HealthArgs {
    /// URL of Fabrik instance to check
    #[arg(long)]
    pub url: Option<String>,

    /// Request timeout
    #[arg(long, default_value = "5s")]
    pub timeout: String,

    /// Output format (text, json)
    #[arg(long, default_value = "text")]
    pub format: String,
}
