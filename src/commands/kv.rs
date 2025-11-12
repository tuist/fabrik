/// `fabrik kv` command implementation
///
/// Key-Value storage operations for action cache and metadata.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::cli::{KvArgs, KvCommand};
use crate::cli_utils::fabrik_prefix;
use crate::storage::{default_cache_dir, FilesystemStorage, Storage};

// JSON output structures
#[derive(Serialize, Deserialize)]
struct GetOutput {
    key: String,
    value_bytes: usize,
    success: bool,
}

#[derive(Serialize, Deserialize)]
struct PutOutput {
    key: String,
    value_bytes: usize,
    success: bool,
}

#[derive(Serialize, Deserialize)]
struct ExistsOutput {
    key: String,
    exists: bool,
}

#[derive(Serialize, Deserialize)]
struct DeleteOutput {
    key: String,
    deleted: bool,
}

#[derive(Serialize, Deserialize)]
struct StatsOutput {
    total_keys: usize,
    total_bytes: u64,
}

pub async fn run(args: &KvArgs) -> Result<()> {
    let cache_dir = args
        .config_cache_dir
        .as_deref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(default_cache_dir);

    let storage = FilesystemStorage::new(&cache_dir)?;

    match &args.command {
        KvCommand::Get {
            key,
            output,
            verbose,
            json,
        } => get(&storage, key, output.as_deref(), *verbose, *json).await,
        KvCommand::Put {
            key,
            value,
            file,
            verbose,
            json,
        } => {
            put(
                &storage,
                key,
                value.as_deref(),
                file.as_deref(),
                *verbose,
                *json,
            )
            .await
        }
        KvCommand::Exists { key, json } => exists(&storage, key, *json).await,
        KvCommand::Delete { key, force, json } => delete(&storage, key, *force, *json).await,
        KvCommand::List {
            prefix,
            verbose,
            json,
        } => list(&storage, prefix.as_deref(), *verbose, *json).await,
        KvCommand::Stats { json } => stats(&storage, *json).await,
    }
}

/// Convert key to bytes with KV namespace prefix
fn key_to_bytes(key: &str) -> Vec<u8> {
    format!("kv:{}", key).into_bytes()
}

/// Convert bytes back to key (removes namespace prefix)
fn bytes_to_key(bytes: &[u8]) -> Result<String> {
    let s = String::from_utf8(bytes.to_vec())?;
    Ok(s.strip_prefix("kv:").unwrap_or(&s).to_string())
}

