/// `fabrik cas` command implementation
///
/// Content-Addressed Storage (CAS) operations for blob storage.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::cli::{CasArgs, CasCommand};
use crate::cli_utils::fabrik_prefix;
use crate::storage::{default_cache_dir, FilesystemStorage, Storage};

// JSON output structures
#[derive(Serialize, Deserialize)]
struct GetOutput {
    hash: String,
    output_path: String,
    size_bytes: usize,
    success: bool,
}

#[derive(Serialize, Deserialize)]
struct PutOutput {
    hash: String,
    size_bytes: usize,
    success: bool,
}

#[derive(Serialize, Deserialize)]
struct ExistsOutput {
    hash: String,
    exists: bool,
}

#[derive(Serialize, Deserialize)]
struct DeleteOutput {
    hash: String,
    deleted: bool,
}

#[derive(Serialize, Deserialize)]
struct InfoOutput {
    hash: String,
    size_bytes: u64,
}

#[derive(Serialize, Deserialize)]
struct StatsOutput {
    total_objects: u64,
    total_bytes: u64,
    cache_dir: String,
}

pub async fn run(args: &CasArgs) -> Result<()> {
    let cache_dir = args
        .config_cache_dir
        .as_deref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(default_cache_dir);

    let storage = FilesystemStorage::new(&cache_dir)?;

    match &args.command {
        CasCommand::Get {
            hash,
            output,
            verbose,
            json,
        } => get(&storage, hash, output.as_deref(), *verbose, *json).await,
        CasCommand::Put {
            file,
            hash,
            verbose,
            json,
        } => put(&storage, file, hash.as_deref(), *verbose, *json).await,
        CasCommand::Exists { hash, json } => exists(&storage, hash, *json).await,
        CasCommand::Delete { hash, force, json } => delete(&storage, hash, *force, *json).await,
        CasCommand::Info { hash, json } => info(&storage, hash, *json).await,
        CasCommand::List { verbose, json } => list(&storage, *verbose, *json).await,
        CasCommand::Stats { json } => stats(&storage, *json).await,
    }
}

/// Get a blob from the cache by content hash
async fn get(
    storage: &FilesystemStorage,
    hash: &str,
    output_path: Option<&str>,
    verbose: bool,
    json: bool,
) -> Result<()> {
    use std::fs;
    use std::io::{self, Write};

    if verbose && !json {
        println!("{} Retrieving blob: {}", fabrik_prefix(), hash);
    }

    let data = storage
        .get(hash.as_bytes())
        .with_context(|| format!("Failed to retrieve blob: {}", hash))?;

    if let Some(data) = data {
        match output_path {
            Some(path) => {
                fs::write(path, &data).with_context(|| format!("Failed to write to: {}", path))?;

                if json {
                    let output = GetOutput {
                        hash: hash.to_string(),
                        output_path: path.to_string(),
                        size_bytes: data.len(),
                        success: true,
                    };
                    println!("{}", serde_json::to_string(&output)?);
                } else {
                    println!(
                        "{} Blob retrieved: {} ({} bytes)",
                        fabrik_prefix(),
                        hash,
                        data.len()
                    );
                    println!("{} Written to: {}", fabrik_prefix(), path);
                }
            }
            None => {
                // Write to stdout
                io::stdout().write_all(&data)?;
                io::stdout().flush()?;
            }
        }
        Ok(())
    } else {
        anyhow::bail!("Blob not found: {}", hash);
    }
}

/// Put a file into the cache (returns content hash)
async fn put(
    storage: &FilesystemStorage,
    input_path: &str,
    expected_hash: Option<&str>,
    verbose: bool,
    json: bool,
) -> Result<()> {
    use sha2::{Digest, Sha256};
    use std::fs;

    let data =
        fs::read(input_path).with_context(|| format!("Failed to read file: {}", input_path))?;
    let data_len = data.len();

    // Compute hash
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let computed_hash = format!("{:x}", hasher.finalize());

    // Verify if hash was provided
    if let Some(expected) = expected_hash {
        if computed_hash != expected {
            anyhow::bail!(
                "Hash mismatch: expected {} but computed {}",
                expected,
                computed_hash
            );
        }

        if verbose && !json {
            println!("{} Hash verified: {}", fabrik_prefix(), expected);
        }
    } else if verbose && !json {
        println!("{} Computed hash: {}", fabrik_prefix(), computed_hash);
    }

    if verbose && !json {
        println!("{} Storing blob: {}", fabrik_prefix(), computed_hash);
    }

    storage
        .put(computed_hash.as_bytes(), &data)
        .with_context(|| format!("Failed to store blob: {}", computed_hash))?;

    if json {
        let output = PutOutput {
            hash: computed_hash,
            size_bytes: data_len,
            success: true,
        };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!("{} Blob stored: {}", fabrik_prefix(), computed_hash);
        println!("{} Size: {} bytes", fabrik_prefix(), data_len);
    }

    Ok(())
}

