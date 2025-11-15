// OAuth2 acceptance tests
//
// These tests verify OAuth2 authentication with an in-memory mock OAuth2 server.
// The mock server's lifecycle is scoped to each test.
//
// To run: `cargo test --test oauth2_acceptance -- --nocapture`

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Form, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;
use tokio::net::TcpListener;

/// Mock OAuth2 server state
#[derive(Clone)]
struct MockOAuthState {
    /// device_code -> access_token (once authorized)
    tokens: Arc<Mutex<HashMap<String, String>>>,
    /// device_code -> user_code
    device_codes: Arc<Mutex<HashMap<String, String>>>,
    /// Auto-authorize device codes after creation (for testing)
    auto_authorize: Arc<Mutex<bool>>,
}

impl MockOAuthState {
    fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
            device_codes: Arc::new(Mutex::new(HashMap::new())),
            auto_authorize: Arc::new(Mutex::new(false)),
        }
    }

    /// Enable auto-authorization for testing
    fn enable_auto_authorize(&self) {
        *self.auto_authorize.lock().unwrap() = true;
    }

    /// Simulate user authorizing a device code
    fn authorize_device(&self, device_code: &str) {
        let access_token = format!("access_token_{}", uuid::Uuid::new_v4());
        self.tokens
            .lock()
            .unwrap()
            .insert(device_code.to_string(), access_token);
    }
}

/// Device authorization request
#[derive(Deserialize)]
struct DeviceAuthRequest {
    #[allow(dead_code)]
    client_id: String,
    #[allow(dead_code)]
    scope: Option<String>,
}

/// Device authorization response
#[derive(Serialize)]
struct DeviceAuthResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

/// Token request
#[derive(Deserialize)]
struct TokenRequest {
    grant_type: String,
    device_code: String,
    #[allow(dead_code)]
    client_id: String,
}

/// Token response (success)
#[derive(Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
}

/// Token error response
#[derive(Serialize)]
struct TokenErrorResponse {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_description: Option<String>,
}

/// Device authorization endpoint handler
async fn device_authorization(
    State(state): State<MockOAuthState>,
    Form(_req): Form<DeviceAuthRequest>,
) -> Json<DeviceAuthResponse> {
    let device_code = format!("device_{}", uuid::Uuid::new_v4());
    let user_code = format!("USER-{}", rand::random::<u16>());

    // Store device code
    state
        .device_codes
        .lock()
        .unwrap()
        .insert(device_code.clone(), user_code.clone());

    // Auto-authorize if enabled (for testing)
    if *state.auto_authorize.lock().unwrap() {
        let state_clone = state.clone();
        let device_code_clone = device_code.clone();
        tokio::spawn(async move {
            // Simulate user authorization after a short delay
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            state_clone.authorize_device(&device_code_clone);
            println!(
                "[mock-oauth] Auto-authorized device code: {}",
                device_code_clone
            );
        });
    }

    Json(DeviceAuthResponse {
        device_code,
        user_code,
        verification_uri: "http://example.com/activate".to_string(),
        expires_in: 1800,
        interval: 1, // Short interval for testing
    })
}

/// Token endpoint handler
async fn token(
    State(state): State<MockOAuthState>,
    Form(req): Form<TokenRequest>,
) -> impl IntoResponse {
    if req.grant_type != "urn:ietf:params:oauth:grant-type:device_code" {
        return (
            StatusCode::BAD_REQUEST,
            Json(TokenErrorResponse {
                error: "unsupported_grant_type".to_string(),
                error_description: Some("Only device_code grant type is supported".to_string()),
            }),
        )
            .into_response();
    }

    // Check if device code exists
    if !state
        .device_codes
        .lock()
        .unwrap()
        .contains_key(&req.device_code)
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(TokenErrorResponse {
                error: "invalid_grant".to_string(),
                error_description: Some("Invalid device code".to_string()),
            }),
        )
            .into_response();
    }

    // Check if device code has been authorized
    if let Some(access_token) = state.tokens.lock().unwrap().get(&req.device_code) {
        return (
            StatusCode::OK,
            Json(TokenResponse {
                access_token: access_token.clone(),
                token_type: "Bearer".to_string(),
                expires_in: 3600,
                refresh_token: Some(format!("refresh_{}", uuid::Uuid::new_v4())),
            }),
        )
            .into_response();
    }

    // Not yet authorized
    (
        StatusCode::BAD_REQUEST,
        Json(TokenErrorResponse {
            error: "authorization_pending".to_string(),
            error_description: Some("User has not yet authorized the device".to_string()),
        }),
    )
        .into_response()
}

/// Health check endpoint
async fn health() -> &'static str {
    "OK"
}

/// Mock OAuth2 server
struct MockOAuthServer {
    #[allow(dead_code)]
    state: MockOAuthState,
    base_url: String,
    _shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl MockOAuthServer {
    /// Start a new mock OAuth2 server
    async fn start() -> Self {
        Self::start_with_auto_authorize(false).await
    }

    /// Start a new mock OAuth2 server with auto-authorization
    async fn start_with_auto_authorize(auto_authorize: bool) -> Self {
        let state = MockOAuthState::new();
        if auto_authorize {
            state.enable_auto_authorize();
        }

        let app = Router::new()
            .route("/oauth/device/code", post(device_authorization))
            .route("/oauth/token", post(token))
            .route("/health", get(health))
            .with_state(state.clone());

        // Bind to random port
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to address");
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", addr);

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        // Spawn server in background
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    shutdown_rx.await.ok();
                })
                .await
                .expect("Server failed");
        });

        // Wait for server to be ready
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Self {
            state,
            base_url,
            _shutdown_tx: shutdown_tx,
        }
    }

    /// Simulate user authorizing a device code
    #[allow(dead_code)]
    fn authorize_device(&self, device_code: &str) {
        self.state.authorize_device(device_code);
    }

    /// Get base URL
    fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Helper to create a test config with OAuth2
