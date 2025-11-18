# Remote Recipe Examples

## TypeScript Build Recipe

A reusable recipe for building TypeScript projects.

**Repository:** `@tuist/recipes/typescript-build.js@v1.0.0`

```javascript
// typescript-build.js
async function build() {
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
}

async function clean() {
    console.log("Cleaning build artifacts...");
    await Fabrik.exec("rm", ["-rf", "dist", "node_modules"]);
}

async function test() {
    console.log("Running tests...");
    const exitCode = await Fabrik.exec("npm", ["test"]);
    if (exitCode !== 0) {
        throw new Error("Tests failed");
    }
}
```

**Usage:**
```bash
# Build
fabrik run @tuist/recipes/typescript-build.js@v1.0.0 -- build

# Clean
fabrik run @tuist/recipes/typescript-build.js@v1.0.0 -- clean

# Test
fabrik run @tuist/recipes/typescript-build.js@v1.0.0 -- test
```

---

## CI/CD Pipeline Recipe

A complete CI/CD pipeline recipe.

**Repository:** `@company/ci/pipeline.js@main`

```javascript
// pipeline.js
async function lint() {
    console.log("Running linter...");
    const exitCode = await Fabrik.exec("npm", ["run", "lint"]);
    if (exitCode !== 0) {
        throw new Error("Linting failed");
    }
}

async function test() {
    console.log("Running test suite...");
    const exitCode = await Fabrik.exec("npm", ["test", "--", "--coverage"]);
    if (exitCode !== 0) {
        throw new Error("Tests failed");
    }
}

async function build() {
    console.log("Building application...");
    await Fabrik.exec("npm", ["run", "build"]);

    const buildFiles = await Fabrik.glob("build/**/*");
    console.log(`Built ${buildFiles.length} files`);
}

async function deploy() {
    console.log("Deploying to staging...");
    await Fabrik.exec("aws", ["s3", "sync", "build/", "s3://staging-bucket/"]);
}

async function ci() {
    console.log("Running full CI pipeline...");
    await lint();
    await test();
    await build();
    console.log("CI pipeline completed successfully!");
}

async function cd() {
    console.log("Running CD pipeline...");
    await ci();
    await deploy();
    console.log("CD pipeline completed successfully!");
}
```

**Usage:**
```bash
# CI pipeline
fabrik run @company/ci/pipeline.js -- ci

# Full CD pipeline
fabrik run @company/ci/pipeline.js -- cd

# Individual steps
fabrik run @company/ci/pipeline.js -- lint
fabrik run @company/ci/pipeline.js -- test
```

---

## Multi-Environment Deployment

Separate recipes for different environments.

**Repository:** `@company/infra/deploy/`

**`staging.js`:**
```javascript
async function deploy() {
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
}
```

**`production.js`:**
```javascript
async function deploy() {
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
}
```

**Usage:**
```bash
# Deploy to staging
fabrik run @company/infra/deploy/staging.js -- deploy

# Deploy to production
fabrik run @company/infra/deploy/production.js -- deploy
```

---

## Docker Build Recipe

Build and push Docker images.

**Repository:** `@tuist/recipes/docker-build.js@v1.0.0`

```javascript
async function build() {
    console.log("Building Docker image...");

    // Get current git commit
    const gitSha = await getGitSha();
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
}

async function push() {
    console.log("Pushing Docker image...");

    const gitSha = await getGitSha();
    const imageTag = `myapp:${gitSha}`;

    // Push tagged image
    await Fabrik.exec("docker", ["push", imageTag]);

    // Push latest
    await Fabrik.exec("docker", ["push", "myapp:latest"]);

    console.log("Images pushed successfully!");
}

async function all() {
    await build();
    await push();
}

// Helper function
async function getGitSha() {
    // Simple implementation - in real usage you'd capture output
    return "abc123def";
}
```

**Usage:**
```bash
# Build only
fabrik run @tuist/recipes/docker-build.js@v1.0.0 -- build

# Build and push
fabrik run @tuist/recipes/docker-build.js@v1.0.0 -- all
```

---

## Monorepo Build Recipe

Build multiple packages in a monorepo.

**Repository:** `@company/monorepo/build.js@main`

```javascript
async function buildAll() {
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
}

async function buildPackage() {
    console.log("Building specific package...");
    // In real usage, package name would come from args
    await Fabrik.exec("npm", ["run", "build", "--workspace", "packages/core"]);
}

async function testAll() {
    console.log("Testing all packages...");

    const exitCode = await Fabrik.exec("npm", ["test", "--workspaces"]);

    if (exitCode !== 0) {
        throw new Error("Tests failed");
    }
}
```

**Usage:**
```bash
# Build all packages
fabrik run @company/monorepo/build.js -- buildAll

# Test all packages
fabrik run @company/monorepo/build.js -- testAll
```

---

## Version Pinning Best Practices

### Production: Pin to Specific Version

```bash
# ✅ Good - stable, predictable
fabrik run @tuist/recipes/build.js@v1.0.0 -- build

# ❌ Risky - may break unexpectedly
fabrik run @tuist/recipes/build.js@main -- build
```

### Development: Use Latest

```bash
# Development environment
fabrik run @company/ci/pipeline.js@develop -- ci

# Production environment
fabrik run @company/ci/pipeline.js@v2.1.0 -- ci
```

### Upgrading Recipes

```bash
# Test new version first
fabrik run @tuist/recipes/build.js@v2.0.0 -- build

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
# Create recipe
cat > build.js << 'EOF'
async function build() {
    console.log("Building...");
    await Fabrik.exec("npm", ["run", "build"]);
}
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
fabrik run @myorg/my-recipes/build.js@v1.0.0 -- build
```

---

## Next Steps

- [Syntax Reference](/cache/recipes/remote/syntax) - Learn the full `@` prefix syntax
- [Local Recipes](/cache/recipes/local/) - Learn about local script recipes
