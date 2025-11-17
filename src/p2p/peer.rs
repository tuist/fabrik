use std::net::IpAddr;
use std::time::SystemTime;

/// Information about a discovered peer
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PeerInfo {
    /// Machine ID (unique identifier)
    pub machine_id: String,

    /// Hostname
    pub hostname: String,

    /// IP address
    pub address: IpAddr,

    /// P2P port
    pub port: u16,

    /// When this peer was last seen
    pub last_seen: SystemTime,

    /// Whether this peer is currently accepting requests
    pub accepting_requests: bool,
}

/// A peer in the P2P network
#[derive(Debug, Clone)]
pub struct Peer {
    pub info: PeerInfo,
}

impl Peer {
    pub fn new(info: PeerInfo) -> Self {
        Self { info }
    }

    /// Get the peer's gRPC endpoint URL
    pub fn endpoint(&self) -> String {
        format!("http://{}:{}", self.info.address, self.info.port)
    }

    /// Get a display name for this peer
    pub fn display_name(&self) -> String {
        format!("{}@{}", self.info.hostname, self.info.address)
    }

    /// Check if peer has been seen recently (within 30 seconds)
    pub fn is_alive(&self) -> bool {
        if let Ok(elapsed) = SystemTime::now().duration_since(self.info.last_seen) {
            elapsed.as_secs() < 30
        } else {
            false
        }
    }
}
