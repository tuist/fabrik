# Fabrik Implementation Plan

> **Status**: In Progress
> **Last Updated**: 2025-10-23
> **Current Phase**: Phase 0 - Project Setup (In Progress)

This document tracks the implementation roadmap for Fabrik. Update this file as phases complete and requirements evolve.

---

## Phase 0: Project Setup & Foundation

**Goal**: Bootstrap the Rust project with essential infrastructure

### Tasks
- [x] Initialize Rust project with Cargo
- [x] Add mise configuration for Rust toolchain
- [x] Create CLI with clap (basic help menu and server command structure)
- [x] Create basic README with build/run instructions
- [x] Configure CI/CD pipeline (GitHub Actions)
  - [x] `cargo fmt` check
  - [x] `cargo clippy` with zero warnings
  - [x] `cargo test`
  - [x] `cargo build --release`
  - [x] Automated releases with git-cliff
  - [x] Multi-platform binary builds (Linux, macOS, Windows)
  - [x] SHA256 checksums generation
- [ ] Set up project structure (modules: cache, storage, auth, server, config)
- [ ] Set up development tools
  - [ ] `.editorconfig` or similar
  - [ ] Pre-commit hooks (optional)
- [ ] Add license file (if applicable)

**Dependencies added**:
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**Progress**: Basic CLI structure complete and compiles successfully. Server command accepts all planned configuration flags.

**Deliverable**: Buildable Rust project with CI passing

---

## Phase 1: Configuration System

**Goal**: Implement flexible configuration with precedence (CLI > env > file)

### Tasks
- [ ] Define configuration schema (struct)
  - [ ] Storage backend selection (rocksdb, s3)
  - [ ] RocksDB options (path, max size)
  - [ ] S3 options (bucket, prefix, region)
  - [ ] Upstream URL (for layer fallback)
  - [ ] JWT public key path or JWKS URL
  - [ ] Server options (HTTP port, gRPC port, metrics port)
- [ ] Implement CLI argument parsing (`clap` crate)
- [ ] Implement environment variable parsing
- [ ] Implement config file parsing (TOML or YAML)
- [ ] Implement precedence logic (merge CLI > env > file)
- [ ] Add validation for required fields
- [ ] Write unit tests for configuration merging
- [ ] Document configuration options in README

**Dependencies**:
```toml
clap = { version = "4", features = ["derive"] }
config = "0.14"  # or similar for file parsing
```

**Deliverable**: Working configuration system with tests

---

## Phase 2: JWT Authentication

**Goal**: Implement zero-latency JWT validation with RS256

### Tasks
- [ ] Create `auth` module
- [ ] Implement JWT validation using RS256
  - [ ] Load public key from file
  - [ ] Support JWKS URL (optional, future)
  - [ ] Validate signature
  - [ ] Validate expiry (`exp` claim)
  - [ ] Extract claims (customer_id, permissions, etc.)
- [ ] Implement hot-reload for public key
  - [ ] File watch or SIGHUP signal handler
- [ ] Support multiple active keys (identified by `kid`)
- [ ] Create middleware for HTTP/gRPC servers
- [ ] Write unit tests with test fixtures (valid/expired/invalid JWTs)
- [ ] Write integration test with real JWT generation
- [ ] Add benchmarks to ensure <1ms validation time

**Dependencies**:
```toml
jsonwebtoken = "9"
```

**Deliverable**: Fast, secure JWT validation with tests and benchmarks

---

## Phase 3: Storage Abstraction Layer

**Goal**: Create clean abstraction for different storage backends

### Tasks
- [ ] Define `Storage` trait with methods:
  - [ ] `get(key: &str) -> Result<Option<Vec<u8>>>`
  - [ ] `put(key: &str, value: Vec<u8>) -> Result<()>`
  - [ ] `exists(key: &str) -> Result<bool>`
  - [ ] `delete(key: &str) -> Result<()>`
  - [ ] `size() -> Result<u64>`
- [ ] Implement `RocksDBStorage` backend
  - [ ] Initialize RocksDB with path
  - [ ] Configure LRU/LFU eviction
  - [ ] Implement size limits
  - [ ] Handle errors gracefully
- [ ] Implement `S3Storage` backend
  - [ ] Initialize AWS S3 client
  - [ ] Support custom endpoints (for LocalStack testing)
  - [ ] Handle prefixes for customer isolation
  - [ ] Implement retries and error handling
- [ ] Implement `LayeredStorage` (cascade through multiple backends)
  - [ ] Try local → upstream → S3
  - [ ] Write-through to all layers
  - [ ] Pull and cache on miss
- [ ] Write unit tests for each storage backend
- [ ] Write integration tests with real RocksDB and LocalStack
- [ ] Add metrics hooks (cache hits/misses)

**Dependencies**:
```toml
rocksdb = "0.22"
aws-sdk-s3 = "1"
aws-config = "1"
```

**Deliverable**: Working storage layer with RocksDB and S3 support

---

## Phase 4: HTTP REST API Server

