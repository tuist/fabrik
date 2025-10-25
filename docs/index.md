---
layout: home

hero:
  name: "Fabrik"
  text: "Multi-Layer Build Cache Technology"
  tagline: To build infrastructure for modern build systems
  actions:
    - theme: brand
      text: Get Started
      link: /guide/introduction
    - theme: alt
      text: View on GitHub
      link: https://github.com/tuist/fabrik

features:
  - title: 🏗️ Layered Architecture
    details: Three-tier caching design with automatic fallback from hot to warm to cold storage, ensuring minimal latency at every level of your build infrastructure
  - title: 🔧 Multi Build System Support
    details: Seamlessly integrates with all modern build systems that support caching - from Gradle and Bazel to Nx, TurboRepo, and sccache
  - title: 🚀 High Performance
    details: Built in Rust with RocksDB and locally-mounted volumes for maximum speed, delivering sub-10ms cache hits on your build nodes
  - title: 💎 Open Source
    details: MPL-2.0 licensed and fully transparent, allowing you to build, customize, and deploy your own build cache infrastructure
---
