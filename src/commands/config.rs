use anyhow::Result;
use tracing::info;

use crate::cli::ConfigCommands;
use crate::config::FabrikConfig;

pub fn run(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Validate { path } => validate(&path),
        ConfigCommands::Generate { template } => generate(&template),
        ConfigCommands::Show { config } => show(config),
    }
}

fn validate(path: &str) -> Result<()> {
    info!("Validating config file: {}", path);

    let config = FabrikConfig::from_file(path)?;
    config.validate()?;

    println!("✓ Configuration file is valid: {}", path);
    println!("\nSummary:");
    println!("  - Cache directory: {}", config.cache.dir);
    println!("  - Max cache size: {}", config.cache.max_size);
    println!("  - Eviction policy: {}", config.cache.eviction_policy);
    println!("  - Upstream layers: {}", config.upstream.len());

    for (i, upstream) in config.upstream.iter().enumerate() {
        println!(
            "    {}. {} (timeout: {})",
            i + 1,
            upstream.url,
            upstream.timeout
        );
    }

    Ok(())
}

fn generate(template: &str) -> Result<()> {
    info!("Generating example config for template: {}", template);

    let config_toml = match template {
        "exec" | "daemon" => FabrikConfig::example_exec(),
        "server" => FabrikConfig::example_server(),
        _ => {
            anyhow::bail!(
                "Unknown template: {}. Valid templates: exec, daemon, server",
                template
            );
        }
    };

    println!("{}", config_toml);

    Ok(())
}

fn show(config_path: Option<String>) -> Result<()> {
    info!("Showing effective configuration");

    let config = if let Some(path) = config_path {
        FabrikConfig::from_file(path)?
    } else {
        FabrikConfig::default()
    };

    println!("Effective Configuration:\n");
    println!("{}", toml::to_string_pretty(&config)?);

    Ok(())
}
