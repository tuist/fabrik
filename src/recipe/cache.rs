// Recipe cache APIs - Content-addressed caching for portable recipes
//
// This module provides runCached() and needsRun() APIs that recipes can use
// for content-addressed caching of build operations.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Configuration options for cache operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheOptions {
    /// Input file patterns (globs) that affect cache key
    #[serde(default)]
    pub inputs: Vec<String>,

    /// Output paths (files or directories) to cache/restore
    #[serde(default)]
    pub outputs: Vec<String>,

    /// Environment variables that affect cache key
    #[serde(default)]
    pub env: Vec<String>,

    /// Cache directory override (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_dir: Option<String>,

    /// Upstream cache servers override (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<Vec<String>>,

    /// Cache TTL override (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,

    /// Hash method: "content", "mtime", or "size"
    #[serde(default = "default_hash_method")]
    pub hash_method: String,
}

fn default_hash_method() -> String {
    "content".to_string()
}

impl Default for CacheOptions {
    fn default() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            env: Vec::new(),
            cache_dir: None,
            upstream: None,
            ttl: None,
            hash_method: default_hash_method(),
        }
    }
}

/// Result of a cache operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CacheResult {
    /// Was this a cache hit?
    pub cached: bool,

    /// The computed cache key (SHA256 hash)
    pub cache_key: String,

    /// Files that were restored (only for cache hits)
    #[serde(default)]
    pub restored_files: Vec<String>,

    /// Duration in milliseconds (only for cache misses after action runs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// Compute cache key from options
///
/// Cache key = SHA256(inputs_hash + env_values + hash_method)
pub async fn compute_cache_key(options: &CacheOptions, working_dir: &Path) -> Result<String> {
    let mut hasher = Sha256::new();

    // Hash input files
    for pattern in &options.inputs {
        let input_hash = hash_input_pattern(pattern, &options.hash_method, working_dir).await?;
        hasher.update(input_hash.as_bytes());
    }

    // Hash environment variables
    for env_var in &options.env {
        if let Ok(value) = std::env::var(env_var) {
            hasher.update(env_var.as_bytes());
            hasher.update(b"=");
            hasher.update(value.as_bytes());
        }
    }

    // Include hash method in cache key
    hasher.update(options.hash_method.as_bytes());

    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

/// Hash files matching a pattern
async fn hash_input_pattern(
    pattern: &str,
    hash_method: &str,
    working_dir: &Path,
) -> Result<String> {
    let mut hasher = Sha256::new();

    // Resolve pattern relative to working directory
    let pattern_path = if Path::new(pattern).is_absolute() {
        pattern.to_string()
    } else {
        working_dir.join(pattern).to_string_lossy().to_string()
    };

    // Find all matching files
    let paths = glob::glob(&pattern_path)
        .context("Failed to parse glob pattern")?
        .filter_map(|entry| entry.ok())
        .collect::<Vec<_>>();

    // Sort for deterministic hashing
    let mut sorted_paths = paths;
    sorted_paths.sort();

    // Hash each file
    for path in sorted_paths {
        if !path.is_file() {
            continue;
        }

        // Include relative path in hash for uniqueness
        let rel_path = path
            .strip_prefix(working_dir)
            .unwrap_or(&path)
            .to_string_lossy();
        hasher.update(rel_path.as_bytes());

        match hash_method {
            "content" => {
                // Hash file content
                let content = tokio::fs::read(&path)
                    .await
                    .context("Failed to read input file")?;
                hasher.update(&content);
            }
            "mtime" => {
                // Hash modification time
                let metadata = tokio::fs::metadata(&path)
                    .await
                    .context("Failed to read file metadata")?;
                if let Ok(mtime) = metadata.modified() {
                    if let Ok(duration) = mtime.duration_since(std::time::UNIX_EPOCH) {
                        hasher.update(duration.as_secs().to_le_bytes());
                    }
                }
            }
            "size" => {
                // Hash file size
                let metadata = tokio::fs::metadata(&path)
                    .await
                    .context("Failed to read file metadata")?;
                hasher.update(metadata.len().to_le_bytes());
            }
            _ => {
                anyhow::bail!("Unknown hash method: {}", hash_method);
            }
        }
    }

    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

/// KV store for tracking cache keys
pub struct KvStore {
    store_path: PathBuf,
}

impl KvStore {
    /// Create a new KV store
    pub fn new(cache_dir: &Path) -> Self {
        let store_path = cache_dir.join("kv.json");
        Self { store_path }
    }

    /// Load KV store from disk
    async fn load(&self) -> Result<HashMap<String, serde_json::Value>> {
        if !self.store_path.exists() {
            return Ok(HashMap::new());
        }

        let content = tokio::fs::read_to_string(&self.store_path)
            .await
            .context("Failed to read KV store")?;
        let map: HashMap<String, serde_json::Value> =
            serde_json::from_str(&content).context("Failed to parse KV store")?;
        Ok(map)
    }

    /// Save KV store to disk
    async fn save(&self, map: &HashMap<String, serde_json::Value>) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.store_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create KV store directory")?;
        }

        let content = serde_json::to_string_pretty(map).context("Failed to serialize KV store")?;
        tokio::fs::write(&self.store_path, content)
            .await
            .context("Failed to write KV store")?;
        Ok(())
    }

    /// Check if key exists
    pub async fn has(&self, key: &str) -> Result<bool> {
        let map = self.load().await?;
        Ok(map.contains_key(key))
    }

    /// Get value for key
    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let map = self.load().await?;
        Ok(map.get(key).cloned())
    }

    /// Set value for key
    pub async fn set(&self, key: &str, value: serde_json::Value) -> Result<()> {
        let mut map = self.load().await?;
        map.insert(key.to_string(), value);
        self.save(&map).await?;
        Ok(())
    }
}