/// Get a value by key
async fn get(
    storage: &FilesystemStorage,
    key: &str,
    output_path: Option<&str>,
    verbose: bool,
    json: bool,
) -> Result<()> {
    use std::fs;
    use std::io::{self, Write};

    if verbose && !json {
        println!("{} Retrieving key: {}", fabrik_prefix(), key);
    }

    let data = storage
        .get(&key_to_bytes(key))
        .with_context(|| format!("Failed to retrieve key: {}", key))?;

    if let Some(data) = data {
        match output_path {
            Some(path) => {
                fs::write(path, &data).with_context(|| format!("Failed to write to: {}", path))?;

                if json {
                    let output = GetOutput {
                        key: key.to_string(),
                        value_bytes: data.len(),
                        success: true,
                    };
                    println!("{}", serde_json::to_string(&output)?);
                } else {
                    println!(
                        "{} Value retrieved: {} ({} bytes)",
                        fabrik_prefix(),
                        key,
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
        anyhow::bail!("Key not found: {}", key);
    }
}

/// Put a key-value pair
async fn put(
    storage: &FilesystemStorage,
    key: &str,
    value: Option<&str>,
    file: Option<&str>,
    verbose: bool,
    json: bool,
) -> Result<()> {
    use std::fs;

    let data = if let Some(value_str) = value {
        value_str.as_bytes().to_vec()
    } else if let Some(file_path) = file {
        fs::read(file_path).with_context(|| format!("Failed to read file: {}", file_path))?
    } else {
        anyhow::bail!("Either value or --file must be provided");
    };

    let data_len = data.len();

    if verbose && !json {
        println!("{} Storing key: {}", fabrik_prefix(), key);
    }

    storage
        .put(&key_to_bytes(key), &data)
        .with_context(|| format!("Failed to store key: {}", key))?;

    if json {
        let output = PutOutput {
            key: key.to_string(),
            value_bytes: data_len,
            success: true,
        };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!("{} Key stored: {}", fabrik_prefix(), key);
        println!("{} Size: {} bytes", fabrik_prefix(), data_len);
    }

    Ok(())
}

/// Check if a key exists
async fn exists(storage: &FilesystemStorage, key: &str, json: bool) -> Result<()> {
    let exists = storage
        .exists(&key_to_bytes(key))
        .with_context(|| format!("Failed to check existence: {}", key))?;

    if json {
        let output = ExistsOutput {
            key: key.to_string(),
            exists,
        };
        println!("{}", serde_json::to_string(&output)?);
        std::process::exit(if exists { 0 } else { 1 });
    } else if exists {
        println!("{} Key exists: {}", fabrik_prefix(), key);
        std::process::exit(0);
    } else {
        println!("{} Key not found: {}", fabrik_prefix(), key);
        std::process::exit(1);
    }
}

/// Delete a key-value pair
async fn delete(storage: &FilesystemStorage, key: &str, force: bool, json: bool) -> Result<()> {
    use std::io::{self, Write};

    if !force && !json {
        print!("Delete key {}? [y/N]: ", key);
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().eq_ignore_ascii_case("y") {
            println!("{} Deletion cancelled.", fabrik_prefix());
            return Ok(());
        }
    }

    storage
        .delete(&key_to_bytes(key))
        .with_context(|| format!("Failed to delete key: {}", key))?;

    if json {
        let output = DeleteOutput {
            key: key.to_string(),
            deleted: true,
        };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!("{} Key deleted: {}", fabrik_prefix(), key);
    }

    Ok(())
}

/// List all keys (optionally filtered by prefix)
async fn list(
    storage: &FilesystemStorage,
    prefix: Option<&str>,
    verbose: bool,
    json: bool,
) -> Result<()> {
    let all_ids = storage.list_ids()?;

    // Filter for KV entries and apply prefix filter
    let kv_keys: Vec<String> = all_ids
        .iter()
        .filter_map(|id| {
            bytes_to_key(id).ok().and_then(|key| {
                if let Some(p) = prefix {
                    if key.starts_with(p) {
                        Some(key)
                    } else {
                        None
                    }
                } else {
                    Some(key)
                }
            })
        })
        .collect();

    if json {
        let keys: Vec<_> = kv_keys
            .iter()
            .map(|key| {
                if verbose {
                    let size = storage.size(&key_to_bytes(key)).ok().flatten().unwrap_or(0);
                    serde_json::json!({
                        "key": key,
                        "value_bytes": size,
                    })
                } else {
                    serde_json::json!({"key": key})
                }
            })
            .collect();
        println!("{}", serde_json::to_string(&keys)?);
    } else {
        if kv_keys.is_empty() {
            if let Some(p) = prefix {
                println!("No keys found with prefix: {}", p);
            } else {
                println!("No keys found.");
            }
            return Ok(());
        }

        if let Some(p) = prefix {
            println!("Keys with prefix '{}' ({} total):", p, kv_keys.len());
        } else {
            println!("All keys ({} total):", kv_keys.len());
        }

        for key in kv_keys {
            if verbose {
                if let Ok(Some(size)) = storage.size(&key_to_bytes(&key)) {
                    println!("  {} ({:.2} KB)", key, size as f64 / 1_000.0);
                } else {
                    println!("  {}", key);
                }
            } else {
                println!("  {}", key);
            }
        }
    }

    Ok(())
}

/// Show KV storage statistics
async fn stats(storage: &FilesystemStorage, json: bool) -> Result<()> {
    let all_ids = storage.list_ids()?;

    // Filter for KV entries
    let kv_keys: Vec<_> = all_ids
        .iter()
        .filter_map(|id| bytes_to_key(id).ok())
        .collect();

    let total_keys = kv_keys.len();
    let mut total_bytes = 0u64;

    for key in kv_keys {
        if let Ok(Some(size)) = storage.size(&key_to_bytes(&key)) {
            total_bytes += size;
        }
    }

    if json {
        let output = StatsOutput {
            total_keys,
            total_bytes,
        };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!("KV Storage Statistics");
        println!();
        println!("Total keys: {}", total_keys);
        println!("Total size: {:.2} MB", total_bytes as f64 / 1_000_000.0);

        if total_keys > 0 {
            println!(
                "Average size per key: {:.2} KB",
                (total_bytes as f64 / total_keys as f64) / 1_000.0
            );
        }
    }

    Ok(())
}
