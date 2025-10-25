/// Integration test for GitHub Actions Cache storage backend
///
/// This test only runs when ACTIONS_CACHE_URL and ACTIONS_RUNTIME_TOKEN
/// environment variables are present (i.e., in GitHub Actions CI)

use std::env;

#[test]
fn test_github_actions_cache_integration() {
    // Check if we're in GitHub Actions with cache enabled
    let cache_url = env::var("ACTIONS_CACHE_URL");
    let runtime_token = env::var("ACTIONS_RUNTIME_TOKEN");

    if cache_url.is_err() || runtime_token.is_err() {
        println!("⚠ Skipping GitHub Actions cache integration test");
        println!("  ACTIONS_CACHE_URL: {:?}", cache_url.err());
        println!("  ACTIONS_RUNTIME_TOKEN: {:?}", runtime_token.err());
        println!("  This test only runs in GitHub Actions CI with cache enabled");
        return;
    }

    println!("✓ GitHub Actions cache environment detected");
    println!("  ACTIONS_CACHE_URL: present");
    println!("  ACTIONS_RUNTIME_TOKEN: present");

    // Test storage backend auto-detection
    let temp_dir = tempfile::TempDir::new().unwrap();
    let backend = fabrik::storage::StorageBackend::auto_detect(temp_dir.path().to_str().unwrap())
        .expect("Failed to auto-detect storage backend");

    match backend {
        fabrik::storage::StorageBackend::GithubActions(_) => {
            println!("✓ Auto-detected GitHub Actions storage backend");
        }
        fabrik::storage::StorageBackend::Filesystem(_) => {
            panic!(
                "Expected GitHub Actions storage backend, but got Filesystem. \
                 ACTIONS_CACHE_URL and ACTIONS_RUNTIME_TOKEN are present but not detected."
            );
        }
    }

    // Test basic put/get operations
    use fabrik::storage::Storage;

    let test_key = b"test-key-12345";
    let test_data = b"Hello from GitHub Actions cache!";

    println!("✓ Testing PUT operation...");
    backend
        .put(test_key, test_data)
        .expect("Failed to put data to GitHub Actions cache");

    println!("✓ Testing GET operation...");
    let retrieved = backend
        .get(test_key)
        .expect("Failed to get data from GitHub Actions cache");

    match retrieved {
        Some(data) => {
            println!("✓ Successfully retrieved data from cache");
            assert_eq!(
                data, test_data,
                "Retrieved data doesn't match stored data"
            );
            println!("✓ Data integrity verified");
        }
        None => {
            panic!(
                "Data not found in cache after PUT. GitHub Actions cache may have restrictions."
            );
        }
    }

    println!("✓ GitHub Actions cache integration test PASSED");
}
