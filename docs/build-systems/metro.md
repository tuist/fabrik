# Metro

Metro is the JavaScript bundler used by React Native and other JavaScript projects. Fabrik integrates with Metro's cache system via the `@fabrik/metro` NPM package.

## Installation

```bash
npm install @fabrik/metro
# or
pnpm add @fabrik/metro
# or
yarn add @fabrik/metro
```

The Fabrik binary will be automatically downloaded during installation based on your platform.

## Configuration

Add Fabrik to your Metro configuration:

```javascript
// metro.config.js
const { FabrikStore } = require('@fabrik/metro');

module.exports = {
  cacheStores: [
    FabrikStore({
      // Optional: Local cache directory
      cacheDir: '.fabrik/cache',

      // Optional: Upstream Fabrik server
      upstream: 'grpc://cache.tuist.io:7070',

      // Optional: Maximum cache size
      maxSize: '5GB',

      // Optional: Authentication token
      token: process.env.TUIST_TOKEN,
    }),
  ],
};
```

## How It Works

The `FabrikStore` integrates Metro with Fabrik's multi-layer caching:

1. **Automatic Binary Management** - Downloads the correct Fabrik binary for your platform during `npm install`
2. **Daemon Lifecycle** - Automatically starts the Fabrik daemon when Metro builds
3. **Transparent Caching** - Metro cache operations flow through Fabrik's HTTP API
4. **Multi-Layer Fallback** - Local cache → Regional cache → S3 (configured via `upstream`)

## Environment Variables

You can configure Fabrik using environment variables:

```bash
export FABRIK_CACHE_DIR=".fabrik/cache"
export FABRIK_UPSTREAM="grpc://cache.tuist.io:7070"
export FABRIK_MAX_SIZE="5GB"
export TUIST_TOKEN="your-token"
export FABRIK_PORT="7070"
export FABRIK_LOG_LEVEL="info"
```

## Development Mode

When developing Fabrik itself, the package automatically detects if it's running in the Fabrik repository and uses `cargo run` instead of the downloaded binary.

This enables rapid iteration:
1. Make changes to Fabrik's Rust code
2. Test Metro integration immediately
3. See daemon logs for debugging

## Manual Installation

If automatic binary download fails, you can install Fabrik manually:

```bash
# Using mise
mise use -g ubi:tuist/fabrik

# Or download from releases
# https://github.com/tuist/fabrik/releases
```

Then disable auto-start:

```javascript
FabrikStore({
  autoStart: false,  // Don't start daemon, use existing one
  port: 7070,
})
```

## API

The Fabrik daemon exposes an HTTP cache API that Metro uses:

- `GET /api/v1/artifacts/{hash}` - Retrieve cached artifact
- `PUT /api/v1/artifacts/{hash}` - Store artifact
- `GET /health` - Health check

## Shared Cache

Metro's cache is **shared** with other build tools using Fabrik:
- Bazel (via gRPC)
- Gradle (via HTTP)
- Nx, TurboRepo (via HTTP)

All build tools share the same RocksDB storage, maximizing cache efficiency.

## Example: React Native Project

```javascript
// metro.config.js
const { FabrikStore } = require('@fabrik/metro');
const { getDefaultConfig } = require('@react-native/metro-config');

module.exports = (async () => {
  const config = await getDefaultConfig(__dirname);

  return {
    ...config,
    cacheStores: [
      FabrikStore({
        cacheDir: '.fabrik/cache',
        maxSize: '2GB',
      }),
    ],
  };
})();
```

## Troubleshooting

### Binary not found

If you see "Fabrik binary not found", try:

```bash
npm install @fabrik/metro --force
```

Or install Fabrik manually with mise.

### Daemon fails to start

Check the logs:

```bash
export FABRIK_LOG_LEVEL=debug
```

Then run your Metro build to see detailed daemon logs.

### Cache not working

Verify the daemon is running:

```bash
curl http://localhost:7070/health
```

Should return `OK`.
