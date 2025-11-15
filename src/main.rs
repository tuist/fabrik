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
mod xdg;

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
        Commands::Cache(args) => commands::cache::cache(&args).await,
        Commands::Auth(args) => {
            use cli::AuthCommand;
            use config::FabrikConfig;

            // Helper to load config with discovery
            let load_config = |config_path: &Option<String>| -> Result<FabrikConfig> {
                if let Some(path) = config_path {
                    FabrikConfig::from_file(path)
                } else {
                    // Try to discover config by traversing up directory tree
                    match config_discovery::discover_config(&std::env::current_dir()?)? {
                        Some(config_path) => {
                            tracing::info!("[fabrik] Using config: {}", config_path.display());
                            FabrikConfig::from_file(&config_path)
                        }
                        None => {
                            tracing::warn!("[fabrik] No configuration file found, using defaults");
                            Ok(FabrikConfig::default())
                        }
                    }
                }
            };

            // Dispatch to auth subcommand
            match args.command {
                AuthCommand::Login(subargs) => {
                    let config = load_config(&subargs.config)?;
                    commands::auth::login(config).await
                }
                AuthCommand::Logout(subargs) => {
                    let config = load_config(&subargs.config)?;
                    commands::auth::logout(config).await
                }
                AuthCommand::Status(subargs) => {
                    let config = load_config(&subargs.config)?;
                    commands::auth::status(config).await
                }
                AuthCommand::Token(subargs) => {
                    let config = load_config(&subargs.config)?;
                    commands::auth::token(config).await
                }
            }
        }
    }
}
