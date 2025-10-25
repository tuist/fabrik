use super::proto::bytestream::*;
use crate::storage::Storage;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{debug, info, warn};

/// ByteStream service implementation for large blob transfers
pub struct BazelByteStreamService<S: Storage> {
    storage: Arc<S>,
}

impl<S: Storage> BazelByteStreamService<S> {
    pub fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }

    /// Parse resource name to extract hash and size
    /// Format: [instance_name/]uploads/[uuid]/blobs/{hash}/{size}
    /// or: [instance_name/]blobs/{hash}/{size}
    fn parse_resource_name(resource_name: &str) -> Option<(String, i64)> {
        let parts: Vec<&str> = resource_name.split('/').collect();

        // Find "blobs" in the path
        if let Some(blobs_idx) = parts.iter().position(|&p| p == "blobs") {
            if blobs_idx + 2 < parts.len() {
                let hash = parts[blobs_idx + 1].to_string();
                if let Ok(size) = parts[blobs_idx + 2].parse::<i64>() {
                    return Some((hash, size));
                }
            }
        }

        None
    }

    /// Generate CAS blob key from hash and size
    fn cas_blob_key(hash: &str, size: i64) -> Vec<u8> {
        format!("cas:{}:{}", hash, size).into_bytes()
    }
}

#[tonic::async_trait]
impl<S: Storage + 'static> byte_stream_server::ByteStream for BazelByteStreamService<S> {
    type ReadStream = tokio_stream::wrappers::ReceiverStream<Result<ReadResponse, Status>>;

    async fn read(
        &self,
        request: Request<ReadRequest>,
    ) -> Result<Response<Self::ReadStream>, Status> {
        let req = request.into_inner();

        debug!("==> ByteStream Read - resource: {}", req.resource_name);

        let (hash, size) = Self::parse_resource_name(&req.resource_name)
            .ok_or_else(|| Status::invalid_argument("Invalid resource name format"))?;

        let key = Self::cas_blob_key(&hash, size);

        // Retrieve blob from storage
        let data = match self.storage.get(&key) {
            Ok(Some(blob_data)) => blob_data,
            Ok(None) => {
                return Err(Status::not_found(format!("Blob not found: {}", hash)));
            }
            Err(e) => {
                return Err(Status::internal(format!("Storage error: {}", e)));
            }
        };

        // Create streaming response
        let (tx, rx) = tokio::sync::mpsc::channel(4);
        let data_len = data.len();

        tokio::spawn(async move {
            let chunk_size = 1024 * 1024; // 1MB chunks
            let offset = req.read_offset as usize;
            let limit = if req.read_limit > 0 {
                Some(req.read_limit as usize)
            } else {
                None
            };

            let end = if let Some(limit) = limit {
                std::cmp::min(offset + limit, data.len())
            } else {
                data.len()
            };

            let mut current = offset;
            while current < end {
                let chunk_end = std::cmp::min(current + chunk_size, end);
                let chunk = data[current..chunk_end].to_vec();

                if tx.send(Ok(ReadResponse { data: chunk })).await.is_err() {
                    break;
                }

                current = chunk_end;
            }
        });

        info!("<== ByteStream Read - streaming {} bytes", data_len);

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }

    async fn write(
        &self,
        request: Request<tonic::Streaming<WriteRequest>>,
    ) -> Result<Response<WriteResponse>, Status> {
        let mut stream = request.into_inner();

        let mut resource_name: Option<String> = None;
        let mut buffer = Vec::new();
        let mut total_written = 0usize;

        debug!("==> ByteStream Write");

        while let Some(req) = stream.message().await? {
            // First message should have resource_name
            if resource_name.is_none() {
                if req.resource_name.is_empty() {
                    return Err(Status::invalid_argument(
                        "Missing resource_name in first WriteRequest",
                    ));
                }
                resource_name = Some(req.resource_name.clone());
                debug!("  Resource: {}", req.resource_name);
            }

            // Verify write_offset matches our current position
            if req.write_offset != total_written as i64 {
                return Err(Status::invalid_argument(format!(
                    "Write offset mismatch: expected {}, got {}",
                    total_written, req.write_offset
                )));
            }

            // Append data to buffer
            buffer.extend_from_slice(&req.data);
            total_written += req.data.len();

            // If this is the final write, store in storage
            if req.finish_write {
                let resource = resource_name
                    .as_ref()
                    .ok_or_else(|| Status::internal("Missing resource_name"))?;

                let (hash, size) = Self::parse_resource_name(resource)
                    .ok_or_else(|| Status::invalid_argument("Invalid resource name format"))?;

                // Verify size matches
                if size != buffer.len() as i64 {
                    warn!(
                        "Size mismatch: resource claims {}, got {}",
                        size,
                        buffer.len()
                    );
                }

                let key = Self::cas_blob_key(&hash, size);

                // Store in storage
                self.storage
                    .put(&key, &buffer)
                    .map_err(|e| Status::internal(format!("Failed to store blob: {}", e)))?;

                info!(
                    "<== ByteStream Write - Stored {} bytes for hash {}",
                    buffer.len(),
                    hash
                );

                return Ok(Response::new(WriteResponse {
                    committed_size: buffer.len() as i64,
                }));
            }
        }

        // If we get here, finish_write was never set
        Err(Status::invalid_argument(
            "Stream ended without finish_write",
        ))
    }

    async fn query_write_status(
        &self,
        _request: Request<QueryWriteStatusRequest>,
    ) -> Result<Response<QueryWriteStatusResponse>, Status> {
        // We don't support resumable uploads yet
        Err(Status::unimplemented("QueryWriteStatus is not implemented"))
    }
}
