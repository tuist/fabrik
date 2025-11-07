# Fabrik

Open-source, multi-layer build cache infrastructure for modern build tools.

## 🎯 What is Fabrik?

Fabrik is a **technology to build** transparent, high-performance caching infrastructure for any build tool with remote caching capabilities. It provides a three-tier caching hierarchy (hot, warm, cold) that transparently accelerates builds across local development, CI/CD, and team environments.

**Think of it as:** The narrow waist between build tools and cache infrastructure—a minimal, universal interface that any build tool can plug into, and any storage backend can support.

## 🚀 Quick Start

### 1. Install Fabrik

**Using Mise (recommended):**

```bash
mise use -g ubi:tuist/fabrik
```

**Or download from [GitHub Releases](https://github.com/tuist/fabrik/releases)**

### 2. Set Up Shell Integration

**Bash:**
```bash
echo 'eval "$(fabrik activate bash)"' >> ~/.bashrc
source ~/.bashrc
```

**Zsh:**
```bash
echo 'eval "$(fabrik activate zsh)"' >> ~/.zshrc
source ~/.zshrc
```

**Fish:**
```bash
echo 'fabrik activate fish | source' >> ~/.config/fish/config.fish
source ~/.config/fish/config.fish
```

### 3. Initialize Your Project

```bash
cd ~/your-project
fabrik init
```

That's it! Your builds will now use the cache automatically.

## 📚 Documentation

**Full documentation is available at: [https://fabrik.dev](https://fabrik.dev)**

Or read locally:
- [Getting Started Guide](./docs/getting-started.md)
- [Build System Integration](./docs/build-systems/)
- [CLI Reference](./docs/reference/cli.md)
- [Architecture](./docs/guide/architecture.md)

## ✨ Key Features

- 🔥 **Transparent Caching**: Three-tier hierarchy with automatic fallback
- 🔧 **Universal Compatibility**: Supports Gradle, Bazel, Nx, TurboRepo, sccache, BuildKit, and more
- ⚡ **High Performance**: Built in Rust with RocksDB for ultra-low latency (<10ms p99)
- 🎯 **Zero Configuration**: Automatically detects CI environments
- 🌍 **Multi-Region**: Deploy dedicated instances in your preferred regions
- 🔒 **Secure**: JWT-based authentication
- 💎 **Open Source**: MPL-2.0 licensed

## 🏗️ Architecture

Fabrik implements a three-tier caching strategy:

1. **🔥 Hot Cache** - Local/CI, ultra-fast (<5ms)
2. **🌡️ Warm Cache** - Shared team cache (~20ms)
3. **❄️ Cold Cache** - S3-backed permanent storage (~100ms)

Cache misses automatically fall back to the next layer. Writes propagate through all layers.

## 🤝 Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development setup and guidelines.

## 📄 License

MPL-2.0 - See [LICENSE.md](./LICENSE.md) for details.

## 🔗 Related Projects

- [Tuist](https://tuist.dev) - Managed Fabrik infrastructure as a service
