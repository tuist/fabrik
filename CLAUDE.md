# Fabrik - Multi-Layer Build Cache Infrastructure

## Project Overview

Fabrik is the foundational infrastructure for build caching, designed to be deployed and managed as a service by **Tuist**.

**Relationship to Tuist:**
> **Postgres is to Supabase** what **Fabrik is to Tuist**

Just as Supabase deploys and manages Postgres databases for customers, Tuist will deploy and manage Fabrik instances to provide build cache as a service. Fabrik is tenant-agnostic infrastructure; Tuist handles all customer logic, billing, and orchestration.

Fabrik provides a transparent, high-performance caching hierarchy to optimize build performance across different environments, supporting any build tool with remote caching capabilities including build systems (Gradle, Bazel, Nx, TurboRepo), compiler caches (sccache for Cargo/Rust), container build tools (BuildKit), and more.

## Implementation Plan

**See [PLAN.md](./PLAN.md) for the detailed implementation roadmap.**

The PLAN.md file tracks:
- Phase-by-phase implementation tasks
- Current progress and completed phases
- Performance targets and benchmarks
- Open questions and architectural decisions

**IMPORTANT**: Keep PLAN.md up to date as you progress through implementation. Mark tasks as completed, update the current phase, and add notes about decisions made.

## Architecture

### Service Model: Three-Layer Caching Strategy

From the developer's perspective, there are three transparent caching layers:

**Layer 1: Local Cache (CI environments)**
- Fabrik instance bound to build process lifecycle
- Deployed automatically in CI environments
- Uses mounted volumes for persistence
- RocksDB for in-memory + disk caching
- Closest to the build, lowest latency

**Layer 2: Regional Cache (Customer-dedicated)**
- Dedicated Fabrik instance deployed by Tuist for each customer/project
- Similar to how Supabase provisions a Postgres instance per project
- Deployed in customer's preferred region
- RocksDB with mounted volumes
- Shared across team's machines

**Layer 3: Tuist Server (Default/Fallback)**
- Always available, managed by Tuist
- S3-backed permanent storage
- Multiple customers can share S3 buckets with object key prefixes for isolation
- No eviction policy (permanent retention)

### Multi-Tenancy Model

**Dedicated instances per customer:**
- Each customer gets their own Fabrik instance(s) at Layer 2
- Similar to Supabase's model: one Postgres DB per project
- Layer 3 (S3) can be shared across customers with key prefixes (configurable at runtime)
- Complete isolation at the compute layer

**Fabrik is tenant-agnostic:**
- Fabrik doesn't understand "customers" or "projects"
- Authentication identifies the request, but Fabrik treats all requests equally
- Tuist manages customer relationships, billing, quotas

### Cache Behavior

- **Transparency**: Cache misses automatically fall back to the next configured layer
- **Write Strategy**: Write-through to all configured layers
- **Read Strategy**: Lazy pull from next layer on cache miss, then cache locally
- **Eviction**: Frequency-based eviction (LFU/LRU) for local and regional layers using RocksDB
- **Content Addressing**: Artifacts identified by content hash (natural deduplication)
- **Lifecycle**: Layer 1 instances start/stop with build process; Layer 2/3 are long-running

### Protocol Architecture

**Key Design Decision: Build System Adapters + Unified Fabrik Protocol**

**Two-Protocol Design:**
1. **Build Tool Protocols** (Layer 1 only): Gradle HTTP, Bazel gRPC, sccache S3, BuildKit registry, etc.
2. **Fabrik Protocol** (inter-layer): Unified gRPC-based protocol for Layer 1 ↔ Layer 2 communication

**Architecture Diagram:**

```
┌─────────────────────────────────────────────────────────┐
│ Build Tools                                             │
│  - Gradle (HTTP API)                                    │
│  - Bazel (gRPC API)                                     │
│  - Nx/TurboRepo (HTTP API)                             │
│  - sccache (S3 API)                                     │
│  - BuildKit (Registry API)                              │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ Layer 1: Local Cache (Build Tool Adapters)             │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│  │ Gradle   │  │ Bazel    │  │ BuildKit │             │
│  │ Adapter  │  │ Adapter  │  │ Adapter  │             │
│  │ (HTTP)   │  │ (gRPC)   │  │ (Registry)│            │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘             │
│       └────────────┬┴──────────────┘                   │
│                    ▼                                    │
│           Content-Addressed                             │
│           Normalization                                 │
│                    │                                    │
│                    ▼                                    │
│          ┌────────────────┐                             │
│          │  RocksDB Cache │                             │
│          └────────────────┘                             │
│                    │                                    │
│                    ▼                                    │
│        ┌─────────────────────┐                          │
│        │ Fabrik Protocol     │  ◄── Unified gRPC        │
│        │ Client (gRPC)       │                          │
│        └─────────────────────┘                          │
└────────────────────┬────────────────────────────────────┘
                     │
            Fabrik Protocol (gRPC)
          GET/PUT/EXISTS /v1/cache/{hash}
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│ Layer 2: Regional Cache (Fabrik Protocol Server)       │
│                                                         │
│        ┌─────────────────────┐                          │
│        │ Fabrik Protocol     │                          │
│        │ Server (gRPC)       │                          │
│        └─────────────────────┘                          │
│                    │                                    │
│                    ▼                                    │
│          ┌────────────────┐                             │
│          │  RocksDB Cache │                             │
│          └────────────────┘                             │
│                    │                                    │
│                    ▼                                    │
│             S3 Client                                   │
└────────────────────┬───────────────────────────────────┘
                     │
                     ▼
              ┌──────────┐
              │ S3 (S3)  │
              └──────────┘
```

**Example flow (Gradle build):**
1. Gradle makes `GET /cache/abc123` to Layer 1 Gradle adapter (HTTP)
2. Layer 1 Gradle adapter normalizes to content hash
3. Layer 1 checks local RocksDB → MISS
4. Layer 1 Fabrik client makes `Get(hash)` to Layer 2 (gRPC)
5. Layer 2 checks local RocksDB → MISS
6. Layer 2 fetches from S3 → HIT
7. Layer 2 caches locally, returns to Layer 1 (gRPC)
8. Layer 1 caches locally, returns to Gradle (HTTP)

