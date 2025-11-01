# Xcode

Fabrik provides transparent build caching for Xcode projects using Unix domain sockets for optimal performance with iOS, macOS, watchOS, and tvOS builds.

## Quick Start

### 1. Activate Fabrik (One-Time Setup)

```bash
# Add to your shell config
echo 'eval "$(fabrik activate bash)"' >> ~/.bashrc
source ~/.bashrc
```

### 2. Configure Your Project

Create `.fabrik.toml` in your project root:

```toml
[cache]
dir = ".fabrik/cache"
max_size = "20GB"

# Optional: Connect to remote cache
[[upstream]]
url = "http://cache.example.com:8080"
timeout = "30s"
```

### 3. Configure Xcode Build Settings

Xcode reads the cache server from the `XCODE_CACHE_SERVER` environment variable. When using Fabrik activation, this is automatically set to a Unix socket path for best performance.

Add to your scheme's environment variables (optional, for Xcode GUI builds):

1. Edit Scheme → Run → Arguments → Environment Variables
2. Add: `XCODE_CACHE_SERVER = ${XCODE_CACHE_SERVER}`

### 4. Build Normally

```bash
cd ~/my-xcode-project

# Daemon starts automatically with Unix socket
xcodebuild -project MyApp.xcodeproj -scheme MyApp

# Or use fabrik exec for CI
fabrik exec xcodebuild -workspace MyApp.xcworkspace -scheme MyApp -configuration Release
```

## Unix Socket vs HTTP

**Xcode uses Unix domain sockets by default for better performance:**

- **Unix Socket** (preferred): `XCODE_CACHE_SERVER=/path/to/socket`
- **HTTP Fallback**: `XCODE_CACHE_SERVER=http://127.0.0.1:{port}`

Fabrik automatically creates a Unix socket when activated, providing:
- ✅ Lower latency (no TCP overhead)
- ✅ Higher throughput
- ✅ Better security (filesystem permissions)

The `XCODE_CACHE_SERVER` environment variable is automatically set to the Unix socket path.

## Configuration

### Shell Activation (Recommended for Development)

```bash
cd ~/xcode-project
# Daemon starts with Unix socket
xcodebuild -project MyApp.xcodeproj -scheme MyApp

cd ~/another-project
# New daemon starts if different config
```

### Explicit Execution (CI/CD)

For CI/CD pipelines, use `fabrik exec`:

```bash
# In your CI script
fabrik exec xcodebuild -workspace MyApp.xcworkspace \
  -scheme MyApp \
  -configuration Release \
  -destination 'platform=iOS Simulator,name=iPhone 15'
```

## How It Works

When Fabrik is activated:

1. **Daemon starts** with a Unix domain socket for Xcode
2. **Environment variable exported**: `XCODE_CACHE_SERVER=/path/to/socket`
3. **Xcode connects** to the socket (via environment variable or build settings)
4. **Build artifacts** cached through Fabrik's multi-layer cache

## Examples

### Development Workflow

```bash
# One-time setup
echo 'eval "$(fabrik activate bash)"' >> ~/.bashrc
source ~/.bashrc

# Daily usage
cd ~/MyiOSApp
xcodebuild -project MyApp.xcodeproj -scheme MyApp         # First build
xcodebuild clean
xcodebuild -project MyApp.xcodeproj -scheme MyApp         # Second build (cached) - much faster!
```

### CI/CD Workflow

```yaml
# GitHub Actions
steps:
  - uses: actions/checkout@v4
  
  - uses: maxim-lobanov/setup-xcode@v1
    with:
      xcode-version: '15.0'
  
  - run: |
      curl -fsSL https://raw.githubusercontent.com/tuist/fabrik/main/install.sh | sh
      
  - run: |
      fabrik exec xcodebuild \
        -workspace MyApp.xcworkspace \
        -scheme MyApp \
        -destination 'platform=iOS Simulator,name=iPhone 15' \
        clean build
```

### Testing with Fabrik

```bash
# Run unit tests
fabrik exec xcodebuild test \
  -workspace MyApp.xcworkspace \
  -scheme MyAppTests \
  -destination 'platform=iOS Simulator,name=iPhone 15'

# Run UI tests
fabrik exec xcodebuild test \
  -workspace MyApp.xcworkspace \
  -scheme MyAppUITests \
  -destination 'platform=iOS Simulator,name=iPhone 15'
```

## Xcode GUI Integration

To use Fabrik with builds triggered from Xcode's GUI:

1. **Edit your scheme**: Product → Scheme → Edit Scheme
2. **Add environment variable**: Run → Arguments → Environment Variables
3. **Add**: `XCODE_CACHE_SERVER` = `${XCODE_CACHE_SERVER}`

Now when you build from Xcode GUI (⌘B), it will use the Fabrik cache if the daemon is running.

::: tip
Make sure to activate Fabrik in your terminal before opening Xcode:
```bash
cd ~/my-project
fabrik activate --status  # Start daemon
open MyApp.xcworkspace     # Open Xcode
```
:::

## Troubleshooting

### Xcode Not Using Cache

Check that environment variable is set:

```bash
echo $XCODE_CACHE_SERVER
# Should output: /path/to/fabrik/socket or http://127.0.0.1:{port}
```

Check daemon is running:

```bash
fabrik daemon list
```

Verify socket exists:

```bash
ls -la $XCODE_CACHE_SERVER
```

### Socket Permission Issues

If you get permission denied errors:

```bash
# Restart the daemon
fabrik daemon stop
fabrik activate --status
```

### Connection Issues

Check if socket is accessible:

```bash
# Test socket connection
nc -U $XCODE_CACHE_SERVER
```

Restart daemon:

```bash
fabrik daemon stop
fabrik daemon start
```

### Build from GUI Not Caching

Ensure the environment variable is set in your scheme:

1. Product → Scheme → Edit Scheme
2. Run → Arguments → Environment Variables
3. Verify `XCODE_CACHE_SERVER` is present

## Performance Tips

1. **Use Unix socket** instead of HTTP for ~30% better performance
2. **Cache derived data**: Large max_size in .fabrik.toml (20GB+)
3. **Clean builds occasionally**: Test cache effectiveness
4. **Monitor cache hits**: Check daemon logs

## See Also

- [Getting Started](/getting-started) - Complete setup guide
- [CLI Reference](/reference/cli) - Command-line options
- [Configuration](/reference/config-file) - Configuration reference
