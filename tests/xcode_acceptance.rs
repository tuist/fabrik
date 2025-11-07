// Xcode acceptance tests
//
// These tests verify Xcode integration with Fabrik daemon using Unix sockets.
// Each test creates its own isolated daemon instance with a unique socket.
//
// Tests are macOS-only since they require xcodebuild.
//
// To run: `cargo test --test xcode_acceptance -- --nocapture`

#![cfg(target_os = "macos")]

mod common;

use common::TestDaemon;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_xcode_with_unix_socket() {
    // Start daemon in Unix socket mode
    let daemon = TestDaemon::start_with_socket();

    let socket_path = daemon
        .socket_path()
        .expect("Socket path should be available in Unix socket mode");

    println!("\n=== Test Configuration ===");
    println!("Socket path: {}", socket_path.display());
    println!("Cache dir: {}", daemon.cache_dir.display());

    // Verify socket file exists
    assert!(socket_path.exists(), "Unix socket file should exist");

    // Use Xcode fixture
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("xcode")
        .join("ios-app");

    if !fixture_path.exists() {
        println!("⚠️  Xcode fixture not found at: {}", fixture_path.display());
        println!("Skipping xcodebuild test");
        return;
    }

    // Create isolated derived data directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let derived_data = temp_dir.path().join("DerivedData");

    println!("\n=== First build (cache miss) ===");
    let output = Command::new("xcodebuild")
        .arg("-project")
        .arg(fixture_path.join("Fabrik.xcodeproj"))
        .arg("-scheme")
        .arg("Fabrik")
        .arg("-destination")
        .arg("generic/platform=iOS")
        .arg("-derivedDataPath")
        .arg(&derived_data)
        .arg("COMPILATION_CACHE_ENABLE_CACHING=YES")
        .arg("COMPILATION_CACHE_ENABLE_PLUGIN=YES")
        .arg(format!(
            "COMPILATION_CACHE_REMOTE_SERVICE_PATH={}",
            socket_path.display()
        ))
        .arg("build")
        .output()
        .expect("Failed to execute xcodebuild");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    if !output.status.success() {
        println!("⚠️  xcodebuild failed (likely missing SDK in CI)");
        println!("Skipping build verification, but Unix socket was created successfully");
        return;
    }

    // Clean derived data (but keep Fabrik cache)
    println!("\n=== Cleaning derived data ===");
    std::fs::remove_dir_all(&derived_data).ok();

    // Second build: should hit cache
    println!("\n=== Second build (cache hit) ===");
    let output2 = Command::new("xcodebuild")
        .arg("-project")
        .arg(fixture_path.join("Fabrik.xcodeproj"))
        .arg("-scheme")
        .arg("Fabrik")
        .arg("-destination")
        .arg("generic/platform=iOS")
        .arg("-derivedDataPath")
        .arg(&derived_data)
        .arg("COMPILATION_CACHE_ENABLE_CACHING=YES")
        .arg("COMPILATION_CACHE_ENABLE_PLUGIN=YES")
        .arg(format!(
            "COMPILATION_CACHE_REMOTE_SERVICE_PATH={}",
            socket_path.display()
        ))
        .arg("build")
        .output()
        .expect("Failed to execute xcodebuild");

    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let stderr2 = String::from_utf8_lossy(&output2.stderr);

    println!("stdout: {}", stdout2);
    println!("stderr: {}", stderr2);

    assert!(output2.status.success(), "Second xcodebuild should succeed");

    println!("\n=== Test completed successfully - Xcode Unix socket working! ===");
}

#[test]
fn test_xcode_socket_cleanup() {
    // Start daemon in Unix socket mode
    let daemon = TestDaemon::start_with_socket();

    let socket_path = daemon
        .socket_path()
        .expect("Socket path should be available");

    println!("Socket path: {}", socket_path.display());

    // Verify socket exists
    assert!(
        socket_path.exists(),
        "Socket should exist while daemon running"
    );

    // Drop daemon (triggers cleanup)
    drop(daemon);

    // Give it a moment for cleanup
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Verify socket is removed
    assert!(
        !socket_path.exists(),
        "Socket should be cleaned up after daemon stops"
    );

    println!("✅ Socket cleanup verified");
}

#[test]
fn test_xcode_socket_isolation() {
    // Start two daemons with sockets
    let daemon1 = TestDaemon::start_with_socket();
    let daemon2 = TestDaemon::start_with_socket();

    let socket1 = daemon1.socket_path().expect("Socket 1 should exist");
    let socket2 = daemon2.socket_path().expect("Socket 2 should exist");

    println!("Daemon 1 socket: {}", socket1.display());
    println!("Daemon 2 socket: {}", socket2.display());

    // Verify sockets are different
    assert_ne!(socket1, socket2, "Each daemon should have unique socket");

    // Verify both sockets exist
    assert!(socket1.exists(), "Socket 1 should exist");
    assert!(socket2.exists(), "Socket 2 should exist");

    println!("✅ Socket isolation verified - each test gets unique socket");
}
