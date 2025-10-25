pub mod filesystem;
pub mod github_actions;

pub use filesystem::FilesystemStorage;
pub use github_actions::GithubActionsStorage;

use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// Storage backend trait for content-addressable storage
#[allow(dead_code)]
pub trait Storage: Send + Sync {
    /// Store a blob with the given ID
    fn put(&self, id: &[u8], data: &[u8]) -> Result<()>;

    /// Retrieve a blob by ID
    fn get(&self, id: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Check if a blob exists
    fn exists(&self, id: &[u8]) -> Result<bool>;

    /// Delete a blob by ID
    fn delete(&self, id: &[u8]) -> Result<()>;

    /// Get the size of a blob in bytes
    fn size(&self, id: &[u8]) -> Result<Option<u64>>;

    /// Update access time for LRU tracking
    fn touch(&self, id: &[u8]) -> Result<()>;

    /// List all blob IDs (for eviction/cleanup)
    fn list_ids(&self) -> Result<Vec<Vec<u8>>>;

    /// Get cache statistics
    fn stats(&self) -> Result<StorageStats>;
}

/// Storage statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StorageStats {
    pub total_objects: u64,
    pub total_bytes: u64,
    pub cache_dir: PathBuf,
}

/// Storage backend type enum
pub enum StorageBackend {
    Filesystem(FilesystemStorage),
    GithubActions(GithubActionsStorage),
}

impl StorageBackend {
    /// Auto-detect the best storage backend for the current environment
    ///
    /// Detection logic:
    /// 1. GitHub Actions: ACTIONS_CACHE_URL + ACTIONS_RUNTIME_TOKEN present
    /// 2. GitLab CI: CI_API_V4_URL + CI_JOB_TOKEN present (future)
    /// 3. Fallback: Filesystem storage
    pub fn auto_detect(cache_dir: &str) -> Result<Self> {
        Self::auto_detect_with_env(cache_dir, |key| std::env::var(key).ok())
    }

    /// Auto-detect with custom environment lookup (for testing)
    pub fn auto_detect_with_env<F>(cache_dir: &str, env_lookup: F) -> Result<Self>
    where
        F: Fn(&str) -> Option<String>,
    {
        // Log environment check for debugging
        let has_cache_url = env_lookup("ACTIONS_CACHE_URL").is_some();
        let has_runtime_token = env_lookup("ACTIONS_RUNTIME_TOKEN").is_some();

        info!(
            "Storage backend detection: ACTIONS_CACHE_URL={}, ACTIONS_RUNTIME_TOKEN={}",
            has_cache_url, has_runtime_token
        );

        // Check for GitHub Actions
        if GithubActionsStorage::is_available_with_env(&env_lookup) {
            info!("✓ Detected GitHub Actions environment");
            info!("✓ Using storage backend: github-actions");

            let cache_url = env_lookup("ACTIONS_CACHE_URL")
                .ok_or_else(|| anyhow::anyhow!("ACTIONS_CACHE_URL not found"))?;
            let token = env_lookup("ACTIONS_RUNTIME_TOKEN")
                .ok_or_else(|| anyhow::anyhow!("ACTIONS_RUNTIME_TOKEN not found"))?;

            let storage = GithubActionsStorage::new(cache_url, token);
            return Ok(StorageBackend::GithubActions(storage));
        }

        // Fallback to filesystem
        info!(
            "✓ No CI environment detected (GITHUB_ACTIONS={})",
            env_lookup("GITHUB_ACTIONS").unwrap_or_else(|| "false".to_string())
        );
        info!("✓ Using storage backend: filesystem");
        info!("✓ Cache directory: {}", cache_dir);
        let storage = FilesystemStorage::new(cache_dir)?;
        Ok(StorageBackend::Filesystem(storage))
    }
}

impl Storage for StorageBackend {
    fn put(&self, id: &[u8], data: &[u8]) -> Result<()> {
        match self {
            StorageBackend::Filesystem(s) => s.put(id, data),
            StorageBackend::GithubActions(s) => s.put(id, data),
        }
    }

    fn get(&self, id: &[u8]) -> Result<Option<Vec<u8>>> {
        match self {
            StorageBackend::Filesystem(s) => s.get(id),
            StorageBackend::GithubActions(s) => s.get(id),
        }
    }

