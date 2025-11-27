use crate::cli::{P2pArgs, P2pCommand};
use crate::config::FabrikConfig;
use crate::config_discovery::load_config_with_discovery;
use crate::p2p::consent::ConsentManager;
use crate::p2p::P2PManager;
use anyhow::{Context, Result};
use rand::Rng;
use std::sync::Arc;

pub async fn run(args: P2pArgs) -> Result<()> {
    // Secret generation doesn't require config or P2P to be enabled
    if let P2pCommand::Secret { length } = args.command {
        return generate_secret(length);
    }

    // Load config for other commands
    let config = load_config_with_discovery(args.config.as_deref())?
        .context("No configuration file found. Run 'fabrik init' to create one.")?;

    // Check if P2P is enabled
    if !config.p2p.enabled {
        anyhow::bail!(
            "P2P is not enabled in configuration. Set p2p.enabled = true in your fabrik.toml"
        );
    }

    match args.command {
        P2pCommand::List { verbose, json } => list_peers(&config, verbose, json).await,
        P2pCommand::Status { json } => show_status(&config, json).await,
        P2pCommand::Approve { peer, permanent } => approve_peer(&config, &peer, permanent).await,
        P2pCommand::Deny { peer } => deny_peer(&config, &peer).await,
        P2pCommand::Clear { force } => clear_consents(&config, force).await,
        P2pCommand::Secret { .. } => unreachable!(), // Handled above
    }
}

async fn list_peers(config: &FabrikConfig, verbose: bool, json: bool) -> Result<()> {
    // Initialize P2P manager
    let p2p = P2PManager::new(config.p2p.clone()).await?;
    p2p.start().await?;

    // Wait a moment for discovery
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let peers = p2p.get_peers().await;

    if json {
        let peers_json: Vec<serde_json::Value> = peers
            .iter()
            .map(|p| {
                serde_json::json!({
                    "machine_id": p.info.machine_id,
                    "hostname": p.info.hostname,
                    "address": p.info.address.to_string(),
                    "port": p.info.port,
                    "accepting_requests": p.info.accepting_requests,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&peers_json)?);
    } else if peers.is_empty() {
        println!("No P2P peers discovered");
        println!("Make sure other instances are running with P2P enabled");
    } else {
        println!("Discovered {} peer(s):\n", peers.len());
        for peer in peers {
            println!("  • {}", peer.display_name());
            if verbose {
                println!("    Machine ID: {}", peer.info.machine_id);
                println!("    Port: {}", peer.info.port);
                println!("    Accepting requests: {}", peer.info.accepting_requests);
                println!();
            }
        }
    }

    p2p.shutdown().await?;
    Ok(())
}

async fn show_status(config: &FabrikConfig, json: bool) -> Result<()> {
    // Initialize P2P manager
    let p2p = P2PManager::new(config.p2p.clone()).await?;
    p2p.start().await?;

    // Wait for discovery
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let peers = p2p.get_peers().await;

    if json {
        let status = serde_json::json!({
            "enabled": config.p2p.enabled,
            "advertise": config.p2p.advertise,
            "discovery": config.p2p.discovery,
            "bind_port": config.p2p.bind_port,
            "consent_mode": config.p2p.consent_mode,
            "peers_discovered": peers.len(),
            "max_peers": config.p2p.max_peers,
        });
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!("P2P Cache Sharing Status\n");
        println!("  Enabled: {}", config.p2p.enabled);
        println!("  Advertise: {}", config.p2p.advertise);
        println!("  Discovery: {}", config.p2p.discovery);
        println!("  Port: {}", config.p2p.bind_port);
        println!("  Consent mode: {}", config.p2p.consent_mode);
        println!("  Max peers: {}", config.p2p.max_peers);
        println!("\n  Peers discovered: {}", peers.len());
    }

    p2p.shutdown().await?;
    Ok(())
}

async fn approve_peer(config: &FabrikConfig, peer: &str, permanent: bool) -> Result<()> {
    let consent_manager = Arc::new(ConsentManager::new(Arc::new(config.p2p.clone()))?);

    consent_manager.approve_peer(peer, permanent).await?;

    if permanent {
        println!("Permanently approved peer: {}", peer);
    } else {
        println!("Approved peer for this session: {}", peer);
    }

    Ok(())
}

async fn deny_peer(config: &FabrikConfig, peer: &str) -> Result<()> {
    let consent_manager = Arc::new(ConsentManager::new(Arc::new(config.p2p.clone()))?);

    consent_manager.deny_peer(peer).await?;

    println!("Denied peer: {}", peer);

    Ok(())
}

async fn clear_consents(config: &FabrikConfig, force: bool) -> Result<()> {
    if !force {
        println!("This will clear all stored P2P consents.");
        println!("You will need to re-approve peers next time they request access.");
        print!("Continue? [y/N] ");

        use std::io::{self, BufRead};
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines();
        let response = lines.next().unwrap_or(Ok(String::new()))?;

        if !response.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled");
            return Ok(());
        }
    }

    let consent_manager = Arc::new(ConsentManager::new(Arc::new(config.p2p.clone()))?);

    consent_manager.clear_consents().await?;

    println!("Cleared all P2P consents");

    Ok(())
}

fn generate_secret(length: usize) -> Result<()> {
    // Ensure minimum length for security
    if length < 16 {
        anyhow::bail!("Secret length must be at least 16 bytes for security");
    }

    // Generate random bytes
    let mut rng = rand::rng();
    let random_bytes: Vec<u8> = (0..length).map(|_| rng.random()).collect();

    // Encode as hexadecimal
    let secret = random_bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    println!("{}", secret);

    Ok(())
}
