# Xcode Integration

Xcode integration guide for Fabrik. This assumes you've already [completed the getting started guide](../../README.md#-getting-started).

## How It Works

Fabrik provides build caching for Xcode projects using Unix domain sockets for optimal performance with iOS, macOS, watchOS, and tvOS builds. When you navigate to your project, Fabrik exports `XCODE_CACHE_SERVER` which points to a Unix socket for low-latency communication.

## Quick Start

```bash
cd ~/my-xcode-project
xcodebuild -project MyApp.xcodeproj -scheme MyApp
```

The daemon automatically starts and Xcode will use the cache via the `XCODE_CACHE_SERVER` environment variable.

## Xcode Configuration

### For Command-Line Builds

No configuration needed! Xcode automatically reads `XCODE_CACHE_SERVER` from your shell environment.

### For Xcode GUI Builds

To use caching when building from Xcode.app:

1. Edit Scheme → Run → Arguments → Environment Variables
2. Add: `XCODE_CACHE_SERVER = ${XCODE_CACHE_SERVER}`

This passes the environment variable from your shell to Xcode.

## Configuration Examples

### Local Cache Only

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "20GB"  # iOS builds can be large
```

### With Remote Cache

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "10GB"

[[upstream]]
url = "http://cache.tuist.io:8080"
timeout = "60s"  # iOS artifacts can be large
```

## Xcode-Specific Tips

### Cache Directory Location

For better performance, use a local SSD:

```toml
[cache]
dir = "~/.fabrik/xcode-cache"  # Expands to home directory
max_size = "30GB"
```

### Derived Data

Xcode's Derived Data is separate from Fabrik's cache:

```bash
# Clear Xcode's Derived Data
rm -rf ~/Library/Developer/Xcode/DerivedData

# Fabrik cache remains intact
```

### Build Settings

For optimal caching, ensure consistent build settings:

```bash
# In your xcconfig or build settings
COMPILER_INDEX_STORE_ENABLE = NO  # Improves cache hit rate
ENABLE_BITCODE = NO  # For iOS (deprecated in Xcode 14+)
```

## Troubleshooting

### Cache Not Working

1. **Check environment variable:**
   ```bash
   echo $XCODE_CACHE_SERVER
   # Should show: unix:///path/to/socket or http://127.0.0.1:{port}
   ```

2. **Verify daemon is running:**
   ```bash
   fabrik doctor --verbose
   ```

3. **Check Xcode build logs:**
   ```bash
   xcodebuild -project MyApp.xcodeproj -scheme MyApp | grep cache
   ```

### Slow Builds Despite Cache

1. **Increase cache size:**
   ```toml
   [cache]
   max_size = "40GB"
   ```

2. **Clean build folder:**
   ```bash
   xcodebuild clean -project MyApp.xcodeproj -scheme MyApp
   ```

3. **Check for non-deterministic inputs:**
   - Timestamps in build phase scripts
   - Random values in code generation
   - Absolute paths in build settings

### GUI Builds Not Using Cache

Make sure you've added `XCODE_CACHE_SERVER` to your scheme's environment variables (see Xcode Configuration above).

## CI/CD Integration

### GitHub Actions

```yaml
name: Build iOS App
on: [push]

jobs:
  build:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Select Xcode
        run: sudo xcode-select -s /Applications/Xcode_15.0.app

      - name: Install Fabrik
        run: |
          curl -L https://github.com/tuist/fabrik/releases/latest/download/fabrik-aarch64-apple-darwin.tar.gz | tar xz
          sudo mv fabrik /usr/local/bin/

      - name: Build
        run: xcodebuild -project MyApp.xcodeproj -scheme MyApp -configuration Release
        env:
          FABRIK_TOKEN: ${{ secrets.FABRIK_TOKEN }}
```

## Performance Tips

1. **Use Unix sockets** (automatic when daemon runs locally):
   - Lower latency than HTTP
   - Better for large artifacts
   - Preferred for local development

2. **Larger cache for complex projects:**
   ```toml
   [cache]
   max_size = "50GB"
   eviction_policy = "lfu"
   ```

3. **Enable parallel builds:**
   ```bash
   xcodebuild -parallelizeTargets -jobs 8
   ```


