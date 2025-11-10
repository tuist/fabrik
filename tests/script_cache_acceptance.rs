/// Acceptance tests for Fabrik script caching
///
/// These tests validate the end-to-end behavior of script caching using the
/// fixture scripts in fixtures/scripts/
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Helper to get the fabrik binary path with unique cache dir
fn fabrik_with_cache(cache_dir: &Path) -> Command {
    let mut cmd = Command::new(std::env!("CARGO_BIN_EXE_fabrik"));
    cmd.env("FABRIK_CONFIG_CACHE_DIR", cache_dir);
    cmd
}

/// Helper to set up a test workspace
struct TestWorkspace {
    temp_dir: TempDir,
    cache_dir: TempDir,
    fixtures_dir: PathBuf,
}

impl TestWorkspace {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = TempDir::new().unwrap();
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let fixtures_dir = project_root.join("fixtures/scripts");

        Self {
            temp_dir,
            cache_dir,
            fixtures_dir,
        }
    }

    fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    fn cache_path(&self) -> &Path {
        self.cache_dir.path()
    }

    fn fabrik(&self) -> Command {
        fabrik_with_cache(self.cache_path())
    }

    fn copy_script(&self, script_path: &str) -> PathBuf {
        let src = self.fixtures_dir.join(script_path);
        let dest = self
            .temp_dir
            .path()
            .join(Path::new(script_path).file_name().unwrap());

        fs::copy(&src, &dest).unwrap();

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dest).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dest, perms).unwrap();
        }

        dest
    }

    fn create_file(&self, path: &str, content: &str) {
        let file_path = self.temp_dir.path().join(path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(file_path, content).unwrap();
    }

    fn assert_file_exists(&self, path: &str) {
        let file_path = self.temp_dir.path().join(path);
        assert!(file_path.exists(), "File should exist: {}", path);
    }

    fn read_file(&self, path: &str) -> String {
        let file_path = self.temp_dir.path().join(path);
        fs::read_to_string(file_path).unwrap()
    }
}

#[test]
fn test_simple_bash_script_execution() {
    let workspace = TestWorkspace::new();
    let script = workspace.copy_script("bash/simple.sh");

    // First run - cache miss
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from simple bash script"))
        .stderr(predicate::str::contains("Cache key:"))
        .stderr(predicate::str::contains("MISS"));

    // Output should be created
    workspace.assert_file_exists("output.txt");

    // Second run - cache hit
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("HIT"));
}

#[test]
fn test_bash_script_with_inputs() {
    let workspace = TestWorkspace::new();
    let script = workspace.copy_script("bash/with-inputs.sh");

    // Create input file
    workspace.create_file("input.txt", "hello world");

    // First run
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success();

    // Check output
    workspace.assert_file_exists("output.txt");
    let output = workspace.read_file("output.txt");
    assert_eq!(output.trim(), "HELLO WORLD");

    // Second run - cache hit
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("HIT"));

    // Modify input - should invalidate cache
    workspace.create_file("input.txt", "goodbye world");

    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("MISS"));

    // Output should be updated
    let output = workspace.read_file("output.txt");
    assert_eq!(output.trim(), "GOODBYE WORLD");
}

#[test]
fn test_bash_script_with_env_vars() {
    let workspace = TestWorkspace::new();
    let script = workspace.copy_script("bash/with-env.sh");

    // First run
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success();

    workspace.assert_file_exists("env-output.txt");

    // Second run - cache hit
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("HIT"));

    // Change env var - should invalidate cache
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .env("USER", "testuser")
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("MISS"));
}

#[test]
fn test_bash_script_with_glob_inputs() {
    let workspace = TestWorkspace::new();
    let script = workspace.copy_script("bash/with-glob.sh");

    // Create source files
    workspace.create_file("src/file1.txt", "line1\nline2");
    workspace.create_file("src/file2.txt", "a\nb\nc");

    // First run
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success();

    // Check outputs
    workspace.assert_file_exists("dist/file1-lines.txt");
    workspace.assert_file_exists("dist/file2-lines.txt");

    // Second run - cache hit
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("HIT"));

    // Add new file - should invalidate cache
    workspace.create_file("src/file3.txt", "x\ny");

    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("MISS"));

    workspace.assert_file_exists("dist/file3-lines.txt");
}

#[test]
fn test_bash_script_dependencies() {
    let workspace = TestWorkspace::new();
    let build_script = workspace.copy_script("bash/build.sh");
    let deploy_script = workspace.copy_script("bash/deploy.sh");

    // Create source files for build
    workspace.create_file("source/file1.txt", "content1");
    workspace.create_file("source/file2.txt", "content2");

    // Run build first (dependencies not auto-executed yet)
    workspace
        .fabrik()
        .arg("run")
        .arg(&build_script)
        .current_dir(workspace.path())
        .assert()
        .success();

    // Run deploy
    workspace
        .fabrik()
        .arg("run")
        .arg(&deploy_script)
        .env("DEPLOY_ENV", "staging")
        .current_dir(workspace.path())
        .assert()
        .success();

    // Both outputs should exist
    workspace.assert_file_exists("build/manifest.txt");
    workspace.assert_file_exists("deploy.log");

    // Verify deploy log mentions build directory
    let deploy_log = workspace.read_file("deploy.log");
    assert!(deploy_log.contains("Build directory found"));

    // Second run - both should be cached
    workspace
        .fabrik()
        .arg("run")
        .arg(&deploy_script)
        .env("DEPLOY_ENV", "staging")
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("HIT"));
}

