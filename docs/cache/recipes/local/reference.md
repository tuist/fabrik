# Configuration Reference

Complete reference for all `#FABRIK` directives used in script caching.

## Overview

Directives are declared as comments in your script using the `#FABRIK` prefix. The syntax is based on [KDL (KDL Document Language)](https://kdl.dev/), but you don't need to understand KDL to use them - just follow the examples below.

## Input Tracking

### `#FABRIK input`

Track input files that affect the cache key. When any tracked file changes, the cache invalidates.

**Basic syntax:**
```bash
#FABRIK input "path/to/file"
```

**With globs:**
```bash
#FABRIK input "src/**/*.ts"
#FABRIK input "*.json"
#FABRIK input "config/*.yml"
```

**With hash method:**
```bash
#FABRIK input "large-binary.dat" hash=size      # Only track file size
#FABRIK input "video.mp4" hash=mtime            # Only track modification time
#FABRIK input "source.ts" hash=content          # Full content hash (default)
```

**Hash methods:**
- `content` - Hash entire file contents (default, most reliable)
- `mtime` - Hash modification time only (faster, less reliable)
- `size` - Hash file size only (fastest, least reliable)

**Examples:**
```bash
# Track TypeScript source files
#FABRIK input "src/**/*.ts"

# Track package manifest
#FABRIK input "package.json"

# Track multiple config files
#FABRIK input "tsconfig.json"
#FABRIK input ".eslintrc.js"

# Track large binary with size-based hashing
#FABRIK input "assets/video.mp4" hash=size
```

## Output Declaration

### `#FABRIK output`

Declare output paths (files or directories) that should be cached and restored.

**Syntax:**
```bash
#FABRIK output "path/to/output"
```

**Examples:**
```bash
# Cache build directory
#FABRIK output "dist/"

# Cache specific file
#FABRIK output "bundle.js"

# Cache multiple outputs
#FABRIK output "dist/"
#FABRIK output "build/"
#FABRIK output "coverage/"
```

**Notes:**
- Outputs are archived and compressed (tar + zstd)
- On cache hit, outputs are extracted before the script "executes"
- Only cached if script exits with code 0 (success)

## Environment Variables

### `#FABRIK env`

Track environment variable values in the cache key. When the variable changes, cache invalidates.

**Syntax:**
```bash
#FABRIK env "VARIABLE_NAME"
```

**Examples:**
```bash
# Track NODE_ENV
#FABRIK env "NODE_ENV"

# Track multiple variables
#FABRIK env "NODE_ENV"
#FABRIK env "API_KEY"
#FABRIK env "BUILD_TARGET"
```

**Use cases:**
- Configuration that affects output (`NODE_ENV`, `BUILD_MODE`)
- API keys or tokens that affect behavior
- Target platforms or architectures

**Notes:**
- Only the variable **value** is tracked, not its presence/absence
- If variable is unset, it's treated as empty string

## Script Dependencies

### `#FABRIK depends`

Declare dependencies on other scripts. Fabrik will execute dependencies before the current script.

**Basic syntax:**
```bash
#FABRIK depends "path/to/script.sh"
```

**With output reuse:**
```bash
#FABRIK depends "build-deps.sh" use-outputs=true
```

**Examples:**
```bash
# Simple dependency
#FABRIK depends "./prepare.sh"

# Dependency with output reuse (adds dependency outputs as inputs)
#FABRIK depends "./build-libs.sh" use-outputs=true

# Multiple dependencies
#FABRIK depends "./step1.sh"
#FABRIK depends "./step2.sh"
```

**With `use-outputs=true`:**
- Dependency's outputs are automatically added as inputs to current script
- Ensures cache invalidation when dependency outputs change
- Useful for build pipelines (build → test → deploy)

**Notes:**
- Dependencies are resolved recursively
- Cyclic dependencies are detected and rejected
- Dependencies execute in order

## Cache Control

### `#FABRIK cache disable`

Disable caching for this script.

**Syntax:**
```bash
#FABRIK cache disable
```

**Example:**
```bash
#!/usr/bin/env bash
#FABRIK cache disable

# This script will always execute, never cache
echo "Current time: $(date)"
```

**Use cases:**
- Scripts with side effects (API calls, database updates)
- Scripts that depend on current time
- Scripts with non-deterministic outputs

### `#FABRIK cache ttl`

Set cache expiration time. Cached results older than TTL are invalidated.

**Syntax:**
```bash
#FABRIK cache ttl="duration"
```

**Duration format:**
- `h` - hours (e.g., `2h`)
- `d` - days (e.g., `7d`)
- `m` - minutes (e.g., `30m`)

**Examples:**
```bash
# Expire after 2 hours
#FABRIK cache ttl="2h"

# Expire after 7 days
#FABRIK cache ttl="7d"

# Expire after 30 minutes
#FABRIK cache ttl="30m"
```

**Use cases:**
- Time-sensitive scripts (nightly builds, reports)
- Scripts that fetch external data
- Scripts with large outputs (expire old builds)

### `#FABRIK cache key`

Override cache key with a custom value.

**Syntax:**
```bash
#FABRIK cache key="custom-key"
```

**Example:**
```bash
#FABRIK cache key="v2-build-prod"
```

**Use cases:**
- Manual cache invalidation (change key to bust cache)
- Versioned caching
- Environment-specific keys

**Notes:**
- Custom key is **appended** to computed hash (doesn't replace it)
- Useful for forcing cache invalidation without changing script

## Runtime Configuration

### `#FABRIK runtime`

Override the runtime used to execute the script (defaults to shebang).

**Syntax:**
```bash
#FABRIK runtime command
```

**Examples:**
```bash
# Use specific Node version
#FABRIK runtime node

# Use specific Python interpreter
#FABRIK runtime python3.11

# Use bash
#FABRIK runtime bash
```

**Notes:**
- Runtime is resolved from PATH
- Overrides the shebang line
- Useful when shebang isn't flexible enough

### `#FABRIK runtime-arg`

Pass arguments to the runtime.

**Syntax:**
```bash
#FABRIK runtime-arg "argument"
```

**Examples:**
```bash
# Increase Node.js memory
#FABRIK runtime-arg "--max-old-space-size=4096"

# Python unbuffered output
#FABRIK runtime-arg "-u"

# Multiple arguments
#FABRIK runtime-arg "--max-old-space-size=4096"
#FABRIK runtime-arg "--expose-gc"
```

### `#FABRIK runtime-version`

Include runtime version in the cache key.

**Syntax:**
```bash
#FABRIK runtime-version
```

**Example:**
```bash
#!/usr/bin/env node
#FABRIK runtime-version
#FABRIK output "dist/"

// Build with specific Node version
// Cache invalidates when Node version changes
```

**Use cases:**
- Scripts whose output depends on runtime version
- Prevent cache hits across different runtime versions

**Notes:**
- Runs `runtime --version` and includes output in cache key
- Adds slight overhead on first run

## Execution Control

### `#FABRIK exec cwd`

Set working directory for script execution.

**Syntax:**
```bash
#FABRIK exec cwd="path/to/directory"
```

**Examples:**
```bash
# Run in subdirectory
#FABRIK exec cwd="frontend"

# Run in parent directory
#FABRIK exec cwd=".."
```

**Notes:**
- Path is relative to script location
- Defaults to script's directory if not specified

### `#FABRIK exec timeout`

Set maximum execution time. Script is killed if it exceeds timeout.

**Syntax:**
```bash
#FABRIK exec timeout="duration"
```

**Duration format:**
- `s` - seconds (e.g., `30s`)
- `m` - minutes (e.g., `5m`)
- `h` - hours (e.g., `2h`)

**Examples:**
```bash
# Timeout after 5 minutes
#FABRIK exec timeout="5m"

# Timeout after 30 seconds
#FABRIK exec timeout="30s"
```

**Use cases:**
- Prevent hanging scripts
- CI/CD time limits
- Enforce performance requirements

### `#FABRIK exec shell`

Execute via shell (enables shell features like pipes, redirects, etc.).

**Syntax:**
```bash
#FABRIK exec shell
```

**Example:**
```bash
#FABRIK exec shell

# Now you can use shell features
echo "Building..." | tee build.log
```

**Notes:**
- Without this, script is executed directly by the runtime
- With this, script is executed via shell wrapper
- Slightly slower, but enables shell features

## Complete Example

Here's a comprehensive example using multiple directives:

```bash
#!/usr/bin/env bash
#FABRIK input "src/**/*.ts"
#FABRIK input "package.json"
#FABRIK input "tsconfig.json"
#FABRIK output "dist/"
#FABRIK output "build-stats.json"
#FABRIK env "NODE_ENV"
#FABRIK env "BUILD_TARGET"
#FABRIK depends "./install-deps.sh" use-outputs=true
#FABRIK cache ttl="24h"
#FABRIK runtime-version
#FABRIK exec timeout="10m"

# Build TypeScript project
echo "Building for $BUILD_TARGET..."
npm run build

# Generate stats
npm run stats > build-stats.json
```

## See Also

- [Introduction](/cache/scripts/) - Overview and quick start
- [Examples](/cache/scripts/examples) - Real-world examples
