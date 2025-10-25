# CLI Reference

Complete reference for all Fabrik CLI commands.

## `fabrik bazel`

Wrapper for Bazel with automatic remote cache configuration.

### Usage

```bash
fabrik bazel [OPTIONS] -- <BAZEL_ARGS>...
```

### Options

| Option | Environment Variable | Description |
|--------|---------------------|-------------|
| `--config <PATH>` | - | Path to configuration file |
| `--config-cache-dir <DIR>` | `TUIST_CONFIG_CACHE_DIR` | Cache directory path |
| `--config-cache-max-size <SIZE>` | `TUIST_CONFIG_CACHE_MAX_SIZE` | Maximum cache size (e.g., "10GB") |
| `--config-upstream <URL>` | `TUIST_CONFIG_UPSTREAM_0_URL` | Upstream cache URL |
| `--config-jwt-token <TOKEN>` | `TUIST_CONFIG_AUTH_TOKEN` or `TUIST_TOKEN` | JWT authentication token |
| `--config-bazel-port <PORT>` | `FABRIK_CONFIG_BAZEL_PORT` | Bazel gRPC server port (0 = random) |

### Examples

```bash
# Basic usage (local cache only)
fabrik bazel -- build //...

# Build specific target
fabrik bazel -- build //src:myapp

# Run tests
fabrik bazel -- test //...

# Build with Bazel configuration
fabrik bazel -- build //... --config=release --jobs=8

# With upstream cache
fabrik bazel --config-upstream grpc://cache.example.com:7070 -- build //...

# With authentication
fabrik bazel \
  --config-upstream grpc://cache.example.com:7070 \
  --config-jwt-token $TUIST_TOKEN \
  -- build //...

# Using configuration file
fabrik bazel --config .fabrik.toml -- build //...

# Custom cache directory and port
fabrik bazel \
  --config-cache-dir /tmp/bazel-cache \
  --config-bazel-port 9090 \
  -- build //...
```

### How It Works

The `fabrik bazel` command:
1. Starts a local gRPC server implementing the Bazel Remote Caching protocol
2. Automatically injects `--remote_cache=grpc://localhost:{port}` flag
3. Passes through all other Bazel arguments unchanged
4. Handles graceful shutdown when Bazel exits

See the [Bazel integration guide](/build-systems/bazel) for more details.

## `fabrik exec`

Wrap a command with ephemeral cache (hot cache for CI/local builds).

### Usage

```bash
fabrik exec [OPTIONS] -- <COMMAND> [ARGS...]
```

### Options

| Option | Environment Variable | Description |
|--------|---------------------|-------------|
| `--config <PATH>` | - | Path to configuration file |
| `--config-cache-dir <DIR>` | `TUIST_CONFIG_CACHE_DIR` | Cache directory path |
| `--config-cache-max-size <SIZE>` | `TUIST_CONFIG_CACHE_MAX_SIZE` | Maximum cache size (e.g., "10GB") |
| `--config-upstream <URL>` | `TUIST_CONFIG_UPSTREAM_0_URL` | Upstream cache URL |
| `--config-jwt-token <TOKEN>` | `TUIST_CONFIG_AUTH_TOKEN` or `TUIST_TOKEN` | JWT authentication token |
| `--no-auto-configure` | - | Disable automatic build system configuration |
| `--dry-run` | - | Show what would be configured without executing |

### Examples

```bash
# Basic usage
fabrik exec -- ./gradlew build

# With upstream cache
fabrik exec --config-upstream grpc://cache.example.com:7070 -- ./gradlew build

# With authentication
fabrik exec \
  --config-upstream grpc://cache.example.com:7070 \
  --config-jwt-token $TUIST_TOKEN \
  -- ./gradlew build

# Using configuration file
fabrik exec --config .fabrik.toml -- npm run build

# Dry run to see what would be configured
fabrik exec --dry-run -- ./gradlew build
```

## `fabrik daemon`

Run a long-lived local cache daemon (hot cache for development).

### Usage

```bash
fabrik daemon [OPTIONS]
```

### Options

Same as `fabrik exec`, plus:

| Option | Description |
|--------|-------------|
| `--bind <ADDRESS>` | Address to bind daemon to (default: "127.0.0.1:7070") |

### Examples

```bash
# Start daemon with default settings
fabrik daemon

# Start with custom configuration
fabrik daemon --config .fabrik.toml

# Start on custom address
fabrik daemon --bind 0.0.0.0:8080
```

## `fabrik server`

Run a regional/cloud cache server (warm/cold cache).

### Usage

```bash
fabrik server [OPTIONS]
```

### Options

Same as `fabrik exec`, plus:

| Option | Description |
|--------|-------------|
| `--config-fabrik-enabled` | Enable Fabrik protocol server |
| `--config-fabrik-bind <ADDRESS>` | Fabrik protocol bind address |
| `--config-metrics-bind <ADDRESS>` | Metrics API bind address |
| `--config-health-bind <ADDRESS>` | Health API bind address |

### Examples

```bash
# Start server with configuration file
fabrik server --config /etc/fabrik/config.toml

# Start with environment variables
export TUIST_CONFIG_CACHE_DIR=/data/cache
export TUIST_CONFIG_FABRIK_ENABLED=true
fabrik server
```

## `fabrik config`

Configuration utilities (validate, generate, show).

### Usage

```bash
fabrik config <SUBCOMMAND> [OPTIONS]
```

### Subcommands

#### `validate`

Validate a configuration file.

```bash
fabrik config validate <PATH>

# Example:
fabrik config validate .fabrik.toml
```

#### `generate`

Generate example configuration.

```bash
fabrik config generate --template <TEMPLATE>

# Templates:
#   exec   - Configuration for fabrik exec/daemon
#   server - Configuration for fabrik server

# Examples:
fabrik config generate --template exec > .fabrik.toml
fabrik config generate --template server > server.toml
```

#### `show`

Show effective configuration (merged from all sources).

```bash
fabrik config show [OPTIONS]

# Examples:
fabrik config show
fabrik config show --config .fabrik.toml
fabrik config show --config-upstream grpc://override.example.com
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
