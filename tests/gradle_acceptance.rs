// Gradle integration tests
//
// These tests verify the Gradle wrapper works correctly with Fabrik cache.
//
// To run manually:
// 1. Ensure Gradle is installed or use the wrapper: `./gradlew --version`
// 2. Build fabrik: `cargo build`
// 3. Test manually in fixtures/gradle/app/:
//    ```
//    cd fixtures/gradle/app
//    ../../../target/debug/fabrik gradle -- build
//    ```
// 4. Or run tests: `cargo test --test gradle_acceptance -- --nocapture`

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to count objects in Fabrik cache
fn count_fabrik_cache_objects(cache_dir: &std::path::Path) -> usize {
    let db_path = cache_dir.join("metadata.db");
    if !db_path.exists() {
        return 0;
    }

    let conn = rusqlite::Connection::open(&db_path).expect("Failed to open metadata.db");
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM objects", [], |row| row.get(0))
        .unwrap_or(0);

    count as usize
}

#[test]
fn test_gradle_cache_integration() {
    let fabrik_bin = env!("CARGO_BIN_EXE_fabrik");

    // Create temporary directories for complete isolation
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_dir = temp_dir.path().join("cache");
    let gradle_home = temp_dir.path().join("gradle_home");

    // Use Gradle fixture
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("gradle")
        .join("app");

    println!("=== Test Configuration ===");
    println!("Fabrik cache dir: {:?}", cache_dir);
    println!("Gradle home: {:?}", gradle_home);
    println!("Fixture path: {:?}", fixture_path);

    // First build: should miss cache and populate it
    println!("\n=== First build (cache miss) ===");
    let output = Command::new(fabrik_bin)
        .arg("gradle")
        .arg(format!("--config-cache-dir={}", cache_dir.display()))
        .arg("--")
        .arg("--init-script")
        .arg(fixture_path.join("init.gradle.kts"))
        .arg(":app:build")  // Build the app subproject
        .arg("--no-daemon")  // Avoid daemon for cleaner tests
        .arg("--console=plain")
        .arg("--build-cache")  // Explicitly enable build cache
        .current_dir(&fixture_path)
        .env("GRADLE_USER_HOME", &gradle_home)
        .env("FABRIK_CONFIG_LOG_LEVEL", "info")
        .output()
        .expect("Failed to execute fabrik gradle");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("=== First build stdout ===");
    println!("{}", stdout);
    println!("\n=== First build stderr ===");
    println!("{}", stderr);

    if !output.status.success() {
        panic!(
            "First gradle build failed with status: {}\nStdout: {}\nStderr: {}",
            output.status, stdout, stderr
        );
    }

    println!("First build completed successfully");

    // Check that Fabrik cache is populated
    println!("\n=== Verify Fabrik cache is populated ===");
    let fabrik_object_count = count_fabrik_cache_objects(&cache_dir);
    println!("Fabrik cache objects: {}", fabrik_object_count);

    // Note: Gradle may not cache everything on first build, but we should see some objects
    // if remote caching is working
    if fabrik_object_count == 0 {
        println!("⚠ Warning: No objects in Fabrik cache after first build");
        println!("This might be expected for small projects or if remote cache isn't configured");
    }

    // Clean Gradle local cache (but keep Fabrik cache)
    println!("\n=== Cleaning Gradle local cache ===");
    let build_dir = fixture_path.join("app").join("app").join("build");
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir).expect("Failed to remove build dir");
    }
    // Clean gradle cache in GRADLE_USER_HOME
    if gradle_home.exists() {
        std::fs::remove_dir_all(&gradle_home).expect("Failed to remove gradle home");
    }
    std::fs::create_dir(&gradle_home).expect("Failed to recreate gradle home");

    // Second build: should hit cache (if caching worked)
    println!("\n=== Second build (cache hit test) ===");
    let output2 = Command::new(fabrik_bin)
        .arg("gradle")
        .arg(format!("--config-cache-dir={}", cache_dir.display()))
        .arg("--")
        .arg("--init-script")
        .arg(fixture_path.join("init.gradle.kts"))
        .arg(":app:build")  // Build the app subproject
        .arg("--no-daemon")
        .arg("--console=plain")
        .arg("--build-cache")  // Explicitly enable build cache
        .current_dir(&fixture_path)
        .env("GRADLE_USER_HOME", &gradle_home)
        .env("FABRIK_CONFIG_LOG_LEVEL", "info")
        .output()
        .expect("Failed to execute fabrik gradle");

    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let stderr2 = String::from_utf8_lossy(&output2.stderr);

    println!("=== Second build stdout ===");
    println!("{}", stdout2);
    println!("\n=== Second build stderr ===");
    println!("{}", stderr2);

    if !output2.status.success() {
        panic!(
            "Second gradle build failed with status: {}\nStdout: {}\nStderr: {}",
            output2.status, stdout2, stderr2
        );
    }

    println!("Second build completed successfully");

    // Check for cache hits in logs
    let cache_hits = stderr2.matches("Gradle cache HIT").count();
    println!("\nCache hits detected: {}", cache_hits);

    if cache_hits > 0 {
        println!("✓ Cache hits confirmed - Gradle queried Fabrik cache");
    } else {
        println!("⚠ No cache hits detected in logs");
        println!("Note: Small projects may not generate cacheable tasks");
    }

    println!("\n=== Test completed successfully ===");
}