    fn exists(&self, id: &[u8]) -> Result<bool> {
        match self {
            StorageBackend::Filesystem(s) => s.exists(id),
            StorageBackend::GithubActions(s) => s.exists(id),
        }
    }

    fn delete(&self, id: &[u8]) -> Result<()> {
        match self {
            StorageBackend::Filesystem(s) => s.delete(id),
            StorageBackend::GithubActions(s) => s.delete(id),
        }
    }

    fn size(&self, id: &[u8]) -> Result<Option<u64>> {
        match self {
            StorageBackend::Filesystem(s) => s.size(id),
            StorageBackend::GithubActions(s) => s.size(id),
        }
    }

    fn touch(&self, id: &[u8]) -> Result<()> {
        match self {
            StorageBackend::Filesystem(s) => s.touch(id),
            StorageBackend::GithubActions(s) => s.touch(id),
        }
    }

    fn list_ids(&self) -> Result<Vec<Vec<u8>>> {
        match self {
            StorageBackend::Filesystem(s) => s.list_ids(),
            StorageBackend::GithubActions(s) => s.list_ids(),
        }
    }

    fn stats(&self) -> Result<StorageStats> {
        match self {
            StorageBackend::Filesystem(s) => s.stats(),
            StorageBackend::GithubActions(s) => s.stats(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_auto_detect_without_ci() {
        let env: HashMap<String, String> = HashMap::new();
        let env_lookup = |key: &str| env.get(key).cloned();

        let temp_dir = TempDir::new().unwrap();
        let backend =
            StorageBackend::auto_detect_with_env(temp_dir.path().to_str().unwrap(), env_lookup)
                .unwrap();

        assert!(matches!(backend, StorageBackend::Filesystem(_)));
    }

    /// Test that prints the actual backend being used in the current environment
    /// This is useful for verifying what backend is selected on CI
    #[test]
    fn test_detect_current_environment() {
        let temp_dir = TempDir::new().unwrap();
        let backend = StorageBackend::auto_detect(temp_dir.path().to_str().unwrap()).unwrap();

        match backend {
            StorageBackend::Filesystem(_) => {
                println!("✓ Using Filesystem storage backend");
                println!(
                    "  ACTIONS_CACHE_URL: {:?}",
                    std::env::var("ACTIONS_CACHE_URL").ok()
                );
                println!(
                    "  ACTIONS_RUNTIME_TOKEN: {:?}",
                    std::env::var("ACTIONS_RUNTIME_TOKEN")
                        .ok()
                        .map(|_| "<redacted>")
                );
                println!(
                    "  GITHUB_ACTIONS: {:?}",
                    std::env::var("GITHUB_ACTIONS").ok()
                );
            }
            StorageBackend::GithubActions(_) => {
                println!("✓ Using GitHub Actions storage backend");
                println!("  ACTIONS_CACHE_URL: present");
                println!("  ACTIONS_RUNTIME_TOKEN: present (redacted)");
            }
        }
    }

    #[test]
    fn test_auto_detect_github_actions() {
        let mut env = HashMap::new();
        env.insert(
            "ACTIONS_CACHE_URL".to_string(),
            "https://test.com".to_string(),
        );
        env.insert("ACTIONS_RUNTIME_TOKEN".to_string(), "token".to_string());

        let env_lookup = |key: &str| env.get(key).cloned();

        let temp_dir = TempDir::new().unwrap();
        let backend =
            StorageBackend::auto_detect_with_env(temp_dir.path().to_str().unwrap(), env_lookup)
                .unwrap();

        assert!(matches!(backend, StorageBackend::GithubActions(_)));
    }

    #[test]
    fn test_auto_detect_github_actions_missing_token() {
        let mut env = HashMap::new();
        env.insert(
            "ACTIONS_CACHE_URL".to_string(),
            "https://test.com".to_string(),
        );
        // Missing ACTIONS_RUNTIME_TOKEN

        let env_lookup = |key: &str| env.get(key).cloned();

        let temp_dir = TempDir::new().unwrap();
        // Should fall back to filesystem since env is incomplete
        let backend =
            StorageBackend::auto_detect_with_env(temp_dir.path().to_str().unwrap(), env_lookup)
                .unwrap();

        assert!(matches!(backend, StorageBackend::Filesystem(_)));
    }
}
