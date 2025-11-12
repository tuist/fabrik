/// `fabrik cache` command implementation (DEPRECATED)
///
/// This module is kept for backward compatibility during migration.
/// The `fabrik cache` command has been split into:
/// - `fabrik cas` - Content-Addressed Storage operations
/// - `fabrik kv` - Key-Value storage operations
/// - `fabrik run --status/--list/--stats` - Script cache management
///
/// This stub prints a deprecation warning.
use anyhow::Result;

#[allow(dead_code)]
pub async fn cache_deprecated() -> Result<()> {
    eprintln!("WARNING: The `fabrik cache` command is deprecated.");
    eprintln!();
    eprintln!("Please use the new commands:");
    eprintln!("  - `fabrik cas` - Content-Addressed Storage operations");
    eprintln!("  - `fabrik kv` - Key-Value storage operations");
    eprintln!("  - `fabrik run --status <script>` - Check script cache status");
    eprintln!("  - `fabrik run --list` - List cached scripts");
    eprintln!("  - `fabrik run --stats` - Show cache statistics");
    eprintln!();
    eprintln!("See `fabrik cas --help`, `fabrik kv --help`, or `fabrik run --help` for details.");

    std::process::exit(1);
}
