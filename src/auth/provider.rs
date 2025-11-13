use anyhow::Result;
use schlussel::prelude::*;
use std::sync::Arc;
use thiserror::Error;

use crate::config::{AuthConfig, AuthProvider as ConfigAuthProvider, OAuth2Config};

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AuthenticationError {
    #[error("No authentication provider configured")]
    NoProvider,

    #[error("Token not found")]
    TokenNotFound,

    #[error("Failed to read token from environment variable: {0}")]
    EnvVarError(String),

    #[error("Failed to read token from file: {0}")]
    FileError(String),

    #[error("OAuth2 error: {0}")]
    OAuth2Error(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl From<schlussel::error::OAuthError> for AuthenticationError {
    fn from(err: schlussel::error::OAuthError) -> Self {
        AuthenticationError::OAuth2Error(err.to_string())
    }
}

/// OAuth client wrapper that works with different storage backends
#[derive(Clone)]
enum OAuth2ClientWrapper {
    Keychain {
        client: Arc<OAuthClient<SecureStorage>>,
        refresher: TokenRefresher<SecureStorage>,
        storage: Arc<SecureStorage>,
    },
    File {
        client: Arc<OAuthClient<FileStorage>>,
        refresher: TokenRefresher<FileStorage>,
        storage: Arc<FileStorage>,
    },
    Memory {
        client: Arc<OAuthClient<MemoryStorage>>,
        refresher: TokenRefresher<MemoryStorage>,
        storage: Arc<MemoryStorage>,
    },
}

/// Authentication provider that supports token-based and OAuth2 authentication
pub struct AuthProvider {
    config: AuthConfig,
    oauth2_wrapper: Option<OAuth2ClientWrapper>,
    oauth2_url: Option<String>, // Resolved OAuth2 URL (from oauth2.url or root url)
}

impl OAuth2ClientWrapper {
    fn create(oauth2_config: &OAuth2Config, root_url: Option<String>) -> Result<Self> {
        // Use oauth2-specific URL if provided, otherwise use root URL
        let base_url = oauth2_config.url.clone().or(root_url).ok_or_else(|| {
            anyhow::anyhow!(
                "OAuth2 URL not configured. Set either 'url' at root level or 'auth.oauth2.url'"
            )
        })?;

        // Build OAuth configuration
        let oauth_config = OAuthConfig {
            client_id: oauth2_config.client_id.clone(),
            authorization_endpoint: oauth2_config
                .authorization_endpoint
                .clone()
                .unwrap_or_else(|| format!("{}/oauth/authorize", base_url)),
            token_endpoint: oauth2_config
                .token_endpoint
                .clone()
                .unwrap_or_else(|| format!("{}/oauth/token", base_url)),
            redirect_uri: "http://127.0.0.1:8080/callback".to_string(),
            scope: Some(oauth2_config.scopes.clone()),
            device_authorization_endpoint: oauth2_config
                .device_authorization_endpoint
                .clone()
                .or_else(|| Some(format!("{}/oauth/device/code", base_url))),
        };

        // Create storage backend and OAuth client based on configuration
        match oauth2_config.storage.as_str() {
            "keychain" => {
                let storage = Arc::new(
                    SecureStorage::new("fabrik")
                        .map_err(|e| anyhow::anyhow!("Failed to create secure storage: {}", e))?,
                );
                let client = Arc::new(OAuthClient::new(oauth_config, storage.clone()));
                let refresher = TokenRefresher::with_file_locking(client.clone(), "fabrik")
                    .map_err(|e| anyhow::anyhow!("Failed to create token refresher: {}", e))?;
                Ok(OAuth2ClientWrapper::Keychain {
                    client,
                    refresher,
                    storage,
                })
            }
            "file" => {
                // FileStorage respects XDG_DATA_HOME environment variable
                // Stores tokens in: $XDG_DATA_HOME/fabrik/ or ~/.local/share/fabrik/
                let storage = Arc::new(
                    FileStorage::new("fabrik")
                        .map_err(|e| anyhow::anyhow!("Failed to create file storage: {}", e))?,
                );
                let client = Arc::new(OAuthClient::new(oauth_config, storage.clone()));
                let refresher = TokenRefresher::with_file_locking(client.clone(), "fabrik")
                    .map_err(|e| anyhow::anyhow!("Failed to create token refresher: {}", e))?;
                Ok(OAuth2ClientWrapper::File {
                    client,
                    refresher,
                    storage,
                })
            }
            "memory" => {
                let storage = Arc::new(MemoryStorage::new());
                let client = Arc::new(OAuthClient::new(oauth_config, storage.clone()));
                let refresher = TokenRefresher::with_file_locking(client.clone(), "fabrik")
                    .map_err(|e| anyhow::anyhow!("Failed to create token refresher: {}", e))?;
                Ok(OAuth2ClientWrapper::Memory {
                    client,
                    refresher,
                    storage,
                })
            }
            other => {
                anyhow::bail!(
                    "Invalid storage backend: {}. Must be one of: keychain, file, memory",
                    other
                );
            }
        }
    }

    fn get_token_with_refresh(&self, key: &str) -> Result<Option<Token>, AuthenticationError> {
        match self {
            OAuth2ClientWrapper::Keychain { refresher, .. } => refresher
                .get_valid_token_with_threshold(key, 0.8)
                .map(Some)
                .map_err(AuthenticationError::from),
            OAuth2ClientWrapper::File { refresher, .. } => refresher
                .get_valid_token_with_threshold(key, 0.8)
                .map(Some)
                .map_err(AuthenticationError::from),
            OAuth2ClientWrapper::Memory { refresher, .. } => refresher
                .get_valid_token_with_threshold(key, 0.8)
                .map(Some)
                .map_err(AuthenticationError::from),
        }
    }

    fn authorize_device(&self) -> Result<Token, AuthenticationError> {
        match self {
            OAuth2ClientWrapper::Keychain { client, .. } => {
                client.authorize_device().map_err(AuthenticationError::from)
            }
            OAuth2ClientWrapper::File { client, .. } => {
                client.authorize_device().map_err(AuthenticationError::from)
            }
            OAuth2ClientWrapper::Memory { client, .. } => {
                client.authorize_device().map_err(AuthenticationError::from)
            }
        }
    }

    fn save_token(&self, key: &str, token: Token) -> Result<(), AuthenticationError> {
        match self {
            OAuth2ClientWrapper::Keychain { client, .. } => client
                .save_token(key, token)
                .map_err(AuthenticationError::from),
            OAuth2ClientWrapper::File { client, .. } => client
                .save_token(key, token)
                .map_err(AuthenticationError::from),
            OAuth2ClientWrapper::Memory { client, .. } => client
                .save_token(key, token)
                .map_err(AuthenticationError::from),
        }
    }

    fn delete_token(&self, key: &str) -> Result<(), AuthenticationError> {
        match self {
            OAuth2ClientWrapper::Keychain { storage, .. } => storage
                .delete_token(key)
                .map_err(AuthenticationError::OAuth2Error),
            OAuth2ClientWrapper::File { storage, .. } => storage
                .delete_token(key)
                .map_err(AuthenticationError::OAuth2Error),
            OAuth2ClientWrapper::Memory { storage, .. } => storage
                .delete_token(key)
                .map_err(AuthenticationError::OAuth2Error),
        }
    }

    fn get_token(&self, key: &str) -> Result<Option<Token>, AuthenticationError> {
        match self {
            OAuth2ClientWrapper::Keychain { client, .. } => {
                client.get_token(key).map_err(AuthenticationError::from)
            }
            OAuth2ClientWrapper::File { client, .. } => {
                client.get_token(key).map_err(AuthenticationError::from)
            }
            OAuth2ClientWrapper::Memory { client, .. } => {
                client.get_token(key).map_err(AuthenticationError::from)
            }
        }
    }
}

impl AuthProvider {
    /// Create a new authentication provider from configuration
    pub fn new(config: AuthConfig, root_url: Option<String>) -> Result<Self> {
        let (oauth2_wrapper, oauth2_url) = if matches!(
            config.provider,
            Some(ConfigAuthProvider::OAuth2)
        ) {
            let oauth2_config = config.oauth2.as_ref().ok_or_else(|| {
                AuthenticationError::ConfigError(
                    "OAuth2 provider selected but no oauth2 configuration provided".to_string(),
                )
            })?;

            // Resolve the effective OAuth2 URL
            let resolved_url = oauth2_config
                .url
                .clone()
                .or(root_url.clone())
                .ok_or_else(|| {
                    AuthenticationError::ConfigError(
                        "OAuth2 URL not configured. Set either 'url' at root level or 'auth.oauth2.url'".to_string(),
                    )
                })?;

            (
                Some(OAuth2ClientWrapper::create(oauth2_config, root_url)?),
                Some(resolved_url),
            )
        } else {
            (None, None)
        };

        Ok(Self {
            config,
            oauth2_wrapper,
            oauth2_url,
        })
    }

    /// Get a valid access token for authentication
    pub async fn get_token(&self) -> Result<String, AuthenticationError> {
        match self.config.provider {
            Some(ConfigAuthProvider::Token) => self.get_token_from_config(),
            Some(ConfigAuthProvider::OAuth2) => self.get_oauth2_token().await,
            None => Err(AuthenticationError::NoProvider),
        }
    }

    /// Get token from token configuration (env var or file) or convention-based env vars
    fn get_token_from_config(&self) -> Result<String, AuthenticationError> {
        // If token config is provided, check custom env var or file first
        if let Some(token_config) = &self.config.token {
            // Try custom environment variable if specified
            if let Some(env_var) = &token_config.env_var {
                return std::env::var(env_var)
                    .map_err(|_| AuthenticationError::EnvVarError(env_var.clone()));
            }

            // Try file
            if let Some(file_path) = &token_config.file {
                return std::fs::read_to_string(file_path)
                    .map(|s| s.trim().to_string())
                    .map_err(|e| AuthenticationError::FileError(format!("{}: {}", file_path, e)));
            }
        }

        // Fall back to convention-based environment variable (FABRIK_TOKEN)
        if let Ok(token) = std::env::var("FABRIK_TOKEN") {
            return Ok(token);
        }

        Err(AuthenticationError::TokenNotFound)
    }

    /// Get OAuth2 access token (with automatic refresh)
    async fn get_oauth2_token(&self) -> Result<String, AuthenticationError> {
        let wrapper = self
            .oauth2_wrapper
            .as_ref()
            .ok_or(AuthenticationError::ConfigError(
                "OAuth2 client not initialized".to_string(),
            ))?;

        // Get token key (use server URL as key)
        let token_key = format!(
            "{}:fabrik",
            self.oauth2_url.as_ref().expect("OAuth2 URL should be set")
        );

        // Get valid token with automatic refresh (80% threshold for proactive refresh)
        match wrapper.get_token_with_refresh(&token_key)? {
            Some(token) => Ok(token.access_token),
            None => Err(AuthenticationError::TokenNotFound),
        }
    }

    /// Login with OAuth2 device code flow
    pub async fn login(&self) -> Result<(), AuthenticationError> {
        let wrapper = self
            .oauth2_wrapper
            .as_ref()
            .ok_or(AuthenticationError::ConfigError(
                "OAuth2 provider not configured".to_string(),
            ))?
            .clone();

        tracing::info!("[fabrik] Starting OAuth2 device code flow");

        let oauth2_url = self.oauth2_url.clone();

        // Run blocking OAuth operations in a separate thread to avoid
        // "Cannot drop a runtime in a context where blocking is not allowed" errors
        let (token, token_key) = tokio::task::spawn_blocking(move || {
            // Start device code flow (opens browser and polls for completion)
            let token = wrapper.authorize_device()?;

            // Prepare token key
            let token_key = format!(
                "{}:fabrik",
                oauth2_url.as_ref().expect("OAuth2 URL should be set")
            );

            Ok::<_, AuthenticationError>((token, token_key))
        })
        .await
        .map_err(|e| AuthenticationError::OAuth2Error(format!("Task join error: {}", e)))??;

        // Save token (also needs to run in blocking context)
        let wrapper_for_save = self.oauth2_wrapper.as_ref().unwrap().clone();
        tokio::task::spawn_blocking(move || wrapper_for_save.save_token(&token_key, token))
            .await
            .map_err(|e| AuthenticationError::OAuth2Error(format!("Task join error: {}", e)))??;

        tracing::info!("[fabrik] Successfully authenticated");

        Ok(())
    }

    /// Logout (delete stored token)
    pub async fn logout(&self) -> Result<(), AuthenticationError> {
        match self.config.provider {
            Some(ConfigAuthProvider::OAuth2) => {
                let wrapper =
                    self.oauth2_wrapper
                        .as_ref()
                        .ok_or(AuthenticationError::ConfigError(
                            "OAuth2 provider not configured".to_string(),
                        ))?;

                let token_key = format!(
                    "{}:fabrik",
                    self.oauth2_url.as_ref().expect("OAuth2 URL should be set")
                );
                wrapper.delete_token(&token_key)?;

                tracing::info!("[fabrik] Successfully logged out");
                Ok(())
            }
            Some(ConfigAuthProvider::Token) => {
                // For token-based auth, we don't store anything, so nothing to delete
                tracing::info!("[fabrik] Token-based authentication doesn't require logout");
                Ok(())
            }
            None => Err(AuthenticationError::NoProvider),
        }
    }

    /// Check authentication status
    pub async fn status(&self) -> Result<AuthStatus, AuthenticationError> {
        match self.config.provider {
            Some(ConfigAuthProvider::Token) => {
                // Check if we can get a token
                match self.get_token_from_config() {
                    Ok(token) => Ok(AuthStatus {
                        authenticated: true,
                        provider: "token".to_string(),
                        token_preview: Self::preview_token(&token),
                        expires_at: None,
                    }),
                    Err(_) => Ok(AuthStatus {
                        authenticated: false,
                        provider: "token".to_string(),
                        token_preview: None,
                        expires_at: None,
                    }),
                }
            }
            Some(ConfigAuthProvider::OAuth2) => {
                let wrapper =
                    self.oauth2_wrapper
                        .as_ref()
                        .ok_or(AuthenticationError::ConfigError(
                            "OAuth2 provider not configured".to_string(),
                        ))?;

                let token_key = format!(
                    "{}:fabrik",
                    self.oauth2_url.as_ref().expect("OAuth2 URL should be set")
                );

                // Try to get token
                match wrapper.get_token(&token_key)? {
                    Some(token) => Ok(AuthStatus {
                        authenticated: true,
                        provider: "oauth2".to_string(),
                        token_preview: Self::preview_token(&token.access_token),
                        expires_at: token.expires_at,
                    }),
                    None => Ok(AuthStatus {
                        authenticated: false,
                        provider: "oauth2".to_string(),
                        token_preview: None,
                        expires_at: None,
                    }),
                }
            }
            None => Err(AuthenticationError::NoProvider),
        }
    }

    /// Create a preview of the token (first 8 and last 4 characters)
    fn preview_token(token: &str) -> Option<String> {
        if token.len() > 12 {
            Some(format!("{}...{}", &token[..8], &token[token.len() - 4..]))
        } else {
            Some("***".to_string())
        }
    }
}

/// Authentication status information
#[derive(Debug)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub provider: String,
    pub token_preview: Option<String>,
    pub expires_at: Option<u64>,
}
