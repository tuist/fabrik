# Fabrik

Fabrik is the core infrastructure for build system caching and remote execution. Think of it as **Postgres to Supabase** - Fabrik provides the caching engine while the service layer manages deployment and multi-tenancy.

> [!NOTE]
> [Tuist](https://tuist.dev) provides managed hosting of Fabrik as a service, similar to how Supabase hosts Postgres.

## 🎯 Overview

High-performance caching for build systems (Gradle, Bazel, Nx, TurboRepo, sccache) with transparent multi-layer fallback strategy:

### Three-Layer Caching Hierarchy

**🔥 Hot Layer (Layer 1): Local Cache**
- Bound to build process lifecycle
- Deployed automatically in CI environments
- RocksDB for in-memory + disk caching
- Closest to the build, lowest latency (<5ms)
- Uses mounted volumes for persistence

**🌡️ Warm Layer (Layer 2): Regional Cache**
- Dedicated Fabrik instance per customer/project
- Deployed in customer's preferred region by Tuist
- RocksDB with mounted volumes
- Shared across team's machines
- Medium latency (~20ms)

**❄️ Cold Layer (Layer 3): Tuist Server**
- S3-backed permanent storage
- Always available, managed by Tuist
- No eviction policy (permanent retention)
- Multi-tenant with object key prefixes
- Higher latency (~100ms) but unlimited capacity

**Transparent Fallback**: Cache misses automatically fall back to the next layer. Writes propagate through all configured layers.

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
# Bazel with automatic cache
fabrik bazel build //...

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
