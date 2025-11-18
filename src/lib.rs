// Library interface for Fabrik
// This allows integration tests and external code to use Fabrik's modules

pub mod auth;
pub mod bazel;
pub mod capi; // C API (FFI) for external integrations
pub mod cli_utils;
pub mod config;
pub mod config_discovery;
pub mod config_expansion; // Environment variable expansion for config files
pub mod logging;
pub mod p2p; // P2P cache sharing
pub mod recipe; // Portable recipes (QuickJS/JavaScript)
pub mod script; // Script recipes (bash, python, etc.)
pub mod storage;
pub mod xdg;

// Re-export commonly used types
pub use auth::AuthProvider;
pub use config::FabrikConfig;
pub use config_discovery::{discover_config, hash_config, DaemonState};
pub use recipe::{RecipeExecutor, RecipeMetadata};
pub use storage::{create_storage, default_cache_dir, FilesystemStorage, Storage};
