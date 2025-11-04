# Fabrik

Open-source, multi-layer build cache infrastructure for modern build tools.

## 🎯 What is Fabrik?

Fabrik is a **technology to build** transparent, high-performance caching infrastructure for any build tool with remote caching capabilities. It can be deployed and customized to optimize build performance across different environments.

**Think of it as:** The narrow waist between build tools and cache infrastructure—a minimal, universal interface that any build tool can plug into, and any storage backend can support.

## 🤔 Why Fabrik?

The landscape of software development is transforming rapidly. With the rise of agentic coding and AI-assisted development, the amount of code we're producing is growing exponentially. As codebases expand, **sharing compile artifacts across environments has become more critical than ever.**

Modern build tools—build systems like Gradle, Bazel, Nx, TurboRepo, compiler caches like sccache, and container build tools like BuildKit—are being designed with remote caching capabilities built-in. They understand the value of reusing work across builds, teams, and CI pipelines. But having the capability isn't enough. These tools need **infrastructure** to unlock their full potential.

**Fabrik is the technology to build that infrastructure.**

## ✨ Key Features

- 🔥 **Transparent Caching**: Three-tier caching hierarchy (hot, warm, cold) with automatic fallback
- 🔧 **Universal Compatibility**: Supports any build tool with remote caching (Gradle, Bazel, Nx, TurboRepo, sccache, BuildKit, and more)
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

3. **❄️ Cold Cache** - S3-backed permanent storage (~100ms)
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

## 🚀 Getting Started

### Step 1: Install Fabrik

**Using Mise (Recommended)**

```bash
# Install Mise if you haven't already
curl https://mise.run | sh

# Install Fabrik
mise use -g ubi:tuist/fabrik
```

<details>
<summary>Alternative: Install from GitHub Releases</summary>

Download the latest release for your platform from [GitHub Releases](https://github.com/tuist/fabrik/releases):

```bash
# macOS (ARM)
curl -L https://github.com/tuist/fabrik/releases/latest/download/fabrik-aarch64-apple-darwin.tar.gz | tar xz
sudo mv fabrik /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/tuist/fabrik/releases/latest/download/fabrik-x86_64-apple-darwin.tar.gz | tar xz
sudo mv fabrik /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/tuist/fabrik/releases/latest/download/fabrik-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv fabrik /usr/local/bin/
```

</details>

<details>
<summary>Alternative: Docker</summary>

```bash
# Pull the latest image
docker pull ghcr.io/tuist/fabrik:latest

# Run the server
docker run -p 7070:7070 -p 8888:8888 -p 9091:9091 ghcr.io/tuist/fabrik:latest server
```

**Docker Registry**: [ghcr.io/tuist/fabrik](https://github.com/tuist/fabrik/pkgs/container/fabrik)

</details>

<details>
<summary>Alternative: Build from Source</summary>

```bash
git clone https://github.com/tuist/fabrik.git
cd fabrik
cargo build --release
sudo cp target/release/fabrik /usr/local/bin/
```

</details>

### Step 2: Set Up Shell Integration (Required)

Fabrik uses shell integration to automatically start cache daemons when you navigate into projects. This step is **required** for Fabrik to work.

**For Bash:**
```bash
echo 'eval "$(fabrik activate bash)"' >> ~/.bashrc
source ~/.bashrc
```

**For Zsh:**
```bash
echo 'eval "$(fabrik activate zsh)"' >> ~/.zshrc
source ~/.zshrc
```

**For Fish:**
```bash
echo 'fabrik activate fish | source' >> ~/.config/fish/config.fish
source ~/.config/fish/config.fish
```

### Step 3: Verify Installation

Run the doctor command to verify everything is configured correctly:

```bash
fabrik doctor
```

You should see:
```
✅ Fabrik binary found
✅ Shell detected
✅ Shell integration configured
```

### Step 4: Configure Your Project

Create a `.fabrik.toml` file in your project root:

```toml
[cache]
dir = ".fabrik/cache"
max_size = "5GB"

# Optional: Configure upstream cache
# [[upstream]]
# url = "grpc://cache.tuist.io:7070"
# timeout = "30s"
```

### Step 5: Choose Your Build System

Fabrik works with any build system that supports remote caching. Follow the guide for your build system:

- **[Gradle →](docs/build-systems/gradle.md)** - Java, Kotlin, Android projects
- **[Bazel →](docs/build-systems/bazel.md)** - Multi-language monorepos
- **[Nx →](docs/build-systems/nx.md)** - JavaScript/TypeScript monorepos
- **[TurboRepo →](docs/build-systems/turborepo.md)** - JavaScript/TypeScript monorepos
- **[Xcode →](docs/build-systems/xcode.md)** - iOS, macOS apps
- **[sccache →](docs/build-systems/sccache.md)** - Rust compiler cache

## 💡 How It Works

Once shell integration is set up, Fabrik automatically manages cache daemons for you:

```bash
# Navigate to your project
cd ~/myproject

# Daemon automatically starts (if .fabrik.toml exists)
# Build tools automatically use the cache
gradle build    # ✅ Uses cache
nx build        # ✅ Uses cache
xcodebuild      # ✅ Uses cache
```

**Behind the scenes:**
1. Shell hook detects `.fabrik.toml`
2. Daemon starts with random available ports (no conflicts!)
3. Environment variables exported automatically
4. Build tools read env vars and connect to daemon
5. Cache hits = faster builds! 🚀

**Multi-Project Support:**
Each project gets its own isolated daemon:

```bash
# Terminal 1
cd ~/project-a
gradle build  # Uses daemon on ports 54321/54322

# Terminal 2 (simultaneously)
cd ~/project-b  
gradle build  # Uses different daemon on ports 54401/54402
```

## 📚 Documentation

- **[CLAUDE.md](./CLAUDE.md)** - Architecture and design decisions
- **[CLI Reference](./docs/cli-reference.md)** - Command-line interface documentation
- **[Build System Integration](./docs/build-systems/)** - Integration guides for specific build systems
- **[PLAN.md](./PLAN.md)** - Implementation roadmap

## 🛠️ Development

```bash
# Build the project
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Run doctor command for debugging
cargo run -- doctor --verbose
```

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## 📄 License

MPL-2.0 - See [LICENSE](./LICENSE) for details.

## 🙏 Acknowledgments

Fabrik is built on the shoulders of giants:
- [RocksDB](https://rocksdb.org/) - High-performance embedded database
- [Tokio](https://tokio.rs/) - Async runtime for Rust
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Tonic](https://github.com/hyperium/tonic) - gRPC framework

## 🔗 Related Projects

- [Tuist](https://tuist.dev) - Managed Fabrik infrastructure as a service
- [Bazel](https://bazel.build) - Build system with remote execution
- [Gradle](https://gradle.org) - Build automation tool
- [Nx](https://nx.dev) - Smart monorepo build system
