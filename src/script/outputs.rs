/// Output archiving and restoration
///
/// Handles creating tar+zstd archives of script outputs and extracting them for cache restoration.
use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use zstd::{decode_all, encode_all};

use super::annotations::OutputSpec;

/// Information about archived outputs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchivedOutput {
    pub path: String,
    pub artifact_hash: String,
    pub size_bytes: u64,
    pub file_count: usize,
    pub is_directory: bool,
}

/// Archive outputs to a tar+zstd file
pub fn archive_outputs(
    outputs: &[OutputSpec],
    base_dir: &Path,
    archive_path: &Path,
) -> Result<Vec<ArchivedOutput>> {
    // Create tar archive in memory
    let mut tar_data = Vec::new();
    let mut tar = Builder::new(&mut tar_data);

    let mut archived_outputs = Vec::new();

    for output in outputs {
        let output_path = if Path::new(&output.path).is_absolute() {
            PathBuf::from(&output.path)
        } else {
            base_dir.join(&output.path)
        };

        if !output_path.exists() {
            if output.required {
                return Err(anyhow::anyhow!(
                    "Required output not found: {}",
                    output.path
                ));
            } else {
                // Optional output missing - skip
                continue;
            }
        }

        let is_directory = output_path.is_dir();
        let (size, file_count) = if is_directory {
            // Archive directory recursively
            tar.append_dir_all(&output.path, &output_path)
                .with_context(|| format!("Failed to archive directory: {}", output.path))?;
            get_dir_size_and_count(&output_path)?
        } else {
            // Archive single file
            let mut file = File::open(&output_path)
                .with_context(|| format!("Failed to open file: {}", output.path))?;
            tar.append_file(&output.path, &mut file)
                .with_context(|| format!("Failed to archive file: {}", output.path))?;
            (file.metadata()?.len(), 1)
        };

        // Compute hash of output
        let hash = compute_path_hash(&output_path)?;

        archived_outputs.push(ArchivedOutput {
            path: output.path.clone(),
            artifact_hash: hash,
            size_bytes: size,
            file_count,
            is_directory,
        });
    }

    // Finish tar archive
    tar.finish().context("Failed to finalize tar archive")?;

    drop(tar); // Release mutable borrow

    // Compress with zstd
    let compressed =
        encode_all(tar_data.as_slice(), 3).context("Failed to compress archive with zstd")?;

    // Write to file
    let mut file = File::create(archive_path)
        .with_context(|| format!("Failed to create archive: {}", archive_path.display()))?;
    file.write_all(&compressed)
        .context("Failed to write compressed archive")?;

    Ok(archived_outputs)
}

/// Extract outputs from tar+zstd archive
pub fn extract_outputs(archive_path: &Path, base_dir: &Path) -> Result<()> {
    // Read compressed archive
    let compressed = fs::read(archive_path)
        .with_context(|| format!("Failed to read archive: {}", archive_path.display()))?;

    // Decompress
    let tar_data =
        decode_all(compressed.as_slice()).context("Failed to decompress archive with zstd")?;

    // Extract tar archive
    let mut archive = Archive::new(tar_data.as_slice());
    archive
        .unpack(base_dir)
        .with_context(|| format!("Failed to extract archive to: {}", base_dir.display()))?;

    Ok(())
}

/// Compute hash of file or directory
fn compute_path_hash(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    if path.is_dir() {
        // Hash all files in directory
        for entry in walkdir::WalkDir::new(path).sort_by_file_name() {
            let entry = entry?;
            if entry.file_type().is_file() {
                let content = fs::read(entry.path())?;
                hasher.update(&content);
            }
        }
    } else {
        // Hash single file
        let content = fs::read(path)?;
        hasher.update(&content);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Get total size and file count of a directory
fn get_dir_size_and_count(path: &Path) -> Result<(u64, usize)> {
    let mut total_size = 0;
    let mut file_count = 0;

    for entry in walkdir::WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            total_size += entry.metadata()?.len();
            file_count += 1;
        }
    }

    Ok((total_size, file_count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_archive_and_extract_single_file() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        // Create output file
        fs::write(base.join("output.txt"), "hello world").unwrap();

        let outputs = vec![OutputSpec {
            path: "output.txt".to_string(),
            required: true,
        }];

        let archive_path = base.join("outputs.tar.zst");

        // Archive
        let archived = archive_outputs(&outputs, base, &archive_path).unwrap();
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].path, "output.txt");
        assert!(!archived[0].is_directory);

        // Delete original
        fs::remove_file(base.join("output.txt")).unwrap();

        // Extract
        extract_outputs(&archive_path, base).unwrap();

        // Verify
        assert!(base.join("output.txt").exists());
        let content = fs::read_to_string(base.join("output.txt")).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_archive_and_extract_directory() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        // Create output directory
        fs::create_dir(base.join("dist")).unwrap();
        fs::write(base.join("dist/file1.txt"), "content1").unwrap();
        fs::write(base.join("dist/file2.txt"), "content2").unwrap();

        let outputs = vec![OutputSpec {
            path: "dist/".to_string(),
            required: true,
        }];

        let archive_path = base.join("outputs.tar.zst");

        // Archive
        let archived = archive_outputs(&outputs, base, &archive_path).unwrap();
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].path, "dist/");
        assert!(archived[0].is_directory);
        assert_eq!(archived[0].file_count, 2);

        // Delete original
        fs::remove_dir_all(base.join("dist")).unwrap();

        // Extract
        extract_outputs(&archive_path, base).unwrap();

        // Verify
        assert!(base.join("dist").exists());
        assert!(base.join("dist/file1.txt").exists());
        assert!(base.join("dist/file2.txt").exists());
    }

    #[test]
    fn test_optional_output_missing() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        let outputs = vec![OutputSpec {
            path: "nonexistent.txt".to_string(),
            required: false,
        }];

        let archive_path = base.join("outputs.tar.zst");

        // Should succeed with no outputs
        let archived = archive_outputs(&outputs, base, &archive_path).unwrap();
        assert_eq!(archived.len(), 0);
    }

    #[test]
    fn test_required_output_missing() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        let outputs = vec![OutputSpec {
            path: "nonexistent.txt".to_string(),
            required: true,
        }];

        let archive_path = base.join("outputs.tar.zst");

        // Should fail
        let result = archive_outputs(&outputs, base, &archive_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Required output not found"));
    }
}
