use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

use crate::storage::Storage;

/// HTTP server state
#[derive(Clone)]
struct AppState<S: Storage + Clone> {
    storage: Arc<S>,
}

/// HTTP cache server for Metro, Gradle, Nx, TurboRepo, etc.
///
/// Implements a simple HTTP API:
/// - GET /api/v1/artifacts/{hash} - Retrieve artifact (Metro, Gradle)
/// - PUT /api/v1/artifacts/{hash} - Store artifact (Metro, Gradle)
/// - GET /v1/cache/{hash} - Retrieve artifact (Nx, TurboRepo)
/// - PUT /v1/cache/{hash} - Store artifact (Nx, TurboRepo)
/// - GET /health - Health check
pub struct HttpServer<S: Storage + Clone> {
    port: u16,
    storage: Arc<S>,
}

impl<S: Storage + Clone + 'static> HttpServer<S> {
    pub fn new(port: u16, storage: Arc<S>) -> Self {
        Self { port, storage }
    }

    /// Start the HTTP server
    pub async fn run(self) -> Result<()> {
        let state = AppState {
            storage: self.storage.clone(),
        };

        let app = Router::new()
            .route("/health", get(health_handler))
            // Metro, Gradle routes
            .route("/api/v1/artifacts/:hash", get(get_artifact))
            .route("/api/v1/artifacts/:hash", put(put_artifact))
            // Nx, TurboRepo routes (same handlers, different paths)
            .route("/v1/cache/:hash", get(get_artifact))
            .route("/v1/cache/:hash", put(put_artifact))
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        let addr = format!("0.0.0.0:{}", self.port);
        info!("HTTP server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Health check handler
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Get artifact handler
async fn get_artifact<S: Storage + Clone>(
    Path(hash): Path<String>,
    State(state): State<AppState<S>>,
) -> Response {
    // Decode hex hash to bytes
    let hash_bytes = match hex::decode(&hash) {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!("Invalid hash format: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid hash format").into_response();
        }
    };

    // Get from storage
    match state.storage.get(&hash_bytes) {
        Ok(Some(data)) => {
            info!("Cache hit: {}", hash);
            (StatusCode::OK, data).into_response()
        }
        Ok(None) => {
            info!("Cache miss: {}", hash);
            (StatusCode::NOT_FOUND, "Not found").into_response()
        }
        Err(e) => {
            warn!("Storage error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
        }
    }
}

/// Put artifact handler
async fn put_artifact<S: Storage + Clone>(
    Path(hash): Path<String>,
    State(state): State<AppState<S>>,
    body: Bytes,
) -> Response {
    // Decode hex hash to bytes
    let hash_bytes = match hex::decode(&hash) {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!("Invalid hash format: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid hash format").into_response();
        }
    };

    // Store in cache
    match state.storage.put(&hash_bytes, &body) {
        Ok(()) => {
            info!("Stored artifact: {} ({} bytes)", hash, body.len());
            (StatusCode::OK, "Stored").into_response()
        }
        Err(e) => {
            warn!("Storage error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::FilesystemStorage;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_http_server_health() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(FilesystemStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
        let server = HttpServer::new(0, storage);

        // Just test that we can create the server
        assert_eq!(server.port, 0);
    }
}