**Key Insights:**
- **Layer 1**: Runs build tool adapters, speaks Fabrik protocol to upstream
- **Layer 2**: Only speaks Fabrik protocol (no build tool knowledge)
- **Build tool independence**: Layer 2 doesn't care about Gradle vs Bazel vs BuildKit
- **Simplified Layer 2**: Single protocol implementation, multi-tenant by default

**Benefits:**
- ✅ **Simpler Layer 2**: Only implements Fabrik protocol
- ✅ **Easier to extend**: Add new build tools by writing Layer 1 adapters
- ✅ **Efficient inter-layer**: gRPC for low latency and streaming
- ✅ **Build tool agnostic**: Layer 2 works with any build tool

### Fabrik Protocol Specification

**Protocol Definition**: `proto/fabrik.proto`

**gRPC Service:**
```protobuf
service FabrikCache {
  // Check if artifact exists
  rpc Exists(ExistsRequest) returns (ExistsResponse);

  // Retrieve artifact (streaming)
  rpc Get(GetRequest) returns (stream GetResponse);

  // Store artifact (streaming)
  rpc Put(stream PutRequest) returns (PutResponse);

  // Delete artifact (optional)
  rpc Delete(DeleteRequest) returns (DeleteResponse);

  // Get cache statistics
  rpc GetStats(GetStatsRequest) returns (GetStatsResponse);
}
```

**Key characteristics:**
- **Content-addressed**: All operations use SHA256 hash as identifier
- **Streaming**: Get/Put support streaming for large artifacts (efficient memory usage)
- **Stateless**: No session state, each request is independent
- **Port**: Default 7070 for Fabrik protocol gRPC server

**Example usage:**
```rust
// Layer 1 client queries Layer 2
let response = client.exists(ExistsRequest { hash: "abc123..." }).await?;
if response.exists {
    let stream = client.get(GetRequest { hash: "abc123..." }).await?;
    // Stream chunks...
}
```

### Build Tool Integration Strategy

**Approach: Tool-Specific Wrappers**

Each build tool requires different integration approaches based on how they accept remote cache configuration:

**Bazel**: Uses command-line flags (`--remote_cache`), so Fabrik provides a wrapper command `fabrik bazel` that:
1. Starts the Bazel gRPC adapter (ContentAddressableStorage + ActionCache)
2. Automatically injects `--remote_cache=grpc://127.0.0.1:{port}` flag
3. Passes through all other bazel arguments
4. Handles graceful shutdown

**Usage:**
```bash
# Instead of: bazel build //...
# Use:
fabrik bazel -- build //...

# All bazel flags work as normal:
fabrik bazel -- build //... --config=release --jobs=8

# With custom cache directory:
fabrik bazel --config-cache-dir=/custom/path -- build //...
```

**Configuration:**
```toml
[build_systems.bazel]
port = 0              # 0 = random port (default)
bind = "127.0.0.1"    # Listen only on localhost
```

**Benefits:**
- ✅ **Zero configuration**: Bazel cache works automatically
- ✅ **Transparent**: Drop-in replacement for `bazel` command
- ✅ **Compatible**: All bazel flags pass through unchanged
- ✅ **Safe**: Adapter stops when bazel process exits

### Multi-Instance Configuration

**How to Model Multiple Regions/Instances:**

The upstream array naturally models multiple instances. Array order determines priority.

**Use Case 1: Fallback Chain (Office → Regional → S3)**

```toml
# Layer 1 configuration
[[upstream]]
url = "https://office-cache.local"      # Try office first (5ms)
timeout = "5s"

[[upstream]]
url = "https://cache.tuist.io"          # Fallback to regional (20ms)
timeout = "15s"

[[upstream]]
url = "s3://backup/tenant-123/"         # Ultimate fallback
timeout = "60s"
permanent = true
```

**Behavior:**
- **Reads**: Sequential fallback (try each in order until hit)
- **Writes**: Write-through to all if `write_through=true`

**Use Case 2: Multi-Region with Geographic Priority**

```toml
# Layer 1 in US (prefers US region)
[[upstream]]
url = "https://cache-us-east.tuist.io"  # Primary (low latency)
timeout = "10s"

[[upstream]]
url = "https://cache-eu-west.tuist.io"  # Fallback (high latency)
timeout = "30s"
```

```toml
# Layer 1 in EU (reversed priority)
[[upstream]]
url = "https://cache-eu-west.tuist.io"  # Primary (low latency)
timeout = "10s"

[[upstream]]
url = "https://cache-us-east.tuist.io"  # Fallback (high latency)
timeout = "30s"
```

**Use Case 3: DNS-Based Geo-Routing (Recommended)**

```toml
# Layer 1 anywhere in the world (same config)
[[upstream]]
url = "https://cache.tuist.io"          # DNS resolves to nearest region
timeout = "15s"

[[upstream]]
url = "s3://global/tenant-123/"
permanent = true
```

**How it works:**
- Tuist manages GeoDNS for `cache.tuist.io`
- DNS returns different IPs based on client location
- Layer 1 doesn't need region-specific configuration
- Simpler deployment, more scalable

**Use Case 4: Cross-Region Replication (Layer 2)**

```toml
# Layer 2 US server configuration
[[upstream]]
url = "s3://us-cache/tenant-123/"
region = "us-east-1"
permanent = true
write_through = true
workers = 20

[[upstream]]
url = "s3://eu-cache/tenant-123/"
region = "eu-west-1"
permanent = true
write_through = true      # Replicate writes to EU
workers = 10
read_only = true          # Don't read from EU (higher latency)
```

**Design Principles:**
- **Array order = priority order** for reads
- **No built-in load balancing** - use external load balancer or DNS
- **No parallel queries** - sequential fallback keeps behavior predictable
- **Per-upstream configuration** - each upstream can have different settings

### Layer 2 High Availability & Multi-Region

**Design Decision: Independent Instances (No Clustering)**

Each Layer 2 instance is completely independent - no distributed consensus, no cache synchronization, no inter-instance communication.

