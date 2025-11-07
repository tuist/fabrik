// Common test utilities shared across acceptance tests
//
// ## Test Isolation Strategy
//
// Each acceptance test creates a completely isolated Fabrik daemon instance
// with NO GLOBAL STATE to prevent test interference.
//
// ### What's Isolated:
// - Cache directory: Each daemon uses a unique temp directory
// - State directory: FABRIK_STATE_DIR env var points to test-specific temp dir
// - Ports: Each daemon binds to port 0, OS assigns random available ports
// - Process: Each daemon runs in its own child process
//
// ### What's NOT Global:
// - ❌ No shared ~/.fabrik/daemons/ directory (isolated via FABRIK_STATE_DIR)
// - ❌ No shared cache (each test has its own temp cache dir)
// - ❌ No port conflicts (random port allocation)
// - ❌ No process conflicts (each test spawns its own daemon)
//
// ### Cleanup:
// When TestDaemon is dropped:
// 1. Daemon process is killed
// 2. Temp directory (containing cache + state) is automatically deleted
// 3. No cleanup needed in ~/.fabrik/ or any global location
//
// This ensures tests can run in parallel without interfering with each other
// or with development daemons running on the same machine.

use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Daemon mode for testing
#[allow(dead_code)]
enum DaemonMode {
    Tcp,        // TCP mode: HTTP + gRPC servers
    UnixSocket, // Unix socket mode: For Xcode
}

/// Helper to start a Fabrik daemon for testing
/// Each test gets its own isolated daemon with unique ports and cache
/// NO GLOBAL STATE - all state is in temporary directories
pub struct TestDaemon {
    _temp_dir: TempDir,
    #[allow(dead_code)]
    pub cache_dir: PathBuf,
    state_dir: PathBuf, // Isolated state directory for this test
    child: Child,
    pub http_port: u16,
    #[allow(dead_code)]
    pub grpc_port: u16,
    config_hash: String,
}

impl TestDaemon {
    /// Start a new test daemon with isolated cache and state (TCP mode)
    #[allow(dead_code)]
    pub fn start() -> Self {
        Self::start_with_mode(DaemonMode::Tcp)
    }

    /// Start a new test daemon with Unix socket (for Xcode tests)
    #[allow(dead_code)]
    pub fn start_with_socket() -> Self {
        Self::start_with_mode(DaemonMode::UnixSocket)
    }

    fn start_with_mode(mode: DaemonMode) -> Self {
        let fabrik_bin = env!("CARGO_BIN_EXE_fabrik");
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let state_dir = temp_dir.path().join("state");

        // Create state directory
        std::fs::create_dir_all(&state_dir).expect("Failed to create state dir");

        // Create a test config file with optional socket
        let config_path = temp_dir.path().join("fabrik.toml");

        // Convert paths to forward slashes for TOML (Windows compatibility)
        let cache_dir_str = cache_dir.display().to_string().replace('\\', "/");

        let config_content = match mode {
            DaemonMode::Tcp => {
                format!(
                    r#"
[cache]
dir = "{}"
max_size = "1GB"
"#,
                    cache_dir_str
                )
            }
            DaemonMode::UnixSocket => {
                let socket_path = temp_dir.path().join("xcode.sock");
                let socket_path_str = socket_path.display().to_string().replace('\\', "/");
                format!(
                    r#"
[cache]
dir = "{}"
max_size = "1GB"

[daemon]
socket = "{}"
"#,
                    cache_dir_str, socket_path_str
                )
            }
        };

        std::fs::write(&config_path, config_content).expect("Failed to write test config");

        // Compute config hash
        let config_hash = {
            use sha2::{Digest, Sha256};
            let content = std::fs::read_to_string(&config_path).unwrap();
            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            format!("{:x}", hasher.finalize())[..16].to_string()
        };

        println!("Starting test daemon with config hash: {}", config_hash);
        println!("  Isolated state dir: {}", state_dir.display());

        // Start daemon in background with isolated state directory
        let mut child = Command::new(fabrik_bin)
            .arg("daemon")
            .arg("--config")
            .arg(&config_path)
            .env("FABRIK_STATE_DIR", &state_dir) // USE ISOLATED STATE DIR
            .spawn()
            .expect("Failed to start daemon");

        // Wait for daemon to start and write state
        thread::sleep(Duration::from_secs(1));

        // Check if daemon is still running
        match child.try_wait() {
            Ok(Some(status)) => panic!("Daemon exited immediately with status: {}", status),
            Ok(None) => {
                // Still running, good
            }
            Err(e) => panic!("Error checking daemon status: {}", e),
        }

        // Read state to get actual ports from our isolated state directory
        let daemon_state_dir = state_dir.join(&config_hash);

        // Wait for state file to be created (max 5 seconds)
        let mut ports_json_content = None;
        for _ in 0..50 {
            if let Ok(content) = std::fs::read_to_string(daemon_state_dir.join("ports.json")) {
                ports_json_content = Some(content);
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }

        let ports_json =
            ports_json_content.expect("Failed to read ports.json - daemon may not have started");
        let ports: serde_json::Value =
            serde_json::from_str(&ports_json).expect("Failed to parse ports.json");

        let http_port = ports["http"].as_u64().expect("Missing http port") as u16;
        let grpc_port = ports["grpc"].as_u64().expect("Missing grpc port") as u16;

        println!("Test daemon started:");
        println!("  Config hash: {}", config_hash);
        println!("  HTTP port: {}", http_port);
        println!("  gRPC port: {}", grpc_port);
        println!("  Cache dir: {}", cache_dir.display());

        Self {
            _temp_dir: temp_dir,
            cache_dir,
            state_dir,
            child,
            http_port,
            grpc_port,
            config_hash,
        }
    }

    #[allow(dead_code)]
    pub fn http_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.http_port)
    }

    #[allow(dead_code)]
    pub fn grpc_url(&self) -> String {
        format!("grpc://127.0.0.1:{}", self.grpc_port)
    }

    /// Get Unix socket path (for Xcode tests)
    /// Returns the socket path from daemon state
    #[allow(dead_code)]
    pub fn socket_path(&self) -> Option<PathBuf> {
        let daemon_state_dir = self.state_dir.join(&self.config_hash);
        let ports_file = daemon_state_dir.join("ports.json");

        if let Ok(content) = std::fs::read_to_string(ports_file) {
            if let Ok(ports) = serde_json::from_str::<serde_json::Value>(&content) {
                return ports["unix_socket"].as_str().map(PathBuf::from);
            }
        }

        None
    }
}

impl Drop for TestDaemon {
    fn drop(&mut self) {
        println!(
            "Cleaning up test daemon (config hash: {})",
            self.config_hash
        );

        // Kill daemon
        let _ = self.child.kill();
        let _ = self.child.wait();

        // Note: State directory cleanup happens automatically when _temp_dir is dropped
        // The daemon state is in: self.state_dir (which is inside _temp_dir)
        // No global state in ~/.fabrik/ is touched!
        println!("  All state cleaned up (no global state leaked)");
    }
}
