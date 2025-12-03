// Layer 2 Regional Server acceptance tests
//
// These tests verify the Fabrik Protocol gRPC server functionality
// for Layer 2 (regional) cache servers.
//
// Each test creates its own isolated server instance with a unique port.
//
// To run: `cargo test --test layer2_server_acceptance -- --nocapture`

mod common;

use common::TestServer;
use sha2::{Digest, Sha256};

/// Helper to compute SHA256 hash of data and return as hex string
fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[tokio::test]
async fn test_layer2_server_exists_not_found() {
    // Start a Layer 2 regional server
    let server = TestServer::start();

    println!("\n=== Test Configuration ===");
    println!("gRPC URL: {}", server.grpc_url());
    println!("Cache dir: {}", server.cache_dir.display());

    // Connect to the server using gRPC client
    let mut client =
        fabrik::fabrik_protocol::proto::fabrik_cache_client::FabrikCacheClient::connect(
            server.grpc_url(),
        )
        .await
        .expect("Failed to connect to server");

    // Check for a hash that doesn't exist
    let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let request = fabrik::fabrik_protocol::proto::ExistsRequest {
        hash: hash.to_string(),
    };

    let response = client.exists(request).await.expect("Exists request failed");
    let inner = response.into_inner();

    assert!(!inner.exists, "Artifact should not exist");
    assert_eq!(
        inner.size_bytes, 0,
        "Size should be 0 for non-existent artifact"
    );

    println!("✅ Exists returns false for non-existent hash");
}

#[tokio::test]
async fn test_layer2_server_put_and_exists() {
    // Start a Layer 2 regional server
    let server = TestServer::start();

    println!("\n=== Test Configuration ===");
    println!("gRPC URL: {}", server.grpc_url());
    println!("Cache dir: {}", server.cache_dir.display());

    // Connect to the server using gRPC client
    let mut client =
        fabrik::fabrik_protocol::proto::fabrik_cache_client::FabrikCacheClient::connect(
            server.grpc_url(),
        )
        .await
        .expect("Failed to connect to server");

    // Create test data
    let test_data = b"Hello, Fabrik Layer 2!";
    let hash = compute_hash(test_data);

    println!("Test data: {:?}", String::from_utf8_lossy(test_data));
    println!("Hash: {}", hash);

    // PUT the data
    let put_requests = vec![fabrik::fabrik_protocol::proto::PutRequest {
        hash: hash.clone(),
        chunk: test_data.to_vec(),
        metadata: std::collections::HashMap::new(),
    }];

    let put_stream = tokio_stream::iter(put_requests);
    let put_response = client.put(put_stream).await.expect("Put request failed");
    let put_inner = put_response.into_inner();

    assert!(put_inner.success, "Put should succeed");
    assert_eq!(
        put_inner.size_bytes,
        test_data.len() as i64,
        "Size should match data length"
    );

    println!("✅ Put succeeded with {} bytes", put_inner.size_bytes);

    // Now check EXISTS
    let exists_request = fabrik::fabrik_protocol::proto::ExistsRequest { hash: hash.clone() };

    let exists_response = client
        .exists(exists_request)
        .await
        .expect("Exists request failed");
    let exists_inner = exists_response.into_inner();

    assert!(exists_inner.exists, "Artifact should exist after PUT");
    assert_eq!(
        exists_inner.size_bytes,
        test_data.len() as i64,
        "Size should match data length"
    );

    println!("✅ Exists returns true after PUT");
}

#[tokio::test]
async fn test_layer2_server_put_and_get() {
    // Start a Layer 2 regional server
    let server = TestServer::start();

    println!("\n=== Test Configuration ===");
    println!("gRPC URL: {}", server.grpc_url());
    println!("Cache dir: {}", server.cache_dir.display());

    // Connect to the server using gRPC client
    let mut client =
        fabrik::fabrik_protocol::proto::fabrik_cache_client::FabrikCacheClient::connect(
            server.grpc_url(),
        )
        .await
        .expect("Failed to connect to server");

    // Create test data
    let test_data = b"This is test data for Layer 2 cache verification";
    let hash = compute_hash(test_data);

    println!("Test data: {:?}", String::from_utf8_lossy(test_data));
    println!("Hash: {}", hash);

    // PUT the data
    let put_requests = vec![fabrik::fabrik_protocol::proto::PutRequest {
        hash: hash.clone(),
        chunk: test_data.to_vec(),
        metadata: std::collections::HashMap::new(),
    }];

    let put_stream = tokio_stream::iter(put_requests);
    let put_response = client.put(put_stream).await.expect("Put request failed");

    assert!(put_response.into_inner().success, "Put should succeed");
    println!("✅ Put succeeded");

    // GET the data back
    let get_request = fabrik::fabrik_protocol::proto::GetRequest { hash: hash.clone() };

    let get_response = client.get(get_request).await.expect("Get request failed");

    // Collect all chunks from the stream
    let mut stream = get_response.into_inner();
    let mut received_data = Vec::new();

    while let Some(chunk) = stream.message().await.expect("Failed to receive chunk") {
        received_data.extend(chunk.chunk);
    }

    assert_eq!(
        received_data.as_slice(),
        test_data,
        "Retrieved data should match original"
    );

    println!(
        "✅ Get returned correct data: {:?}",
        String::from_utf8_lossy(&received_data)
    );
}

