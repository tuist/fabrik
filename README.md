# Fabrik

> Multi-layer build cache infrastructure

Fabrik is the foundational infrastructure for build caching, designed to be deployed and managed as a service. Think of it as **Postgres to Supabase** - Fabrik provides the core caching engine while Tuist manages deployment and customer experience.

## 🎯 What is Fabrik?

Fabrik provides transparent, high-performance caching for build systems like Gradle, Bazel, Nx, TurboRepo, and compiler caches like sccache (Cargo/Rust). It supports a three-layer caching strategy:

- **Layer 1**: Local cache (CI environments with mounted volumes)
- **Layer 2**: Regional cache (dedicated instances per customer)
- **Layer 3**: S3-backed permanent storage

## ✨ Features

- 🚀 **High Performance**: Sub-10ms p99 latency for cache hits
- 🔒 **Secure**: JWT-based authentication with zero-latency validation
- 📦 **Multi-Protocol**: Supports HTTP (Gradle, Nx, TurboRepo), gRPC (Bazel), and S3 API (sccache)
- 🗄️ **Smart Storage**: RocksDB for hot cache with LRU/LFU eviction, S3 for cold storage
- 🔄 **Transparent Fallback**: Automatic cascading through cache layers
- 📊 **Observable**: Prometheus metrics endpoint for monitoring
- ⚙️ **Configurable**: Single binary with flexible deployment options
- 🔮 **Future-Ready**: Planned support for Vite+ when available

## 🚀 Quick Start

### Prerequisites

- Rust 1.90+ (managed via mise)

### Installation

```bash
# Install dependencies
mise install

# Build the project
mise exec -- cargo build --release

# Run Fabrik
mise exec -- cargo run -- server --help
```

### Running the Server

```bash
# Start with RocksDB storage
fabrik server \
  --storage-backend=rocksdb \
  --rocksdb-path=/tmp/cache \
  --max-cache-size=10GB \
  --jwt-public-key=/path/to/public-key.pem

# Start with S3 storage
fabrik server \
  --storage-backend=s3 \
  --s3-bucket=my-build-cache \
  --jwt-public-key=/path/to/public-key.pem
```

## 📖 Documentation

- **[CLAUDE.md](./CLAUDE.md)** - Full architectural documentation and guidelines
- **[PLAN.md](./PLAN.md)** - Implementation roadmap and progress tracking

## 🛠️ Development

### Building

```bash
mise exec -- cargo build
```

### Running Tests

```bash
mise exec -- cargo test
```

### Code Quality

```bash
# Format code
mise exec -- cargo fmt

# Run linter
mise exec -- cargo clippy
```

## 🏗️ Project Status

**Current Phase**: Phase 0 - Project Setup (In Progress)

See [PLAN.md](./PLAN.md) for detailed progress tracking.

## 🤝 Contributing

This project is in early development. Contributions will be welcome once the initial architecture is established.

## 📝 License

MIT

## 🔗 Related Projects

- [Tuist](https://github.com/tuist/tuist) - The managed service built on top of Fabrik
