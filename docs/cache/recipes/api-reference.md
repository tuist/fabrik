# JavaScript API Reference

Fabrik recipes have access to Node.js-compatible APIs (via LLRT) and Fabrik-specific APIs for caching and file operations.

## Module Overview

### Standard Modules (LLRT)

These modules provide Node.js-compatible APIs:

- **`fs`** - File system operations (synchronous)
- **`fs/promises`** - File system operations (promise-based)
- **`child_process`** - Process spawning
- **`path`** - Path manipulation utilities
- **`buffer`** - Buffer and binary data handling
- **`console`** - Console logging

### Fabrik Modules

These modules provide Fabrik-specific functionality:

- **`fabrik:cache`** - Content-addressed caching
- **`fabrik:fs`** - File utilities (glob, hashing)
- **`fabrik:kv`** - Key-value storage for cache metadata

---

## Fabrik Global Object

The `Fabrik` global object is automatically available in all recipes and provides core functionality for file operations, process execution, and caching.

### `Fabrik.readFile(path)`

Read a file as a byte array.

**Parameters:**
- `path` (string): Path to the file

**Returns:**
- `Promise<Uint8Array>`: File contents as bytes

**Example:**
```javascript
const data = await Fabrik.readFile("config.json");
const text = new TextDecoder().decode(data);
```

---

### `Fabrik.writeFile(path, data)`

Write data to a file.

**Parameters:**
- `path` (string): Path to the file
- `data` (Uint8Array): Data to write

**Returns:**
- `Promise<void>`

**Example:**
```javascript
const data = new TextEncoder().encode("Hello World");
await Fabrik.writeFile("output.txt", data);
```

---

### `Fabrik.exists(path)`

Check if a file or directory exists.

**Parameters:**
- `path` (string): Path to check

**Returns:**
- `Promise<boolean>`: `true` if exists, `false` otherwise

**Example:**
```javascript
if (await Fabrik.exists("dist/")) {
  console.log("Build output exists");
}
```

---

### `Fabrik.glob(pattern)`

Find files matching a glob pattern.

**Parameters:**
- `pattern` (string): Glob pattern

**Returns:**
- `Promise<string[]>`: Array of matching file paths

**Example:**
```javascript
const tsFiles = await Fabrik.glob("src/**/*.ts");
console.log(`Found ${tsFiles.length} TypeScript files`);
```

---

### `Fabrik.exec(command, args)`

Execute a command and return the exit code.

**Parameters:**
- `command` (string): Command to execute
- `args` (string[], optional): Command arguments

**Returns:**
- `Promise<number>`: Exit code (0 = success)

**Example:**
```javascript
const exitCode = await Fabrik.exec("npm", ["run", "build"]);
if (exitCode !== 0) {
  throw new Error(`Build failed with exit code ${exitCode}`);
}
```

> [!TIP]
> For commands that need stdout/stderr capture, use LLRT's `child_process` module.

---

### `Fabrik.hashFile(path)`

Compute SHA256 hash of a file.

**Parameters:**
- `path` (string): Path to the file

**Returns:**
- `Promise<string>`: SHA256 hash as hex string (64 characters)

**Example:**
```javascript
const hash = await Fabrik.hashFile("package-lock.json");
console.log(`Package lock hash: ${hash.slice(0, 8)}`);
```

---

### `Fabrik.cache`

Low-level cache operations object. Contains:
- `Fabrik.cache.get(hash)` - Get artifact from cache
- `Fabrik.cache.put(hash, data)` - Store artifact in cache
- `Fabrik.cache.has(hash)` - Check if artifact exists

> [!NOTE]
> These are low-level APIs. Most recipes should use `runCached()` from `fabrik:cache` instead.

---

## fabrik:cache

Content-addressed caching APIs for recipe optimization.

### `runCached(action, options)`

Run an action only if cache miss. Automatically handles cache checking, action execution, and output archiving.

