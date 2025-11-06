// Gradle acceptance tests
//
// These tests verify Gradle integration with Fabrik daemon.
// Each test creates its own isolated daemon instance.
//
// To run: `cargo test --test gradle_acceptance -- --nocapture`

mod common;

use common::TestDaemon;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_gradle_cache_with_daemon() {
    // Start a test daemon
    let daemon = TestDaemon::start();

    // Create isolated Gradle home
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let gradle_home = temp_dir.path().join("gradle_home");
    std::fs::create_dir(&gradle_home).expect("Failed to create gradle home");

    // Use Gradle fixture
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("gradle")
        .join("app");

    println!("\n=== Test Configuration ===");
    println!("Fabrik HTTP URL: {}", daemon.http_url());
    println!("Gradle home: {:?}", gradle_home);
    println!("Fixture path: {:?}", fixture_path);

    // First build: should miss cache and populate it
    println!("\n=== First build (cache miss) ===");
    let output = Command::new("gradle")
        .arg("build")
        .arg("--no-daemon")
        .arg("--console=plain")
        .arg("--build-cache")
        .current_dir(&fixture_path)
        .env("GRADLE_USER_HOME", &gradle_home)
        .env("GRADLE_BUILD_CACHE_URL", daemon.http_url())
        .output()
        .expect("Failed to execute gradle build");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(output.status.success(), "First gradle build should succeed");

    // Clean Gradle local cache
    println!("\n=== Cleaning Gradle local cache ===");
    let build_dir = fixture_path.join("build");
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir).ok();
    }
    std::fs::remove_dir_all(&gradle_home).ok();
    std::fs::create_dir(&gradle_home).expect("Failed to recreate gradle home");

    // Second build: should hit cache
    println!("\n=== Second build (cache hit) ===");
    let output2 = Command::new("gradle")
        .arg("build")
        .arg("--no-daemon")
        .arg("--console=plain")
        .arg("--build-cache")
        .current_dir(&fixture_path)
        .env("GRADLE_USER_HOME", &gradle_home)
        .env("GRADLE_BUILD_CACHE_URL", daemon.http_url())
        .output()
        .expect("Failed to execute gradle build");

    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let stderr2 = String::from_utf8_lossy(&output2.stderr);

    println!("stdout: {}", stdout2);
    println!("stderr: {}", stderr2);

    assert!(
        output2.status.success(),
        "Second gradle build should succeed"
    );

    println!("\n=== Test completed successfully ===");
}

#[test]
fn test_gradle_version_with_daemon() {
    let daemon = TestDaemon::start();

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("gradle")
        .join("app");

    println!("Running gradle --version with daemon...");

    let output = Command::new("gradle")
        .arg("--version")
        .current_dir(&fixture_path)
        .env("GRADLE_BUILD_CACHE_URL", daemon.http_url())
        .output()
        .expect("Failed to execute gradle --version");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));

    assert!(
        output.status.success(),
        "Gradle version command should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Gradle"));
}

#[test]
fn test_gradle_tasks_with_daemon() {
    let daemon = TestDaemon::start();

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("gradle")
        .join("app");

    println!("Running gradle tasks...");

    let output = Command::new("gradle")
        .arg("tasks")
        .arg("--no-daemon")
        .arg("--console=plain")
        .current_dir(&fixture_path)
        .env("GRADLE_BUILD_CACHE_URL", daemon.http_url())
        .output()
        .expect("Failed to execute gradle tasks");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));

    assert!(
        output.status.success(),
        "Gradle tasks command should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Build Setup tasks") || stdout.contains("tasks"));
}
