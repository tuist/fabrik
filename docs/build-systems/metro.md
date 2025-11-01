# Metro

Metro is the JavaScript bundler used by React Native and other JavaScript projects. Fabrik integrates with Metro's cache system through two approaches:

1. **Shell Activation** (Recommended) - Use Fabrik's activation system
2. **NPM Package** - Use `@tuist/fabrik` for programmatic integration

## Approach 1: Shell Activation (Recommended)

### Quick Start

#### 1. Activate Fabrik

```bash
# Add to your shell config
echo 'eval "$(fabrik activate bash)"' >> ~/.bashrc
source ~/.bashrc
```

#### 2. Configure Your Project

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

#### 3. Configure Metro

Update your `metro.config.js` to read from environment variables:

```javascript
// metro.config.js
const { FabrikStore } = require('@tuist/fabrik/metro');

module.exports = {
  cacheStores: [
    new FabrikStore({
      // Connect to activated daemon via environment variable
      url: process.env.FABRIK_HTTP_URL,
    }),
  ],
};
```

#### 4. Run Metro Normally

```bash
cd ~/my-react-native-app

# Daemon starts automatically
npm start
npx react-native start
```

The FabrikStore will automatically connect to the activated daemon via `FABRIK_HTTP_URL`.

## Approach 2: NPM Package Integration

For projects that prefer programmatic configuration without shell activation.

### Installation

```bash
npm install @tuist/fabrik
# or
pnpm add @tuist/fabrik
# or
yarn add @tuist/fabrik
```

The Fabrik binary will be automatically downloaded during installation based on your platform.

### Configuration

Add Fabrik to your Metro configuration with explicit settings:

```javascript
// metro.config.js
const { FabrikStore } = require('@tuist/fabrik/metro');

module.exports = {
  cacheStores: [
    new FabrikStore({
      // Local cache directory
      cacheDir: '.fabrik/cache',

      // Optional: Upstream Fabrik server
      upstream: 'http://cache.example.com:8080',

      // Optional: Maximum cache size
      maxSize: '5GB',

      // Optional: Authentication token
      token: process.env.TUIST_TOKEN,
    }),
  ],
};
```

The `FabrikStore` will start its own daemon if no activated daemon is detected.

## How It Works

### With Shell Activation

1. **Daemon starts** when you enter the project directory
2. **Environment variable exported**: `FABRIK_HTTP_URL=http://127.0.0.1:{port}`
3. **FabrikStore connects** to the existing daemon via the URL
4. **Metro cache operations** flow through Fabrik's multi-layer cache

### With NPM Package

1. **Automatic Binary Management** - Downloads the correct Fabrik binary for your platform during `npm install`
2. **Daemon Lifecycle** - Automatically starts the Fabrik daemon when Metro builds
3. **Transparent Caching** - Metro cache operations flow through Fabrik's HTTP API
4. **Multi-Layer Fallback** - Local cache → Regional cache → S3 (configured via `upstream`)

## API

The Fabrik daemon exposes an HTTP cache API that Metro uses:

- `GET /api/v1/artifacts/{hash}` - Retrieve cached artifact
- `PUT /api/v1/artifacts/{hash}` - Store artifact  
- `GET /health` - Health check
