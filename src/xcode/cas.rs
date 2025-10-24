use super::proto::cas::*;
use crate::storage::Storage;
use anyhow::Result;
use prost::Message;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{debug, info};

/// CAS (Content-Addressable Storage) service implementation
pub struct CasService<S: Storage> {
    storage: Arc<S>,
}

impl<S: Storage> CasService<S> {
    pub fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }

    /// Serialize a CASObject to bytes
    fn serialize_object(object: &CasObject) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        object.encode(&mut buf)?;
        Ok(buf)
    }

    /// Deserialize bytes to CASObject
    fn deserialize_object(data: &[u8]) -> Result<CasObject> {
        Ok(CasObject::decode(data)?)
    }

    /// Serialize a CASBlob to bytes
    fn serialize_blob(blob: &CasBlob) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        blob.encode(&mut buf)?;
        Ok(buf)
    }

    /// Deserialize bytes to CASBlob
    fn deserialize_blob(data: &[u8]) -> Result<CasBlob> {
        Ok(CasBlob::decode(data)?)
    }

    /// Extract bytes from CASBytes (either inline data or file path)
    #[allow(dead_code)]
    fn extract_cas_bytes(cas_bytes: Option<CasBytes>) -> Result<Vec<u8>> {
        match cas_bytes {
            Some(CasBytes {
                contents: Some(cas_bytes::Contents::Data(data)),
            }) => Ok(data),
            Some(CasBytes {
                contents: Some(cas_bytes::Contents::FilePath(path)),
            }) => {
                // Read from file path
                std::fs::read(&path)
                    .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", path, e))
            }
            _ => Err(anyhow::anyhow!("Invalid CASBytes: missing contents")),
        }
    }

    /// Create CASBytes from data (inline)
    #[allow(dead_code)]
    fn create_cas_bytes(data: Vec<u8>) -> CasBytes {
        CasBytes {
            contents: Some(cas_bytes::Contents::Data(data)),
        }
    }
}

#[tonic::async_trait]
impl<S: Storage + 'static> super::proto::cas::casdb_service_server::CasdbService for CasService<S> {
    async fn put(
        &self,
        request: Request<CasPutRequest>,
    ) -> Result<Response<CasPutResponse>, Status> {
        debug!("==> CAS Put request received");
        let req = request.into_inner();

        let object = req.data.ok_or_else(|| {
            Status::invalid_argument("Missing CASObject data")
        })?;

        // Serialize the object
        let serialized = Self::serialize_object(&object)
            .map_err(|e| Status::internal(format!("Failed to serialize object: {}", e)))?;

        // Compute content hash (ID)
        let id = crate::storage::filesystem::hash_data(&serialized);

        // Store in storage
        self.storage
            .put(&id, &serialized)
            .map_err(|e| Status::internal(format!("Failed to store object: {}", e)))?;

        info!("<== CAS Put completed - Stored object with ID: {}", hex::encode(&id));

        Ok(Response::new(CasPutResponse {
            contents: Some(cas_put_response::Contents::CasId(CasDataId {
                id: id.clone(),
            })),
        }))
    }

    async fn get(
        &self,
        request: Request<CasGetRequest>,
    ) -> Result<Response<CasGetResponse>, Status> {
        let req = request.into_inner();
        let cas_id = req.cas_id.ok_or_else(|| {
            Status::invalid_argument("Missing CAS ID")
        })?;

        debug!("==> CAS Get request for ID: {}", hex::encode(&cas_id.id));

        // Retrieve from storage
        let data = self.storage
            .get(&cas_id.id)
            .map_err(|e| Status::internal(format!("Failed to retrieve object: {}", e)))?;

        match data {
            Some(bytes) => {
                // Deserialize the object
                let object = Self::deserialize_object(&bytes)
                    .map_err(|e| Status::internal(format!("Failed to deserialize object: {}", e)))?;

                info!("<== CAS Get completed - Retrieved object with ID: {}", hex::encode(&cas_id.id));

                Ok(Response::new(CasGetResponse {
                    outcome: cas_get_response::Outcome::Success as i32,
                    contents: Some(cas_get_response::Contents::Data(object)),
                }))
            }
            None => {
                debug!("<== CAS Get completed - Object not found: {}", hex::encode(&cas_id.id));
                Ok(Response::new(CasGetResponse {
                    outcome: cas_get_response::Outcome::ObjectNotFound as i32,
                    contents: None,
                }))
            }
        }
    }

    async fn save(
        &self,
        request: Request<CasSaveRequest>,
    ) -> Result<Response<CasSaveResponse>, Status> {
        let req = request.into_inner();
        debug!("==> CAS Save request received");

        let blob = req.data.ok_or_else(|| {
            Status::invalid_argument("Missing CASBlob data")
        })?;

        // Serialize the blob
        let serialized = Self::serialize_blob(&blob)
            .map_err(|e| Status::internal(format!("Failed to serialize blob: {}", e)))?;

        // Compute content hash (ID)
        let id = crate::storage::filesystem::hash_data(&serialized);

        // Store in storage
        self.storage
            .put(&id, &serialized)
            .map_err(|e| Status::internal(format!("Failed to store blob: {}", e)))?;

        info!("<== CAS Save completed - Saved blob with ID: {}", hex::encode(&id));

        Ok(Response::new(CasSaveResponse {
            contents: Some(cas_save_response::Contents::CasId(CasDataId {
                id: id.clone(),
            })),
        }))
    }

    async fn load(
        &self,
        request: Request<CasLoadRequest>,
    ) -> Result<Response<CasLoadResponse>, Status> {
        let req = request.into_inner();
        let cas_id = req.cas_id.ok_or_else(|| {
            Status::invalid_argument("Missing CAS ID")
        })?;

        debug!("==> CAS Load request for ID: {}", hex::encode(&cas_id.id));

        // Retrieve from storage
        let data = self.storage
            .get(&cas_id.id)
            .map_err(|e| Status::internal(format!("Failed to retrieve blob: {}", e)))?;

        match data {
            Some(bytes) => {
                // Deserialize the blob
                let blob = Self::deserialize_blob(&bytes)
                    .map_err(|e| Status::internal(format!("Failed to deserialize blob: {}", e)))?;

                info!("<== CAS Load completed - Loaded blob with ID: {}", hex::encode(&cas_id.id));

                Ok(Response::new(CasLoadResponse {
                    outcome: cas_load_response::Outcome::Success as i32,
                    contents: Some(cas_load_response::Contents::Data(blob)),
                }))
            }
            None => {
                debug!("<== CAS Load completed - Blob not found: {}", hex::encode(&cas_id.id));
                Ok(Response::new(CasLoadResponse {
                    outcome: cas_load_response::Outcome::ObjectNotFound as i32,
                    contents: None,
                }))
            }
        }
    }
}
