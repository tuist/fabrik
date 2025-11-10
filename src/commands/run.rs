/// `fabrik run` command implementation
///
/// Executes scripts with caching based on KDL annotations.
use anyhow::{Context, Result};
use std::path::Path;
use std::time::Instant;

use crate::cli::RunArgs;
use crate::script::{
    annotations::parse_annotations,
    cache::{create_metadata, ScriptCache},
    cache_key::compute_cache_key,
    dependencies::DependencyResolver,
    executor::ScriptExecutor,
    outputs::{archive_outputs, extract_outputs},
};
use crate::storage::default_cache_dir;

pub async fn run(args: &RunArgs) -> Result<()> {
    let script_path = Path::new(&args.script);

    if !script_path.exists() {
        anyhow::bail!("Script not found: {}", args.script);
    }

    // Parse annotations
    if args.verbose {
        eprintln!("[fabrik] Parsing annotations from {}", args.script);
    }

    let mut annotations = parse_annotations(script_path)
        .with_context(|| format!("Failed to parse script annotations: {}", args.script))?;

    // Check if caching is disabled
    if annotations.cache_disabled || args.no_cache {
        eprintln!("[fabrik] Caching disabled - executing script directly");
        return execute_script_no_cache(script_path, &annotations, &args.script_args, args.verbose);
    }

    // Resolve dependencies
    if args.verbose && !annotations.depends_on.is_empty() {
        eprintln!(
            "[fabrik] Resolving {} dependencies...",
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
        eprintln!("[fabrik] Cache key: {}", cache_key);
    }

    if args.dry_run {
        eprintln!(
            "[fabrik] Dry run - would check cache with key: {}",
            cache_key
        );
        eprintln!("[fabrik] Inputs: {} globs", annotations.inputs.len());
        eprintln!("[fabrik] Outputs: {} paths", annotations.outputs.len());
        return Ok(());
    }

    // Initialize cache
    let cache_dir = args
        .config_cache_dir
        .as_deref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(default_cache_dir);
    let cache = ScriptCache::new(cache_dir).context("Failed to initialize script cache")?;

    if args.clean {
        if args.verbose {
            eprintln!("[fabrik] Cleaning cache for this script");
        }
        cache.remove(&cache_key)?;
    }

    // Check cache
    if args.verbose {
        eprintln!("[fabrik] Checking cache...");
    }

    let start = Instant::now();

    if let Some(entry) = cache.get(&cache_key)? {
        // Cache hit!
        let duration = start.elapsed();

        if args.verbose {
            eprintln!("[fabrik] Cache HIT ✓");
            eprintln!("[fabrik] Restoring outputs from cache");
            for output in &entry.metadata.outputs {
                eprintln!(
                    "[fabrik]   {} ({} bytes, {} files)",
                    output.path, output.size_bytes, output.file_count
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
            "Cache key: {} | HIT ✓ | {:.2}s (exit: {})",
            cache_key,
            duration.as_secs_f64(),
            entry.metadata.execution.exit_code
        );

        std::process::exit(entry.metadata.execution.exit_code);
    }

    // Cache miss
    if args.verbose {
        eprintln!("[fabrik] Cache MISS ✗");
    }

    if args.cache_only {
        anyhow::bail!("Cache miss and --cache-only flag set");
    }

    // Execute script
    if args.verbose {
        eprintln!(
            "[fabrik] Executing: {} {}",
            annotations.runtime, args.script
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
            "[fabrik] Script completed with exit code: {}",
            result.exit_code
        );
    }

    // Archive outputs (only if successful)
    if result.exit_code == 0 {
        if args.verbose {
            eprintln!("[fabrik] Archiving outputs...");
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
            eprintln!("[fabrik] Archived {} outputs", archived_outputs.len());
            for output in &archived_outputs {
                eprintln!(
                    "[fabrik]   {} ({} bytes, {} files)",
                    output.path, output.size_bytes, output.file_count
                );
            }
        }

        // Create metadata
        let metadata = create_metadata(crate::script::cache::CreateMetadataParams {
            cache_key: cache_key.clone(),
            script_path,
            exit_code: result.exit_code,
            duration: result.duration,
            runtime: annotations.runtime.clone(),
            runtime_version: if annotations.runtime_version {
                crate::script::inputs::get_runtime_version(&annotations.runtime).ok()
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
            eprintln!("[fabrik] Cached as: {}", cache_key);
        }
    } else if args.verbose {
        eprintln!(
            "[fabrik] Not caching (non-zero exit code: {})",
            result.exit_code
        );
    }

    let total_duration = start.elapsed();

    // Compact single-line output
    eprintln!(
        "Cache key: {} | MISS ✗ | {:.2}s (exit: {})",
        cache_key,
        total_duration.as_secs_f64(),
        result.exit_code
    );

    std::process::exit(result.exit_code);
}

/// Execute script without caching
fn execute_script_no_cache(
    script_path: &Path,
    annotations: &crate::script::ScriptAnnotations,
    args: &[String],
    verbose: bool,
) -> Result<()> {
    let executor = ScriptExecutor::new(verbose);
    let result = executor
        .execute(script_path, annotations, args)
        .context("Script execution failed")?;

    if verbose {
        eprintln!(
            "[fabrik] Execution time: {:.2}s",
            result.duration.as_secs_f64()
        );
        eprintln!("[fabrik] Exit code: {}", result.exit_code);
    }

    std::process::exit(result.exit_code);
}
