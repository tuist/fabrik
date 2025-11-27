#![allow(dead_code)] // P2P feature not fully integrated yet
use crate::config::P2PConfig;
use crate::p2p::auth;
use crate::p2p::proto::p2p_cache_client::P2pCacheClient as GrpcP2pCacheClient;
use crate::p2p::proto::{ExistsRequest, GetRequest};
use crate::p2p::Peer;
use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Channel;

/// P2P client for fetching artifacts from peers
pub struct P2PClient {
    config: Arc<P2PConfig>,
    machine_id: String,
    hostname: String,
}

impl P2PClient {
    /// Create a new P2P client
    pub fn new(config: Arc<P2PConfig>) -> Self {
        let machine_id = Self::get_machine_id().unwrap_or_else(|_| "unknown".to_string());
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        Self {
            config,
            machine_id,
            hostname,
        }
    }

    /// Fetch artifact from peers (races all peers in parallel)
    #[allow(dead_code)] // Will be used when integrated with daemon storage layer
    pub async fn fetch_from_peers(&self, peers: &[Peer], hash: &str) -> Result<Bytes> {
        if peers.is_empty() {
            return Err(anyhow!("No peers available"));
        }

        tracing::info!(
            "Querying {} P2P peers in parallel for hash {}",
            peers.len(),
            &hash[..8]
        );

        let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<(String, Bytes)>>(peers.len());

        // Query all peers in parallel
        for peer in peers {
            let tx = tx.clone();
            let hash = hash.to_string();
            let peer = peer.clone();
            let client = self.clone();

            tokio::spawn(async move {
                match client.fetch_from_peer(&peer, &hash).await {
                    Ok(data) => {
                        let _ = tx.send(Ok((peer.info.hostname.clone(), data))).await;
                    }
                    Err(e) => {
                        tracing::debug!("P2P peer {} failed: {}", peer.info.hostname, e);
                    }
                }
            });
        }

        drop(tx);

        // Wait for first success
        if let Some(result) = rx.recv().await {
            let (hostname, data) = result?;
            tracing::info!("P2P HIT from {} ({}bytes)", hostname, data.len());
            return Ok(data);
        }

        Err(anyhow!("All P2P peers failed or timed out"))
    }

    /// Fetch artifact from a specific peer
    async fn fetch_from_peer(&self, peer: &Peer, hash: &str) -> Result<Bytes> {
        // Connect to peer with timeout
        let endpoint = peer.endpoint();
        let timeout = Duration::from_secs(
            self.config
                .request_timeout
                .trim_end_matches('s')
                .parse()
                .unwrap_or(5),
        );

        let channel = Channel::from_shared(endpoint.clone())
            .context("Invalid endpoint")?
            .timeout(timeout)
            .connect()
            .await
            .context("Failed to connect to peer")?;

        let mut client = GrpcP2pCacheClient::new(channel);

        // Check if artifact exists first
        let exists_req = self.create_exists_request(hash);
        let exists_resp = client
            .exists(exists_req)
            .await
            .context("Exists request failed")?
            .into_inner();

        if !exists_resp.found {
            return Err(anyhow!("Artifact not found on peer"));
        }

        if exists_resp.consent_required {
            return Err(anyhow!("Consent required but not granted"));
        }

        // Fetch artifact
        let get_req = self.create_get_request(hash);
        let mut stream = client.get(get_req).await?.into_inner();

        let mut data = Vec::new();
        while let Some(response) = stream.message().await? {
            if response.consent_denied {
                return Err(anyhow!("Consent denied by peer"));
            }

            data.extend_from_slice(&response.chunk);
        }

        Ok(Bytes::from(data))
    }

    /// Create exists request with authentication
    fn create_exists_request(&self, hash: &str) -> ExistsRequest {
        let timestamp = auth::current_timestamp();
        let signature = self.sign_request(hash, timestamp);

        ExistsRequest {
            hash: hash.to_string(),
            timestamp,
            signature,
            requester_id: self.machine_id.clone(),
            requester_hostname: self.hostname.clone(),
        }
    }

    /// Create get request with authentication
    fn create_get_request(&self, hash: &str) -> GetRequest {
        let timestamp = auth::current_timestamp();
        let signature = self.sign_request(hash, timestamp);

        GetRequest {
            hash: hash.to_string(),
            timestamp,
            signature,
            requester_id: self.machine_id.clone(),
            requester_hostname: self.hostname.clone(),
        }
    }

    /// Sign a request
    fn sign_request(&self, hash: &str, timestamp: i64) -> Vec<u8> {
        let secret = self
            .config
            .secret
            .as_ref()
            .expect("P2P secret must be configured");
        auth::sign_request(secret, hash, timestamp)
    }

    fn get_machine_id() -> Result<String> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
                return Ok(id.trim().to_string());
            }
        }

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

        let hostname = hostname::get()
            .context("Failed to get hostname")?
            .to_string_lossy()
            .to_string();
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        Ok(format!("{}-{}", hostname, user))
    }
}

impl Clone for P2PClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            machine_id: self.machine_id.clone(),
            hostname: self.hostname.clone(),
        }
    }
}
