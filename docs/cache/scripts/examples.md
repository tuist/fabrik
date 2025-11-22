# Examples

Real-world examples of script caching for common build workflows.

## TypeScript Compilation

Cache TypeScript compilation with automatic invalidation when source files change.

```bash
#!/usr/bin/env bash
#FABRIK input "src/**/*.ts"
#FABRIK input "src/**/*.tsx"
#FABRIK input "tsconfig.json"
#FABRIK input "package.json"
#FABRIK output "dist/"
#FABRIK env "NODE_ENV"

echo "Compiling TypeScript..."
npx tsc
```

**Run:**
```bash
fabrik run compile-ts.sh
```

## Code Generation

Cache code generation that depends on schema files.

```bash
#!/usr/bin/env node
#FABRIK input "schema/**/*.graphql"
#FABRIK input "codegen.yml"
#FABRIK output "src/generated/"

// Generate TypeScript types from GraphQL schema
const { exec } = require('child_process');
exec('graphql-codegen --config codegen.yml', (error, stdout, stderr) => {
  if (error) {
    console.error(stderr);
    process.exit(1);
  }
  console.log(stdout);
});
```

**Run:**
```bash
fabrik run codegen.js
```

## Image Optimization

Cache image processing with size-based input tracking (faster for large files).

```bash
#!/usr/bin/env bash
#FABRIK input "assets/images/**/*.png" hash=size
#FABRIK input "assets/images/**/*.jpg" hash=size
#FABRIK output "dist/images/"

echo "Optimizing images..."
mkdir -p dist/images

# Optimize all images
for img in assets/images/*; do
  filename=$(basename "$img")
  magick "$img" -quality 85 -strip "dist/images/$filename"
done
```

**Run:**
```bash
fabrik run optimize-images.sh
```

## Test Runner

Cache test execution when source files haven't changed.

```bash
#!/usr/bin/env bash
#FABRIK input "src/**/*.ts"
#FABRIK input "tests/**/*.test.ts"
#FABRIK input "jest.config.js"
#FABRIK output "coverage/"
#FABRIK env "CI"

echo "Running tests..."
npm test -- --coverage
```

**Run:**
```bash
fabrik run test.sh
```

**Note:** Only cache if tests are deterministic. Skip flaky tests or tests with external dependencies.

## Dependency Installation

Cache `node_modules` when `package.json` or lock file hasn't changed.

```bash
#!/usr/bin/env bash
#FABRIK input "package.json"
#FABRIK input "package-lock.json"
#FABRIK output "node_modules/"

echo "Installing dependencies..."
npm ci
```

**Run:**
```bash
fabrik run install-deps.sh
```

## Docker Image Build

Cache Docker image builds with multi-stage caching.

```bash
#!/usr/bin/env bash
#FABRIK input "Dockerfile"
#FABRIK input "src/**/*.go"
#FABRIK input "go.mod"
#FABRIK input "go.sum"
#FABRIK output "image-digest.txt"
#FABRIK env "DOCKER_BUILDKIT"

echo "Building Docker image..."
docker build -t myapp:latest . --iidfile=image-digest.txt
```

**Run:**
```bash
fabrik run docker-build.sh
```

## Multi-Step Pipeline

Chain multiple scripts with dependencies.

**Step 1: Install dependencies**
```bash
#!/usr/bin/env bash
# install.sh
#FABRIK input "package.json"
#FABRIK input "package-lock.json"
#FABRIK output "node_modules/"

npm ci
```

**Step 2: Build application**
```bash
#!/usr/bin/env bash
# build.sh
#FABRIK input "src/**/*.ts"
#FABRIK input "tsconfig.json"
#FABRIK depends "./install.sh" use-outputs=true
#FABRIK output "dist/"

npm run build
```

**Step 3: Run tests**
```bash
#!/usr/bin/env bash
# test.sh
#FABRIK input "tests/**/*.test.ts"
#FABRIK depends "./build.sh" use-outputs=true
#FABRIK output "coverage/"

npm test -- --coverage
```

**Run:**
```bash
# Run the entire pipeline (automatically resolves dependencies)
fabrik run test.sh
```

## Linting & Formatting

Cache linting results when source files haven't changed.

```bash
#!/usr/bin/env bash
#FABRIK input "src/**/*.ts"
#FABRIK input ".eslintrc.js"
#FABRIK input ".prettierrc"
#FABRIK output ".lint-cache/"

echo "Running ESLint..."
npx eslint src/ --cache --cache-location .lint-cache/

echo "Running Prettier..."
npx prettier src/ --check
```

