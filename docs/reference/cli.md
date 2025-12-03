# CLI Reference

Complete reference for all Fabrik CLI commands.

## Overview

Fabrik uses an activation-based approach for managing build caches. Instead of wrapping your build commands, Fabrik runs as a background daemon that your build tools connect to automatically.

**Two main workflows:**

1. **Shell Integration** (`fabrik activate`) - Automatic daemon management for development
2. **Explicit Execution** (`fabrik exec`) - Manual daemon management for CI/CD

See the [Getting Started Guide](/getting-started) for detailed usage examples.

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

[server]
layer = "regional"
bind = "0.0.0.0:7070"  # gRPC server for Fabrik protocol

[observability]
metrics_bind = "0.0.0.0:9091"
health_bind = "0.0.0.0:8888"
```

See [Architecture](/guide/architecture) for complete server setup.

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

## `fabrik run`

Execute scripts with automatic caching based on KDL annotations.

### Usage

```bash
# Execute script with caching
fabrik run <SCRIPT> [-- SCRIPT_ARGS...]

# Script management operations
fabrik run --status <SCRIPT>    # Check cache status
fabrik run --list               # List all cached scripts
fabrik run --stats              # Show cache statistics
```

### Options

| Option | Description |
|--------|-------------|
| `--status` | Check cache status for a script |
| `--list` | List all cached scripts |
| `--stats` | Show script cache statistics |
| `--no-cache` | Force execution without checking cache |
| `--clean` | Remove cached outputs before running |
| `--dry-run` | Show what would happen without executing |
| `--cache-only` | Fail if cache miss (for CI validation) |
| `--verbose`, `-v` | Verbose output |

### Examples

```bash
# Execute script with caching
fabrik run build.sh

# Check cache status
fabrik run --status build.sh

# List all cached scripts
fabrik run --list

# Show cache statistics
fabrik run --stats

# Force re-execution
fabrik run --no-cache build.sh

# Clean cache and re-run
fabrik run --clean build.sh
```

See [Standard Recipes Documentation](/cache/recipes/standard/) for details on FABRIK annotations and script caching.

## `fabrik cas`

Content-Addressed Storage operations for blob storage.

CAS operations work with content hashes (SHA256) to store and retrieve arbitrary binary data.

### Commands

```bash
# Get a blob by hash
fabrik cas get <HASH> [--output <FILE>]

# Store a file (returns hash)
fabrik cas put <FILE> [--hash <EXPECTED_HASH>]

# Check if blob exists
fabrik cas exists <HASH>

# Delete a blob
fabrik cas delete <HASH> [--force]

# Show blob information
fabrik cas info <HASH>

# List all blobs
fabrik cas list [--verbose]

# Show storage statistics
fabrik cas stats
```

### Examples

```bash
# Store a file in CAS
fabrik cas put myfile.bin
# Output: abc123def456... (hash)

# Retrieve blob by hash
fabrik cas get abc123def456... --output restored.bin

# Check if blob exists
fabrik cas exists abc123def456...

# Get blob information
fabrik cas info abc123def456...

# List all blobs
fabrik cas list --verbose

# Show CAS statistics
fabrik cas stats

# Delete a blob
fabrik cas delete abc123def456... --force
```

### JSON Output

Most commands support `--json` flag for machine-readable output:

```bash
fabrik cas put file.bin --json
# {"hash":"abc123...","size_bytes":1024,"success":true}

fabrik cas get abc123... --output file.bin --json
# {"hash":"abc123...","output_path":"file.bin","size_bytes":1024,"success":true}
```

## `fabrik kv`

Key-Value storage operations for action cache and metadata.

KV operations use arbitrary string keys (not content hashes) to store and retrieve data.

### Commands

```bash
# Get value by key
fabrik kv get <KEY> [--output <FILE>]

# Store key-value pair
fabrik kv put <KEY> <VALUE>
fabrik kv put <KEY> --file <FILE>

# Check if key exists
fabrik kv exists <KEY>

# Delete key-value pair
fabrik kv delete <KEY> [--force]

# List all keys
fabrik kv list [--prefix <PREFIX>]

# Show storage statistics
fabrik kv stats
```

### Examples

```bash
# Store a value
fabrik kv put build-result "success"

# Store from file
fabrik kv put build-metadata --file metadata.json

# Retrieve value
fabrik kv get build-result

# Retrieve to file
fabrik kv get build-metadata --output metadata.json

# Check if key exists
fabrik kv exists build-result

# List all keys
fabrik kv list

# List keys with prefix
fabrik kv list --prefix build-

# Show KV statistics
fabrik kv stats

# Delete a key
fabrik kv delete build-result --force
```

### JSON Output

All commands support `--json` flag:

```bash
fabrik kv put mykey "myvalue" --json
# {"key":"mykey","value_bytes":7,"success":true}

fabrik kv list --json
# [{"key":"build-result"},{"key":"build-metadata"}]

fabrik kv stats --json
# {"total_keys":10,"total_bytes":5242880}
```

### Use Cases

**Action Cache**: Store build results keyed by input hash
```bash
# Bazel/Gradle-style action cache
INPUT_HASH=$(sha256sum inputs.txt | cut -d' ' -f1)
fabrik kv put "action:$INPUT_HASH" --file result.json
```

**Build Metadata**: Store timestamps, versions, etc.
```bash
fabrik kv put "last-build-time" "$(date -Iseconds)"
fabrik kv put "app-version" "1.2.3"
```

## `fabrik p2p`

Manage peer-to-peer cache sharing on local networks.

P2P cache sharing (Layer 0.5) allows automatic discovery and sharing of build caches between machines on the same network, with 1-5ms latency.

### Commands

```bash
# Generate a secure random secret
fabrik p2p secret [--length <BYTES>]

