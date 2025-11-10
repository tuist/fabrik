/// `fabrik cache` command implementation
///
/// Manages both script cache entries and object cache (content-addressed storage).
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::cli::{CacheArgs, CacheCommands};
use crate::cli_utils::fabrik_prefix;
use crate::script::{
    annotations::parse_annotations, cache::ScriptCache, cache_key::compute_cache_key,
};
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
    created_at: String,
    accessed_at: Option<String>,
    access_count: u32,
    content_type: Option<String>,
}

pub async fn cache(args: &CacheArgs) -> Result<()> {
    let cache_dir = args
        .config_cache_dir
        .as_deref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(default_cache_dir);

    match &args.command {
        // Script cache commands
        CacheCommands::Status { script, verbose } => {
            let script_cache =
                ScriptCache::new(cache_dir).context("Failed to initialize script cache")?;
            status(&script_cache, script, *verbose).await
        }
        CacheCommands::Clean { script, all } => {
            let script_cache =
                ScriptCache::new(cache_dir).context("Failed to initialize script cache")?;
            clean(&script_cache, script.as_deref(), *all).await
        }
        CacheCommands::List { verbose } => {
            let script_cache =
                ScriptCache::new(cache_dir).context("Failed to initialize script cache")?;
            list(&script_cache, *verbose).await
        }
        CacheCommands::Stats => {
            let script_cache =
                ScriptCache::new(cache_dir).context("Failed to initialize script cache")?;
            stats(&script_cache).await
        }

        // Object cache commands
        CacheCommands::Get {
            hash,
            output,
            verbose,
            json,
        } => {
            let storage = FilesystemStorage::new(&cache_dir)?;
            object_get(&storage, hash, output, *verbose, *json).await
        }
        CacheCommands::Put {
            input,
            hash,
            verbose,
            json,
        } => {
            let storage = FilesystemStorage::new(&cache_dir)?;
            object_put(&storage, input, hash.as_deref(), *verbose, *json).await
        }
        CacheCommands::Exists { hash, json } => {
            let storage = FilesystemStorage::new(&cache_dir)?;
            object_exists(&storage, hash, *json).await
        }
        CacheCommands::Delete { hash, force, json } => {
            let storage = FilesystemStorage::new(&cache_dir)?;
            object_delete(&storage, hash, *force, *json).await
        }
        CacheCommands::Info { hash, json } => {
            let storage = FilesystemStorage::new(&cache_dir)?;
            object_info(&storage, hash, *json).await
        }
    }
}

/// Show cache status for a script
async fn status(cache: &ScriptCache, script_path: &str, verbose: bool) -> Result<()> {
    let path = Path::new(script_path);

    if !path.exists() {
        anyhow::bail!("Script not found: {}", script_path);
    }

    // Parse annotations
    let annotations = parse_annotations(path)
        .with_context(|| format!("Failed to parse script annotations: {}", script_path))?;

    // Compute cache key
    let cache_key = compute_cache_key(path, &annotations).context("Failed to compute cache key")?;

    println!("Script: {}", script_path);
    println!("Cache key: {}", cache_key);

    // Check cache
    if let Some(entry) = cache.get(&cache_key)? {
        println!("Status: CACHED ✓");
        println!();
        println!("Cache entry:");
        println!(
            "  Created: {}",
            entry.metadata.created_at.format("%Y-%m-%d %H:%M:%S")
        );

        if let Some(expires_at) = entry.metadata.expires_at {
            let ttl = expires_at - entry.metadata.created_at;
            println!(
                "  Expires: {} ({}d TTL)",
                expires_at.format("%Y-%m-%d %H:%M:%S"),
                ttl.num_days()
            );
        } else {
            println!("  Expires: Never");
        }

        println!("  Exit code: {}", entry.metadata.execution.exit_code);
        println!(
            "  Duration: {:.2}s",
            entry.metadata.execution.duration_ms as f64 / 1000.0
        );

        if !entry.metadata.outputs.is_empty() {
            println!("  Outputs:");
            for output in &entry.metadata.outputs {
                println!(
                    "    {} ({:.2} MB, {} files)",
                    output.path,
                    output.size_bytes as f64 / 1_000_000.0,
                    output.file_count
                );
            }
        }

        if verbose {
            println!();
            println!("Cache layers:");
            println!("  ✓ Local (RocksDB)");

            if let Some(upstream) = &entry.metadata.cache_info.upstream_used {
                println!("  ✓ Upstream ({})", upstream);
            }
        }
    } else {
        println!("Status: NOT CACHED ✗");
        println!();
        println!("Run `fabrik run {}` to cache this script.", script_path);
    }

    Ok(())
}