/// Check if a blob exists in the cache
async fn exists(storage: &FilesystemStorage, hash: &str, json: bool) -> Result<()> {
    let exists = storage
        .exists(hash.as_bytes())
        .with_context(|| format!("Failed to check existence: {}", hash))?;

    if json {
        let output = ExistsOutput {
            hash: hash.to_string(),
            exists,
        };
        println!("{}", serde_json::to_string(&output)?);
        std::process::exit(if exists { 0 } else { 1 });
    } else if exists {
        println!("{} Blob exists: {}", fabrik_prefix(), hash);
        std::process::exit(0);
    } else {
        println!("{} Blob not found: {}", fabrik_prefix(), hash);
        std::process::exit(1);
    }
}

/// Delete a blob from the cache
async fn delete(storage: &FilesystemStorage, hash: &str, force: bool, json: bool) -> Result<()> {
    use std::io::{self, Write};

    if !force && !json {
        print!("Delete blob {}? [y/N]: ", hash);
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().eq_ignore_ascii_case("y") {
            println!("{} Deletion cancelled.", fabrik_prefix());
            return Ok(());
        }
    }

    storage
        .delete(hash.as_bytes())
        .with_context(|| format!("Failed to delete blob: {}", hash))?;

    if json {
        let output = DeleteOutput {
            hash: hash.to_string(),
            deleted: true,
        };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!("{} Blob deleted: {}", fabrik_prefix(), hash);
    }

    Ok(())
}

/// Show information about a cached blob
async fn info(storage: &FilesystemStorage, hash: &str, json: bool) -> Result<()> {
    // Check if blob exists
    let exists = storage
        .exists(hash.as_bytes())
        .with_context(|| format!("Failed to check existence: {}", hash))?;

    if !exists {
        anyhow::bail!("Blob not found: {}", hash);
    }

    // Get size
    let size = storage
        .size(hash.as_bytes())
        .with_context(|| format!("Failed to get size: {}", hash))?
        .ok_or_else(|| anyhow::anyhow!("Blob not found: {}", hash))?;

    if json {
        let output = InfoOutput {
            hash: hash.to_string(),
            size_bytes: size,
        };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!("{} Blob: {}", fabrik_prefix(), hash);
        println!(
            "{} Size: {} bytes ({:.2} MB)",
            fabrik_prefix(),
            size,
            size as f64 / 1_000_000.0
        );
    }

    Ok(())
}

/// List all cached blobs
async fn list(storage: &FilesystemStorage, verbose: bool, json: bool) -> Result<()> {
    let ids = storage.list_ids()?;

    if json {
        let blobs: Vec<_> = ids
            .iter()
            .map(|id| {
                let hash = hex::encode(id);
                if verbose {
                    let size = storage.size(id).ok().flatten().unwrap_or(0);
                    serde_json::json!({
                        "hash": hash,
                        "size_bytes": size,
                    })
                } else {
                    serde_json::json!({"hash": hash})
                }
            })
            .collect();
        println!("{}", serde_json::to_string(&blobs)?);
    } else {
        if ids.is_empty() {
            println!("No cached blobs.");
            return Ok(());
        }

        println!("Cached blobs ({} total):", ids.len());
        for id in ids {
            let hash = hex::encode(&id);
            if verbose {
                if let Ok(Some(size)) = storage.size(&id) {
                    println!("  {} ({:.2} MB)", hash, size as f64 / 1_000_000.0);
                } else {
                    println!("  {}", hash);
                }
            } else {
                println!("  {}", hash);
            }
        }
    }

    Ok(())
}

/// Show CAS storage statistics
async fn stats(storage: &FilesystemStorage, json: bool) -> Result<()> {
    let stats = storage.stats()?;

    if json {
        let output = StatsOutput {
            total_objects: stats.total_objects,
            total_bytes: stats.total_bytes,
            cache_dir: stats.cache_dir.display().to_string(),
        };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!("CAS Storage Statistics");
        println!();
        println!("Total blobs: {}", stats.total_objects);
        println!(
            "Total size: {:.2} MB",
            stats.total_bytes as f64 / 1_000_000.0
        );
        println!("Cache directory: {}", stats.cache_dir.display());

        if stats.total_objects > 0 {
            println!(
                "Average size per blob: {:.2} MB",
                (stats.total_bytes as f64 / stats.total_objects as f64) / 1_000_000.0
            );
        }
    }

    Ok(())
}