fn create_oauth2_config(temp_dir: &TempDir, oauth_url: &str) -> PathBuf {
    let config_path = temp_dir.path().join("fabrik.toml");

    let config_content = format!(
        r#"
url = "{}"

[auth]
provider = "oauth2"

[auth.oauth2]
client_id = "test-client"
scopes = "cache:read cache:write"
storage = "file"
device_authorization_endpoint = "{}/oauth/device/code"
token_endpoint = "{}/oauth/token"
"#,
        oauth_url, oauth_url, oauth_url
    );

    std::fs::write(&config_path, config_content).expect("Failed to write test config");
    config_path
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_oauth2_device_flow_login() {
    // Disable browser opening during tests
    std::env::set_var("SCHLUSSEL_NO_BROWSER", "1");

    // Start mock OAuth2 server with auto-authorization
    let server = MockOAuthServer::start_with_auto_authorize(true).await;
    println!(
        "[fabrik] Mock OAuth2 server started at: {}",
        server.base_url()
    );

    // Create test config
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = create_oauth2_config(&temp_dir, server.base_url());

    println!("[fabrik] Test config created");

    // Load config and create AuthProvider
    let config_content = std::fs::read_to_string(&config_path).unwrap();
    let config: fabrik::config::FabrikConfig =
        toml::from_str(&config_content).expect("Failed to parse config");

    let provider =
        fabrik::auth::provider::AuthProvider::new(config.auth.clone(), config.url.clone())
            .expect("Failed to create auth provider");

    // Perform login (mock server will auto-authorize after 500ms)
    println!("[fabrik] Starting OAuth2 login flow...");
    let login_result = provider.login().await;

    if let Err(ref e) = login_result {
        println!("[fabrik] Login error: {:?}", e);
    }

    assert!(
        login_result.is_ok(),
        "Login should succeed with auto-authorization: {:?}",
        login_result.err()
    );
    println!("[fabrik] Login completed successfully");

    // Leak provider to avoid runtime drop issues (acceptable for tests)
    std::mem::forget(provider);

    // Create new provider instance to check status
    let provider2 =
        fabrik::auth::provider::AuthProvider::new(config.auth.clone(), config.url.clone())
            .expect("Failed to create auth provider");

    // Check status - should be authenticated
    let status = provider2.status().await.expect("Failed to get status");
    assert!(status.authenticated, "Should be authenticated after login");
    assert_eq!(status.provider, "oauth2");
    assert!(status.token_preview.is_some(), "Should have token preview");
    println!("[fabrik] Authentication status verified");

    // Test logout
    provider2.logout().await.expect("Failed to logout");
    println!("[fabrik] Logout completed successfully");

    // Leak provider2
    std::mem::forget(provider2);

    // Create new provider instance to verify logout
    let provider3 = fabrik::auth::provider::AuthProvider::new(config.auth, config.url)
        .expect("Failed to create auth provider");

    // Check status after logout - should not be authenticated
    let status_after_logout = provider3.status().await.expect("Failed to get status");
    assert!(
        !status_after_logout.authenticated,
        "Should not be authenticated after logout"
    );
    println!("[fabrik] Logout verified - no longer authenticated");

    // Leak provider3, server, and temp_dir to avoid runtime drop issues
    std::mem::forget(provider3);
    std::mem::forget(server);
    std::mem::forget(temp_dir);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_oauth2_status_not_authenticated() {
    // Disable browser opening during tests
    std::env::set_var("SCHLUSSEL_NO_BROWSER", "1");

    // Start mock OAuth2 server
    let server = MockOAuthServer::start().await;

    // Create test config
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = create_oauth2_config(&temp_dir, server.base_url());

    // Load config
    let config_content = std::fs::read_to_string(&config_path).unwrap();
    let config: fabrik::config::FabrikConfig =
        toml::from_str(&config_content).expect("Failed to parse config");

    let provider = fabrik::auth::provider::AuthProvider::new(config.auth, config.url)
        .expect("Failed to create auth provider");

    // Check status - should not be authenticated
    let status = provider.status().await.expect("Failed to get status");
    assert!(
        !status.authenticated,
        "Should not be authenticated initially"
    );
    assert_eq!(status.provider, "oauth2");
    assert!(status.token_preview.is_none());

    // Leak provider and server to avoid runtime drop issues in tests
    // (This is acceptable for test code)
    std::mem::forget(provider);
    std::mem::forget(server);
    std::mem::forget(temp_dir);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_oauth2_token_refresh() {
    // Disable browser opening during tests
    std::env::set_var("SCHLUSSEL_NO_BROWSER", "1");

    // This test would verify automatic token refresh
    // Currently blocked by the same limitation - need to mock the full flow
    // including token storage and retrieval

    // Start mock OAuth2 server
    let server = MockOAuthServer::start().await;

    println!(
        "Token refresh test - mock server ready at: {}",
        server.base_url()
    );

    // TODO: Implement once we can properly mock the device authorization flow
}
