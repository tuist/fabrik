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

<div style="display: grid; gap: 1rem; margin: 2rem 0;">
  <a href="./build-systems/gradle" style="display: flex; align-items: center; gap: 0.75rem; padding: 1rem; border: 1px solid var(--vp-c-divider); border-radius: 8px; text-decoration: none;">
    <img src="/images/gradle.svg" style="width: 32px; height: 32px;" alt="Gradle">
    <div>
      <strong>Gradle</strong>
      <div style="font-size: 0.875rem; color: var(--vp-c-text-2);">Java, Kotlin, Android projects</div>
    </div>
  </a>
  
  <a href="./build-systems/bazel" style="display: flex; align-items: center; gap: 0.75rem; padding: 1rem; border: 1px solid var(--vp-c-divider); border-radius: 8px; text-decoration: none;">
    <img src="/images/bazel.svg" style="width: 32px; height: 32px;" alt="Bazel">
    <div>
      <strong>Bazel</strong>
      <div style="font-size: 0.875rem; color: var(--vp-c-text-2);">Multi-language monorepos</div>
    </div>
  </a>
  
  <a href="./build-systems/xcode" style="display: flex; align-items: center; gap: 0.75rem; padding: 1rem; border: 1px solid var(--vp-c-divider); border-radius: 8px; text-decoration: none;">
    <img src="/images/xcode.png" style="width: 32px; height: 32px;" alt="Xcode">
    <div>
      <strong>Xcode</strong>
      <div style="font-size: 0.875rem; color: var(--vp-c-text-2);">iOS, macOS, watchOS, tvOS apps</div>
    </div>
  </a>
  
  <a href="./build-systems/nx" style="display: flex; align-items: center; gap: 0.75rem; padding: 1rem; border: 1px solid var(--vp-c-divider); border-radius: 8px; text-decoration: none;">
    <img src="/images/nx.svg" style="width: 32px; height: 32px;" alt="Nx">
    <div>
      <strong>Nx</strong>
      <div style="font-size: 0.875rem; color: var(--vp-c-text-2);">JavaScript/TypeScript monorepos</div>
    </div>
  </a>
  
  <a href="./build-systems/metro" style="display: flex; align-items: center; gap: 0.75rem; padding: 1rem; border: 1px solid var(--vp-c-divider); border-radius: 8px; text-decoration: none;">
    <img src="/images/metro.svg" style="width: 32px; height: 32px;" alt="Metro">
    <div>
      <strong>Metro</strong>
      <div style="font-size: 0.875rem; color: var(--vp-c-text-2);">React Native bundler</div>
    </div>
  </a>
</div>

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
- Read [Architecture](/guide/architecture.md) for how Fabrik works internally
