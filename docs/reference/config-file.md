# Configuration File Reference

Complete reference for the `.fabrik.toml` configuration file.

## File Location

- **Project config**: `.fabrik.toml` in project root
- **System config**: `/etc/fabrik/config.toml`
- **Custom**: Specify with `--config <path>`

## Configuration Precedence

1. Command-line arguments (highest)
2. Environment variables
3. Configuration file (lowest)

## Complete Example

```toml
[settings]
lockfile = true

[cache]
dir = "/data/fabrik/cache"
max_size = "100GB"
eviction_policy = "lfu"  # lru | lfu | ttl
default_ttl = "7d"

[[upstream]]
url = "grpc://cache-us-east.example.com:7070"
timeout = "10s"
read_only = false
permanent = false

[[upstream]]
url = "s3://my-build-cache/project-name/"
timeout = "60s"
permanent = true
write_through = true
workers = 20
region = "us-east-1"

[auth]
public_key_file = "/etc/fabrik/jwt-public-key.pem"
key_refresh_interval = "5m"
required = true

[p2p]
enabled = true
secret = "my-team-secret-2024-min-16-chars"
consent_mode = "notify-once"  # notify-once | notify-always | always-allow
bind_port = 7071
advertise = true
discovery = true
max_peers = 10

[build_systems]
enabled = ["gradle", "bazel", "nx", "turborepo", "sccache"]

[build_systems.gradle]
port = 0
auto_configure = true

[fabrik]
enabled = false
bind = "0.0.0.0:7070"

[observability]
log_level = "info"
log_format = "json"
metrics_bind = "0.0.0.0:9091"
health_bind = "0.0.0.0:8888"
metrics_enabled = true
cache_query_api_enabled = true
admin_api_enabled = false

[runtime]
graceful_shutdown_timeout = "30s"
max_concurrent_requests = 10000
worker_threads = 0
```

## Section Reference

### `[settings]`

Global settings.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `lockfile` | boolean | `false` | Enable lockfile for cache operations |

### `[cache]`

Local cache storage configuration.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `dir` | string | `.fabrik/cache` | Cache directory path |
| `max_size` | string | `10GB` | Maximum cache size (e.g., "10GB", "500MB") |
| `eviction_policy` | string | `lfu` | Eviction policy: `lru`, `lfu`, or `ttl` |
| `default_ttl` | string | `7d` | Default TTL for cached items (e.g., "7d", "24h") |

### `[[upstream]]`

Upstream cache layers (array, can be specified multiple times).

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `url` | string | *required* | Upstream URL (grpc://, s3://, https://) |
| `timeout` | string | `30s` | Request timeout |
| `read_only` | boolean | `false` | If true, never write to this upstream |
| `permanent` | boolean | `false` | If true, never evict from this upstream |
| `write_through` | boolean | `false` | Write immediately to this upstream |
| `workers` | number | `10` | Concurrent upload workers (S3 only) |
| `region` | string | - | AWS region (S3 only) |
| `endpoint` | string | - | Custom S3 endpoint (S3 only) |
| `access_key` | string | - | AWS access key (or use `AWS_ACCESS_KEY_ID` env) |
| `secret_key` | string | - | AWS secret key (or use `AWS_SECRET_ACCESS_KEY` env) |

### `[auth]`

Authentication configuration for server mode.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `public_key_file` | string | - | Path to JWT public key (RS256) |
| `key_refresh_interval` | string | `5m` | How often to reload public key |
| `required` | boolean | `false` | Require authentication for all requests |

### `[p2p]`

> [!IMPORTANT]
> P2P cache sharing (Layer 0.5) enables automatic discovery and sharing of build caches across local networks with 1-5ms latency.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable P2P cache sharing |
| `secret` | string | *required* | Shared secret for HMAC authentication (min 16 chars) |
| `consent_mode` | string | `notify-once` | User consent mode: `notify-once`, `notify-always`, `always-allow` |
| `bind_port` | number | `7071` | Port for P2P gRPC server |
| `advertise` | boolean | `true` | Advertise this machine to peers via mDNS |
| `discovery` | boolean | `true` | Discover other peers via mDNS |
| `max_peers` | number | `10` | Maximum number of peers to connect to |

**Example:**
```toml
[p2p]
enabled = true
secret = "my-team-secret-2024-fabrik"
consent_mode = "notify-once"
bind_port = 7071
advertise = true
discovery = true
max_peers = 10
```

**Security Notes:**
- All P2P communication is authenticated via HMAC-SHA256
- Secret must be at least 16 characters
- Secret should be shared securely across team (e.g., via 1Password, team config)
- Replay protection with 5-minute time window
- User consent required before cache access (except in `always-allow` mode)

**Consent Modes:**
- `notify-once`: System notification on first access from each peer, remembered permanently
- `notify-always`: System notification on every cache request (secure but annoying)
- `always-allow`: No notifications, always allow (⚠️ use only on trusted networks)

### `[build_systems]`

Build system adapter configuration (Layer 1 only).

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | array | `[]` | Enabled build systems: `gradle`, `bazel`, `nx`, `turborepo`, etc. |

**Per-adapter configuration:**

```toml
[build_systems.gradle]
port = 0              # 0 = random port (recommended)
auto_configure = true # Auto-set GRADLE_BUILD_CACHE_URL
```

### `[fabrik]`

Fabrik protocol server configuration (Layer 2 only).

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable Fabrik protocol gRPC server |
| `bind` | string | `0.0.0.0:7070` | gRPC bind address |

**Note:** Enable this for Layer 2 servers. Layer 1 (local daemons) should keep this disabled.

### `[observability]`

Metrics and monitoring configuration.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `log_level` | string | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `log_format` | string | `text` | Log format: `text` or `json` |
| `metrics_bind` | string | `0.0.0.0:9091` | Prometheus metrics endpoint |
| `health_bind` | string | `0.0.0.0:8888` | Health check endpoint |
| `metrics_enabled` | boolean | `true` | Enable Prometheus metrics |
| `cache_query_api_enabled` | boolean | `false` | Enable cache query API (for Tuist Dashboard) |
| `admin_api_enabled` | boolean | `false` | Enable admin API (management operations) |

### `[runtime]`

Runtime behavior configuration.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `graceful_shutdown_timeout` | string | `30s` | How long to wait for in-flight requests on shutdown |
| `max_concurrent_requests` | number | `10000` | Maximum concurrent requests |
| `worker_threads` | number | `0` | Worker thread count (0 = auto, num CPUs) |

## Environment Variable Overrides

All configuration options can be overridden via environment variables using the `TUIST_CONFIG_*` prefix:

```bash
# Override cache directory
export TUIST_CONFIG_CACHE_DIR=/tmp/fabrik-cache

# Override upstream
export TUIST_CONFIG_UPSTREAM_0_URL=grpc://cache.example.com:7070

# Override P2P secret
export TUIST_CONFIG_P2P_SECRET="my-secret-from-env"

# Standard AWS credentials (fallback)
export AWS_ACCESS_KEY_ID=AKIA...
export AWS_SECRET_ACCESS_KEY=secret...
export AWS_REGION=us-east-1
```

**Naming convention:**
- Nested: `cache.dir` → `TUIST_CONFIG_CACHE_DIR`
- Arrays: `upstream[0].url` → `TUIST_CONFIG_UPSTREAM_0_URL`
- All uppercase, underscores separate words