#[tokio::test]
async fn test_layer2_server_get_not_found() {
    // Start a Layer 2 regional server
    let server = TestServer::start();

    // Connect to the server using gRPC client
    let mut client =
        fabrik::fabrik_protocol::proto::fabrik_cache_client::FabrikCacheClient::connect(
            server.grpc_url(),
        )
        .await
        .expect("Failed to connect to server");

    // Try to GET a non-existent hash
    let hash = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    let get_request = fabrik::fabrik_protocol::proto::GetRequest {
        hash: hash.to_string(),
    };

    let result = client.get(get_request).await;

    assert!(result.is_err(), "Get should fail for non-existent hash");

    let status = result.unwrap_err();
    assert_eq!(
        status.code(),
        tonic::Code::NotFound,
        "Should return NotFound error"
    );

    println!("✅ Get returns NotFound for non-existent hash");
}

#[tokio::test]
async fn test_layer2_server_delete() {
    // Start a Layer 2 regional server
    let server = TestServer::start();

    // Connect to the server using gRPC client
    let mut client =
        fabrik::fabrik_protocol::proto::fabrik_cache_client::FabrikCacheClient::connect(
            server.grpc_url(),
        )
        .await
        .expect("Failed to connect to server");

    // Create and store test data
    let test_data = b"Data to be deleted";
    let hash = compute_hash(test_data);

    let put_requests = vec![fabrik::fabrik_protocol::proto::PutRequest {
        hash: hash.clone(),
        chunk: test_data.to_vec(),
        metadata: std::collections::HashMap::new(),
    }];

    let put_stream = tokio_stream::iter(put_requests);
    client.put(put_stream).await.expect("Put request failed");

    // Verify it exists
    let exists_request = fabrik::fabrik_protocol::proto::ExistsRequest { hash: hash.clone() };
    let exists_response = client.exists(exists_request).await.unwrap();
    assert!(
        exists_response.into_inner().exists,
        "Should exist before delete"
    );

    // DELETE the artifact
    let delete_request = fabrik::fabrik_protocol::proto::DeleteRequest { hash: hash.clone() };

    let delete_response = client
        .delete(delete_request)
        .await
        .expect("Delete request failed");
    let delete_inner = delete_response.into_inner();

    assert!(delete_inner.success, "Delete should succeed");
    assert!(delete_inner.existed, "Artifact should have existed");

    // Verify it no longer exists
    let exists_request2 = fabrik::fabrik_protocol::proto::ExistsRequest { hash: hash.clone() };
    let exists_response2 = client.exists(exists_request2).await.unwrap();
    assert!(
        !exists_response2.into_inner().exists,
        "Should not exist after delete"
    );

    println!("✅ Delete successfully removes artifact");
}

#[tokio::test]
async fn test_layer2_server_get_stats() {
    // Start a Layer 2 regional server
    let server = TestServer::start();

    // Connect to the server using gRPC client
    let mut client =
        fabrik::fabrik_protocol::proto::fabrik_cache_client::FabrikCacheClient::connect(
            server.grpc_url(),
        )
        .await
        .expect("Failed to connect to server");

    // Initial stats
    let stats_request = fabrik::fabrik_protocol::proto::GetStatsRequest {
        since_timestamp: None,
    };

    let stats_response = client
        .get_stats(stats_request)
        .await
        .expect("GetStats request failed");
    let stats = stats_response.into_inner();

    println!("Initial stats:");
    println!("  Cache hits: {}", stats.cache_hits);
    println!("  Cache misses: {}", stats.cache_misses);
    println!("  Artifact count: {}", stats.artifact_count);
    println!("  Total bytes: {}", stats.total_bytes);
    println!("  Uptime seconds: {}", stats.uptime_seconds);

    // Store some data
    let test_data = b"Stats test data";
    let hash = compute_hash(test_data);

    let put_requests = vec![fabrik::fabrik_protocol::proto::PutRequest {
        hash: hash.clone(),
        chunk: test_data.to_vec(),
        metadata: std::collections::HashMap::new(),
    }];

    let put_stream = tokio_stream::iter(put_requests);
    client.put(put_stream).await.expect("Put failed");

    // Do an EXISTS (should be a hit)
    let exists_request = fabrik::fabrik_protocol::proto::ExistsRequest { hash: hash.clone() };
    client.exists(exists_request).await.expect("Exists failed");

    // Do an EXISTS for non-existent (should be a miss)
    let miss_request = fabrik::fabrik_protocol::proto::ExistsRequest {
        hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    };
    client.exists(miss_request).await.expect("Exists failed");

    // Get updated stats
    let stats_request2 = fabrik::fabrik_protocol::proto::GetStatsRequest {
        since_timestamp: None,
    };

    let stats_response2 = client
        .get_stats(stats_request2)
        .await
        .expect("GetStats request failed");
    let stats2 = stats_response2.into_inner();

    println!("\nUpdated stats:");
    println!("  Cache hits: {}", stats2.cache_hits);
    println!("  Cache misses: {}", stats2.cache_misses);
    println!("  Artifact count: {}", stats2.artifact_count);
    println!("  Total bytes: {}", stats2.total_bytes);

    // Verify stats were updated
    assert!(stats2.cache_hits >= 1, "Should have at least 1 cache hit");
    assert!(
        stats2.cache_misses >= 1,
        "Should have at least 1 cache miss"
    );

    println!("✅ GetStats returns correct metrics");
}

