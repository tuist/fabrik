use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};

use crate::cli::InitArgs;

pub fn run(args: InitArgs) -> Result<()> {
    println!("🚀 Fabrik Initialization\n");

    // Check if fabrik.toml already exists
    if std::path::Path::new("fabrik.toml").exists() {
        print!("⚠️  fabrik.toml already exists. Overwrite? [y/N] ");
        io::stdout().flush()?;

        if !args.non_interactive {
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }
    }

    // Get configuration values
    let cache_dir = if let Some(dir) = args.cache_dir {
        dir
    } else if args.non_interactive {
        ".fabrik/cache".to_string()
    } else {
        prompt_with_default("Cache directory", ".fabrik/cache")?
    };

    let max_cache_size = if let Some(size) = args.max_cache_size {
        size
    } else if args.non_interactive {
        "5GB".to_string()
    } else {
        prompt_with_default("Max cache size", "5GB")?
    };

    let upstream_url = if let Some(url) = args.upstream_url {
        Some(url)
    } else if args.non_interactive {
        None
    } else {
        print!("Do you have a remote cache server? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().eq_ignore_ascii_case("y") {
            Some(prompt(
                "Remote cache URL (e.g., grpc://cache.tuist.io:7070)",
            )?)
        } else {
            None
        }
    };

    // Generate fabrik.toml content
    let mut config = format!(
        r#"# Fabrik configuration
# See https://github.com/tuist/fabrik for more information

[cache]
dir = "{}"
max_size = "{}"
"#,
        cache_dir, max_cache_size
    );

    if let Some(ref url) = upstream_url {
        config.push_str(&format!(
            r#"
# Upstream cache configuration
[[upstream]]
url = "{}"
timeout = "30s"
"#,
            url
        ));
    }

    // Write fabrik.toml
    fs::write("fabrik.toml", &config).context("Failed to write fabrik.toml")?;

    println!("\n✅ Created fabrik.toml");
    println!("\n📄 Configuration:");
    println!("   Cache directory: {}", cache_dir);
    println!("   Max cache size: {}", max_cache_size);
    if let Some(ref url) = upstream_url {
        println!("   Remote cache: {}", url);
    }

    // Show next steps
    println!("\n🎯 Next Steps:");
    println!("   1. Verify your configuration:");
    println!("      fabrik doctor");
    println!();
    println!("   2. Navigate to your project and start building:");
    println!("      cd ~/your-project");
    println!("      gradle build  # or your build command");
    println!();
    println!("   3. The daemon will start automatically and cache your builds!");

    Ok(())
}

fn prompt(message: &str) -> Result<String> {
    print!("{}: ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

fn prompt_with_default(message: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", message, default);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}