/// Clean cache for a script or all scripts
async fn clean(cache: &ScriptCache, script_path: Option<&str>, all: bool) -> Result<()> {
    if all {
        println!("{} Cleaning all script caches...", fabrik_prefix());
        cache.clean_all().context("Failed to clean all caches")?;
        println!("{} All script caches cleaned.", fabrik_prefix());
        return Ok(());
    }

    let Some(script_path) = script_path else {
        anyhow::bail!("Specify --all to clean all caches, or provide a script path");
    };

    let path = Path::new(script_path);

    if !path.exists() {
        anyhow::bail!("Script not found: {}", script_path);
    }

    // Parse annotations
    let annotations = parse_annotations(path)
        .with_context(|| format!("Failed to parse script annotations: {}", script_path))?;

    // Compute cache key
    let cache_key = compute_cache_key(path, &annotations).context("Failed to compute cache key")?;

    println!("{} Cleaning cache for: {}", fabrik_prefix(), script_path);
    println!("{} Cache key: {}", fabrik_prefix(), cache_key);

    cache.remove(&cache_key)?;

    println!("{} Cache cleaned.", fabrik_prefix());

    Ok(())
}

/// List all cached scripts
async fn list(cache: &ScriptCache, verbose: bool) -> Result<()> {
    let entries = cache.list().context("Failed to list cache entries")?;

    if entries.is_empty() {
        println!("No cached scripts.");
        return Ok(());
    }

    println!("Cached scripts ({} entries):", entries.len());
    println!();

    for cache_key in entries {
        if let Some(entry) = cache.get(&cache_key)? {
            println!("  {}", cache_key);
            println!("    Script: {}", entry.metadata.script_path);
            println!(
                "    Created: {}",
                entry.metadata.created_at.format("%Y-%m-%d %H:%M:%S")
            );

            if verbose {
                println!("    Exit code: {}", entry.metadata.execution.exit_code);
                println!(
                    "    Duration: {:.2}s",
                    entry.metadata.execution.duration_ms as f64 / 1000.0
                );
                println!("    Outputs: {}", entry.metadata.outputs.len());

                let total_size: u64 = entry.metadata.outputs.iter().map(|o| o.size_bytes).sum();
                println!("    Total size: {:.2} MB", total_size as f64 / 1_000_000.0);
            }

            println!();
        }
    }

    Ok(())
}

/// Show cache statistics
async fn stats(cache: &ScriptCache) -> Result<()> {
    let stats = cache.stats().context("Failed to get cache statistics")?;

    println!("Script Cache Statistics");
    println!();
    println!("Total entries: {}", stats.total_entries);
    println!(
        "Total size: {:.2} MB",
        stats.total_size_bytes as f64 / 1_000_000.0
    );
    println!("Total files: {}", stats.total_files);

    if stats.total_entries > 0 {
        println!(
            "Average size per entry: {:.2} MB",
            (stats.total_size_bytes as f64 / stats.total_entries as f64) / 1_000_000.0
        );
    }

    Ok(())
}

// ============================================================================
// Object Cache Commands
// ============================================================================

/// Get an artifact from the cache
async fn object_get(
    storage: &FilesystemStorage,
    hash: &str,
    output_path: &str,
    verbose: bool,
    json: bool,
) -> Result<()> {
    use std::fs;

    if verbose && !json {
        println!("{} Retrieving artifact: {}", fabrik_prefix(), hash);
    }

    let data = storage
        .get(hash.as_bytes())
        .with_context(|| format!("Failed to retrieve artifact: {}", hash))?;

    if let Some(data) = data {
        fs::write(output_path, &data)
            .with_context(|| format!("Failed to write to: {}", output_path))?;

        if json {
            let output = GetOutput {
                hash: hash.to_string(),
                output_path: output_path.to_string(),
                size_bytes: data.len(),
                success: true,
            };
            println!("{}", serde_json::to_string(&output)?);
        } else {
            println!(
                "{} Artifact retrieved: {} ({} bytes)",
                fabrik_prefix(),
                hash,
                data.len()
            );
            println!("{} Written to: {}", fabrik_prefix(), output_path);
        }
        Ok(())
    } else {
        anyhow::bail!("Artifact not found: {}", hash);
    }
}

