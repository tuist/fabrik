# Bazel

Fabrik provides a wrapper command for Bazel that automatically configures remote caching via the Bazel Remote Caching protocol.

## Usage

The `fabrik bazel` command is a drop-in replacement for the standard `bazel` command:

```bash
# Instead of: bazel build //...
# Use:
fabrik bazel -- build //...
```

All `bazel` arguments and flags work as normal:

```bash
# Build a specific target
fabrik bazel -- build //src:myapp

# Run tests
fabrik bazel -- test //...

# Build with configuration
fabrik bazel -- build //... --config=release --jobs=8

# Query targets
fabrik bazel -- query 'deps(//...)'

# Clean build artifacts
fabrik bazel -- clean
```

## How It Works

When you run `fabrik bazel`, Fabrik:

1. Starts a local gRPC server implementing the Bazel Remote Caching protocol
2. Automatically injects the `--remote_cache=grpc://localhost:{port}` flag
3. Passes through all your bazel arguments unchanged
4. Handles graceful shutdown when the build completes

The local cache server implements the following gRPC services from the Bazel Remote APIs:
- **ContentAddressableStorage (CAS)**: For storing and retrieving build artifacts (blobs) by content-addressed digest
- **ActionCache**: For caching action results (mapping action hashes to outputs)
- **Capabilities**: For advertising supported features to Bazel clients

## Requirements

- Bazel must be installed and available in `PATH`

## See Also

- [CLI Reference](/reference/cli) - Full command-line options
- [Configuration File](/reference/config-file) - Complete configuration reference
- [Bazel Remote Caching Documentation](https://bazel.build/remote/caching) - Official Bazel docs
