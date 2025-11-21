# Portable Recipes - Cross-Platform Automation

**Status**: ✅ **WORKING** (QuickJS + LLRT runtime)

## Overview

Portable recipes provide cross-platform automation using JavaScript that runs on Fabrik's embedded QuickJS runtime with AWS LLRT modules. Unlike script recipes (which require platform-specific runtimes like bash, python, node), portable recipes are guaranteed to work on any operating system.

## Architecture

```
┌─────────────────────────────────────┐
│ User Recipe (JavaScript)            │
│  build.recipe.js                    │
│                                     │
│  async function build() {           │
│    await Fabrik.exec("npm", [       │
│      "run", "build"                 │
│    ]);                              │
│  }                                  │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│ QuickJS Runtime (embedded)          │
│  - JavaScript execution             │
│  - LLRT modules (console, fs, etc.) │
│  - Fabrik API injected              │
│  - Sandboxed environment            │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│ Fabrik API (Rust bindings)          │
│  - Fabrik.exec()                    │
│  - Fabrik.glob()                    │
│  - Fabrik.exists()                  │
└─────────────────────────────────────┘
```

## Implementation Status

### ✅ Completed (Phase 1: Local Recipes)

1. **QuickJS Runtime** (`runtime.rs`)
   - Embedded QuickJS engine via `rquickjs 0.10` (git)
   - LLRT module integration (console, buffer, fs, child_process, path)
   - Fabrik API injection

2. **Fabrik API** (`runtime.rs`)
   - `Fabrik.exec(command, args)` - Execute commands (streaming output)
   - `Fabrik.glob(pattern)` - File pattern matching
   - `Fabrik.exists(path)` - Check file existence

3. **Recipe Executor** (`executor.rs`)
   - Execute entire script at root level
   - Execute specific exported functions by name
   - Async/await support

4. **Remote Recipe Support** (`remote.rs`)
   - Parse `@org/repo/path/script.js@ref` syntax
   - Fetch from Git repositories (GitHub, GitLab, self-hosted)
   - XDG cache directory convention (`~/.cache/fabrik/recipes/`)
   - Automatic shallow cloning with `git clone --depth 1`

5. **CLI Integration** (`commands/run.rs`)
   - `fabrik run @tuist/recipes/build.js` - Remote recipes
   - `fabrik run ./local.recipe.js` - Local recipes
   - Target function support: `fabrik run recipe.js -- functionName`

6. **Tests**
   - 9 unit tests for remote recipe parsing
   - 5 integration tests for end-to-end functionality
   - All tests passing ✅

## Remote Recipe Syntax

### Format

```
@[host/]org/repo/path/script.js[@ref]
```

### Examples

```bash
# GitHub (default host)
fabrik run @tuist/recipes/build.js

# With version/tag
fabrik run @tuist/recipes/build.js@v1.0.0

# GitLab
fabrik run @gitlab.com/myorg/myrepo/scripts/deploy.js

# Self-hosted Git
fabrik run @git.company.com/team/project/build.js@main

# Nested paths
fabrik run @tuist/recipes/scripts/deploy/prod.js
```

### Cache Location

Remote recipes are cached following XDG convention:

```
~/.cache/fabrik/recipes/
├── github.com/
│   └── tuist/
│       └── recipes/
│           ├── main/           # Default branch
│           │   ├── build.js
│           │   └── deploy.js
│           └── v1.0.0/         # Tagged version
│               └── build.js
├── gitlab.com/
│   └── org/
│       └── project/
│           └── main/
│               └── script.js
└── git.company.com/
    └── team/
        └── project/
            └── main/
                └── build.js
```

Once cached, recipes are reused without re-fetching (unless you clear the cache).

## API Reference

### Fabrik.exec(command, args)

Execute a shell command with streaming output.

**Parameters:**
- `command` (string): Command to execute
- `args` (array): Command arguments

**Returns:** Exit code (number)

**Example:**
```javascript
async function build() {
    const exitCode = await Fabrik.exec("npm", ["run", "build"]);
    if (exitCode !== 0) {
        throw new Error("Build failed");
    }
}
```

### Fabrik.glob(pattern)

Find files matching a glob pattern.

**Parameters:**
- `pattern` (string): Glob pattern (e.g., `"src/**/*.ts"`)

**Returns:** Array of file paths (strings)

**Example:**
```javascript
async function test() {
    const testFiles = await Fabrik.glob("tests/**/*.test.js");
    console.log(`Found ${testFiles.length} test files`);
}
```

### Fabrik.exists(path)

Check if a file or directory exists.

**Parameters:**
- `path` (string): Path to check

**Returns:** Boolean

**Example:**
```javascript
async function deploy() {
    const hasConfig = await Fabrik.exists("deploy.config.json");
    if (!hasConfig) {
        throw new Error("Missing deploy configuration");
    }
}
```

## Example Recipes

### Simple Build Recipe

```javascript
// build.recipe.js
async function build() {
    console.log("Building project...");
    await Fabrik.exec("npm", ["install"]);
    await Fabrik.exec("npm", ["run", "build"]);
}
```

**Usage:**
```bash
fabrik run build.recipe.js -- build
```

### Multi-Target Recipe

