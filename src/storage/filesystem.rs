use super::{Storage, StorageStats};
use anyhow::{Context, Result};
use crossbeam_channel::{bounded, Sender};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
use rusqlite_migration::{Migrations, M};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::debug;

/// Database migrations for the cache metadata
///
/// Migrations are run automatically on storage initialization.
/// The migration version is tracked using SQLite's `user_version` pragma.
///
/// ## Adding a new migration
///
/// To add a new migration, append a new `M::up()` to the vector:
///
/// ```ignore
/// M::up("ALTER TABLE objects ADD COLUMN ttl INTEGER;"),
/// ```
///
/// **Important**: Never modify existing migrations. Always add new ones to the end.
///
/// ## Example migration sequence
///
/// ```ignore
/// vec![
///     M::up("CREATE TABLE objects (...);"),           // Migration 1
///     M::up("ALTER TABLE objects ADD COLUMN ttl;"),   // Migration 2
///     M::up("CREATE INDEX idx_ttl ON objects(ttl);"), // Migration 3
/// ]
/// ```
fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        // Migration 1: Initial schema
        M::up(
            "CREATE TABLE objects (
                id BLOB PRIMARY KEY,
                size INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                accessed_at INTEGER NOT NULL,
                access_count INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX idx_accessed_at ON objects(accessed_at);
            CREATE INDEX idx_access_count ON objects(access_count);",
        ),
        // Future migrations go here:
        // M::up("ALTER TABLE objects ADD COLUMN ttl INTEGER;"),
    ])
}

/// Message type for batched access tracking updates
#[derive(Debug, Clone)]
struct TouchMessage {
    id: Vec<u8>,
    timestamp: i64,
}

/// Filesystem-based storage with SQLite metadata tracking
///
/// Layout:
/// - `.fabrik/cache/objects/ab/cd1234...` - Content-addressed blob storage (first 2 chars = subdir)
/// - `.fabrik/cache/metadata.db` - SQLite database for access tracking and eviction
///
/// Optimizations:
/// - Connection pool for concurrent database access
/// - Async batched access tracking (touch operations)
/// - WAL mode for better read/write concurrency
pub struct FilesystemStorage {
    objects_dir: PathBuf,
    db_pool: Pool<SqliteConnectionManager>,
    touch_sender: Sender<TouchMessage>,
}

impl FilesystemStorage {
    /// Create a new filesystem storage at the given cache directory
    ///
    /// Runs database migrations to ensure the schema is up to date.
    /// Spawns a background worker for batched access tracking.
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self> {
        let cache_dir = cache_dir.as_ref();
        let objects_dir = cache_dir.join("objects");
        let db_path = cache_dir.join("metadata.db");

        // Create directories
        fs::create_dir_all(&objects_dir).context("Failed to create objects directory")?;

        // Initialize database for migrations
        let mut init_db = Connection::open(&db_path).context("Failed to open metadata database")?;

        // Enable WAL mode for better concurrency (multiple readers, single writer)
        init_db
            .pragma_update(None, "journal_mode", "WAL")
            .context("Failed to enable WAL mode")?;

        // Set busy timeout to 5 seconds to handle lock contention
        init_db
            .pragma_update(None, "busy_timeout", "5000")
            .context("Failed to set busy timeout")?;

        // Run migrations
        migrations()
            .to_latest(&mut init_db)
            .context("Failed to run database migrations")?;

        // Drop initial connection before creating pool
        drop(init_db);

        // Create connection pool (max 16 connections for high concurrency)
        let manager = SqliteConnectionManager::file(&db_path).with_init(|conn| {
            // Ensure WAL mode for all connections
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "busy_timeout", "5000")?;
            Ok(())
        });

        let db_pool = Pool::builder()
            .max_size(16)
            .build(manager)
            .context("Failed to create connection pool")?;

        // Create channel for async touch operations (buffered for batching)
        let (touch_sender, touch_receiver) = bounded::<TouchMessage>(1000);

