#![allow(dead_code)] // P2P feature not fully integrated yet
use crate::config::P2PConfig;
use crate::p2p::{Peer, PeerInfo};
use anyhow::{Context, Result};
use mdns_sd::{ResolvedService, ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

const SERVICE_TYPE: &str = "_fabrik._tcp.local.";
const P2P_VERSION: &str = "1.0";

/// mDNS discovery service for finding P2P peers on the local network
pub struct DiscoveryService {
    config: Arc<P2PConfig>,
    mdns: Arc<ServiceDaemon>,
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    machine_id: String,
    hostname: String,
}

impl DiscoveryService {
    /// Create a new discovery service
    pub async fn new(config: Arc<P2PConfig>) -> Result<Self> {
        let mdns = ServiceDaemon::new().context("Failed to create mDNS service daemon")?;

        // Get machine ID and hostname
        let machine_id = Self::get_machine_id()?;
        let hostname = hostname::get()
            .context("Failed to get hostname")?
            .to_string_lossy()
            .to_string();

        Ok(Self {
            config,
            mdns: Arc::new(mdns),
            peers: Arc::new(RwLock::new(HashMap::new())),
            machine_id,
            hostname,
        })
    }

    /// Start discovery and advertisement
    pub async fn start(&self) -> Result<()> {
        // Advertise ourselves if enabled
        if self.config.advertise {
            self.advertise().await?;
        }

        // Start discovering peers if enabled
        if self.config.discovery {
            self.start_discovery().await?;
        }

        Ok(())
    }

    /// Advertise this instance on the network
    async fn advertise(&self) -> Result<()> {
        let instance_name = format!("fabrik-{}", self.hostname);
        let port = self.config.bind_port;

        tracing::info!(
            "Advertising P2P service as '{}' on port {}",
            instance_name,
            port
        );

        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &instance_name,
            &self.hostname,
            (), // No IPv6
            port,
            &[
                ("version", P2P_VERSION),
                ("machine_id", self.machine_id.as_str()),
            ][..],
        )
        .context("Failed to create service info")?;

        self.mdns
            .register(service_info)
            .context("Failed to register mDNS service")?;

        Ok(())
    }

    /// Start discovering peers
    async fn start_discovery(&self) -> Result<()> {
        tracing::info!("Starting P2P peer discovery");

        let receiver = self
            .mdns
            .browse(SERVICE_TYPE)
            .context("Failed to browse mDNS services")?;

        let peers = self.peers.clone();
        let machine_id = self.machine_id.clone();
        let max_peers = self.config.max_peers;

        // Spawn background task to handle discovery events
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        // Ignore ourselves
                        if let Some(mid) = info.get_property_val_str("machine_id") {
                            if mid == machine_id {
                                continue;
                            }
                        }

                        // Extract peer info
                        if let Some(peer_info) = Self::extract_peer_info(&info) {
                            let mut peers_lock = peers.write().await;

                            // Check max peers limit
                            if peers_lock.len() >= max_peers
                                && !peers_lock.contains_key(&peer_info.machine_id)
                            {
                                tracing::warn!(
                                    "Max peers limit reached ({}), ignoring peer {}",
                                    max_peers,
                                    peer_info.hostname
                                );
                                continue;
                            }

                            tracing::info!(
                                "Discovered peer: {} at {}:{}",
                                peer_info.hostname,
                                peer_info.address,
                                peer_info.port
                            );

                            peers_lock.insert(peer_info.machine_id.clone(), Peer::new(peer_info));
                        }
                    }
                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        tracing::info!("Peer removed: {}", fullname);
                        // We could remove the peer here, but we'll let it expire naturally
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Extract peer info from mDNS resolved service
    fn extract_peer_info(info: &ResolvedService) -> Option<PeerInfo> {
        let machine_id = info.get_property_val_str("machine_id")?.to_string();
        let hostname = info.get_hostname().to_string();
        let port = info.get_port();

        // Get first IPv4 address
        let address = info
            .get_addresses()
            .iter()
            .find(|addr| addr.is_ipv4())?
            .to_ip_addr();

        Some(PeerInfo {
            machine_id,
            hostname,
            address,
            port,
            last_seen: SystemTime::now(),
            accepting_requests: true,
        })
    }

    /// Get list of discovered peers
    pub async fn get_peers(&self) -> Vec<Peer> {
        let peers = self.peers.read().await;
        peers
            .values()
            .filter(|peer| peer.is_alive())
            .cloned()
            .collect()
    }

    /// Get a specific peer by machine ID
    pub async fn get_peer(&self, machine_id: &str) -> Option<Peer> {
        let peers = self.peers.read().await;
        peers.get(machine_id).cloned()
    }

    /// Remove stale peers (not seen in 60 seconds)
    pub async fn cleanup_stale_peers(&self) {
        let mut peers = self.peers.write().await;
        peers.retain(|_, peer| peer.is_alive());
    }

    /// Shutdown discovery service
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down P2P discovery service");
        self.mdns.shutdown().ok();
        Ok(())
    }

    /// Get machine ID (unique identifier for this machine)
    fn get_machine_id() -> Result<String> {
        // Try to use machine-id on Linux
        #[cfg(target_os = "linux")]
        {
            if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
                return Ok(id.trim().to_string());
            }
        }

        // Try to use system_profiler on macOS
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("ioreg")
                .args(["-rd1", "-c", "IOPlatformExpertDevice"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().find(|l| l.contains("IOPlatformUUID")) {
                    if let Some(uuid) = line.split('"').nth(3) {
                        return Ok(uuid.to_string());
                    }
                }
            }
        }

        // Fallback: use hostname + random component
        let hostname = hostname::get()
            .context("Failed to get hostname")?
            .to_string_lossy()
            .to_string();

        // Use a hash of hostname + current user as machine ID
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let machine_id = format!("{}-{}", hostname, user);

        Ok(machine_id)
    }
}
