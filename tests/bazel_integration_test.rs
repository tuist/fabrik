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

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[test]
#[cfg_attr(target_os = "windows", ignore = "Bazel has directory creation issues on Windows CI")]
fn test_bazel_cache_integration() {
    let fabrik_bin = env!("CARGO_BIN_EXE_fabrik");

    // Create temporary directories for complete isolation
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_dir = temp_dir.path().join("cache");
    let bazel_output_base = temp_dir.path().join("bazel_output");

    // Use simple fixture (no external dependencies)
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("bazel")
        .join("simple");

    // First build: should miss cache and populate it
    println!("=== First build (cache miss) ===");
    let output = Command::new(fabrik_bin)
        .arg("bazel")
        .arg(format!("--config-cache-dir={}", cache_dir.display()))
        .arg("--")
        .arg(format!("--output_base={}", bazel_output_base.display()))
        .arg("build")
        .arg("//:hello")
        .current_dir(&fixture_path)
        .env("FABRIK_CONFIG_LOG_LEVEL", "info")
        .output()
        .expect("Failed to execute fabrik bazel");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success(), "First bazel build should succeed");

    // Clean bazel local cache (but keep Fabrik cache)
    println!("\n=== Cleaning bazel local cache ===");
    std::fs::remove_dir_all(&bazel_output_base).ok();
    std::fs::create_dir(&bazel_output_base).expect("Failed to recreate output base");

    // Second build: should hit cache
    println!("\n=== Second build (cache hit) ===");
    let output2 = Command::new(fabrik_bin)
        .arg("bazel")
        .arg(format!("--config-cache-dir={}", cache_dir.display()))
        .arg("--")
        .arg(format!("--output_base={}", bazel_output_base.display()))
        .arg("build")
        .arg("//:hello")
        .current_dir(&fixture_path)
        .env("FABRIK_CONFIG_LOG_LEVEL", "info")
        .output()
        .expect("Failed to execute fabrik bazel");

    println!("stdout: {}", String::from_utf8_lossy(&output2.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output2.stderr));

    assert!(
        output2.status.success(),
        "Second bazel build should succeed"
    );

    // Verify Fabrik cache was queried by checking for GetActionResult calls
    // Logs are now in stderr with new structured logging format
    let stderr2 = String::from_utf8_lossy(&output2.stderr);

    let cache_queried = stderr2.contains("GetActionResult");

    assert!(
        cache_queried,
        "Second build should query cache for action results"
    );

    println!("\n=== Test completed successfully - Fabrik cache working! ===");
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
