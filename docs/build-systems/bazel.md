# Bazel Integration

Bazel integration guide for Fabrik. This assumes you've already [completed the getting started guide](../../README.md#-getting-started).

## How It Works

Fabrik provides transparent remote caching for Bazel via the Bazel Remote Caching protocol (gRPC). When you navigate to your project, Fabrik exports `FABRIK_GRPC_URL` which you can use with Bazel's `--remote_cache` flag.

## Quick Start

### Using Shell Alias (Recommended)

Create a shell alias to automatically use Fabrik's cache:

```bash
# Add to ~/.bashrc or ~/.zshrc
alias bazel='command bazel --remote_cache="$FABRIK_GRPC_URL"'

# Then just use bazel normally
cd ~/my-bazel-project
bazel build //...
bazel test //...
```

### Using fabrik exec

```bash
cd ~/my-bazel-project
fabrik exec -- bazel build --remote_cache="$FABRIK_GRPC_URL" //...
```

### Passing Flag Manually

```bash
cd ~/my-bazel-project
# Daemon starts automatically, exports FABRIK_GRPC_URL
bazel build --remote_cache="$FABRIK_GRPC_URL" //...
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
- [CLI Reference](../cli-reference.md)
- [Getting Started](../../README.md)
