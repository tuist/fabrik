//! Fabrik Cache gRPC service implementation
//!
//! This service implements the FabrikCache protocol for Layer 2 servers.
//! It handles Exists, Get, Put, Delete, and GetStats operations.

use crate::fabrik_protocol::proto::fabrik_cache_server::FabrikCache;
use crate::fabrik_protocol::proto::{
    DeleteRequest, DeleteResponse, ExistsRequest, ExistsResponse, GetRequest, GetResponse,
    GetStatsRequest, GetStatsResponse, PutRequest, PutResponse,
};
use crate::storage::{FilesystemStorage, Storage};
use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::{debug, info, warn};

/// Fabrik Cache gRPC service
pub struct FabrikCacheService {
    storage: Arc<FilesystemStorage>,
}

impl FabrikCacheService {
    /// Create a new Fabrik cache service
    pub fn new(storage: Arc<FilesystemStorage>) -> Self {
        Self { storage }
    }

    /// Convert hash string to bytes (hex decode)
    fn hash_to_bytes(hash: &str) -> Result<Vec<u8>, Status> {
        hex::decode(hash)
            .map_err(|e| Status::invalid_argument(format!("Invalid hash format: {}", e)))
    }
}

#[tonic::async_trait]
impl FabrikCache for FabrikCacheService {
    /// Check if an artifact exists in the cache
    async fn exists(
        &self,
        request: Request<ExistsRequest>,
    ) -> Result<Response<ExistsResponse>, Status> {
        let req = request.into_inner();
        let hash_bytes = Self::hash_to_bytes(&req.hash)?;

        debug!(
            "Exists request for hash: {}...",
            &req.hash[..8.min(req.hash.len())]
        );

        match self.storage.exists(&hash_bytes) {
            Ok(exists) => {
                let size_bytes = if exists {
                    // Touch for LRU tracking
                    let _ = self.storage.touch(&hash_bytes);
                    self.storage.size(&hash_bytes).unwrap_or(None).unwrap_or(0) as i64
                } else {
                    0
                };

                Ok(Response::new(ExistsResponse {
                    exists,
                    size_bytes,
                    metadata: std::collections::HashMap::new(),
                }))
            }
            Err(e) => {
                warn!("Storage error in exists: {}", e);
                Err(Status::internal(format!("Storage error: {}", e)))
            }
        }
    }

    type GetStream = ReceiverStream<Result<GetResponse, Status>>;

