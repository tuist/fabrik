use super::proto::google::rpc::Status as RpcStatus;
use super::proto::remote_execution::*;
use crate::logging::{operations, services, status};
use crate::storage::Storage;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{debug, info};

/// Bazel ContentAddressableStorage service implementation
pub struct BazelCasService<S: Storage> {
    storage: Arc<S>,
}

impl<S: Storage> BazelCasService<S> {
    pub fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }

    /// Generate CAS blob key from digest
    fn cas_blob_key(digest: &Digest) -> Vec<u8> {
        format!("cas:{}:{}", digest.hash, digest.size_bytes).into_bytes()
    }
}

#[tonic::async_trait]
impl<S: Storage + 'static> content_addressable_storage_server::ContentAddressableStorage
    for BazelCasService<S>
{
    type GetTreeStream = tokio_stream::wrappers::ReceiverStream<Result<GetTreeResponse, Status>>;
    async fn find_missing_blobs(
        &self,
        request: Request<FindMissingBlobsRequest>,
    ) -> Result<Response<FindMissingBlobsResponse>, Status> {
        let req = request.into_inner();
        let blob_count = req.blob_digests.len();

        debug!(
            service = services::BAZEL_CAS,
            operation = operations::FIND_MISSING,
            instance = %req.instance_name,
            blob_count,
            "checking blobs"
        );

        let mut missing = Vec::new();

        for digest in req.blob_digests {
            let key = Self::cas_blob_key(&digest);

            // Check if blob exists in storage
            match self.storage.get(&key) {
                Ok(None) | Err(_) => {
                    missing.push(digest);
                }
                Ok(Some(_)) => {
                    // Blob exists, don't add to missing
                }
            }
        }

        info!(
            service = services::BAZEL_CAS,
            operation = operations::FIND_MISSING,
            status = status::SUCCESS,
            instance = %req.instance_name,
            blob_count,
            missing_count = missing.len(),
            "check completed"
        );

        Ok(Response::new(FindMissingBlobsResponse {
            missing_blob_digests: missing,
        }))
    }

    async fn batch_update_blobs(
        &self,
        request: Request<BatchUpdateBlobsRequest>,
    ) -> Result<Response<BatchUpdateBlobsResponse>, Status> {
        let req = request.into_inner();

        debug!(
            "==> BatchUpdateBlobs - instance: {}, uploading {} blobs",
            req.instance_name,
            req.requests.len()
        );

        let mut responses = Vec::new();
        let mut success_count = 0;
        let mut error_count = 0;

        for blob_request in req.requests {
            let digest = blob_request
                .digest
                .ok_or_else(|| Status::invalid_argument("Missing digest"))?;

            let key = Self::cas_blob_key(&digest);

            debug!(
                "  Uploading blob: hash={}, size={}",
                digest.hash, digest.size_bytes
            );

            // Verify digest size matches data size
            if digest.size_bytes != blob_request.data.len() as i64 {
                debug!(
                    "  Size mismatch: expected {}, got {}",
                    digest.size_bytes,
                    blob_request.data.len()
                );
            }

            // Store blob in storage
            let status = match self.storage.put(&key, &blob_request.data) {
                Ok(_) => {
                    success_count += 1;
                    debug!("  Blob stored successfully");
                    RpcStatus {
                        code: 0, // OK
                        message: String::new(),
                        details: Vec::new(),
                    }
                }
                Err(e) => {
                    error_count += 1;
                    debug!("  Failed to store blob: {}", e);
                    RpcStatus {
                        code: 13, // INTERNAL
                        message: format!("Failed to store blob: {}", e),
                        details: Vec::new(),
                    }
                }
            };

            responses.push(batch_update_blobs_response::Response {
                digest: Some(digest),
                status: Some(status),
            });
        }

        info!(
            "<== BatchUpdateBlobs - Success: {}, Errors: {}",
            success_count, error_count
        );

        Ok(Response::new(BatchUpdateBlobsResponse { responses }))
    }

    async fn batch_read_blobs(
        &self,
        request: Request<BatchReadBlobsRequest>,
    ) -> Result<Response<BatchReadBlobsResponse>, Status> {
        let req = request.into_inner();

        debug!(
            "==> BatchReadBlobs - instance: {}, reading {} blobs",
            req.instance_name,
            req.digests.len()
        );

        let mut responses = Vec::new();

        for digest in req.digests {
            let key = Self::cas_blob_key(&digest);

            // Retrieve blob from storage
            let (data, status) = match self.storage.get(&key) {
                Ok(Some(blob_data)) => (
                    blob_data,
                    RpcStatus {
                        code: 0, // OK
                        message: String::new(),
                        details: Vec::new(),
                    },
                ),
                Ok(None) | Err(_) => (
                    Vec::new(),
                    RpcStatus {
                        code: 5, // NOT_FOUND
                        message: format!("Blob not found: {}", digest.hash),
                        details: Vec::new(),
                    },
                ),
            };

            responses.push(batch_read_blobs_response::Response {
                digest: Some(digest),
                data,
                compressor: 0, // IDENTITY = 0 (no compression)
                status: Some(status),
            });
        }

        info!(
            "<== BatchReadBlobs - Retrieved {} blobs successfully",
            responses
                .iter()
                .filter(|r| r.status.as_ref().map(|s| s.code == 0).unwrap_or(false))
                .count()
        );

        Ok(Response::new(BatchReadBlobsResponse { responses }))
    }

    async fn get_tree(
        &self,
        _request: Request<GetTreeRequest>,
    ) -> Result<Response<Self::GetTreeStream>, Status> {
        // GetTree is not required for basic caching - return unimplemented
        Err(Status::unimplemented("GetTree is not yet implemented"))
    }
}