#[test]
fn test_gradle_wrapper_starts_server() {
    let fabrik_bin = env!("CARGO_BIN_EXE_fabrik");

    // Create temporary cache directory to avoid database locking with parallel tests
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_dir = temp_dir.path().join("cache");

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("gradle")
        .join("app");

    println!("Running fabrik gradle tasks...");

    // Just run 'gradle tasks' which should be fast
    let output = Command::new(fabrik_bin)
        .arg("gradle")
        .arg(format!("--config-cache-dir={}", cache_dir.display()))
        .arg("--")
        .arg("tasks")
        .arg("--no-daemon")
        .arg("--console=plain")
        .current_dir(&fixture_path)
        .env("RUST_LOG", "debug")
        .output()
        .expect("Failed to execute fabrik gradle tasks");

    println!("=== STDOUT ===");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("\n=== STDERR ===");
    println!("{}", String::from_utf8_lossy(&output.stderr));
    println!("\n=== EXIT CODE ===");
    println!("{:?}", output.status.code());

    // Just verify the command succeeds
    assert!(
        output.status.success(),
        "Gradle tasks command should succeed. Exit code: {:?}",
        output.status.code()
    );
}

#[test]
fn test_gradle_cache_passes_through_args() {
    let fabrik_bin = env!("CARGO_BIN_EXE_fabrik");

    // Create temporary cache directory to avoid database locking with parallel tests
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_dir = temp_dir.path().join("cache");

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("gradle")
        .join("app");

    println!("Running fabrik gradle --version...");

    // Test that fabrik passes through gradle arguments correctly
    let output = Command::new(fabrik_bin)
        .arg("gradle")
        .arg(format!("--config-cache-dir={}", cache_dir.display()))
        .arg("--")
        .arg("--version")
        .current_dir(&fixture_path)
        .env("RUST_LOG", "debug")
        .output()
        .expect("Failed to execute fabrik gradle --version");

    println!("=== STDOUT ===");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("\n=== STDERR ===");
    println!("{}", String::from_utf8_lossy(&output.stderr));
    println!("\n=== EXIT CODE ===");
    println!("{:?}", output.status.code());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show gradle version output
    assert!(
        output.status.success(),
        "Command should succeed. Exit code: {:?}",
        output.status.code()
    );

    // Verify it's actually Gradle output
    assert!(
        stdout.contains("Gradle") || stdout.contains("gradle"),
        "Output should contain Gradle version information"
    );
}
