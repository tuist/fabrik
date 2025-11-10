/// Script cache storage and retrieval
///
/// Integrates with Fabrik's existing cache infrastructure to store/retrieve script outputs.
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::outputs::ArchivedOutput;

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub version: u32,
    pub cache_key: String,
    pub script_path: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub execution: ExecutionInfo,
    pub inputs: Vec<InputInfo>,
    pub outputs: Vec<ArchivedOutput>,
    pub environment: std::collections::HashMap<String, String>,
    pub cache_info: CacheInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInfo {
    pub exit_code: i32,
    pub duration_ms: u64,
    pub runtime: String,
    pub runtime_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputInfo {
    pub glob: String,
    pub files: Vec<String>,
    pub combined_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub cache_hit: bool,
    pub upstream_used: Option<String>,
    pub restore_time_ms: Option<u64>,
}

/// Cache entry with metadata and archive path
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub metadata: CacheMetadata,
    pub archive_path: PathBuf,
}

/// Script cache manager
pub struct ScriptCache {
    cache_dir: PathBuf,
}

impl ScriptCache {
    /// Create new script cache manager
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        let script_cache_dir = cache_dir.join("scripts");
        fs::create_dir_all(&script_cache_dir).with_context(|| {
            format!(
                "Failed to create cache directory: {}",
                script_cache_dir.display()
            )
        })?;

        Ok(Self {
            cache_dir: script_cache_dir,
        })
    }

    /// Get cache entry if it exists and is not expired
    pub fn get(&self, cache_key: &str) -> Result<Option<CacheEntry>> {
        let entry_dir = self.cache_dir.join(cache_key);

        if !entry_dir.exists() {
            return Ok(None);
        }

        let metadata_path = entry_dir.join("metadata.json");
        let archive_path = entry_dir.join("outputs.tar.zst");

        if !metadata_path.exists() || !archive_path.exists() {
            // Invalid cache entry - clean it up
            let _ = fs::remove_dir_all(&entry_dir);
            return Ok(None);
        }

        // Load metadata
        let metadata_json = fs::read_to_string(&metadata_path)
            .with_context(|| format!("Failed to read metadata: {}", metadata_path.display()))?;

        let metadata: CacheMetadata =
            serde_json::from_str(&metadata_json).context("Failed to parse metadata JSON")?;

        // Check if expired
        if let Some(expires_at) = metadata.expires_at {
            if Utc::now() > expires_at {
                // Expired - remove and return None
                let _ = fs::remove_dir_all(&entry_dir);
                return Ok(None);
            }
        }

        Ok(Some(CacheEntry {
            metadata,
            archive_path,
        }))
    }

    /// Store cache entry
    pub fn put(&self, cache_key: &str, metadata: CacheMetadata, archive_path: &Path) -> Result<()> {
        let entry_dir = self.cache_dir.join(cache_key);
        fs::create_dir_all(&entry_dir).with_context(|| {
            format!("Failed to create entry directory: {}", entry_dir.display())
        })?;

        // Write metadata
        let metadata_path = entry_dir.join("metadata.json");
        let metadata_json =
            serde_json::to_string_pretty(&metadata).context("Failed to serialize metadata")?;
        fs::write(&metadata_path, metadata_json)
            .with_context(|| format!("Failed to write metadata: {}", metadata_path.display()))?;

        // Copy archive
        let dest_archive = entry_dir.join("outputs.tar.zst");
        fs::copy(archive_path, &dest_archive)
            .with_context(|| format!("Failed to copy archive to: {}", dest_archive.display()))?;

        Ok(())
    }

    /// Remove cache entry
    pub fn remove(&self, cache_key: &str) -> Result<()> {
        let entry_dir = self.cache_dir.join(cache_key);

        if entry_dir.exists() {
            fs::remove_dir_all(&entry_dir).with_context(|| {
                format!("Failed to remove cache entry: {}", entry_dir.display())
            })?;
        }

        Ok(())
    }

    /// List all cache entries
    pub fn list(&self) -> Result<Vec<String>> {
        let mut entries = Vec::new();

        if !self.cache_dir.exists() {
            return Ok(entries);
        }

        for entry in fs::read_dir(&self.cache_dir).with_context(|| {
            format!(
                "Failed to read cache directory: {}",
                self.cache_dir.display()
            )
        })? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    entries.push(name.to_string());
                }
            }
        }

        entries.sort();
        Ok(entries)
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let mut total_entries = 0;
        let mut total_size = 0;
        let mut total_files = 0;

        if !self.cache_dir.exists() {
            return Ok(CacheStats {
                total_entries,
                total_size_bytes: total_size,
                total_files,
            });
        }

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                total_entries += 1;

                // Read metadata
                let metadata_path = entry.path().join("metadata.json");
                if let Ok(metadata_json) = fs::read_to_string(&metadata_path) {
                    if let Ok(metadata) = serde_json::from_str::<CacheMetadata>(&metadata_json) {
                        for output in &metadata.outputs {
                            total_size += output.size_bytes;
                            total_files += output.file_count;
                        }
                    }
                }
            }
        }

        Ok(CacheStats {
            total_entries,
            total_size_bytes: total_size,
            total_files,
        })
    }

    /// Clean all cache entries
    pub fn clean_all(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir).with_context(|| {
                format!(
                    "Failed to remove cache directory: {}",
                    self.cache_dir.display()
                )
            })?;
            fs::create_dir_all(&self.cache_dir).with_context(|| {
                format!(
                    "Failed to recreate cache directory: {}",
                    self.cache_dir.display()
                )
            })?;
        }
        Ok(())
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub total_files: usize,
}

