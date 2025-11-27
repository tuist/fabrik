# Fabrik

Vendor and environment-agnostic technology to optimize developer, CI, and agentic workflows.

> [!IMPORTANT]
> This project is in the ideation phase. We may open PRs and address issues, but we're not actively monitoring repository activity.

## 🎯 What is Fabrik?

Fabrik is **the Kubernetes of development environments**—a universal orchestration layer that optimizes builds, tests, and scripts across any environment. Whether you're running locally, in CI, or powering agentic workflows, Fabrik provides transparent caching and execution optimization without locking you into specific vendors or platforms.

**Think of it as:** The narrow waist between your workflows and infrastructure—a minimal, universal interface that any build tool, test runner, or script can plug into, and any storage backend or execution environment can support.

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

**Full documentation is available at: [https://fabrik.tuist.dev](https://fabrik.tuist.dev)**

Or read locally:
- [Getting Started Guide](./docs/getting-started.md)
- [Build System Integration](./docs/build-systems/)
- [CLI Reference](./docs/reference/cli.md)
- [Architecture](./docs/guide/architecture.md)

## ✨ Key Features

- 🌐 **Vendor Agnostic**: Works with any build system, test runner, or CI platform
- 🏢 **Environment Agnostic**: Seamless operation across local dev, CI/CD, and cloud environments
- 🤖 **AI-Ready**: Optimized for agentic coding workflows and automated development
- 🔥 **Transparent Optimization**: Intelligent caching with automatic fallback across storage tiers
- 🔧 **Universal Compatibility**: Supports Gradle, Bazel, Nx, TurboRepo, Xcode, and custom scripts
- 🚀 **P2P Cache Sharing**: Automatic discovery and sharing of build caches across local networks (1-5ms latency)
- ⚡ **High Performance**: Built in Rust with RocksDB for ultra-low latency (<10ms p99)
- 🎯 **Zero Configuration**: Automatically adapts to your environment
- 💎 **Open Source**: MPL-2.0 licensed—deploy anywhere, customize freely

## 🤝 Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development setup and guidelines.

## 📄 License

MPL-2.0 - See [LICENSE.md](./LICENSE.md) for details.

## 🔗 Related Projects

- [Tuist](https://tuist.dev) - Managed Fabrik infrastructure as a service
