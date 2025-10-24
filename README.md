# Fabrik

> Layered infrastructure for build system caching and remote execution

Fabrik is the core infrastructure for build system caching and remote execution. Think of it as **Postgres to Supabase** - Fabrik provides the caching engine while the service layer manages deployment and multi-tenancy.

> [!NOTE]
> [Tuist](https://tuist.io) provides managed hosting of Fabrik as a service, similar to how Supabase hosts Postgres.

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
