---
title: Introducing Fabrik - Multi-Layer Build Cache Infrastructure
description: Fabrik is a new foundation for build caching, designed to provide transparent, high-performance caching across different environments.
date: 2025-01-15
author: Tuist Team
tags:
  - announcement
  - build-cache
  - infrastructure
---

# Introducing Fabrik

We're excited to introduce **Fabrik**, a multi-layer build cache infrastructure designed to optimize build performance across different environments. Fabrik supports various build systems including Gradle, Bazel, Nx, TurboRepo, and compiler caches like sccache.

## What is Fabrik?

Fabrik provides a transparent, high-performance caching hierarchy that works seamlessly across local development, CI environments, and cloud infrastructure. Think of it as the foundational layer for build caching - similar to how Postgres is to Supabase, Fabrik is to Tuist.

## Key Features

### Three-Layer Caching Strategy

Fabrik implements a smart caching hierarchy:

1. **Layer 1: Local Cache** - Fast, ephemeral caching bound to your build process
2. **Layer 2: Regional Cache** - Dedicated instances for your team, deployed in your preferred region
3. **Layer 3: Cloud Storage** - Permanent S3-backed storage with no eviction

### Build System Support

Fabrik works with your existing build tools:

- **Gradle** - HTTP REST API integration
- **Bazel** - gRPC Remote Execution API
- **Nx & TurboRepo** - HTTP-based remote caching
- **sccache** - S3-compatible API for Rust/Cargo builds

### Zero-Configuration CI

Fabrik automatically detects your CI environment and configures itself:

- GitHub Actions support out of the box
- GitLab CI integration (coming soon)
- Local development fallback to filesystem

## Getting Started

Install Fabrik and start caching your builds:

```bash
# For Bazel projects
fabrik bazel -- build //...

# For Gradle projects
fabrik gradle build

# Run as a daemon for local development
fabrik daemon
```

## What's Next?

We're actively developing Fabrik with a focus on:

- Multi-region distribution and replication
- Advanced eviction policies
- Enhanced observability with distributed tracing
- Support for additional build systems

Stay tuned for more updates, and check out our [documentation](/guide/introduction) to learn more!

## Learn More

- [Read the introduction guide](/guide/introduction)
- [Check out build system integrations](/build-systems/bazel)
- [View the API reference](/reference/api)
- [Contribute on GitHub](https://github.com/tuist/fabrik)
