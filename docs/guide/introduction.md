# Introduction

Fabrik is an open-source, vendor and environment-agnostic technology that optimizes developer, CI, and agentic workflows.

## Why Fabrik?

The landscape of software development is transforming rapidly. With the rise of agentic coding and AI-assisted development, the amount of code we're producing is growing exponentially. Development workflows—builds, tests, scripts—are running more frequently across more diverse environments than ever before.

Modern development tools are powerful, but they're often locked into specific vendors, cloud platforms, or execution environments. Teams find themselves constrained by tooling choices, unable to optimize across their entire workflow, or forced to rebuild infrastructure when switching platforms.

That's where Fabrik comes in. **Fabrik is the Kubernetes of development environments**—a universal orchestration layer that sits between your workflows and infrastructure. It acts as the [**narrow waist**](https://en.wikipedia.org/wiki/Hourglass_model) between your tools and your infrastructure—a minimal, universal interface that any build tool, test runner, or script can plug into, and any storage backend or execution environment can support.

## What is Fabrik?

Fabrik is a technology to build transparent, high-performance workflow optimization for builds, tests, and scripts across any environment. Whether you're running locally, in GitHub Actions, GitLab CI, or a custom cloud setup, Fabrik provides intelligent caching and execution without vendor lock-in. It's built with Rust for maximum speed and reliability.

## Key Features

- 🌐 **Vendor Agnostic**: Works with any build system, test runner, or CI platform
- 🏢 **Environment Agnostic**: Seamless operation across local dev, CI/CD, and cloud environments
- 🤖 **AI-Ready**: Optimized for agentic coding workflows and automated development
- 🔥 **Transparent Optimization**: Intelligent caching with automatic fallback across storage tiers
- 🔧 **Universal Compatibility**: Supports Gradle, Bazel, Nx, TurboRepo, Xcode, and custom scripts
- ⚡ **High Performance**: Built in Rust with RocksDB for ultra-low latency (<10ms p99)
- 🎯 **Zero Configuration**: Automatically adapts to your environment
- 💎 **Open Source**: MPL-2.0 licensed—deploy anywhere, customize freely

## Use Cases

Fabrik provides transparent workflow optimization across different environments, supporting:

- 🏗️ **Build Systems**: Gradle, Bazel, Nx, TurboRepo
- 📦 **JavaScript Bundlers**: Metro (React Native)
- 🍎 **Apple Development**: Xcode (Unix socket support)
- 📜 **Custom Scripts**: Bash, Python, Node.js—any script with `fabrik run`
- 🤖 **Agentic Workflows**: AI-driven builds, tests, and deployments

