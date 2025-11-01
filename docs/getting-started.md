# Getting Started with Fabrik

Fabrik provides a transparent build cache that works with your existing build tools. Instead of wrapping your build commands, Fabrik runs as a background daemon that your tools connect to automatically.

## Quick Start

### Installation

```bash
# Download the latest release for your platform
curl -fsSL https://raw.githubusercontent.com/tuist/fabrik/main/install.sh | sh

# Or install with cargo
cargo install fabrik
```

### Shell Integration (Recommended for Development)

Add Fabrik to your shell to automatically activate the cache when you enter a project:

```bash
# Bash
echo 'eval "$(fabrik activate bash)"' >> ~/.bashrc

# Zsh
echo 'eval "$(fabrik activate zsh)"' >> ~/.zshrc

# Fish
echo 'fabrik activate fish | source' >> ~/.config/fish/config.fish
```

Restart your shell or source the config file.

### Project Configuration

Create a `.fabrik.toml` file in your project root:

```toml
[cache]
dir = ".fabrik/cache"
max_size = "10GB"

# Optional: Connect to a remote cache
[[upstream]]
url = "https://cache.example.com"
timeout = "30s"
```

### Usage

Once activated, just use your build tools normally:

```bash
cd ~/my-project

# The daemon starts automatically
# Build tools will use the cache transparently

bazel build //...
nx build my-app
gradle build
xcodebuild ...
```

That's it! The cache is now active for all your builds.

## Commands

### `fabrik activate`

Sets up shell integration to automatically manage the cache daemon.

**Usage:**

```bash
# Initial setup - add to your shell config
eval "$(fabrik activate bash)"   # For bash
eval "$(fabrik activate zsh)"    # For zsh
fabrik activate fish | source    # For fish
```

**What it does:**

- Detects when you enter a directory with `.fabrik.toml`
- Starts a daemon with that project's configuration
- Exports environment variables for build tools
- Stops daemons when no longer needed

**Environment Variables Exported:**

```bash
FABRIK_HTTP_URL=http://127.0.0.1:58234
FABRIK_GRPC_URL=grpc://127.0.0.1:58235

# Convenience variables for build tools
GRADLE_BUILD_CACHE_URL=http://127.0.0.1:58234
NX_SELF_HOSTED_REMOTE_CACHE_SERVER=http://127.0.0.1:58234
XCODE_CACHE_SERVER=http://127.0.0.1:58234
```

**Manual activation:**

```bash
# Check status and start daemon if needed
fabrik activate --status
```

---

### `fabrik exec`

Run a command with the cache daemon guaranteed to be running.

**Usage:**

```bash
fabrik exec <command> [args...]
```

**Examples:**

```bash
# CI builds
fabrik exec bazel build //...
fabrik exec nx build my-app
fabrik exec gradle build

# Keep daemon alive for multiple commands
fabrik exec --keep-alive nx build my-app
nx test my-app  # Reuses the daemon
fabrik deactivate  # Clean up when done
```

**Options:**

- `--keep-alive` - Don't stop the daemon after command exits (default)
- `--kill-after` - Stop the daemon when the command completes

**When to use:**

- **CI/CD pipelines** - Ensures consistent cache behavior
- **One-off builds** - Don't want shell integration
- **Scripts** - Programmatic cache management

---

### `fabrik daemon`

Manually manage cache daemons.

**Commands:**

```bash
# Start daemon for current directory's config
fabrik daemon start

# Stop daemon for current config
fabrik daemon stop

# List all running daemons
fabrik daemon list

# Stop all daemons
fabrik daemon stop --all

# Clean up orphaned daemons
fabrik daemon clean
```

**Examples:**

```bash
# Start daemon explicitly
fabrik daemon start

# Use in multiple terminals
bazel build //...    # Terminal 1
nx build demo        # Terminal 2

# Stop when done
fabrik daemon stop
```

---

### `fabrik deactivate`

Remove Fabrik environment variables and optionally stop daemons.

**Usage:**

```bash
# Unset environment variables
fabrik deactivate

# Also stop the daemon
fabrik deactivate --stop-daemon
```

---

## Configuration

### Project Configuration (`.fabrik.toml`)

Place this in your project root:

```toml
[cache]
dir = ".fabrik/cache"        # Local cache directory
max_size = "10GB"             # Maximum cache size
eviction_policy = "lru"       # lru, lfu, or ttl

# Optional: Connect to remote cache
[[upstream]]
url = "https://cache.example.com:7070"
timeout = "30s"

# Optional: Authentication
[auth]
token = "${FABRIK_TOKEN}"     # JWT token from environment
```

### Global Configuration (`~/.config/fabrik/config.toml`)

Used as fallback when no project config is found:

