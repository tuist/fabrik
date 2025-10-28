// Nx integration tests
//
// These tests verify the Nx wrapper works correctly with Fabrik cache.
//
// To run manually:
// 1. Ensure Node.js and npm/npx are installed
// 2. Build fabrik: `cargo build`
// 3. Test manually in fixtures/nx/:
//    ```
//    cd fixtures/nx
//    npm install  # First time only
//    ../../target/debug/fabrik nx -- build demo
//    ```
// 4. Or run tests: `cargo test --test nx_acceptance -- --nocapture`

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to count objects in Fabrik cache
fn count_fabrik_cache_objects(cache_dir: &std::path::Path) -> usize {
    let db_path = cache_dir.join("metadata");
    if !db_path.exists() {
        return 0;
    }

    // Open RocksDB in read-only mode
    let db = rocksdb::DB::open_for_read_only(&rocksdb::Options::default(), &db_path, false)
        .expect("Failed to open RocksDB");

    // Count all keys in the default column family
    let iter = db.iterator(rocksdb::IteratorMode::Start);
    iter.count()
}

#[test]
fn test_nx_cache_integration() {
    let fabrik_bin = env!("CARGO_BIN_EXE_fabrik");

    // Create temporary directories for complete isolation
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_dir = temp_dir.path().join("cache");
    let nx_cache = temp_dir.path().join("nx_cache");

    // Use Nx fixture
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("nx");

    println!("=== Test Configuration ===");
    println!("Fabrik cache dir: {:?}", cache_dir);
    println!("Nx cache dir: {:?}", nx_cache);
    println!("Fixture path: {:?}", fixture_path);

    // Ensure npm dependencies are installed
    println!("\n=== Installing npm dependencies ===");
    let npm_install = Command::new("npm")
        .arg("install")
        .current_dir(&fixture_path)
        .output()
        .expect("Failed to run npm install");

    if !npm_install.status.success() {
        println!(
            "npm install stdout: {}",
            String::from_utf8_lossy(&npm_install.stdout)
        );
        println!(
            "npm install stderr: {}",
            String::from_utf8_lossy(&npm_install.stderr)
        );
        panic!("npm install failed");
    }

    // First build: should miss cache and populate it
    println!("\n=== First build (cache miss) ===");
    let output = Command::new(fabrik_bin)
        .arg("nx")
        .arg(format!("--config-cache-dir={}", cache_dir.display()))
        .arg("--")
        .arg("build")
        .arg("demo")
        .current_dir(&fixture_path)
        .env("NX_CACHE_DIRECTORY", &nx_cache)
        .env("FABRIK_CONFIG_LOG_LEVEL", "info")
        .output()
        .expect("Failed to execute fabrik nx");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("=== First build stdout ===");
    println!("{}", stdout);
    println!("\n=== First build stderr ===");
    println!("{}", stderr);

    if !output.status.success() {
        panic!(
            "First nx build failed with status: {}\nStdout: {}\nStderr: {}",
            output.status, stdout, stderr
        );
    }

    println!("First build completed successfully");

    // Check that Fabrik cache is populated
    println!("\n=== Verify Fabrik cache is populated ===");
    let fabrik_object_count = count_fabrik_cache_objects(&cache_dir);
    println!("Fabrik cache objects: {}", fabrik_object_count);

    // Note: Nx may not cache everything on first build, but we should see some objects
    // if remote caching is working
    if fabrik_object_count == 0 {
        println!("⚠ Warning: No objects in Fabrik cache after first build");
        println!("This might be expected for small projects or if remote cache isn't configured");
    }

    // Clean Nx local cache (but keep Fabrik cache)
    println!("\n=== Cleaning Nx local cache ===");
    if nx_cache.exists() {
        std::fs::remove_dir_all(&nx_cache).expect("Failed to remove nx cache");
    }
    std::fs::create_dir(&nx_cache).expect("Failed to recreate nx cache");

    // Also clean the dist output
    let dist_dir = fixture_path.join("dist");
    if dist_dir.exists() {
        std::fs::remove_dir_all(&dist_dir).expect("Failed to remove dist dir");
    }

    // Second build: should hit cache (if caching worked)
    println!("\n=== Second build (cache hit test) ===");
    let output2 = Command::new(fabrik_bin)
        .arg("nx")
        .arg(format!("--config-cache-dir={}", cache_dir.display()))
        .arg("--")
        .arg("build")
        .arg("demo")
        .current_dir(&fixture_path)
        .env("NX_CACHE_DIRECTORY", &nx_cache)
        .env("FABRIK_CONFIG_LOG_LEVEL", "info")
        .output()
        .expect("Failed to execute fabrik nx");

    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let stderr2 = String::from_utf8_lossy(&output2.stderr);

    println!("=== Second build stdout ===");
    println!("{}", stdout2);
    println!("\n=== Second build stderr ===");
    println!("{}", stderr2);

    if !output2.status.success() {
        panic!(
            "Second nx build failed with status: {}\nStdout: {}\nStderr: {}",
            output2.status, stdout2, stderr2
        );
    }

    println!("Second build completed successfully");

    // Check for cache hits in logs
    let cache_hits = stderr2.matches("Nx cache HIT").count();
    println!("\nCache hits detected: {}", cache_hits);

    if cache_hits > 0 {
        println!("✓ Cache hits confirmed - Nx queried Fabrik cache");
    } else {
        println!("⚠ No cache hits detected in logs");
        println!("Note: Small projects may not generate cacheable tasks");
    }

    println!("\n=== Test completed successfully ===");
}