/// Check if action needs to run (uses only KV storage)
///
/// Returns true if cache miss (action should run)
/// Returns false if cache hit (action can be skipped)
pub async fn needs_run(options: CacheOptions, working_dir: &Path) -> Result<bool> {
    // Compute cache key
    let cache_key = compute_cache_key(&options, working_dir).await?;

    // Determine cache directory
    let cache_dir = if let Some(ref dir) = options.cache_dir {
        PathBuf::from(dir)
    } else {
        working_dir.join(".fabrik/cache")
    };

    // Check KV store
    let kv = KvStore::new(&cache_dir);
    let exists = kv.has(&cache_key).await?;

    // Return true if needs run (cache miss)
    Ok(!exists)
}

/// Archive outputs to cache
pub async fn archive_outputs(
    outputs: &[String],
    cache_dir: &Path,
    cache_key: &str,
    working_dir: &Path,
) -> Result<Vec<String>> {
    let mut archived = Vec::new();

    // Create archive directory
    let archive_dir = cache_dir.join("artifacts").join(cache_key);
    tokio::fs::create_dir_all(&archive_dir)
        .await
        .context("Failed to create archive directory")?;

    for output_pattern in outputs {
        let output_path = working_dir.join(output_pattern);

        if output_path.is_file() {
            // Archive single file
            let dest = archive_dir.join(
                output_path
                    .file_name()
                    .context("Invalid output file name")?,
            );
            tokio::fs::copy(&output_path, &dest)
                .await
                .context("Failed to archive output file")?;
            archived.push(output_pattern.clone());
        } else if output_path.is_dir() {
            // Archive directory recursively
            copy_dir_all(&output_path, &archive_dir.join(output_pattern))
                .await
                .context("Failed to archive output directory")?;
            archived.push(output_pattern.clone());
        }
    }

    Ok(archived)
}

/// Restore outputs from cache
pub async fn restore_outputs(
    outputs: &[String],
    cache_dir: &Path,
    cache_key: &str,
    working_dir: &Path,
) -> Result<Vec<String>> {
    let mut restored = Vec::new();
    let archive_dir = cache_dir.join("artifacts").join(cache_key);

    if !archive_dir.exists() {
        return Ok(restored);
    }

    for output_pattern in outputs {
        let archived_path = archive_dir.join(output_pattern);
        let dest_path = working_dir.join(output_pattern);

        if archived_path.is_file() {
            // Restore single file
            if let Some(parent) = dest_path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .context("Failed to create output directory")?;
            }
            tokio::fs::copy(&archived_path, &dest_path)
                .await
                .context("Failed to restore output file")?;
            restored.push(output_pattern.clone());
        } else if archived_path.is_dir() {
            // Restore directory recursively
            copy_dir_all(&archived_path, &dest_path)
                .await
                .context("Failed to restore output directory")?;
            restored.push(output_pattern.clone());
        }
    }

    Ok(restored)
}