    /// Retrieve an artifact from the cache (streaming)
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<Self::GetStream>, Status> {
        let req = request.into_inner();
        let hash_bytes = Self::hash_to_bytes(&req.hash)?;
        let hash_display = req.hash[..8.min(req.hash.len())].to_string();

        debug!("Get request for hash: {}...", hash_display);

        // Check if artifact exists
        let data = match self.storage.get(&hash_bytes) {
            Ok(Some(data)) => {
                // Touch for LRU tracking
                let _ = self.storage.touch(&hash_bytes);
                data
            }
            Ok(None) => {
                debug!("Artifact not found: {}...", hash_display);
                return Err(Status::not_found(format!(
                    "Artifact not found: {}",
                    req.hash
                )));
            }
            Err(e) => {
                warn!("Storage error in get: {}", e);
                return Err(Status::internal(format!("Storage error: {}", e)));
            }
        };

        let total_size = data.len();
        info!(
            "Serving artifact: {}... ({} bytes)",
            hash_display, total_size
        );

        // Create channel for streaming response
        let (tx, rx) = tokio::sync::mpsc::channel(128);

        // Stream data in chunks (64KB chunks for efficiency)
        tokio::spawn(async move {
            const CHUNK_SIZE: usize = 64 * 1024;
            let mut first = true;

            for chunk in data.chunks(CHUNK_SIZE) {
                let response = GetResponse {
                    chunk: chunk.to_vec(),
                    metadata: if first {
                        first = false;
                        std::collections::HashMap::new()
                    } else {
                        std::collections::HashMap::new()
                    },
                };

                if tx.send(Ok(response)).await.is_err() {
                    // Client disconnected
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    /// Store an artifact in the cache (streaming)
    async fn put(
        &self,
        request: Request<Streaming<PutRequest>>,
    ) -> Result<Response<PutResponse>, Status> {
        let mut stream = request.into_inner();

        let mut hash: Option<String> = None;
        let mut data = Vec::new();

        // Collect all chunks
        while let Some(req) = stream
            .message()
            .await
            .map_err(|e| Status::internal(format!("Stream error: {}", e)))?
        {
            // First message contains the hash
            if hash.is_none() && !req.hash.is_empty() {
                hash = Some(req.hash);
            }

            // Append chunk data
            data.extend(req.chunk);
        }

        let hash = hash.ok_or_else(|| Status::invalid_argument("Hash not provided in stream"))?;
        let hash_bytes = Self::hash_to_bytes(&hash)?;
        let size = data.len();

        debug!(
            "Put request for hash: {}... ({} bytes)",
            &hash[..8.min(hash.len())],
            size
        );

        // Store the artifact
        match self.storage.put(&hash_bytes, &data) {
            Ok(()) => {
                info!(
                    "Stored artifact: {}... ({} bytes)",
                    &hash[..8.min(hash.len())],
                    size
                );
                Ok(Response::new(PutResponse {
                    success: true,
                    size_bytes: size as i64,
                }))
            }
            Err(e) => {
                warn!("Storage error in put: {}", e);
                Err(Status::internal(format!("Storage error: {}", e)))
            }
        }
    }

    /// Delete an artifact from the cache
    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let req = request.into_inner();
        let hash_bytes = Self::hash_to_bytes(&req.hash)?;

        debug!(
            "Delete request for hash: {}...",
            &req.hash[..8.min(req.hash.len())]
        );

        // Check if exists before delete
        let existed = self.storage.exists(&hash_bytes).unwrap_or(false);

        match self.storage.delete(&hash_bytes) {
            Ok(()) => {
                if existed {
                    info!(
                        "Deleted artifact: {}...",
                        &req.hash[..8.min(req.hash.len())]
                    );
                }
                Ok(Response::new(DeleteResponse {
                    success: true,
                    existed,
                }))
            }
            Err(e) => {
                warn!("Storage error in delete: {}", e);
                Err(Status::internal(format!("Storage error: {}", e)))
            }
        }
    }

    /// Get cache statistics
    async fn get_stats(
        &self,
        _request: Request<GetStatsRequest>,
    ) -> Result<Response<GetStatsResponse>, Status> {
        debug!("GetStats request");

        let stats = match self.storage.stats() {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to get storage stats: {}", e);
                return Err(Status::internal(format!("Failed to get stats: {}", e)));
            }
        };

        Ok(Response::new(GetStatsResponse {
            cache_hits: 0,
            cache_misses: 0,
            artifact_count: stats.total_objects,
            total_bytes: stats.total_bytes,
            uptime_seconds: 0,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, Arc<FilesystemStorage>) {
        let temp_dir = TempDir::new().unwrap();
        let storage = FilesystemStorage::new(temp_dir.path().to_str().unwrap()).unwrap();
        (temp_dir, Arc::new(storage))
    }

    #[tokio::test]
    async fn test_exists_not_found() {
        let (_temp, storage) = create_test_storage();
        let service = FabrikCacheService::new(storage);

        let request = Request::new(ExistsRequest {
            hash: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        });

        let response = service.exists(request).await.unwrap();
        assert!(!response.into_inner().exists);
    }

    #[tokio::test]
    async fn test_put_and_exists() {
        let (_temp, storage) = create_test_storage();
        let service = FabrikCacheService::new(storage.clone());

        // First, manually put data in storage
        let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let hash_bytes = hex::decode(hash).unwrap();
        storage.put(&hash_bytes, b"test data").unwrap();

        // Now check exists
        let request = Request::new(ExistsRequest {
            hash: hash.to_string(),
        });

        let response = service.exists(request).await.unwrap();
        let inner = response.into_inner();
        assert!(inner.exists);
        assert_eq!(inner.size_bytes, 9); // "test data" is 9 bytes
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let (_temp, storage) = create_test_storage();
        let service = FabrikCacheService::new(storage);

        let request = Request::new(GetRequest {
            hash: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        });

        let result = service.get(request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn test_delete() {
        let (_temp, storage) = create_test_storage();
        let service = FabrikCacheService::new(storage.clone());

        // Put data first
        let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let hash_bytes = hex::decode(hash).unwrap();
        storage.put(&hash_bytes, b"test data").unwrap();

        // Delete it
        let request = Request::new(DeleteRequest {
            hash: hash.to_string(),
        });

        let response = service.delete(request).await.unwrap();
        let inner = response.into_inner();
        assert!(inner.success);
        assert!(inner.existed);

        // Verify it's gone
        assert!(!storage.exists(&hash_bytes).unwrap());
    }

    #[tokio::test]
    async fn test_get_stats() {
        let (_temp, storage) = create_test_storage();
        let service = FabrikCacheService::new(storage);

        let request = Request::new(GetStatsRequest {
            since_timestamp: None,
        });

        let response = service.get_stats(request).await.unwrap();
        let inner = response.into_inner();
        assert_eq!(inner.cache_hits, 0);
        assert_eq!(inner.cache_misses, 0);
        // uptime_seconds is u64, so it's always >= 0
        let _ = inner.uptime_seconds;
    }
}
