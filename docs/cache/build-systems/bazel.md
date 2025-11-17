# Bazel Integration

Bazel integration guide for Fabrik. This assumes you've already [completed the getting started guide](/getting-started).

## How It Works

Fabrik provides **zero-configuration** remote caching for Bazel via the Bazel Remote Caching protocol (gRPC). When the Fabrik daemon starts, it automatically:

1. Generates a `bazelrc` file with the correct cache configuration
2. Exports the `BAZELRC` environment variable pointing to this file
3. Bazel automatically picks up the configuration - no manual setup required

> [!IMPORTANT]
> **Bazel Version Requirement**: Zero-configuration via `BAZELRC` environment variable requires **Bazel 9.0.0+** (currently in release candidate). The `BAZELRC` environment variable support was added in [Bazel commit f5b64aee9af](https://github.com/bazelbuild/bazel/commit/f5b64aee9af) and is available in:
> - Bazel 9.0.0rc1 and later ✅
> - Bazel 9.0.0 (stable - coming soon) ✅
>
> **For Bazel 7.x users**: Use the [alternative methods](#alternative-methods-manual-control) below (shell alias, `fabrik exec`, or manual flags) until you upgrade to Bazel 9.0.0+.

## Quick Start (Zero Configuration)

**Requirements**: Bazel 9.0.0rc1 or later

Just use Bazel normally - Fabrik handles everything automatically:

```bash
cd ~/my-bazel-project
# Fabrik daemon starts automatically via shell activation
bazel build //...
bazel test //...
```

**That's it!** No configuration files to edit, no flags to pass.

> [!TIP]
> Check your Bazel version with `bazel --version`. If you're using Bazel 7.x, see the [alternative methods](#alternative-methods-manual-control) below.

> [!TIP]
> When you `cd` into a project with `fabrik.toml`, the daemon automatically starts and exports `BAZELRC` pointing to an auto-generated configuration file. Bazel reads this environment variable and loads the cache settings.

> [!NOTE]
> The bazelrc file is stored in `~/.local/state/fabrik/daemons/{config-hash}/bazelrc` and is updated automatically when the daemon starts. You never need to touch it.

## How It Works Internally

When Fabrik's shell integration activates:

1. **Daemon starts** (if not already running for this project)
2. **Generates bazelrc** in `~/.local/state/fabrik/daemons/{hash}/bazelrc`
3. **Exports `BAZELRC`** environment variable:
   ```bash
   export BAZELRC=/Users/you/.local/state/fabrik/daemons/a3f5d9c2/bazelrc
   ```
4. **Bazel loads configuration** from this file automatically

## Alternative Methods (Manual Control)

### Using Shell Alias

Create a shell alias to automatically use Fabrik's cache:

```bash
# Add to ~/.bashrc or ~/.zshrc
alias bazel='command bazel --remote_cache="$FABRIK_GRPC_URL"'

# Then just use bazel normally
cd ~/my-bazel-project
bazel build //...
bazel test //...
```

### Using fabrik exec

```bash
cd ~/my-bazel-project
fabrik exec -- bazel build --remote_cache="$FABRIK_GRPC_URL" //...
```

### Passing Flag Manually

```bash
cd ~/my-bazel-project
# Daemon starts automatically, exports FABRIK_GRPC_URL
bazel build --remote_cache="$FABRIK_GRPC_URL" //...
```