**Run:**
```bash
fabrik run lint.sh
```

## Asset Bundling

Cache webpack/rollup builds with proper input tracking.

```bash
#!/usr/bin/env bash
#FABRIK input "src/**/*.js"
#FABRIK input "src/**/*.css"
#FABRIK input "webpack.config.js"
#FABRIK input "package.json"
#FABRIK output "dist/"
#FABRIK env "NODE_ENV"
#FABRIK exec timeout="10m"

echo "Bundling assets..."
npx webpack --mode production
```

**Run:**
```bash
NODE_ENV=production fabrik run bundle.sh
```

## Python Data Processing

Cache Python script execution with virtual environment.

```python
#!/usr/bin/env python3
#FABRIK input "data/*.csv"
#FABRIK input "requirements.txt"
#FABRIK output "processed/"
#FABRIK env "PROCESSING_MODE"

import pandas as pd
import os

mode = os.getenv('PROCESSING_MODE', 'standard')
print(f"Processing data in {mode} mode...")

# Read all CSV files
for file in os.listdir('data'):
    if file.endswith('.csv'):
        df = pd.read_csv(f'data/{file}')
        # Process data...
        df.to_csv(f'processed/{file}', index=False)
```

**Run:**
```bash
fabrik run process-data.py
```

## Protobuf Generation

Cache protobuf compilation.

```bash
#!/usr/bin/env bash
#FABRIK input "proto/**/*.proto"
#FABRIK output "src/generated/"

echo "Generating protobuf code..."
mkdir -p src/generated

protoc \
  --proto_path=proto \
  --go_out=src/generated \
  --go_opt=paths=source_relative \
  proto/**/*.proto
```

**Run:**
```bash
fabrik run gen-proto.sh
```

## Environment-Specific Builds

Use environment variables to create separate caches per environment.

```bash
#!/usr/bin/env bash
#FABRIK input "src/**/*.ts"
#FABRIK input "config/${BUILD_ENV}.json"
#FABRIK output "dist/"
#FABRIK env "BUILD_ENV"
#FABRIK env "API_URL"

echo "Building for $BUILD_ENV environment..."
npm run build -- --env=$BUILD_ENV
```

**Run:**
```bash
# Each environment gets its own cache
BUILD_ENV=development fabrik run build.sh
BUILD_ENV=staging fabrik run build.sh
BUILD_ENV=production fabrik run build.sh
```

## Custom Cache Keys

Manually version your cache for breaking changes.

```bash
#!/usr/bin/env bash
#FABRIK input "src/**/*.ts"
#FABRIK output "dist/"
#FABRIK cache key="v2"  # Bump this to invalidate all caches

# After major refactoring, bump key to v3 to bust all caches
npm run build
```

**Run:**
```bash
fabrik run build.sh
```

## Time-Based Cache Expiration

Cache nightly reports with 24-hour expiration.

```bash
#!/usr/bin/env bash
#FABRIK input "data/*.log"
#FABRIK output "reports/"
#FABRIK cache ttl="24h"

echo "Generating daily report..."
./generate-report.sh > reports/$(date +%Y-%m-%d).html
```

**Run:**
```bash
fabrik run daily-report.sh
```

## CI/CD Integration

Example GitHub Actions workflow using script caching.

```yaml
name: Build

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Install Fabrik
      - run: curl -fsSL https://fabrik.sh/install | bash

      # All scripts use caching automatically
      - run: fabrik run install-deps.sh
      - run: fabrik run build.sh
      - run: fabrik run test.sh

      # Check cache statistics
      - run: fabrik cache stats
```

## Advanced: Conditional Caching

Disable caching for release builds.

```bash
#!/usr/bin/env bash
if [ "$RELEASE_BUILD" = "true" ]; then
  #FABRIK cache disable
fi

#FABRIK input "src/**/*.ts"
#FABRIK output "dist/"

npm run build
```

**Run:**
```bash
# Development build (cached)
fabrik run build.sh

# Release build (not cached)
RELEASE_BUILD=true fabrik run build.sh
```

## Debugging

Use verbose mode to see exactly what's happening.

```bash
# See input/output tracking and cache operations
fabrik run --verbose build.sh

# Dry run to see cache key without executing
fabrik run --dry-run build.sh

# Force re-execution
fabrik run --no-cache build.sh

# Clean cache and re-execute
fabrik run --clean build.sh
```

## See Also

- [Introduction](/cache/scripts/) - Overview and quick start
- [Configuration Reference](/cache/scripts/reference) - Complete directive reference
