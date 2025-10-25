# Fabrik

Open-source, multi-layer build cache technology for modern build systems.

## 🎯 What is Fabrik?

Fabrik is a **technology to build** transparent, high-performance caching infrastructure for modern build systems. It can be deployed and customized to optimize build performance across different environments.

**Think of it as:** The narrow waist between build systems and cache infrastructure—a minimal, universal interface that any build system can plug into, and any storage backend can support.

## 🤔 Why Fabrik?

The landscape of software development is transforming rapidly. With the rise of agentic coding and AI-assisted development, the amount of code we're producing is growing exponentially. As codebases expand, **sharing compile artifacts across environments has become more critical than ever.**

Modern build systems—Gradle, Bazel, Nx, TurboRepo—are being designed with caching capabilities built-in. They understand the value of reusing work across builds, teams, and CI pipelines. But having the capability isn't enough. These build systems need **infrastructure** to unlock their full potential.

**Fabrik is the technology to build that infrastructure.**

## ✨ Key Features

- 🔥 **Transparent Caching**: Three-tier caching hierarchy (hot, warm, cold) with automatic fallback
- 🔧 **Multiple Build Systems**: Supports Xcode, Bazel, and Gradle
- ⚡ **High Performance**: Built in Rust with RocksDB for ultra-low latency (<10ms p99)
- 🎯 **Zero Configuration**: Automatically detects CI environments and uses their cache capabilities
- 🌍 **Multi-Region**: Deploy dedicated instances in your preferred regions
- 🔒 **Secure**: JWT-based authentication with zero-latency validation
- 💎 **Open Source**: MPL-2.0 licensed for transparency and customization

## 🏗️ Architecture at a Glance

Fabrik implements a three-tier caching strategy:

1. **🔥 Hot Cache** - Build-local, ultra-fast, lowest latency (<5ms)
   - In-process caching bound to the build lifecycle
   - Caches in local or mounted volumes
   - Automatically detects and uses CI caching capabilities (GitHub Actions Cache, etc.)

2. **🌡️ Warm Cache** - Shared team cache, dedicated instances (~20ms)
   - Remote Fabrik instances
   - Shared across team's machines

3. **❄️ Cold Storage** - S3-backed permanent storage (~100ms)
   - Always available
   - No eviction policy (permanent retention)
   - Unlimited capacity

**Transparent Fallback**: Cache misses automatically fall back to the next configured layer. Writes propagate through all configured layers.

> [!IMPORTANT]
> **Infrastructure Challenge**
>
> To maximize performance, warm caches need to be deployed as close as possible to where builds happen. For example, you might have CI builds running in `us-east-1` while your developers are distributed across `eu-west-1`. Managing this multi-region infrastructure—provisioning instances, handling authentication, monitoring performance, and ensuring high availability—is complex.
>
> This is where services like [Tuist](https://tuist.dev) come in. Just as Supabase manages Postgres infrastructure, Tuist can manage Fabrik infrastructure for you, automatically deploying warm cache instances in the right regions and handling all the operational complexity.

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
