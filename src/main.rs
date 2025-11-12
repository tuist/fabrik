mod api;
mod auth;
mod bazel;
mod cli;
mod cli_utils;
mod commands;
mod config;
mod config_discovery;
mod http;
mod logging;
mod merger;
mod script;
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
        Commands::Activate(args) => commands::activate::run(args),
        Commands::Exec(args) => commands::exec::run(args).await,
        Commands::Daemon(args) => commands::daemon::run(args).await,
        Commands::Deactivate(args) => commands::deactivate::run(args),
        Commands::Server(args) => commands::server::run(*args).await,
        Commands::Config(args) => commands::config::run(args.command),
        Commands::Health(args) => commands::health::run(args),
        Commands::Doctor(args) => commands::doctor::run(args),
        Commands::Init(args) => commands::init::run(args),
        Commands::Run(args) => commands::run::run(&args).await,
        Commands::Cas(args) => commands::cas::run(&args).await,
        Commands::Kv(args) => commands::kv::run(&args).await,
        Commands::Auth(args) => {
            use cli::AuthCommand;
            use config::FabrikConfig;

            // Load config
            let config = if let Some(config_path) = &args.config {
                FabrikConfig::from_file(config_path)?
            } else {
                FabrikConfig::default()
            };

            // Dispatch to auth subcommand
            match args.command {
                AuthCommand::Login => commands::auth::login(config).await,
                AuthCommand::Logout => commands::auth::logout(config).await,
                AuthCommand::Status => commands::auth::status(config).await,
                AuthCommand::Token => commands::auth::token(config).await,
            }
        }
    }
}
