# Xcode Integration

Xcode integration guide for Fabrik. This assumes you've already [completed the getting started guide](../../README.md#-getting-started).

## How It Works

Xcode's compilation cache (Xcode 16+) requires a Unix socket path set via the `COMPILATION_CACHE_REMOTE_SERVICE_PATH` build setting. Fabrik creates a Unix socket when configured, which Xcode connects to for caching.

## Configuration

Add socket configuration to your `fabrik.toml`:

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "20GB"

[daemon]
socket = ".fabrik/xcode.sock"  # Relative path, will be gitignored
```

## Quick Start

### For Command-Line Builds

```bash
cd ~/my-xcode-project

xcodebuild \
  -project MyApp.xcodeproj \
  -scheme MyApp \
  COMPILATION_CACHE_ENABLE_CACHING=YES \
  COMPILATION_CACHE_ENABLE_PLUGIN=YES \
  COMPILATION_CACHE_REMOTE_SERVICE_PATH=.fabrik/xcode.sock
```

### For Xcode GUI Builds

Set these build settings in your project or scheme:

```
COMPILATION_CACHE_ENABLE_CACHING = YES
COMPILATION_CACHE_ENABLE_PLUGIN = YES
COMPILATION_CACHE_REMOTE_SERVICE_PATH = $(SRCROOT)/.fabrik/xcode.sock
```

Or add to your `.xcconfig`:

```
// Build.xcconfig
COMPILATION_CACHE_ENABLE_CACHING = YES
COMPILATION_CACHE_ENABLE_PLUGIN = YES
COMPILATION_CACHE_REMOTE_SERVICE_PATH = $(SRCROOT)/.fabrik/xcode.sock
```

## Important Notes

1. **Socket path must match**: The path in `fabrik.toml` and Xcode build settings must be identical
2. **Relative paths work**: Paths are resolved relative to the project root (where `fabrik.toml` is located)
3. **Gitignore the socket**: Add `.fabrik/` to your `.gitignore`
4. **Daemon creates only socket**: When socket is configured, daemon creates ONLY the Unix socket (no TCP servers)
