use crate::storage::Storage;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Nx HTTP cache server
///
/// Implements the Nx remote cache HTTP API:
/// - GET /v1/cache/{hash} - Retrieve cached artifact
/// - PUT /v1/cache/{hash} - Store artifact
pub struct NxHttpServer {
    storage: Arc<dyn Storage>,
}

impl NxHttpServer {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Create the Axum router for Nx cache endpoints
    pub fn router(self) -> Router {
        let shared_state = Arc::new(self);

        Router::new()
            .route("/v1/cache/:hash", get(get_artifact))
            .route("/v1/cache/:hash", put(put_artifact))
            .with_state(shared_state)
    }
}

/// GET /v1/cache/{hash} - Retrieve cached artifact
async fn get_artifact(
    Path(hash): Path<String>,
    State(server): State<Arc<NxHttpServer>>,
) -> impl IntoResponse {
    debug!(hash = %hash, "Nx cache GET request");

    // Decode hex hash to bytes
    let hash_bytes = match hex::decode(&hash) {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!(hash = %hash, error = %e, "Nx cache GET - invalid hash format");
            return (StatusCode::BAD_REQUEST, Vec::new());
        }
    };

    match server.storage.get(&hash_bytes) {
        Ok(Some(data)) => {
            info!(hash = %hash, size = data.len(), "Nx cache HIT");
            (StatusCode::OK, data)
        }
        Ok(None) => {
            debug!(hash = %hash, "Nx cache MISS");
            (StatusCode::NOT_FOUND, Vec::new())
        }
        Err(e) => {
            warn!(hash = %hash, error = %e, "Nx cache GET error");
            (StatusCode::INTERNAL_SERVER_ERROR, Vec::new())
        }
    }
}

/// PUT /v1/cache/{hash} - Store artifact
async fn put_artifact(
    Path(hash): Path<String>,
    State(server): State<Arc<NxHttpServer>>,
    body: Bytes,
) -> impl IntoResponse {
    debug!(hash = %hash, size = body.len(), "Nx cache PUT request");

    // Decode hex hash to bytes
    let hash_bytes = match hex::decode(&hash) {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!(hash = %hash, error = %e, "Nx cache PUT - invalid hash format");
            return StatusCode::BAD_REQUEST;
        }
    };

    match server.storage.put(&hash_bytes, &body) {
        Ok(_) => {
            info!(hash = %hash, size = body.len(), "Nx cache stored");
            StatusCode::OK
        }
        Err(e) => {
            warn!(hash = %hash, error = %e, "Nx cache PUT error");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