**Parameters:**
- `action` (Function): Async function to execute on cache miss
- `options` (Object): Cache configuration
  - `inputs` (string[]): Input file patterns (globs) that affect cache key
  - `outputs` (string[]): Output paths (files or directories) to cache
  - `env` (string[]): Environment variable names to include in cache key
  - `cacheDir` (string, optional): Override cache directory (default: `.fabrik/cache`)
  - `upstream` (string[], optional): Override upstream cache servers
  - `ttl` (string, optional): Cache expiration (e.g., `"7d"`, `"2h"`)
  - `hashMethod` (string, optional): How to hash input files (`"content"`, `"mtime"`, `"size"`)

**Returns:**
- `Promise<Object>`:
  - `cacheKey` (string): Computed cache key (SHA256 hash)
  - `cached` (boolean): `true` if cache hit, `false` if cache miss
  - `restoredFiles` (string[], optional): Files restored on cache hit
  - `durationMs` (number, optional): Execution time in milliseconds on cache miss

**Example:**

```javascript
import { runCached } from 'fabrik:cache';

const result = await runCached(
  async () => {
    // Build logic (only runs on cache miss)
    const exitCode = await Fabrik.exec("npm", ["run", "build"]);
    if (exitCode !== 0) {
      throw new Error("Build failed");
    }
  },
  {
    inputs: ["src/**/*.ts", "tsconfig.json"],
    outputs: ["dist/"],
    env: ["NODE_ENV"],
    ttl: "7d"
  }
);

if (result.cached) {
  console.log(`Cache HIT: ${result.cacheKey.slice(0, 8)}`);
  console.log(`Restored ${result.restoredFiles.length} files`);
} else {
  console.log(`Cache MISS: ${result.cacheKey.slice(0, 8)}`);
  console.log(`Built in ${result.durationMs}ms`);
}
```

---

### `needsRun(options)`

Check if action needs to run based on cache state. Useful when you want to control the execution logic yourself.

**Parameters:**
- `options` (Object): Same as `runCached` (except no `outputs` required)
  - `inputs` (string[]): Input file patterns to check
  - `env` (string[]): Environment variables to check
  - `cacheDir` (string, optional): Override cache directory
  - `hashMethod` (string, optional): Hash method

**Returns:**
- `Promise<boolean>`: `true` if action needs to run (cache miss), `false` if cached

**Example:**

```javascript
import { needsRun } from 'fabrik:cache';

const shouldRun = await needsRun({
  inputs: ["src/**/*.rs", "Cargo.toml"],
  env: ["RUSTFLAGS"]
});

if (shouldRun) {
  console.log("Source changed, rebuilding...");
  const exitCode = await Fabrik.exec("cargo", ["build", "--release"]);
  if (exitCode !== 0) {
    throw new Error("Build failed");
  }
} else {
  console.log("Nothing changed, skipping build");
}
```

---

## fabrik:fs

File system utilities for recipes.

### `glob(pattern)`

Find files matching a glob pattern.

**Parameters:**
- `pattern` (string): Glob pattern (e.g., `"src/**/*.ts"`, `"*.json"`)

**Returns:**
- `Promise<string[]>`: Array of matching file paths

**Example:**

```javascript
import { glob } from 'fabrik:fs';

// Find all TypeScript files
const tsFiles = await glob("src/**/*.ts");
console.log(`Found ${tsFiles.length} TypeScript files`);

// Find all test files
const testFiles = await glob("tests/**/*.test.js");

// Multiple patterns require multiple calls
const configFiles = [
  ...await glob("*.json"),
  ...await glob("*.toml"),
  ...await glob("*.yml")
];
```

**Supported glob syntax:**
- `*` - Match any characters except `/`
- `**` - Match any characters including `/` (recursive)
- `?` - Match single character
- `[abc]` - Match any character in set
- `{a,b}` - Match either `a` or `b`

---

### `hashFile(path)`

Compute SHA256 hash of a file.

**Parameters:**
- `path` (string): File path to hash

**Returns:**
- `Promise<string>`: SHA256 hash as hex string (64 characters)

**Example:**

