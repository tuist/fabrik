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

Refer to the complete configuration options above for detailed explanations of each section.