# List discovered peers
fabrik p2p list [--verbose] [--json]

# Show P2P status
fabrik p2p status [--json]

# Approve a peer (grant cache access)
fabrik p2p approve <PEER> [--permanent]

# Deny a peer (revoke cache access)
fabrik p2p deny <PEER>

# Clear all consent records
fabrik p2p clear [--force]
```

### Examples

```bash
# Generate a secure random secret (default: 32 bytes = 64 hex chars)
fabrik p2p secret
# Output: 2295b4779c0fee78a732f249a32e25a03b7b3329db51719058b56aabae426d43

# Generate shorter secret (minimum 16 bytes for security)
fabrik p2p secret --length 16
# Output: fc5d669b4220c90e3ac0e48c3c8fcaac

# Generate and save secret to environment
export P2P_SECRET=$(fabrik p2p secret)

# Create config that uses the environment variable
cat >> .fabrik.toml <<EOF
[p2p]
enabled = true
secret = "\${P2P_SECRET}"  # Uses environment variable
consent_mode = "notify-once"
EOF

# Start daemon with P2P enabled
fabrik daemon start

# List discovered peers
fabrik p2p list
# Output:
# [fabrik] Discovered 2 peer(s):
#
#   • alice-macbook (192.168.1.100:7071)
#   • bob-desktop (192.168.1.101:7071)

# List peers with details
fabrik p2p list --verbose
# Output:
# [fabrik] Discovered 2 peer(s):
#
#   • alice-macbook (192.168.1.100:7071)
#     Machine ID: a3f5d9c2b1e8f7a4
#     Port: 7071
#     Accepting requests: true
#
#   • bob-desktop (192.168.1.101:7071)
#     Machine ID: b7e4a1f9c8d2e3f6
#     Port: 7071
#     Accepting requests: true

# Show P2P status
fabrik p2p status
# Output:
# [fabrik] P2P Cache Sharing Status
#
#   Enabled: true
#   Advertise: true
#   Discovery: true
#   Port: 7071
#   Consent mode: notify-once
#   Max peers: 10
#
#   Peers discovered: 2

# Approve peer permanently
fabrik p2p approve alice-macbook --permanent
# Output: [fabrik] Permanently approved peer: alice-macbook

# Approve peer for current session only
fabrik p2p approve bob-desktop
# Output: [fabrik] Approved peer for this session: bob-desktop

# Deny a peer
fabrik p2p deny charlie-laptop
# Output: [fabrik] Denied peer: charlie-laptop

# Clear all consent records
fabrik p2p clear
# Output:
# [fabrik] This will clear all stored P2P consents.
# [fabrik] You will need to re-approve peers next time they request access.
# [fabrik] Continue? [y/N] y
# [fabrik] Cleared all P2P consents

# Force clear without confirmation
fabrik p2p clear --force
```

### JSON Output

All P2P commands support `--json` for machine-readable output:

```bash
fabrik p2p list --json
# [
#   {
#     "machine_id": "a3f5d9c2b1e8f7a4",
#     "hostname": "alice-macbook",
#     "address": "192.168.1.100",
#     "port": 7071,
#     "accepting_requests": true
#   },
#   {
#     "machine_id": "b7e4a1f9c8d2e3f6",
#     "hostname": "bob-desktop",
#     "address": "192.168.1.101",
#     "port": 7071,
#     "accepting_requests": true
#   }
# ]

fabrik p2p status --json
# {
#   "enabled": true,
#   "advertise": true,
#   "discovery": true,
#   "bind_port": 7071,
#   "consent_mode": "notify-once",
#   "peers_discovered": 2,
#   "max_peers": 10
# }
```

### Configuration

P2P must be enabled in your `.fabrik.toml`:

```toml
[p2p]
enabled = true
secret = "${P2P_SECRET}"        # Use env var (min 16 chars, shared across team)
consent_mode = "notify-once"    # notify-once | notify-always | always-allow
bind_port = 7071                # Port for P2P server (default: 7071)
advertise = true                # Advertise this machine to peers
discovery = true                # Discover other peers
max_peers = 10                  # Maximum number of peers to connect to
```

**Generate and set the secret:**

```bash
# Generate a secure secret
fabrik p2p secret

# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.)
export P2P_SECRET=<generated-secret>
```

### Consent Modes

- `notify-once` - System notification on first access, remembered
- `notify-always` - System notification every time
- `always-allow` - No notifications, always allow (use with caution)

### Security

- All P2P communication authenticated via HMAC-SHA256 with shared secret
- Replay protection with 5-minute time window
- User consent required before cache access
- Consent records stored in `~/.local/share/fabrik/p2p/consents.json`

### Use Cases

**Team collaboration:**
```bash
# Same office network
# Developer A builds feature → Developer B instantly gets cached artifacts
# 1-5ms latency vs 20-50ms from cloud cache
```

**Multi-machine development:**
```bash
# MacBook + Linux desktop on same home network
# Build on one machine → cache available on other
```

**CI/CD optimization:**
```bash
# Multiple CI runners on same LAN
# First runner builds → subsequent runners use P2P cache
# Reduces cloud cache bandwidth costs
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
