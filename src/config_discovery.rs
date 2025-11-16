use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

use crate::xdg;

/// Discovers Fabrik configuration by traversing up the directory tree
pub fn discover_config(start_dir: &Path) -> Result<Option<PathBuf>> {
    let mut current = start_dir.to_path_buf();

    loop {
        let config_path = current.join("fabrik.toml");
        if config_path.exists() {
            return Ok(Some(config_path));
        }

        // Try to go up one level
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }

    // Fallback to global config
    if let Some(home) = dirs::home_dir() {
        let global_config = home.join(".config/fabrik/config.toml");
        if global_config.exists() {
            return Ok(Some(global_config));
        }
    }

    Ok(None)
}

/// Computes a hash of the configuration file for daemon identification
pub fn hash_config(config_path: &Path) -> Result<String> {
    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();

    Ok(format!("{:x}", result)[..16].to_string())
}

/// Loads configuration with auto-discovery support
///
/// If `explicit_path` is provided, loads config from that path.
/// Otherwise, auto-discovers config by traversing up directory tree from cwd.
///
/// Returns Ok(None) if no config is found (neither explicit nor discovered).
pub fn load_config_with_discovery(
    explicit_path: Option<&str>,
) -> Result<Option<crate::config::FabrikConfig>> {
    use crate::config::FabrikConfig;

    if let Some(config_path) = explicit_path {
        // Explicit path provided - load it
        Ok(Some(FabrikConfig::from_file(config_path)?))
    } else {
        // Auto-discover by traversing up directory tree
        let current_dir = std::env::current_dir()
            .context("Failed to get current directory for config discovery")?;

        if let Some(discovered_path) = discover_config(&current_dir)? {
            Ok(Some(FabrikConfig::from_file(&discovered_path)?))
        } else {
            Ok(None)
        }
    }
}

/// Daemon state information
#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonState {
    pub config_hash: String,
    pub pid: u32,
    pub http_port: u16,
    pub grpc_port: u16,
    pub metrics_port: u16,
    pub unix_socket: Option<PathBuf>, // For Xcode integration
    pub config_path: PathBuf,
}

impl DaemonState {
    /// Get the base directory for daemon state
    /// Can be overridden with FABRIK_STATE_DIR for testing
    fn state_base_dir() -> PathBuf {
        if let Ok(state_dir) = std::env::var("FABRIK_STATE_DIR") {
            PathBuf::from(state_dir)
        } else {
            xdg::daemon_state_dir()
        }
    }

    pub fn state_dir(&self) -> PathBuf {
        Self::state_base_dir().join(&self.config_hash)
    }

    pub fn pid_file(&self) -> PathBuf {
        self.state_dir().join("pid")
    }

    pub fn ports_file(&self) -> PathBuf {
        self.state_dir().join("ports.json")
    }

    #[allow(dead_code)]
    pub fn env_file(&self) -> PathBuf {
        self.state_dir().join("env")
    }

    pub fn save(&self) -> Result<()> {
        let state_dir = self.state_dir();
        fs::create_dir_all(&state_dir)
            .with_context(|| format!("Failed to create state dir: {}", state_dir.display()))?;

        // Save PID
        fs::write(self.pid_file(), self.pid.to_string()).context("Failed to write PID file")?;

        // Save ports and socket
        let mut ports = serde_json::json!({
            "http": self.http_port,
            "grpc": self.grpc_port,
            "metrics": self.metrics_port,
        });

        if let Some(ref socket) = self.unix_socket {
            ports["unix_socket"] = serde_json::json!(socket.to_string_lossy());
        }

        fs::write(self.ports_file(), serde_json::to_string_pretty(&ports)?)
            .context("Failed to write ports file")?;

        // Save config path
        let config_file = state_dir.join("config_path.txt");
        fs::write(config_file, self.config_path.to_string_lossy().as_bytes())
            .context("Failed to write config path")?;

        Ok(())
    }

    /// Remove daemon state files
    pub fn cleanup(&self) -> Result<()> {
        let state_dir = self.state_dir();
        if state_dir.exists() {
            fs::remove_dir_all(&state_dir)
                .with_context(|| format!("Failed to remove state dir: {}", state_dir.display()))?;
        }
        Ok(())
    }

