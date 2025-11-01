# Nx

Fabrik provides transparent remote task caching for Nx via the Nx Remote Cache HTTP API.

## Quick Start

### 1. Activate Fabrik (One-Time Setup)

```bash
# Add to your shell config
echo 'eval "$(fabrik activate bash)"' >> ~/.bashrc
source ~/.bashrc
```

### 2. Configure Your Project

Create `.fabrik.toml` in your project root:

```toml
[cache]
dir = ".fabrik/cache"
max_size = "10GB"

# Optional: Connect to remote cache
[[upstream]]
url = "http://cache.example.com:8080"
timeout = "30s"
```

### 3. Enable Nx Remote Caching

Add to your `nx.json`:

```json
{
  "tasksRunnerOptions": {
    "default": {
      "runner": "nx/tasks-runners/default",
      "options": {
        "cacheableOperations": ["build", "test", "lint"],
        "parallel": 3,
        "remoteCache": {
          "enabled": true
        }
      }
    }
  }
}
```

**Note:** Nx automatically reads the cache URL from the `NX_SELF_HOSTED_REMOTE_CACHE_SERVER` environment variable, so you don't need to configure the URL in `nx.json`.

### 4. Use Nx Normally

```bash
cd ~/my-nx-workspace

# Daemon starts automatically
nx build my-app
nx test my-app
nx run-many --target=build --all
```

That's it! Nx automatically uses Fabrik's cache via the `NX_SELF_HOSTED_REMOTE_CACHE_SERVER` environment variable.

## Configuration

### Shell Activation (Recommended for Development)

Shell activation automatically manages the daemon:

```bash
cd ~/nx-workspace
# Daemon starts, exports NX_SELF_HOSTED_REMOTE_CACHE_SERVER
nx build my-app

cd ~/another-workspace
# New daemon starts if different config
```

### Explicit Execution (CI/CD)

For CI/CD pipelines, use `fabrik exec`:

```bash
# In your CI script
fabrik exec nx build my-app
fabrik exec nx run-many --target=test --all
```

## How It Works

When Fabrik is activated:

1. **Daemon starts** with an HTTP server implementing Nx's Remote Cache API
2. **Environment variable exported**: `NX_SELF_HOSTED_REMOTE_CACHE_SERVER=http://127.0.0.1:{port}`
3. **Nx connects** to the HTTP server (automatically via the environment variable)
4. **Cache operations** flow through Fabrik's multi-layer cache

The daemon implements:
- **GET /health**: Health check
- **GET /v1/cache/{hash}**: Retrieve cached task outputs (tar archives)
- **PUT /v1/cache/{hash}**: Store task outputs

Nx uses numeric string hashes (e.g., `519241863493579149`) and transfers tar archives as binary data.

## Examples

### Development Workflow

```bash
# One-time setup
echo 'eval "$(fabrik activate bash)"' >> ~/.bashrc
source ~/.bashrc

# Daily usage
cd ~/my-nx-workspace
nx build my-app           # First build (cache miss)
nx build my-app           # Second build (cache hit) - instant!
nx run-many --target=test --all
```

### CI/CD Workflow

```yaml
# GitHub Actions
steps:
  - uses: actions/checkout@v4
  
  - uses: actions/setup-node@v4
    with:
      node-version: '20'
  
  - run: npm ci
  
  - run: |
      curl -fsSL https://raw.githubusercontent.com/tuist/fabrik/main/install.sh | sh
      
  - run: fabrik exec nx run-many --target=build --all
  
  - run: fabrik exec nx run-many --target=test --all
```

### Monorepo Workflow

```bash
cd ~/my-monorepo

# Build affected projects only
nx affected --target=build

# Test everything
nx run-many --target=test --all

# Lint specific app
nx lint my-app

# All tasks use the same cache
```

## Advanced Configuration

### Customize Cacheable Operations

```json
{
  "tasksRunnerOptions": {
    "default": {
      "runner": "nx/tasks-runners/default",
      "options": {
        "cacheableOperations": [
          "build",
          "test",
          "lint",
          "e2e",
          "bundle"
        ],
        "parallel": 3,
        "remoteCache": {
          "enabled": true
        }
      }
    }
  }
}
```

### Per-Project Cache Configuration

```json
{
  "name": "my-app",
  "targets": {
    "build": {
      "cache": true,
      "outputs": ["{projectRoot}/dist"]
    },
    "test": {
      "cache": true
    }
  }
}
```

## Troubleshooting

### Nx Not Using Remote Cache

Check that environment variable is set:

```bash
echo $NX_SELF_HOSTED_REMOTE_CACHE_SERVER
# Should output: http://127.0.0.1:{port}
```

Check daemon is running:

```bash
fabrik daemon list
```

Enable Nx verbose logging:

```bash
NX_VERBOSE_LOGGING=true nx build my-app
```

### Cache Misses

Check Nx cache:

```bash
nx reset  # Clear local cache
nx build my-app --verbose
```

Verify task is cacheable in `project.json`:

```json
{
  "targets": {
    "build": {
      "cache": true
    }
  }
}
```

### Connection Issues

Restart the daemon:

```bash
fabrik daemon stop
fabrik activate --status
```

Check Nx configuration:

```bash
nx report
```

## Performance Tips

1. **Enable parallel execution**: Set `"parallel": 3` in `nx.json`
2. **Cache all tasks**: Add operations to `cacheableOperations`
3. **Use affected commands**: `nx affected --target=build` only builds changed projects
4. **Optimize outputs**: Specify exact output paths in `project.json`

## See Also

- [Getting Started](/getting-started) - Complete setup guide
- [CLI Reference](/reference/cli) - Command-line options
- [Configuration](/reference/config-file) - Configuration reference
- [Nx Remote Caching Docs](https://nx.dev/ci/features/remote-cache) - Official Nx documentation
