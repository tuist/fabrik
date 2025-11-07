# CLI Reference

Complete reference for all Fabrik CLI commands.

## Overview

Fabrik uses an activation-based approach for managing build caches. Instead of wrapping your build commands, Fabrik runs as a background daemon that your build tools connect to automatically.

**Two main workflows:**

1. **Shell Integration** (`fabrik activate`) - Automatic daemon management for development
2. **Explicit Execution** (`fabrik exec`) - Manual daemon management for CI/CD

See the [Getting Started Guide](/getting-started.md) for detailed usage examples.

## `fabrik activate`

Set up shell integration to automatically manage the cache daemon.

### Usage

```bash
# Initial setup - outputs shell hook code
fabrik activate <SHELL>

# Check status and start daemon if needed
fabrik activate --status
```

### Shells

- `bash` - Bash shell
- `zsh` - Zsh shell  
- `fish` - Fish shell

### Examples

```bash
# One-time shell setup (add to .bashrc or .zshrc)
echo 'eval "$(fabrik activate bash)"' >> ~/.bashrc
source ~/.bashrc

# For zsh
echo 'eval "$(fabrik activate zsh)"' >> ~/.zshrc

# For fish
echo 'fabrik activate fish | source' >> ~/.config/fish/config.fish

# Manual activation (check/start daemon)
fabrik activate --status
```

### What It Does

When you `cd` into a directory:

1. **Searches** for `.fabrik.toml` up the directory tree
2. **Computes** configuration hash to identify unique daemon
3. **Checks** if daemon with that config is running
4. **Starts** daemon if not running
5. **Exports** environment variables for build tools:
   ```bash
   FABRIK_HTTP_URL=http://127.0.0.1:58234
   FABRIK_GRPC_URL=grpc://127.0.0.1:58235
   GRADLE_BUILD_CACHE_URL=http://127.0.0.1:58234
   NX_SELF_HOSTED_REMOTE_CACHE_SERVER=http://127.0.0.1:58234
   XCODE_CACHE_SERVER=http://127.0.0.1:58234
   ```

### Configuration

Searches for configuration in this order:

1. `$PWD/.fabrik.toml`
2. `$PWD/../.fabrik.toml` (continues up to root)
3. `~/.config/fabrik/config.toml` (global fallback)

Different configurations = different daemon instances.

