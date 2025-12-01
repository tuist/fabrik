//! Hot-reload support for Fabrik configuration
//!
//! This module provides:
//! - File watching for automatic config reload on changes
//! - SIGHUP signal handling for manual reload (Unix only)
//! - Thread-safe shared config state with atomic updates
//!
//! # Reloadable vs Non-Reloadable Settings
//!
//! **Reloadable** (applied immediately):
//! - Upstream URLs and settings
//! - Eviction policy and thresholds
//! - Log level
//! - Auth settings (public keys)
//!
//! **Non-Reloadable** (require restart):
//! - Bind addresses/ports
//! - Cache directory
//! - Storage backend type

use crate::config::FabrikConfig;
use anyhow::{Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{broadcast, watch, RwLock};
use tracing::{error, info, warn};

/// Configuration reload event
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API for subscribers to react to config changes
pub enum ReloadEvent {
    /// Config was successfully reloaded
    Reloaded(Arc<FabrikConfig>),
    /// Config reload failed (includes error message)
    Failed(String),
}

/// Shared configuration state that can be updated atomically
pub struct HotReloadableConfig {
    /// Current configuration
    config: RwLock<Arc<FabrikConfig>>,
    /// Path to the config file being watched
    config_path: PathBuf,
    /// Broadcast channel for reload events
    reload_tx: broadcast::Sender<ReloadEvent>,
}

impl HotReloadableConfig {
    /// Create a new hot-reloadable config
    pub fn new(config: FabrikConfig, config_path: PathBuf) -> Self {
        let (reload_tx, _) = broadcast::channel(16);
        Self {
            config: RwLock::new(Arc::new(config)),
            config_path,
            reload_tx,
        }
    }

    /// Get the current configuration
    #[allow(dead_code)] // Public API for components to read current config
    pub async fn get(&self) -> Arc<FabrikConfig> {
        self.config.read().await.clone()
    }

    /// Subscribe to reload events
    #[allow(dead_code)] // Public API for components to react to config changes
    pub fn subscribe(&self) -> broadcast::Receiver<ReloadEvent> {
        self.reload_tx.subscribe()
    }

    /// Reload configuration from file
    pub async fn reload(&self) -> Result<()> {
        info!(
            "Reloading configuration from {}",
            self.config_path.display()
        );

        match FabrikConfig::from_file(&self.config_path) {
            Ok(new_config) => {
                // Validate the new config
                if let Err(e) = new_config.validate() {
                    let msg = format!("Invalid configuration: {}", e);
                    warn!("{}", msg);
                    let _ = self.reload_tx.send(ReloadEvent::Failed(msg));
                    return Err(e);
                }

                let new_config = Arc::new(new_config);

                // Log what changed
                let old_config = self.config.read().await;
                Self::log_config_changes(&old_config, &new_config);
                drop(old_config);

                // Update the config
                *self.config.write().await = new_config.clone();

                info!("Configuration reloaded successfully");
                let _ = self.reload_tx.send(ReloadEvent::Reloaded(new_config));
                Ok(())
            }
            Err(e) => {
                let msg = format!("Failed to load configuration: {}", e);
                error!("{}", msg);
                let _ = self.reload_tx.send(ReloadEvent::Failed(msg.clone()));
                Err(anyhow::anyhow!(msg))
            }
        }
    }

    /// Log configuration changes
    fn log_config_changes(old: &FabrikConfig, new: &FabrikConfig) {
        // Check upstream changes
        if old.upstream.len() != new.upstream.len() {
            info!(
                "Upstream count changed: {} -> {}",
                old.upstream.len(),
                new.upstream.len()
            );
        }

        // Check eviction policy changes
        if old.cache.eviction_policy != new.cache.eviction_policy {
            info!(
                "Eviction policy changed: {} -> {}",
                old.cache.eviction_policy, new.cache.eviction_policy
            );
        }

        // Check max cache size changes
        if old.cache.max_size != new.cache.max_size {
            info!(
                "Max cache size changed: {} -> {}",
                old.cache.max_size, new.cache.max_size
            );
        }

        // Warn about non-reloadable changes
        if old.cache.dir != new.cache.dir {
            warn!(
                "Cache directory changed ({} -> {}), but this requires a restart to take effect",
                old.cache.dir, new.cache.dir
            );
        }

        if old.fabrik.bind != new.fabrik.bind {
            warn!(
                "Fabrik bind address changed ({} -> {}), but this requires a restart to take effect",
                old.fabrik.bind, new.fabrik.bind
            );
        }
    }
}

/// Hot-reload manager that coordinates file watching and signal handling
pub struct HotReloadManager {
    #[allow(dead_code)] // Public API for components to access config
    config: Arc<HotReloadableConfig>,
    #[allow(dead_code)] // Held to keep watcher alive
    watcher: Option<RecommendedWatcher>,
    shutdown_tx: watch::Sender<bool>,
}

impl HotReloadManager {
    /// Create a new hot-reload manager
    ///
    /// This starts watching the config file for changes and sets up
    /// SIGHUP handling on Unix systems.
    pub async fn new(config: FabrikConfig, config_path: impl AsRef<Path>) -> Result<Self> {
        let config_path = config_path.as_ref().to_path_buf();
        let hot_config = Arc::new(HotReloadableConfig::new(config, config_path.clone()));

        let (shutdown_tx, _) = watch::channel(false);

        // Set up file watcher
        let watcher = Self::setup_file_watcher(hot_config.clone(), &config_path)?;

        // Set up SIGHUP handler (Unix only)
        #[cfg(unix)]
        Self::setup_sighup_handler(hot_config.clone(), shutdown_tx.subscribe());

        Ok(Self {
            config: hot_config,
            watcher: Some(watcher),
            shutdown_tx,
        })
    }

    /// Get the shared configuration
    #[allow(dead_code)] // Public API for components to access config
    pub fn config(&self) -> Arc<HotReloadableConfig> {
        self.config.clone()
    }

    /// Trigger a manual reload
    #[allow(dead_code)] // Public API for programmatic config reload
    pub async fn reload(&self) -> Result<()> {
        self.config.reload().await
    }

    /// Shutdown the hot-reload manager
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    /// Set up file watcher for config changes
    fn setup_file_watcher(
        config: Arc<HotReloadableConfig>,
        config_path: &Path,
    ) -> Result<RecommendedWatcher> {
        let config_path_clone = config_path.to_path_buf();

        // Create a channel to receive file events
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);

        // Create the watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        })
        .context("Failed to create file watcher")?;

        // Watch the config file's parent directory (to catch file replacements)
        let watch_path = config_path.parent().unwrap_or(config_path);
        watcher
            .watch(watch_path, RecursiveMode::NonRecursive)
            .context("Failed to watch config file")?;

        info!(
            "Watching config file for changes: {}",
            config_path.display()
        );

        // Spawn task to handle file events
        tokio::spawn(async move {
            // Debounce: wait a bit after changes to avoid multiple reloads
            let mut last_reload = std::time::Instant::now();
            let debounce_duration = std::time::Duration::from_millis(500);

            while let Some(event) = rx.recv().await {
                // Only handle modify/create events for our config file
                let is_relevant = matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
                    && event
                        .paths
                        .iter()
                        .any(|p| p.file_name() == config_path_clone.file_name());

                if is_relevant && last_reload.elapsed() > debounce_duration {
                    last_reload = std::time::Instant::now();
                    info!("Config file changed, reloading...");
                    if let Err(e) = config.reload().await {
                        error!("Failed to reload config: {}", e);
                    }
                }
            }
        });

        Ok(watcher)
    }

    /// Set up SIGHUP handler for manual reload (Unix only)
    #[cfg(unix)]
    fn setup_sighup_handler(
        config: Arc<HotReloadableConfig>,
        mut shutdown_rx: watch::Receiver<bool>,
    ) {
        tokio::spawn(async move {
            use tokio::signal::unix::{signal, SignalKind};

            let mut sighup = match signal(SignalKind::hangup()) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Failed to set up SIGHUP handler: {}", e);
                    return;
                }
            };

            info!("SIGHUP handler ready (send SIGHUP to reload config)");

            loop {
                tokio::select! {
                    _ = sighup.recv() => {
                        info!("Received SIGHUP, reloading configuration...");
                        if let Err(e) = config.reload().await {
                            error!("Failed to reload config on SIGHUP: {}", e);
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            break;
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config(dir: &Path) -> PathBuf {
        let config_path = dir.join("fabrik.toml");
        let config_content = r#"
[cache]
dir = "/tmp/test-cache"
max_size = "1GB"
eviction_policy = "lfu"
default_ttl = "7d"

[fabrik]
enabled = true
bind = "127.0.0.1:7070"
"#;
        fs::write(&config_path, config_content).unwrap();
        config_path
    }

    #[tokio::test]
    async fn test_hot_reloadable_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());

        let config = FabrikConfig::from_file(&config_path).unwrap();
        let hot_config = HotReloadableConfig::new(config, config_path.clone());

        // Get initial config
        let current = hot_config.get().await;
        assert_eq!(current.cache.max_size, "1GB");

        // Modify config file
        let new_content = r#"
[cache]
dir = "/tmp/test-cache"
max_size = "2GB"
eviction_policy = "lru"
default_ttl = "7d"

[fabrik]
enabled = true
bind = "127.0.0.1:7070"
"#;
        fs::write(&config_path, new_content).unwrap();

        // Reload
        hot_config.reload().await.unwrap();

        // Verify changes
        let updated = hot_config.get().await;
        assert_eq!(updated.cache.max_size, "2GB");
        assert_eq!(updated.cache.eviction_policy, "lru");
    }

    #[tokio::test]
    async fn test_reload_invalid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());

        let config = FabrikConfig::from_file(&config_path).unwrap();
        let hot_config = HotReloadableConfig::new(config, config_path.clone());

        // Write invalid config
        fs::write(&config_path, "invalid toml [[[").unwrap();

        // Reload should fail
        let result = hot_config.reload().await;
        assert!(result.is_err());

        // Original config should be preserved
        let current = hot_config.get().await;
        assert_eq!(current.cache.max_size, "1GB");
    }
}
