# Portable Recipe Examples

This page shows practical examples of **portable recipes** - JavaScript recipes executed in Fabrik's embedded QuickJS runtime with built-in caching support.

## What Are Portable Recipes?

Portable recipes are JavaScript files that:
- ✅ Run in Fabrik's embedded runtime (no Node.js/Deno installation needed)
- ✅ Have access to Fabrik APIs (`fabrik:fs`, `child_process`)
- ✅ Can be distributed via Git repositories using `@` syntax
- ✅ Support content-addressed caching with FABRIK annotations
- ✅ Start instantly (~1ms vs ~50ms for Node.js)

All examples demonstrate how to leverage Fabrik's content-addressed caching to speed up repetitive build tasks.

## TypeScript Compilation Cache

Cache TypeScript compilation to avoid rebuilding unchanged code.

**Repository:** `@tuist/recipes/typescript-build.js@v1.0.0`

```javascript
// typescript-build.js
// FABRIK input "src/**/*.ts"
// FABRIK input "tsconfig.json"
// FABRIK input "package.json"
// FABRIK output "dist/"
// FABRIK env "NODE_ENV"

import { spawn } from 'child_process';
import { glob } from 'fabrik:fs';

console.log("Compiling TypeScript...");

const result = await spawn("tsc", ["--build"]);
if (result.exitCode !== 0) {
    throw new Error("TypeScript compilation failed");
}

const distFiles = await glob("dist/**/*");
console.log(`Generated ${distFiles.length} output files`);
```

**Cache Behavior:**
- ✅ Cache hits when source files unchanged
- ✅ Invalidates when `tsconfig.json` changes
- ✅ Separate caches for `NODE_ENV=production` vs `development`

**Usage:**
```bash
# First run: builds and caches (2.5s)
fabrik run @tuist/recipes/typescript-build.js@v1.0.0

# Second run: cache hit, restores dist/ instantly (0.1s)
fabrik run @tuist/recipes/typescript-build.js@v1.0.0
```

---

## Test Suite Cache

Cache test runs to skip running tests when code hasn't changed.

**Repository:** `@company/ci/test.js@v1.0.0`

```javascript
// test.js
// FABRIK input "src/**/*.{ts,tsx}"
// FABRIK input "tests/**/*.test.{ts,tsx}"
// FABRIK input "package.json"
// FABRIK input "jest.config.js"
// FABRIK output "coverage/"
// FABRIK env "CI"

import { spawn } from 'child_process';

console.log("Running test suite...");

const result = await spawn("npm", ["test", "--", "--coverage"]);
if (result.exitCode !== 0) {
    throw new Error("Tests failed");
}

console.log("All tests passed! Coverage report saved to coverage/");
```

**Cache Behavior:**
- ✅ Skips test runs when source and test files unchanged
- ✅ Restores previous coverage reports instantly
- ✅ Separate cache for CI vs local (via `CI` env var)

**Usage:**
```bash
# First run: executes full test suite (45s)
fabrik run @company/ci/test.js@v1.0.0

# Subsequent runs with no changes: instant (0.1s)
fabrik run @company/ci/test.js@v1.0.0
# Cache key: script-a8f3c9e2 | HIT ✓ | 0.10s (exit: 0)

# After changing a test file: cache miss, re-runs tests
echo "// new test" >> tests/app.test.ts
fabrik run @company/ci/test.js@v1.0.0
# Cache key: script-b9f4d0e3 | MISS ✗ | 45.2s (exit: 0)
```

---

## Asset Processing Cache

Cache expensive asset transformations like image optimization.

**Repository:** `@tuist/recipes/optimize-images.js@v1.0.0`

```javascript
// optimize-images.js
// FABRIK input "assets/images/**/*.{png,jpg,jpeg}"
// FABRIK output "public/images/"
// FABRIK cache ttl="30d"

import { spawn } from 'child_process';
import { glob } from 'fabrik:fs';

console.log("Optimizing images...");

const images = await glob("assets/images/**/*.{png,jpg,jpeg}");
console.log(`Found ${images.length} images to optimize`);

// Use imagemagick to optimize and resize
for (const img of images) {
    const outputPath = img.replace('assets/images', 'public/images');
    await spawn("convert", [
        img,
        "-resize", "1920x1920>",
        "-quality", "85",
        outputPath
    ]);
}

console.log(`Optimized ${images.length} images → public/images/`);
```