#[tokio::test]
async fn test_layer2_server_large_artifact() {
    // Start a Layer 2 regional server
    let server = TestServer::start();

    // Connect to the server using gRPC client
    let mut client =
        fabrik::fabrik_protocol::proto::fabrik_cache_client::FabrikCacheClient::connect(
            server.grpc_url(),
        )
        .await
        .expect("Failed to connect to server");

    // Create a larger test artifact (1MB)
    let test_data: Vec<u8> = (0..1024 * 1024).map(|i| (i % 256) as u8).collect();
    let hash = compute_hash(&test_data);

    println!("Large artifact size: {} bytes", test_data.len());
    println!("Hash: {}", hash);

    // PUT the data (send in chunks for streaming)
    const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks
    let mut put_requests = Vec::new();

    for (i, chunk) in test_data.chunks(CHUNK_SIZE).enumerate() {
        put_requests.push(fabrik::fabrik_protocol::proto::PutRequest {
            hash: if i == 0 { hash.clone() } else { String::new() },
            chunk: chunk.to_vec(),
            metadata: std::collections::HashMap::new(),
        });
    }

    let put_stream = tokio_stream::iter(put_requests);
    let put_response = client.put(put_stream).await.expect("Put request failed");
    let put_inner = put_response.into_inner();

    assert!(put_inner.success, "Put should succeed");
    assert_eq!(
        put_inner.size_bytes,
        test_data.len() as i64,
        "Size should match"
    );

    println!("✅ Put large artifact succeeded");

    // GET the data back
    let get_request = fabrik::fabrik_protocol::proto::GetRequest { hash: hash.clone() };

    let get_response = client.get(get_request).await.expect("Get request failed");

    // Collect all chunks from the stream
    let mut stream = get_response.into_inner();
    let mut received_data = Vec::new();

    while let Some(chunk) = stream.message().await.expect("Failed to receive chunk") {
        received_data.extend(chunk.chunk);
    }

    assert_eq!(
        received_data.len(),
        test_data.len(),
        "Received data size should match"
    );
    assert_eq!(
        received_data.as_slice(),
        test_data.as_slice(),
        "Retrieved data should match original"
    );

    println!("✅ Get large artifact returned correct data");
}

#[tokio::test]
async fn test_layer2_server_isolation() {
    // Start two independent Layer 2 servers
    let server1 = TestServer::start();
    let server2 = TestServer::start();

    assert_ne!(
        server1.grpc_port, server2.grpc_port,
        "Servers should have different ports"
    );

    println!("Server 1 port: {}", server1.grpc_port);
    println!("Server 2 port: {}", server2.grpc_port);

    // Connect to both servers
    let mut client1 =
        fabrik::fabrik_protocol::proto::fabrik_cache_client::FabrikCacheClient::connect(
            server1.grpc_url(),
        )
        .await
        .expect("Failed to connect to server 1");

    let mut client2 =
        fabrik::fabrik_protocol::proto::fabrik_cache_client::FabrikCacheClient::connect(
            server2.grpc_url(),
        )
        .await
        .expect("Failed to connect to server 2");

    // Store data in server 1 only
    let test_data = b"Data only in server 1";
    let hash = compute_hash(test_data);

    let put_requests = vec![fabrik::fabrik_protocol::proto::PutRequest {
        hash: hash.clone(),
        chunk: test_data.to_vec(),
        metadata: std::collections::HashMap::new(),
    }];

    let put_stream = tokio_stream::iter(put_requests);
    client1.put(put_stream).await.expect("Put failed");

    // Verify it exists in server 1
    let exists_request1 = fabrik::fabrik_protocol::proto::ExistsRequest { hash: hash.clone() };
    let exists1 = client1.exists(exists_request1).await.unwrap();
    assert!(exists1.into_inner().exists, "Should exist in server 1");

    // Verify it does NOT exist in server 2
    let exists_request2 = fabrik::fabrik_protocol::proto::ExistsRequest { hash: hash.clone() };
    let exists2 = client2.exists(exists_request2).await.unwrap();
    assert!(!exists2.into_inner().exists, "Should NOT exist in server 2");

    println!("✅ Server isolation verified - data is not shared between instances");
}
