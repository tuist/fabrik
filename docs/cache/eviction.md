# Cache Eviction

Fabrik provides automatic cache eviction to prevent unbounded disk usage. When the cache exceeds the configured `max_size`, Fabrik automatically removes objects based on the configured eviction policy.

## Configuration

Configure eviction in your `fabrik.toml`:

```toml
[cache]
dir = ".fabrik/cache"
max_size = "5GB"           # Maximum cache size
eviction_policy = "lfu"    # lru | lfu | ttl
default_ttl = "7d"         # Default time-to-live for TTL policy
```

## Eviction Policies

### LFU (Least Frequently Used) - Default

The LFU policy evicts objects with the lowest access count first. This is the default policy because it tends to preserve frequently-accessed build artifacts.

```toml
[cache]
eviction_policy = "lfu"
```

**Best for:**
- Build caches with stable dependency trees
- Projects where common artifacts are accessed repeatedly
- CI environments with shared caches

### LRU (Least Recently Used)

The LRU policy evicts objects that haven't been accessed for the longest time.

```toml
[cache]
eviction_policy = "lru"
```

**Best for:**
- Development environments with rapidly changing dependencies
- Projects with distinct build phases
- When recent builds are more important than frequency

### TTL (Time To Live)

The TTL policy evicts objects older than the configured TTL. Objects are evicted based on creation time, not access time.

```toml
[cache]
eviction_policy = "ttl"
default_ttl = "7d"  # Evict objects older than 7 days
```

**Best for:**
- Compliance requirements with data retention limits
- Ensuring cache freshness
- Periodic cache invalidation scenarios

## How Eviction Works

1. **Trigger**: Eviction runs automatically when a `put()` operation would cause the cache to exceed `max_size`
2. **Target**: Fabrik evicts until the cache is at 90% of `max_size` (configurable via `target_ratio`)
3. **Selection**: Objects are selected based on the configured policy
4. **Batch Processing**: Up to 1000 objects are evicted per run to avoid blocking

### Metadata Tracking

Fabrik tracks the following metadata for each cached object:
- **Size**: Object size in bytes
- **Created At**: When the object was first cached
- **Accessed At**: Last access timestamp (updated on get/exists)
- **Access Count**: Number of times the object was accessed

This metadata is stored efficiently in RocksDB secondary indexes for fast eviction candidate selection.

## Size Format

The `max_size` configuration accepts human-readable size strings:

| Format | Example | Bytes |
|--------|---------|-------|
| Terabytes | `1TB` | 1,099,511,627,776 |
| Gigabytes | `5GB` | 5,368,709,120 |
| Megabytes | `500MB` | 524,288,000 |
| Kilobytes | `512KB` | 524,288 |
| Bytes | `1024` | 1,024 |

## TTL Format

The `default_ttl` configuration accepts duration strings:

| Format | Example | Seconds |
|--------|---------|---------|
| Days | `7d` | 604,800 |
| Hours | `24h` | 86,400 |
| Minutes | `30m` | 1,800 |
| Seconds | `3600s` or `3600` | 3,600 |

## Monitoring Eviction

Fabrik logs eviction activity at the INFO level:

```
INFO [fabrik] Eviction manager initialized: policy=lfu, max_size=5120MB, target_ratio=0.9
INFO [fabrik] Eviction complete: evicted 42 objects (128 MB) in 25ms
```

## Environment Variable Overrides

You can override eviction settings via environment variables:

```bash
# Override max cache size
export FABRIK_CONFIG_CACHE_MAX_SIZE=10GB

# Override eviction policy
export FABRIK_CONFIG_CACHE_EVICTION_POLICY=lru

# Override default TTL
export FABRIK_CONFIG_CACHE_DEFAULT_TTL=14d
```

## C API Support

The C API provides two initialization functions:

```c
// Basic initialization with default eviction (5GB, LFU, 7 days)
FabrikCache* cache = fabrik_cache_init("/path/to/cache");

// Custom eviction settings
FabrikCache* cache = fabrik_cache_init_with_eviction(
    "/path/to/cache",
    10ULL * 1024 * 1024 * 1024,  // 10GB max size
    1,                           // 0=LRU, 1=LFU, 2=TTL
    7 * 24 * 60 * 60            // 7 days TTL
);
```

## Best Practices

1. **Set realistic limits**: Choose a `max_size` that fits your available disk space while leaving room for other applications
2. **Choose the right policy**: LFU works best for most build caches, but LRU may be better for development
3. **Monitor eviction**: Watch the logs to ensure eviction is working as expected and adjust settings if needed
4. **Consider TTL for compliance**: Use TTL policy when you need predictable cache expiration

> [!NOTE]
> Eviction is triggered only on `put()` operations. To force immediate eviction, you can use the admin API (if enabled) or restart the daemon.

> [!WARNING]
> Setting `max_size` too low may cause frequent eviction and reduce cache hit rates. Monitor your cache performance after changing eviction settings.
