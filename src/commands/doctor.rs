use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;

use crate::cli::DoctorArgs;

pub fn run(args: DoctorArgs) -> Result<()> {
    println!("🔍 Fabrik Doctor - System Configuration Check\n");

    let mut all_ok = true;

    // Check 1: Fabrik binary
    if let Ok(exe_path) = env::current_exe() {
        println!("✅ Fabrik binary found: {}", exe_path.display());
        if args.verbose {
            println!("   Version: {}", env!("CARGO_PKG_VERSION"));
        }
    } else {
        println!("❌ Could not determine Fabrik binary path");
        all_ok = false;
    }

    // Check 2: Shell detection
    let shell = detect_shell();
    match &shell {
        Some(shell_name) => {
            println!("✅ Shell detected: {}", shell_name);
        }
        None => {
            println!("⚠️  Could not detect shell (SHELL env var not set)");
        }
    }

    // Check 3: Shell integration
    if let Some(shell_name) = &shell {
        let integration_status = check_shell_integration(shell_name, args.verbose);
        if integration_status {
            println!("✅ Shell integration configured");
        } else {
            println!("❌ Shell integration NOT configured");
            println!("   Run this command to set it up:");
            println!();
            match shell_name.as_str() {
                "bash" => {
                    println!("   echo 'eval \"$(fabrik activate bash)\"' >> ~/.bashrc");
                    println!("   source ~/.bashrc");
                }
                "zsh" => {
                    println!("   echo 'eval \"$(fabrik activate zsh)\"' >> ~/.zshrc");
                    println!("   source ~/.zshrc");
                }
                "fish" => {
                    println!(
                        "   echo 'fabrik activate fish | source' >> ~/.config/fish/config.fish"
                    );
                    println!("   source ~/.config/fish/config.fish");
                }
                _ => {
                    println!("   fabrik activate --help");
                }
            }
            all_ok = false;
        }
    }

    // Check 4: State directory
    let state_dir = if let Some(home) = dirs::home_dir() {
        home.join(".fabrik/daemons")
    } else {
        PathBuf::from("/tmp/fabrik/daemons")
    };

    if state_dir.exists() {
        let daemon_count = fs::read_dir(&state_dir)
            .map(|entries| entries.count())
            .unwrap_or(0);
        println!("✅ State directory exists: {}", state_dir.display());
        if args.verbose {
            println!("   Active daemons: {}", daemon_count);
        }
    } else {
        println!(
            "ℹ️  State directory not yet created: {}",
            state_dir.display()
        );
        println!("   (Will be created when first daemon starts)");
    }

    // Check 5: Current directory config
    if let Ok(current_dir) = env::current_dir() {
        if let Ok(Some(config_path)) = crate::config_discovery::discover_config(&current_dir) {
            println!("✅ Configuration found: {}", config_path.display());
            if args.verbose {
                if let Ok(config_hash) = crate::config_discovery::hash_config(&config_path) {
                    println!("   Config hash: {}", config_hash);

                    // Check if daemon is running for this config
                    if let Ok(Some(state)) =
                        crate::config_discovery::DaemonState::load(&config_hash)
                    {
                        if state.is_running() {
                            println!("   Daemon status: ✅ Running");
                            println!("   HTTP port: {}", state.http_port);
                            println!("   gRPC port: {}", state.grpc_port);
                            println!("   PID: {}", state.pid);
                        } else {
                            println!("   Daemon status: ⚠️  State exists but process not running");
                        }
                    } else {
                        println!("   Daemon status: ⚠️  Not running");
                    }
                }
            }
        } else {
            println!("ℹ️  No fabrik.toml found in current directory");
            if args.verbose {
                println!("   Run 'fabrik init' to create fabrik.toml");
            }
        }
    }

    // Check 6: Environment variables
    if args.verbose {
        println!("\n📋 Environment Variables:");
        let env_vars = [
            "FABRIK_HTTP_URL",
            "FABRIK_GRPC_URL",
            "FABRIK_CONFIG_HASH",
            "FABRIK_DAEMON_PID",
            "GRADLE_BUILD_CACHE_URL",
            "NX_SELF_HOSTED_REMOTE_CACHE_SERVER",
        ];

        let mut any_set = false;
        for var in &env_vars {
            if let Ok(value) = env::var(var) {
                println!("   {} = {}", var, value);
                any_set = true;
            }
        }

        if !any_set {
            println!("   (None set - daemon may not be active in this directory)");
        }
    }

    // Summary
    println!();
    if all_ok {
        println!("✅ All checks passed! Fabrik is properly configured.");
    } else {
        println!("⚠️  Some issues detected. Please fix the items marked with ❌ above.");
    }

    if !all_ok {
        std::process::exit(1);
    }

    Ok(())
}

fn detect_shell() -> Option<String> {
    env::var("SHELL").ok().and_then(|shell_path| {
        PathBuf::from(shell_path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    })
}

fn check_shell_integration(shell: &str, verbose: bool) -> bool {
    let config_file = match shell {
        "bash" => dirs::home_dir().map(|h| h.join(".bashrc")),
        "zsh" => dirs::home_dir().map(|h| h.join(".zshrc")),
        "fish" => dirs::home_dir().map(|h| h.join(".config/fish/config.fish")),
        _ => None,
    };

    if let Some(config_path) = config_file {
        if config_path.exists() {
            if let Ok(contents) = fs::read_to_string(&config_path) {
                let has_integration = contents.contains("fabrik activate");

                if verbose {
                    println!("   Config file: {}", config_path.display());
                    if has_integration {
                        println!("   Found 'fabrik activate' in config");
                    }
                }

                return has_integration;
            }
        } else if verbose {
            println!("   Config file not found: {}", config_path.display());
        }
    }

    false
}
