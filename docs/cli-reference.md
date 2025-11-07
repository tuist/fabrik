# CLI Reference

Complete reference for Fabrik's command-line interface.

## Global Options

```
fabrik [OPTIONS] <COMMAND>

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Commands

### `fabrik activate`

Generate shell integration hook or check daemon status.

```bash
fabrik activate <SHELL>     # Generate shell hook
fabrik activate --status    # Check/start daemon and export env vars
```

**Arguments:**
- `<SHELL>` - Shell type: `bash`, `zsh`, or `fish`

**Flags:**
- `--status` - Check daemon status and start if needed

**Examples:**

```bash
# Generate shell integration for bash
eval "$(fabrik activate bash)"

# Check daemon status (used by shell hook)
fabrik activate --status
```

**What it does:**
- **Without flags**: Outputs shell integration code to stdout
- **With `--status`**: 
  1. Detects `fabrik.toml` in current directory (walks up tree)
  2. Computes config hash
  3. Checks if daemon is running for this config
  4. If not running: spawns daemon in background
  5. Exports environment variables with daemon ports

**Environment variables exported:**
- `FABRIK_HTTP_URL` - HTTP server URL (e.g., `http://127.0.0.1:54321`)
- `FABRIK_GRPC_URL` - gRPC server URL (e.g., `grpc://127.0.0.1:54322`)
- `FABRIK_CONFIG_HASH` - Config file hash
- `FABRIK_DAEMON_PID` - Daemon process ID
- `GRADLE_BUILD_CACHE_URL` - For Gradle
- `NX_SELF_HOSTED_REMOTE_CACHE_SERVER` - For Nx
- `XCODE_CACHE_SERVER` - For Xcode

---

### `fabrik init`

Initialize Fabrik configuration for a project.

```bash
fabrik init [OPTIONS]
```

**Options:**
- `--non-interactive` - Skip interactive prompts and use defaults
- `--cache-dir <DIR>` - Cache directory (default: `.fabrik/cache`)
- `--max-cache-size <SIZE>` - Max cache size (default: `5GB`)
- `--upstream-url <URL>` - Upstream cache URL (optional)

**Examples:**

```bash
# Interactive initialization (recommended)
fabrik init

# Non-interactive with defaults
fabrik init --non-interactive

# Non-interactive with custom values
fabrik init --non-interactive \
  --cache-dir /tmp/cache \
  --max-cache-size 10GB \
  --upstream-url grpc://cache.tuist.io:7070
```

**What it does:**
1. Checks if `fabrik.toml` already exists (prompts to overwrite)
2. Asks for configuration values interactively:
   - Cache directory location
   - Maximum cache size
   - Whether you have a remote cache server
   - Remote cache URL (if applicable)
3. Generates `fabrik.toml` in current directory
4. Shows configuration summary
5. Displays next steps

**Interactive prompts:**
```
Cache directory [.fabrik/cache]:
Max cache size [5GB]:
Do you have a remote cache server? [y/N]
Remote cache URL (e.g., grpc://cache.tuist.io:7070):
```

---

### `fabrik daemon`

Manually start a cache daemon.

```bash
fabrik daemon [OPTIONS]
```

**Options:**
- `-c, --config <PATH>` - Path to config file
- `--config-cache-dir <DIR>` - Cache directory
- `--config-max-cache-size <SIZE>` - Max cache size (e.g., "5GB")
- `--config-upstream <URL>` - Upstream cache URLs (comma-separated)
- `--config-http-port <PORT>` - HTTP server port (0 = random)
- `--config-grpc-port <PORT>` - gRPC server port (0 = random)
- `--config-log-level <LEVEL>` - Log level (trace|debug|info|warn|error)

**Examples:**

```bash
# Start daemon with config file
fabrik daemon --config fabrik.toml

# Start daemon with CLI options
fabrik daemon --config-cache-dir /tmp/cache --config-http-port 0
```

**What it does:**
1. Loads configuration from file and/or CLI options
2. Binds HTTP server to port 0 (or specified port)
3. Binds gRPC server to port 0 (or specified port)
4. Writes state to `~/.fabrik/daemons/{config_hash}/`
5. Starts servers and waits for shutdown signal (Ctrl+C or SIGTERM)
6. On shutdown: waits for in-flight requests, then cleans up state

---

### `fabrik deactivate`

Deactivate Fabrik and optionally stop the daemon.

```bash
fabrik deactivate [OPTIONS]
```

**Options:**
- `--stop-daemon` - Also stop the running daemon

**Examples:**

```bash
# Unset environment variables
fabrik deactivate

# Unset env vars and stop daemon
fabrik deactivate --stop-daemon
```