```javascript
import { hashFile } from 'fabrik:fs';

// Check if file changed
const currentHash = await hashFile("package-lock.json");
const previousHash = "abc123..."; // Stored somewhere

if (currentHash !== previousHash) {
  console.log("Dependencies changed, reinstalling...");
  await Fabrik.exec("npm", ["install"]);
}

// Use hash as cache key component
const buildHash = await hashFile("dist/bundle.js");
console.log(`Build hash: ${buildHash}`);
```

---

## fabrik:kv

Low-level key-value storage for cache metadata.

> [!NOTE]
> Most recipes should use `runCached()` or `needsRun()` instead of accessing KV directly. The KV API is for advanced use cases where you need custom cache key logic.

### `has(key)`

Check if key exists in KV store.

**Parameters:**
- `key` (string): Key to check

**Returns:**
- `Promise<boolean>`: `true` if key exists, `false` otherwise

**Example:**

```javascript
import { has } from 'fabrik:kv';

const cacheKey = "build-v1-prod-abc123";
if (await has(cacheKey)) {
  console.log("Already built this version");
} else {
  console.log("Need to build");
}
```

---

### `get(key)`

Get value from KV store.

**Parameters:**
- `key` (string): Key to retrieve

**Returns:**
- `Promise<Object | null>`: Stored value as object, or `null` if key doesn't exist

**Example:**

```javascript
import { get } from 'fabrik:kv';

const metadata = await get("build-metadata");
if (metadata) {
  console.log("Last build:", metadata);
} else {
  console.log("No previous build metadata");
}
```

---

### `set(key, value)`

Store value in KV store.

**Parameters:**
- `key` (string): Key to store
- `value` (any): Value to store (will be JSON serialized)

**Returns:**
- `Promise<void>`

**Example:**

```javascript
import { set } from 'fabrik:kv';

// Store build metadata
await set("build-metadata", {
  timestamp: Date.now(),
  version: "1.0.0",
  commit: "abc123"
});
```

---

## Standard Node.js APIs (via LLRT)

### fs (File System)

```javascript
import { existsSync, readFileSync, writeFileSync } from 'fs';

// Check if file exists
if (existsSync("package.json")) {
  console.log("Found package.json");
}

// Read file (synchronous)
const content = readFileSync("config.json", "utf-8");

// Write file (synchronous)
writeFileSync("output.txt", "Hello world");
```

**Available functions:**
- `existsSync(path)` - Check if file/directory exists
- `readFileSync(path, encoding)` - Read file synchronously
- `writeFileSync(path, data)` - Write file synchronously
- `mkdirSync(path, options)` - Create directory
- `statSync(path)` - Get file stats
- And more...

