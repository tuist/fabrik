/// `fabrik run` command implementation
///
/// Executes scripts with caching based on KDL annotations,
/// or runs portable recipes (QuickJS) from local or remote sources.
use anyhow::{Context, Result};
use std::path::Path;
use std::time::Instant;

use crate::cli::RunArgs;
use crate::cli_utils::fabrik_prefix;
use crate::recipe::{
    annotations::parse_annotations,
    cache::{create_metadata, ScriptCache},
    cache_key::compute_cache_key,
    dependencies::DependencyResolver,
    executor::ScriptExecutor,
    outputs::{archive_outputs, extract_outputs},
};
use crate::recipe_portable::{RecipeExecutor, RemoteRecipe};
use crate::storage::default_cache_dir;

pub async fn run(args: &RunArgs) -> Result<()> {
    use crate::config_discovery::load_config_with_discovery;

    // Load config file with auto-discovery
    let file_config = load_config_with_discovery(args.config.as_deref())?;

    // Initialize cache directory (CLI arg > config file > default)
    let cache_dir = args
        .config_cache_dir
        .as_deref()
        .map(std::path::PathBuf::from)
        .or_else(|| {
            file_config
                .as_ref()
                .map(|c| std::path::PathBuf::from(&c.cache.dir))
        })
        .unwrap_or_else(default_cache_dir);

    // Handle script management operations
    if args.status {
        return run_status(args, &cache_dir).await;
    }
    if args.list {
        return run_list(args, &cache_dir).await;
    }
    if args.stats {
        return run_stats(&cache_dir).await;
    }

    // Normal script execution
    if args.positional_args.is_empty() {
        anyhow::bail!("Script path required for execution (or use --status, --list, or --stats)");
    }

    let (cli_runtime, script) = args.parse_runtime_and_script();

    // Check if this is a remote recipe (starts with @)
    if script.starts_with('@') {
        return run_remote_recipe(&script, args).await;
    }

    let script_path = Path::new(&script);

    if !script_path.exists() {
        anyhow::bail!("Script not found: {}", script);
    }

    // Check if this is a local portable recipe (.js file)
    // Portable recipes are executed with QuickJS runtime
    if script_path
        .extension()
        .map(|ext| ext == "js")
        .unwrap_or(false)
    {
        return run_local_portable_recipe(script_path, args).await;
    }

    // Parse annotations
    if args.verbose {
        eprintln!("{} Parsing annotations from {}", fabrik_prefix(), script);
    }

    let mut annotations = parse_annotations(script_path)
        .with_context(|| format!("Failed to parse script annotations: {}", script))?;

    // Apply runtime priority: CLI arg > directive > shebang
    if let Some(runtime) = cli_runtime {
        annotations.runtime = runtime;
    }

    // Check if caching is disabled
    if annotations.cache_disabled || args.no_cache {
        eprintln!(
            "{} Caching disabled - executing script directly",
            fabrik_prefix()
        );
        return execute_script_no_cache(script_path, &annotations, &args.script_args, args.verbose);
    }

    // Resolve dependencies
    if args.verbose && !annotations.depends_on.is_empty() {
        eprintln!(
            "{} Resolving {} dependencies...",
            fabrik_prefix(),
            annotations.depends_on.len()
        );
    }

    let mut resolver = DependencyResolver::new();
    let dependencies = resolver
        .resolve(script_path)
        .context("Failed to resolve dependencies")?;

    // Augment annotations with dependency outputs
    DependencyResolver::augment_with_dependency_outputs(
        script_path,
        &mut annotations,
        &dependencies,
    );

    // Compute cache key
    let cache_key =
        compute_cache_key(script_path, &annotations).context("Failed to compute cache key")?;

    if args.verbose {
        eprintln!("{} Cache key: {}", fabrik_prefix(), cache_key);
    }

    if args.dry_run {
        eprintln!(
            "{} Dry run - would check cache with key: {}",
            fabrik_prefix(),
            cache_key
        );
        eprintln!(
            "{} Inputs: {} globs",
            fabrik_prefix(),
            annotations.inputs.len()
        );
        eprintln!(
            "{} Outputs: {} paths",
            fabrik_prefix(),
            annotations.outputs.len()
        );
        return Ok(());
    }

    // Initialize cache
    let cache_dir = args
        .config_cache_dir
        .as_deref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(default_cache_dir);
    let cache =
        ScriptCache::new(cache_dir.to_path_buf()).context("Failed to initialize script cache")?;

    if args.clean {
        if args.verbose {
            eprintln!("{} Cleaning cache for this script", fabrik_prefix());
        }
        cache.remove(&cache_key)?;
    }

    // Check cache
    if args.verbose {
        eprintln!("{} Checking cache...", fabrik_prefix());
    }

    let start = Instant::now();

    if let Some(entry) = cache.get(&cache_key)? {
        // Cache hit!
        let duration = start.elapsed();

        if args.verbose {
            eprintln!("{} Cache HIT ✓", fabrik_prefix());
            eprintln!("{} Restoring outputs from cache", fabrik_prefix());
            for output in &entry.metadata.outputs {
                eprintln!(
                    "{}   {} ({} bytes, {} files)",
                    fabrik_prefix(),
                    output.path,
                    output.size_bytes,
                    output.file_count
                );
            }
        }

        // Extract outputs
        let base_dir = script_path
            .parent()
            .and_then(|p| {
                if p == std::path::Path::new("") {
                    None
                } else {
                    Some(p)
                }
            })
            .unwrap_or_else(|| std::path::Path::new("."));

        extract_outputs(&entry.archive_path, base_dir)
            .context("Failed to extract cached outputs")?;

        // Compact single-line output
        eprintln!(
            "{} Cache key: {} | HIT ✓ | {:.2}s (exit: {})",
            fabrik_prefix(),
            cache_key,
            duration.as_secs_f64(),
            entry.metadata.execution.exit_code
        );

        std::process::exit(entry.metadata.execution.exit_code);
    }

    // Cache miss
    if args.verbose {
        eprintln!("{} Cache MISS ✗", fabrik_prefix());
    }

    if args.cache_only {
        anyhow::bail!("Cache miss and --cache-only flag set");
    }

    // Execute script
    if args.verbose {
        eprintln!(
            "{} Executing: {} {}",
            fabrik_prefix(),
            annotations.runtime,
            script
        );
    }

    let executor = ScriptExecutor::new(args.verbose);
    let result = executor
        .execute(script_path, &annotations, &args.script_args)
        .context("Script execution failed")?;

    // Print script output
    if !result.stdout.is_empty() {
        std::io::Write::write_all(&mut std::io::stdout(), &result.stdout)
            .context("Failed to write stdout")?;
    }
    if !result.stderr.is_empty() {
        std::io::Write::write_all(&mut std::io::stderr(), &result.stderr)
            .context("Failed to write stderr")?;
    }

    if args.verbose {
        eprintln!(
            "{} Script completed with exit code: {}",
            fabrik_prefix(),
            result.exit_code
        );
    }

    // Archive outputs (only if successful)
    if result.exit_code == 0 {
        if args.verbose {
            eprintln!("{} Archiving outputs...", fabrik_prefix());
        }

        let base_dir = script_path
            .parent()
            .and_then(|p| {
                if p == std::path::Path::new("") {
                    None
                } else {
                    Some(p)
                }
            })
            .unwrap_or_else(|| std::path::Path::new("."));

        let temp_archive =
            tempfile::NamedTempFile::new().context("Failed to create temporary archive")?;

        let archived_outputs = archive_outputs(&annotations.outputs, base_dir, temp_archive.path())
            .context("Failed to archive outputs")?;

        if args.verbose {
            eprintln!(
                "{} Archived {} outputs",
                fabrik_prefix(),
                archived_outputs.len()
            );
            for output in &archived_outputs {
                eprintln!(
                    "{}   {} ({} bytes, {} files)",
                    fabrik_prefix(),
                    output.path,
                    output.size_bytes,
                    output.file_count
                );
            }
        }

        // Create metadata
        let metadata = create_metadata(crate::recipe::CreateMetadataParams {
            cache_key: cache_key.clone(),
            script_path,
            exit_code: result.exit_code,
            duration: result.duration,
            runtime: annotations.runtime.clone(),
            runtime_version: if annotations.runtime_version {
                crate::recipe::inputs::get_runtime_version(&annotations.runtime).ok()
            } else {
                None
            },
            outputs: archived_outputs,
            env_vars: &annotations.env_vars,
            ttl: annotations.cache_ttl,
        });

        // Store in cache
        cache
            .put(&cache_key, metadata, temp_archive.path())
            .context("Failed to store in cache")?;

        if args.verbose {
            eprintln!("{} Cached as: {}", fabrik_prefix(), cache_key);
        }
    } else if args.verbose {
        eprintln!(
            "{} Not caching (non-zero exit code: {})",
            fabrik_prefix(),
            result.exit_code
        );
    }

    let total_duration = start.elapsed();

    // Compact single-line output
    eprintln!(
        "{} Cache key: {} | MISS ✗ | {:.2}s (exit: {})",
        fabrik_prefix(),
        cache_key,
        total_duration.as_secs_f64(),
        result.exit_code
    );

    std::process::exit(result.exit_code);
}