---

### `fabrik doctor`

Check system configuration and shell integration.

```bash
fabrik doctor [OPTIONS]
```

**Options:**
- `-v, --verbose` - Show verbose output

**Examples:**

```bash
# Quick check
fabrik doctor

# Detailed check
fabrik doctor --verbose
```

**What it checks:**
- ✅ Fabrik binary exists and is accessible
- ✅ Shell detected (bash, zsh, fish)
- ✅ Shell integration configured (checks rc file)
- ✅ State directory exists
- ✅ `fabrik.toml` in current directory
- ✅ Daemon running for current config
- ✅ Environment variables set (verbose mode)

**Exit codes:**
- `0` - All checks passed
- `1` - Some checks failed

---

### `fabrik server`

Run a remote cache server (Layer 2).

```bash
fabrik server [OPTIONS]
```

**Options:**
- `-c, --config <PATH>` - Path to config file
- `--config-cache-dir <DIR>` - Cache directory
- `--config-max-cache-size <SIZE>` - Max cache size
- `--config-upstream <URL>` - Upstream cache URLs
- `--config-http-port <PORT>` - HTTP server port
- `--config-grpc-port <PORT>` - gRPC server port
- `--config-metrics-port <PORT>` - Metrics server port

**Examples:**

```bash
# Run server with config file
fabrik server --config /etc/fabrik/config.toml

# Run server on specific ports
fabrik server --config-http-port 8080 --config-grpc-port 9090
```

**Use case:**
- Deploy as a long-running service
- Shared cache for team members
- Regional cache instances

---

### `fabrik config`

Configuration management utilities.

```bash
fabrik config <SUBCOMMAND>
```

**Subcommands:**
- `validate` - Validate configuration file
- `generate` - Generate example configuration
- `show` - Show effective configuration

**Examples:**

```bash
# Validate config file
fabrik config validate fabrik.toml

# Generate example config
fabrik config generate --template=server > config.toml

# Show effective configuration
fabrik config show --config fabrik.toml
```

---

### `fabrik health`

Health check and diagnostics.

```bash
fabrik health [OPTIONS]
```

**Options:**
- `--url <URL>` - URL of Fabrik instance to check
- `--timeout <DURATION>` - Request timeout (default: 5s)
- `--format <FORMAT>` - Output format: text or json (default: text)

**Examples:**

```bash
# Check local daemon
fabrik health --url http://127.0.0.1:54321

# Check remote server
fabrik health --url https://cache.tuist.io --timeout 10s

# JSON output
fabrik health --url http://127.0.0.1:54321 --format json
```

---

## Configuration File

Fabrik can be configured via a `fabrik.toml` file:

```toml
# Cache settings
[cache]
dir = ".fabrik/cache"
max_size = "5GB"
eviction_policy = "lfu"  # lru, lfu, or ttl

# Upstream caches (optional)
[[upstream]]
url = "grpc://cache.tuist.io:7070"
timeout = "30s"

# Authentication (optional)
[auth]
token_file = ".fabrik.token"  # Path to JWT token file

# Daemon settings (optional)
[daemon]
http_port = 0  # 0 = random port (recommended)
grpc_port = 0
metrics_port = 9091

# Logging (optional)
[observability]
log_level = "info"
log_format = "json"
```

## Environment Variables

All configuration options can be set via environment variables with the `FABRIK_CONFIG_*` prefix:

| Config Option | Environment Variable |
|--------------|---------------------|
| `cache.dir` | `FABRIK_CONFIG_CACHE_DIR` |
| `cache.max_size` | `FABRIK_CONFIG_MAX_CACHE_SIZE` |
| `upstream[0].url` | `FABRIK_CONFIG_UPSTREAM_0_URL` |
| `auth.token` | `FABRIK_CONFIG_AUTH_TOKEN` or `FABRIK_TOKEN` |
| `daemon.http_port` | `FABRIK_CONFIG_HTTP_PORT` |
| `daemon.grpc_port` | `FABRIK_CONFIG_GRPC_PORT` |

## Configuration Precedence

Configuration is loaded in this order (highest to lowest priority):

1. **Command-line arguments** - `--config-*` flags
2. **Environment variables** - `FABRIK_CONFIG_*` variables
3. **Configuration file** - `fabrik.toml`

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Configuration error |
| `3` | Network error |
| `4` | Storage error |

## See Also

- [Build System Integration](./build-systems/) - Integration guides
- [CLAUDE.md](../CLAUDE.md) - Architecture documentation
- [README.md](../README.md) - Getting started guide