        // Spawn background worker for batched access tracking
        let pool_clone = db_pool.clone();
        thread::spawn(move || {
            let mut batch = Vec::with_capacity(100);
            let batch_timeout = Duration::from_millis(100);

            loop {
                // Collect messages for up to 100ms or 100 items
                match touch_receiver.recv_timeout(batch_timeout) {
                    Ok(msg) => {
                        batch.push(msg);

                        // Drain the channel up to 100 items
                        while batch.len() < 100 {
                            match touch_receiver.try_recv() {
                                Ok(msg) => batch.push(msg),
                                Err(_) => break,
                            }
                        }

                        // Execute batch update
                        if let Err(e) = Self::batch_touch(&pool_clone, &batch) {
                            debug!("Failed to batch update access tracking: {}", e);
                        }

                        batch.clear();
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                        // Flush any pending items on timeout
                        if !batch.is_empty() {
                            if let Err(e) = Self::batch_touch(&pool_clone, &batch) {
                                debug!("Failed to batch update access tracking: {}", e);
                            }
                            batch.clear();
                        }
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                        // Channel closed, flush and exit
                        if !batch.is_empty() {
                            let _ = Self::batch_touch(&pool_clone, &batch);
                        }
                        break;
                    }
                }
            }
        });

        Ok(Self {
            objects_dir,
            db_pool,
            touch_sender,
        })
    }

    /// Batch update access tracking for multiple objects
    fn batch_touch(pool: &Pool<SqliteConnectionManager>, batch: &[TouchMessage]) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        let conn = pool.get().context("Failed to get database connection")?;

        // Use a transaction for batched updates
        let tx = conn.unchecked_transaction()?;

        for msg in batch {
            tx.execute(
                "UPDATE objects SET accessed_at = ?1, access_count = access_count + 1 WHERE id = ?2",
                params![msg.timestamp, msg.id],
            ).ok(); // Ignore errors for individual updates
        }

        tx.commit()
            .context("Failed to commit batch touch transaction")?;
        debug!("Batched {} access tracking updates", batch.len());

        Ok(())
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
            fs::create_dir_all(parent).context("Failed to create parent directory")?;
        }

        // Write data atomically (write to temp file, then rename)
        // Use PID + thread ID to avoid collisions in concurrent writes
        let temp_name = format!(
            "{}.tmp.{}.{:?}",
            path.file_name().unwrap().to_str().unwrap(),
            std::process::id(),
            thread::current().id()
        );
        let temp_path = path.parent().unwrap().join(temp_name);

        let mut file = fs::File::create(&temp_path).context("Failed to create temp file")?;
        file.write_all(data).context("Failed to write data")?;
        file.sync_all().context("Failed to sync file")?;
        fs::rename(&temp_path, &path).context("Failed to rename temp file")?;

        // Update metadata using connection pool
        let now = Self::current_timestamp();
        let size = data.len() as i64;
        let conn = self
            .db_pool
            .get()
            .context("Failed to get database connection")?;

        conn.execute(
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
        let data = fs::read(&path).context("Failed to read object")?;

        // Update access metadata asynchronously (non-blocking)
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
            fs::remove_file(&path).context("Failed to delete object")?;
        }

        // Delete metadata
        let conn = self
            .db_pool
            .get()
            .context("Failed to get database connection")?;
        conn.execute("DELETE FROM objects WHERE id = ?1", params![id])
            .context("Failed to delete metadata")?;

        Ok(())
    }

    fn size(&self, id: &[u8]) -> Result<Option<u64>> {
        let conn = self
            .db_pool
            .get()
            .context("Failed to get database connection")?;
        let mut stmt = conn.prepare("SELECT size FROM objects WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let size: i64 = row.get(0)?;
            Ok(Some(size as u64))
        } else {
            Ok(None)
        }
    }

    fn touch(&self, id: &[u8]) -> Result<()> {
        // Send to async batch worker (non-blocking)
        let msg = TouchMessage {
            id: id.to_vec(),
            timestamp: Self::current_timestamp(),
        };

        // Use try_send to avoid blocking if channel is full
        // If channel is full, we simply drop the update (acceptable trade-off for performance)
        self.touch_sender.try_send(msg).ok();

        Ok(())
    }

    fn list_ids(&self) -> Result<Vec<Vec<u8>>> {
        let conn = self
            .db_pool
            .get()
            .context("Failed to get database connection")?;
        let mut stmt = conn.prepare("SELECT id FROM objects")?;
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
        let conn = self
            .db_pool
            .get()
            .context("Failed to get database connection")?;
        let mut stmt = conn.prepare("SELECT COUNT(*), COALESCE(SUM(size), 0) FROM objects")?;
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
