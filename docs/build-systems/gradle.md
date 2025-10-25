# Gradle

Fabrik provides a wrapper command for Gradle that automatically configures remote build caching via the Gradle Build Cache HTTP API.

## Usage

The `fabrik gradle` command is a drop-in replacement for the standard `./gradlew` command:

```bash
# Instead of: ./gradlew build
# Use:
fabrik gradle -- build
```

All `gradle` arguments and flags work as normal:

```bash
# Build a specific project
fabrik gradle -- :app:build

# Run tests
fabrik gradle -- test

# Clean and build
fabrik gradle -- clean build

# Build with configuration cache
fabrik gradle -- build --configuration-cache

# Run specific task
fabrik gradle -- :app:assemble --parallel

# Show tasks
fabrik gradle -- tasks --all
```

## How It Works

When you run `fabrik gradle`, Fabrik:

1. Starts a local HTTP server implementing the Gradle Build Cache HTTP API
2. Automatically injects the cache URL via Gradle system properties
3. Passes through all your gradle arguments unchanged
4. Handles graceful shutdown when the build completes

The local cache server implements the following endpoints from the Gradle Build Cache HTTP API:
- **GET /cache/{hash}**: Retrieve cached build artifacts by content hash
- **PUT /cache/{hash}**: Store build artifacts

## Configuration

Fabrik automatically enables the build cache and configures the remote cache URL. You can optionally create a `gradle.properties` file in your project to enable additional caching features:

```properties
# gradle.properties
org.gradle.caching=true
```

And an `init.gradle.kts` file for remote cache configuration:

```kotlin
// init.gradle.kts
gradle.settingsEvaluated {
    buildCache {
        remote<HttpBuildCache> {
            val cacheUrl = System.getProperty("org.gradle.caching.buildCache.remote.url")
            if (cacheUrl != null) {
                url = uri(cacheUrl)
                isPush = true
            }
        }
    }
}
```

When using Fabrik, these configurations are optional as the wrapper automatically handles cache configuration via system properties.

## Requirements

- Gradle must be installed or use the Gradle wrapper (`gradlew`) in your project
- Java Development Kit (JDK) matching your project's requirements

## See Also

- [CLI Reference](/reference/cli) - Full command-line options
- [Configuration File](/reference/config-file) - Complete configuration reference
- [Gradle Build Cache Documentation](https://docs.gradle.org/current/userguide/build_cache.html) - Official Gradle docs