**Single Region HA (3 instances behind load balancer):**
```
                    DNS: cache-us-east.tuist.io
                              │
                              ▼
                    ┌──────────────────┐
                    │  Load Balancer   │
                    │  (AWS ALB/NLB)   │
                    └──────────────────┘
                       │      │      │
          ─────────────┴──────┴──────┴─────────────
          │                  │                    │
          ▼                  ▼                    ▼
    ┌─────────┐        ┌─────────┐        ┌─────────┐
    │ Layer 2 │        │ Layer 2 │        │ Layer 2 │
    │ Instance│        │ Instance│        │ Instance│
    │   #1    │        │   #2    │        │   #3    │
    └────┬────┘        └────┬────┘        └────┬────┘
         │                  │                    │
         └──────────────────┴────────────────────┘
                            │
                            ▼
                  S3 us-cache/tenant-123/
```

**Multi-Region (independent instances per region):**
```
┌────────────────────────────────┐  ┌────────────────────────────────┐
│ US Region                      │  │ EU Region                      │
│ cache-us-east.tuist.io:7070    │  │ cache-eu-west.tuist.io:7070    │
│                                │  │                                │
│  ┌────────────────┐            │  │  ┌────────────────┐            │
│  │ RocksDB        │            │  │  │ RocksDB        │            │
│  │ (US hot set)   │            │  │  │ (EU hot set)   │            │
│  └────────┬───────┘            │  │  └────────┬───────┘            │
│           │                    │  │           │                    │
│           ▼                    │  │           ▼                    │
│  S3 us-cache/tenant/           │  │  S3 eu-cache/tenant/           │
└────────────────────────────────┘  └────────────────────────────────┘
```

**Why No Clustering:**
- ✅ **Simpler architecture**: No Raft/Paxos, no distributed state
- ✅ **More reliable**: No cascading failures, no split-brain
- ✅ **Scales linearly**: Add regions without coordination overhead
- ✅ **Predictable latency**: No cross-region synchronization
- ✅ **S3 as shared state**: Eliminates need for distributed consensus

**Tradeoff:**
- Duplicate S3 fetches when load balancer routes same hash to different instances
- Acceptable because S3 is fast and hot cache naturally forms per instance

## Authentication & Security

### JWT-Based Authentication

**All build systems support Bearer tokens via `Authorization` header**, making JWT the ideal choice.

**Token Generation (Tuist Server):**
- Tuist signs JWTs using RS256 (asymmetric cryptography)
- Private key remains on Tuist server only
- Developers receive JWT after `tuist auth login`
- Build systems read token from environment variables

**Token Validation (Fabrik):**
- Fabrik validates JWT signature locally using public key
- **Zero-latency validation** - no network calls or database lookups
- Public key loaded at startup (from file or JWKS endpoint)
- Hot-reloadable: file watch or periodic refresh
- Extracts claims for logging/metrics (customer_id, permissions, etc.)

**JWT Claims Example:**
```json
{
  "sub": "customer_123",           // customer ID
  "project_id": "proj_456",        // optional project isolation
  "permissions": ["cache:read", "cache:write"],
  "instance_id": "fabrik_regional_us-east",
  "exp": 1730000000,               // expiry (1-24 hours for devs)
  "iat": 1729900000,               // issued at
  "kid": "key_2025_01"             // key ID for rotation
}
```

**Key Rotation:**
- Support multiple active public keys (identified by `kid` in JWT header)
- Graceful rotation: add new key, wait for old tokens to expire, remove old key
- No service restart required

**Token Lifecycle:**
- **Developer tokens**: Short-lived (1-24 hours) - safe if leaked
- **Machine-to-machine**: Longer TTL (days/weeks) for Layer-to-Layer communication
- **CI tokens**: Generated per build, scoped to project

**Security Benefits:**
- No token leakage risk (auto-expiring)
- Stateless (scales horizontally)
- Local validation (microseconds)
- Rich metadata in claims

### Build Tool Authentication Support

All target build tools natively support Bearer tokens or S3 credentials:

| Build Tool | Protocol | Auth Method | Environment Variable |
|------------|----------|-------------|----------------------|
| **Gradle** | HTTP | `Authorization: Bearer <token>` or Basic Auth | Custom or `ORG_GRADLE_PROJECT_*` |
| **Bazel** | gRPC | `--remote_header=Authorization=Bearer $TOKEN` | `$TOKEN` (user-defined) |
| **Nx** | HTTP | `Authorization: Bearer <token>` | `NX_SELF_HOSTED_REMOTE_CACHE_ACCESS_TOKEN` |
| **TurboRepo** | HTTP | `Authorization: Bearer <token>` | `TURBO_TOKEN` |
| **sccache** | S3 API | AWS credentials or custom endpoint | `SCCACHE_BUCKET`, `SCCACHE_ENDPOINT` |
| **BuildKit** | Registry | `Authorization: Bearer <token>` | Docker registry auth |

Fabrik must support HTTP, gRPC, S3-compatible, and OCI registry protocols to accommodate all build tools.

## Analytics & Observability

### Design Philosophy: Pull-Based Model (Like Supabase)

**Tuist actively queries Fabrik instances** rather than Fabrik pushing telemetry elsewhere.

