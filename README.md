# Fabrik

Open-source, multi-layer build cache infrastructure for modern build tools.

## What is Fabrik?

Fabrik is a transparent, high-performance caching infrastructure for any build tool with remote caching capabilities—build systems like Gradle, Bazel, Nx, TurboRepo, compiler caches like sccache, and container build tools like BuildKit.

**Think of it as:** The narrow waist between build tools and cache infrastructure—a minimal, universal interface that any build tool can plug into, and any storage backend can support.

## Installation

**Using Mise (Recommended):**

```bash
# Install Mise
curl https://mise.run | sh

# Install Fabrik
mise use -g ubi:tuist/fabrik
```

<details>
<summary>Alternative: Install from GitHub Releases</summary>

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
docker pull ghcr.io/tuist/fabrik:latest
docker run -p 7070:7070 ghcr.io/tuist/fabrik:latest server
```

</details>

## Documentation

**[📚 Read the complete documentation →](https://tuist.dev/fabrik)**

- [Getting Started Guide](https://tuist.dev/fabrik/getting-started)
- [Build System Integration](https://tuist.dev/fabrik/build-systems/)
- [CLI Reference](https://tuist.dev/fabrik/reference/cli)
- [Architecture](https://tuist.dev/fabrik/guide/architecture)

## Quick Start

```bash
# Install
mise use -g ubi:tuist/fabrik

# Set up shell integration
echo 'eval "$(fabrik activate zsh)"' >> ~/.zshrc
source ~/.zshrc

# Initialize your project
cd ~/your-project
fabrik init

# Start building
gradle build  # or your build command
```

## Development

```bash
# Install dependencies
mise install

# Build
cargo build

# Run unit tests
cargo test --lib

# Run acceptance tests
mise exec -- cargo test --test bazel_integration_test
mise exec -- cargo test --test gradle_acceptance
cargo test --test xcode_acceptance

# Format and lint
cargo fmt
cargo clippy
```

## License

MPL-2.0 - See [LICENSE](./LICENSE) for details.

## Links

- **[Documentation](https://tuist.dev/fabrik)** - Complete guide
- **[GitHub](https://github.com/tuist/fabrik)** - Source code
- **[Issues](https://github.com/tuist/fabrik/issues)** - Bug reports and features
- **[Tuist](https://tuist.dev)** - Managed Fabrik infrastructure as a service
