use anyhow::Result;

use crate::cli::DeactivateArgs;

pub fn run(args: DeactivateArgs) -> Result<()> {
    // Output unset commands for environment variables
    output_unset_commands();

    if args.stop_daemon {
        println!("# TODO: Stop daemon for current directory");
    }

    Ok(())
}

fn output_unset_commands() {
    // Detect shell from SHELL env var, default to bash/zsh syntax
    let shell = std::env::var("SHELL").unwrap_or_default();
    
    if shell.contains("fish") {
        println!("set -e FABRIK_HTTP_URL 2>/dev/null");
        println!("set -e FABRIK_GRPC_URL 2>/dev/null");
        println!("set -e FABRIK_UNIX_SOCKET 2>/dev/null");
        println!("set -e FABRIK_CONFIG_HASH 2>/dev/null");
        println!("set -e FABRIK_DAEMON_PID 2>/dev/null");
        println!("set -e GRADLE_BUILD_CACHE_URL 2>/dev/null");
        println!("set -e NX_SELF_HOSTED_REMOTE_CACHE_SERVER 2>/dev/null");
        println!("set -e XCODE_CACHE_SERVER 2>/dev/null");
    } else {
        // bash/zsh
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
