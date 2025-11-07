# Xcode Integration

Xcode integration guide for Fabrik. This assumes you've already [completed the getting started guide](../../README.md#-getting-started).

## How It Works

Xcode's compilation cache (Xcode 16+) uses a Unix socket for communication with the cache server. Fabrik creates a Unix socket when configured via the `[daemon] socket` setting in `fabrik.toml`.

## Configuration

Configure the Unix socket path in `fabrik.toml`:

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "20GB"

[daemon]
socket = ".fabrik/xcode.sock"  # Relative to project root
```

**Important:** When socket is configured, the daemon creates **ONLY** the Unix socket server (no HTTP/gRPC TCP servers).

## Xcode Project Setup

Set these build settings in your Xcode project. The socket path **must match** the path in `fabrik.toml`:

### Option 1: Build Settings

In your project's build settings:

```
COMPILATION_CACHE_ENABLE_CACHING = YES
COMPILATION_CACHE_ENABLE_PLUGIN = YES
COMPILATION_CACHE_REMOTE_SERVICE_PATH = $(SRCROOT)/.fabrik/xcode.sock
```

### Option 2: .xcconfig File

Create or update your `.xcconfig`:

```
// Build.xcconfig
COMPILATION_CACHE_ENABLE_CACHING = YES
COMPILATION_CACHE_ENABLE_PLUGIN = YES
COMPILATION_CACHE_REMOTE_SERVICE_PATH = $(SRCROOT)/.fabrik/xcode.sock
```

### Option 3: Command Line

Pass settings when calling `xcodebuild`:

```bash
cd ~/my-xcode-project

xcodebuild \
  -project MyApp.xcodeproj \
  -scheme MyApp \
  COMPILATION_CACHE_ENABLE_CACHING=YES \
  COMPILATION_CACHE_ENABLE_PLUGIN=YES \
  COMPILATION_CACHE_REMOTE_SERVICE_PATH=.fabrik/xcode.sock
```

## Quick Start

Once configured, the daemon starts automatically when you navigate to your project:

```bash
cd ~/my-xcode-project
# Daemon starts and creates .fabrik/xcode.sock

xcodebuild -project MyApp.xcodeproj -scheme MyApp
# Xcode connects to socket and uses cache
```

## Gitignore

Add to your `.gitignore`:

```
.fabrik/
```

This ignores both the cache directory and the socket file.
