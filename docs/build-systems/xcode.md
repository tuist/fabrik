# Xcode Integration

Xcode integration guide for Fabrik. This assumes you've already [completed the getting started guide](../../README.md#-getting-started).

## How It Works

Xcode's compilation cache (available in Xcode 16+) requires a Unix socket path set via the `COMPILATION_CACHE_REMOTE_SERVICE_PATH` build setting.

> **Note:** Unix socket support is planned but not yet implemented in Fabrik. For now, Xcode integration requires using HTTP with the socket path workaround below, or using a remote Fabrik server with a fixed address.

## Quick Start

### Using xcodebuild with Build Setting

When the daemon starts, Fabrik will export `XCODE_CACHE_SERVER`. Pass it to xcodebuild:

```bash
cd ~/my-xcode-project

xcodebuild \
  -project MyApp.xcodeproj \
  -scheme MyApp \
  COMPILATION_CACHE_ENABLE_CACHING=YES \
  COMPILATION_CACHE_ENABLE_PLUGIN=YES \
  COMPILATION_CACHE_REMOTE_SERVICE_PATH="$XCODE_CACHE_SERVER"
```

### Using Xcode GUI

To use caching when building from Xcode.app:

1. Edit Scheme → Run → Arguments → Environment Variables
2. Add: `XCODE_CACHE_SERVER = ${XCODE_CACHE_SERVER}`
3. Edit your project's build settings and set:
   - `COMPILATION_CACHE_ENABLE_CACHING = YES`
   - `COMPILATION_CACHE_ENABLE_PLUGIN = YES`
   - `COMPILATION_CACHE_REMOTE_SERVICE_PATH = $(XCODE_CACHE_SERVER)`

> **Limitation:** Currently, `XCODE_CACHE_SERVER` points to an HTTP URL, not a Unix socket. Unix socket support (for better performance) is coming soon.