**Cache Behavior:**
- ✅ Only reprocesses images when source files change
- ✅ 30-day TTL keeps cache fresh
- ✅ Saves minutes on large image sets

**Performance:**
```bash
# First run: processes 150 images (2m 15s)
fabrik run @tuist/recipes/optimize-images.js@v1.0.0

# Subsequent runs: restores optimized images (0.2s)
fabrik run @tuist/recipes/optimize-images.js@v1.0.0

# After adding 5 new images: only processes new ones
# (In practice, recipe runs on all images, but cache invalidates the whole set)
```

---

## Docker Build Cache

Cache Docker builds to avoid rebuilding identical images.

**Repository:** `@tuist/recipes/docker-build.js@v1.0.0`

```javascript
// docker-build.js
// FABRIK input "Dockerfile"
// FABRIK input "src/**/*"
// FABRIK input "package.json"
// FABRIK input "package-lock.json"
// FABRIK output ".docker-cache/image.tar"
// FABRIK env "DOCKER_TAG"

import { spawn } from 'child_process';
import { existsSync } from 'fs';

console.log("Building Docker image...");

const tag = process.env.DOCKER_TAG || "myapp:latest";

// Check if cached image exists
if (existsSync(".docker-cache/image.tar")) {
    console.log("Loading cached Docker image...");
    await spawn("docker", ["load", "-i", ".docker-cache/image.tar"]);
    await spawn("docker", ["tag", "myapp:cached", tag]);
    console.log(`Loaded cached image as ${tag}`);
} else {
    console.log(`Building fresh image: ${tag}`);
    const result = await spawn("docker", [
        "build",
        "-t", tag,
        "."
    ]);

    if (result.exitCode !== 0) {
        throw new Error("Docker build failed");
    }

    // Save image to cache
    await spawn("docker", ["save", "-o", ".docker-cache/image.tar", tag]);
    console.log(`Built and cached image: ${tag}`);
}
```

**Cache Behavior:**
- ✅ Skips Docker build when Dockerfile and source unchanged
- ✅ Different cache for each `DOCKER_TAG` value
- ✅ Saves 5-10 minutes on large images

**Usage:**
```bash
# First build: runs docker build (8m 30s)
DOCKER_TAG=myapp:v1.2.3 fabrik run @tuist/recipes/docker-build.js@v1.0.0

# Subsequent builds: loads from cache (15s)
DOCKER_TAG=myapp:v1.2.3 fabrik run @tuist/recipes/docker-build.js@v1.0.0
# Cache key: script-f8a3d9c2 | HIT ✓ | 15.2s (exit: 0)
```

---

## Dependency Installation Cache

Cache `node_modules` to avoid reinstalling dependencies.

**Repository:** `@tuist/recipes/npm-install.js@v1.0.0`

```javascript
// npm-install.js
// FABRIK input "package.json"
// FABRIK input "package-lock.json"
// FABRIK output "node_modules/"
// FABRIK cache ttl="7d"

import { spawn } from 'child_process';
import { existsSync } from 'fs';

console.log("Installing dependencies...");

// Fabrik will restore node_modules/ if cache hits
if (!existsSync("node_modules")) {
    const result = await spawn("npm", ["ci"]);
    if (result.exitCode !== 0) {
        throw new Error("npm install failed");
    }
    console.log("Dependencies installed successfully");
} else {
    console.log("Dependencies restored from cache");
}
```

**Cache Behavior:**
- ✅ Skips `npm ci` when package-lock.json unchanged
- ✅ 7-day TTL ensures fresh dependencies
- ✅ Massive time savings in CI

**Performance:**
```bash
# First run: runs npm ci (45s)
fabrik run @tuist/recipes/npm-install.js@v1.0.0

# Subsequent runs: restores node_modules/ (2s)
fabrik run @tuist/recipes/npm-install.js@v1.0.0
# Cache key: script-c8f9a3e2 | HIT ✓ | 2.1s (exit: 0)

# After updating a dependency
npm install lodash@latest
fabrik run @tuist/recipes/npm-install.js@v1.0.0
# Cache key: script-d9f0b4e3 | MISS ✗ | 48.3s (exit: 0)
```

---

## Code Generation Cache

Cache generated code from protobuf, GraphQL schemas, or OpenAPI specs.

**Repository:** `@tuist/recipes/codegen.js@v1.0.0`

