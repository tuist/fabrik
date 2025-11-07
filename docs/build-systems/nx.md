# Nx Integration

Nx integration guide for Fabrik. This assumes you've already [completed the getting started guide](../../README.md#-getting-started).

## How It Works

Fabrik provides remote caching for Nx via HTTP. When you navigate to your project, Fabrik exports `NX_SELF_HOSTED_REMOTE_CACHE_SERVER` which Nx automatically reads.

## Quick Start

```bash
cd ~/my-nx-workspace
nx build my-app
```

That's it! Nx will automatically use Fabrik's cache via the `NX_SELF_HOSTED_REMOTE_CACHE_SERVER` environment variable.
