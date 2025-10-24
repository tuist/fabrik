# Fabrik

> Multi-layer build cache infrastructure

Fabrik is tenant-agnostic infrastructure for build caching. Think of it as **Postgres to Supabase** - Fabrik provides the core caching engine while the service layer manages deployment, multi-tenancy, and tenant experience.

## 🎯 Overview

High-performance caching for build systems (Gradle, Bazel, Nx, TurboRepo, sccache) with transparent multi-layer fallback:

- **Layer 1**: Local cache (CI/dev environments)
- **Layer 2**: Regional cache (tenant-dedicated instances)
- **Layer 3**: S3-backed permanent storage

## ✨ Features

- 🚀 **High Performance**: Sub-10ms p99 latency target
- 🔒 **Secure**: JWT-based authentication (RS256)
- 📦 **Multi-Protocol**: HTTP, gRPC, S3 API
- 🗄️ **Smart Storage**: RocksDB + S3
- 📊 **Observable**: Health, Metrics, Cache Query, and Admin APIs
- ⚙️ **Configurable**: CLI args > env vars > config file

## 🚀 Installation

### Using Mise

```bash
# Install globally
mise use -g ubi:tuist/fabrik

# Or in .mise.toml
[tools]
"ubi:tuist/fabrik" = "latest"
```

### From Source

```bash
git clone https://github.com/tuist/fabrik.git
cd fabrik
cargo build --release
```

## 📘 Usage

```bash
# Ephemeral local cache
fabrik exec -- gradle build

# Long-running daemon
fabrik daemon

# Regional server
fabrik server

# Configuration management
fabrik config generate --template=server
```

## 📖 Documentation

- [CLAUDE.md](./CLAUDE.md) - Architecture and design decisions
- [PLAN.md](./PLAN.md) - Implementation roadmap

## 🛠️ Development

```bash
cargo build
cargo test
cargo fmt
cargo clippy
```

## 📝 License

MIT