```javascript
// ci.recipe.js

async function install() {
    console.log("Installing dependencies...");
    await Fabrik.exec("npm", ["install"]);
}

async function build() {
    await install();
    console.log("Building...");
    await Fabrik.exec("npm", ["run", "build"]);
}

async function test() {
    console.log("Running tests...");
    const exitCode = await Fabrik.exec("npm", ["test"]);
    if (exitCode !== 0) {
        throw new Error("Tests failed");
    }
}

async function all() {
    await build();
    await test();
    console.log("CI pipeline completed!");
}
```

**Usage:**
```bash
fabrik run ci.recipe.js -- install
fabrik run ci.recipe.js -- build
fabrik run ci.recipe.js -- test
fabrik run ci.recipe.js -- all
```

### Root-Level Recipe

```javascript
// Simple root-level script (no functions)
console.log("Running build...");

const files = await Fabrik.glob("src/**/*.ts");
console.log("Found", files.length, "TypeScript files");

await Fabrik.exec("tsc", ["--build"]);
```

**Usage:**
```bash
fabrik run build.recipe.js  # No target needed
```

### Remote Recipe Example

```javascript
// Published at: @tuist/recipes/typescript-build.js

async function build() {
    console.log("Building TypeScript project...");

    // Check dependencies
    const hasPackageJson = await Fabrik.exists("package.json");
    if (!hasPackageJson) {
        throw new Error("No package.json found");
    }

    // Install
    await Fabrik.exec("npm", ["install"]);

    // Build
    const exitCode = await Fabrik.exec("tsc", ["--build"]);
    if (exitCode !== 0) {
        throw new Error("TypeScript build failed");
    }

    // Verify outputs
    const distFiles = await Fabrik.glob("dist/**/*");
    console.log(`Generated ${distFiles.length} output files`);
}

async function clean() {
    console.log("Cleaning build artifacts...");
    await Fabrik.exec("rm", ["-rf", "dist"]);
}
```

**Usage:**
```bash
fabrik run @tuist/recipes/typescript-build.js -- build
fabrik run @tuist/recipes/typescript-build.js -- clean
```

## Comparison: Script vs Portable Recipes

| Feature | Script Recipes | Portable Recipes |
|---------|---------------|------------------|
| **Languages** | bash, python, ruby, node, etc. | JavaScript only |
| **Runtime** | External (must be installed) | Embedded QuickJS (always available) |
| **Cross-platform** | ⚠️ Depends on script | ✅ Guaranteed |
| **Remote execution** | ❌ Local files only | ✅ `@org/repo/script.js` |
| **Shareable** | ❌ Copy files manually | ✅ Git repositories |
| **Caching** | ✅ Content-addressed with KDL | ⚠️ Not yet implemented |
| **Status** | ✅ Working | ✅ Working |

## CLI Commands

### Run Remote Recipe
```bash
fabrik run @org/repo/path/script.js [-- target]
```

### Run Local Recipe
```bash
fabrik run ./local.recipe.js [-- target]
```

### Verbose Mode
```bash
fabrik run --verbose @tuist/recipes/build.js
```

Output:
```
[fabrik] Parsing remote recipe: @tuist/recipes/build.js
[fabrik] Remote recipe: github.com/tuist/recipes (ref: main)
[fabrik] Fetching from https://github.com/tuist/recipes.git
[fabrik] Recipe cached at: ~/.cache/fabrik/recipes/github.com/tuist/recipes/main/build.js
[fabrik] Executing target: build
Building project...
```

## Next Steps

### Phase 2: Caching

- Add content-addressed caching for recipe execution
- Cache recipe outputs based on inputs
- Implement cache invalidation strategies

### Phase 3: Registry

- Recipe registry for publishing/discovering recipes
- Versioning and dependency management
- Recipe templates and scaffolding

### Phase 4: Enhanced API

- `Fabrik.cache.get/put/has()` - Direct cache access
- `Fabrik.config()` - Runtime configuration
- `Fabrik.env` - Environment variables

## Files

- `mod.rs` - Module exports
- `remote.rs` - Remote recipe parsing and fetching
- `runtime.rs` - QuickJS runtime with LLRT + Fabrik API
- `executor.rs` - Recipe execution engine
- `README.md` - This documentation

## Development

### Run Tests
```bash
# All recipe tests
cargo test recipe

# Remote recipe tests only
cargo test remote_recipe

# Unit tests
cargo test --lib recipe::remote::tests
```

### Add New API

1. Add Rust function in `runtime.rs`
2. Bind to JavaScript in `create_fabrik_runtime()`
3. Document in this README
4. Add tests

### Publish a Recipe

1. Create repository: `github.com/org/recipes`
2. Add recipe files: `build.js`, `deploy.js`, etc.
3. Tag versions: `git tag v1.0.0 && git push --tags`
4. Use: `fabrik run @org/recipes/build.js@v1.0.0`

## Troubleshooting

### Recipe not found
```
Error: Script not found at recipes/build.js in repository
```

**Solution**: Verify the path exists in the repository at the specified ref.

### Git clone fails
```
Error: Failed to clone repository https://github.com/org/repo.git: ...
```

**Solutions:**
- Check repository is public (or you have SSH keys configured)
- Verify the repository exists
- Check network connectivity

### Recipe execution fails
```
Error: Failed to execute remote recipe
```

**Solution**: Run with `--verbose` to see detailed execution logs.