**Goal**: Implement HTTP server for Gradle, Nx, and TurboRepo

### Tasks
- [ ] Choose HTTP framework (`axum` recommended)
- [ ] Create `http_server` module
- [ ] Implement routes:
  - [ ] `GET /cache/:hash` - retrieve artifact
  - [ ] `PUT /cache/:hash` - store artifact
  - [ ] `HEAD /cache/:hash` - check existence
  - [ ] `GET /v8/artifacts/:hash` - TurboRepo specific
  - [ ] `PUT /v8/artifacts/:hash` - TurboRepo specific
- [ ] Add JWT authentication middleware
- [ ] Extract hash from URL, read/write to storage layer
- [ ] Handle content streaming for large artifacts
- [ ] Implement proper HTTP status codes (200, 404, 401, 500, etc.)
- [ ] Add request logging (structured logs)
- [ ] Add metrics middleware (request count, latency)
- [ ] Write integration tests for each endpoint
- [ ] Test with real Gradle/Nx/TurboRepo clients (optional, future)

**Dependencies**:
```toml
axum = "0.7"
tower = "0.5"
tower-http = { version = "0.5", features = ["trace"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Deliverable**: Working HTTP API for cache operations

---

## Phase 5: gRPC Server for Bazel

**Goal**: Implement Bazel Remote Execution API

### Tasks
- [ ] Research Bazel Remote Execution API specification
  - [ ] Download proto files from Bazel repository
  - [ ] Understand ContentAddressableStorage (CAS)
  - [ ] Understand ActionCache
  - [ ] Understand Capabilities service
- [ ] Set up `tonic` with proto compilation
- [ ] Create `grpc_server` module
- [ ] Implement ContentAddressableStorage service
  - [ ] `FindMissingBlobs`
  - [ ] `BatchUpdateBlobs`
  - [ ] `BatchReadBlobs`
  - [ ] `GetTree` (if needed)
- [ ] Implement ActionCache service
  - [ ] `GetActionResult`
  - [ ] `UpdateActionResult`
- [ ] Implement Capabilities service
  - [ ] Return supported features
- [ ] Add JWT authentication via gRPC metadata
- [ ] Map gRPC calls to storage layer
- [ ] Add request logging and metrics
- [ ] Write integration tests
- [ ] Test with real Bazel client (optional, future)

**Dependencies**:
```toml
tonic = "0.12"
prost = "0.13"
tonic-build = "0.12"
```

**Deliverable**: Working gRPC API for Bazel

---

## Phase 6: Metrics & Observability

**Goal**: Expose metrics endpoint for Tuist to consume

### Tasks
- [ ] Create `metrics` module
- [ ] Define metrics to track:
  - [ ] Cache hits/misses (counter)
  - [ ] Storage bytes used (gauge)
  - [ ] Object count (gauge)
  - [ ] Request latency (histogram, p50/p95/p99)
  - [ ] Upload/download bandwidth (counter)
  - [ ] Error rate by type (counter)
  - [ ] Active connections (gauge)
  - [ ] Evictions (counter)
- [ ] Implement metrics collection with `prometheus` crate
- [ ] Add `GET /metrics` endpoint (Prometheus format)
- [ ] Instrument HTTP and gRPC servers
- [ ] Instrument storage layer
- [ ] Add structured logging with `tracing`
- [ ] Write tests for metrics accuracy
- [ ] Document metrics in README

**Dependencies**:
```toml
prometheus = "0.13"
lazy_static = "1.4"  # for global metrics
```

**Deliverable**: Prometheus-compatible metrics endpoint

---

## Phase 7: Integration & End-to-End Testing

**Goal**: Ensure all components work together correctly

### Tasks
- [ ] Write end-to-end tests:
  - [ ] Start Fabrik server (HTTP + gRPC)
  - [ ] Authenticate with JWT
  - [ ] Store artifact via HTTP
  - [ ] Retrieve artifact via HTTP
  - [ ] Verify layered storage (local → upstream → S3)
  - [ ] Check metrics endpoint
- [ ] Test configuration loading (CLI, env, file)
- [ ] Test JWT expiry and invalid tokens
- [ ] Test storage backend failures (graceful degradation)
- [ ] Test with realistic build artifacts (large files, many files)
- [ ] Performance testing:
  - [ ] Measure p99 latency for GET/PUT operations
  - [ ] Measure throughput (requests/second)
  - [ ] Ensure <10ms p99 for cache hits
- [ ] Load testing with tools like `wrk` or `k6`
- [ ] Document testing approach in README

**Deliverable**: Comprehensive test suite with performance benchmarks

---

## Phase 8: Docker & Deployment

**Goal**: Make Fabrik easy to deploy

### Tasks
- [ ] Create `Dockerfile` with multi-stage build
  - [ ] Build stage (Rust compilation)
  - [ ] Runtime stage (minimal image with binary)
- [ ] Create `docker-compose.yml` for local testing
  - [ ] Fabrik service
  - [ ] LocalStack (S3 emulation)
  - [ ] Example configuration
- [ ] Write deployment guide for different scenarios:
  - [ ] Docker on single host
  - [ ] Kubernetes (basic manifests)
  - [ ] Cloud providers (AWS, GCP, Azure notes)
- [ ] Add health check endpoint (`GET /health`)
- [ ] Add readiness check endpoint (`GET /ready`)
- [ ] Document environment variables for deployment
- [ ] Test deployment in Docker locally

**Deliverable**: Production-ready Docker image and deployment docs

---

## Phase 9: Documentation & Polish

**Goal**: Make Fabrik easy to understand and use

### Tasks
- [ ] Write comprehensive README
  - [ ] What is Fabrik?
  - [ ] Quick start guide
  - [ ] Configuration reference
  - [ ] API documentation (HTTP and gRPC)
  - [ ] Deployment guide
- [ ] Add inline code documentation (rustdoc)
- [ ] Generate and publish API docs (`cargo doc`)
- [ ] Create example configurations for each layer
- [ ] Write troubleshooting guide
- [ ] Add architecture diagram (optional)
- [ ] Create CHANGELOG.md for version tracking
- [ ] Tag v1.0.0 release

**Deliverable**: Well-documented v1.0.0 release

---

## Future Phases (Post v1.0)

### Phase 10: Multi-Region Support
- [ ] Design replication strategy
- [ ] Implement cross-region artifact distribution
- [ ] Add region-aware routing
- [ ] Test failover scenarios

### Phase 11: Advanced Features
- [ ] Compression support (gzip, zstd)
- [ ] Encryption at rest
- [ ] Distributed tracing (OpenTelemetry)
- [ ] Cache warming strategies
- [ ] Intelligent prefetching

### Phase 12: Additional Build Systems
- [ ] Vite+ support (when available - currently in early access)
- [ ] Buck2 support
- [ ] Pants support
- [ ] Research other build systems

---

## Progress Tracking

### Completed Phases
- ✅ None yet (Phase 0 in progress)

### Current Phase
- 🚧 Phase 0: Project Setup (5/7 tasks complete)
  - ✅ Rust project initialized with Cargo
  - ✅ mise configuration added for toolchain management (rust + git-cliff)
  - ✅ CLI structure with clap (help menu working)
  - ✅ README with build/run instructions
  - ✅ CI/CD pipeline (GitHub Actions with automated releases)
  - ⏳ Module structure
  - ⏳ Development tools
  - ⏳ License

### Blocked Items
- None currently

---

## Notes & Decisions

### Key Architectural Decisions
- **Single binary design**: Simplifies deployment and maintenance
- **JWT with RS256**: Balances security and performance
- **RocksDB for hot cache**: Best performance/features for frequency-based eviction
- **S3 for cold storage**: Standard, reliable, cost-effective
- **Both HTTP and gRPC**: Required for build system compatibility
- **clap for CLI**: Using derive macros for clean, type-safe argument parsing
- **mise for toolchain**: Ensures consistent Rust version across developers

### Implementation Notes (2025-10-23)
- CLI structure includes server command with all anticipated flags
- Chose edition "2021" for Rust (stable, well-supported)
- Default ports: HTTP 8080, gRPC 9090, metrics 9091
- Server command ready to accept all configuration options from CLAUDE.md
- README.md created with emoji sections for better readability
- GitHub repository description and topics updated for discoverability
- Added support documentation for sccache (Cargo/Rust compiler cache)
- Vite+ added as planned future support (currently in early access)
- CI/CD pipeline configured with GitHub Actions:
  - CI workflow: fmt, clippy, test, build on all platforms
  - Release workflow: automated releases using git-cliff with semantic versioning
  - Multi-platform builds: Linux (x86_64, ARM64, ARMv7 GNU/musl), macOS (x86_64, ARM64), Windows (x86_64, ARM64)
  - Multiple archive formats: tar.gz, tar.xz, tar.zst for Unix, zip for Windows
  - SHA256 checksums generated for all artifacts
  - CHANGELOG.md automatically updated on releases

### Open Questions
- Should we support custom storage backends via plugins? (Future consideration)
- What's the optimal default cache size for Layer 1/2? (Tunable, needs testing)
- Should we implement cache warming on startup? (Post v1.0)

### Performance Targets
- **Latency**: <10ms p99 for cache hits (local storage)
- **Throughput**: Handle 1000s of requests/second per instance
- **JWT validation**: <1ms per token

---

## References

- [CLAUDE.md](./CLAUDE.md) - Full architectural documentation
- [README.md](./README.md) - User-facing documentation
- [Bazel Remote Execution API](https://bazel.build/remote/rbe)
- [Gradle Build Cache](https://docs.gradle.org/current/userguide/build_cache.html)
- [Nx Remote Cache](https://nx.dev/docs/guides/tasks--caching/self-hosted-caching)
- [TurboRepo Remote Cache](https://turborepo.com/docs/core-concepts/remote-caching)
- [sccache (Cargo/Rust)](https://github.com/mozilla/sccache)
- [Vite+](https://viteplus.dev) - In early access
