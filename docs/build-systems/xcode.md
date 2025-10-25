# Xcode

Fabrik provides a wrapper command for Xcode-driven builds that automatically configures build caching for your iOS, macOS, watchOS, and tvOS projects.

::: warning Work in Progress
Support for builds triggered from Xcode's GUI (i.e., building a scheme directly in Xcode) is currently work in progress. For now, use the `fabrik xcodebuild` command-line wrapper documented below. GUI-based build support is under active development.
:::

## Usage

The `fabrik xcodebuild` command is a drop-in replacement for the standard `xcodebuild` command:

```bash
# Instead of: xcodebuild -project MyApp.xcodeproj -scheme MyApp
# Use:
fabrik xcodebuild -- -project MyApp.xcodeproj -scheme MyApp
```

All `xcodebuild` arguments and flags work as normal:

```bash
# Build a workspace
fabrik xcodebuild -- -workspace MyApp.xcworkspace -scheme MyApp -configuration Release

# Clean and build
fabrik xcodebuild -- clean build -project MyApp.xcodeproj -scheme MyApp

# Run tests
fabrik xcodebuild -- test -workspace MyApp.xcworkspace -scheme MyAppTests -destination 'platform=iOS Simulator,name=iPhone 15'
```

## How It Works

When you run `fabrik xcodebuild`, Fabrik:

1. Starts the Fabrik cache daemon (if not already running)
2. Automatically configures Xcode build settings to use the Fabrik cache
3. Passes through all other xcodebuild arguments unchanged
4. Handles graceful shutdown when the build completes
