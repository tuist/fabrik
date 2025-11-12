// Library interface for Fabrik
// This allows integration tests and external code to use Fabrik's modules

pub mod auth;
pub mod bazel;
pub mod capi; // C API (FFI) for external integrations
pub mod cli_utils;
pub mod config;
pub mod config_discovery;
pub mod logging;
pub mod script;
pub mod storage;

// Re-export commonly used types
pub use auth::AuthProvider;
pub use config::FabrikConfig;
pub use config_discovery::{discover_config, hash_config, DaemonState};
pub use storage::{create_storage, default_cache_dir, FilesystemStorage, Storage};