#[test]
fn test_cache_status_command() {
    let workspace = TestWorkspace::new();
    let script = workspace.copy_script("bash/simple.sh");

    // Run script to populate cache
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success();

    // Check status
    workspace
        .fabrik()
        .arg("cache")
        .arg("status")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: CACHED"))
        .stdout(predicate::str::contains("Exit code: 0"));
}

#[test]
fn test_cache_clean_command() {
    let workspace = TestWorkspace::new();
    let script = workspace.copy_script("bash/simple.sh");

    // Run script to populate cache
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success();

    // Verify cached
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("HIT"));

    // Clean cache
    workspace
        .fabrik()
        .arg("cache")
        .arg("clean")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Cache cleaned"));

    // Next run should be a miss
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("MISS"));
}

#[test]
fn test_cache_list_command() {
    let workspace = TestWorkspace::new();
    let script1 = workspace.copy_script("bash/simple.sh");
    let script2 = workspace.copy_script("bash/with-env.sh");

    // Run both scripts
    workspace
        .fabrik()
        .arg("run")
        .arg(&script1)
        .current_dir(workspace.path())
        .assert()
        .success();

    workspace
        .fabrik()
        .arg("run")
        .arg(&script2)
        .current_dir(workspace.path())
        .assert()
        .success();

    // List cache
    workspace
        .fabrik()
        .arg("cache")
        .arg("list")
        .current_dir(workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Cached scripts"))
        .stdout(predicate::str::contains("script-"));
}

#[test]
fn test_cache_stats_command() {
    let workspace = TestWorkspace::new();
    let script = workspace.copy_script("bash/simple.sh");

    // Run script
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success();

    // Get stats
    workspace
        .fabrik()
        .arg("cache")
        .arg("stats")
        .current_dir(workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Script Cache Statistics"))
        .stdout(predicate::str::contains("Total entries:"));
}

#[test]
fn test_no_cache_flag() {
    let workspace = TestWorkspace::new();
    let script = workspace.copy_script("bash/simple.sh");

    // First run to cache
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success();

    // Run with --no-cache should execute despite cache
    workspace
        .fabrik()
        .arg("run")
        .arg("--no-cache")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Caching disabled"));
}

#[test]
fn test_dry_run_flag() {
    let workspace = TestWorkspace::new();
    let script = workspace.copy_script("bash/simple.sh");

    // Dry run should not execute
    workspace
        .fabrik()
        .arg("run")
        .arg("--dry-run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Dry run"))
        .stderr(predicate::str::contains("would check cache"));

    // Output should not exist
    assert!(!workspace.path().join("output.txt").exists());
}

#[test]
fn test_simple_node_script() {
    let workspace = TestWorkspace::new();
    let script = workspace.copy_script("node/simple.js");

    // First run
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from simple Node script"));

    workspace.assert_file_exists("output.json");

    // Verify JSON content
    let output = workspace.read_file("output.json");
    assert!(output.contains("Generated by Fabrik"));

    // Second run - cache hit
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("HIT"));
}

#[test]
fn test_node_script_with_inputs() {
    let workspace = TestWorkspace::new();
    let script = workspace.copy_script("node/with-inputs.js");

    // Create package.json
    workspace.create_file(
        "package.json",
        r#"{"name": "test-pkg", "scripts": {"test": "echo test"}}"#,
    );

    // First run
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success();

    workspace.assert_file_exists("analysis.json");

    let analysis = workspace.read_file("analysis.json");
    assert!(analysis.contains("test-pkg"));
    assert!(analysis.contains("\"scriptCount\": 1"));

    // Second run - cache hit
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("HIT"));
}

#[test]
fn test_timeout_handling() {
    let workspace = TestWorkspace::new();
    let script = workspace.copy_script("bash/timeout-test.sh");

    // Should timeout after 2 seconds
    workspace
        .fabrik()
        .arg("run")
        .arg(&script)
        .current_dir(workspace.path())
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .failure()
        .stderr(predicate::str::contains("timed out"));
}

#[test]
fn test_verbose_flag() {
    let workspace = TestWorkspace::new();
    let script = workspace.copy_script("bash/simple.sh");

    workspace
        .fabrik()
        .arg("run")
        .arg("--verbose")
        .arg(&script)
        .current_dir(workspace.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("[fabrik] Parsing annotations"))
        .stderr(predicate::str::contains("[fabrik] Cache key:"))
        .stderr(predicate::str::contains("[fabrik] Executing:"));
}
