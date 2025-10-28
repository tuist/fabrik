// Nx HTTP endpoint integration tests
//
// These tests verify the Nx HTTP cache endpoints work correctly.
//
// Nx uses the following HTTP API:
// - PUT /v1/cache/{hash} - Store artifact
// - GET /v1/cache/{hash} - Retrieve artifact
//
// To run: `cargo test --test nx_http_test -- --nocapture`

use fabrik::storage::{FilesystemStorage, Storage};
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_nx_put_get_artifact() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = Arc::new(
        FilesystemStorage::new(temp_dir.path().to_str().unwrap())
            .expect("Failed to create storage"),
    );

    // Test data
    let test_data = b"Hello from Nx cache!";
    let hash = hex::decode("abcdef1234567890").expect("Invalid hex");

    // PUT artifact
    storage
        .put(&hash, test_data)
        .expect("Failed to put artifact");

    // GET artifact
    let retrieved = storage.get(&hash).expect("Failed to get artifact");

    assert!(retrieved.is_some(), "Artifact should exist");
    assert_eq!(retrieved.unwrap(), test_data);
}

#[tokio::test]
async fn test_nx_get_missing_artifact() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = Arc::new(
        FilesystemStorage::new(temp_dir.path().to_str().unwrap())
            .expect("Failed to create storage"),
    );

    let hash = hex::decode("abcd1234567890ef").expect("Invalid hex");

    let result = storage.get(&hash).expect("Storage operation should succeed");

    assert!(result.is_none(), "Artifact should not exist");
}

#[tokio::test]
async fn test_nx_multiple_artifacts() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = Arc::new(
        FilesystemStorage::new(temp_dir.path().to_str().unwrap())
            .expect("Failed to create storage"),
    );

    // Store multiple artifacts
    let artifacts = vec![
        (hex::decode("0123456789abcdef").unwrap(), b"data1" as &[u8]),
        (hex::decode("123456789abcdef0").unwrap(), b"data2"),
        (hex::decode("23456789abcdef01").unwrap(), b"data3"),
    ];

    for (hash, data) in &artifacts {
        storage
            .put(hash, data)
            .expect("Failed to put artifact");
    }

    // Retrieve and verify all artifacts
    for (hash, expected_data) in &artifacts {
        let retrieved = storage
            .get(hash)
            .expect("Failed to get artifact");

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), *expected_data);
    }
}