/// Parameters for creating cache metadata
#[allow(clippy::too_many_arguments)]
pub struct CreateMetadataParams<'a> {
    pub cache_key: String,
    pub script_path: &'a Path,
    pub exit_code: i32,
    pub duration: Duration,
    pub runtime: String,
    pub runtime_version: Option<String>,
    pub outputs: Vec<ArchivedOutput>,
    pub env_vars: &'a [String],
    pub ttl: Option<Duration>,
}

/// Helper to create cache metadata
pub fn create_metadata(params: CreateMetadataParams) -> CacheMetadata {
    let created_at = Utc::now();
    let expires_at = params
        .ttl
        .map(|ttl| created_at + chrono::Duration::from_std(ttl).unwrap());

    let environment = params
        .env_vars
        .iter()
        .filter_map(|var| std::env::var(var).ok().map(|value| (var.clone(), value)))
        .collect();

    CacheMetadata {
        version: 1,
        cache_key: params.cache_key,
        script_path: params.script_path.display().to_string(),
        created_at,
        expires_at,
        execution: ExecutionInfo {
            exit_code: params.exit_code,
            duration_ms: params.duration.as_millis() as u64,
            runtime: params.runtime,
            runtime_version: params.runtime_version,
        },
        inputs: Vec::new(), // TODO: populate from input hashes
        outputs: params.outputs,
        environment,
        cache_info: CacheInfo {
            cache_hit: false,
            upstream_used: None,
            restore_time_ms: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_put_and_get() {
        let temp = TempDir::new().unwrap();
        let cache_dir = temp.path().join("cache");

        let cache = ScriptCache::new(cache_dir).unwrap();

        // Create test archive
        let archive_path = temp.path().join("test.tar.zst");
        fs::write(&archive_path, b"test archive data").unwrap();

        // Create metadata
        let metadata = create_metadata(CreateMetadataParams {
            cache_key: "test-key-123".to_string(),
            script_path: Path::new("/path/to/script.sh"),
            exit_code: 0,
            duration: Duration::from_secs(10),
            runtime: "bash".to_string(),
            runtime_version: None,
            outputs: Vec::new(),
            env_vars: &[],
            ttl: None,
        });

        // Store
        cache
            .put("test-key-123", metadata.clone(), &archive_path)
            .unwrap();

        // Retrieve
        let entry = cache.get("test-key-123").unwrap();
        assert!(entry.is_some());

        let entry = entry.unwrap();
        assert_eq!(entry.metadata.cache_key, "test-key-123");
        assert!(entry.archive_path.exists());
    }

    #[test]
    fn test_cache_expiry() {
        let temp = TempDir::new().unwrap();
        let cache_dir = temp.path().join("cache");

        let cache = ScriptCache::new(cache_dir).unwrap();

        let archive_path = temp.path().join("test.tar.zst");
        fs::write(&archive_path, b"test archive data").unwrap();

        // Create metadata with immediate expiry
        let mut metadata = create_metadata(CreateMetadataParams {
            cache_key: "test-key-123".to_string(),
            script_path: Path::new("/path/to/script.sh"),
            exit_code: 0,
            duration: Duration::from_secs(10),
            runtime: "bash".to_string(),
            runtime_version: None,
            outputs: Vec::new(),
            env_vars: &[],
            ttl: Some(Duration::from_secs(0)),
        });

        // Force expiry in the past
        metadata.expires_at = Some(Utc::now() - chrono::Duration::seconds(1));

        cache.put("test-key-123", metadata, &archive_path).unwrap();

        // Should return None (expired)
        let entry = cache.get("test-key-123").unwrap();
        assert!(entry.is_none());
    }

    #[test]
    fn test_cache_list() {
        let temp = TempDir::new().unwrap();
        let cache_dir = temp.path().join("cache");

        let cache = ScriptCache::new(cache_dir).unwrap();

        let archive_path = temp.path().join("test.tar.zst");
        fs::write(&archive_path, b"test archive data").unwrap();

        // Store multiple entries
        for i in 1..=3 {
            let key = format!("test-key-{}", i);
            let metadata = create_metadata(CreateMetadataParams {
                cache_key: key.clone(),
                script_path: Path::new("/path/to/script.sh"),
                exit_code: 0,
                duration: Duration::from_secs(10),
                runtime: "bash".to_string(),
                runtime_version: None,
                outputs: Vec::new(),
                env_vars: &[],
                ttl: None,
            });
            cache.put(&key, metadata, &archive_path).unwrap();
        }

        let entries = cache.list().unwrap();
        assert_eq!(entries.len(), 3);
    }
}
