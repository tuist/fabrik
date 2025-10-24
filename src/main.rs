mod api;
mod cli;
mod commands;
mod config;
mod merger;
mod storage;
mod xcode;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};

use cli::{Cli, Commands};

fn main() -> Result<()> {
    // Initialize logging
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    fmt().with_env_filter(filter).with_target(false).init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Dispatch to appropriate command handler
    match cli.command {
        Commands::Exec(args) => commands::exec::run(args),
        Commands::Xcodebuild(args) => commands::xcodebuild::run(args),
        Commands::Daemon(args) => commands::daemon::run(args),
        Commands::Server(args) => commands::server::run(*args),
        Commands::Config(args) => commands::config::run(args.command),
        Commands::Health(args) => commands::health::run(args),
    }
}
