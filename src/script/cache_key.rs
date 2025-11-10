/// Cache key generation for script caching
///
/// Generates content-addressed cache keys based on:
/// - Script content (normalized)
/// - Input files (hashed)
/// - Environment variables
/// - Runtime version (optional)
/// - Custom key component (optional)
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::Path;

use super::annotations::ScriptAnnotations;
use super::inputs::{get_runtime_version, hash_inputs};

/// Compute cache key for a script
///
/// The cache key is deterministic based on all inputs that affect the script's output.
/// Format: "script-{hex_hash}" where hex_hash is first 16 characters of SHA256.
pub fn compute_cache_key(script_path: &Path, annotations: &ScriptAnnotations) -> Result<String> {
    let mut hasher = Sha256::new();

    // 1. Hash normalized script content
    let script_content = normalize_script_content(script_path)?;
    hasher.update(script_content.as_bytes());

    // 2. Hash all input files
    let base_dir = script_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Script has no parent directory"))?;

    let input_hashes =
        hash_inputs(&annotations.inputs, base_dir).with_context(|| "Failed to hash input files")?;

    for input_hash in input_hashes {
        hasher.update(input_hash.combined_hash.as_bytes());
    }

    // 3. Hash environment variables
    for var in &annotations.env_vars {
        hasher.update(var.as_bytes());
        if let Ok(value) = env::var(var) {
            hasher.update(value.as_bytes());
        } else {
            // Variable not set - include marker to make key different
            hasher.update(b"<unset>");
        }
    }

    // 4. Include runtime version (if requested)
    if annotations.runtime_version {
        let version = get_runtime_version(&annotations.runtime)
            .with_context(|| format!("Failed to get runtime version: {}", annotations.runtime))?;
        hasher.update(version.as_bytes());
    }

    // 5. Custom cache key component
    if let Some(key) = &annotations.cache_key {
        hasher.update(key.as_bytes());
    }

    // 6. Include OS for cross-platform considerations
    hasher.update(std::env::consts::OS.as_bytes());

    let hash = hex::encode(hasher.finalize());

    // Use first 16 characters (64 bits) for shorter keys
    Ok(format!("script-{}", &hash[..16]))
}

/// Normalize script content by removing volatile directives
///
/// This ensures that changes to cache TTL or other non-functional metadata
/// don't invalidate the cache.
fn normalize_script_content(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read script: {}", path.display()))?;

    let mut normalized_lines = Vec::new();

    for line in content.lines() {
        // Exclude volatile directives that shouldn't affect cache key
        let trimmed = line.trim();

        // Skip cache TTL changes
        if trimmed.contains("cache ttl=") || trimmed.contains("cache disabled=") {
            continue;
        }

        // Skip cache key directive itself (to avoid circular dependency)
        if trimmed.contains("cache key=") {
            continue;
        }

        normalized_lines.push(line);
    }

    Ok(normalized_lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::annotations::{HashMethod, InputSpec};
    use tempfile::TempDir;

    #[test]
    fn test_normalize_script_content() {
        let temp = TempDir::new().unwrap();
        let script = temp.path().join("script.sh");

        let content = r#"#!/usr/bin/env -S fabrik run bash
#FABRIK input "src/*.ts"
#FABRIK cache ttl="7d"
#FABRIK cache key="v2"

echo "hello"
"#;

        fs::write(&script, content).unwrap();

        let normalized = normalize_script_content(&script).unwrap();

        // TTL and cache key lines should be excluded
        assert!(!normalized.contains("cache ttl="));
        assert!(!normalized.contains("cache key="));

        // Other lines should be present
        assert!(normalized.contains("#FABRIK input"));
        assert!(normalized.contains("echo"));
    }

    #[test]
    fn test_compute_cache_key_deterministic() {
        let temp = TempDir::new().unwrap();
        let script = temp.path().join("script.sh");

        let content = r#"#!/usr/bin/env -S fabrik run bash
#FABRIK input "*.txt"

echo "hello"
"#;

        fs::write(&script, content).unwrap();

        // Create input files
        fs::write(temp.path().join("file1.txt"), "content1").unwrap();

        let annotations = ScriptAnnotations {
            runtime: "bash".to_string(),
            runtime_args: vec![],
            inputs: vec![InputSpec {
                path: "*.txt".to_string(),
                hash: HashMethod::Content,
            }],
            outputs: vec![],
            env_vars: vec![],
            cache_ttl: None,
            cache_key: None,
            cache_disabled: false,
            runtime_version: false,
            exec_cwd: None,
            exec_timeout: None,
            exec_shell: false,
            depends_on: vec![],
        };

        let key1 = compute_cache_key(&script, &annotations).unwrap();
        let key2 = compute_cache_key(&script, &annotations).unwrap();

        // Should be deterministic
        assert_eq!(key1, key2);
        assert!(key1.starts_with("script-"));
    }

    #[test]
    fn test_compute_cache_key_changes_with_input() {
        let temp = TempDir::new().unwrap();
        let script = temp.path().join("script.sh");

        let content = r#"#!/usr/bin/env -S fabrik run bash
#FABRIK input "*.txt"

echo "hello"
"#;

        fs::write(&script, content).unwrap();
        fs::write(temp.path().join("file1.txt"), "content1").unwrap();

        let annotations = ScriptAnnotations {
            runtime: "bash".to_string(),
            runtime_args: vec![],
            inputs: vec![InputSpec {
                path: "*.txt".to_string(),
                hash: HashMethod::Content,
            }],
            outputs: vec![],
            env_vars: vec![],
            cache_ttl: None,
            cache_key: None,
            cache_disabled: false,
            runtime_version: false,
            exec_cwd: None,
            exec_timeout: None,
            exec_shell: false,
            depends_on: vec![],
        };

        let key1 = compute_cache_key(&script, &annotations).unwrap();

        // Change input file
        fs::write(temp.path().join("file1.txt"), "content2").unwrap();

        let key2 = compute_cache_key(&script, &annotations).unwrap();

        // Cache key should be different
        assert_ne!(key1, key2);
    }
}