```javascript
// codegen.js
// FABRIK input "schema/**/*.proto"
// FABRIK input "codegen.config.js"
// FABRIK output "generated/"
// FABRIK cache ttl="30d"

import { spawn } from 'child_process';
import { glob } from 'fabrik:fs';

console.log("Running code generation...");

const protoFiles = await glob("schema/**/*.proto");
console.log(`Found ${protoFiles.length} proto files`);

// Generate TypeScript code from protobuf
const result = await spawn("protoc", [
    "--plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts",
    "--ts_out=generated/",
    ...protoFiles
]);

if (result.exitCode !== 0) {
    throw new Error("Code generation failed");
}

const generatedFiles = await glob("generated/**/*.ts");
console.log(`Generated ${generatedFiles.length} TypeScript files`);
```

**Cache Behavior:**
- ✅ Skips codegen when schemas unchanged
- ✅ Long TTL (30 days) for stable schemas
- ✅ Instant builds when schema stable

**Performance:**
```bash
# First run: runs protoc (12s)
fabrik run @tuist/recipes/codegen.js@v1.0.0
# Cache key: script-e8a9f3c2 | MISS ✗ | 12.4s (exit: 0)

# Subsequent runs: restores generated/ (0.3s)
fabrik run @tuist/recipes/codegen.js@v1.0.0
# Cache key: script-e8a9f3c2 | HIT ✓ | 0.30s (exit: 0)

# After updating schema
echo "message NewMessage {}" >> schema/api.proto
fabrik run @tuist/recipes/codegen.js@v1.0.0
# Cache key: script-f9b0a4d3 | MISS ✗ | 12.8s (exit: 0)
```

---

## Multi-Step Build Pipeline with Dependencies

Chain multiple cached recipes together for complex builds.

**Repository:** `@company/ci/full-build.js@v1.0.0`

```javascript
// full-build.js
// FABRIK depends "@company/ci/install-deps.js@v1.0.0" use-outputs=true
// FABRIK depends "@company/ci/lint.js@v1.0.0"
// FABRIK depends "@company/ci/test.js@v1.0.0" use-outputs=true
// FABRIK depends "@company/ci/build.js@v1.0.0" use-outputs=true
// FABRIK output "build/"
// FABRIK output "coverage/"

console.log("Running full CI pipeline...");
console.log("All dependent steps completed successfully");
console.log("Build artifacts ready in build/");
console.log("Coverage report available in coverage/");
```

**Dependency recipes:**

```javascript
// install-deps.js
// FABRIK input "package.json"
// FABRIK input "package-lock.json"
// FABRIK output "node_modules/"
import { spawn } from 'child_process';
await spawn("npm", ["ci"]);
```

```javascript
// lint.js
// FABRIK input "src/**/*.ts"
// FABRIK input ".eslintrc.js"
import { spawn } from 'child_process';
const result = await spawn("npm", ["run", "lint"]);
if (result.exitCode !== 0) throw new Error("Linting failed");
```

```javascript
// test.js
// FABRIK input "src/**/*.ts"
// FABRIK input "tests/**/*.test.ts"
// FABRIK output "coverage/"
import { spawn } from 'child_process';
await spawn("npm", ["test", "--", "--coverage"]);
```

```javascript
// build.js
// FABRIK input "src/**/*.ts"
// FABRIK input "tsconfig.json"
// FABRIK output "build/"
import { spawn } from 'child_process';
await spawn("npm", ["run", "build"]);
```

**Cache Behavior:**
- ✅ Each step caches independently
- ✅ Unchanged steps skip execution instantly
- ✅ Only changed steps re-run
- ✅ `use-outputs=true` shares artifacts between steps

**Example Workflow:**
```bash
# First run: all steps execute
fabrik run @company/ci/full-build.js@v1.0.0
# [fabrik] Running: install-deps.js (45s)
# [fabrik] Running: lint.js (8s)
# [fabrik] Running: test.js (32s)
# [fabrik] Running: build.js (28s)
# Total: 113s

# Second run: all cached
fabrik run @company/ci/full-build.js@v1.0.0
# [fabrik] Cache HIT: install-deps.js (0.2s)
# [fabrik] Cache HIT: lint.js (0.1s)
# [fabrik] Cache HIT: test.js (0.3s)
# [fabrik] Cache HIT: build.js (0.2s)
# Total: 0.8s

# After changing a test file
echo "new test" >> tests/app.test.ts
fabrik run @company/ci/full-build.js@v1.0.0
# [fabrik] Cache HIT: install-deps.js (0.2s)
# [fabrik] Cache HIT: lint.js (0.1s)
# [fabrik] Running: test.js (34s)  ← Only this re-runs
# [fabrik] Cache HIT: build.js (0.2s)
# Total: 34.5s
```

