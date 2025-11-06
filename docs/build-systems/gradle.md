# Gradle Integration

Gradle integration guide for Fabrik. This assumes you've already [completed the getting started guide](../../README.md#-getting-started).

## How It Works

Gradle automatically reads the `GRADLE_BUILD_CACHE_URL` environment variable that Fabrik exports when you `cd` into your project. No Gradle configuration changes needed!

## Quick Start

```bash
cd ~/my-gradle-project
./gradlew build
```

That's it! Gradle will automatically use Fabrik's cache via the `GRADLE_BUILD_CACHE_URL` environment variable.

## Verification

Check that caching is working:

```bash
# First build (cache miss)
./gradlew clean build

# Second build (cache hit - should be much faster)
./gradlew clean build
```

You should see significant speedup on the second build.
