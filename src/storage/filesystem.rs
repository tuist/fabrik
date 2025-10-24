use super::{Storage, StorageStats};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Filesystem-based storage with SQLite metadata tracking
///
/// Layout:
/// - `.fabrik/cache/objects/ab/cd1234...` - Content-addressed blob storage (first 2 chars = subdir)
/// - `.fabrik/cache/metadata.db` - SQLite database for access tracking and eviction
pub struct FilesystemStorage {
    objects_dir: PathBuf,
    db: Arc<Mutex<Connection>>,
}

impl FilesystemStorage {
    /// Create a new filesystem storage at the given cache directory
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self> {
        let cache_dir = cache_dir.as_ref();
        let objects_dir = cache_dir.join("objects");
        let db_path = cache_dir.join("metadata.db");

        // Create directories
        fs::create_dir_all(&objects_dir)
            .context("Failed to create objects directory")?;

        // Initialize SQLite database
        let db = Connection::open(&db_path)
            .context("Failed to open metadata database")?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS objects (
                id BLOB PRIMARY KEY,
                size INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                accessed_at INTEGER NOT NULL,
                access_count INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )
        .context("Failed to create objects table")?;

        // Create index for LRU/LFU eviction queries
        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_accessed_at ON objects(accessed_at)",
            [],
        )
        .context("Failed to create accessed_at index")?;

        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_access_count ON objects(access_count)",
            [],
        )
        .context("Failed to create access_count index")?;

        Ok(Self {
            objects_dir,
            db: Arc::new(Mutex::new(db)),
        })
    }

    /// Convert blob ID to filesystem path
    /// Uses git-style sharding: first 2 hex chars as subdirectory
    fn id_to_path(&self, id: &[u8]) -> PathBuf {
        let hex_id = hex::encode(id);
        let (prefix, suffix) = hex_id.split_at(2);
        self.objects_dir.join(prefix).join(suffix)
    }

    /// Get current Unix timestamp
    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
}

impl Storage for FilesystemStorage {
    fn put(&self, id: &[u8], data: &[u8]) -> Result<()> {
        let path = self.id_to_path(id);

        // Create parent directory
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create parent directory")?;
        }

        // Write data atomically (write to temp file, then rename)
        let temp_path = path.with_extension("tmp");
        let mut file = fs::File::create(&temp_path)
            .context("Failed to create temp file")?;
        file.write_all(data)
            .context("Failed to write data")?;
        file.sync_all()
            .context("Failed to sync file")?;
        fs::rename(&temp_path, &path)
            .context("Failed to rename temp file")?;

        // Update metadata
        let now = Self::current_timestamp();
        let size = data.len() as i64;
        let db = self.db.lock().unwrap();

        db.execute(
            "INSERT OR REPLACE INTO objects (id, size, created_at, accessed_at, access_count)
             VALUES (?1, ?2, ?3, ?3, COALESCE((SELECT access_count FROM objects WHERE id = ?1), 0))",
            params![id, size, now],
        )
        .context("Failed to update metadata")?;

        Ok(())
    }

    fn get(&self, id: &[u8]) -> Result<Option<Vec<u8>>> {
        let path = self.id_to_path(id);

        if !path.exists() {
            return Ok(None);
        }

        // Read data
        let data = fs::read(&path)
            .context("Failed to read object")?;

        // Update access metadata
        self.touch(id)?;

        Ok(Some(data))
    }

    fn exists(&self, id: &[u8]) -> Result<bool> {
        let path = self.id_to_path(id);
        Ok(path.exists())
    }

    fn delete(&self, id: &[u8]) -> Result<()> {
        let path = self.id_to_path(id);

        // Delete file
        if path.exists() {
            fs::remove_file(&path)
                .context("Failed to delete object")?;
        }

        // Delete metadata
        let db = self.db.lock().unwrap();
        db.execute("DELETE FROM objects WHERE id = ?1", params![id])
            .context("Failed to delete metadata")?;

        Ok(())
    }

    fn size(&self, id: &[u8]) -> Result<Option<u64>> {
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare("SELECT size FROM objects WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let size: i64 = row.get(0)?;
            Ok(Some(size as u64))
        } else {
            Ok(None)
        }
    }

    fn touch(&self, id: &[u8]) -> Result<()> {
        let now = Self::current_timestamp();
        let db = self.db.lock().unwrap();

        db.execute(
            "UPDATE objects SET accessed_at = ?1, access_count = access_count + 1 WHERE id = ?2",
            params![now, id],
        )
        .context("Failed to update access metadata")?;

        Ok(())
    }

    fn list_ids(&self) -> Result<Vec<Vec<u8>>> {
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare("SELECT id FROM objects")?;
        let rows = stmt.query_map([], |row| {
            let id: Vec<u8> = row.get(0)?;
            Ok(id)
        })?;

        let mut ids = Vec::new();
        for id in rows {
            ids.push(id?);
        }

        Ok(ids)
    }

    fn stats(&self) -> Result<StorageStats> {
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare("SELECT COUNT(*), COALESCE(SUM(size), 0) FROM objects")?;
        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            let total_objects: i64 = row.get(0)?;
            let total_bytes: i64 = row.get(1)?;

            Ok(StorageStats {
                total_objects: total_objects as u64,
                total_bytes: total_bytes as u64,
                cache_dir: self.objects_dir.parent().unwrap().to_path_buf(),
            })
        } else {
            Ok(StorageStats {
                total_objects: 0,
                total_bytes: 0,
                cache_dir: self.objects_dir.parent().unwrap().to_path_buf(),
            })
        }
    }
}

/// Hash data using SHA256
#[allow(dead_code)]
pub fn hash_data(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_filesystem_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FilesystemStorage::new(temp_dir.path()).unwrap();

        // Test put and get
        let id = hash_data(b"hello");
        storage.put(&id, b"hello world").unwrap();

        let data = storage.get(&id).unwrap();
        assert_eq!(data, Some(b"hello world".to_vec()));

        // Test exists
        assert!(storage.exists(&id).unwrap());

        // Test size
        assert_eq!(storage.size(&id).unwrap(), Some(11));

        // Test stats
        let stats = storage.stats().unwrap();
        assert_eq!(stats.total_objects, 1);
        assert_eq!(stats.total_bytes, 11);

        // Test delete
        storage.delete(&id).unwrap();
        assert!(!storage.exists(&id).unwrap());
    }
}