See the [Getting Started Guide](/getting-started#shell-integration-recommended-for-development) for complete setup.

## `fabrik exec`

Execute a command with guaranteed daemon lifecycle.

### Usage

```bash
fabrik exec [OPTIONS] <COMMAND> [ARGS...]
```

### Options

| Option | Description |
|--------|-------------|
| `--keep-alive` | Don't stop daemon after command exits (default) |
| `--kill-after` | Stop daemon when command completes |
| `--config <PATH>` | Path to configuration file |

### Examples

```bash
# Basic usage - daemon keeps running after
fabrik exec bazel build //...
fabrik exec nx build my-app
fabrik exec gradle build

# Keep daemon alive for subsequent commands
fabrik exec --keep-alive nx build my-app
nx test my-app  # Reuses the same daemon
fabrik deactivate  # Clean up when done

# Stop daemon after command
fabrik exec --kill-after bazel test //...

# With custom configuration
fabrik exec --config .fabrik.toml bazel build //...
```

### What It Does

1. **Finds** `.fabrik.toml` in current directory tree
2. **Starts** daemon if not running (or reuses existing)
3. **Exports** environment variables (FABRIK_HTTP_URL, etc.)
4. **Executes** your command with those variables set
5. **Optionally** stops daemon after completion (if `--kill-after`)

### When to Use

- **CI/CD pipelines** - Ensures consistent cache behavior
- **One-off builds** - Don't want permanent shell integration
- **Scripts** - Programmatic daemon management

See the [Getting Started Guide](/getting-started#workflow-2-explicit-execution-ci-friendly) for complete examples.

## `fabrik daemon`

Manually manage cache daemons.

### Commands

```bash
# Start daemon for current directory's config
fabrik daemon start [OPTIONS]

# Stop daemon for current config  
fabrik daemon stop

# List all running daemons
fabrik daemon list

# Stop all daemons
fabrik daemon stop --all

# Clean up orphaned daemons
fabrik daemon clean
```

### Options (for `start`)

| Option | Description |
|--------|-------------|
| `--config <PATH>` | Path to configuration file |

### Examples

```bash
# Start daemon explicitly
fabrik daemon start

# Start with custom config
fabrik daemon start --config .fabrik.toml

# Use in multiple terminals
bazel build //...    # Terminal 1
nx build demo        # Terminal 2
gradle build         # Terminal 3

# List running daemons
fabrik daemon list

# Stop specific daemon (for current directory)
fabrik daemon stop

# Stop all daemons
fabrik daemon stop --all

# Clean up orphaned daemons
fabrik daemon clean
```

### Daemon State

Daemons are tracked in `~/.fabrik/daemons/<config-hash>/`:

```
~/.fabrik/daemons/<config-hash>/
├── pid              # Process ID
├── ports.json       # HTTP/gRPC/metrics ports
├── config_path.txt  # Path to config file
```

Each unique configuration gets its own daemon instance.

## `fabrik deactivate`

Remove Fabrik environment variables and optionally stop daemons.

### Usage

```bash
fabrik deactivate [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--stop-daemon` | Also stop the daemon for current directory |

### Examples

```bash
# Unset environment variables only
fabrik deactivate

# Also stop the daemon
fabrik deactivate --stop-daemon
```

### What It Does

Removes Fabrik environment variables from your shell:

- `FABRIK_HTTP_URL`
- `FABRIK_GRPC_URL`
- `FABRIK_CONFIG_HASH`
- `FABRIK_DAEMON_PID`
- `GRADLE_BUILD_CACHE_URL`
- `NX_SELF_HOSTED_REMOTE_CACHE_SERVER`
- `XCODE_CACHE_SERVER`

## `fabrik server`

Run a regional/cloud cache server (Layer 2 cache).

**Note:** This is for running Fabrik as a remote cache server that multiple developers/CI runners connect to. Most users should use `fabrik activate` or `fabrik exec` instead.

### Usage

```bash
fabrik server --config <CONFIG_FILE>
```

### Options

| Option | Description |
|--------|-------------|
| `--config <PATH>` | Path to server configuration file (required) |

### Examples

```bash
# Start Layer 2 server
fabrik server --config /etc/fabrik/server.toml

# Generate example server config
fabrik config generate --template server > server.toml
fabrik server --config server.toml
```

### Server Configuration

Server configuration requires more settings than local daemon:

```toml
[cache]
dir = "/data/fabrik/cache"
max_size = "500GB"

[[upstream]]
url = "s3://my-bucket/cache/"
region = "us-east-1"
permanent = true

[auth]
public_key_file = "/etc/fabrik/jwt-public-key.pem"

[fabrik]
enabled = true
bind = "0.0.0.0:7070"  # gRPC server for Fabrik protocol

[observability]
metrics_bind = "0.0.0.0:9091"
health_bind = "0.0.0.0:8888"
```

See [guide/architecture.md](/guide/architecture.md#layer-2-regional-server-with-s3-upstream) for complete server setup.

## `fabrik config`

Configuration utilities.

### Commands

```bash
# Validate configuration file
fabrik config validate <PATH>

# Generate example configuration
fabrik config generate --template <TEMPLATE>

# Show effective configuration
fabrik config show
```

### Examples

```bash
# Validate project config
fabrik config validate .fabrik.toml

# Generate project config template
fabrik config generate --template project > .fabrik.toml

# Generate server config template
fabrik config generate --template server > server.toml

# Show current effective configuration
fabrik config show
```

### Templates

- `project` - Local daemon configuration (for `.fabrik.toml`)
- `server` - Remote server configuration (for Layer 2)

### Project Config Example

```toml
[cache]
dir = ".fabrik/cache"
max_size = "10GB"

# Optional: Connect to remote cache
[[upstream]]
url = "grpc://cache.example.com:7070"
timeout = "30s"

# Optional: Authentication
[auth]
token = "${FABRIK_TOKEN}"
```

## `fabrik health`

Health check and diagnostics.

### Usage

```bash
fabrik health [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--url <URL>` | Health endpoint URL (default: `http://localhost:8888/health`) |
| `--timeout <DURATION>` | Request timeout (default: "5s") |

### Examples

```bash
# Check local daemon
fabrik health

# Check remote server
fabrik health --url https://cache.example.com:8888/health

# With custom timeout
fabrik health --timeout 10s
```

## Global Options

Available for all commands:

| Option | Description |
|--------|-------------|
| `--help`, `-h` | Show help information |
| `--version`, `-V` | Show version information |
| `--verbose`, `-v` | Enable verbose logging |
| `--quiet`, `-q` | Suppress non-error output |

## Environment Variables

### Authentication

- `TUIST_TOKEN` - Shorthand for authentication token
- `TUIST_CONFIG_AUTH_TOKEN` - Full form for authentication token

### AWS Credentials (for S3 upstream)

- `AWS_ACCESS_KEY_ID` - AWS access key
- `AWS_SECRET_ACCESS_KEY` - AWS secret key
- `AWS_REGION` - AWS region

### Configuration Prefix

Any configuration option can be set via `TUIST_CONFIG_*` environment variables:

```bash
export TUIST_CONFIG_CACHE_DIR=/tmp/cache
export TUIST_CONFIG_CACHE_MAX_SIZE=10GB
export TUIST_CONFIG_UPSTREAM_0_URL=grpc://cache.example.com:7070
```

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Authentication error |
| 130 | Interrupted by user (Ctrl+C) |
