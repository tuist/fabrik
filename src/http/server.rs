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
/// - GET /api/v1/artifacts/{hash} - Retrieve artifact (Metro) - hex-encoded
/// - PUT /api/v1/artifacts/{hash} - Store artifact (Metro) - hex-encoded
/// - GET /v1/cache/{hash} - Retrieve artifact (Nx, TurboRepo) - raw string
/// - PUT /v1/cache/{hash} - Store artifact (Nx, TurboRepo) - raw string
/// - GET /cache/{hash} - Retrieve artifact (Gradle) - raw string
/// - PUT /cache/{hash} - Store artifact (Gradle) - raw string
/// - GET /health - Health check
pub struct HttpServer<S: Storage + Clone> {
    port: u16,
    storage: Arc<S>,
}

impl<S: Storage + Clone + 'static> HttpServer<S> {
    pub fn new(port: u16, storage: Arc<S>) -> Self {
        Self { port, storage }
    }

    /// Create a new HTTP server with automatic port allocation (port 0)
    /// Returns the server, actual assigned port, and the pre-bound listener
    pub async fn new_with_port_zero(
        storage: Arc<S>,
    ) -> Result<(Self, u16, tokio::net::TcpListener)> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let actual_port = listener.local_addr()?.port();
        let server = Self {
            port: actual_port,
            storage,
        };
        Ok((server, actual_port, listener))
    }

    /// Run the server with a pre-bound listener
    /// This is useful when you need to know the actual port before starting the server
    pub async fn run_with_listener(self, listener: tokio::net::TcpListener) -> Result<()> {
        let app = self.router();
        info!("HTTP server listening on {}", listener.local_addr()?);
        axum::serve(listener, app).await?;
        Ok(())
    }

    /// Create the Axum router with all cache endpoints
    pub fn router(self) -> Router {
        let state = AppState {
            storage: self.storage,
        };

        Router::new()
            .route("/health", get(health_handler))
            // Metro routes (hex-encoded)
            .route("/api/v1/artifacts/:hash", get(get_metro_artifact))
            .route("/api/v1/artifacts/:hash", put(put_metro_artifact))
            // Nx, TurboRepo routes (raw string - numeric hashes)
            .route("/v1/cache/:hash", get(get_nx_artifact))
            .route("/v1/cache/:hash", put(put_nx_artifact))
            // Gradle routes (raw string)
            .route("/cache/:hash", get(get_gradle_artifact))
            .route("/cache/:hash", put(put_gradle_artifact))
            .layer(TraceLayer::new_for_http())
            .with_state(state)
    }

    /// Start the HTTP server
    pub async fn run(self) -> Result<()> {
        let port = self.port;
        let app = self.router();

        let addr = format!("0.0.0.0:{}", port);
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

/// Get artifact handler for Metro
/// Metro uses hex-encoded hashes via /api/v1/artifacts/{hash}
async fn get_metro_artifact<S: Storage + Clone>(
    Path(hash): Path<String>,
    State(state): State<AppState<S>>,
) -> Response {
    // Decode hex hash to bytes
    let hash_bytes = match hex::decode(&hash) {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!(build_system = "metro", hash = %hash, error = %e, "Invalid hash format");
            return (StatusCode::BAD_REQUEST, "Invalid hash format").into_response();
        }
    };

    // Get from storage
    match state.storage.get(&hash_bytes) {
        Ok(Some(data)) => {
            info!(build_system = "metro", hash = %hash, size = data.len(), "Cache HIT");
            (StatusCode::OK, data).into_response()
        }
        Ok(None) => {
            info!(build_system = "metro", hash = %hash, "Cache MISS");
            (StatusCode::NOT_FOUND, "Not found").into_response()
        }
        Err(e) => {
            warn!(build_system = "metro", hash = %hash, error = %e, "Storage error");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
        }
    }
}