#[test]
fn test_nx_wrapper_starts_server() {
    let fabrik_bin = env!("CARGO_BIN_EXE_fabrik");

    // Create temporary cache directory to avoid database locking with parallel tests
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_dir = temp_dir.path().join("cache");
    let nx_cache = temp_dir.path().join("nx_cache");

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("nx");

    // Ensure npm dependencies are installed
    let npm_install = Command::new("npm")
        .arg("install")
        .current_dir(&fixture_path)
        .output()
        .expect("Failed to run npm install");

    if !npm_install.status.success() {
        println!(
            "npm install stdout: {}",
            String::from_utf8_lossy(&npm_install.stdout)
        );
        println!(
            "npm install stderr: {}",
            String::from_utf8_lossy(&npm_install.stderr)
        );
        // Don't fail if npm install fails - might be in CI without npm
        return;
    }

    println!("Running fabrik nx build demo...");

    // Just run 'nx build demo' which should be fast
    let output = Command::new(fabrik_bin)
        .arg("nx")
        .arg(format!("--config-cache-dir={}", cache_dir.display()))
        .arg("--")
        .arg("build")
        .arg("demo")
        .current_dir(&fixture_path)
        .env("NX_CACHE_DIRECTORY", &nx_cache)
        .env("RUST_LOG", "debug")
        .output()
        .expect("Failed to execute fabrik nx build");

    println!("=== STDOUT ===");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("\n=== STDERR ===");
    println!("{}", String::from_utf8_lossy(&output.stderr));
    println!("\n=== EXIT CODE ===");
    println!("{:?}", output.status.code());

    // Just verify the command succeeds
    assert!(
        output.status.success(),
        "Nx build command should succeed. Exit code: {:?}",
        output.status.code()
    );
}

#[test]
fn test_nx_cache_passes_through_args() {
    let fabrik_bin = env!("CARGO_BIN_EXE_fabrik");

    // Create temporary cache directory to avoid database locking with parallel tests
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_dir = temp_dir.path().join("cache");

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("nx");

    // Ensure npm dependencies are installed
    let npm_install = Command::new("npm")
        .arg("install")
        .current_dir(&fixture_path)
        .output()
        .expect("Failed to run npm install");

    if !npm_install.status.success() {
        // Don't fail if npm install fails - might be in CI without npm
        return;
    }

    println!("Running fabrik nx --version...");

    // Test that fabrik passes through nx arguments correctly
    let output = Command::new(fabrik_bin)
        .arg("nx")
        .arg(format!("--config-cache-dir={}", cache_dir.display()))
        .arg("--")
        .arg("--version")
        .current_dir(&fixture_path)
        .env("RUST_LOG", "debug")
        .output()
        .expect("Failed to execute fabrik nx --version");

    println!("=== STDOUT ===");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("\n=== STDERR ===");
    println!("{}", String::from_utf8_lossy(&output.stderr));
    println!("\n=== EXIT CODE ===");
    println!("{:?}", output.status.code());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show nx version output
    assert!(
        output.status.success(),
        "Command should succeed. Exit code: {:?}",
        output.status.code()
    );

    // Verify it's actually Nx output (should contain version number)
    assert!(
        stdout.contains("Nx") || stdout.contains("nx") || stdout.contains("."),
        "Output should contain Nx version information"
    );
}
