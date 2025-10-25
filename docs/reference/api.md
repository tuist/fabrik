# API Reference

Fabrik exposes three HTTP APIs for monitoring and management.

## Health API (Port 8888)

Simple health checks for orchestration.

### GET /health

```http
GET /health HTTP/1.1
```

**Response**:
```json
{
  "status": "healthy",
  "uptime_seconds": 345600,
  "version": "0.1.0"
}
```

## Metrics API (Port 9091)

Prometheus-compatible metrics.

### GET /metrics

```http
GET /metrics HTTP/1.1
```

**Response** (Prometheus format):
```prometheus
fabrik_cache_hits_total 123456
fabrik_cache_misses_total 7890
fabrik_cache_hit_ratio 0.94
fabrik_cache_size_bytes 5368709120
```

## Cache Query API (Port 9091)

REST API for querying cache state.

### GET /api/v1/artifacts

List artifacts (paginated).

**Query Parameters**:
- `limit` - Number of results (default: 100, max: 1000)
- `offset` - Pagination offset
- `sort` - Sort order (size_desc, created_desc, accessed_desc)

**Response**:
```json
{
  "artifacts": [
    {
      "hash": "abc123def456...",
      "size_bytes": 104857600,
      "created_at": "2025-10-24T10:00:00Z",
      "last_accessed": "2025-10-24T12:30:00Z",
      "access_count": 45
    }
  ],
  "total": 45678,
  "limit": 100,
  "offset": 0
}
```

### GET /api/v1/artifacts/{hash}

Check artifact existence.

### GET /api/v1/stats

Cache statistics.

**Response**:
```json
{
  "cache": {
    "total_objects": 45678,
    "total_size_bytes": 5368709120
  },
  "performance": {
    "cache_hits": 123456,
    "cache_misses": 7890,
    "hit_ratio": 0.94
  }
}
```

## Admin API (Port 9091)

Management operations (disabled by default).

### POST /api/v1/admin/evict

Trigger cache eviction.

### POST /api/v1/admin/clear

Clear entire cache.

## Authentication

All APIs (except Health) require JWT authentication:

```http
Authorization: Bearer <token>
```

## Configuration

```toml
[observability]
health_bind = "0.0.0.0:8888"
api_bind = "0.0.0.0:9091"
metrics_enabled = true
cache_query_api_enabled = true
admin_api_enabled = false
api_auth_required = true
```
