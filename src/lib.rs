// Library interface for Fabrik
// This allows integration tests and external code to use Fabrik's modules

pub mod auth;
pub mod bazel;
pub mod capi; // C API (FFI) for external integrations
pub mod cli_utils;
pub mod config;
pub mod config_discovery;
pub mod config_expansion; // Environment variable expansion for config files
pub mod eviction; // Cache eviction policies (LRU, LFU, TTL)
pub mod fabrik_protocol; // Fabrik gRPC protocol (Layer 1 <-> Layer 2 communication)
pub mod hot_reload; // Hot-reload support for configuration
pub mod logging;
pub mod p2p; // P2P cache sharing
pub mod recipe; // Script recipes with content-addressed caching (bash, node, python, etc.)
pub mod recipe_portable; // Portable recipes executed in Fabrik's embedded JS runtime
pub mod storage;
pub mod xdg;

// Re-export commonly used types
pub use auth::AuthProvider;
pub use config::FabrikConfig;
pub use config_discovery::{discover_config, hash_config, DaemonState};
pub use eviction::{EvictionConfig, EvictionManager, EvictionPolicyType};
pub use fabrik_protocol::FabrikCacheService;
pub use recipe_portable::RecipeExecutor;
pub use storage::{
    create_storage, create_storage_with_eviction, default_cache_dir, FilesystemStorage, Storage,
};
