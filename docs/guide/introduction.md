# Introduction

Fabrik is an open-source, multi-layer build cache technology designed for modern build systems.

## Why Fabrik?

The landscape of software development is transforming rapidly. With the rise of agentic coding and AI-assisted development, the amount of code we're producing is growing exponentially. As codebases expand, sharing compile artifacts across environments has become more critical than ever.

Modern build systems—Gradle, Bazel, Nx, TurboRepo—are being designed with caching capabilities built-in. They understand the value of reusing work across builds, teams, and CI pipelines. But having the capability isn't enough. These build systems need **infrastructure** to unlock their full potential.

That's where Fabrik comes in. Fabrik is the technology to build the cache infrastructure that gives build systems their superpowers. It acts as the [**narrow waist**](https://en.wikipedia.org/wiki/Hourglass_model) between build systems and cache infrastructure—a minimal, universal interface that any build system can plug into, and any storage backend can support.

## What is Fabrik?

Fabrik is a technology to build transparent, high-performance caching infrastructure for modern build systems. It can be deployed and customized to optimize build performance across different environments. It's built with Rust for maximum speed and reliability.

## Key Features

- 🔥 **Transparent Caching**: Three-tier caching hierarchy (hot, warm, cold) with automatic fallback
- 🔧 **Universal Support**: Works with Gradle, Bazel, Nx, Metro, Xcode, and more
- ⚡ **High Performance**: Built in Rust with RocksDB for ultra-low latency (<10ms p99)
- 🎯 **Shell Activation**: Mise-inspired workflow - activate once, works everywhere
- 🌍 **Multi-Region**: Deploy dedicated instances in your preferred regions
- 🔒 **Secure**: JWT-based authentication with zero-latency validation
- 💎 **Open Source**: MPL-2.0 licensed for transparency and customization

## Use Cases

Fabrik provides a transparent, high-performance caching hierarchy to optimize build performance across different environments, supporting:

- 🏗️ **Build Systems**: Gradle, Bazel, Nx
- 📦 **JavaScript Bundlers**: Metro (React Native)
- 🍎 **Apple Development**: Xcode (Unix socket support)

## Architecture at a Glance

Fabrik implements a three-tier caching strategy:

1. 🔥 **Hot Cache** - Build-local, ultra-fast, lowest latency
2. 🌡️ **Warm Cache** - Shared team cache, dedicated instances
3. ❄️ **Cold Cache** - S3-backed permanent storage

Cache misses automatically fall back to the next tier, and writes propagate through all configured layers.
