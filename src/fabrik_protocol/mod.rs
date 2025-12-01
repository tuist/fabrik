//! Fabrik Protocol - Inter-layer cache communication
//!
//! This module implements the Fabrik gRPC protocol for Layer 1 <-> Layer 2
//! communication. Layer 2 servers expose this protocol to accept cache
//! requests from Layer 1 daemons.
//!
//! The protocol is content-addressed: all artifacts are identified by SHA256 hash.

pub mod service;

pub use service::FabrikCacheService;

// Re-export generated proto types
pub mod proto {
    tonic::include_proto!("fabrik.v1");
}
