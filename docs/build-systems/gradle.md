# Gradle Integration

Gradle integration guide for Fabrik. This assumes you've already [completed the getting started guide](../../README.md#-getting-started).

## How It Works

Gradle automatically reads the `GRADLE_BUILD_CACHE_URL` environment variable that Fabrik exports when you `cd` into your project. No Gradle configuration changes needed!

1. You navigate to your project: `cd ~/my-gradle-project`
2. Fabrik daemon starts and exports `GRADLE_BUILD_CACHE_URL=http://127.0.0.1:{port}`
3. Gradle reads the env var automatically
4. Build outputs are cached in Fabrik

## Quick Start

```bash
cd ~/my-gradle-project
./gradlew build
```

That's it! Gradle will use the cache automatically.

## Verification

Check that caching is working:

```bash
# First build (cache miss)
./gradlew clean build

# Second build (cache hit - should be much faster)
./gradlew clean build
```

You should see significant speedup on the second build.

## Configuration Examples

### Local Cache Only

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "10GB"
```

### With Remote Cache

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "5GB"

[[upstream]]
url = "grpc://cache.tuist.io:7070"
timeout = "30s"

[auth]
token_file = ".fabrik.token"
```

### Android Projects

Android builds benefit from larger cache sizes:

```toml
# fabrik.toml
[cache]
dir = ".fabrik/cache"
max_size = "15GB"  # Android needs more space

[[upstream]]
url = "grpc://cache.tuist.io:7070"
timeout = "60s"  # Longer timeout for large APKs
```

## Advanced Gradle Configuration

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

## Gradle-Specific Tips

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

### Enable Parallel Builds

```properties
# gradle.properties
org.gradle.parallel=true
org.gradle.workers.max=8
org.gradle.caching=true
```

### Use Configuration Cache

```bash
./gradlew build --configuration-cache
```

## Multi-Module Projects

Fabrik works seamlessly with multi-module Gradle projects:

```
my-app/
├── fabrik.toml
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

## Troubleshooting

### Cache Not Working

1. **Check environment variable:**
   ```bash
   echo $GRADLE_BUILD_CACHE_URL
   # Should show: http://127.0.0.1:{port}
   ```

2. **Enable build cache explicitly:**
   ```bash
   ./gradlew build --build-cache
   ```

3. **Check daemon status:**
   ```bash
   fabrik doctor --verbose
   ```

### Slow Builds Despite Cache

1. **Check cache hit rate with build scan:**
   ```bash
   ./gradlew build --scan
   # Check "Build Cache" section in build scan
   ```

2. **Increase cache size:**
   ```toml
   [cache]
   max_size = "20GB"
   ```

3. **Check for cache-busting:**
   - Ensure inputs are stable (no timestamps or random values)
   - Avoid absolute paths in build configuration

### Check Cache Statistics

Use Gradle's built-in reporting:

```bash
# Enable build cache statistics
./gradlew build --build-cache --info | grep "Build cache"
```

## Performance Tips

1. **Larger cache = better hit rate**
   ```toml
   [cache]
   max_size = "20GB"
   eviction_policy = "lfu"  # Keep frequently used artifacts
   ```

2. **Profile your build**
   ```bash
   ./gradlew build --profile
   # Check HTML report in build/reports/profile/
   ```

3. **Use Gradle 8+** for best caching performance

## CI/CD Integration

### GitHub Actions

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

      - name: Build
        run: ./gradlew build
        env:
          FABRIK_TOKEN: ${{ secrets.FABRIK_TOKEN }}
```

