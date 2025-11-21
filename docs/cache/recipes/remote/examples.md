# Remote Recipe Examples

## TypeScript Build Recipe

A reusable recipe for building TypeScript projects.

**Repository:** `@tuist/recipes/typescript-build.js@v1.0.0`

```javascript
// typescript-build.js - Root-level script (no exported functions)
import { existsSync } from 'fs';
import { spawn } from 'child_process';
import { glob } from 'fabrik:fs';

console.log("Building TypeScript project...");

// Check dependencies
if (!existsSync("package.json")) {
    throw new Error("No package.json found");
}

// Install dependencies
console.log("Installing dependencies...");
await spawn("npm", ["install"]);

// Run TypeScript compiler
console.log("Compiling TypeScript...");
const result = await spawn("tsc", ["--build"]);
if (result.exitCode !== 0) {
    throw new Error("TypeScript compilation failed");
}

// Verify outputs
const distFiles = await glob("dist/**/*");
console.log(`Generated ${distFiles.length} output files`);
```

**Usage:**
```bash
fabrik run @tuist/recipes/typescript-build.js@v1.0.0
```

---

## CI/CD Pipeline Recipe

A complete CI/CD pipeline recipe.

**Repository:** `@company/ci/lint.js`, `@company/ci/test.js`, `@company/ci/build.js`

**`lint.js`:**
```javascript
import { spawn } from 'child_process';

console.log("Running linter...");
const result = await spawn("npm", ["run", "lint"]);
if (result.exitCode !== 0) {
    throw new Error("Linting failed");
}
console.log("Linting passed!");
```

**`test.js`:**
```javascript
import { spawn } from 'child_process';

console.log("Running test suite...");
const result = await spawn("npm", ["test", "--", "--coverage"]);
if (result.exitCode !== 0) {
    throw new Error("Tests failed");
}
console.log("All tests passed!");
```

**`build.js`:**
```javascript
import { spawn } from 'child_process';
import { glob } from 'fabrik:fs';

console.log("Building application...");
await spawn("npm", ["run", "build"]);

const buildFiles = await glob("build/**/*");
console.log(`Built ${buildFiles.length} files`);
console.log("Build complete!");
```

**Usage:**
```bash
# Run CI steps
fabrik run @company/ci/lint.js
fabrik run @company/ci/test.js
fabrik run @company/ci/build.js
```

---

## Multi-Environment Deployment

Separate recipes for different environments.

**Repository:** `@company/infra/deploy/`

**`staging.js`:**
```javascript
import { spawn } from 'child_process';

console.log("Deploying to STAGING environment...");

// Build with staging config
await spawn("npm", ["run", "build:staging"]);

// Deploy to staging
await spawn("aws", [
    "s3", "sync", "build/",
    "s3://staging-bucket/",
    "--delete"
]);

console.log("Staging deployment complete!");
console.log("URL: https://staging.example.com");
```

**`production.js`:**
```javascript
import { spawn } from 'child_process';

console.log("Deploying to PRODUCTION environment...");

// Build with production config
await spawn("npm", ["run", "build:production"]);

// Deploy to production
await spawn("aws", [
    "s3", "sync", "build/",
    "s3://production-bucket/",
    "--delete"
]);

// Invalidate CloudFront cache
await spawn("aws", [
    "cloudfront", "create-invalidation",
    "--distribution-id", "E1234567890ABC",
    "--paths", "/*"
]);

console.log("Production deployment complete!");
console.log("URL: https://example.com");
```

**Usage:**
```bash
# Deploy to staging
fabrik run @company/infra/deploy/staging.js

# Deploy to production
fabrik run @company/infra/deploy/production.js
```

---

## Docker Build Recipe

Build and push Docker images.

**Repository:** `@tuist/recipes/docker-build.js@v1.0.0`

