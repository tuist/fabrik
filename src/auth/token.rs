use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::provider::{AuthProvider, AuthenticationError};
use crate::config::AuthConfig;

/// Token manager that caches the current authentication token
#[allow(dead_code)]
pub struct TokenManager {
    provider: Arc<RwLock<Option<AuthProvider>>>,
}

impl TokenManager {
    /// Create a new token manager
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            provider: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize with authentication configuration
    #[allow(dead_code)]
    pub async fn init(&self, config: AuthConfig) -> Result<()> {
        let provider = AuthProvider::new(config)?;
        let mut lock = self.provider.write().await;
        *lock = Some(provider);
        Ok(())
    }

    /// Get a valid access token
    #[allow(dead_code)]
    pub async fn get_token(&self) -> Result<String, AuthenticationError> {
        let lock = self.provider.read().await;
        match lock.as_ref() {
            Some(provider) => provider.get_token().await,
            None => Err(AuthenticationError::NoProvider),
        }
    }

    /// Check if authentication is configured
    #[allow(dead_code)]
    pub async fn is_configured(&self) -> bool {
        let lock = self.provider.read().await;
        lock.is_some()
    }
}

impl Default for TokenManager {
    fn default() -> Self {
        Self::new()
    }
}
