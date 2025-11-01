mod api;
mod bazel;
mod cli;
mod commands;
mod config;
mod http;
mod logging;
mod merger;
mod storage;
mod xcode;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    logging::init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Dispatch to appropriate command handler
    match cli.command {
        Commands::Exec(args) => commands::exec::run(args),
        Commands::Bazel(args) => commands::bazel::run_bazel(args).await,
        Commands::Gradle(args) => commands::gradle::run_gradle(args).await,
        Commands::Nx(args) => commands::nx::run_nx(args).await,
        #[cfg(unix)]
        Commands::Xcodebuild(args) => commands::xcodebuild::run(args).await,
        Commands::Daemon(args) => commands::daemon::run(args).await,
        Commands::Server(args) => commands::server::run(*args).await,
        Commands::Config(args) => commands::config::run(args.command),
        Commands::Health(args) => commands::health::run(args),
    }
}