See [LLRT fs documentation](https://github.com/awslabs/llrt/blob/main/llrt_modules/README.md#fs) for full API.

---

### fs/promises (Async File System)

```javascript
import { readFile, writeFile, mkdir } from 'fs/promises';

// Read file (async)
const data = await readFile("config.json", "utf-8");

// Write file (async)
await writeFile("output.txt", "Hello world");

// Create directory
await mkdir("build", { recursive: true });
```

**Available functions:**
- `readFile(path, encoding)` - Read file
- `writeFile(path, data)` - Write file
- `mkdir(path, options)` - Create directory
- `stat(path)` - Get file stats
- And more...

---

### child_process (Process Spawning)

> [!NOTE]
> LLRT's `child_process` module is provided for Node.js compatibility but may have different behavior than Node.js. For simpler process execution, consider using the global `Fabrik.exec()` function.

```javascript
import { spawn } from 'child_process';

// Spawn process using LLRT's child_process module
// See LLRT documentation for exact API behavior
const child = spawn("npm", ["install"]);
```

**Alternative: Fabrik.exec() (recommended)**

The global `Fabrik.exec()` function provides simpler process execution:

```javascript
// Simpler process execution via Fabrik global
const exitCode = await Fabrik.exec("npm", ["install"]);

if (exitCode === 0) {
  console.log("Command succeeded");
} else {
  throw new Error(`Command failed with exit code: ${exitCode}`);
}
```

> [!TIP]
> `Fabrik.exec()` currently returns only the exit code. Support for capturing stdout/stderr may be added in a future release.

---

### path (Path Utilities)

```javascript
import { join, basename, dirname, extname } from 'path';

// Join paths
const fullPath = join("src", "components", "Button.tsx");
// => "src/components/Button.tsx"

// Extract filename
const filename = basename("/path/to/file.txt");
// => "file.txt"

// Extract directory
const dir = dirname("/path/to/file.txt");
// => "/path/to"

// Extract extension
const ext = extname("file.txt");
// => ".txt"
```

---

### buffer (Binary Data)

```javascript
import { Buffer } from 'buffer';

// Create buffer from string
const buf = Buffer.from("Hello", "utf-8");

// Convert buffer to string
const str = buf.toString("utf-8");

// Create buffer from array
const buf2 = Buffer.from([0x48, 0x65, 0x6c, 0x6c, 0x6f]);
```

---

## Configuration Discovery

When recipes call `runCached()` or `needsRun()`, Fabrik automatically discovers configuration:

1. **Start from recipe location**: Look for `fabrik.toml` in the directory containing the recipe
2. **Traverse up**: Walk up parent directories until `fabrik.toml` is found
3. **Global fallback**: Use `~/.config/fabrik/config.toml` if no project config found
4. **Runtime override**: Parameters passed to API calls take precedence

**Configuration precedence** (highest to lowest):
1. Runtime parameters (passed to `runCached()` / `needsRun()`)
2. Project `fabrik.toml` (discovered from recipe directory)
3. Global `~/.config/fabrik/config.toml`

**Example:**

```javascript
// Uses discovered fabrik.toml automatically
await runCached(
  async () => { /* action */ },
  { inputs: ["src/**/*.ts"], outputs: ["dist/"] }
);

// Override cache directory at runtime
await runCached(
  async () => { /* action */ },
  {
    inputs: ["src/**/*.ts"],
    outputs: ["dist/"],
    cacheDir: ".custom-cache"  // Override
  }
);
```

---

## Complete Example

Here's a comprehensive recipe using multiple APIs:

```javascript
import { existsSync } from 'fs';
import { join } from 'path';
import { runCached, needsRun } from 'fabrik:cache';
import { glob, hashFile } from 'fabrik:fs';

console.log("Building TypeScript project...");

// Check prerequisites
if (!existsSync("package.json")) {
  throw new Error("No package.json found");
}

// Install dependencies (with caching)
const depsChanged = await needsRun({
  inputs: ["package.json", "package-lock.json"],
  hashMethod: "content"
});

if (depsChanged) {
  console.log("Installing dependencies...");
  const exitCode = await Fabrik.exec("npm", ["install"]);
  if (exitCode !== 0) {
    throw new Error("npm install failed");
  }
}

// Build TypeScript (with caching)
const buildResult = await runCached(
  async () => {
    console.log("Compiling TypeScript...");
    const exitCode = await Fabrik.exec("npm", ["run", "build"]);

    if (exitCode !== 0) {
      throw new Error("TypeScript compilation failed");
    }
  },
  {
    inputs: ["src/**/*.ts", "tsconfig.json"],
    outputs: ["dist/"],
    env: ["NODE_ENV"],
    ttl: "7d"
  }
);

if (buildResult.cached) {
  console.log(`✓ Restored from cache (${buildResult.restoredFiles.length} files)`);
} else {
  console.log(`✓ Built in ${buildResult.durationMs}ms`);
}

// Verify outputs
const distFiles = await glob("dist/**/*.js");
console.log(`Generated ${distFiles.length} JavaScript files`);

// Compute bundle hash
const bundleHash = await hashFile("dist/bundle.js");
console.log(`Bundle hash: ${bundleHash.slice(0, 8)}`);
```

---

## See Also

- [Portable Recipes](/cache/recipes/portable/) - Execute recipes from Git repositories
- [Standard Recipes](/cache/recipes/standard/) - Script recipes with FABRIK annotations
- [Configuration Reference](/reference/config-file) - Fabrik configuration options
