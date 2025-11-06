# Build System Integration

Fabrik supports any build system with remote caching capabilities. Choose your build system below for detailed integration instructions.

## Supported Build Systems

### Build Systems

- **[Gradle](./gradle.md)** - Java, Kotlin, Android projects
- **[Bazel](./bazel.md)** - Multi-language monorepos *(coming soon)*
- **[Nx](./nx.md)** - JavaScript/TypeScript monorepos *(coming soon)*
- **[TurboRepo](./turborepo.md)** - JavaScript/TypeScript monorepos *(coming soon)*

### Platform-Specific

- **[Xcode](./xcode.md)** - iOS, macOS, watchOS, tvOS apps *(coming soon)*

### Compiler Caches

- **[sccache](./sccache.md)** - Rust, C, C++ compiler cache *(coming soon)*

## General Integration Pattern

All build systems follow the same pattern:

1. **Install Fabrik** ([Getting Started](../../README.md#-getting-started))
2. **Set up shell integration** (Required - see [Step 2](../../README.md#step-2-set-up-shell-integration-required))
3. **Create `fabrik.toml`** in your project root
4. **Navigate to project** - Daemon starts automatically
5. **Run your build** - Build tool reads environment variables and uses cache

## Environment Variables

Fabrik exports these environment variables for build tools:

| Variable | Build Systems | Purpose |
|----------|--------------|---------|
| `FABRIK_HTTP_URL` | All HTTP-based | Generic HTTP cache URL |
| `FABRIK_GRPC_URL` | Bazel, Buck2 | gRPC cache URL |
| `GRADLE_BUILD_CACHE_URL` | Gradle | Gradle-specific cache URL |
| `NX_SELF_HOSTED_REMOTE_CACHE_SERVER` | Nx | Nx-specific cache URL |
| `TURBO_TOKEN` | TurboRepo | TurboRepo auth (future) |
| `XCODE_CACHE_SERVER` | Xcode | Xcode cache URL |

## Example Configuration

Minimal `fabrik.toml` that works for all build systems:

```toml
[cache]
dir = ".fabrik/cache"
max_size = "5GB"
```

With upstream cache:

```toml
[cache]
dir = ".fabrik/cache"
max_size = "5GB"

[[upstream]]
url = "grpc://cache.tuist.io:7070"
timeout = "30s"

[auth]
token_file = ".fabrik.token"
```

## Common Troubleshooting

### Cache Not Working

1. **Verify shell integration:**
   ```bash
   fabrik doctor
   ```

2. **Check environment variables:**
   ```bash
   env | grep FABRIK
   env | grep GRADLE  # For Gradle
   env | grep NX      # For Nx
   ```

3. **Check daemon is running:**
   ```bash
   fabrik doctor --verbose
   ```

### Performance Issues

1. **Increase cache size:**
   ```toml
   [cache]
   max_size = "20GB"
   ```

2. **Use LFU eviction:**
   ```toml
   [cache]
   eviction_policy = "lfu"
   ```

3. **Add upstream cache:**
   ```toml
   [[upstream]]
   url = "grpc://cache.tuist.io:7070"
   ```

## Contributing

Want to add support for a new build system? See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## See Also

- [CLI Reference](../cli-reference.md)
- [Architecture](../../CLAUDE.md)
- [Getting Started](../../README.md)
