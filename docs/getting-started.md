# Getting Started with Fabrik

Fabrik provides transparent build caching for your existing build tools. This guide will get you set up in minutes.

## Step 1: Install Fabrik

**Using Mise:**

```bash
# Install Mise if you haven't already
curl https://mise.run | sh

# Install Fabrik
mise use -g ubi:tuist/fabrik
```

<details>
<summary>Alternative: Install from GitHub Releases</summary>

Download the latest release for your platform:

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

## Step 2: Set Up Shell Integration

Fabrik uses shell integration to automatically start cache daemons when you navigate into projects.

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

## Step 3: Verify Installation

Run the doctor command to verify everything is set up correctly:

```bash
fabrik doctor
```

You should see:
```
✅ Fabrik binary found
✅ Shell detected
✅ Shell integration configured
```

## Step 4: Initialize Your Project

Navigate to your project and run the interactive initialization:

```bash
cd ~/your-project
fabrik init
```

This will ask you:
- Cache directory location (default: `.fabrik/cache`)
- Maximum cache size (default: `5GB`)
- Whether you have a remote cache server (optional)

The command creates a `fabrik.toml` configuration file in your project root.

## Step 5: Choose Your Build System

Fabrik works with any build system that supports remote caching. Continue with the guide for your build system:

- **[🏗️ Gradle](./build-systems/gradle.md)** - Java, Kotlin, Android projects
- **[📦 Bazel](./build-systems/bazel.md)** - Multi-language monorepos
- **[📱 Xcode](./build-systems/xcode.md)** - iOS, macOS, watchOS, tvOS apps  
- **[⚡ Nx](./build-systems/nx.md)** - JavaScript/TypeScript monorepos
- **[📲 Metro](./build-systems/metro.md)** - React Native bundler

[View all build systems →](./build-systems/README.md)

## How It Works

Once set up, Fabrik runs transparently in the background:

```bash
cd ~/myproject
# → Daemon starts automatically
# → Environment variables exported
# → Build tools connect to cache

gradle build
# → Faster builds with caching! 🚀
```

Each project gets its own isolated daemon with unique ports - no conflicts, no configuration needed.

## Next Steps

- Follow your build system's integration guide (links above)
- See [CLI Reference](./cli-reference.md) for all available commands
- Read [Architecture](../CLAUDE.md) for how Fabrik works internally
