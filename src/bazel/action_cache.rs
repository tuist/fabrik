use super::proto::remote_execution::*;
use crate::storage::Storage;
use prost::Message;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{debug, info};

/// Bazel ActionCache service implementation
pub struct BazelActionCacheService<S: Storage> {
    storage: Arc<S>,
}

impl<S: Storage> BazelActionCacheService<S> {
    pub fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }

    /// Generate cache key from action digest and instance name
    fn action_cache_key(instance_name: &str, digest: &Digest) -> Vec<u8> {
        format!("action_cache:{}:{}:{}", instance_name, digest.hash, digest.size_bytes).into_bytes()
    }

    /// Serialize ActionResult to bytes
    fn serialize_result(result: &ActionResult) -> Result<Vec<u8>, Status> {
        let mut buf = Vec::new();
        result
            .encode(&mut buf)
            .map_err(|e| Status::internal(format!("Failed to serialize ActionResult: {}", e)))?;
        Ok(buf)
    }

    /// Deserialize bytes to ActionResult
    fn deserialize_result(data: &[u8]) -> Result<ActionResult, Status> {
        ActionResult::decode(data)
            .map_err(|e| Status::internal(format!("Failed to deserialize ActionResult: {}", e)))
    }
}

#[tonic::async_trait]
impl<S: Storage + 'static> action_cache_server::ActionCache for BazelActionCacheService<S> {
    async fn get_action_result(
        &self,
        request: Request<GetActionResultRequest>,
    ) -> Result<Response<ActionResult>, Status> {
        let req = request.into_inner();

        debug!(
            "==> GetActionResult - instance: {}, digest: {}",
            req.instance_name,
            req.action_digest.as_ref().map(|d| &d.hash).unwrap_or(&String::new())
        );

        let digest = req
            .action_digest
            .ok_or_else(|| Status::invalid_argument("Missing action_digest"))?;

        let key = Self::action_cache_key(&req.instance_name, &digest);

        // Retrieve from storage
        match self.storage.get(&key) {
            Ok(Some(data)) => {
                let result = Self::deserialize_result(&data)?;

                info!(
                    "<== GetActionResult - Cache HIT for action {}",
                    digest.hash
                );

                Ok(Response::new(result))
            }
            Ok(None) | Err(_) => {
                debug!("<== GetActionResult - Cache MISS for action {}", digest.hash);
                Err(Status::not_found("ActionResult not found in cache"))
            }
        }
    }

    async fn update_action_result(
        &self,
        request: Request<UpdateActionResultRequest>,
    ) -> Result<Response<ActionResult>, Status> {
        let req = request.into_inner();

        debug!(
            "==> UpdateActionResult - instance: {}, digest: {}",
            req.instance_name,
            req.action_digest.as_ref().map(|d| &d.hash).unwrap_or(&String::new())
        );

        let digest = req
            .action_digest
            .ok_or_else(|| Status::invalid_argument("Missing action_digest"))?;

        let result = req
            .action_result
            .ok_or_else(|| Status::invalid_argument("Missing action_result"))?;

        let key = Self::action_cache_key(&req.instance_name, &digest);
        let serialized = Self::serialize_result(&result)?;

        // Store in storage
        self.storage
            .put(&key, &serialized)
            .map_err(|e| Status::internal(format!("Failed to store ActionResult: {}", e)))?;

        info!(
            "<== UpdateActionResult - Stored action result for {}",
            digest.hash
        );

        Ok(Response::new(result))
    }
}
