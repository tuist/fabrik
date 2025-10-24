pub mod filesystem;

pub use filesystem::{FilesystemStorage, hash_data};

use anyhow::Result;
use std::path::PathBuf;

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
