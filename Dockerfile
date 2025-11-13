# syntax=docker/dockerfile:1.4
# Multi-stage Dockerfile for Fabrik
# Optimized for fast builds with cargo-chef and BuildKit cache mounts
# Expected speedup: 5-10x faster on subsequent builds

# ============================================================================
# Stage 1: Chef Planner (generates dependency recipe)
# ============================================================================
FROM rust:1.91-bookworm AS chef

# Install cargo-chef from pre-built binary (much faster than compiling from source)
# This saves 15-20 minutes of build time
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    apt-get update && \
    apt-get install -y wget && \
    CARGO_CHEF_VERSION=0.1.68 && \
    ARCH=$(uname -m) && \
    if [ "$ARCH" = "x86_64" ]; then \
        CARGO_CHEF_ARCH="x86_64-unknown-linux-musl"; \
    elif [ "$ARCH" = "aarch64" ]; then \
        CARGO_CHEF_ARCH="aarch64-unknown-linux-musl"; \
    else \
        echo "Unsupported architecture: $ARCH" && exit 1; \
    fi && \
    wget -q "https://github.com/LukeMathWalker/cargo-chef/releases/download/v${CARGO_CHEF_VERSION}/cargo-chef-${CARGO_CHEF_ARCH}.tar.gz" && \
    tar -xzf "cargo-chef-${CARGO_CHEF_ARCH}.tar.gz" && \
    mv cargo-chef /usr/local/cargo/bin/ && \
    rm "cargo-chef-${CARGO_CHEF_ARCH}.tar.gz" && \
    cargo chef --version

WORKDIR /build

# ============================================================================
# Stage 2: Recipe (cache dependencies metadata)
# ============================================================================
FROM chef AS planner

# Install protoc 28.3 (needed for build.rs to run during planning)
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    apt-get update && \
    apt-get install -y curl unzip && \
    PROTOC_VERSION=28.3 && \
    PROTOC_ARCH=$(uname -m | sed 's/x86_64/x86_64/;s/aarch64/aarch_64/') && \
    curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-${PROTOC_ARCH}.zip && \
    unzip protoc-${PROTOC_VERSION}-linux-${PROTOC_ARCH}.zip -d /usr/local && \
    rm protoc-${PROTOC_VERSION}-linux-${PROTOC_ARCH}.zip && \
    chmod +x /usr/local/bin/protoc

COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY cbindgen.toml ./
COPY proto ./proto
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# ============================================================================
# Stage 3: Builder (builds dependencies and binary)
# ============================================================================
FROM chef AS builder

# Install build dependencies with cache mount
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    unzip \
    clang \
    lld

# Install protoc 28.3
RUN PROTOC_VERSION=28.3 && \
    PROTOC_ARCH=$(uname -m | sed 's/x86_64/x86_64/;s/aarch64/aarch_64/') && \
    curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-${PROTOC_ARCH}.zip && \
    unzip protoc-${PROTOC_VERSION}-linux-${PROTOC_ARCH}.zip -d /usr/local && \
    rm protoc-${PROTOC_VERSION}-linux-${PROTOC_ARCH}.zip && \
    chmod +x /usr/local/bin/protoc

# Copy recipe from planner
COPY --from=planner /build/recipe.json recipe.json

# Build dependencies with cache mounts for cargo registry and git repos
# This is the key optimization - dependencies are cached across builds!
# Disable LTO and optimize for faster builds (not runtime performance during deps build)
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    CARGO_PROFILE_RELEASE_LTO=off \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
    cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY cbindgen.toml ./
COPY proto ./proto
COPY src ./src

# Build the actual binary with optimizations
# LTO is disabled to prevent build hangs (minimal performance impact for a cache server)
# Symbol stripping reduces binary size
# More codegen units = faster compilation (slight runtime tradeoff)
ENV CARGO_PROFILE_RELEASE_STRIP=symbols
ENV CARGO_PROFILE_RELEASE_LTO=off
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --release --locked

# ============================================================================
# Stage 4: Runtime
# ============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl3

# Create a non-root user for running the application
RUN useradd -m -u 1000 -s /bin/bash fabrik

# Create necessary directories
RUN mkdir -p /data/fabrik/cache && \
    chown -R fabrik:fabrik /data/fabrik

# Copy the binary from builder
COPY --from=builder /build/target/release/fabrik /usr/local/bin/fabrik

# Set ownership
RUN chmod +x /usr/local/bin/fabrik

# Switch to non-root user
USER fabrik
WORKDIR /data/fabrik

# Expose ports
# 7070: Fabrik protocol (gRPC)
# 8888: Health API
# 9091: Metrics + Cache Query API + Admin API
EXPOSE 7070 8888 9091

# Default command (can be overridden)
ENTRYPOINT ["/usr/local/bin/fabrik"]
CMD ["server"]

# Metadata
LABEL org.opencontainers.image.title="Fabrik"
LABEL org.opencontainers.image.description="Multi-layer build cache infrastructure"
LABEL org.opencontainers.image.vendor="Tuist"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.source="https://github.com/tuist/fabrik"
