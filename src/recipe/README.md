# Portable Recipes - Cross-Platform Automation (WIP)

**Status**: Implementation complete, temporarily disabled due to Deno API compatibility issues

## Overview

Portable recipes provide cross-platform automation using TypeScript/JavaScript that runs on Fabrik's embedded Deno runtime. Unlike script recipes (which require platform-specific runtimes like bash, python, node), portable recipes are guaranteed to work on any operating system.

## Architecture

```
┌─────────────────────────────────────┐
│ User Recipe (TypeScript)            │
│  build.recipe.ts                    │
│                                     │
│  import { $, glob } from            │
│          "fabrik:runtime"           │
│  import { get, put } from           │
│          "fabrik:cache"             │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│ Deno Runtime (embedded)             │
│  - TypeScript execution             │
│  - Fabrik extensions loaded         │
│  - Sandboxed environment            │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│ Fabrik Ops (Rust FFI)               │
│  - op_fabrik_run()                  │
│  - op_fabrik_glob()                 │
│  - op_fabrik_hash_file()            │
│  - op_fabrik_cache_get/put/has()    │
│  - op_fabrik_get_config()           │
└─────────────────────────────────────┘
```

## Implementation Status

### ✅ Completed

1. **Module Structure**
   - `mod.rs` - Main module
   - `ops.rs` - Rust ops (7 ops implemented)
   - `runtime.rs` - Deno runtime initialization
   - `metadata.rs` - Recipe metadata parsing
   - `executor.rs` - Recipe execution engine

2. **TypeScript API Modules**
   - `js/runtime.ts` - Execution primitives ($, run, glob, hashFile)
   - `js/cache.ts` - Cache operations (get, put, has)
   - `js/config.ts` - Configuration access

3. **Documentation**
   - Full API documentation in CLAUDE.md
   - Example recipes
   - Migration guide from script recipes

### ⚠️ Blocked

**Issue**: Error type conversion between Rust standard library errors and Deno's `AnyError`

The `#[op2]` macro in `deno_core` 0.368 requires that all errors implement the `JsErrorClass` trait. Currently:

- `std::io::Error` → Uses `?` operator → Tries to convert via `anyhow::Error` → Fails because `anyhow::Error` doesn't implement `JsErrorClass`
- `glob::PatternError` → Same issue

**Solution needed**: Either:
1. Manually convert errors using `.map_err(|e| deno_core::error::AnyError::from(e))`
2. Find a Deno-compatible error handling pattern
3. Wait for Deno to stabilize error handling API

## API Examples

### Fabrik Runtime Module

```typescript
// fabrik:runtime - Execution primitives
import { $, run, glob, hashFile } from "fabrik:runtime";

// Template literal for shell commands (zx-style)
await $`npm install`;
await $`tsc --build`;

// Explicit command execution
const exitCode = await run("npm", ["test"]);

// File globbing
const files = await glob("src/**/*.ts");

// Content hashing
const hash = hashFile("package.json");
```

### Fabrik Cache Module

```typescript
// fabrik:cache - Direct cache access
import { get, put, has } from "fabrik:cache";

// Check if cached
if (await has("abc123...")) {
  console.log("Cache hit!");
}

// Get from cache (returns Uint8Array or null)
const data = await get("abc123...");

// Put into cache
await put("abc123...", new Uint8Array([...]));
```

### Fabrik Config Module

```typescript
// fabrik:config - Runtime configuration
import { getConfig } from "fabrik:config";

const config = getConfig();
console.log("Cache dir:", config.cacheDir);
console.log("Upstream:", config.upstream);
```

## Example Recipe

```typescript
// build.recipe.ts
import { $, glob } from "fabrik:runtime";

export const recipe = {
  name: "typescript-build",
  version: "1.0.0",
  inputs: ["src/**/*.ts", "package.json"],
  outputs: ["dist/"],
  env: ["NODE_ENV"],
  cacheTtl: "7d",
};

export async function install() {
  await $`npm install`;
}

export async function build() {
  await install();
  await $`npm run build`;

  const files = await glob("dist/**/*");
  console.log(`Generated ${files.length} files`);
}

export async function test() {
  const exitCode = await run("npm", ["test"]);
  if (exitCode !== 0) {
    throw new Error("Tests failed");
  }
}

export async function all() {
  await build();
  await test();
}
```

## Usage (when enabled)

```bash
# Run recipe target
fabrik run build.recipe.ts build

# Install from registry
fabrik recipe install @tuist/typescript-build

# Run installed recipe
fabrik run @tuist/typescript-build build
```

## Comparison: Script vs Portable Recipes

| Feature | Script Recipes | Portable Recipes |
|---------|---------------|------------------|
| **Languages** | bash, python, ruby, node, etc. | TypeScript/JavaScript only |
| **Runtime** | External (must be installed) | Embedded Deno (always available) |
| **Cross-platform** | ⚠️ Depends on script | ✅ Guaranteed |
| **IDE Support** | Basic | ✅ Full LSP, autocomplete |
| **Shareable** | ❌ Not easily | ✅ Can publish to registry |
| **Status** | ✅ Working | ⚠️ WIP (blocked on Deno API) |

## Next Steps

1. Resolve error type conversion issues
2. Test with real Deno runtime
3. Enable module in `src/lib.rs`
4. Add CLI commands (`fabrik recipe run`, etc.)
5. Create formula registry
6. Publish example recipes

## Files

- `ops.rs` - 7 Rust ops exposing Fabrik functionality to JavaScript
- `runtime.rs` - Deno runtime initialization with Fabrik extensions
- `executor.rs` - Recipe execution engine
- `metadata.rs` - Recipe metadata parsing and validation
- `js/runtime.ts` - TypeScript wrapper for execution primitives
- `js/cache.ts` - TypeScript wrapper for cache operations
- `js/config.ts` - TypeScript wrapper for configuration access

## Development

To enable portable recipes:

1. Uncomment `deno_core = "0.368"` in `Cargo.toml`
2. Fix error type conversions in `ops.rs`
3. Uncomment `pub mod recipe` in `src/lib.rs`
4. Run `cargo build`

The infrastructure is complete and ready - just needs the Deno compatibility layer working.
