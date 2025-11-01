use anyhow::{Context, Result};
use std::env;

use crate::cli::ActivateArgs;
use crate::config_discovery::{discover_config, hash_config, DaemonState};

pub fn run(args: ActivateArgs) -> Result<()> {
    // If shell specified, output shell integration hook
    if let Some(shell) = args.shell {
        output_shell_hook(&shell)?;
        return Ok(());
    }

    // If --status, check/start daemon and output env vars
    if args.status {
        activate_current_directory()?;
        return Ok(());
    }

    // Default: show help
    println!("Usage:");
    println!("  fabrik activate <shell>    Generate shell integration hook");
    println!("  fabrik activate --status   Check/start daemon and export env vars");
    println!();
    println!("Shells: bash, zsh, fish");

    Ok(())
}

fn output_shell_hook(shell: &str) -> Result<()> {
    match shell {
        "bash" => {
            println!(r#"_fabrik_hook() {{
  eval "$(fabrik activate --status 2>/dev/null)"
}}

# Run on directory change
if [[ -n "${{PROMPT_COMMAND}}" ]]; then
  PROMPT_COMMAND="_fabrik_hook;${{PROMPT_COMMAND}}"
else
  PROMPT_COMMAND="_fabrik_hook"
fi
"#);
        }
        "zsh" => {
            println!(r#"_fabrik_hook() {{
  eval "$(fabrik activate --status 2>/dev/null)"
}}

# Run on directory change
autoload -U add-zsh-hook
add-zsh-hook chpwd _fabrik_hook

# Run now
_fabrik_hook
"#);
        }
        "fish" => {
            println!(r#"function _fabrik_hook --on-variable PWD
  fabrik activate --status 2>/dev/null | source
end

# Run now
_fabrik_hook
"#);
        }
        _ => {
            anyhow::bail!("Unsupported shell: {}. Use bash, zsh, or fish", shell);
        }
    }

    Ok(())
}

fn activate_current_directory() -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Find config
    let config_path = match discover_config(&current_dir)? {
        Some(path) => path,
        None => {
            // No config found, unset variables
            println!("# No .fabrik.toml found");
            output_unset_env_vars("bash");
            return Ok(());
        }
    };

    // Compute config hash
    let config_hash = hash_config(&config_path)?;

    // Check if daemon already running
    if let Some(state) = DaemonState::load(&config_hash)? {
        if state.is_running() {
            // Daemon running, export env vars
            println!("{}", state.generate_env_exports("bash"));
            return Ok(());
        }
    }

    // Need to start daemon
    println!("# Starting Fabrik daemon for config: {}", config_path.display());
    
    // TODO: Actually start the daemon process
    // For now, just indicate it would start
    println!("# TODO: Start daemon with config hash: {}", config_hash);
    
    Ok(())
}

fn output_unset_env_vars(shell: &str) {
    match shell {
        "fish" => {
            println!("set -e FABRIK_HTTP_URL 2>/dev/null");
            println!("set -e FABRIK_GRPC_URL 2>/dev/null");
            println!("set -e FABRIK_UNIX_SOCKET 2>/dev/null");
            println!("set -e FABRIK_CONFIG_HASH 2>/dev/null");
            println!("set -e FABRIK_DAEMON_PID 2>/dev/null");
            println!("set -e GRADLE_BUILD_CACHE_URL 2>/dev/null");
            println!("set -e NX_SELF_HOSTED_REMOTE_CACHE_SERVER 2>/dev/null");
            println!("set -e XCODE_CACHE_SERVER 2>/dev/null");
        }
        _ => {
            println!("unset FABRIK_HTTP_URL");
            println!("unset FABRIK_GRPC_URL");
            println!("unset FABRIK_UNIX_SOCKET");
            println!("unset FABRIK_CONFIG_HASH");
            println!("unset FABRIK_DAEMON_PID");
            println!("unset GRADLE_BUILD_CACHE_URL");
            println!("unset NX_SELF_HOSTED_REMOTE_CACHE_SERVER");
            println!("unset XCODE_CACHE_SERVER");
        }
    }
}
