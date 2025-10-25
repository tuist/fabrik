# Multi-stage Dockerfile for Fabrik
# Builds a minimal container image for running Fabrik cache server

# ============================================================================
# Stage 1: Builder
# ============================================================================
FROM rust:1.83-bookworm AS builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for building
RUN useradd -m -u 1000 builder

WORKDIR /build

# Copy dependency manifests first for better layer caching
COPY --chown=builder:builder Cargo.toml Cargo.lock ./
COPY --chown=builder:builder build.rs ./

# Create dummy source to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    chown -R builder:builder .

# Build dependencies only (this layer will be cached)
USER builder
RUN cargo build --release && \
    rm -rf src target/release/fabrik* target/release/deps/fabrik*

# Copy actual source code
COPY --chown=builder:builder src ./src
COPY --chown=builder:builder proto ./proto

# Build the actual binary
RUN cargo build --release --locked

# ============================================================================
# Stage 2: Runtime
# ============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

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
