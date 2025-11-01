# Fabrik Activation-Based Architecture

## Overview

Drawing inspiration from Mise, Fabrik uses an activation-based approach for managing cache daemons, avoiding the need to wrap build tool commands.

## Commands

### `fabrik activate`

**Purpose**: Shell integration for automatic daemon management

**Behavior**:
1. Traverses directory tree upward to find `.fabrik.toml`
2. Computes configuration hash
3. Checks if daemon with that config is already running
4. If not running, starts daemon as background process
5. Exports environment variables for build tools to consume
6. Cleans up daemons from previous directories (optional)

**Shell Integration**:
```bash
# In ~/.bashrc or ~/.zshrc
eval "$(fabrik activate bash)"  # or zsh, fish
```

**Generated Hook** (example):
```bash
_fabrik_hook() {
  eval "$(fabrik activate --status)"
}

# Run on directory change
if [[ -n "${ZSH_VERSION}" ]]; then
  chpwd_functions+=(_fabrik_hook)
elif [[ -n "${BASH_VERSION}" ]]; then
  PROMPT_COMMAND="_fabrik_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
fi
```

### `fabrik exec <command>`

**Purpose**: Execute command with guaranteed daemon lifecycle

**Behavior**:
1. Finds `.fabrik.toml` in current directory tree
2. Starts daemon if not running
3. Sets environment variables
4. Executes command
5. **Optionally** kills daemon when command exits (configurable)

**Usage**:
```bash
fabrik exec bazel build //...
fabrik exec nx build demo
fabrik exec gradle build
```

**Lifecycle Options**:
- `--keep-alive` - Don't kill daemon after command exits (default)
- `--kill-after` - Kill daemon when command completes

## Configuration Discovery

**Search Path**:
1. `$PWD/.fabrik.toml`
2. `$PWD/../.fabrik.toml`
3. Continue up to root
4. Fallback to `~/.config/fabrik/config.toml` (global)

**Config Hash**:
- SHA256 of canonical config content
- Used to uniquely identify daemon instances
- Different configs = different daemons

## Daemon Management

### Daemon State Directory

```
~/.fabrik/daemons/<config-hash>/
├── pid              # Process ID (for signal sending)
├── ports.json       # HTTP/gRPC ports allocated
├── env              # Environment variables to export
├── config.toml      # Resolved configuration
└── socket           # Unix socket path (alternative to HTTP)
```

### Daemon Lifecycle

1. **Start**: `fabrik activate` or `fabrik exec` starts daemon
2. **Health Check**: Periodic HTTP GET to `/health`
3. **Stop**: 
   - `fabrik deactivate` (explicit)
   - `fabrik activate` in directory with different config (optional cleanup)
   - `fabrik exec --kill-after` (when command exits)

### Port Allocation

- **Random ports** assigned on daemon start
- Stored in `~/.fabrik/daemons/<hash>/ports.json`:
```json
{
  "http": 58234,
  "grpc": 58235,
  "metrics": 58236
}
```

## Environment Variables

### Standard Variables (Always Exported)

```bash
FABRIK_HTTP_URL=http://127.0.0.1:58234
FABRIK_GRPC_URL=grpc://127.0.0.1:58235
FABRIK_CONFIG_HASH=abc123def456
FABRIK_DAEMON_PID=12345
```

### Build Tool Convenience Variables (Optional)

```bash
# Gradle
GRADLE_BUILD_CACHE_URL=$FABRIK_HTTP_URL

# Nx
NX_SELF_HOSTED_REMOTE_CACHE_SERVER=$FABRIK_HTTP_URL

# Bazel (requires .bazelrc usage since no env var support)
# Users add to .bazelrc:
# build --remote_cache=$FABRIK_GRPC_URL

# Xcode (custom)
XCODE_CACHE_SERVER=$FABRIK_HTTP_URL
```

### Authentication

If `.fabrik.toml` contains `auth.token`, export:
```bash
FABRIK_TOKEN=eyJ0eXAi...
```

Build tools can then use:
```bash
# Gradle (via init script or gradle.properties)
# Nx (via nx.json or NX_CLOUD_AUTH_TOKEN)
# etc.
```

## Shell Integration Output

When `fabrik activate` runs, it outputs shell commands to eval:

```bash
$ fabrik activate --status
export FABRIK_HTTP_URL=http://127.0.0.1:58234
export FABRIK_GRPC_URL=grpc://127.0.0.1:58235
export FABRIK_CONFIG_HASH=abc123
export GRADLE_BUILD_CACHE_URL=http://127.0.0.1:58234
export NX_SELF_HOSTED_REMOTE_CACHE_SERVER=http://127.0.0.1:58234
# Started daemon with PID 12345
```

## User Workflows

### Workflow 1: Shell Activation (Automatic)

```bash
# Setup (once)
echo 'eval "$(fabrik activate bash)"' >> ~/.bashrc

# Usage (automatic on cd)
cd ~/my-project          # Daemon starts automatically
bazel build //...        # Uses FABRIK_GRPC_URL from env
nx build demo            # Uses NX_SELF_HOSTED_REMOTE_CACHE_SERVER
gradle build             # Uses GRADLE_BUILD_CACHE_URL

cd ~/other-project       # Different daemon starts
cd ~                     # Daemons cleaned up (optional)
```

### Workflow 2: Explicit Execution (CI-friendly)

```bash
# CI or one-off builds
fabrik exec bazel build //...
fabrik exec nx build demo
fabrik exec gradle build

# Keeps daemon alive for subsequent commands
fabrik exec --keep-alive nx build demo
nx test demo  # Reuses daemon
fabrik deactivate  # Explicit cleanup
```

### Workflow 3: Long-running Daemon

```bash
# Start daemon explicitly
fabrik daemon start

# Use in multiple terminals/commands
bazel build //...  # Terminal 1
nx build demo      # Terminal 2

# Stop when done
fabrik daemon stop
```

## Implementation Phases

### Phase 1: Basic Daemon Management
- [ ] Config discovery (traverse directories)
- [ ] Config hashing
- [ ] Daemon start/stop
- [ ] PID/port file management
- [ ] Health checks

### Phase 2: Activation Command
- [ ] `fabrik activate --status` (check/start daemon)
- [ ] Environment variable generation
- [ ] Shell integration (`fabrik activate bash/zsh/fish`)

### Phase 3: Exec Command
- [ ] `fabrik exec <command>`
- [ ] Environment variable passing
- [ ] Lifecycle management (--keep-alive, --kill-after)

### Phase 4: Daemon Cleanup
- [ ] Orphan daemon detection
- [ ] Automatic cleanup on `activate` (optional)
- [ ] `fabrik daemon list/stop/clean` commands

## Benefits Over Wrapper Approach

1. **No Command Wrapping**: Build tools run as-is
2. **Flexible**: Works with shell activation OR explicit exec
3. **CI-Friendly**: `fabrik exec` for CI, `fabrik activate` for dev
4. **Multi-Tool**: One daemon serves all build tools
5. **Configuration-Driven**: Different projects, different daemons
6. **Clean State**: Daemons tracked and cleanable

## Migration from Wrapper Approach

Old approach:
```bash
fabrik bazel -- build //...
fabrik nx -- build demo
```

New approach (shell activation):
```bash
eval "$(fabrik activate bash)"
bazel build //...  # Just works
nx build demo      # Just works
```

New approach (explicit):
```bash
fabrik exec bazel build //...
fabrik exec nx build demo
```
