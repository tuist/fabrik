# Bazel Integration

Bazel integration guide for Fabrik. This assumes you've already [completed the getting started guide](../../README.md#-getting-started).

## How It Works

Fabrik provides transparent remote caching for Bazel via the Bazel Remote Caching protocol (gRPC). When you navigate to your project, Fabrik exports `FABRIK_GRPC_URL` which you can use in your Bazel configuration.

## Quick Start

### 1. Configure Bazel

Add to your `.bazelrc`:

```bash
# Use Fabrik cache
build --remote_cache=grpc://localhost:9090
build --remote_upload_local_results=true
```

**Note**: You'll need to update the port dynamically since Fabrik uses random ports. See configuration options below.

### 2. Build

```bash
cd ~/my-bazel-project
bazel build //...
```

## Configuration Options

Since Bazel doesn't directly read environment variables in `.bazelrc`, you have several options:

### Option 1: Wrapper Script (Recommended)

Create a `bazel` wrapper script:

```bash
#!/bin/bash
# bazel-wrapper.sh
if [ -n "$FABRIK_GRPC_URL" ]; then
    # Convert grpc://localhost:54322 to grpc://localhost:54322
    exec command bazel --remote_cache="$FABRIK_GRPC_URL" "$@"
else
    exec command bazel "$@"
fi
```

Make it executable and use it:

```bash
chmod +x bazel-wrapper.sh
./bazel-wrapper.sh build //...
```

### Option 2: Shell Alias

Add to your shell config:

```bash
# ~/.bashrc or ~/.zshrc
alias bazel='command bazel --remote_cache="$FABRIK_GRPC_URL"'
```

### Option 3: Explicit Flag

Pass the flag directly:

```bash
bazel build --remote_cache="$FABRIK_GRPC_URL" //...
```

## Bazel-Specific Configuration

### Local Cache Only

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "20GB"  # Bazel can generate lots of artifacts
```

### With Remote Cache

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "10GB"

[[upstream]]
url = "grpc://cache.tuist.io:7070"
timeout = "30s"
```

## Advanced Bazel Configuration

### Remote Execution Settings

```bash
# .bazelrc
build --remote_cache=grpc://localhost:9090
build --remote_upload_local_results=true
build --remote_timeout=60s
build --remote_retries=3

# Optional: Remote execution (if server supports it)
# build --remote_executor=grpc://localhost:9090
```

### Disk Cache + Remote Cache

```bash
# .bazelrc
# Use both disk cache and remote cache
build --disk_cache=~/.cache/bazel
build --remote_cache=grpc://localhost:9090
```

### Platform-Specific Settings

```bash
# .bazelrc
build:linux --remote_cache=grpc://localhost:9090
build:macos --remote_cache=grpc://localhost:9090
build:windows --remote_cache=grpc://localhost:9090
```

## Troubleshooting

### Cache Not Working

1. **Check environment variable:**
   ```bash
   echo $FABRIK_GRPC_URL
   # Should show: grpc://127.0.0.1:{port}
   ```

2. **Verify Bazel sees the cache:**
   ```bash
   bazel build --remote_cache="$FABRIK_GRPC_URL" //... --explain=explain.txt
   cat explain.txt | grep cache
   ```

3. **Check daemon status:**
   ```bash
   fabrik doctor --verbose
   ```

### Connection Errors

If you see "failed to connect to remote cache":

1. **Verify daemon is running:**
   ```bash
   fabrik doctor
   ```

2. **Check gRPC port:**
   ```bash
   echo $FABRIK_GRPC_URL
   # Should be: grpc://127.0.0.1:{port}
   ```

3. **Test connection:**
   ```bash
   grpcurl -plaintext 127.0.0.1:{port} list
   ```

### Slow Builds Despite Cache

1. **Enable verbose logging:**
   ```bash
   bazel build --remote_cache="$FABRIK_GRPC_URL" //... --execution_log_json_file=exec.json
   ```

2. **Check cache hit rate:**
   ```bash
   cat exec.json | jq '.[] | .remoteCacheHit' | sort | uniq -c
   ```

3. **Increase cache size:**
   ```toml
   [cache]
   max_size = "50GB"
   ```

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
        run: bazel build --remote_cache="$FABRIK_GRPC_URL" //...
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
   ```

3. **Use build without the bytes:**
   ```bash
   # .bazelrc
   build --remote_download_minimal
   ```

## See Also

- [Bazel Remote Caching Documentation](https://bazel.build/remote/caching)
- [Bazel Remote Execution API](https://github.com/bazelbuild/remote-apis)
- [CLI Reference](../cli-reference.md)
- [Getting Started](../../README.md)