/// Put an artifact into the cache
async fn object_put(
    storage: &FilesystemStorage,
    input_path: &str,
    hash: Option<&str>,
    verbose: bool,
    json: bool,
) -> Result<()> {
    use sha2::{Digest, Sha256};
    use std::fs;

    let data =
        fs::read(input_path).with_context(|| format!("Failed to read file: {}", input_path))?;
    let data_len = data.len();

    let computed_hash = if let Some(provided_hash) = hash {
        // Verify provided hash
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let computed = format!("{:x}", hasher.finalize());

        if computed != provided_hash {
            anyhow::bail!(
                "Hash mismatch: provided {} but computed {}",
                provided_hash,
                computed
            );
        }

        if verbose && !json {
            println!("{} Hash verified: {}", fabrik_prefix(), provided_hash);
        }

        provided_hash.to_string()
    } else {
        // Compute hash
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let computed = format!("{:x}", hasher.finalize());

        if verbose && !json {
            println!("{} Computed hash: {}", fabrik_prefix(), computed);
        }

        computed
    };

    if verbose && !json {
        println!("{} Storing artifact: {}", fabrik_prefix(), computed_hash);
    }

    storage
        .put(computed_hash.as_bytes(), &data)
        .with_context(|| format!("Failed to store artifact: {}", computed_hash))?;

    if json {
        let output = PutOutput {
            hash: computed_hash,
            size_bytes: data_len,
            success: true,
        };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!("{} Artifact stored: {}", fabrik_prefix(), computed_hash);
    }

    Ok(())
}

/// Check if an artifact exists in the cache
async fn object_exists(storage: &FilesystemStorage, hash: &str, json: bool) -> Result<()> {
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
        println!("{} Artifact exists: {}", fabrik_prefix(), hash);
        std::process::exit(0);
    } else {
        println!("{} Artifact not found: {}", fabrik_prefix(), hash);
        std::process::exit(1);
    }
}

/// Delete an artifact from the cache
async fn object_delete(
    storage: &FilesystemStorage,
    hash: &str,
    force: bool,
    json: bool,
) -> Result<()> {
    use std::io::{self, Write};

    if !force && !json {
        print!("Delete artifact {}? [y/N]: ", hash);
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
        .with_context(|| format!("Failed to delete artifact: {}", hash))?;

    if json {
        let output = DeleteOutput {
            hash: hash.to_string(),
            deleted: true,
        };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!("{} Artifact deleted: {}", fabrik_prefix(), hash);
    }

    Ok(())
}

/// Show information about a cached artifact
async fn object_info(storage: &FilesystemStorage, hash: &str, json: bool) -> Result<()> {
    // Check if artifact exists
    let exists = storage
        .exists(hash.as_bytes())
        .with_context(|| format!("Failed to check existence: {}", hash))?;

    if !exists {
        anyhow::bail!("Artifact not found: {}", hash);
    }

    // Get size
    let size = storage
        .size(hash.as_bytes())
        .with_context(|| format!("Failed to get size: {}", hash))?
        .ok_or_else(|| anyhow::anyhow!("Artifact not found: {}", hash))?;

    if json {
        let output = InfoOutput {
            hash: hash.to_string(),
            size_bytes: size,
            created_at: "N/A".to_string(),
            accessed_at: None,
            access_count: 0,
            content_type: None,
        };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!("{} Artifact: {}", fabrik_prefix(), hash);
        println!(
            "{} Size: {} bytes ({:.2} MB)",
            fabrik_prefix(),
            size,
            size as f64 / 1_000_000.0
        );
    }

    Ok(())
}
