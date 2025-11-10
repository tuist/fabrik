// Library interface for Fabrik
// This allows integration tests and external code to use Fabrik's modules

pub mod bazel;
pub mod cli_utils;
pub mod config_discovery;
pub mod logging;
pub mod script;
pub mod storage;

// Re-export commonly used types
pub use config_discovery::{discover_config, hash_config, DaemonState};
pub use storage::{create_storage, default_cache_dir, FilesystemStorage, Storage};