**Why Pull-Based:**
- ✅ Tuist controls query frequency (no overwhelming)
- ✅ No additional infrastructure (no telemetry pipeline, no Kafka/PubSub)
- ✅ Simpler security (Fabrik doesn't need outbound access)
- ✅ Works in private networks
- ✅ Real-time on-demand data when needed

**This gives Tuist the same visibility that Supabase has over Postgres, but for build caches.**

---

### API Surface for Tuist

Fabrik exposes **three HTTP APIs** for Tuist to consume:

#### 1. Health API (Port 8888)

**Simple health checks for orchestration**

```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "uptime_seconds": 345600,
  "version": "0.1.0"
}
```

**Use cases:**
- Load balancer health checks
- Kubernetes liveness/readiness probes
- Tuist instance monitoring

---

#### 2. Metrics API (Port 9091)

**Prometheus-compatible metrics for monitoring & billing**

```http
GET /metrics
```

**Response (Prometheus format):**
```prometheus
# Cache performance
fabrik_cache_hits_total 123456
fabrik_cache_misses_total 7890
fabrik_cache_hit_ratio 0.94

# Storage
fabrik_cache_size_bytes 5368709120
fabrik_cache_objects 45678

# Latency (histogram)
fabrik_request_duration_seconds_bucket{le="0.005"} 9500
fabrik_request_duration_seconds_bucket{le="0.01"} 9800
fabrik_request_duration_seconds_bucket{le="0.025"} 9950

# Bandwidth
fabrik_bandwidth_bytes_total{direction="upload"} 1073741824
fabrik_bandwidth_bytes_total{direction="download"} 5368709120

# Upstream
fabrik_upstream_requests_total{result="hit"} 5000
fabrik_upstream_requests_total{result="miss"} 200

# Evictions
fabrik_evictions_total 123
```

**Use cases:**
- Tuist Grafana dashboards
- Customer billing (bandwidth, storage)
- Performance monitoring
- Alerting (high miss rate, disk full)

**Tuist polling:** Every 30-60 seconds, stores time-series data in Prometheus/TimescaleDB

---

#### 3. Cache Query API (Port 9091)

**REST API for Tuist to query cache state (like Supabase table viewer)**

**List artifacts (paginated):**
```http
GET /api/v1/artifacts?limit=100&offset=0&sort=size_desc
```

**Response:**
```json
{
  "artifacts": [
    {
      "hash": "abc123def456...",
      "size_bytes": 104857600,
      "created_at": "2025-10-24T10:00:00Z",
      "last_accessed": "2025-10-24T12:30:00Z",
      "access_count": 45,
      "metadata": {
        "build_system": "gradle"
      }
    }
  ],
  "total": 45678,
  "limit": 100,
  "offset": 0
}
```

**Check artifact existence:**
```http
GET /api/v1/artifacts/{hash}
```

**Cache statistics (detailed):**
```http
GET /api/v1/stats
```

**Response:**
```json
{
  "cache": {
    "total_objects": 45678,
    "total_size_bytes": 5368709120,
    "size_by_type": {
      "gradle": 2147483648,
      "bazel": 1073741824
    }
  },
  "performance": {
    "cache_hits": 123456,
    "cache_misses": 7890,
    "hit_ratio": 0.94,
    "latency_p50_ms": 5,
    "latency_p95_ms": 12,
    "latency_p99_ms": 25
  },
  "bandwidth": {
    "upload_bytes_total": 1073741824,
    "download_bytes_total": 5368709120
  },
  "upstream": {
    "requests_total": 5200,
    "hits": 5000,
    "misses": 200
  },
  "evictions": {
    "total": 123,
    "eviction_rate_per_hour": 5
  }
}
```

**Top artifacts (most accessed):**
```http
GET /api/v1/artifacts/top?limit=50
```

**Search artifacts:**
```http
GET /api/v1/artifacts/search?query=gradle&min_size=1MB
```

**Use cases:**
- Tuist Dashboard: "Cache Explorer" (like Supabase table viewer)
- Customer support: "Why is my cache not working?"
- Debugging: "Is artifact X cached?"
- Analytics: "Which artifacts are most popular?"
- Billing: "Show storage breakdown by build system"

---

#### 4. Admin API (Port 9091, Optional)

**Management operations for Tuist orchestration**

**Trigger eviction:**
```http
POST /api/v1/admin/evict
{
  "target_size_bytes": 4294967296,
  "strategy": "lru"
}
```

**Clear cache:**
```http
POST /api/v1/admin/clear
{
  "confirm": true
}
```

**Get configuration:**
```http
GET /api/v1/admin/config
```

**Update configuration (hot-reload):**
```http
POST /api/v1/admin/config
{
  "max_cache_size": "100GB",
  "eviction_policy": "lfu"
}
```

**Use cases:**
- Tuist orchestration: resize instances dynamically
- Emergency: clear cache if corrupted
- Operations: change eviction policy without restart

---

### Configuration

```toml
[observability]
# Health check endpoint
health_bind = "0.0.0.0:8888"
health_enabled = true

# Metrics + Cache Query API + Admin API
api_bind = "0.0.0.0:9091"

# Enable/disable APIs
metrics_enabled = true
cache_query_api_enabled = true   # For Tuist Dashboard
admin_api_enabled = false        # Disable in production by default

# Authentication for APIs
api_auth_required = true
api_jwt_public_key_file = "/etc/fabrik/api-public-key.pem"

# Logging
log_level = "info"
log_format = "json"

# Tracing
tracing_enabled = false
tracing_endpoint = ""
```

---

### Tuist Dashboard Integration

**How Tuist uses these APIs:**

```
┌─────────────────────────────────────────────────────┐
│ Tuist Dashboard (Web UI)                            │
│                                                     │
│  Cache Explorer:                                    │
│  - List all artifacts                               │
│  - Search by hash/build system                      │
│  - Show access patterns                             │
│                                                     │
│  Analytics:                                         │
│  - Cache hit ratio chart                            │
│  - Bandwidth usage (for billing)                    │
│  - Top artifacts                                    │
│  - Latency metrics                                  │
│                                                     │
│  Billing:                                           │
│  - Storage used: 5.0 GB                             │
│  - Bandwidth this month: 500 GB                     │
└─────────────────────────────────────────────────────┘
                        │
                        ▼
              ┌──────────────────┐
              │ Tuist API Server │
              └──────────────────┘
                        │
            ┌───────────┴───────────┐
            ▼                       ▼
  ┌──────────────────┐    ┌──────────────────┐
  │ Fabrik (US East) │    │ Fabrik (EU West) │
  │ GET /metrics     │    │ GET /metrics     │
  │ GET /api/v1/...  │    │ GET /api/v1/...  │
  └──────────────────┘    └──────────────────┘
```

**Tuist polling loop (pseudocode):**
```python
# Every 60 seconds
for instance in fabrik_instances:
    # Pull metrics
    metrics = http.get(f"{instance.url}:9091/metrics")
    store_in_timeseries_db(metrics)

    # Pull cache stats
    stats = http.get(f"{instance.url}:9091/api/v1/stats")
    update_billing(customer_id, stats.bandwidth)

    # Check health
    health = http.get(f"{instance.url}:8888/health")
    if not health.ok:
        alert_ops_team(instance)
```

---

### Summary: API Endpoints

| API | Port | Endpoint | Purpose |
|-----|------|----------|---------|
| **Health** | 8888 | `GET /health` | Health checks, uptime monitoring |
| **Metrics** | 9091 | `GET /metrics` | Prometheus metrics for monitoring/billing |
| **Cache Query** | 9091 | `GET /api/v1/artifacts` | List/search artifacts (Tuist Dashboard) |
| **Cache Query** | 9091 | `GET /api/v1/stats` | Detailed statistics |
| **Admin** | 9091 | `POST /api/v1/admin/*` | Management operations (optional) |

**Security:**
- Health API: Public (for load balancers)
- Metrics API: JWT authentication (Tuist token)
- Cache Query API: JWT authentication (Tuist token)
- Admin API: JWT authentication with admin scope

## Technical Stack

### Primary Language
- **Rust**: Chosen for portability, memory safety, performance, and low-level control

### Key Dependencies
- **RocksDB**: Embedded key-value store for disk + in-memory caching with built-in LFU/LRU eviction
- **JWT validation**: `jsonwebtoken` crate for RS256 signature verification
- **HTTP server**: `axum` or `actix-web` for REST APIs (Gradle, Nx, TurboRepo)
- **gRPC server**: `tonic` for Bazel Remote Execution API
- **Metrics**: `prometheus` crate for metrics exposition
- **S3 client**: `aws-sdk-s3` or `rusoto_s3` for Layer 3 storage

### Supported Build Tools (Initial Focus)

**Gradle** (HTTP REST API)
- Protocol: HTTP with Basic Auth or Bearer token
- Endpoints: `PUT/GET /cache/{hash}`
- Documentation: https://docs.gradle.org/current/userguide/build_cache.html

**Bazel** (gRPC Remote Execution API)
- Protocol: gRPC with metadata headers for auth
- Services: ContentAddressableStorage, ActionCache, Capabilities
- Documentation: https://bazel.build/remote/rbe

**Nx** (HTTP REST API)
- Protocol: HTTP with Bearer token
- Endpoints: `PUT/GET` for artifacts
- Documentation: https://nx.dev/docs/guides/tasks--caching/self-hosted-caching

**TurboRepo** (HTTP REST API)
- Protocol: HTTP with Bearer token
- Endpoints: `PUT/GET /v8/artifacts/:hash?teamId=<id>`
- Documentation: https://turborepo.com/docs/core-concepts/remote-caching

**sccache (Cargo/Rust)** (S3 API)
- Protocol: S3 API (also supports GCS, Redis)
- Integration: Via `RUSTC_WRAPPER` environment variable
- Storage: Compatible with Fabrik's S3 storage layer
- Documentation: https://github.com/mozilla/sccache

### Planned Support

**Vite+** (In Development)
- Unified toolchain with built-in monorepo caching
- Positioned as alternative to TurboRepo/Nx
- Expected to support HTTP-based remote cache protocol
- Website: https://viteplus.dev (Currently in early access)

**Protocol Support:** Fabrik must implement HTTP, gRPC, S3-compatible, and OCI registry APIs to support all build tools.

## Configuration

### Configuration Precedence (industry standard)
1. Command-line arguments (highest priority)
2. Environment variables
3. Configuration file (lowest priority)

### Design Philosophy
- **Single binary** with configurable behavior via flags
- **Unified upstream model**: S3, GCS, and other Fabrik instances are all treated as "upstream" layers
- **Config-backed options**: CLI flags prefixed with `--config-*` can be set via config file, env vars, or CLI
- **Flexible deployment**: Same binary can be Layer 1 (local/CI), Layer 2 (regional server), or both

### CLI Commands

**`fabrik bazel`** - Wrapper for Bazel with automatic cache configuration
```bash
fabrik bazel <BAZEL_ARGS>...
```
Starts Bazel gRPC adapter and injects `--remote_cache` flag automatically.

**`fabrik activate`** - Activate shell integration for automatic daemon management
```bash
fabrik activate <shell>     # Generate shell hook (bash, zsh, fish)
fabrik activate --status    # Check/start daemon and export env vars
```

**`fabrik daemon`** - Run long-lived local cache daemon (Layer 1 for development)
```bash
fabrik daemon [OPTIONS]
```

**`fabrik deactivate`** - Deactivate Fabrik and optionally stop daemon
```bash
fabrik deactivate [--stop-daemon]
```

**`fabrik server`** - Run regional/cloud cache server (Layer 2)
```bash
fabrik server [OPTIONS]
```

**`fabrik config`** - Configuration utilities (validate, generate, show)
```bash
fabrik config <validate|generate|show> [OPTIONS]
```

**`fabrik health`** - Health check and diagnostics
```bash
fabrik health [OPTIONS]
```

### Daemon Architecture (Activation-Based)

**Design Philosophy:**

Fabrik uses an **activation-based daemon model** where each project automatically gets its own daemon instance when you `cd` into the project directory. This provides:

- **Zero configuration**: Daemon starts automatically when needed
- **Project isolation**: Each project gets its own daemon with independent ports
- **No conflicts**: Multiple projects can run simultaneously without port conflicts
- **Automatic cleanup**: Daemon state is tracked and cleaned up properly

**How It Works:**

1. **Config Hash as Identity**: Each `.fabrik.toml` is hashed (SHA256, 16 chars) to create a unique daemon identifier
   - Same config = same daemon (automatic reuse)
   - Different config = different daemon (isolation)
   - Config changes = new daemon (safety)

2. **Dynamic Port Allocation**: Daemons bind to port 0 (random available ports)
   - OS assigns free ports automatically
   - No port conflicts between projects
   - Actual ports written to state file: `~/.fabrik/daemons/{hash}/ports.json`

3. **Shell Integration**: `fabrik activate` hook runs on directory change
   - Detects `.fabrik.toml` (walks up directory tree)
   - Computes config hash
   - Checks if daemon is running for this config
   - If not running: starts daemon in background
   - Exports environment variables with actual ports

4. **State Management**: Each daemon writes state to `~/.fabrik/daemons/{hash}/`
   ```
   ~/.fabrik/daemons/a3f5d9c2b1e8f7a4/
   ├── pid                 # Process ID
   ├── ports.json          # { "http": 54321, "grpc": 54322, "metrics": 9091 }
   └── config_path.txt     # /path/to/project/.fabrik.toml
   ```

5. **Graceful Shutdown**: Daemon handles SIGTERM/SIGINT
   - Waits for in-flight requests to complete (5 second timeout)
   - Cleans up state directory
   - Removes PID and port files

**Usage Example:**

```bash
# 1. Add shell integration (one-time setup)
eval "$(fabrik activate bash)"   # Or zsh, fish

# 2. Navigate to project
cd ~/myproject

# Behind the scenes:
# - Finds .fabrik.toml
# - Computes hash: a3f5d9c2b1e8f7a4
# - Checks ~/.fabrik/daemons/a3f5d9c2b1e8f7a4/ → not found
# - Spawns: fabrik daemon --config .fabrik.toml &
# - Daemon binds to random ports (e.g., 54321, 54322)
# - Daemon writes ports.json
# - Shell hook exports:
export FABRIK_HTTP_URL=http://127.0.0.1:54321
export FABRIK_GRPC_URL=grpc://127.0.0.1:54322
export GRADLE_BUILD_CACHE_URL=http://127.0.0.1:54321
export NX_SELF_HOSTED_REMOTE_CACHE_SERVER=http://127.0.0.1:54321

# 3. Run build (uses daemon automatically)
gradle build
# Gradle reads GRADLE_BUILD_CACHE_URL and connects to daemon

# 4. Navigate to different project
cd ~/other-project
# New daemon starts for this project's config (different hash)

# 5. Return to first project
cd ~/myproject
# Same daemon still running (hash matches), env vars exported again
```

**Multi-Project Support:**

Each project gets its own daemon instance, identified by config hash:

```bash
# Terminal 1
cd ~/project-a
gradle build  # Uses daemon on ports 54321/54322

# Terminal 2 (simultaneously)
cd ~/project-b
gradle build  # Uses different daemon on ports 54401/54402

# Terminal 3 (simultaneously)
cd ~/project-c
gradle build  # Uses different daemon on ports 54515/54516
```

**Daemon Lifecycle:**

```
User Event              Daemon Action
──────────────────────  ────────────────────────────────────────
cd ~/project            1. Detect .fabrik.toml
                        2. Compute hash: a3f5d9c2
                        3. Check state: ~/.fabrik/daemons/a3f5d9c2/
                        4. If not running: spawn daemon
                        5. Daemon binds to port 0 → gets 54321
                        6. Daemon writes ports.json
                        7. Shell exports GRADLE_BUILD_CACHE_URL
                        
gradle build            → Connects to http://127.0.0.1:54321 ✅

cd ~/other-project      1. Detect different .fabrik.toml
                        2. Compute hash: b7e4a1f9 (different!)
                        3. Spawn new daemon on different ports
                        4. Export new env vars
                        
Ctrl+C on daemon        1. Receive SIGINT
                        2. Wait for servers to shutdown (5s timeout)
                        3. Remove ~/.fabrik/daemons/a3f5d9c2/
                        4. Exit cleanly
```

**Authentication:**

Each daemon loads its authentication token from the project's config:

```toml
# .fabrik.toml (checked into git)
[cache]
dir = ".fabrik/cache"

[[upstream]]
url = "grpc://cache.tuist.io:7070"

[auth]
token_file = ".fabrik.token"  # Gitignored, per-developer

[daemon]
http_port = 0  # 0 = random (recommended for daemon mode)
grpc_port = 0
```

```bash
# Developer workflow
tuist auth login              # Creates .fabrik.token
cd ~/project                  # Daemon starts, loads .fabrik.token
gradle build                  # Daemon uses token for upstream requests
```

**Benefits:**

- ✅ **Zero config**: Just `cd` into project
- ✅ **No conflicts**: Each project gets unique ports
- ✅ **Clean shutdown**: State files cleaned up properly
- ✅ **Fast startup**: Daemon starts in <500ms
- ✅ **Multi-tenant**: Multiple projects run simultaneously
- ✅ **Secure**: Each daemon loads project-specific token

### Configuration File Format (TOML)

```toml
# Local storage (always present, first layer)
[cache]
dir = "/data/fabrik/cache"
max_size = "100GB"
eviction_policy = "lfu"  # lru | lfu | ttl
default_ttl = "7d"

# Upstream array (optional, each entry is tried in order)
[[upstream]]
url = "https://regional-cache.example.com"
timeout = "10s"
read_only = false        # Optional: if true, never write to this upstream
permanent = false        # Optional: if true, never evict from this upstream

[[upstream]]
url = "s3://tuist-build-cache/customer-{customer_id}/"
timeout = "60s"
permanent = true         # S3 is permanent storage
write_through = true     # Write immediately
workers = 20             # Concurrent upload workers

# S3-specific settings
region = "us-east-1"
endpoint = ""            # Optional: for S3-compatible services
access_key = ""          # Or use AWS_ACCESS_KEY_ID env
secret_key = ""          # Or use AWS_SECRET_ACCESS_KEY env

# Authentication (for server mode)
[auth]
public_key_file = "/etc/fabrik/jwt-public-key.pem"
key_refresh_interval = "5m"
required = true

# Build tool adapters (Layer 1 only)
[build_tools]
enabled = ["gradle", "bazel", "nx", "turborepo", "sccache", "buildkit"]

# Optional: Per-adapter configuration
[build_tools.gradle]
port = 0  # 0 = random port
auto_configure = true  # Auto-set GRADLE_BUILD_CACHE_URL

# Fabrik protocol (Layer 2 server, Layer 1 client)
[fabrik]
enabled = false  # true for Layer 2, false for Layer 1
bind = "0.0.0.0:7070"  # gRPC bind address

# Observability
[observability]
log_level = "info"
log_format = "json"
metrics_bind = "0.0.0.0:9091"
metrics_enabled = true
tracing_enabled = false
health_bind = "0.0.0.0:8888"

# Runtime
[runtime]
graceful_shutdown_timeout = "30s"
max_concurrent_requests = 10000
worker_threads = 0  # 0 = auto (num CPUs)
```

### Automatic Storage Backend Detection

**Design Philosophy: Zero Configuration**

Fabrik automatically detects the best storage backend for the environment with zero configuration required. This means the same configuration (or no configuration at all) works seamlessly in CI and local development.

#### Detection Logic

When Fabrik starts, it automatically detects the runtime environment:

1. **GitHub Actions**: Detected via `ACTIONS_CACHE_URL` + `ACTIONS_RUNTIME_TOKEN` environment variables
   - Automatically uses GitHub Actions Cache API
   - Up to 10GB cache storage (GitHub's limit)
   - 7-day retention for unused caches
   - No configuration needed - just works

2. **GitLab CI**: Detected via `CI_API_V4_URL` + `CI_JOB_TOKEN` (planned)
   - Automatically uses GitLab CI cache

3. **Local/Other**: Falls back to filesystem storage
   - Uses `cache.dir` from config or default `/tmp/fabrik-cache`

#### Environment Variables

All configuration options can be overridden via environment variables with the `TUIST_CONFIG_*` prefix:

| Config Option | Environment Variable | Example |
|--------------|---------------------|---------|
| `cache.dir` | `TUIST_CONFIG_CACHE_DIR` | `/tmp/fabrik-cache` |
| `cache.max_size` | `TUIST_CONFIG_CACHE_MAX_SIZE` | `10GB` |
| `cache.eviction_policy` | `TUIST_CONFIG_CACHE_EVICTION_POLICY` | `lfu` |
| `auth.token` | `TUIST_CONFIG_AUTH_TOKEN` or `TUIST_TOKEN` | `eyJ0eXAi...` |
| `upstream[0].url` | `TUIST_CONFIG_UPSTREAM_0_URL` | `grpc://cache.tuist.io:7070` |
| `upstream[0].timeout` | `TUIST_CONFIG_UPSTREAM_0_TIMEOUT` | `30s` |

**AWS Credentials** (for S3 upstream):
- `AWS_ACCESS_KEY_ID` or `TUIST_CONFIG_UPSTREAM_X_ACCESS_KEY`
- `AWS_SECRET_ACCESS_KEY` or `TUIST_CONFIG_UPSTREAM_X_SECRET_KEY`
- `AWS_REGION` or `TUIST_CONFIG_UPSTREAM_X_REGION`

#### Zero-Config CI Examples

**GitHub Actions** (no configuration needed):
```yaml
name: Build

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Fabrik automatically detects GitHub Actions and uses cache API
      - run: fabrik bazel -- build //...
        env:
          TUIST_TOKEN: ${{ secrets.TUIST_TOKEN }}  # Optional: for upstream cache
```

**What happens:**
1. Fabrik detects `ACTIONS_CACHE_URL` environment variable (auto-provided by GitHub)
2. Automatically uses GitHub Actions Cache as primary storage
3. Cache persists across workflow runs
4. No `.fabrik.toml` needed

**GitLab CI** (no configuration needed):
```yaml
build:
  script:
    - fabrik bazel -- build //...
  variables:
    TUIST_TOKEN: $CI_TUIST_TOKEN
```

**Local Development** (no configuration needed):
```bash
# Set token once
export TUIST_TOKEN=xxx

# Just run - uses filesystem automatically
fabrik bazel -- build //...
```

#### Logging

Fabrik logs the detected storage backend at startup:

```
INFO fabrik::storage - Detected GitHub Actions environment (ACTIONS_CACHE_URL present)
INFO fabrik::storage - Using storage backend: github-actions
INFO fabrik::storage - Cache URL: https://artifactcache.actions.githubusercontent.com/...
```

Or in local dev:

```
INFO fabrik::storage - No CI environment detected
INFO fabrik::storage - Using storage backend: filesystem
INFO fabrik::storage - Cache directory: /tmp/fabrik-cache
```

### Example Configurations by Layer

**Layer 1 (CI with mounted volume - Bazel):**
```toml
# .fabrik.toml in repository (optimized for Bazel)
[cache]
dir = "/mnt/build-cache"
max_size = "20GB"

[[upstream]]
url = "grpc://cache-us-east.tuist.io:7070"  # Fabrik protocol
timeout = "30s"

[build_systems.bazel]
port = 0  # Random port (default)
```

```bash
# CI command (wrapper automatically injects --remote_cache):
fabrik bazel -- build //...
```

**Layer 1 (Local development):**
```toml
# .fabrik.toml in repository
[cache]
dir = ".fabrik/cache"
max_size = "5GB"

[[upstream]]
url = "grpc://cache.tuist.io:7070"  # Fabrik protocol
timeout = "30s"

[build_systems.bazel]
port = 0
```

```bash
# Drop-in replacement for bazel:
fabrik bazel -- build //...
fabrik bazel -- test //...
fabrik bazel -- build //... --config=release
```

**Layer 2 (Regional server with S3 upstream):**
```toml
# /etc/fabrik/config.toml on server
[cache]
dir = "/data/fabrik/cache"
max_size = "500GB"
eviction_policy = "lfu"

[[upstream]]
url = "s3://tuist-build-cache/tenant-acme-corp/"
timeout = "60s"
permanent = true
write_through = true
region = "us-east-1"
workers = 20

[auth]
public_key_file = "/etc/fabrik/jwt-public-key.pem"
key_refresh_interval = "5m"
required = true

# Layer 2 doesn't run build tool adapters
[build_tools]
enabled = []  # Empty - Layer 2 only speaks Fabrik protocol

# Instead, run Fabrik protocol server
[fabrik]
enabled = true
bind = "0.0.0.0:7070"  # gRPC server for Fabrik protocol

[observability]
metrics_bind = "0.0.0.0:9091"
health_bind = "0.0.0.0:8888"
```

**Command:**
```bash
fabrik server --config /etc/fabrik/config.toml
```

**What Layer 2 does:**
- ✅ Runs Fabrik protocol gRPC server on port 7070
- ✅ Does NOT run Gradle/Bazel/Nx adapters
- ✅ Multi-tenant by default (all tenants use same Fabrik protocol)
- ✅ Simpler, more efficient

**Layer 2 (Multi-region with replication):**
```toml
[[upstream]]
url = "s3://us-cache/tenant-acme/"
region = "us-east-1"
permanent = true
workers = 20

[[upstream]]
url = "s3://eu-cache/tenant-acme/"
region = "eu-west-1"
permanent = true
workers = 10
```

### Configuration Naming Convention

**Config-backed options** use `--config-*` prefix:
- CLI: `--config-cache-dir /tmp/cache`
- Env: `FABRIK_CONFIG_CACHE_DIR=/tmp/cache`
- File: `cache.dir = "/tmp/cache"`

**Runtime-only options** have no prefix:
- `--config <path>` - config file path
- `--export-env` - export cache URLs as env vars
- `--help`, `--version`

### Environment Variable Fallbacks

Fabrik checks both `FABRIK_CONFIG_*` and standard environment variables:

| Config Option | FABRIK_CONFIG_* | Standard Env Var |
|---------------|----------------|------------------|
| S3 access key | `FABRIK_CONFIG_S3_ACCESS_KEY` | `AWS_ACCESS_KEY_ID` |
| S3 secret key | `FABRIK_CONFIG_S3_SECRET_KEY` | `AWS_SECRET_ACCESS_KEY` |
| S3 region | `FABRIK_CONFIG_S3_REGION` | `AWS_REGION` |

### Complete Flow Examples

**Scenario: CI build with Bazel (Layer 1 -> Layer 2 -> S3)**

1. **CI runner** runs:
   ```bash
   fabrik bazel -- build //...
   ```

2. On cache miss:
   - Bazel requests artifact from local Fabrik adapter (gRPC)
   - Layer 1 adapter checks local RocksDB → MISS
   - Layer 1 queries Layer 2 (regional server via Fabrik protocol) → ...

3. **Layer 2 (regional server)** receives request:
   - Layer 2 checks local RocksDB → MISS
   - Layer 2 queries S3 → HIT
   - Layer 2 downloads from S3, caches locally
   - Layer 2 returns artifact to Layer 1

4. Layer 1 receives artifact, caches locally in RocksDB
5. Layer 1 returns to Bazel via gRPC
6. Bazel receives cached artifact and continues build

**Generating config files:**
```bash
# Generate example Layer 1 config
fabrik config generate --template layer1 > .fabrik.toml

# Generate example server config
fabrik config generate --template server > /etc/fabrik/config.toml

# Validate config
fabrik config validate /etc/fabrik/config.toml

# Show effective configuration (merged from all sources)
fabrik config show --config config.toml --config-upstream s3://override
```

## Development Guidelines

### Rust Conventions
- Follow standard Rust conventions and idioms
- Use `rustfmt` for code formatting (enforced in CI)
- Use `clippy` for linting (zero warnings policy)
- Prioritize safety, idiomatic patterns, and zero-cost abstractions

### Project Principles
- **Performance**: Low latency (target: <10ms p99), high throughput
- **Reliability**: Data integrity, fault tolerance, graceful degradation
- **Transparency**: Cache layer fallback should be invisible to clients
- **Observability**: Rich metrics, structured logging, distributed tracing
- **Security**: JWT validation, secure defaults, defense in depth
- **Operational simplicity**: Single binary, hot-reloadable config, zero-downtime updates

### Testing Strategy
- Unit tests for core logic (cache eviction, JWT validation, protocol parsing)
- Integration tests with real RocksDB and S3 (using LocalStack)
- Protocol compliance tests for each build system
- Load/performance tests for latency and throughput benchmarks
- Chaos testing for failure scenarios (network partitions, disk full, etc.)

## Instructions for Claude

### When Working on This Codebase

1. **Ask Questions**: The project is in early stages - ask clarifying questions when requirements are ambiguous
2. **Rust Best Practices**: Recommend idiomatic Rust patterns and explain reasoning (user is new to Rust)
3. **Performance is Critical**: Always consider latency implications - this is hot path code
4. **Protocol Accuracy**: When implementing build system protocols, research and follow specifications exactly
5. **Zero-Latency Auth**: JWT validation must be local and fast - never add network calls
6. **Configuration Design**: Maintain precedence order (CLI > env vars > config file)
7. **Observability**: Suggest metrics, logs, and traces for operational visibility
8. **Testing**: Recommend tests for cache behavior, protocol compliance, security, and performance
9. **Documentation**: Document architectural decisions, trade-offs, and Rust patterns
10. **Update PLAN.md**: As you complete tasks, mark them done in PLAN.md and update the current phase

### Key Areas to Focus On

- **RocksDB integration**: Efficient use for caching, eviction policies, tuning
- **Protocol implementation**: HTTP REST (Gradle/Nx/TurboRepo) and gRPC (Bazel)
- **JWT validation**: Fast, secure, with key rotation support
- **Layer abstraction**: Clean separation between local/regional/S3 storage backends
- **Metrics exposition**: Prometheus-compatible metrics for Tuist to consume
- **Error handling**: Graceful degradation, retries, circuit breakers
- **Configuration**: Flexible, validated, hot-reloadable where possible

### Questions to Ask When Uncertain

- How should error scenarios be handled? (fail fast vs. degraded operation)
- What metrics/observability should be added for this feature?
- What are the performance implications? (latency, throughput, memory)
- How does this scale horizontally across multiple instances?
- What are the security implications?
- How will Tuist interact with this feature?

### Important Context

- **Tuist is the customer-facing layer** - Fabrik is infrastructure
- **Performance over features** - latency matters more than functionality in early stages
- **Operational simplicity** - Tuist will manage many instances, make it easy
- **Multi-region is future work** - design for it, but don't implement yet

## Future Roadmap

### Near-term (v1.0)
- [ ] Core caching with RocksDB (Layer 1 & 2)
- [ ] S3 storage backend (Layer 3)
- [ ] JWT authentication with RS256
- [ ] HTTP REST API for Gradle, Nx, TurboRepo
- [ ] gRPC API for Bazel
- [ ] Prometheus metrics endpoint
- [ ] Configuration via CLI/env/file
- [ ] Basic observability (structured logs)

### Mid-term (v2.0)
- [ ] Multi-region distribution/replication
- [ ] Advanced eviction policies (size + frequency + TTL)
- [ ] Distributed tracing (OpenTelemetry)
- [ ] Compression for storage efficiency
- [ ] Encryption at rest
- [ ] Rate limiting per customer (if needed)

### Long-term (v3.0+)
- [ ] Vite+ support (when available)
- [ ] Additional build systems (Buck2, Pants, etc.)
- [ ] Cache warming strategies
- [ ] Intelligent prefetching
- [ ] Cache analytics and insights
- [ ] Self-healing and auto-scaling

## Evolution

This document will evolve as the project matures. Update both CLAUDE.md and PLAN.md when:
- New build systems are supported
- Architecture decisions are finalized or changed
- Authentication model evolves
- Multi-region support is implemented
- New storage backends are added
- Operational patterns emerge
- Implementation phases are completed

**Workflow**: As you complete tasks in PLAN.md, update the "Current Phase" and mark tasks as done. If architectural decisions change, update both CLAUDE.md and the "Notes & Decisions" section in PLAN.md.
- Keep the documentation up to date whenever you change something adding or removing pages, or adjusting the content on existing ones. Make sure the navigation of the site is well designed such that's easy to navigate.
- Ensure cargo format and clippy pass before consider the work done