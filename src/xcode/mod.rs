mod cas;
mod keyvalue;

pub use cas::CasService;
pub use keyvalue::KeyValueService;

// Include generated proto code
pub mod proto {
    pub mod cas {
        tonic::include_proto!("compilation_cache_service.cas.v1");
    }

    pub mod keyvalue {
        tonic::include_proto!("compilation_cache_service.keyvalue.v1");
    }
}
