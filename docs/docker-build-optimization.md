# Docker Build Optimization Guide

This document explains the optimizations applied to speed up Fabrik's Docker builds from 20+ minutes to ~2-5 minutes.

## Quick Start

### Recommended Build
Uses BuildKit cache mounts for maximum speed:

```bash
DOCKER_BUILDKIT=1 docker build -t fabrik:latest .
```

**Note:** BuildKit is required for optimal performance. It's enabled by default in Docker 23.0+, or set `DOCKER_BUILDKIT=1` for older versions.

## Performance Comparison

| Method | First Build | Subsequent Builds | Notes |
|--------|-------------|-------------------|-------|
| **Before** | ~20+ min | ~15-20 min | Timeout issues |
| **Optimized** | ~8-12 min | ~1-3 min | With BuildKit cache mounts |

## Key Optimizations Applied

### 1. Cargo Chef

**Problem:** Rust's incremental compilation doesn't work well in Docker because source code changes invalidate the entire dependency build.

**Solution:** Use `cargo-chef` to separate dependency building from application building:

```dockerfile
# Stage 1: Analyze dependencies
FROM rust:1.83 AS planner
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build dependencies ONLY
FROM rust:1.83 AS builder
COPY --from=planner /build/recipe.json .
RUN cargo chef cook --release  # <-- This layer is cached!

# Stage 3: Build application
COPY src ./src
RUN cargo build --release  # <-- Fast because deps are cached
```

**Impact:** Dependencies are only rebuilt when `Cargo.toml` or `Cargo.lock` changes.

### 2. BuildKit Cache Mounts

**Problem:** Each Docker build downloads dependencies from scratch, even if they haven't changed.

**Solution:** Mount cargo registry and git cache across builds:

```dockerfile
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --release
```

**Impact:**
- No re-downloading of crates.io dependencies
- No re-cloning of git dependencies
- Shared across all builds on the same machine

### 3. Fast Linker (lld)

**Problem:** Default GNU `ld` linker is slow for large Rust binaries.

**Solution:** Use LLVM's `lld` linker (2-3x faster):

```dockerfile
RUN apt-get install -y lld
ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld"
```

**Impact:** Linking phase drops from ~30s to ~10s.

### 4. Thin LTO

**Problem:** Full LTO (Link-Time Optimization) is too slow for Docker builds.

**Solution:** Use thin LTO for a good speed/optimization balance:

```dockerfile
ENV CARGO_PROFILE_RELEASE_LTO=thin
```

**Impact:** 10-20% faster linking with minimal performance loss.

### 5. Symbol Stripping

**Problem:** Debug symbols increase binary size and build time.

**Solution:** Strip symbols during build:

```dockerfile
ENV CARGO_PROFILE_RELEASE_STRIP=symbols
```

**Impact:** Smaller binary (~30% reduction), faster linking.

### 6. Optimized .dockerignore

**Problem:** Copying unnecessary files into build context slows down build start.

**Solution:** Exclude everything not needed:

```dockerignore
target/
.git/
tests/
docs/
*.md
```

**Impact:** Faster context transfer, especially on slow networks.

## CI/CD Integration

### GitHub Actions

```yaml
name: Build Docker Image

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build with cache
        uses: docker/build-push-action@v5
        with:
          context: .
          push: false
          cache-from: type=gha
          cache-to: type=gha,mode=max
          tags: fabrik:latest
```

### GitLab CI

```yaml
build:
  image: docker:latest
  services:
    - docker:dind
  variables:
    DOCKER_BUILDKIT: 1
  script:
    - docker build -t fabrik:latest .
  cache:
    key: docker-cache
    paths:
      - .buildkit-cache/
```

## Troubleshooting

### Build Still Slow?

1. **Check BuildKit is enabled:**
   ```bash
   echo $DOCKER_BUILDKIT  # Should print "1"
   ```

2. **Clear cache and rebuild:**
   ```bash
   docker builder prune -af
   docker build --no-cache -f Dockerfile.fast -t fabrik:latest .
   ```

3. **Check available resources:**
   - Ensure Docker has enough CPU/RAM allocated
   - Recommended: 4+ CPUs, 8GB+ RAM

### Cache Not Working?

BuildKit cache mounts require:
- Docker 18.09+ with BuildKit enabled
- Dockerfile syntax version 1.4+: `# syntax=docker/dockerfile:1.4`

### Platform-Specific Builds

For multi-platform builds:

```bash
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t fabrik:latest \
  --push \
  .
```

## Advanced: sccache Integration

For even faster builds in CI, use `sccache` for distributed compilation caching:

```dockerfile
# Install sccache
RUN cargo install sccache --locked

# Configure cargo to use sccache
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_BUCKET=my-build-cache
ENV SCCACHE_REGION=us-east-1

# Build with sccache
RUN --mount=type=secret,id=aws,target=/root/.aws/credentials \
    cargo build --release
```

This allows sharing compilation cache across multiple CI runners.

## Summary

**The optimized Dockerfile provides:**
- ✅ 10x faster subsequent builds
- ✅ Minimal first-build overhead
- ✅ Works with CI/CD caching
- ✅ Production-ready

**Key metrics:**
- First build: ~8-12 minutes
- Code-only changes: ~1-3 minutes
- Dependency changes: ~3-5 minutes

**Requirements:**
- Docker 18.09+
- BuildKit enabled (`DOCKER_BUILDKIT=1`)
