# Script Caching

Cache any script execution (bash, node, python, etc.) with automatic invalidation based on inputs, outputs, and environment variables.

## What is Script Caching?

Script caching allows you to cache arbitrary script executions that fall outside traditional build systems. While Fabrik integrates with build tools like Gradle and Bazel, many build workflows include custom scripts that aren't covered by those tools.

**Examples:**
- Running TypeScript compiler (`tsc`) directly
- Custom code generation scripts
- Asset processing (image optimization, etc.)
- Test runners that aren't part of your build system
- Deployment scripts
- Docker image builds

With script caching, you declare inputs and outputs using special comments in your scripts, and Fabrik handles the rest.

## How It Works

1. **Add annotations to your script** - Declare inputs, outputs, and environment variables using `#FABRIK` comments
2. **Run with `fabrik run`** - Execute your script through Fabrik
3. **Automatic caching** - Fabrik computes a cache key based on script content, inputs, and env vars
4. **Instant restoration** - On cache hit, outputs are restored without re-executing the script

## Quick Example

Create a script with caching annotations:

```bash
#!/usr/bin/env bash
#FABRIK input "src/**/*.ts"
#FABRIK input "package.json"
#FABRIK output "dist/"
#FABRIK env "NODE_ENV"

# Build TypeScript project
npm run build
```

Run it with Fabrik:

```bash
# First run - cache miss, executes script
fabrik run build.sh
# Output: Cache key: script-abc123 | MISS ✗ | 2.45s (exit: 0)

# Second run - cache hit, restores outputs instantly
fabrik run build.sh
# Output: Cache key: script-abc123 | HIT ✓ | 0.01s (exit: 0)
```

Change an input file, and the cache automatically invalidates:

```bash
# Modify a TypeScript file
echo "export const foo = 42;" >> src/index.ts

# Cache miss - script executes again
fabrik run build.sh
# Output: Cache key: script-def456 | MISS ✗ | 2.50s (exit: 0)
```

## Key Features

- **Content-addressed caching** - Cache key based on script content + inputs + environment variables
- **Automatic invalidation** - Cache invalidates when inputs, env vars, or script content changes
- **Output restoration** - Cached outputs (files/directories) are automatically restored
- **Dependency resolution** - Scripts can depend on other scripts with automatic execution ordering
- **Cross-platform** - Works with any runtime (bash, node, python, ruby, etc.)
- **Flexible input tracking** - Support for globs, hash methods (content/mtime/size)

## When to Use Script Caching

**Good use cases:**
- ✅ Scripts that produce deterministic outputs
- ✅ Scripts that take time to run (>1 second)
- ✅ Scripts that are frequently re-run with same inputs
- ✅ Build steps that aren't covered by your build system

**Not suitable for:**
- ❌ Scripts with side effects (database updates, API calls)
- ❌ Scripts that depend on current time or random values
- ❌ Scripts that are already fast (<1 second)

## Cache Key Computation

The cache key is computed as:

```
cache_key = SHA256(
    script_content +
    runtime + runtime_version (optional) +
    hash(input_files) +
    env_var_values +
    custom_cache_key (optional)
)
```

This ensures cache invalidation whenever any of these factors change.

## CLI Commands

```bash
# Run script with caching
fabrik run build.sh

# Pass arguments to script
fabrik run build.sh -- --production --verbose

# Disable caching for this run
fabrik run --no-cache build.sh

# Dry run (show cache key without executing)
fabrik run --dry-run build.sh

# Force clean and re-execute
fabrik run --clean build.sh

# Verbose output (show cache operations)
fabrik run --verbose build.sh

# Check cache status
fabrik cache status build.sh

# Clean cache for specific script
fabrik cache clean build.sh

# List all cached scripts
fabrik cache list

# View cache statistics
fabrik cache stats
```

## Output Format

Fabrik provides compact, single-line output:

```bash
# Cache hit
Cache key: script-49597b2a298253b8 | HIT ✓ | 0.00s (exit: 0)

# Cache miss
Cache key: script-49597b2a298253b8 | MISS ✗ | 2.45s (exit: 0)
```

Use `--verbose` for detailed output including input/output tracking.

## Next Steps

- [Configuration Reference](/script-caching/reference) - Complete list of all directives
- [Examples](/script-caching/examples) - Real-world examples and recipes
