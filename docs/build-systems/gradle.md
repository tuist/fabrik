# Gradle

Fabrik provides transparent remote build caching for Gradle via the Gradle Build Cache HTTP API.

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
max_size = "10GB"

# Optional: Connect to remote cache
[[upstream]]
url = "http://cache.example.com:8080"
timeout = "30s"
```

### 3. Enable Gradle Build Cache

Add to your `settings.gradle` or `settings.gradle.kts`:

```kotlin
// settings.gradle.kts
buildCache {
    local {
        isEnabled = true
    }
    remote<HttpBuildCache> {
        url = uri(System.getenv("GRADLE_BUILD_CACHE_URL") ?: "")
        isEnabled = System.getenv("GRADLE_BUILD_CACHE_URL") != null
        isPush = true
    }
}
```

Or for Groovy:

```groovy
// settings.gradle
buildCache {
    local {
        enabled = true
    }
    remote(HttpBuildCache) {
        url = System.getenv('GRADLE_BUILD_CACHE_URL') ?: ''
        enabled = System.getenv('GRADLE_BUILD_CACHE_URL') != null
        push = true
    }
}
```

### 4. Use Gradle Normally

```bash
cd ~/my-gradle-project

# Daemon starts automatically
./gradlew build
./gradlew test
./gradlew :app:assemble
```

That's it! Gradle automatically uses Fabrik's cache via the `GRADLE_BUILD_CACHE_URL` environment variable.

## Configuration

### Shell Activation (Recommended for Development)

Shell activation automatically manages the daemon:

```bash
cd ~/gradle-project
# Daemon starts, exports GRADLE_BUILD_CACHE_URL
./gradlew build

cd ~/another-project
# New daemon starts if different config
```

### Explicit Execution (CI/CD)

For CI/CD pipelines, use `fabrik exec`:

```bash
# In your CI script
fabrik exec ./gradlew build
fabrik exec ./gradlew test
```

## How It Works

When Fabrik is activated:

1. **Daemon starts** with an HTTP server implementing Gradle's Build Cache API
2. **Environment variable exported**: `GRADLE_BUILD_CACHE_URL=http://127.0.0.1:{port}`
3. **Gradle connects** to the HTTP server (via your settings.gradle configuration)
4. **Cache operations** flow through Fabrik's multi-layer cache

The daemon implements:
- **GET /cache/{hash}**: Retrieve cached build outputs
- **PUT /cache/{hash}**: Store build outputs
- **HEAD /cache/{hash}**: Check if artifact exists

## Examples

### Development Workflow

```bash
# One-time setup
echo 'eval "$(fabrik activate bash)"' >> ~/.bashrc
source ~/.bashrc

# Daily usage
cd ~/my-gradle-project
./gradlew clean build        # First build (cache miss)
./gradlew clean build        # Second build (cache hit) - much faster!
./gradlew :app:test          # Reuses cached dependencies
```

### CI/CD Workflow

```yaml
# GitHub Actions
steps:
  - uses: actions/checkout@v4
  
  - uses: actions/setup-java@v4
    with:
      java-version: '17'
      distribution: 'temurin'
  
  - run: |
      curl -fsSL https://raw.githubusercontent.com/tuist/fabrik/main/install.sh | sh
      
  - run: fabrik exec ./gradlew build
  
  - run: fabrik exec ./gradlew test
```

### Multi-Module Projects

```bash
cd ~/my-monorepo

# Build specific modules
./gradlew :backend:build
./gradlew :frontend:build

# All modules share the same cache
./gradlew build  # Reuses artifacts from previous builds
```

## Advanced Configuration

### Gradle Properties

Enable additional caching features in `gradle.properties`:

```properties
# gradle.properties
org.gradle.caching=true
org.gradle.parallel=true
org.gradle.configureondemand=true
```

### Custom Cache Configuration

For more control, you can customize the cache behavior:

```kotlin
// settings.gradle.kts
buildCache {
    local {
        directory = file(".gradle/build-cache")
        removeUnusedEntriesAfterDays = 7
    }
    remote<HttpBuildCache> {
        url = uri(System.getenv("GRADLE_BUILD_CACHE_URL") ?: "")
        isEnabled = System.getenv("GRADLE_BUILD_CACHE_URL") != null
        isPush = true
        
        // Optional: Authentication
        credentials {
            username = System.getenv("GRADLE_CACHE_USERNAME")
            password = System.getenv("GRADLE_CACHE_PASSWORD")
        }
    }
}
```

### Selective Caching

Cache only specific tasks:

```kotlin
// build.gradle.kts
tasks.withType<Test> {
    outputs.cacheIf { true }
}

tasks.withType<JavaCompile> {
    outputs.cacheIf { true }
}
```

## Troubleshooting

### Gradle Not Using Cache

Check that environment variable is set:

```bash
echo $GRADLE_BUILD_CACHE_URL
# Should output: http://127.0.0.1:{port}
```

Check daemon is running:

```bash
fabrik daemon list
```

Enable Gradle build cache logging:

```bash
./gradlew build --info | grep cache
```

### Cache Misses

Check Gradle build scan:

```bash
./gradlew build --scan
```

Verify tasks are cacheable:

```kotlin
tasks.withType<MyTask> {
    outputs.cacheIf { true }
}
```

### Connection Issues

Restart the daemon:

```bash
fabrik daemon stop
fabrik activate --status
```

## Performance Tips

1. **Enable parallel builds**: Add `org.gradle.parallel=true` to `gradle.properties`
2. **Use configuration cache**: Run with `--configuration-cache`
3. **Exclude generated files**: Don't cache generated source files
4. **Cache unit tests**: Enable for Test tasks with `outputs.cacheIf { true }`

## See Also

- [Getting Started](/getting-started) - Complete setup guide
- [CLI Reference](/reference/cli) - Command-line options
- [Configuration](/reference/config-file) - Configuration reference
- [Gradle Build Cache Docs](https://docs.gradle.org/current/userguide/build_cache.html) - Official Gradle documentation