---

## Best Practices for Cache-Optimized Recipes

### 1. Declare All Inputs Explicitly

```javascript
// ✅ Good - explicit inputs ensure proper cache invalidation
// FABRIK input "src/**/*.ts"
// FABRIK input "tsconfig.json"
// FABRIK input "package.json"

// ❌ Bad - missing inputs cause stale cache hits
// (no input declarations)
```

### 2. Use Appropriate TTLs

```javascript
// Fast-changing data: short TTL
// FABRIK cache ttl="1h"

// Stable artifacts: longer TTL
// FABRIK cache ttl="30d"

// Build outputs: no TTL (invalidate by input hash)
// (no TTL declaration)
```

### 3. Track Environment Variables

```javascript
// ✅ Good - separate caches for different environments
// FABRIK env "NODE_ENV"
// FABRIK env "BUILD_TARGET"

// Cache keys will differ for:
// NODE_ENV=development vs NODE_ENV=production
// BUILD_TARGET=web vs BUILD_TARGET=mobile
```

### 4. Pin Versions in Production

```bash
# ✅ Production - pin to specific version
fabrik run @tuist/recipes/build.js@v1.2.3

# ⚠️ Development - can use branch
fabrik run @tuist/recipes/build.js@main

# ❌ Production - never use mutable refs
fabrik run @tuist/recipes/build.js@latest
```

### 5. Output Only What's Needed

```javascript
// ✅ Good - only cache essential outputs
// FABRIK output "dist/"
// FABRIK output "coverage/summary.json"

// ❌ Bad - caching unnecessary files wastes storage
// FABRIK output "dist/"
// FABRIK output "node_modules/"  // Too large, change frequently
// FABRIK output "*.log"          // Not needed
```

### 6. Use Dependencies for Build Steps

```javascript
// ✅ Good - chain steps with dependencies
// FABRIK depends "@company/ci/install.js@v1.0.0" use-outputs=true
// FABRIK depends "@company/ci/lint.js@v1.0.0"
// FABRIK depends "@company/ci/build.js@v1.0.0" use-outputs=true

// Each step caches independently
// Only changed steps re-run
```

---

## Cache Performance Comparison

| Scenario | Without Fabrik | With Fabrik (Cache Hit) | Speedup |
|----------|---------------|------------------------|---------|
| TypeScript build | 2.5s | 0.1s | **25x** |
| Test suite (300 tests) | 45s | 0.1s | **450x** |
| Image optimization (150 images) | 2m 15s | 0.2s | **675x** |
| Docker build | 8m 30s | 15s (load) | **34x** |
| npm install | 45s | 2s (restore) | **22x** |
| Protobuf codegen | 12s | 0.3s | **40x** |
| Full CI pipeline | 113s | 0.8s | **141x** |

> **Note:** Cache hit times include downloading from remote cache (if using Layer 2/3). Local cache hits are even faster (<0.1s).

---

## Publishing Your Own Recipes

### Step 1: Create Repository

```bash
mkdir my-recipes
cd my-recipes
git init
```

### Step 2: Add Recipe Files with Cache Annotations

```bash
# Create recipe with proper caching
cat > build.js << 'EOF'
// FABRIK input "src/**/*.ts"
// FABRIK input "tsconfig.json"
// FABRIK output "dist/"
// FABRIK env "NODE_ENV"

import { spawn } from 'child_process';

console.log("Building...");
await spawn("npm", ["run", "build"]);
console.log("Build complete!");
EOF

git add build.js
git commit -m "Add build recipe with caching"
```

### Step 3: Push to Git

```bash
# Push to GitHub
git remote add origin https://github.com/myorg/my-recipes.git
git push -u origin main

# Tag a version
git tag v1.0.0
git push --tags
```

### Step 4: Use Your Recipe

```bash
# Use from anywhere
fabrik run @myorg/my-recipes/build.js@v1.0.0

# First run: builds and caches
# Second run: instant cache hit
```

---

## Next Steps

- [JavaScript API Reference](/cache/recipes/api-reference) - Complete API documentation for portable recipe development
- [Syntax Reference](/cache/recipes/portable/syntax) - Learn the full `@` prefix syntax for fetching portable recipes
- [Standard Recipes](/cache/recipes/) - Learn about standard script recipes (bash, node, python) with caching annotations
