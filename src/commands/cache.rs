/// `fabrik cache` command implementation
///
/// Manages script cache entries (status, clean, list, stats).
use anyhow::{Context, Result};
use std::path::Path;

use crate::cli::{CacheArgs, CacheCommands};
use crate::cli_utils::fabrik_prefix;
use crate::script::{
    annotations::parse_annotations, cache::ScriptCache, cache_key::compute_cache_key,
};
use crate::storage::default_cache_dir;

pub async fn cache(args: &CacheArgs) -> Result<()> {
    let cache_dir = args
        .config_cache_dir
        .as_deref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(default_cache_dir);
    let cache = ScriptCache::new(cache_dir).context("Failed to initialize script cache")?;

    match &args.command {
        CacheCommands::Status { script, verbose } => status(&cache, script, *verbose).await,
        CacheCommands::Clean { script, all } => clean(&cache, script.as_deref(), *all).await,
        CacheCommands::List { verbose } => list(&cache, *verbose).await,
        CacheCommands::Stats => stats(&cache).await,
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
