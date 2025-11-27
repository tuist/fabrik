/// P2P cache sharing module
///
/// This module implements peer-to-peer cache sharing on local networks.
/// It uses mDNS for discovery, gRPC for communication, HMAC for authentication,
/// and system notifications for user consent.
pub mod auth;
pub mod client;
pub mod consent;
pub mod discovery;
pub mod metrics;
pub mod peer;
pub mod server;

pub use client::P2PClient;
pub use discovery::DiscoveryService;
pub use metrics::P2PMetrics;
pub use peer::{Peer, PeerInfo};
pub use server::P2PServer;

use crate::config::P2PConfig;
use anyhow::Result;
use std::sync::Arc;

// Re-export generated proto types
pub mod proto {
    tonic::include_proto!("fabrik.p2p");
}

/// P2P manager coordinates discovery, server, and client
pub struct P2PManager {
    #[allow(dead_code)] // Kept for future use (e.g., runtime config inspection)
    config: Arc<P2PConfig>,
    discovery: Option<Arc<DiscoveryService>>,
    server: Option<Arc<P2PServer>>,
    #[allow(dead_code)] // Will be used when integrated with daemon storage layer
    client: Arc<P2PClient>,
    #[allow(dead_code)] // Will be exposed via metrics endpoint
    metrics: Arc<P2PMetrics>,
}

impl P2PManager {
    /// Create a new P2P manager
    pub async fn new(config: P2PConfig) -> Result<Self> {
        let config = Arc::new(config);

        // Initialize metrics
        let metrics = Arc::new(P2PMetrics::new());

        // Initialize discovery service if enabled
        let discovery = if config.discovery {
            tracing::info!("Initializing P2P discovery service");
            Some(Arc::new(DiscoveryService::new(config.clone()).await?))
        } else {
            None
        };

        // Initialize P2P server if advertising
        let server = if config.advertise {
            tracing::info!("Initializing P2P server on port {}", config.bind_port);
            Some(Arc::new(P2PServer::new(config.clone()).await?))
        } else {
            None
        };

        // Initialize P2P client (always needed for fetching from peers)
        let client = Arc::new(P2PClient::new(config.clone()));

        Ok(Self {
            config,
            discovery,
            server,
            client,
            metrics,
        })
    }

    /// Start P2P services
    pub async fn start(&self) -> Result<()> {
        // Start discovery if enabled
        if let Some(discovery) = &self.discovery {
            discovery.start().await?;
        }

        // Start server if advertising
        if let Some(server) = &self.server {
            server.start().await?;
        }

        tracing::info!("P2P services started successfully");
        Ok(())
    }

    /// Get the P2P client for making requests to peers
    #[allow(dead_code)] // Will be used when integrated with daemon storage layer
    pub fn client(&self) -> Arc<P2PClient> {
        self.client.clone()
    }

    /// Get discovered peers
    pub async fn get_peers(&self) -> Vec<Peer> {
        if let Some(discovery) = &self.discovery {
            discovery.get_peers().await
        } else {
            vec![]
        }
    }

    /// Get P2P metrics
    #[allow(dead_code)] // Will be exposed via metrics endpoint
    pub fn metrics(&self) -> Arc<P2PMetrics> {
        self.metrics.clone()
    }

    /// Shutdown P2P services
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down P2P services");

        if let Some(server) = &self.server {
            server.shutdown().await?;
        }

        if let Some(discovery) = &self.discovery {
            discovery.shutdown().await?;
        }

        Ok(())
    }
}