/// Execute script without caching
fn execute_script_no_cache(
    script_path: &Path,
    annotations: &crate::recipe::ScriptAnnotations,
    args: &[String],
    verbose: bool,
) -> Result<()> {
    let executor = ScriptExecutor::new(verbose);
    let result = executor
        .execute(script_path, annotations, args)
        .context("Script execution failed")?;

    if verbose {
        eprintln!(
            "{} Execution time: {:.2}s",
            fabrik_prefix(),
            result.duration.as_secs_f64()
        );
        eprintln!("{} Exit code: {}", fabrik_prefix(), result.exit_code);
    }

    std::process::exit(result.exit_code);
}

// ============================================================================
// Script Management Operations (--status, --list, --stats)
// ============================================================================

/// Show cache status for a script (`fabrik run --status script.sh`)
async fn run_status(args: &RunArgs, cache_dir: &std::path::Path) -> Result<()> {
    if args.positional_args.is_empty() {
        anyhow::bail!("Script path required for --status");
    }

    let script_path = &args.positional_args[0];
    let path = Path::new(script_path);

    if !path.exists() {
        anyhow::bail!("Script not found: {}", script_path);
    }

    let cache =
        ScriptCache::new(cache_dir.to_path_buf()).context("Failed to initialize script cache")?;

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

        if args.verbose {
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

/// List all cached scripts (`fabrik run --list`)
async fn run_list(args: &RunArgs, cache_dir: &std::path::Path) -> Result<()> {
    let cache =
        ScriptCache::new(cache_dir.to_path_buf()).context("Failed to initialize script cache")?;
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

            if args.verbose {
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

/// Show cache statistics (`fabrik run --stats`)
async fn run_stats(cache_dir: &std::path::Path) -> Result<()> {
    let cache =
        ScriptCache::new(cache_dir.to_path_buf()).context("Failed to initialize script cache")?;
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

/// Execute a remote recipe (from Git repository)
async fn run_remote_recipe(recipe_ref: &str, args: &RunArgs) -> Result<()> {
    if args.verbose {
        eprintln!("{} Parsing remote recipe: {}", fabrik_prefix(), recipe_ref);
    }

    // Parse remote recipe reference
    let remote = RemoteRecipe::parse(recipe_ref)
        .with_context(|| format!("Failed to parse remote recipe: {}", recipe_ref))?;

    if args.verbose {
        eprintln!(
            "{} Remote recipe: {}/{}/{} (ref: {})",
            fabrik_prefix(),
            remote.host,
            remote.org,
            remote.repo,
            remote.git_ref.as_deref().unwrap_or("main")
        );
    }

    // Fetch repository to local cache
    if args.verbose {
        eprintln!("{} Fetching from {}", fabrik_prefix(), remote.git_url());
    }

    let script_path = remote
        .fetch()
        .await
        .with_context(|| format!("Failed to fetch remote recipe: {}", recipe_ref))?;

    if args.verbose {
        eprintln!(
            "{} Recipe cached at: {}",
            fabrik_prefix(),
            script_path.display()
        );
    }

    // Execute recipe with RecipeExecutor
    let executor = RecipeExecutor::new(script_path);

    if args.verbose {
        eprintln!("{} Executing recipe at root level", fabrik_prefix());
    }

    executor
        .execute()
        .await
        .with_context(|| format!("Failed to execute remote recipe: {}", recipe_ref))?;

    Ok(())
}

/// Execute a local portable recipe (.js file with QuickJS runtime)
async fn run_local_portable_recipe(script_path: &Path, args: &RunArgs) -> Result<()> {
    if args.verbose {
        eprintln!(
            "{} Running local portable recipe: {}",
            fabrik_prefix(),
            script_path.display()
        );
    }

    // Get absolute path for the recipe
    let absolute_path = if script_path.is_absolute() {
        script_path.to_path_buf()
    } else {
        std::env::current_dir()?.join(script_path)
    };

    // Execute recipe with RecipeExecutor (QuickJS runtime)
    let executor = RecipeExecutor::new(absolute_path);

    if args.verbose {
        eprintln!("{} Executing recipe with QuickJS runtime", fabrik_prefix());
    }

    executor.execute().await.with_context(|| {
        format!(
            "Failed to execute portable recipe: {}",
            script_path.display()
        )
    })?;

    Ok(())
}
