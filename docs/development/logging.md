# Structured Logging

Fabrik uses structured logging via the `tracing` crate to provide consistent, parseable logs that are useful both for development and production monitoring.

## Log Formats

Fabrik supports three output formats:

### Pretty Format (Default for Development)

Human-readable format with colors:

```
2025-10-25T17:53:04.123456Z INFO(fabrik): service=xcode.cas operation=get status=success object_id=0164f7fc size_bytes=2048: cache hit
```

### Compact Format (CI/Production)

Single-line format without colors:

```
2025-10-25T17:53:04.123456Z INFO(fabrik): service=xcode.cas operation=get status=success object_id=0164f7fc size_bytes=2048: cache hit
```

### JSON Format (Log Aggregation)

Machine-parseable JSON for log aggregation systems:

```json
{
  "timestamp": "2025-10-25T17:53:04.123456Z",
  "level": "INFO",
  "fields": {
    "service": "xcode.cas",
    "operation": "get",
    "status": "success",
    "object_id": "0164f7fc2284e6b01b7627c72a733944fcf1daf933223673597dda442f3ab742",
    "size_bytes": 2048
  },
  "message": "cache hit"
}
```

## Configuring Log Format

Set the `FABRIK_LOG_FORMAT` environment variable:

```bash
# Pretty format (default)
FABRIK_LOG_FORMAT=pretty fabrik xcodebuild -- ...

# Compact format
FABRIK_LOG_FORMAT=compact fabrik xcodebuild -- ...

# JSON format
FABRIK_LOG_FORMAT=json fabrik xcodebuild -- ...
```

In CI environments, Fabrik automatically defaults to compact format when the `CI` environment variable is set.

## Configuring Log Level

Use the standard `RUST_LOG` environment variable:

```bash
# Info level (default)
RUST_LOG=info fabrik xcodebuild -- ...

# Debug level (verbose)
RUST_LOG=debug fabrik xcodebuild -- ...

# Only warnings and errors
RUST_LOG=warn fabrik xcodebuild -- ...
```

## Structured Fields

All logs use consistent structured fields for easy filtering and analysis:

### Standard Fields

| Field | Description | Example |
|-------|-------------|---------|
| `service` | Service name | `xcode.cas`, `bazel.cas`, `xcode.keyvalue` |
| `operation` | Operation type | `get`, `put`, `save`, `load` |
| `status` | Operation result | `success`, `miss`, `error` |
| `object_id` | Content hash (hex) | `0164f7fc2284e6b01b...` |
| `key` | Key (for keyvalue ops) | `abc123...` |
| `size_bytes` | Data size in bytes | `2048` |
| `entry_count` | Number of entries | `5` |
| `instance` | Bazel instance name | `main` |

### Service Names

- `xcode.cas` - Xcode Content-Addressable Storage
- `xcode.keyvalue` - Xcode Key-Value database
- `bazel.cas` - Bazel Content-Addressable Storage
- `bazel.action_cache` - Bazel Action Cache
- `bazel.bytestream` - Bazel ByteStream

### Operation Names

- `get` - Retrieve object/value
- `put` - Store object/value
- `save` - Save blob
- `load` - Load blob
- `find_missing` - Check for missing blobs
- `batch_update` - Batch upload
- `batch_read` - Batch download

### Status Values

- `success` - Operation succeeded (cache hit)
- `miss` - Cache miss (object not found)
- `error` - Operation failed
- `not_found` - Key/object not found

## Example Log Messages

### Cache Hit

```
2025-10-25T17:53:04.123456Z INFO(fabrik): service=xcode.cas operation=get status=success object_id=0164f7fc size_bytes=2048: cache hit
```

### Cache Miss

```
2025-10-25T17:53:04.234567Z INFO(fabrik): service=xcode.cas operation=get status=miss object_id=abc123: cache miss
```

### Cache Write

```
2025-10-25T17:53:04.345678Z INFO(fabrik): service=xcode.cas operation=put status=success object_id=0164f7fc size_bytes=2048: object stored
```

### Batch Operation

```
2025-10-25T17:53:04.456789Z INFO(fabrik): service=bazel.cas operation=find_missing status=success instance=main blob_count=50 missing_count=5: check completed
```

## Parsing Logs in Tests

The structured format makes it easy to parse logs in tests:

```rust
// Count cache hits
let hits = stdout
    .matches(r#"service="xcode.cas" operation="get" status="success""#)
    .count();

// Count cache misses
let misses = stdout
    .matches(r#"service="xcode.cas" operation="get" status="miss""#)
    .count();
```

## Adding New Logs

When adding new log statements, use the structured format:

```rust
use crate::logging::{services, operations, status};
use tracing::info;

// Cache hit
info!(
    service = services::XCODE_CAS,
    operation = operations::GET,
    status = status::SUCCESS,
    object_id = %hex::encode(&id),
    size_bytes = data.len(),
    "cache hit"
);

// Cache miss
info!(
    service = services::XCODE_CAS,
    operation = operations::GET,
    status = status::MISS,
    object_id = %hex::encode(&id),
    "cache miss"
);
```

### Field Formatting

- Use `=` for field assignment
- Use `%` prefix for `Display` formatting (hex strings, etc.)
- Use no prefix for primitive types (numbers, bools)
- The final string is the human-readable message

## Best Practices

1. **Always use structured fields** - Don't embed data in the message string
2. **Use constants from `logging` module** - Ensures consistency
3. **Log at appropriate levels**:
   - `debug!()` - Detailed tracing, disabled in production
   - `info!()` - Important events (cache hits/misses, operations)
   - `warn!()` - Recoverable errors, degraded performance
   - `error!()` - Serious errors requiring attention
4. **Keep messages concise** - "cache hit", "object stored", etc.
5. **Include relevant context** - object IDs, sizes, counts, etc.

## Production Monitoring

With JSON format, you can easily:

- **Filter by service**: `jq 'select(.fields.service == "xcode.cas")'`
- **Calculate hit ratio**: Count `status=success` vs `status=miss`
- **Track operation latency**: Add span instrumentation
- **Alert on errors**: Filter by `level=ERROR`
- **Aggregate metrics**: Parse structured fields for dashboards
