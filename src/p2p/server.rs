#![allow(dead_code)] // P2P feature not fully integrated yet
use crate::config::P2PConfig;
use crate::p2p::auth;
use crate::p2p::consent::ConsentManager;
use crate::p2p::proto::p2p_cache_server::{P2pCache, P2pCacheServer};
use crate::p2p::proto::{
    ExistsRequest, ExistsResponse, GetRequest, GetResponse, HelloRequest, HelloResponse,
};
use crate::p2p::PeerInfo;
use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

/// P2P gRPC server
pub struct P2PServer {
    config: Arc<P2PConfig>,
    consent_manager: Arc<ConsentManager>,
    cache_dir: Arc<RwLock<String>>, // We'll get this from main cache config later
    bind_addr: SocketAddr,
    machine_id: String,
    hostname: String,
}

impl P2PServer {
    /// Create a new P2P server
    pub async fn new(config: Arc<P2PConfig>) -> Result<Self> {
        let bind_addr: SocketAddr = format!("0.0.0.0:{}", config.bind_port)
            .parse()
            .context("Failed to parse bind address")?;

        let consent_manager = Arc::new(ConsentManager::new(config.clone())?);

        let machine_id = Self::get_machine_id()?;
        let hostname = hostname::get()
            .context("Failed to get hostname")?
            .to_string_lossy()
            .to_string();

        Ok(Self {
            config,
            consent_manager,
            cache_dir: Arc::new(RwLock::new(String::new())),
            bind_addr,
            machine_id,
            hostname,
        })
    }

    /// Set cache directory (will be called by main daemon)
    pub async fn set_cache_dir(&self, cache_dir: String) {
        let mut dir = self.cache_dir.write().await;
        *dir = cache_dir;
    }

    /// Start the P2P server
    pub async fn start(&self) -> Result<()> {
        let service = P2PCacheService {
            config: self.config.clone(),
            consent_manager: self.consent_manager.clone(),
            cache_dir: self.cache_dir.clone(),
            machine_id: self.machine_id.clone(),
            hostname: self.hostname.clone(),
        };

        let bind_addr = self.bind_addr;

        tracing::info!("[fabrik] P2P gRPC server listening on {}", bind_addr);

        tokio::spawn(async move {
            Server::builder()
                .add_service(P2pCacheServer::new(service))
                .serve(bind_addr)
                .await
                .expect("P2P server failed");
        });

        Ok(())
    }

    /// Shutdown the server
    pub async fn shutdown(&self) -> Result<()> {
        // Server will shutdown when runtime shuts down
        Ok(())
    }

