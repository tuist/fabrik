# Metro

Metro is the JavaScript bundler used by React Native and other JavaScript projects. Fabrik integrates with Metro's cache system via the `@tuist/fabrik` NPM package.

## Installation

```bash
npm install @tuist/fabrik
# or
pnpm add @tuist/fabrik
# or
yarn add @tuist/fabrik
```

The Fabrik binary will be automatically downloaded during installation based on your platform.

## Configuration

Add Fabrik to your Metro configuration:

```javascript
// metro.config.js
const { FabrikStore } = require('@tuist/fabrik/metro');

module.exports = {
  cacheStores: [
    new FabrikStore({
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

## Shared Cache

Metro's cache is **shared** with other build tools using Fabrik:
- Bazel (via gRPC)
- Gradle (via HTTP)
- Nx, TurboRepo (via HTTP)

All build tools share the same RocksDB storage, maximizing cache efficiency.

## API

The Fabrik daemon exposes an HTTP cache API that Metro uses:

- `GET /api/v1/artifacts/{hash}` - Retrieve cached artifact
- `PUT /api/v1/artifacts/{hash}` - Store artifact
- `GET /health` - Health check
