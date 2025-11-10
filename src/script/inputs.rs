/// Input file handling for cache key generation
///
/// Handles glob expansion and file hashing with different strategies.
use anyhow::{Context, Result};
use glob::glob;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

use super::annotations::{HashMethod, InputSpec};

/// Result of hashing input files
#[derive(Debug, Clone)]
pub struct InputHash {
    #[allow(dead_code)] // May be used in future for detailed cache metadata
    pub files: Vec<PathBuf>,
    pub combined_hash: String,
}

/// Hash all input files according to their specifications
pub fn hash_inputs(inputs: &[InputSpec], base_dir: &Path) -> Result<Vec<InputHash>> {
    let mut results = Vec::new();

    for input in inputs {
        let input_hash = hash_input(input, base_dir)
            .with_context(|| format!("Failed to hash input: {}", input.path))?;
        results.push(input_hash);
    }

    Ok(results)
}

/// Hash a single input specification
fn hash_input(input: &InputSpec, base_dir: &Path) -> Result<InputHash> {
    // Expand glob pattern
    let files = expand_glob(&input.path, base_dir)?;

    if files.is_empty() {
        // Empty input is valid (might be optional files)
        return Ok(InputHash {
            files: vec![],
            combined_hash: String::from("empty"),
        });
    }

    // Hash each file and combine
    let mut hasher = Sha256::new();

    for file in &files {
        let file_hash = match input.hash {
            HashMethod::Content => hash_file_content(file)?,
            HashMethod::Mtime => hash_file_mtime(file)?,
            HashMethod::Size => hash_file_size(file)?,
        };

        // Include file path (relative to base_dir) in hash for uniqueness
        let rel_path = file
            .strip_prefix(base_dir)
            .unwrap_or(file)
            .to_string_lossy();
        hasher.update(rel_path.as_bytes());
        hasher.update(&file_hash);
    }

    let combined_hash = hex::encode(hasher.finalize());

    Ok(InputHash {
        files,
        combined_hash,
    })
}

/// Expand glob pattern relative to base directory
pub fn expand_glob(pattern: &str, base_dir: &Path) -> Result<Vec<PathBuf>> {
    // Make pattern relative to base_dir
    let full_pattern = if pattern.starts_with('/') {
        pattern.to_string()
    } else {
        base_dir.join(pattern).to_string_lossy().to_string()
    };

    let mut paths = Vec::new();

    for entry in
        glob(&full_pattern).with_context(|| format!("Invalid glob pattern: {}", pattern))?
    {
        let path = entry.with_context(|| format!("Failed to read glob entry for: {}", pattern))?;

        // Only include files (not directories)
        if path.is_file() {
            paths.push(path);
        }
    }

    // Sort for deterministic ordering
    paths.sort();

    Ok(paths)
}

/// Hash file contents using SHA256
fn hash_file_content(path: &Path) -> Result<Vec<u8>> {
    let content =
        fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))?;

    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(hasher.finalize().to_vec())
}

/// Hash file modification time
fn hash_file_mtime(path: &Path) -> Result<Vec<u8>> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to read metadata: {}", path.display()))?;

    let mtime = metadata
        .modified()
        .with_context(|| format!("Failed to get mtime: {}", path.display()))?;

    let timestamp = mtime
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let mut hasher = Sha256::new();
    hasher.update(timestamp.to_le_bytes());
    Ok(hasher.finalize().to_vec())
}

/// Hash file size
fn hash_file_size(path: &Path) -> Result<Vec<u8>> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to read metadata: {}", path.display()))?;

    let size = metadata.len();

    let mut hasher = Sha256::new();
    hasher.update(size.to_le_bytes());
    Ok(hasher.finalize().to_vec())
}

/// Get runtime version (e.g., bash --version)
pub fn get_runtime_version(runtime: &str) -> Result<String> {
    let output = std::process::Command::new(runtime)
        .arg("--version")
        .output()
        .with_context(|| format!("Failed to get version for runtime: {}", runtime))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Runtime version check failed: {}", runtime));
    }

    let version = String::from_utf8_lossy(&output.stdout);
    // Take first line only
    let first_line = version.lines().next().unwrap_or(&version);

    Ok(first_line.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_expand_glob() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        // Create test files
        fs::write(base.join("file1.txt"), "content1").unwrap();
        fs::write(base.join("file2.txt"), "content2").unwrap();
        fs::create_dir(base.join("subdir")).unwrap();
        fs::write(base.join("subdir/file3.txt"), "content3").unwrap();

        // Test simple glob
        let files = expand_glob("*.txt", base).unwrap();
        assert_eq!(files.len(), 2);

        // Test recursive glob
        let files = expand_glob("**/*.txt", base).unwrap();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_hash_file_content() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");
        fs::write(&file, "hello world").unwrap();

        let hash = hash_file_content(&file).unwrap();
        assert!(!hash.is_empty());

        // Same content = same hash
        let hash2 = hash_file_content(&file).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_input() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        fs::write(base.join("file1.txt"), "content1").unwrap();
        fs::write(base.join("file2.txt"), "content2").unwrap();

        let input = InputSpec {
            path: "*.txt".to_string(),
            hash: HashMethod::Content,
        };

        let result = hash_input(&input, base).unwrap();
        assert_eq!(result.files.len(), 2);
        assert!(!result.combined_hash.is_empty());
    }
}