/// Put artifact handler for Metro
/// Metro uses hex-encoded hashes via /api/v1/artifacts/{hash}
async fn put_metro_artifact<S: Storage + Clone>(
    Path(hash): Path<String>,
    State(state): State<AppState<S>>,
    body: Bytes,
) -> Response {
    // Decode hex hash to bytes
    let hash_bytes = match hex::decode(&hash) {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!(build_system = "metro", hash = %hash, error = %e, "Invalid hash format");
            return (StatusCode::BAD_REQUEST, "Invalid hash format").into_response();
        }
    };

    // Store in cache
    match state.storage.put(&hash_bytes, &body) {
        Ok(()) => {
            info!(build_system = "metro", hash = %hash, size = body.len(), "Artifact stored");
            (StatusCode::OK, "Stored").into_response()
        }
        Err(e) => {
            warn!(build_system = "metro", hash = %hash, error = %e, "Storage error");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
        }
    }
}

/// Get artifact handler for Nx/TurboRepo
/// Nx/TurboRepo use raw string hashes (numeric) via /v1/cache/{hash}
async fn get_nx_artifact<S: Storage + Clone>(
    Path(hash): Path<String>,
    State(state): State<AppState<S>>,
) -> Response {
    // Use hash string directly as bytes (no hex decoding)
    // Nx sends numeric hashes like "3928369906857521520"
    let hash_bytes = hash.as_bytes();

    // Get from storage
    match state.storage.get(hash_bytes) {
        Ok(Some(data)) => {
            info!(build_system = "nx", hash = %hash, size = data.len(), "Cache HIT");
            (
                StatusCode::OK,
                [("Content-Type", "application/octet-stream")],
                data,
            )
                .into_response()
        }
        Ok(None) => {
            info!(build_system = "nx", hash = %hash, "Cache MISS");
            (StatusCode::NOT_FOUND, Vec::new()).into_response()
        }
        Err(e) => {
            warn!(build_system = "nx", hash = %hash, error = %e, "Storage error");
            (StatusCode::INTERNAL_SERVER_ERROR, Vec::new()).into_response()
        }
    }
}

/// Put artifact handler for Nx/TurboRepo
/// Nx/TurboRepo use raw string hashes (numeric) via /v1/cache/{hash}
async fn put_nx_artifact<S: Storage + Clone>(
    Path(hash): Path<String>,
    State(state): State<AppState<S>>,
    body: Bytes,
) -> Response {
    // Use hash string directly as bytes (no hex decoding)
    // Nx sends numeric hashes like "3928369906857521520"
    let hash_bytes = hash.as_bytes();

    // Store in cache
    match state.storage.put(hash_bytes, &body) {
        Ok(()) => {
            info!(build_system = "nx", hash = %hash, size = body.len(), "Artifact stored");
            (StatusCode::OK, "Stored").into_response()
        }
        Err(e) => {
            warn!(build_system = "nx", hash = %hash, error = %e, "Storage error");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
        }
    }
}

/// Get artifact handler for Gradle
/// Gradle uses raw string hashes (no hex encoding) via /cache/{hash}
async fn get_gradle_artifact<S: Storage + Clone>(
    Path(hash): Path<String>,
    State(state): State<AppState<S>>,
) -> Response {
    // Use hash string directly as bytes (no hex decoding)
    let hash_bytes = hash.as_bytes();

    // Get from storage
    match state.storage.get(hash_bytes) {
        Ok(Some(data)) => {
            info!(build_system = "gradle", hash = %hash, size = data.len(), "Cache HIT");
            (StatusCode::OK, data).into_response()
        }
        Ok(None) => {
            info!(build_system = "gradle", hash = %hash, "Cache MISS");
            (StatusCode::NOT_FOUND, Vec::new()).into_response()
        }
        Err(e) => {
            warn!(build_system = "gradle", hash = %hash, error = %e, "Storage error");
            (StatusCode::INTERNAL_SERVER_ERROR, Vec::new()).into_response()
        }
    }
}

/// Put artifact handler for Gradle
/// Gradle uses raw string hashes (no hex encoding) via /cache/{hash}
async fn put_gradle_artifact<S: Storage + Clone>(
    Path(hash): Path<String>,
    State(state): State<AppState<S>>,
    body: Bytes,
) -> Response {
    // Use hash string directly as bytes (no hex decoding)
    let hash_bytes = hash.as_bytes();

    // Store in cache
    match state.storage.put(hash_bytes, &body) {
        Ok(()) => {
            info!(build_system = "gradle", hash = %hash, size = body.len(), "Artifact stored");
            (StatusCode::OK, "Stored").into_response()
        }
        Err(e) => {
            warn!(build_system = "gradle", hash = %hash, error = %e, "Storage error");
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
