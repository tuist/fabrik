# Nx Integration

Nx integration guide for Fabrik. This assumes you've already [completed the getting started guide](../../README.md#-getting-started).

## How It Works

Fabrik provides remote caching for Nx via HTTP. When you navigate to your project, Fabrik exports `NX_SELF_HOSTED_REMOTE_CACHE_SERVER` which Nx automatically reads.

## Quick Start

```bash
cd ~/my-nx-workspace
nx build my-app
```

That's it! Nx will automatically use Fabrik's cache.

## Configuration Examples

### Local Cache Only

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "10GB"
```

### With Remote Cache

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "5GB"

[[upstream]]
url = "grpc://cache.tuist.io:7070"
timeout = "30s"
```

## Nx Configuration

Nx reads `NX_SELF_HOSTED_REMOTE_CACHE_SERVER` automatically. No `nx.json` changes needed!

However, you can verify the configuration in `nx.json`:

```json
{
  "tasksRunnerOptions": {
    "default": {
      "runner": "nx/tasks-runners/default",
      "options": {
        "cacheableOperations": ["build", "test", "lint"]
      }
    }
  }
}
```

## Troubleshooting

### Cache Not Working

1. **Check environment variable:**
   ```bash
   echo $NX_SELF_HOSTED_REMOTE_CACHE_SERVER
   # Should show: http://127.0.0.1:{port}
   ```

2. **Verify daemon is running:**
   ```bash
   fabrik doctor --verbose
   ```

3. **Check Nx cache status:**
   ```bash
   nx reset
   nx build my-app --verbose
   ```

### Slow Builds Despite Cache

1. **Check cache hit rate:**
   ```bash
   nx build my-app --verbose | grep cache
   ```

2. **Increase cache size:**
   ```toml
   [cache]
   max_size = "20GB"
   ```

3. **Review cacheable operations:**
   ```json
   {
     "tasksRunnerOptions": {
       "default": {
         "options": {
           "cacheableOperations": ["build", "test", "lint", "e2e"]
         }
       }
     }
   }
   ```

## CI/CD Integration

### GitHub Actions

```yaml
name: Build
on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Install Fabrik
        run: |
          curl -L https://github.com/tuist/fabrik/releases/latest/download/fabrik-x86_64-unknown-linux-gnu.tar.gz | tar xz
          sudo mv fabrik /usr/local/bin/

      - name: Build
        run: nx build my-app
        env:
          FABRIK_TOKEN: ${{ secrets.FABRIK_TOKEN }}
```

## Performance Tips

1. **Enable parallel execution:**
   ```bash
   nx run-many --target=build --all --parallel=3
   ```

2. **Use Nx Cloud + Fabrik:**
   - Nx Cloud for distributed task execution
   - Fabrik for artifact caching

3. **Configure cache directory:**
   ```toml
   [cache]
   dir = ".fabrik/cache"
   max_size = "15GB"
   eviction_policy = "lfu"
   ```


