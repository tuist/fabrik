use crate::config::P2PConfig;
use crate::p2p::PeerInfo;
use anyhow::{Context, Result};
use notify_rust::{Notification, Timeout};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Consent state for a peer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsentState {
    /// User has not been asked yet
    NotAsked,
    /// User approved once (for this session)
    Once,
    /// User approved permanently
    Always,
    /// User denied
    Denied,
}

/// Consent manager handles user consent for P2P requests
pub struct ConsentManager {
    config: Arc<P2PConfig>,
    consents: Arc<RwLock<HashMap<String, ConsentState>>>,
    storage_path: PathBuf,
}

impl ConsentManager {
    /// Create a new consent manager
    pub fn new(config: Arc<P2PConfig>) -> Result<Self> {
        // Use XDG data directory for consent storage
        let data_dir = dirs::data_dir()
            .context("Failed to get data directory")?
            .join("fabrik")
            .join("p2p");

        fs::create_dir_all(&data_dir).context("Failed to create P2P data directory")?;

        let storage_path = data_dir.join("consents.json");

        // Load existing consents
        let consents = if storage_path.exists() {
            let data = fs::read_to_string(&storage_path).context("Failed to read consents file")?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self {
            config,
            consents: Arc::new(RwLock::new(consents)),
            storage_path,
        })
    }

    /// Check if consent is required for a peer
    pub async fn check_consent(&self, peer_info: &PeerInfo, hash: &str) -> Result<bool> {
        // Check consent mode
        match self.config.consent_mode.as_str() {
            "disabled" => return Ok(true),     // Always allow
            "auto-approve" => return Ok(true), // Always allow if secret is valid
            _ => {}
        }

        // Check if we have stored consent
        let consents = self.consents.read().await;
        if let Some(state) = consents.get(&peer_info.machine_id) {
            match state {
                ConsentState::Always => return Ok(true),
                ConsentState::Once => return Ok(true),
                ConsentState::Denied => return Ok(false),
                ConsentState::NotAsked => {}
            }
        }
        drop(consents);

        // Need to ask user
        self.request_consent(peer_info, hash).await
    }

    /// Request consent from user via notification
    async fn request_consent(&self, peer_info: &PeerInfo, hash: &str) -> Result<bool> {
        tracing::info!(
            "Requesting consent from user for peer {}",
            peer_info.hostname
        );

        let summary = "Fabrik Cache Request";
        let body = format!(
            "{} wants to access your build cache\nArtifact: {}",
            peer_info.hostname,
            &hash[..8.min(hash.len())]
        );

        // Show notification (blocking until user responds)
        let result = match Notification::new()
            .summary(summary)
            .body(&body)
            .icon("network-workgroup")
            .timeout(Timeout::Milliseconds(30000)) // 30 second timeout
            .show()
        {
            Ok(_) => {
                // For now, we can't wait for button clicks cross-platform
                // So we default to "allow once" on notification acknowledgment
                // A future improvement could use platform-specific notification APIs

                tracing::info!("User acknowledged notification, allowing once");
                self.set_consent(&peer_info.machine_id, ConsentState::Once)
                    .await?;
                true
            }
            Err(e) => {
                tracing::warn!("Failed to show notification: {}", e);
                // If notification fails, check mode
                match self.config.consent_mode.as_str() {
                    "notify-once" | "notify-always" => {
                        // Default to deny if notification fails
                        false
                    }
                    _ => true,
                }
            }
        };

        Ok(result)
    }

    /// Set consent state for a peer
    async fn set_consent(&self, machine_id: &str, state: ConsentState) -> Result<()> {
        let mut consents = self.consents.write().await;
        consents.insert(machine_id.to_string(), state);

        // Save to disk (for persistent consent)
        if let ConsentState::Always = consents.get(machine_id).unwrap() {
            self.save_consents(&consents).await?;
        }

        Ok(())
    }

    /// Save consents to disk
    async fn save_consents(&self, consents: &HashMap<String, ConsentState>) -> Result<()> {
        let data =
            serde_json::to_string_pretty(consents).context("Failed to serialize consents")?;
        fs::write(&self.storage_path, data).context("Failed to write consents file")?;
        Ok(())
    }

    /// Manually approve a peer (for CLI usage)
    pub async fn approve_peer(&self, machine_id: &str, permanent: bool) -> Result<()> {
        let state = if permanent {
            ConsentState::Always
        } else {
            ConsentState::Once
        };
        self.set_consent(machine_id, state).await
    }

    /// Manually deny a peer (for CLI usage)
    pub async fn deny_peer(&self, machine_id: &str) -> Result<()> {
        self.set_consent(machine_id, ConsentState::Denied).await
    }

    /// Clear all consents (for CLI usage)
    pub async fn clear_consents(&self) -> Result<()> {
        let mut consents = self.consents.write().await;
        consents.clear();
        fs::remove_file(&self.storage_path).ok(); // Ignore error if file doesn't exist
        Ok(())
    }
}
