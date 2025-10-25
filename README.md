# Fabrik

High-performance caching infrastructure for build systems.

## 🎯 Overview

Fabrik provides transparent multi-layer caching for build systems (Gradle, Bazel, Nx, TurboRepo, sccache) with automatic fallback:

### Multi-Layer Caching Strategy

**🔥 Hot Cache**
- In-process caching bound to the build lifecycle
- Caches in local or mounted volumes
- Automatically detects and uses CI caching capabilities (GitHub Actions Cache, etc.)
- Lowest latency (<5ms)

**🌡️ Warm Cache**
- Remote Fabrik instances
- Shared across team's machines
- Medium latency (~20ms)

**❄️ Cold Cache**
- S3-backed permanent storage
- Always available
- No eviction policy (permanent retention)
- Higher latency (~100ms) but unlimited capacity

**Transparent Fallback**: Cache misses automatically fall back to the next configured layer. Writes propagate through all configured layers.

## ✨ Features

- 🚀 **High Performance**: Sub-10ms p99 latency target
- 🔒 **Secure**: JWT-based authentication (RS256)
- 📦 **Multi-Protocol**: HTTP, gRPC, S3 API
- 🗄️ **Smart Storage**: Efficient disk + in-memory caching with S3 backend
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
fabrik bazel -- build //...

# Long-running daemon
fabrik daemon

# Remote cache server
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