```javascript
import { spawn } from 'child_process';

console.log("Building Docker image...");

// Get current git commit (simplified - in real usage you'd capture output)
const gitSha = "abc123def";
const imageTag = `myapp:${gitSha}`;

console.log(`Building image: ${imageTag}`);

const result = await spawn("docker", [
    "build",
    "-t", imageTag,
    "-t", "myapp:latest",
    "."
]);

if (result.exitCode !== 0) {
    throw new Error("Docker build failed");
}

console.log(`Built image: ${imageTag}`);

// Push images
console.log("Pushing Docker images...");
await spawn("docker", ["push", imageTag]);
await spawn("docker", ["push", "myapp:latest"]);

console.log("Images pushed successfully!");
```

**Usage:**
```bash
fabrik run @tuist/recipes/docker-build.js@v1.0.0
```

---

## Monorepo Build Recipe

Build multiple packages in a monorepo.

**Repository:** `@company/monorepo/build-all.js@main`

```javascript
import { spawn } from 'child_process';

console.log("Building all packages...");

const packages = [
    "packages/core",
    "packages/utils",
    "packages/ui",
    "packages/app"
];

for (const pkg of packages) {
    console.log(`Building ${pkg}...`);

    const result = await spawn("npm", [
        "run", "build",
        "--workspace", pkg
    ]);

    if (result.exitCode !== 0) {
        throw new Error(`Build failed for ${pkg}`);
    }
}

console.log("All packages built successfully!");
```

**Usage:**
```bash
fabrik run @company/monorepo/build-all.js
```

---

## Conditional Logic Recipe

Recipe with conditional execution based on file existence.

**Repository:** `@tuist/recipes/smart-build.js@v1.0.0`

```javascript
import { existsSync } from 'fs';
import { spawn } from 'child_process';
import { glob } from 'fabrik:fs';

console.log("Smart build - checking what needs to be built...");

// Check if package.json changed
if (existsSync("package.json")) {
    console.log("Installing dependencies...");
    await spawn("npm", ["install"]);
}

// Check if TypeScript files exist
const tsFiles = await glob("src/**/*.ts");
if (tsFiles.length > 0) {
    console.log(`Found ${tsFiles.length} TypeScript files, compiling...`);
    await spawn("tsc", ["--build"]);
}

// Check if tests exist
const testFiles = await glob("tests/**/*.test.js");
if (testFiles.length > 0) {
    console.log(`Found ${testFiles.length} test files, running tests...`);
    const result = await spawn("npm", ["test"]);
    if (result.exitCode !== 0) {
        throw new Error("Tests failed");
    }
}

console.log("Smart build complete!");
```

**Usage:**
```bash
fabrik run @tuist/recipes/smart-build.js@v1.0.0
```

---

## Version Pinning Best Practices

### Production: Pin to Specific Version

```bash
# ✅ Good - stable, predictable
fabrik run @tuist/recipes/build.js@v1.0.0

# ❌ Risky - may break unexpectedly
fabrik run @tuist/recipes/build.js@main
```

### Development: Use Latest

```bash
# Development environment
fabrik run @company/ci/pipeline.js@develop

# Production environment
fabrik run @company/ci/pipeline.js@v2.1.0
```

### Upgrading Recipes

```bash
# Test new version first
fabrik run @tuist/recipes/build.js@v2.0.0

# If successful, update in CI config
# Update: @tuist/recipes/build.js@v1.0.0
# To:     @tuist/recipes/build.js@v2.0.0
```

---

## Publishing Your Own Recipes

### Step 1: Create Repository

```bash
mkdir my-recipes
cd my-recipes
git init
```

### Step 2: Add Recipe Files

```bash
# Create recipe (root-level script)
cat > build.js << 'EOF'
import { spawn } from 'child_process';

console.log("Building...");
await spawn("npm", ["run", "build"]);
console.log("Build complete!");
EOF

git add build.js
git commit -m "Add build recipe"
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
```

---

## Next Steps

- [JavaScript API Reference](/cache/recipes/api-reference) - Complete API documentation for recipe development
- [Syntax Reference](/cache/recipes/remote/syntax) - Learn the full `@` prefix syntax
- [Local Recipes](/cache/recipes/local/) - Learn about local script recipes
