# Bazel Integration

Bazel integration guide for Fabrik. This assumes you've already [completed the getting started guide](../../README.md#-getting-started).

## How It Works

Fabrik provides transparent remote caching for Bazel via the Bazel Remote Caching protocol (gRPC). When you navigate to your project, Fabrik exports `FABRIK_GRPC_URL` which you can use with Bazel's `--remote_cache` flag.

## Quick Start

### Option 1: Shell Integration (Development)

Once you have shell integration set up, Bazel can use the `FABRIK_GRPC_URL` environment variable:

```bash
cd ~/my-bazel-project
# Daemon starts automatically, exports FABRIK_GRPC_URL

bazel build --remote_cache="$FABRIK_GRPC_URL" //...
```

### Option 2: fabrik exec (CI/Development)

Use `fabrik exec` to automatically manage the daemon lifecycle:

```bash
cd ~/my-bazel-project
fabrik exec -- bazel build --remote_cache="$FABRIK_GRPC_URL" //...
```

The `fabrik exec` command:
1. Starts the daemon with your project's config
2. Exports `FABRIK_GRPC_URL`
3. Runs your command
4. Keeps daemon alive for subsequent commands

## Configuration

### .bazelrc Setup

To avoid typing `--remote_cache` every time, add to your `.bazelrc`:

```bash
# .bazelrc
build --remote_cache=grpc://localhost:9090
build --remote_upload_local_results=true
build --remote_timeout=60s
```

However, since Fabrik uses dynamic ports, you'll need to either:

1. **Use the environment variable in commands:**
   ```bash
   bazel build --remote_cache="$FABRIK_GRPC_URL" //...
   ```

2. **Use a shell alias:**
   ```bash
   alias bazel='command bazel --remote_cache="$FABRIK_GRPC_URL"'
   ```

3. **Use fabrik exec:**
   ```bash
   fabrik exec -- bazel build //...
   # Automatically uses FABRIK_GRPC_URL
   ```

### Fabrik Configuration Examples

#### Local Cache Only

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "20GB"  # Bazel can generate lots of artifacts
```

#### With Remote Cache

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "10GB"

[[upstream]]
url = "grpc://cache.tuist.io:7070"
timeout = "30s"

[auth]
token_file = ".fabrik.token"
```

## Advanced Bazel Configuration

### Remote Execution Settings

```bash
# .bazelrc
build --remote_upload_local_results=true
build --remote_timeout=60s
build --remote_retries=3
build --experimental_remote_cache_compression

# Optional: Remote execution (if server supports it)
# build --remote_executor=grpc://localhost:9090
```

### Disk Cache + Remote Cache

```bash
# .bazelrc
# Use both disk cache and remote cache
build --disk_cache=~/.cache/bazel
# Remote cache passed via command line
```

### Platform-Specific Settings

```bash
# .bazelrc
build:linux --remote_instance_name=linux
build:macos --remote_instance_name=macos
build:windows --remote_instance_name=windows
```

## Usage Examples

### Development

```bash
# With shell integration
cd ~/my-bazel-project
# Daemon starts automatically

# Option 1: Pass remote_cache explicitly
bazel build --remote_cache="$FABRIK_GRPC_URL" //src/main:app

# Option 2: Use alias
alias bzl='bazel --remote_cache="$FABRIK_GRPC_URL"'
bzl build //...
bzl test //...
```

### CI/CD

```bash
# Use fabrik exec for managed daemon lifecycle
fabrik exec -- bazel build --remote_cache="$FABRIK_GRPC_URL" //...
fabrik exec -- bazel test --remote_cache="$FABRIK_GRPC_URL" //...
```

## Verification

Check that caching is working:

```bash
# First build (cache miss)
bazel clean
bazel build --remote_cache="$FABRIK_GRPC_URL" //...

# Second build (cache hit - should be much faster)
bazel clean
bazel build --remote_cache="$FABRIK_GRPC_URL" //...
```

You should see "remote cache hit" messages in the output.

## Troubleshooting

### Cache Not Working

1. **Check environment variable:**
   ```bash
   echo $FABRIK_GRPC_URL
   # Should show: grpc://127.0.0.1:{port}
   ```

2. **Verify daemon is running:**
   ```bash
   fabrik doctor --verbose
   ```

3. **Test with verbose output:**
   ```bash
   bazel build --remote_cache="$FABRIK_GRPC_URL" //... --execution_log_json_file=exec.json
   cat exec.json | jq '.[] | select(.remoteCacheHit != null)'
   ```

### Connection Errors

If you see "failed to connect to remote cache":

1. **Verify GRPC_URL format:**
   ```bash
   echo $FABRIK_GRPC_URL
   # Should be: grpc://127.0.0.1:{port} (not http://)
   ```

2. **Check daemon logs:**
   ```bash
   fabrik doctor --verbose
   ```

3. **Test gRPC connection:**
   ```bash
   # Install grpcurl
   grpcurl -plaintext ${FABRIK_GRPC_URL#grpc://} list
   # Should list Bazel services
   ```

### Slow Builds Despite Cache

1. **Enable execution log:**
   ```bash
   bazel build --remote_cache="$FABRIK_GRPC_URL" //... --execution_log_json_file=exec.json
   cat exec.json | jq '.[] | .remoteCacheHit' | sort | uniq -c
   ```

2. **Increase cache size:**
   ```toml
   [cache]
   max_size = "50GB"
   ```

3. **Check for non-deterministic builds:**
   - Timestamps in genrules
   - Absolute paths
   - Random values

## CI/CD Integration

### GitHub Actions

```yaml
name: Build
on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Bazel
        run: |
          wget https://github.com/bazelbuild/bazelisk/releases/latest/download/bazelisk-linux-amd64
          chmod +x bazelisk-linux-amd64
          sudo mv bazelisk-linux-amd64 /usr/local/bin/bazel

      - name: Install Fabrik
        run: |
          curl -L https://github.com/tuist/fabrik/releases/latest/download/fabrik-x86_64-unknown-linux-gnu.tar.gz | tar xz
          sudo mv fabrik /usr/local/bin/

      - name: Build
        run: fabrik exec -- bazel build --remote_cache="$FABRIK_GRPC_URL" //...
        env:
          FABRIK_TOKEN: ${{ secrets.FABRIK_TOKEN }}

      - name: Test
        run: fabrik exec -- bazel test --remote_cache="$FABRIK_GRPC_URL" //...
        env:
          FABRIK_TOKEN: ${{ secrets.FABRIK_TOKEN }}
```

## Performance Tips

1. **Larger cache for monorepos:**
   ```toml
   [cache]
   max_size = "50GB"
   eviction_policy = "lfu"
   ```

2. **Tune remote cache settings:**
   ```bash
   # .bazelrc
   build --remote_timeout=120s
   build --remote_max_connections=100
   build --experimental_remote_cache_compression
   ```

3. **Use build without the bytes:**
   ```bash
   # .bazelrc
   build --remote_download_minimal
   ```

4. **Enable local disk cache as L1:**
   ```bash
   # .bazelrc
   build --disk_cache=~/.cache/bazel
   # Fabrik becomes L2/L3
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

- [Bazel Remote Caching Documentation](https://bazel.build/remote/caching)
- [Bazel Remote Execution API](https://github.com/bazelbuild/remote-apis)
- [CLI Reference](../cli-reference.md)
- [Getting Started](../../README.md)
