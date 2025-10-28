# Nx

> **Note:** Nx support is currently in development. This documentation describes the planned implementation.

Fabrik will provide a wrapper command for Nx that automatically configures remote task caching via the Nx Remote Cache HTTP API.

## Usage

The `fabrik nx` command is a drop-in replacement for the standard `nx` command:

```bash
# Instead of: nx run-many --target=build
# Use:
fabrik nx -- run-many --target=build
```

All `nx` arguments and flags work as normal:

```bash
# Build all projects
fabrik nx -- run-many --target=build --all

# Run tests
fabrik nx -- run-many --target=test --all

# Build specific project
fabrik nx -- build my-app

# Run affected tasks
fabrik nx -- affected --target=build

# Run with parallel execution
fabrik nx -- run-many --target=build --all --parallel=3

# Show project graph
fabrik nx -- graph
```

## How It Works

When you run `fabrik nx`, Fabrik:

1. Starts a local HTTP server implementing the Nx Remote Cache HTTP API
2. Automatically configures Nx to use the cache via environment variables
3. Passes through all your nx arguments unchanged
4. Handles graceful shutdown when the command completes

The local cache server implements the following endpoints from the Nx Remote Cache HTTP API:
- **GET /**: Health check endpoint
- **GET /{hash}**: Retrieve cached task outputs by content hash
- **PUT /{hash}**: Store task outputs

## Configuration

Fabrik automatically configures Nx remote caching. You can optionally create an `nx.json` file in your project to customize cache behavior:

```json
{
  "tasksRunnerOptions": {
    "default": {
      "runner": "nx/tasks-runners/default",
      "options": {
        "cacheableOperations": ["build", "test", "lint"],
        "parallel": 3
      }
    }
  }
}
```

When using Fabrik, the remote cache URL is automatically configured via the `NX_CLOUD_API` environment variable, so you don't need to manually configure the remote cache endpoint.

## Requirements

- Nx must be installed in your project (via npm, yarn, or pnpm)
- Node.js version matching your project's requirements
- `npx` must be available (comes with npm 5.2+)

## How Tool Discovery Works

Unlike Bazel (which is typically in your system PATH), Nx is usually installed as a local Node.js dependency in `node_modules/.bin/`. When you run `fabrik nx`, Fabrik will use **npx** to discover and execute Nx:

```bash
npx nx <your-arguments>
```

This approach:
- ✅ **Finds local installations** - `npx` automatically resolves to `./node_modules/.bin/nx`
- ✅ **Works across package managers** - npm, yarn, pnpm all place binaries in `node_modules/.bin/`
- ✅ **Falls back to global** - If not found locally, tries global installation
- ✅ **Cross-platform** - Works on Windows, macOS, and Linux

**Example resolution order:**
1. `./node_modules/.bin/nx` (project-local installation) ← Most common
2. `~/.npm/_npx/*/node_modules/.bin/nx` (npx cache)
3. Global nx (if installed with `npm install -g nx`)

The wrapper will:
1. Start a local HTTP server implementing the Nx Remote Cache API
2. Set the `NX_CLOUD_API` environment variable to point to the local cache
3. Execute `npx nx <your-arguments>`
4. Wait for nx to complete
5. Gracefully shut down the cache server

**Why npx?** Each build system has different conventions:
- **Gradle**: Uses wrapper script `./gradlew` (checked into repo)
- **Bazel**: Global installation in PATH (`bazel`)
- **Nx**: Local node package in `node_modules/.bin/` → **npx resolves this automatically**

## See Also

- [CLI Reference](/reference/cli) - Full command-line options
- [Configuration File](/reference/config-file) - Complete configuration reference
- [Nx Remote Caching Documentation](https://nx.dev/ci/features/remote-cache) - Official Nx docs
