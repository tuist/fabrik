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
        Commands::Activate(args) => {
            // TODO: Implement activate command
            println!("fabrik activate - not yet implemented");
            println!("Args: {:?}", args);
            Ok(())
        }
        Commands::Exec(args) => commands::exec::run(args),
        Commands::Daemon(args) => commands::daemon::run(args).await,
        Commands::Deactivate(args) => {
            // TODO: Implement deactivate command
            println!("fabrik deactivate - not yet implemented");
            println!("Args: {:?}", args);
            Ok(())
        }
        Commands::Server(args) => commands::server::run(*args).await,
        Commands::Config(args) => commands::config::run(args.command),
        Commands::Health(args) => commands::health::run(args),
    }
}
