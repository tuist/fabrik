# Bazel

Fabrik provides transparent remote caching for Bazel via the Bazel Remote Caching protocol (gRPC).

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
url = "grpc://cache.example.com:7070"
timeout = "30s"
```

### 3. Use Bazel Normally

```bash
cd ~/my-bazel-project

# Daemon starts automatically when you enter the directory
# Just use bazel as normal:
bazel build //...
bazel test //...
bazel build //src:myapp --config=release
```

That's it! Bazel automatically uses Fabrik's cache via the `FABRIK_GRPC_URL` environment variable.

## Configuration

### Shell Activation (Recommended for Development)

Shell activation automatically manages the daemon:

```bash
cd ~/project-with-bazel
# Daemon starts, exports FABRIK_GRPC_URL
bazel build //...

cd ~/another-project
# New daemon starts if different config

cd ~
# Daemons cleaned up
```

### Explicit Execution (CI/CD)

For CI/CD pipelines, use `fabrik exec`:

```bash
# In your CI script
fabrik exec bazel build //...
fabrik exec bazel test //...
```

## .bazelrc Configuration

Bazel doesn't automatically read cache URLs from environment variables. You have two options:

### Option 1: Using .bazelrc with Environment Variable

Add to your `.bazelrc`:

```bash
# Use Fabrik cache when available
build --remote_cache=${FABRIK_GRPC_URL}
build --remote_upload_local_results=true
```

**Note:** Bazel doesn't expand environment variables directly. You need to pass it via command:

```bash
bazel build --remote_cache=$(echo $FABRIK_GRPC_URL) //...
```

### Option 2: Script Wrapper (Recommended)

Create a `bazel` wrapper script in your project:

```bash
#!/bin/bash
# bazel-wrapper.sh
if [ -n "$FABRIK_GRPC_URL" ]; then
    exec bazel --remote_cache="$FABRIK_GRPC_URL" "$@"
else
    exec bazel "$@"
fi
```

```bash
chmod +x bazel-wrapper.sh
alias bazel='./bazel-wrapper.sh'
```

### Option 3: Using .bazelrc.user (Per-Developer)

Each developer creates `.bazelrc.user` (gitignored):

```bash
# .bazelrc.user (not in git)
build --remote_cache=grpc://127.0.0.1:58235  # Your local daemon port
build --remote_upload_local_results=true
```

Then in `.bazelrc`:

```bash
# Try to import .bazelrc.user if it exists
try-import %workspace%/.bazelrc.user
```

## How It Works

When Fabrik is activated:

1. **Daemon starts** with a gRPC server implementing Bazel's Remote Caching protocol
2. **Environment variable exported**: `FABRIK_GRPC_URL=grpc://127.0.0.1:{port}`
3. **Bazel connects** to the gRPC server (via your .bazelrc or wrapper)
4. **Cache operations** flow through Fabrik's multi-layer cache

The daemon implements:
- **ContentAddressableStorage (CAS)**: Store/retrieve build artifacts by content hash
- **ActionCache**: Cache action results (mapping action hashes to outputs)
- **Capabilities**: Advertise supported features to Bazel

## Examples

### Development Workflow

```bash
# One-time setup
echo 'eval "$(fabrik activate bash)"' >> ~/.bashrc
source ~/.bashrc

# Daily usage
cd ~/my-bazel-workspace
bazel build //...           # Uses local + remote cache
bazel test //...            # Reuses cached test results
bazel build //src:myapp --config=release
```

### CI/CD Workflow

```yaml
# GitHub Actions
steps:
  - uses: actions/checkout@v4
  
  - run: |
      curl -fsSL https://raw.githubusercontent.com/tuist/fabrik/main/install.sh | sh
      
  - run: fabrik exec bazel build //...
  
  - run: fabrik exec bazel test //...
```

### Multi-Project Setup

Different projects get different daemon instances:

```bash
cd ~/project-a
bazel build //...    # Daemon A (config hash: abc123)

cd ~/project-b  
bazel build //...    # Daemon B (config hash: def456)
```

## Troubleshooting

### Bazel Not Using Cache

Check that environment variable is set:

```bash
echo $FABRIK_GRPC_URL
# Should output: grpc://127.0.0.1:{port}
```

Check daemon is running:

```bash
fabrik daemon list
```

Verify Bazel is configured:

```bash
bazel info | grep remote_cache
```

### Connection Refused

Restart the daemon:

```bash
fabrik daemon stop
fabrik activate --status
```

### Cache Misses

Check cache statistics:

```bash
# TODO: Add stats command
```

## See Also

- [Getting Started](/getting-started) - Complete setup guide
- [CLI Reference](/reference/cli) - Command-line options
- [Configuration](/reference/config-file) - Configuration reference
- [Bazel Remote Caching Docs](https://bazel.build/remote/caching) - Official Bazel documentation
