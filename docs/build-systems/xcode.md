# Xcode Integration

Xcode integration guide for Fabrik. This assumes you've already [completed the getting started guide](../../README.md#-getting-started).

## How It Works

Fabrik provides build caching for Xcode projects. When you navigate to your project, Fabrik exports `XCODE_CACHE_SERVER` which Xcode can use for caching.

## Quick Start

### For Command-Line Builds

```bash
cd ~/my-xcode-project
xcodebuild -project MyApp.xcodeproj -scheme MyApp
```

The daemon automatically starts and Xcode will use the cache via the `XCODE_CACHE_SERVER` environment variable.

### For Xcode GUI Builds

To use caching when building from Xcode.app:

1. Edit Scheme → Run → Arguments → Environment Variables
2. Add: `XCODE_CACHE_SERVER = ${XCODE_CACHE_SERVER}`

This passes the environment variable from your shell to Xcode.