    pub fn load(config_hash: &str) -> Result<Option<Self>> {
        let state_dir = Self::state_base_dir().join(config_hash);

        if !state_dir.exists() {
            return Ok(None);
        }

        let pid_file = state_dir.join("pid");
        let ports_file = state_dir.join("ports.json");
        let config_path_file = state_dir.join("config_path.txt");

        if !pid_file.exists() || !ports_file.exists() {
            return Ok(None);
        }

        let pid: u32 = fs::read_to_string(&pid_file)
            .context("Failed to read PID file")?
            .trim()
            .parse()
            .context("Failed to parse PID")?;

        let ports: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&ports_file).context("Failed to read ports file")?,
        )?;

        let config_path = if config_path_file.exists() {
            PathBuf::from(fs::read_to_string(&config_path_file)?.trim())
        } else {
            PathBuf::new()
        };

        let unix_socket = ports["unix_socket"].as_str().map(PathBuf::from);

        Ok(Some(DaemonState {
            config_hash: config_hash.to_string(),
            pid,
            http_port: ports["http"].as_u64().unwrap() as u16,
            grpc_port: ports["grpc"].as_u64().unwrap() as u16,
            metrics_port: ports["metrics"].as_u64().unwrap() as u16,
            unix_socket,
            config_path,
        }))
    }

    pub fn is_running(&self) -> bool {
        is_process_running(self.pid)
    }

    pub fn generate_env_exports(&self, shell: &str) -> String {
        let http_url = format!("http://127.0.0.1:{}", self.http_port);
        let grpc_url = format!("grpc://127.0.0.1:{}", self.grpc_port);

        let mut exports = Vec::new();

        match shell {
            "fish" => {
                // Fabrik-specific variables
                exports.push(format!("set -gx FABRIK_HTTP_URL {}", http_url));
                exports.push(format!("set -gx FABRIK_GRPC_URL {}", grpc_url));
                exports.push(format!("set -gx FABRIK_CONFIG_HASH {}", self.config_hash));
                exports.push(format!("set -gx FABRIK_DAEMON_PID {}", self.pid));

                // Unix socket for Xcode
                if let Some(ref socket) = self.unix_socket {
                    exports.push(format!("set -gx FABRIK_UNIX_SOCKET {}", socket.display()));
                }

                // Build tool variables
                exports.extend(generate_build_tool_shell_exports(
                    &http_url,
                    self.unix_socket.as_deref(),
                    "fish",
                ));
            }
            _ => {
                // bash/zsh
                // Fabrik-specific variables
                exports.push(format!("export FABRIK_HTTP_URL={}", http_url));
                exports.push(format!("export FABRIK_GRPC_URL={}", grpc_url));
                exports.push(format!("export FABRIK_CONFIG_HASH={}", self.config_hash));
                exports.push(format!("export FABRIK_DAEMON_PID={}", self.pid));

                // Unix socket for Xcode
                if let Some(ref socket) = self.unix_socket {
                    exports.push(format!("export FABRIK_UNIX_SOCKET={}", socket.display()));
                }

                // Build tool variables
                exports.extend(generate_build_tool_shell_exports(
                    &http_url,
                    self.unix_socket.as_deref(),
                    "bash",
                ));
            }
        }

        exports.join("\n")
    }
}

/// Generate a unique token for TurboRepo local development
/// Uses PID XOR timestamp for uniqueness across process restarts
pub fn generate_turbo_token() -> String {
    format!(
        "fabrik-local-{:x}",
        std::process::id()
            ^ (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32)
    )
}

/// Default team name for TurboRepo local development
pub fn default_turbo_team() -> &'static str {
    "fabrik-local"
}

/// Populate all build tool environment variables
/// Returns a HashMap with environment variables for Gradle, Nx, Xcode, TurboRepo, etc.
pub fn populate_build_tool_env_vars(
    http_url: String,
    grpc_url: String,
    unix_socket: Option<std::path::PathBuf>,
) -> std::collections::HashMap<String, String> {
    let mut env_vars = std::collections::HashMap::new();

    // Generic Fabrik URLs
    env_vars.insert("FABRIK_HTTP_URL".to_string(), http_url.clone());
    env_vars.insert("FABRIK_GRPC_URL".to_string(), grpc_url);

    // Unix socket for Xcode (if available)
    if let Some(ref socket) = unix_socket {
        env_vars.insert(
            "FABRIK_UNIX_SOCKET".to_string(),
            socket.display().to_string(),
        );
    }

    // Gradle
    env_vars.insert("GRADLE_BUILD_CACHE_URL".to_string(), http_url.clone());

    // Nx
    env_vars.insert(
        "NX_SELF_HOSTED_REMOTE_CACHE_SERVER".to_string(),
        http_url.clone(),
    );

    // Xcode (prefer Unix socket if available)
    if let Some(socket) = unix_socket {
        env_vars.insert(
            "XCODE_CACHE_SERVER".to_string(),
            socket.display().to_string(),
        );
    } else {
        env_vars.insert("XCODE_CACHE_SERVER".to_string(), http_url.clone());
    }

    // TurboRepo
    env_vars.insert("TURBO_API".to_string(), http_url);
    // Auto-generate TURBO_TEAM if not already set
    if std::env::var("TURBO_TEAM").is_err() {
        env_vars.insert("TURBO_TEAM".to_string(), default_turbo_team().to_string());
    }
    // Auto-generate TURBO_TOKEN if not already set
    if std::env::var("TURBO_TOKEN").is_err() {
        env_vars.insert("TURBO_TOKEN".to_string(), generate_turbo_token());
    }

    env_vars
}

