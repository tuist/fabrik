# Gradle Fixture

A minimal Gradle project with Kotlin for testing Fabrik's Gradle integration.

## Project Structure

```
gradle/
├── app/
│   ├── build.gradle.kts          # Build configuration
│   └── src/main/kotlin/com/example/
│       └── App.kt                # Simple Kotlin file
├── settings.gradle.kts           # Project settings
└── gradlew                       # Gradle wrapper script
```

## Building

```bash
# Build the project
./gradlew build

# Run the application
./gradlew run

# Clean build artifacts
./gradlew clean
```

## What it does

This is the simplest possible Gradle + Kotlin project that:
- Compiles a single Kotlin source file (`App.kt`)
- Produces a runnable application
- Uses Kotlin JVM plugin with Java 17 toolchain
- Outputs "Hello from Gradle + Kotlin!" when run
