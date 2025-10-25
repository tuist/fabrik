// Library interface for Fabrik
// This allows integration tests and external code to use Fabrik's modules

pub mod storage;

// Re-export commonly used types
pub use storage::{create_storage, FilesystemStorage, Storage};
