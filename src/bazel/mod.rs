mod action_cache;
mod cas;

pub use action_cache::BazelActionCacheService;
pub use cas::BazelCasService;

// Include generated proto code
pub mod proto {
    pub mod remote_execution {
        tonic::include_proto!("build.bazel.remote.execution.v2");
    }

    pub mod bytestream {
        tonic::include_proto!("google.bytestream");
    }

    pub mod google {
        pub mod rpc {
            tonic::include_proto!("google.rpc");
        }
    }
}
