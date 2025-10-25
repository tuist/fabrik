pub mod cache_dir;
pub mod filesystem;

pub use cache_dir::default_cache_dir;
pub use filesystem::FilesystemStorage;

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

/// Create storage backend
///
/// Currently only supports filesystem storage. Future versions may add
/// support for cloud storage backends (S3, GCS, etc.)
pub fn create_storage(cache_dir: &str) -> Result<FilesystemStorage> {
    info!("Initializing storage backend: filesystem");
    info!("Cache directory: {}", cache_dir);
    FilesystemStorage::new(cache_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = create_storage(temp_dir.path().to_str().unwrap()).unwrap();

        // Test basic put/get
        let test_id = b"test-id";
        let test_data = b"test-data";

        storage.put(test_id, test_data).unwrap();
        let retrieved = storage.get(test_id).unwrap();

        assert_eq!(retrieved, Some(test_data.to_vec()));
    }
}
