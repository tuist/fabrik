use super::{Storage, StorageStats};
use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// GitHub Actions Cache storage backend
///
/// Uses the GitHub Actions Cache API to store artifacts.
/// Automatically detected when running in GitHub Actions via ACTIONS_CACHE_URL environment variable.
pub struct GithubActionsStorage {
    client: Arc<Client>,
    cache_url: String,
    token: String,
    cache_version: String,
}

#[derive(Serialize, Deserialize)]
struct ReserveCacheRequest {
    key: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
struct ReserveCacheResponse {
    #[serde(rename = "cacheId")]
    cache_id: i64,
}

#[derive(Serialize, Deserialize)]
struct GetCacheResponse {
    #[serde(rename = "archiveLocation")]
    archive_location: String,
}

impl GithubActionsStorage {
    /// Create a new GitHub Actions storage backend with provided credentials
    pub fn new(cache_url: String, token: String) -> Self {
        info!("Initializing GitHub Actions storage backend");
        info!("Cache URL: {}", cache_url);

        Self {
            client: Arc::new(Client::new()),
            cache_url,
            token,
            cache_version: "v1".to_string(),
        }
    }

    /// Create a new GitHub Actions storage backend from environment variables
    ///
    /// Requires:
    /// - ACTIONS_CACHE_URL: Cache service endpoint (auto-provided by GitHub Actions)
    /// - ACTIONS_RUNTIME_TOKEN: Authentication token (auto-provided by GitHub Actions)
    pub fn from_env() -> Result<Self> {
        let cache_url = std::env::var("ACTIONS_CACHE_URL")
            .context("ACTIONS_CACHE_URL not found (not running in GitHub Actions?)")?;
        let token =
            std::env::var("ACTIONS_RUNTIME_TOKEN").context("ACTIONS_RUNTIME_TOKEN not found")?;

        Ok(Self::new(cache_url, token))
    }

    /// Check if GitHub Actions environment is available
    pub fn is_available() -> bool {
        std::env::var("ACTIONS_CACHE_URL").is_ok() && std::env::var("ACTIONS_RUNTIME_TOKEN").is_ok()
    }

    /// Check if GitHub Actions environment is available with provided environment
    pub fn is_available_with_env<F>(env_lookup: F) -> bool
    where
        F: Fn(&str) -> Option<String>,
    {
        env_lookup("ACTIONS_CACHE_URL").is_some() && env_lookup("ACTIONS_RUNTIME_TOKEN").is_some()
    }

    /// Generate cache key from ID
    fn cache_key(&self, id: &[u8]) -> String {
        format!("fabrik-{}", hex::encode(id))
    }

    /// Get cache entry
    async fn get_cache(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let url = format!(
            "{}/_apis/artifactcache/cache?keys={}&version={}",
            self.cache_url, key, self.cache_version
        );

        debug!("GET {}", url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .context("Failed to query GitHub Actions cache")?;

        if response.status() == StatusCode::NO_CONTENT {
            debug!("Cache miss for key: {}", key);
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("GitHub Actions cache query failed: {} - {}", status, body);
            return Ok(None);
        }

        let cache_entry: GetCacheResponse = response
            .json()
            .await
            .context("Failed to parse cache response")?;

        debug!(
            "Cache hit for key: {} at {}",
            key, cache_entry.archive_location
        );

        // Download the cached artifact
        let data = self
            .client
            .get(&cache_entry.archive_location)
            .send()
            .await
            .context("Failed to download cache artifact")?
            .bytes()
            .await
            .context("Failed to read cache artifact")?
            .to_vec();

        Ok(Some(data))
    }

    /// Save cache entry
    async fn save_cache(&self, key: &str, data: &[u8]) -> Result<()> {
        // Step 1: Reserve cache entry
        let reserve_url = format!("{}/_apis/artifactcache/caches", self.cache_url);

        debug!("POST {} (reserving cache for key: {})", reserve_url, key);

        let reserve_request = ReserveCacheRequest {
            key: key.to_string(),
            version: self.cache_version.clone(),
        };

        let response = self
            .client
            .post(&reserve_url)
            .bearer_auth(&self.token)
            .json(&reserve_request)
            .send()
            .await
            .context("Failed to reserve cache")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to reserve cache: {} - {}", status, body);
        }

        let reserve_response: ReserveCacheResponse = response
            .json()
            .await
            .context("Failed to parse reserve response")?;

        debug!("Reserved cache ID: {}", reserve_response.cache_id);

        // Step 2: Upload cache data
        let upload_url = format!(
            "{}/_apis/artifactcache/caches/{}",
            self.cache_url, reserve_response.cache_id
        );

        debug!("PATCH {} ({} bytes)", upload_url, data.len());

        let response = self
            .client
            .patch(&upload_url)
            .bearer_auth(&self.token)
            .header("Content-Type", "application/octet-stream")
            .header(
                "Content-Range",
                format!("bytes 0-{}/{}", data.len() - 1, data.len()),
            )
            .body(data.to_vec())
            .send()
            .await
            .context("Failed to upload cache")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to upload cache: {} - {}", status, body);
        }

