use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

use crate::auth::AuthProvider;
use crate::config::FabrikConfig;

/// Login with OAuth2
pub async fn login(config: FabrikConfig) -> Result<()> {
    tracing::info!("[fabrik] Authenticating with OAuth2");

    let provider = AuthProvider::new(config.auth, config.url)
        .context("Failed to initialize authentication provider")?;

    provider.login().await.context("Authentication failed")?;

    println!("✓ Successfully authenticated!");

    Ok(())
}

/// Logout (delete stored token)
pub async fn logout(config: FabrikConfig) -> Result<()> {
    let provider = AuthProvider::new(config.auth, config.url)
        .context("Failed to initialize authentication provider")?;

    provider.logout().await.context("Logout failed")?;

    println!("✓ Successfully logged out");

    Ok(())
}

/// Check authentication status
pub async fn status(config: FabrikConfig) -> Result<()> {
    let provider = AuthProvider::new(config.auth, config.url)
        .context("Failed to initialize authentication provider")?;

    match provider.status().await {
        Ok(status) => {
            if status.authenticated {
                println!("Authentication Status: ✓ Authenticated");
                println!("Provider: {}", status.provider);

                if let Some(preview) = status.token_preview {
                    println!("Token: {}", preview);
                }

                if let Some(expires_at) = status.expires_at {
                    let dt = DateTime::<Utc>::from_timestamp(expires_at as i64, 0)
                        .unwrap_or_else(Utc::now);
                    println!("Expires: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));

                    let now = Utc::now().timestamp() as u64;
                    if expires_at > now {
                        let remaining = expires_at - now;
                        let hours = remaining / 3600;
                        let minutes = (remaining % 3600) / 60;
                        println!("Time remaining: {}h {}m", hours, minutes);
                    } else {
                        println!("⚠ Token has expired");
                    }
                }
            } else {
                println!("Authentication Status: ✗ Not authenticated");
                println!("Provider: {}", status.provider);
                println!("\nTo authenticate, run: fabrik auth login");
            }

            Ok(())
        }
        Err(e) => {
            println!("Authentication Status: ✗ Error");
            println!("Error: {}", e);
            anyhow::bail!("Failed to check authentication status");
        }
    }
}

/// Show current access token (for debugging)
pub async fn token(config: FabrikConfig) -> Result<()> {
    let provider = AuthProvider::new(config.auth, config.url)
        .context("Failed to initialize authentication provider")?;

    match provider.get_token().await {
        Ok(token) => {
            println!("{}", token);
            Ok(())
        }
        Err(e) => {
            eprintln!("[fabrik] Error: {}", e);
            anyhow::bail!("Failed to get token");
        }
    }
}