```toml
[cache]
dir = "~/.cache/fabrik"
max_size = "50GB"

[[upstream]]
url = "https://cache.example.com:7070"
```

### Environment Variables

Override configuration with environment variables:

```bash
FABRIK_CACHE_DIR=/tmp/cache
FABRIK_CACHE_MAX_SIZE=5GB
FABRIK_TOKEN=eyJ0eXAi...
```

---

## Build Tool Integration

### Gradle

Fabrik automatically exports `GRADLE_BUILD_CACHE_URL`. Just enable remote cache in `settings.gradle`:

```gradle
buildCache {
    remote(HttpBuildCache) {
        enabled = true
        url = System.getenv('GRADLE_BUILD_CACHE_URL')
        push = true
    }
}
```

### Nx

Fabrik exports `NX_SELF_HOSTED_REMOTE_CACHE_SERVER`. Enable in `nx.json`:

```json
{
  "tasksRunnerOptions": {
    "default": {
      "runner": "nx/tasks-runners/default",
      "options": {
        "cacheableOperations": ["build", "test"],
        "remoteCache": {
          "enabled": true
        }
      }
    }
  }
}
```

Nx automatically reads `NX_SELF_HOSTED_REMOTE_CACHE_SERVER` from the environment.

### Bazel

Bazel doesn't support cache URLs via environment variables. Use `.bazelrc`:

```bash
# .bazelrc
build --remote_cache=${FABRIK_GRPC_URL}
build --remote_upload_local_results=true
```

Or use the gRPC URL from the environment:

```bash
bazel build --remote_cache=$(echo $FABRIK_GRPC_URL) //...
```

### Xcode

Configure in your build settings or use `xcodebuild`:

```bash
xcodebuild \
  -scheme MyApp \
  -destination 'platform=iOS Simulator,name=iPhone 15' \
  CACHE_SERVER_URL=$XCODE_CACHE_SERVER \
  build
```

---

## Workflows

### Development Workflow

```bash
# One-time setup
echo 'eval "$(fabrik activate bash)"' >> ~/.bashrc
source ~/.bashrc

# Daily usage - just cd into your project
cd ~/my-project
# Daemon starts automatically ✓

bazel build //...
nx build my-app
gradle build
# All builds use the cache transparently
```

### CI/CD Workflow

```bash
# In your CI pipeline
steps:
  - checkout: code
  
  - run: |
      # Install Fabrik
      curl -fsSL https://raw.githubusercontent.com/tuist/fabrik/main/install.sh | sh
      
      # Run builds with cache
      fabrik exec bazel build //...
      fabrik exec nx test --all
      fabrik exec gradle build
```

### Multi-Project Workflow

Different projects automatically get different daemon instances:

```bash
cd ~/project-a
bazel build //...  # Uses daemon A (config hash: abc123)

cd ~/project-b
nx build app       # Uses daemon B (config hash: def456)

cd ~
# Daemons can be cleaned up automatically or manually
fabrik daemon clean
```

---

## Troubleshooting

### Check if daemon is running

```bash
fabrik daemon list
```

### View daemon logs

```bash
tail -f ~/.fabrik/logs/<config-hash>.log
```

### Daemon not starting

```bash
# Check config is valid
cat .fabrik.toml

# Manually start to see errors
fabrik daemon start
```

### Environment variables not set

```bash
# Re-run activation
fabrik activate --status

# Or manually export
eval "$(fabrik activate bash)"
```

### Clean up all daemons

```bash
fabrik daemon stop --all
fabrik daemon clean
```

---

## Advanced Usage

### Custom Daemon Lifecycle

```bash
# Start daemon and keep it alive
fabrik daemon start

# Run multiple commands
bazel build //...
bazel test //...
nx build app

# Stop when done
fabrik daemon stop
```

### Multiple Configurations

You can have different configs in subdirectories:

```
my-monorepo/
├── .fabrik.toml          # Default config
├── backend/
│   └── .fabrik.toml      # Backend-specific cache
└── frontend/
    └── .fabrik.toml      # Frontend-specific cache
```

Each directory gets its own daemon instance.

---

## Migration from Wrapper Commands

If you were using the old wrapper approach:

**Old:**
```bash
fabrik bazel -- build //...
fabrik nx -- build my-app
```

**New (Shell Activation):**
```bash
eval "$(fabrik activate bash)"
bazel build //...
nx build my-app
```

**New (Explicit Execution):**
```bash
fabrik exec bazel build //...
fabrik exec nx build my-app
```

---

## Next Steps

- Configure your project with `.fabrik.toml`
- Set up shell integration with `fabrik activate`
- Connect to a remote cache (optional)
- Check out [CLAUDE.md](./CLAUDE.md) for architecture details
