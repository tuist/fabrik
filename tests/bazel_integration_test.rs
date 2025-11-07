// Bazel integration tests
//
// These tests verify Bazel integration with Fabrik daemon.
// Each test creates its own isolated daemon instance.
//
// To run: `cargo test --test bazel_integration_test -- --nocapture`

mod common;

use common::TestDaemon;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "Bazel has directory creation issues on Windows CI"
)]
fn test_bazel_cache_with_daemon() {
    // Start a test daemon
    let daemon = TestDaemon::start();

    // Create isolated Bazel output base
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let bazel_output_base = temp_dir.path().join("bazel_output");

    // Use simple fixture
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("bazel")
        .join("simple");

    // First build: should miss cache and populate it
    println!("\n=== First build (cache miss) ===");
    let output = Command::new("bazel")
        .arg(format!("--output_base={}", bazel_output_base.display()))
        .arg("build")
        .arg(format!("--remote_cache={}", daemon.grpc_url()))
        .arg("--remote_upload_local_results=true")
        .arg("//:hello")
        .current_dir(&fixture_path)
        .output()
        .expect("Failed to execute bazel build");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success(), "First bazel build should succeed");

    // Clean bazel local cache (but keep Fabrik cache)
    println!("\n=== Cleaning bazel local cache ===");
    std::fs::remove_dir_all(&bazel_output_base).ok();
    std::fs::create_dir(&bazel_output_base).expect("Failed to recreate output base");

    // Second build: should hit cache
    println!("\n=== Second build (cache hit) ===");
    let output2 = Command::new("bazel")
        .arg(format!("--output_base={}", bazel_output_base.display()))
        .arg("build")
        .arg(format!("--remote_cache={}", daemon.grpc_url()))
        .arg("--remote_upload_local_results=true")
        .arg("//:hello")
        .current_dir(&fixture_path)
        .output()
        .expect("Failed to execute bazel build");

    println!("stdout: {}", String::from_utf8_lossy(&output2.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output2.stderr));

    assert!(
        output2.status.success(),
        "Second bazel build should succeed"
    );

    // Verify cache was used
    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    let cache_hit = stderr2.contains("remote cache hit") || stderr2.contains("remote-cache hit");

    assert!(
        cache_hit,
        "Second build should have cache hits. stderr: {}",
        stderr2
    );

    println!("\n=== Test completed successfully - Bazel cache working! ===");
}

#[test]
#[cfg_attr(target_os = "windows", ignore = "Bazel not well supported on Windows")]
fn test_bazel_version_with_daemon() {
    let _daemon = TestDaemon::start();

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("bazel")
        .join("simple");

    println!("Running bazel version with Fabrik cache...");

    let output = Command::new("bazel")
        .arg("version")
        .current_dir(&fixture_path)
        .output()
        .expect("Failed to execute bazel version");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(
        output.status.success(),
        "Bazel version command should succeed"
    );

    // Verify output contains version info
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Build label:") || stdout.contains("bazel"));
}

#[test]
#[cfg_attr(target_os = "windows", ignore = "Bazel not well supported on Windows")]
fn test_bazel_help_with_daemon() {
    let _daemon = TestDaemon::start();

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("bazel")
        .join("simple");

    println!("Running bazel help...");

    let output = Command::new("bazel")
        .arg("help")
        .current_dir(&fixture_path)
        .output()
        .expect("Failed to execute bazel help");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));

    assert!(output.status.success(), "Bazel help should succeed");

    // Verify help output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Usage:") || stdout.contains("bazel"),
        "Should show bazel help"
    );
}