        // Step 3: Commit cache
        let commit_url = format!(
            "{}/_apis/artifactcache/caches/{}",
            self.cache_url, reserve_response.cache_id
        );

        debug!("POST {} (committing cache)", commit_url);

        let commit_body = serde_json::json!({
            "size": data.len()
        });

        let response = self
            .client
            .post(&commit_url)
            .bearer_auth(&self.token)
            .json(&commit_body)
            .send()
            .await
            .context("Failed to commit cache")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Failed to commit cache: {} - {}", status, body);
        }

        debug!("Successfully saved cache for key: {}", key);
        Ok(())
    }
}

impl Storage for GithubActionsStorage {
    fn put(&self, id: &[u8], data: &[u8]) -> Result<()> {
        let key = self.cache_key(id);
        debug!(
            "Putting {} bytes to GitHub Actions cache (key: {})",
            data.len(),
            key
        );

        // Run async operation in blocking context
        let runtime = tokio::runtime::Handle::try_current().or_else(|_| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map(|rt| rt.handle().clone())
        })?;

        runtime.block_on(self.save_cache(&key, data))?;
        Ok(())
    }

    fn get(&self, id: &[u8]) -> Result<Option<Vec<u8>>> {
        let key = self.cache_key(id);
        debug!("Getting from GitHub Actions cache (key: {})", key);

        // Run async operation in blocking context
        let runtime = tokio::runtime::Handle::try_current().or_else(|_| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map(|rt| rt.handle().clone())
        })?;

        runtime.block_on(self.get_cache(&key))
    }

    fn exists(&self, id: &[u8]) -> Result<bool> {
        // Check existence by trying to get metadata (without downloading full artifact)
        Ok(self.get(id)?.is_some())
    }

    fn delete(&self, _id: &[u8]) -> Result<()> {
        // GitHub Actions cache doesn't support deletion via API
        // Caches are automatically evicted after 7 days or when 10GB limit is reached
        warn!("Delete operation not supported for GitHub Actions cache");
        Ok(())
    }

    fn size(&self, id: &[u8]) -> Result<Option<u64>> {
        // Would need to fetch the artifact to determine size
        // For now, return None (not available without full download)
        if self.exists(id)? {
            Ok(None) // Exists but size unknown without downloading
        } else {
            Ok(None)
        }
    }

    fn touch(&self, _id: &[u8]) -> Result<()> {
        // GitHub Actions cache doesn't support updating access time
        // Access time is tracked automatically by the API
        Ok(())
    }

    fn list_ids(&self) -> Result<Vec<Vec<u8>>> {
        // GitHub Actions cache doesn't provide a list API
        // This would require tracking IDs separately
        warn!("List operation not supported for GitHub Actions cache");
        Ok(vec![])
    }

    fn stats(&self) -> Result<StorageStats> {
        // GitHub Actions cache doesn't provide stats API
        Ok(StorageStats {
            total_objects: 0,
            total_bytes: 0,
            cache_dir: PathBuf::from("github-actions://"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_is_available_without_env() {
        let env: HashMap<String, String> = HashMap::new();
        let env_lookup = |key: &str| env.get(key).cloned();
        assert!(!GithubActionsStorage::is_available_with_env(env_lookup));
    }

    #[test]
    fn test_is_available_with_env() {
        let mut env = HashMap::new();
        env.insert(
            "ACTIONS_CACHE_URL".to_string(),
            "https://test.com".to_string(),
        );
        env.insert("ACTIONS_RUNTIME_TOKEN".to_string(), "token".to_string());

        let env_lookup = |key: &str| env.get(key).cloned();
        assert!(GithubActionsStorage::is_available_with_env(env_lookup));
    }

    #[test]
    fn test_is_available_with_partial_env() {
        let mut env = HashMap::new();
        env.insert(
            "ACTIONS_CACHE_URL".to_string(),
            "https://test.com".to_string(),
        );
        // Missing ACTIONS_RUNTIME_TOKEN

        let env_lookup = |key: &str| env.get(key).cloned();
        assert!(!GithubActionsStorage::is_available_with_env(env_lookup));
    }

    #[test]
    fn test_new_storage() {
        let storage =
            GithubActionsStorage::new("https://test.com".to_string(), "token".to_string());
        assert_eq!(storage.cache_url, "https://test.com");
        assert_eq!(storage.token, "token");
        assert_eq!(storage.cache_version, "v1");
    }

    #[test]
    fn test_cache_key_format() {
        let storage =
            GithubActionsStorage::new("https://test.com".to_string(), "token".to_string());
        let key = storage.cache_key(b"test");
        assert_eq!(key, "fabrik-74657374");
    }

    #[test]
    fn test_cache_key_with_different_data() {
        let storage =
            GithubActionsStorage::new("https://test.com".to_string(), "token".to_string());
        let key1 = storage.cache_key(b"data1");
        let key2 = storage.cache_key(b"data2");
        assert_ne!(key1, key2);
        assert!(key1.starts_with("fabrik-"));
        assert!(key2.starts_with("fabrik-"));
    }
}
