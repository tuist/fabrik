mod action_cache;
mod bytestream;
mod capabilities;
mod cas;
mod rpc_status;

pub use action_cache::BazelActionCacheService;
pub use bytestream::BazelByteStreamService;
pub use capabilities::BazelCapabilitiesService;
pub use cas::BazelCasService;

// Include generated proto code
pub mod proto {
    pub mod remote_execution {
        #![allow(dead_code)] // Allow unused structs in generated code
        tonic::include_proto!("build.bazel.remote.execution.v2");
    }

    pub mod bytestream {
        tonic::include_proto!("google.bytestream");
    }

    // Manual google.rpc module to avoid path issues
    pub mod google {
        pub mod rpc {
            pub use crate::bazel::rpc_status::Status;
        }
    }
}
