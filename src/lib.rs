// Library interface for Fabrik
// This allows integration tests and external code to use Fabrik's modules

pub mod bazel;
pub mod logging;
pub mod nx;
pub mod storage;

// Re-export commonly used types
pub use storage::{create_storage, default_cache_dir, FilesystemStorage, Storage};
