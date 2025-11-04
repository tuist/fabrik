# Gradle Integration

Complete guide for using Fabrik with Gradle builds.

## Prerequisites

1. ✅ Fabrik installed ([Installation Guide](../../README.md#-getting-started))
2. ✅ Shell integration configured ([Step 2](../../README.md#step-2-set-up-shell-integration-required))
3. ✅ Gradle 6.0 or later

## Quick Start

### 1. Create `.fabrik.toml` in your project root:

```toml
[cache]
dir = ".fabrik/cache"
max_size = "5GB"
```

### 2. Navigate to your project:

```bash
cd ~/my-gradle-project
# Daemon automatically starts
```

### 3. Run your build:

```bash
./gradlew build
```

That's it! Gradle will automatically use the Fabrik cache via the `GRADLE_BUILD_CACHE_URL` environment variable.

## How It Works

1. **Shell Hook Activation**
   - When you `cd` into your project, the Fabrik shell hook detects `.fabrik.toml`
   - Daemon starts automatically and binds to random available ports
   - Environment variable exported: `GRADLE_BUILD_CACHE_URL=http://127.0.0.1:{port}`

2. **Gradle Configuration**
   - Gradle reads `GRADLE_BUILD_CACHE_URL` automatically
   - No `settings.gradle` or `build.gradle` changes needed!
   - Works with both Kotlin DSL and Groovy DSL

3. **Cache Operations**
   - Build outputs are stored in Fabrik cache
   - Subsequent builds retrieve cached outputs
   - Cache misses fall back to upstream if configured

## Verification

Check that caching is working:

```bash
# First build (cache miss)
./gradlew clean build

# Second build (cache hit - should be much faster)
./gradlew clean build
```

You should see significant speedup on the second build.

## Configuration

### Basic Configuration

Minimal `.fabrik.toml` for local caching only:

```toml
[cache]
dir = ".fabrik/cache"
max_size = "10GB"
```

### With Upstream Cache

Share cache across team members:

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

### CI Configuration

For GitHub Actions:

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up JDK
        uses: actions/setup-java@v4
        with:
          java-version: '17'
          distribution: 'temurin'

      - name: Install Fabrik
        run: |
          curl -L https://github.com/tuist/fabrik/releases/latest/download/fabrik-x86_64-unknown-linux-gnu.tar.gz | tar xz
          sudo mv fabrik /usr/local/bin/

      - name: Build with Gradle
        run: ./gradlew build
        env:
          FABRIK_TOKEN: ${{ secrets.FABRIK_TOKEN }}
```

## Advanced Configuration

### Custom Gradle Settings

While Fabrik works without Gradle configuration changes, you can customize cache behavior in `settings.gradle.kts`:

```kotlin
buildCache {
    remote<HttpBuildCache> {
        url = uri(System.getenv("GRADLE_BUILD_CACHE_URL") ?: "http://localhost:8080")
        isPush = true  // Enable push to cache
        isAllowUntrustedServer = true  // For local daemon
    }
}
```

Or in `settings.gradle`:

```groovy
buildCache {
    remote(HttpBuildCache) {
        url = System.getenv("GRADLE_BUILD_CACHE_URL") ?: "http://localhost:8080"
        push = true
        allowUntrustedServer = true
    }
}
```

### Disable Cache for Specific Tasks

```kotlin
// build.gradle.kts
tasks.named("test") {
    outputs.cacheIf { false }  // Never cache test results
}
```

### Cache Key Customization

```kotlin
tasks.register("customTask") {
    inputs.files(fileTree("src"))
    inputs.property("version", project.version)
    outputs.dir("build/custom")
    
    doLast {
        // Task implementation
    }
}
```

## Troubleshooting

### Cache Not Working

1. **Check daemon is running:**
   ```bash
   fabrik doctor --verbose
   ```

2. **Check environment variable:**
   ```bash
   echo $GRADLE_BUILD_CACHE_URL
   # Should show: http://127.0.0.1:{port}
   ```

3. **Enable Gradle build cache:**
   ```bash
   ./gradlew build --build-cache
   ```

### Slow Builds Despite Cache

1. **Check cache hit rate:**
   ```bash
   ./gradlew build --scan
   # Check "Build Cache" section in build scan
   ```

2. **Increase cache size:**
   ```toml
   [cache]
   max_size = "20GB"  # Increase from 5GB
   ```

3. **Check for cache-busting:**
   - Ensure inputs are stable
   - Avoid timestamps or random values in build
   - Check for absolute paths

### Daemon Not Starting

1. **Run doctor command:**
   ```bash
   fabrik doctor
   ```

2. **Check `.fabrik.toml` exists:**
   ```bash
   ls -la .fabrik.toml
   ```

3. **Manually start daemon:**
   ```bash
   fabrik daemon --config .fabrik.toml
   ```

## Performance Tips

### 1. Optimize Cache Size

```toml
[cache]
max_size = "20GB"  # Larger cache = better hit rate
eviction_policy = "lfu"  # Keep frequently used artifacts
```

### 2. Enable Parallel Builds

```properties
# gradle.properties
org.gradle.parallel=true
org.gradle.workers.max=8
org.gradle.caching=true
```

### 3. Use Configuration Cache

```bash
./gradlew build --configuration-cache
```

### 4. Profile Your Build

```bash
./gradlew build --profile
# Check HTML report in build/reports/profile/
```

## Multi-Module Projects

Fabrik works seamlessly with multi-module Gradle projects:

```
my-app/
├── .fabrik.toml
├── settings.gradle.kts
├── build.gradle.kts
├── app/
│   └── build.gradle.kts
├── lib/
│   └── build.gradle.kts
└── common/
    └── build.gradle.kts
```

Each module's outputs are cached independently. Changing one module only rebuilds that module and its dependents.

## Examples

### Android Project

```toml
# .fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "15GB"  # Android builds need more cache

[[upstream]]
url = "grpc://cache.tuist.io:7070"
timeout = "60s"  # Longer timeout for large APKs
```

### Spring Boot Project

```toml
# .fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "5GB"

[[upstream]]
url = "grpc://cache.tuist.io:7070"
timeout = "30s"
```

## Next Steps

- [CLI Reference](../cli-reference.md) - Command-line documentation
- [Bazel Integration](./bazel.md) - For Bazel users
- [Nx Integration](./nx.md) - For Nx users
- [Architecture](../../CLAUDE.md) - Deep dive into Fabrik internals

## See Also

- [Gradle Build Cache Documentation](https://docs.gradle.org/current/userguide/build_cache.html)
- [Gradle Performance Guide](https://docs.gradle.org/current/userguide/performance.html)
- [Fabrik Architecture](../../CLAUDE.md)
