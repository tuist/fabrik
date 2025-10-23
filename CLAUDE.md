# Fabrik - Multi-Layer Build Cache Infrastructure

## Project Overview

Fabrik is the foundational infrastructure for build caching, designed to be deployed and managed as a service by **Tuist**.

**Relationship to Tuist:**
> **Postgres is to Supabase** what **Fabrik is to Tuist**

Just as Supabase deploys and manages Postgres databases for customers, Tuist will deploy and manage Fabrik instances to provide build cache as a service. Fabrik is tenant-agnostic infrastructure; Tuist handles all customer logic, billing, and orchestration.

Fabrik provides a transparent, high-performance caching hierarchy to optimize build performance across different environments, supporting build systems like Gradle, Bazel, Nx, TurboRepo, and compiler caches like sccache (Cargo/Rust), with planned support for Vite+ when available.

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

### Build System Authentication Support

All target build systems natively support Bearer tokens or S3 credentials:

| Build System | Protocol | Auth Method | Environment Variable |
|--------------|----------|-------------|----------------------|
| **Gradle** | HTTP | `Authorization: Bearer <token>` or Basic Auth | Custom or `ORG_GRADLE_PROJECT_*` |
| **Bazel** | gRPC | `--remote_header=Authorization=Bearer $TOKEN` | `$TOKEN` (user-defined) |
| **Nx** | HTTP | `Authorization: Bearer <token>` | `NX_SELF_HOSTED_REMOTE_CACHE_ACCESS_TOKEN` |
| **TurboRepo** | HTTP | `Authorization: Bearer <token>` | `TURBO_TOKEN` |
| **sccache** | S3 API | AWS credentials or custom endpoint | `SCCACHE_BUCKET`, `SCCACHE_ENDPOINT` |

Fabrik must support HTTP, gRPC, and S3-compatible protocols to accommodate all build systems.

## Analytics & Observability

### Metrics Interface

**Design: In-memory aggregation + pull-based metrics**

Fabrik exposes metrics via HTTP endpoint (e.g., `/metrics`) that Tuist polls periodically (e.g., every 30 seconds). This allows Tuist to:
- Monitor instance health
- Generate customer billing data
- Provide analytics dashboards (like Supabase's table editor/analytics)

**Metrics to track:**
- **Cache performance**: hits/misses (total + windowed), hit ratio
- **Storage**: bytes used, object count, growth rate
- **Bandwidth**: upload/download bytes per time window
- **Latency**: p50, p95, p99 for GET/PUT operations
- **Errors**: error rates by type (auth failures, storage errors, etc.)
- **Connections**: active connections, request rate
- **Evictions**: objects evicted, eviction rate

**Format**: Prometheus format or custom JSON (configurable)

**Optional: Async event stream** for critical events that need immediate attention (auth failures, disk full, crashes) without blocking cache operations.

**Implementation note:** Metrics aggregation happens in-memory with minimal overhead. Fabrik exposes the data; Tuist processes and augments it with customer context.

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

### Supported Build Systems (Initial Focus)

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

**Protocol Support:** Fabrik must implement both HTTP and gRPC servers, plus S3-compatible API to support all build systems.

## Configuration

### Configuration Precedence (industry standard)
1. Command-line arguments (highest priority)
2. Environment variables
3. Configuration file (lowest priority)

### Design Philosophy
- **Single binary** with configurable behavior via flags
- **No predefined "modes"** - users configure which layers to enable and how they behave
- **Flexible deployment**: Same binary can be Layer 1 (local), Layer 2 (regional), or Layer 3 (S3)

### Example Configurations

**Layer 1 (Local CI cache):**
```bash
fabrik server \
  --storage-backend=rocksdb \
  --rocksdb-path=/mnt/cache \
  --max-cache-size=10GB \
  --upstream-url=https://regional.fabrik.example.com \
  --jwt-public-key=/etc/fabrik/public-key.pem
```

**Layer 2 (Regional dedicated instance):**
```bash
fabrik server \
  --storage-backend=rocksdb \
  --rocksdb-path=/data/cache \
  --max-cache-size=100GB \
  --upstream-url=https://tuist-server.example.com \
  --jwt-public-key=/etc/fabrik/public-key.pem \
  --metrics-port=9090
```

**Layer 3 (S3-backed Tuist server):**
```bash
fabrik server \
  --storage-backend=s3 \
  --s3-bucket=tuist-build-cache \
  --s3-prefix=customer-{customer_id}/ \
  --jwt-public-key=/etc/fabrik/public-key.pem \
  --metrics-port=9090
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