    fn get_machine_id() -> Result<String> {
        // Same logic as discovery service
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

/// P2P cache service implementation
struct P2PCacheService {
    config: Arc<P2PConfig>,
    consent_manager: Arc<ConsentManager>,
    cache_dir: Arc<RwLock<String>>,
    machine_id: String,
    hostname: String,
}

#[tonic::async_trait]
impl P2pCache for P2PCacheService {
    async fn exists(
        &self,
        request: Request<ExistsRequest>,
    ) -> Result<Response<ExistsResponse>, Status> {
        let req = request.into_inner();

        // Verify authentication
        if let Err(e) = self.verify_auth(&req.hash, req.timestamp, &req.signature) {
            tracing::warn!("[fabrik] P2P auth failed: {}", e);
            return Err(Status::unauthenticated(format!(
                "Authentication failed: {}",
                e
            )));
        }

        let peer_info = PeerInfo {
            machine_id: req.requester_id.clone(),
            hostname: req.requester_hostname.clone(),
            address: "0.0.0.0".parse().unwrap(), // We don't have IP in request
            port: 0,
            last_seen: std::time::SystemTime::now(),
            accepting_requests: true,
        };

        // Check consent
        let has_consent = self
            .consent_manager
            .check_consent(&peer_info, &req.hash)
            .await
            .unwrap_or(false);

        if !has_consent {
            tracing::info!(
                "[fabrik] P2P request denied (no consent) from {}",
                req.requester_hostname
            );
            return Ok(Response::new(ExistsResponse {
                found: false,
                consent_required: true,
                consent_message: "User consent required".to_string(),
            }));
        }

        // Check if artifact exists in cache
        // TODO: Integrate with actual cache layer
        let cache_dir = self.cache_dir.read().await;
        let artifact_path = std::path::Path::new(cache_dir.as_str()).join(&req.hash);
        let found = artifact_path.exists();

        tracing::info!(
            "[fabrik] P2P exists check from {}: hash={} found={}",
            req.requester_hostname,
            &req.hash[..8],
            found
        );

        Ok(Response::new(ExistsResponse {
            found,
            consent_required: false,
            consent_message: String::new(),
        }))
    }

    type GetStream = ReceiverStream<Result<GetResponse, Status>>;

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<Self::GetStream>, Status> {
        let req = request.into_inner();

        // Verify authentication
        if let Err(e) = self.verify_auth(&req.hash, req.timestamp, &req.signature) {
            tracing::warn!("[fabrik] P2P auth failed: {}", e);
            return Err(Status::unauthenticated(format!(
                "Authentication failed: {}",
                e
            )));
        }

        let peer_info = PeerInfo {
            machine_id: req.requester_id.clone(),
            hostname: req.requester_hostname.clone(),
            address: "0.0.0.0".parse().unwrap(),
            port: 0,
            last_seen: std::time::SystemTime::now(),
            accepting_requests: true,
        };

        // Check consent
        let has_consent = self
            .consent_manager
            .check_consent(&peer_info, &req.hash)
            .await
            .unwrap_or(false);

        if !has_consent {
            tracing::info!(
                "[fabrik] P2P request denied (no consent) from {}",
                req.requester_hostname
            );
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            tx.send(Ok(GetResponse {
                chunk: vec![],
                total_size: 0,
                consent_required: true,
                consent_denied: true,
            }))
            .await
            .ok();
            return Ok(Response::new(ReceiverStream::new(rx)));
        }

        // Stream artifact from cache
        let cache_dir = self.cache_dir.read().await.clone();
        let hash = req.hash.clone();
        let hostname = req.requester_hostname.clone();

        let (tx, rx) = tokio::sync::mpsc::channel(128);

        tokio::spawn(async move {
            let artifact_path = std::path::Path::new(&cache_dir).join(&hash);

            if !artifact_path.exists() {
                tracing::warn!("[fabrik] P2P artifact not found: {}", hash);
                return;
            }

            // Read and stream file
            match tokio::fs::read(&artifact_path).await {
                Ok(data) => {
                    let total_size = data.len() as i64;
                    tracing::info!(
                        "[fabrik] P2P serving artifact to {}: hash={} size={}",
                        hostname,
                        &hash[..8],
                        total_size
                    );

                    // Send in chunks (32KB)
                    const CHUNK_SIZE: usize = 32 * 1024;
                    for (i, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
                        let response = GetResponse {
                            chunk: chunk.to_vec(),
                            total_size: if i == 0 { total_size } else { 0 },
                            consent_required: false,
                            consent_denied: false,
                        };

                        if tx.send(Ok(response)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("[fabrik] Failed to read artifact: {}", e);
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let req = request.into_inner();

        tracing::debug!("[fabrik] P2P hello from {}", req.hostname);

        Ok(Response::new(HelloResponse {
            machine_id: self.machine_id.clone(),
            hostname: self.hostname.clone(),
            version: "1.0".to_string(),
            accepting_requests: true,
        }))
    }
}

impl P2PCacheService {
    fn verify_auth(&self, hash: &str, timestamp: i64, signature: &[u8]) -> Result<()> {
        let secret = self
            .config
            .secret
            .as_ref()
            .context("P2P secret not configured")?;

        auth::verify_request(secret, hash, timestamp, signature)
    }
}
