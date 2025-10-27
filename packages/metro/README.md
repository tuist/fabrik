# @tuist/fabrik

Metro cache store for Fabrik multi-layer build cache.

## Installation

```bash
npm install @tuist/fabrik
# or
pnpm add @tuist/fabrik
# or
yarn add @tuist/fabrik
```

The Fabrik binary will be automatically downloaded during installation based on your platform.

## Usage

Configure Metro to use Fabrik as a cache store:

```javascript
// metro.config.js
const { FabrikStore } = require('@tuist/fabrik');

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

      // Optional: Automatically start daemon (default: true)
      autoStart: true,

      // Optional: Daemon port (default: 7070)
      port: 7070,

      // Optional: Log level (default: 'info')
      logLevel: 'info',
    }),
  ],
};
```

## How It Works

The `FabrikStore` integrates Metro with Fabrik's multi-layer caching:

1. **Automatic Binary Management** - Downloads the correct Fabrik binary for your platform during `npm install`
2. **Daemon Lifecycle** - Automatically starts the Fabrik daemon when Metro builds
3. **Transparent Caching** - Metro cache operations flow through Fabrik's protocol
4. **Multi-Layer Fallback** - Local cache → Regional cache → S3 (configured via `upstream`)

## Environment Variables

You can configure Fabrik using environment variables:

- `FABRIK_CACHE_DIR` - Local cache directory
- `FABRIK_UPSTREAM` - Upstream server URL
- `FABRIK_MAX_SIZE` - Maximum cache size
- `TUIST_TOKEN` - Authentication token
- `FABRIK_PORT` - Daemon port
- `FABRIK_LOG_LEVEL` - Log level (debug, info, warn, error)

## Manual Installation

If the automatic binary download fails, you can install Fabrik manually:

```bash
# Using mise
mise use -g ubi:tuist/fabrik

# Or download from releases
# https://github.com/tuist/fabrik/releases
```

Then disable auto-start and point to your Fabrik installation:

```javascript
new FabrikStore({
  autoStart: false,  // Don't start daemon, use existing one
  port: 7070,
});
```

## Development Mode

When developing Fabrik itself, the package automatically detects if it's running in the Fabrik repository and uses `cargo run` instead of the downloaded binary.

**Detection:** Checks for `Cargo.toml` in parent directories

**Behavior in dev mode:**
- Uses `cargo run -- daemon` instead of binary
- Shows daemon output (`stdio: 'inherit'`)
- No binary download required
- Compiles Rust code on-the-fly

This allows you to:
1. Make changes to Fabrik's Rust code
2. Test Metro integration immediately
3. See daemon logs for debugging

## Testing

Run the test suite:

```bash
pnpm test
```

Tests use Node's built-in test runner and dependency injection for mocking.

## License

MPL-2.0
