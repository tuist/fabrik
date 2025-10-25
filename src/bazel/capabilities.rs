use super::proto::remote_execution::*;
use tonic::{Request, Response, Status};
use tracing::{debug, info};

/// Bazel Capabilities service implementation
pub struct BazelCapabilitiesService;

impl BazelCapabilitiesService {
    pub fn new() -> Self {
        Self
    }
}

#[tonic::async_trait]
impl capabilities_server::Capabilities for BazelCapabilitiesService {
    async fn get_capabilities(
        &self,
        request: Request<GetCapabilitiesRequest>,
    ) -> Result<Response<ServerCapabilities>, Status> {
        let req = request.into_inner();

        debug!("==> GetCapabilities - instance: {}", req.instance_name);

        // Return capabilities for cache-only server (no remote execution)
        let capabilities = ServerCapabilities {
            cache_capabilities: Some(CacheCapabilities {
                digest_functions: vec![digest_function::Value::Sha256 as i32],
                action_cache_update_capabilities: Some(ActionCacheUpdateCapabilities {
                    update_enabled: true,
                }),
                max_batch_total_size_bytes: 4 * 1024 * 1024, // 4MB max batch size
                supported_compressors: vec![compressor::Value::Identity as i32],
                supported_batch_update_compressors: vec![compressor::Value::Identity as i32],
            }),
            execution_capabilities: None, // We don't support remote execution
            deprecated_api_version: Some(SemVer {
                major: 2,
                minor: 0,
                patch: 0,
                prerelease: String::new(),
            }),
            low_api_version: Some(SemVer {
                major: 2,
                minor: 0,
                patch: 0,
                prerelease: String::new(),
            }),
            high_api_version: Some(SemVer {
                major: 2,
                minor: 2,
                patch: 0,
                prerelease: String::new(),
            }),
        };

        info!("<== GetCapabilities - Returned cache capabilities");

        Ok(Response::new(capabilities))
    }
}
