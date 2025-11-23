/// Script caching module
///
/// Provides Nx-style caching for arbitrary scripts using KDL annotations.
/// Scripts can declare inputs, outputs, environment dependencies, and cache behavior
/// inline using KDL comments.
pub mod annotations;
pub mod cache;
pub mod cache_key;
pub mod dependencies;
pub mod executor;
pub mod inputs;
pub mod outputs;

#[allow(unused_imports)]
pub use annotations::ScriptAnnotations;
#[allow(unused_imports)]
pub use cache::CreateMetadataParams;
