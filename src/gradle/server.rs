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

/// Gradle HTTP cache server
///
/// Implements the Gradle Build Cache HTTP API:
/// - GET /cache/{hash} - Retrieve cached artifact
/// - PUT /cache/{hash} - Store artifact
pub struct GradleHttpServer {
    storage: Arc<dyn Storage>,
}

impl GradleHttpServer {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Create the Axum router for Gradle cache endpoints
    pub fn router(self) -> Router {
        let shared_state = Arc::new(self);

        Router::new()
            .route("/cache/:hash", get(get_artifact))
            .route("/cache/:hash", put(put_artifact))
            .with_state(shared_state)
    }
}

/// GET /cache/{hash} - Retrieve cached artifact
async fn get_artifact(
    Path(hash): Path<String>,
    State(server): State<Arc<GradleHttpServer>>,
) -> impl IntoResponse {
    debug!(hash = %hash, "Gradle cache GET request");

    // Convert hash string to bytes for storage API
    let hash_bytes = hash.as_bytes();

    match server.storage.get(hash_bytes) {
        Ok(Some(data)) => {
            info!(hash = %hash, size = data.len(), "Gradle cache HIT");
            (StatusCode::OK, data)
        }
        Ok(None) => {
            debug!(hash = %hash, "Gradle cache MISS");
            (StatusCode::NOT_FOUND, Vec::new())
        }
        Err(e) => {
            warn!(hash = %hash, error = %e, "Gradle cache GET error");
            (StatusCode::INTERNAL_SERVER_ERROR, Vec::new())
        }
    }
}

/// PUT /cache/{hash} - Store artifact
async fn put_artifact(
    Path(hash): Path<String>,
    State(server): State<Arc<GradleHttpServer>>,
    body: Bytes,
) -> impl IntoResponse {
    debug!(hash = %hash, size = body.len(), "Gradle cache PUT request");

    // Convert hash string to bytes for storage API
    let hash_bytes = hash.as_bytes();

    match server.storage.put(hash_bytes, &body) {
        Ok(_) => {
            info!(hash = %hash, size = body.len(), "Gradle cache stored");
            StatusCode::OK
        }
        Err(e) => {
            warn!(hash = %hash, error = %e, "Gradle cache PUT error");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
