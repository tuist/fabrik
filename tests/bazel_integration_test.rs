// Bazel integration tests
//
// These tests are marked as #[ignore] because they require a properly configured
// Bazel environment and can be slow on first run.
//
// To run manually:
// 1. Ensure Bazel is installed: `bazel --version`
// 2. Build fabrik: `cargo build`
// 3. Test manually in fixtures/bazel/swift/:
//    ```
//    cd fixtures/bazel/swift
//    ../../../target/debug/fabrik bazel -- build //:hello
//    ```
// 4. Or run ignored tests: `cargo test --test bazel_integration_test -- --ignored --nocapture`

use std::process::Command;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_bazel_cache_integration() {
    // Get the fabrik binary path
    let fabrik_bin = env!("CARGO_BIN_EXE_fabrik");

    // Create temporary cache directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_dir = temp_dir.path().join("cache");
    std::fs::create_dir(&cache_dir).expect("Failed to create cache dir");

    // Get fixture project path
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("bazel")
        .join("swift");

    // Kill any existing Bazel server
    let _ = Command::new("bazel")
        .arg("shutdown")
        .current_dir(&fixture_path)
        .output();

    // Pre-fetch all dependencies (Swift rules, etc.) to avoid timeout
    println!("=== Fetching Bazel dependencies ===");
    let fetch_output = Command::new("bazel")
        .arg("fetch")
        .arg("//:hello")
        .current_dir(&fixture_path)
        .output()
        .expect("Failed to fetch dependencies");

    if !fetch_output.status.success() {
        println!("Fetch stdout: {}", String::from_utf8_lossy(&fetch_output.stdout));
        println!("Fetch stderr: {}", String::from_utf8_lossy(&fetch_output.stderr));
        panic!("Failed to fetch dependencies");
    }

    println!("Dependencies fetched successfully\n");

    // First build: should miss cache and populate it
    println!("=== First build (cache miss) ===");
    let output = Command::new(fabrik_bin)
        .arg("bazel")
        .arg(format!("--config-cache-dir={}", cache_dir.display()))
        .arg("--")
        .arg("build")
        .arg("//:hello")
        .current_dir(&fixture_path)
        .env("FABRIK_CONFIG_LOG_LEVEL", "debug")
        .output()
        .expect("Failed to execute fabrik bazel");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(
        output.status.success(),
        "First bazel build should succeed"
    );

    // Clean bazel output
    println!("\n=== Cleaning bazel output ===");
    let clean_output = Command::new("bazel")
        .arg("clean")
        .current_dir(&fixture_path)
        .output()
        .expect("Failed to clean bazel");

    assert!(
        clean_output.status.success(),
        "Bazel clean should succeed"
    );

    // Second build: should hit cache
    println!("\n=== Second build (cache hit) ===");
    let output2 = Command::new(fabrik_bin)
        .arg("bazel")
        .arg(format!("--config-cache-dir={}", cache_dir.display()))
        .arg("--")
        .arg("build")
        .arg("//:hello")
        .current_dir(&fixture_path)
        .env("FABRIK_CONFIG_LOG_LEVEL", "debug")
        .output()
        .expect("Failed to execute fabrik bazel");

    println!("stdout: {}", String::from_utf8_lossy(&output2.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output2.stderr));

    assert!(
        output2.status.success(),
        "Second bazel build should succeed"
    );

    // Verify cache hits occurred by checking the logs
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let stderr2 = String::from_utf8_lossy(&output2.stderr);

    // Look for GetActionResult HIT messages in the logs
    let has_cache_hits = stdout2.contains("Cache HIT") || stderr2.contains("Cache HIT");

    assert!(
        has_cache_hits,
        "Second build should have cache hits. Stdout: {}\nStderr: {}",
        stdout2,
        stderr2
    );

    println!("\n=== Test completed successfully - Cache hits verified! ===");
}

#[test]
fn test_bazel_wrapper_starts_server() {
    // This test verifies the wrapper starts without building
    let fabrik_bin = env!("CARGO_BIN_EXE_fabrik");

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("bazel")
        .join("swift");

    // Kill any existing Bazel server to avoid locking issues
    let _ = Command::new("bazel")
        .arg("shutdown")
        .current_dir(&fixture_path)
        .output();

    println!("Running fabrik bazel version...");

    // Just run 'bazel version' which should be fast
    let output = Command::new(fabrik_bin)
        .arg("bazel")
        .arg("--")
        .arg("version")
        .current_dir(&fixture_path)
        .env("RUST_LOG", "debug")
        .output()
        .expect("Failed to execute fabrik bazel version");

    println!("=== STDOUT ===");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("\n=== STDERR ===");
    println!("{}", String::from_utf8_lossy(&output.stderr));
    println!("\n=== EXIT CODE ===");
    println!("{:?}", output.status.code());

    // Just verify the command succeeds
    assert!(
        output.status.success(),
        "Bazel version command should succeed. Exit code: {:?}",
        output.status.code()
    );
}

#[test]
fn test_bazel_cache_passes_through_args() {
    let fabrik_bin = env!("CARGO_BIN_EXE_fabrik");

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("bazel")
        .join("swift");

    // Kill any existing Bazel server to avoid locking issues
    let _ = Command::new("bazel")
        .arg("shutdown")
        .current_dir(&fixture_path)
        .output();

    println!("Running fabrik bazel help...");

    // Test that fabrik passes through bazel arguments correctly
    let output = Command::new(fabrik_bin)
        .arg("bazel")
        .arg("--")
        .arg("help")
        .current_dir(&fixture_path)
        .env("RUST_LOG", "debug")
        .output()
        .expect("Failed to execute fabrik bazel help");

    println!("=== STDOUT ===");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("\n=== STDERR ===");
    println!("{}", String::from_utf8_lossy(&output.stderr));
    println!("\n=== EXIT CODE ===");
    println!("{:?}", output.status.code());

    // Should show bazel help output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command should succeed. Exit code: {:?}\nStdout: {}\nStderr: {}",
        output.status.code(),
        stdout,
        stderr
    );
}
