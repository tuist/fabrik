# Metro Integration

Metro integration guide for Fabrik. This assumes you've already [completed the getting started guide](../../README.md#-getting-started).

## How It Works

Fabrik provides HTTP-based caching for Metro bundler (React Native). When you navigate to your project, Fabrik exports `FABRIK_HTTP_URL` which Metro can use for remote caching.

## Quick Start

Configure Metro to use Fabrik's cache by updating your `metro.config.js`:

```javascript
const {getDefaultConfig} = require('metro-config');

module.exports = (async () => {
  const config = await getDefaultConfig();
  
  return {
    ...config,
    cacheStores: [
      // Local cache (default)
      require('metro-cache/src/stores/FileStore'),
      
      // Remote cache via Fabrik
      {
        get: async (key) => {
          const url = `${process.env.FABRIK_HTTP_URL}/api/v1/artifacts/${key}`;
          const response = await fetch(url);
          return response.ok ? await response.buffer() : null;
        },
        set: async (key, value) => {
          const url = `${process.env.FABRIK_HTTP_URL}/api/v1/artifacts/${key}`;
          await fetch(url, {
            method: 'PUT',
            body: value,
          });
        },
      },
    ],
  };
})();
```

Then start Metro:

```bash
cd ~/my-react-native-app
npm start
```

## Configuration Examples

### Local Cache Only

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "5GB"
```

### With Remote Cache

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "3GB"

[[upstream]]
url = "grpc://cache.tuist.io:7070"
timeout = "30s"
```

## Metro Cache Configuration

### Full Metro Config Example

```javascript
// metro.config.js
const {getDefaultConfig} = require('metro-config');

const FABRIK_URL = process.env.FABRIK_HTTP_URL;

module.exports = (async () => {
  const config = await getDefaultConfig();
  
  const cacheStores = [
    require('metro-cache/src/stores/FileStore'),
  ];
  
  // Add Fabrik remote cache if available
  if (FABRIK_URL) {
    cacheStores.push({
      get: async (key) => {
        try {
          const response = await fetch(`${FABRIK_URL}/api/v1/artifacts/${key}`);
          if (!response.ok) return null;
          return await response.buffer();
        } catch (error) {
          console.warn('Fabrik cache miss:', error.message);
          return null;
        }
      },
      set: async (key, value) => {
        try {
          await fetch(`${FABRIK_URL}/api/v1/artifacts/${key}`, {
            method: 'PUT',
            body: value,
            headers: {
              'Content-Type': 'application/octet-stream',
            },
          });
        } catch (error) {
          console.warn('Fabrik cache set failed:', error.message);
        }
      },
    });
  }
  
  return {
    ...config,
    cacheStores,
  };
})();
```

## Troubleshooting

### Cache Not Working

1. **Check environment variable:**
   ```bash
   echo $FABRIK_HTTP_URL
   # Should show: http://127.0.0.1:{port}
   ```

2. **Verify daemon is running:**
   ```bash
   fabrik doctor --verbose
   ```

3. **Check Metro cache hits:**
   ```bash
   # Clear Metro cache
   npm start -- --reset-cache
   
   # Run again and check logs
   npm start
   ```

### Metro Not Using Remote Cache

1. **Verify Metro config:**
   ```javascript
   console.log('Fabrik URL:', process.env.FABRIK_HTTP_URL);
   ```

2. **Check Metro bundler logs:**
   ```bash
   npm start -- --verbose
   ```

3. **Test cache endpoint manually:**
   ```bash
   curl -I $FABRIK_HTTP_URL/api/v1/artifacts/test
   ```

## CI/CD Integration

### GitHub Actions

```yaml
name: Build React Native App
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

      - name: Bundle
        run: npm run bundle
        env:
          FABRIK_TOKEN: ${{ secrets.FABRIK_TOKEN }}
```

## Performance Tips

1. **Increase cache size for large apps:**
   ```toml
   [cache]
   max_size = "10GB"
   ```

2. **Use watchman for faster rebuilds:**
   ```bash
   brew install watchman  # macOS
   ```

3. **Enable Metro's transformer cache:**
   ```javascript
   // metro.config.js
   module.exports = {
     transformer: {
       enableBabelRCLookup: false,
       enableBabelRuntime: false,
     },
   };
   ```


## Other Build Systems

Looking for a different build system?

- **[🏗️ Gradle](./gradle.md)** - Java, Kotlin, Android projects
- **[📦 Bazel](./bazel.md)** - Multi-language monorepos
- **[📱 Xcode](./xcode.md)** - iOS, macOS, watchOS, tvOS apps  
- **[⚡ Nx](./nx.md)** - JavaScript/TypeScript monorepos
- **[📲 Metro](./metro.md)** - React Native bundler

[View all build systems →](./README.md)

## See Also

- [Metro Bundler Documentation](https://metrobundler.dev/)
- [React Native Caching](https://reactnative.dev/docs/performance#metro-bundler)
- [CLI Reference](../cli-reference.md)
- [Getting Started](../../README.md)
