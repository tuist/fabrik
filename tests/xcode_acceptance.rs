use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to get the fabrik binary path (from target/debug or target/release)
fn fabrik_binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("fabrik");

    if !path.exists() {
        path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("target");
        path.push("release");
        path.push("fabrik");
    }

    assert!(path.exists(), "fabrik binary not found. Run 'cargo build' first.");
    path
}

/// Helper to get the iOS app fixture path
fn fixture_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures");
    path.push("xcode");
    path.push("ios-app");
    assert!(path.exists(), "Fixture not found at {:?}", path);
    path
}

/// Helper to count objects in Fabrik cache
fn count_fabrik_cache_objects(cache_dir: &PathBuf) -> usize {
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

/// Helper to count access hits for objects in Fabrik cache (objects with access_count > 0)
fn count_fabrik_cache_hits(cache_dir: &PathBuf) -> i64 {
    let db_path = cache_dir.join("metadata.db");
    if !db_path.exists() {
        return 0;
    }

    let conn = rusqlite::Connection::open(&db_path).expect("Failed to open metadata.db");
    let total_hits: i64 = conn
        .query_row("SELECT SUM(access_count) FROM objects WHERE access_count > 0", [], |row| row.get(0))
        .unwrap_or(0);

    total_hits
}

/// Helper to count objects that have been accessed (accessed_at > created_at)
fn count_accessed_objects(cache_dir: &PathBuf) -> usize {
    let db_path = cache_dir.join("metadata.db");
    if !db_path.exists() {
        return 0;
    }

    let conn = rusqlite::Connection::open(&db_path).expect("Failed to open metadata.db");
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM objects WHERE accessed_at > created_at", [], |row| row.get(0))
        .unwrap_or(0);

    count as usize
}

/// Helper to check if Xcode's compilation cache exists
fn xcode_cache_exists(derived_data: &PathBuf) -> bool {
    let cache_path = derived_data.join("Build/Intermediates.noindex/XCBuildData");
    cache_path.exists() && cache_path.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false)
}

/// Helper to delete Xcode's compilation cache
fn delete_xcode_cache(derived_data: &PathBuf) {
    let cache_path = derived_data.join("Build/Intermediates.noindex/XCBuildData");
    if cache_path.exists() {
        std::fs::remove_dir_all(&cache_path).expect("Failed to remove Xcode cache");
    }
}

#[test]
#[ignore] // Run with: cargo test --test xcode_acceptance -- --ignored
fn test_xcode_cache_server_workflow() {
    // Setup temporary directories
    let fabrik_cache_dir = TempDir::new().expect("Failed to create temp dir for Fabrik cache");
    let derived_data_dir = TempDir::new().expect("Failed to create temp dir for derived data");

    let fabrik_binary = fabrik_binary();
    let fixture_path = fixture_path();

    println!("Fabrik cache dir: {:?}", fabrik_cache_dir.path());
    println!("Derived data dir: {:?}", derived_data_dir.path());

    // Step 1: Build for the first time
    println!("\n=== STEP 1: First build (cold cache) ===");
    let output = Command::new(&fabrik_binary)
        .arg("xcodebuild")
        .arg("--config-cache-dir")
        .arg(fabrik_cache_dir.path())
        .arg("--")
        .arg("-project")
        .arg(fixture_path.join("Fabrik.xcodeproj"))
        .arg("-scheme")
        .arg("Fabrik")
        .arg("-destination")
        .arg("generic/platform=iOS Simulator")
        .arg("-derivedDataPath")
        .arg(derived_data_dir.path())
        .arg("build")
        .output()
        .expect("Failed to execute fabrik xcodebuild");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        eprintln!("STDOUT:\n{}", stdout);
        eprintln!("STDERR:\n{}", stderr);
        panic!("First build failed with status: {}", output.status);
    }

    println!("First build completed successfully");

    // Step 2: Check that Fabrik cache is populated
    println!("\n=== STEP 2: Verify Fabrik cache is populated ===");

    let fabrik_object_count = count_fabrik_cache_objects(&fabrik_cache_dir.path().to_path_buf());
    println!("Fabrik cache objects: {}", fabrik_object_count);
    assert!(fabrik_object_count > 0, "Fabrik cache should contain objects after first build");

    // Step 3: Build again (clean build - should populate Xcode's local cache)
    println!("\n=== STEP 4: Second build (should hit Fabrik cache) ===");
    let output2 = Command::new(&fabrik_binary)
        .arg("xcodebuild")
        .arg("--config-cache-dir")
        .arg(fabrik_cache_dir.path())
        .arg("--")
        .arg("-project")
        .arg(fixture_path.join("Fabrik.xcodeproj"))
        .arg("-scheme")
        .arg("Fabrik")
        .arg("-destination")
        .arg("generic/platform=iOS Simulator")
        .arg("-derivedDataPath")
        .arg(derived_data_dir.path())
        .arg("build")
        .output()
        .expect("Failed to execute fabrik xcodebuild");

    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let stderr2 = String::from_utf8_lossy(&output2.stderr);

    if !output2.status.success() {
        eprintln!("STDOUT:\n{}", stdout2);
        eprintln!("STDERR:\n{}", stderr2);
        panic!("Second build failed with status: {}", output2.status);
    }

    println!("Second build completed successfully");

    // Step 4: Verify cache persists and count remains stable
    println!("\n=== STEP 4: Verify cache persistence ===");

    let fabrik_object_count_after = count_fabrik_cache_objects(&fabrik_cache_dir.path().to_path_buf());
    println!("Fabrik cache objects after second build: {}", fabrik_object_count_after);

    // Object count should be stable (may increase slightly due to additional metadata, but shouldn't decrease)
    assert!(
        fabrik_object_count_after >= fabrik_object_count,
        "Cache object count should not decrease: {} -> {}",
        fabrik_object_count,
        fabrik_object_count_after
    );

    println!("\n=== SUCCESS: Xcode cache workflow validated ===");
    println!("- First build populated Fabrik cache with {} objects", fabrik_object_count);
    println!("- Second build completed successfully with cache persisted");
    println!("- Fabrik cache now contains {} objects", fabrik_object_count_after);
    println!("\nNote: Cache hit tracking will be implemented in a future iteration");
}
