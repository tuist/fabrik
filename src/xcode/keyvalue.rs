use super::proto::keyvalue::*;
use crate::logging::{operations, services, status};
use crate::storage::Storage;
use prost::Message;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{debug, info};

/// KeyValue database service implementation
/// Maps build keys to cached value maps
pub struct KeyValueService<S: Storage> {
    storage: Arc<S>,
}

impl<S: Storage> KeyValueService<S> {
    pub fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }

    /// Serialize a Value to bytes
    #[allow(clippy::result_large_err)]
    fn serialize_value(value: &Value) -> Result<Vec<u8>, Status> {
        let mut buf = Vec::new();
        value
            .encode(&mut buf)
            .map_err(|e| Status::internal(format!("Failed to serialize value: {}", e)))?;
        Ok(buf)
    }

    /// Deserialize bytes to Value
    #[allow(clippy::result_large_err)]
    fn deserialize_value(data: &[u8]) -> Result<Value, Status> {
        Value::decode(data)
            .map_err(|e| Status::internal(format!("Failed to deserialize value: {}", e)))
    }

    /// Create a key for the KeyValue store with a prefix to avoid collision with CAS objects
    fn storage_key(key: &[u8]) -> Vec<u8> {
        let mut prefixed = Vec::with_capacity(key.len() + 3);
        prefixed.extend_from_slice(b"kv:");
        prefixed.extend_from_slice(key);
        prefixed
    }
}

#[tonic::async_trait]
impl<S: Storage + 'static> super::proto::keyvalue::key_value_db_server::KeyValueDb
    for KeyValueService<S>
{
    async fn put_value(
        &self,
        request: Request<PutValueRequest>,
    ) -> Result<Response<PutValueResponse>, Status> {
        let req = request.into_inner();
        let key = hex::encode(&req.key);

        let value = req
            .value
            .ok_or_else(|| Status::invalid_argument("Missing value"))?;

        let entry_count = value.entries.len();

        debug!(
            service = services::XCODE_KEYVALUE,
            operation = operations::PUT,
            key = %key,
            entry_count,
            "storing value"
        );

        // Serialize the value
        let serialized = Self::serialize_value(&value)?;

        // Store with prefixed key
        let storage_key = Self::storage_key(&req.key);
        self.storage
            .put(&storage_key, &serialized)
            .map_err(|e| Status::internal(format!("Failed to store value: {}", e)))?;

        info!(
            service = services::XCODE_KEYVALUE,
            operation = operations::PUT,
            status = status::SUCCESS,
            key = %key,
            entry_count,
            size_bytes = serialized.len(),
            "value stored"
        );

        Ok(Response::new(PutValueResponse { error: None }))
    }

    async fn get_value(
        &self,
        request: Request<GetValueRequest>,
    ) -> Result<Response<GetValueResponse>, Status> {
        let req = request.into_inner();
        let key = hex::encode(&req.key);

        debug!(
            service = services::XCODE_KEYVALUE,
            operation = operations::GET,
            key = %key,
            "retrieving value"
        );

        // Retrieve with prefixed key
        let storage_key = Self::storage_key(&req.key);
        let data = self
            .storage
            .get(&storage_key)
            .map_err(|e| Status::internal(format!("Failed to retrieve value: {}", e)))?;

        match data {
            Some(bytes) => {
                // Deserialize the value
                let value = Self::deserialize_value(&bytes)?;

                info!(
                    service = services::XCODE_KEYVALUE,
                    operation = operations::GET,
                    status = status::SUCCESS,
                    key = %key,
                    entry_count = value.entries.len(),
                    size_bytes = bytes.len(),
                    "cache hit"
                );

                Ok(Response::new(GetValueResponse {
                    outcome: get_value_response::Outcome::Success as i32,
                    contents: Some(get_value_response::Contents::Value(value)),
                }))
            }
            None => {
                info!(
                    service = services::XCODE_KEYVALUE,
                    operation = operations::GET,
                    status = status::MISS,
                    key = %key,
                    "cache miss"
                );
                Ok(Response::new(GetValueResponse {
                    outcome: get_value_response::Outcome::KeyNotFound as i32,
                    contents: None,
                }))
            }
        }
    }
}