/// Copy directory recursively
fn copy_dir_all<'a>(
    src: &'a Path,
    dst: &'a Path,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        tokio::fs::create_dir_all(dst)
            .await
            .context("Failed to create destination directory")?;

        let mut entries = tokio::fs::read_dir(src)
            .await
            .context("Failed to read source directory")?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .context("Failed to read directory entry")?
        {
            let file_type = entry
                .file_type()
                .await
                .context("Failed to read file type")?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if file_type.is_dir() {
                copy_dir_all(&src_path, &dst_path).await?;
            } else {
                tokio::fs::copy(&src_path, &dst_path)
                    .await
                    .context("Failed to copy file")?;
            }
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_compute_cache_key() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("input.txt");
        tokio::fs::write(&test_file, b"test content").await.unwrap();

        let options = CacheOptions {
            inputs: vec!["input.txt".to_string()],
            env: vec!["TEST_VAR".to_string()],
            ..Default::default()
        };

        std::env::set_var("TEST_VAR", "test_value");

        let key1 = compute_cache_key(&options, temp_dir.path()).await.unwrap();
        let key2 = compute_cache_key(&options, temp_dir.path()).await.unwrap();

        // Same inputs should produce same key
        assert_eq!(key1, key2);

        // Different env var should produce different key
        std::env::set_var("TEST_VAR", "different_value");
        let key3 = compute_cache_key(&options, temp_dir.path()).await.unwrap();
        assert_ne!(key1, key3);
    }

    #[tokio::test]
    async fn test_kv_store() {
        let temp_dir = TempDir::new().unwrap();
        let kv = KvStore::new(temp_dir.path());

        // Initially empty
        assert!(!kv.has("test_key").await.unwrap());

        // Set value
        kv.set("test_key", serde_json::json!({"foo": "bar"}))
            .await
            .unwrap();

        // Check exists
        assert!(kv.has("test_key").await.unwrap());

        // Get value
        let value = kv.get("test_key").await.unwrap().unwrap();
        assert_eq!(value, serde_json::json!({"foo": "bar"}));
    }

    #[tokio::test]
    async fn test_needs_run() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("input.txt");
        tokio::fs::write(&test_file, b"test content").await.unwrap();

        let options = CacheOptions {
            inputs: vec!["input.txt".to_string()],
            cache_dir: Some(temp_dir.path().join("cache").to_string_lossy().to_string()),
            ..Default::default()
        };

        // First run - should need to run
        assert!(needs_run(options.clone(), temp_dir.path()).await.unwrap());

        // Simulate cache hit by setting KV entry
        let cache_key = compute_cache_key(&options, temp_dir.path()).await.unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let kv = KvStore::new(&cache_dir);
        kv.set(&cache_key, serde_json::json!({"timestamp": 123456}))
            .await
            .unwrap();

        // Second run - should not need to run
        assert!(!needs_run(options, temp_dir.path()).await.unwrap());
    }

    #[tokio::test]
    async fn test_archive_and_restore() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().join("work");
        let cache_dir = temp_dir.path().join("cache");
        tokio::fs::create_dir_all(&working_dir).await.unwrap();

        // Create output file
        let output_file = working_dir.join("output.txt");
        tokio::fs::write(&output_file, b"build output")
            .await
            .unwrap();

        // Archive
        let archived = archive_outputs(
            &["output.txt".to_string()],
            &cache_dir,
            "test_key",
            &working_dir,
        )
        .await
        .unwrap();

        assert_eq!(archived.len(), 1);

        // Remove original
        tokio::fs::remove_file(&output_file).await.unwrap();
        assert!(!output_file.exists());

        // Restore
        let restored = restore_outputs(
            &["output.txt".to_string()],
            &cache_dir,
            "test_key",
            &working_dir,
        )
        .await
        .unwrap();

        assert_eq!(restored.len(), 1);
        assert!(output_file.exists());

        let content = tokio::fs::read_to_string(&output_file).await.unwrap();
        assert_eq!(content, "build output");
    }
}
