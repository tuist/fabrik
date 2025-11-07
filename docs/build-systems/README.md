# Build System Integration

Build system-specific integration guides for Fabrik.

> **Note:** This section assumes you've already completed the [Getting Started Guide](/getting-started) which covers:
> - Installing Fabrik
> - Setting up shell integration
> - Running `fabrik init`
> - Verifying with `fabrik doctor`

The guides below focus on **build system-specific** configuration and tips.

## Supported Build Systems

### Build Systems

- **[Gradle](./gradle.md)** - Java, Kotlin, Android projects
- **[Bazel](./bazel.md)** - Multi-language monorepos *(coming soon)*
- **[Nx](./nx.md)** - JavaScript/TypeScript monorepos *(coming soon)*
- **[TurboRepo](#.md)** - JavaScript/TypeScript monorepos *(coming soon)*

### Platform-Specific

- **[Xcode](./xcode.md)** - iOS, macOS, watchOS, tvOS apps *(coming soon)*

### Compiler Caches

- **[sccache](#.md.md)** - Rust, C, C++ compiler cache *(coming soon)*

## What These Guides Cover

Each build system guide includes:

- **How it works** - How Fabrik integrates with the build system
- **Quick start** - Minimal example to get started
- **Configuration examples** - Build system-specific configurations
- **Advanced tips** - Performance optimization and best practices
- **Troubleshooting** - Common issues and solutions
- **CI/CD integration** - Examples for popular CI platforms

## What These Guides DON'T Cover

Setup instructions that are **common to all build systems**:

- ❌ Installing Fabrik (see [Getting Started](/getting-started.md#step-1-install-fabrik))
- ❌ Shell integration setup (see [Getting Started](/getting-started.md#step-2-set-up-shell-integration))
- ❌ Running `fabrik init` (see [Getting Started](/getting-started.md#step-4-initialize-your-project))
- ❌ Basic troubleshooting with `fabrik doctor` (see [Getting Started](/getting-started.md#step-3-verify-installation))

## General Pattern

All build systems follow the same pattern:

1. **Navigate to project**: `cd ~/myproject`
2. **Daemon starts automatically**: Fabrik detects `fabrik.toml`
3. **Environment variables exported**: Build tool-specific URLs
4. **Build tool reads env var**: Connects to daemon automatically
5. **Cache magic happens**: Builds are faster! 🚀

## Environment Variables

Fabrik exports these for different build tools:

| Variable | Build Systems | Purpose |
|----------|--------------|---------|
| `FABRIK_HTTP_URL` | All HTTP-based | Generic HTTP cache URL |
| `FABRIK_GRPC_URL` | Bazel, Buck2 | gRPC cache URL |
| `GRADLE_BUILD_CACHE_URL` | Gradle | Gradle-specific cache URL |
| `NX_SELF_HOSTED_REMOTE_CACHE_SERVER` | Nx | Nx-specific cache URL |
| `TURBO_TOKEN` | TurboRepo | TurboRepo auth (future) |
| `XCODE_CACHE_SERVER` | Xcode | Xcode cache URL |

## Example Configuration

A typical `fabrik.toml` works for all build systems:

```toml
# Local cache only
[cache]
dir = ".fabrik/cache"
max_size = "5GB"
```

Or with remote cache:

```toml
# With remote cache
[cache]
dir = ".fabrik/cache"
max_size = "5GB"

[[upstream]]
url = "grpc://cache.tuist.io:7070"
timeout = "30s"

[auth]
token_file = ".fabrik.token"
```

## Contributing

Want to add support for a new build system? See the [GitHub repository](https://github.com/tuist/fabrik) for guidelines.

## See Also

- [Getting Started Guide](/getting-started) - Setup instructions
- [CLI Reference](../cli-reference.md) - Command documentation
- [Architecture](/guide/architecture.md) - Deep dive into Fabrik internals
