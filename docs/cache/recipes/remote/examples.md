# Remote Recipe Examples

## TypeScript Build Recipe

A reusable recipe for building TypeScript projects.

**Repository:** `@tuist/recipes/typescript-build.js@v1.0.0`

```javascript
// typescript-build.js - Root-level script (no exported functions)
console.log("Building TypeScript project...");

// Check dependencies
const hasPackageJson = await Fabrik.exists("package.json");
if (!hasPackageJson) {
    throw new Error("No package.json found");
}

// Install dependencies
console.log("Installing dependencies...");
await Fabrik.exec("npm", ["install"]);

// Run TypeScript compiler
console.log("Compiling TypeScript...");
const exitCode = await Fabrik.exec("tsc", ["--build"]);
if (exitCode !== 0) {
    throw new Error("TypeScript compilation failed");
}

// Verify outputs
const distFiles = await Fabrik.glob("dist/**/*");
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
console.log("Running linter...");
const exitCode = await Fabrik.exec("npm", ["run", "lint"]);
if (exitCode !== 0) {
    throw new Error("Linting failed");
}
console.log("Linting passed!");
```

**`test.js`:**
```javascript
console.log("Running test suite...");
const exitCode = await Fabrik.exec("npm", ["test", "--", "--coverage"]);
if (exitCode !== 0) {
    throw new Error("Tests failed");
}
console.log("All tests passed!");
```

**`build.js`:**
```javascript
console.log("Building application...");
await Fabrik.exec("npm", ["run", "build"]);

const buildFiles = await Fabrik.glob("build/**/*");
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
console.log("Deploying to STAGING environment...");

// Build with staging config
await Fabrik.exec("npm", ["run", "build:staging"]);

// Deploy to staging
await Fabrik.exec("aws", [
    "s3", "sync", "build/",
    "s3://staging-bucket/",
    "--delete"
]);

console.log("Staging deployment complete!");
console.log("URL: https://staging.example.com");
```

**`production.js`:**
```javascript
console.log("Deploying to PRODUCTION environment...");

// Build with production config
await Fabrik.exec("npm", ["run", "build:production"]);

// Deploy to production
await Fabrik.exec("aws", [
    "s3", "sync", "build/",
    "s3://production-bucket/",
    "--delete"
]);

// Invalidate CloudFront cache
await Fabrik.exec("aws", [
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
console.log("Building Docker image...");

// Get current git commit (simplified - in real usage you'd capture output)
const gitSha = "abc123def";
const imageTag = `myapp:${gitSha}`;

console.log(`Building image: ${imageTag}`);

const exitCode = await Fabrik.exec("docker", [
    "build",
    "-t", imageTag,
    "-t", "myapp:latest",
    "."
]);

if (exitCode !== 0) {
    throw new Error("Docker build failed");
}

console.log(`Built image: ${imageTag}`);

// Push images
console.log("Pushing Docker images...");
await Fabrik.exec("docker", ["push", imageTag]);
await Fabrik.exec("docker", ["push", "myapp:latest"]);

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
console.log("Building all packages...");

const packages = [
    "packages/core",
    "packages/utils",
    "packages/ui",
    "packages/app"
];

for (const pkg of packages) {
    console.log(`Building ${pkg}...`);

    const exitCode = await Fabrik.exec("npm", [
        "run", "build",
        "--workspace", pkg
    ]);

    if (exitCode !== 0) {
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
console.log("Smart build - checking what needs to be built...");

// Check if package.json changed
const hasPackageJson = await Fabrik.exists("package.json");
if (hasPackageJson) {
    console.log("Installing dependencies...");
    await Fabrik.exec("npm", ["install"]);
}

// Check if TypeScript files exist
const tsFiles = await Fabrik.glob("src/**/*.ts");
if (tsFiles.length > 0) {
    console.log(`Found ${tsFiles.length} TypeScript files, compiling...`);
    await Fabrik.exec("tsc", ["--build"]);
}

// Check if tests exist
const testFiles = await Fabrik.glob("tests/**/*.test.js");
if (testFiles.length > 0) {
    console.log(`Found ${testFiles.length} test files, running tests...`);
    const exitCode = await Fabrik.exec("npm", ["test"]);
    if (exitCode !== 0) {
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
console.log("Building...");
await Fabrik.exec("npm", ["run", "build"]);
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

- [Syntax Reference](/cache/recipes/remote/syntax) - Learn the full `@` prefix syntax
- [Local Recipes](/cache/recipes/local/) - Learn about local script recipes