/// Generate shell export statements for all build tool environment variables
/// For use in shell activation hooks
fn generate_build_tool_shell_exports(
    http_url: &str,
    unix_socket: Option<&std::path::Path>,
    shell: &str,
) -> Vec<String> {
    let mut exports = Vec::new();

    match shell {
        "fish" => {
            // Gradle
            exports.push(format!("set -gx GRADLE_BUILD_CACHE_URL {}", http_url));

            // Nx
            exports.push(format!(
                "set -gx NX_SELF_HOSTED_REMOTE_CACHE_SERVER {}",
                http_url
            ));

            // Xcode (prefer Unix socket if available)
            if let Some(socket) = unix_socket {
                exports.push(format!("set -gx XCODE_CACHE_SERVER {}", socket.display()));
            } else {
                exports.push(format!("set -gx XCODE_CACHE_SERVER {}", http_url));
            }

            // TurboRepo
            exports.push(format!("set -gx TURBO_API {}", http_url));
            exports.push(format!(
                "test -z \"$TURBO_TEAM\"; and set -gx TURBO_TEAM {}",
                default_turbo_team()
            ));
            exports.push(format!(
                "test -z \"$TURBO_TOKEN\"; and set -gx TURBO_TOKEN {}",
                generate_turbo_token()
            ));
        }
        _ => {
            // bash/zsh
            // Gradle
            exports.push(format!("export GRADLE_BUILD_CACHE_URL={}", http_url));

            // Nx
            exports.push(format!(
                "export NX_SELF_HOSTED_REMOTE_CACHE_SERVER={}",
                http_url
            ));

            // Xcode (prefer Unix socket if available)
            if let Some(socket) = unix_socket {
                exports.push(format!("export XCODE_CACHE_SERVER={}", socket.display()));
            } else {
                exports.push(format!("export XCODE_CACHE_SERVER={}", http_url));
            }

            // TurboRepo
            exports.push(format!("export TURBO_API={}", http_url));
            exports.push(format!(
                "[ -z \"$TURBO_TEAM\" ] && export TURBO_TEAM={}",
                default_turbo_team()
            ));
            exports.push(format!(
                "[ -z \"$TURBO_TOKEN\" ] && export TURBO_TOKEN={}",
                generate_turbo_token()
            ));
        }
    }

    exports
}

/// Check if a process is running
#[cfg(unix)]
fn is_process_running(pid: u32) -> bool {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    kill(Pid::from_raw(pid as i32), Signal::SIGCONT).is_ok()
}

#[cfg(windows)]
fn is_process_running(pid: u32) -> bool {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::winnt::PROCESS_QUERY_INFORMATION;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
        if handle.is_null() {
            false
        } else {
            CloseHandle(handle);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_config_finds_nearest() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        // Create nested structure
        let project = root.join("project");
        let subdir = project.join("subdir");
        fs::create_dir_all(&subdir).unwrap();

        // Create config in project root
        let config_path = project.join("fabrik.toml");
        fs::write(&config_path, "# test config").unwrap();

        // Search from subdir should find project config
        let found = discover_config(&subdir).unwrap();
        assert_eq!(found, Some(config_path));
    }

    #[test]
    fn test_hash_config_is_consistent() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("fabrik.toml");
        fs::write(&config_path, "[cache]\ndir = \"/tmp/cache\"").unwrap();

        let hash1 = hash_config(&config_path).unwrap();
        let hash2 = hash_config(&config_path).unwrap();

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 16);
    }
}
