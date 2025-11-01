# Architecture & Design Decisions

This document explains the key architectural decisions behind Fabrik, with a focus on why the local layer (hot cache) is essential to the design.

## Three-Tier Caching Strategy

Fabrik implements a transparent three-tier caching hierarchy:

1. **Hot Cache (Local Layer)** - Build-local, ultra-fast, lowest latency
2. **Warm Cache (Regional Layer)** - Shared team cache, dedicated instances
3. **Cold Cache (Permanent Layer)** - S3-backed permanent storage

Each tier serves a specific purpose, with cache misses automatically falling back to the next tier and writes propagating through all configured layers.

## Design Principles

The local layer embodies Fabrik's core design principles:

1. **Transparency:** Build systems shouldn't know or care about cache complexity
2. **Intelligence at the edge:** Smart routing and decisions happen closest to the build
3. **Defense in depth:** Multiple layers provide redundancy and resilience
4. **Zero-trust security:** Authenticate and authorize at the Fabrik layer, not build system
5. **Observability by default:** Every operation is logged and attributed
6. **Future-proof:** Architecture supports P2P, edge computing, and new protocols

## Why a Local Layer? The Case for Hot Cache

While it might seem simpler to have build systems communicate directly with a remote cache server, the local layer provides critical capabilities that make Fabrik more powerful, secure, and flexible.

### 1. Intelligent Routing Without Configuration Complexity

In modern CI environments, there's often a wealth of caching infrastructure available that you'd like to leverage simultaneously. Providers like [Namespace](https://namespace.so/) and [BuildBuddy](https://www.buildbuddy.io/) offer mounted volumes for ultra-fast local storage, [GitHub Actions](https://github.com/features/actions) provides its own cache API, and teams typically maintain remote shared cache servers or S3 buckets for permanent artifact storage. The challenge is that build systems aren't designed to juggle multiple cache backends intelligently.

Without a local layer, you face a configuration nightmare. Each build system would need to be configured with multiple cache endpoints, and you'd have to implement runtime detection logic to figure out which backends are available in each environment. Worse, the configuration would differ dramatically between CI and local development environments, making it difficult to maintain consistency and creating a higher barrier to entry for new team members.

The hot cache fundamentally solves this problem by acting as a smart routing layer. From the build system's perspective, there's always just one endpoint: localhost. Fabrik handles all the complexity behind the scenes. It automatically detects available cache backends like GitHub Actions cache or mounted volumes, intelligently routes cache requests to the optimal destination based on latency and availability, and manages sophisticated fallback chains without the build system ever needing to know these details. A developer's local machine, a GitHub Actions runner, and a dedicated CI server can all use the exact same Fabrik configuration, with the local layer adapting to whatever infrastructure is available in that specific environment.

### 2. Attribution & Observability

Build systems are fundamentally designed for task execution, not observability. They excel at determining what needs to be built and orchestrating the build process, but they provide little to no insight into cache usage patterns. When a build system talks directly to S3 or another storage backend, you lose critical visibility into who is accessing the cache, how it's being used, and whether it's actually delivering value to your team.

This lack of attribution creates several problems. From a security perspective, you can't identify who pushed or pulled which artifacts, making it impossible to audit access or detect suspicious behavior. From an operational perspective, you can't understand cache hit rates per developer or team, making it difficult to optimize cache configuration or identify developers who might need help with their setup. And when things go wrong, you can't trace cache misses back to specific builds or developers to understand the root cause.

The local layer solves this comprehensively by ensuring every cache operation flows through Fabrik with full context. Each request includes authentication information, build system details, and artifact metadata. This enables rich authentication tracking where you know exactly who accessed which artifacts and when. It provides security auditing capabilities to detect unauthorized cache access attempts or unusual patterns. It delivers usage analytics that help you understand cache hit rates per developer, team, or project. And it makes debugging dramatically easier by allowing you to trace cache misses back to specific builds, developers, or configuration issues. This level of observability is fundamentally impossible when build systems communicate directly with object storage, because the storage layer has no concept of users, teams, or build context.

### 3. Enhanced Security Model

When build systems communicate directly with cloud storage like S3, the security implications are significant and often underestimated. You're forced to distribute AWS credentials to every developer on the team, which means managing and rotating these credentials across potentially hundreds of machines. These credentials typically require broad S3 permissions to function properly, creating a larger attack surface than necessary. There's no practical way to enforce fine-grained access control, such as restricting certain developers to specific projects or preventing contractors from accessing sensitive artifacts. And perhaps most concerning, these credentials often end up exposed in CI logs, configuration files, or environment variable dumps, creating potential security vulnerabilities.

The local layer transforms this security model entirely by implementing a zero-trust architecture where authentication and authorization happen at the Fabrik layer rather than at the build system level. Instead of distributing permanent AWS credentials, developers receive short-lived JWT tokens that expire automatically after one to twenty-four hours. If a token is compromised, the damage window is minimal. These tokens are validated locally by Fabrik without any network calls, adding only microseconds of overhead rather than the milliseconds required for remote validation.

More importantly, the local layer enables fine-grained access control that would be impossible with direct S3 access. You can enforce per-project and per-team permissions, ensuring that developers only access artifacts they're authorized to use. S3 credentials never leave Fabrik's servers and are never distributed to developers, creating a clean separation of concerns where developers authenticate as themselves using JWTs, and Fabrik handles the underlying storage access using role-based credentials. Every cache operation is logged with full user identity, creating a comprehensive audit trail that satisfies compliance requirements and helps investigate security incidents.

### 4. Peer-to-Peer Office Networks

Consider a typical office scenario: a team of developers working on the same codebase, often on the same features or branches. When one developer builds a project, they generate artifacts that their teammates will likely need minutes or hours later. Yet in traditional remote caching setups, the second developer fetches these artifacts from a remote server potentially thousands of miles away, even though an identical copy sits on their colleague's machine just feet away on the same local network.

This inefficiency becomes particularly painful with large artifacts like Docker images, compiled frameworks, or bundled applications. A 500MB artifact takes seconds to fetch from a nearby machine on a gigabit local network, but could take minutes from a remote server, especially with limited internet bandwidth. And for teams in remote offices or regions with poor internet connectivity, this problem is magnified dramatically.

The local layer makes peer-to-peer caching possible by running as a local agent that can participate in network discovery and artifact sharing. When Fabrik is configured for peer-to-peer mode, instances advertise their available artifacts on the local network using protocols like mDNS or Bonjour. When a developer needs an artifact, Fabrik first checks if any local peers have it available before reaching out to remote servers. The decision logic can be sophisticated, preferring local network sources for large artifacts where latency matters most, while still falling back to remote caches when local peers aren't available.

The benefits are substantial: sub-millisecond latency for large artifacts dramatically improves build times, especially for teams doing frequent rebuilds. Bandwidth costs plummet because you're not paying cloud egress fees for artifacts that stay within the office network. Remote offices with slower internet connections see particularly dramatic improvements. And in some scenarios, teams can continue working even during internet outages if the artifacts they need are available from local peers. This peer-to-peer capability is only possible with a local agent architecture; build systems talking directly to remote storage simply can't discover and utilize local caching opportunities.

### 5. Protocol Translation & Abstraction

The build system ecosystem is wonderfully diverse but frustratingly fragmented when it comes to caching protocols. [Gradle](https://gradle.org/) uses an HTTP REST API with specific headers and path structures. [Bazel](https://bazel.build/) implements a sophisticated gRPC-based Remote Execution API. [Nx](https://nx.dev/) and [TurboRepo](https://turbo.build/) use HTTP but with their own unique conventions. [sccache](https://github.com/mozilla/sccache) speaks the S3 API. [BuildKit](https://github.com/moby/buildkit) uses the OCI Registry protocol. Each protocol has its own authentication mechanisms, artifact formats, and operational semantics.

Without a local layer, a remote cache server must implement every single one of these protocols natively. This creates an enormous maintenance burden as new build systems emerge and existing ones evolve their protocols. It also means the cache server needs intimate knowledge of each build system's quirks and conventions, making it difficult to maintain and extend. The coupling between build systems and storage becomes tight and brittle.

The local layer breaks this coupling by providing protocol adapters that translate build system protocols into a unified Fabrik Protocol. From the build system's perspective, it's talking to a native cache implementation that speaks its language fluently. But behind the scenes, Fabrik normalizes everything into a content-addressed storage model with a single consistent protocol for upstream communication. This architectural decision has profound implications: remote servers only need to implement one protocol rather than dozens, making them dramatically simpler to build and maintain. Adding support for a new build system means writing a single adapter at the local layer that works with all existing backends. And the remote cache becomes truly multi-tenant by design, since it has no build system-specific knowledge and treats all artifacts as content-addressed blobs.

### 6. Unified Cache Across Build Systems

Most build systems implement their own local caching mechanisms - Gradle has its local build cache, Bazel maintains a disk cache, and Nx stores artifacts locally. This seems sufficient until you work in a polyglot codebase where different parts of the system use different build tools. Each build system maintains its own separate cache directory with its own eviction policies and storage formats. A frontend team using Nx and a backend team using Gradle can't share cached artifacts even when working on the same machine, leading to duplicate storage and wasted disk space.

The local layer provides a unified caching layer across all build systems. When Gradle requests an artifact that Bazel already downloaded, Fabrik can serve it from the shared cache. This cross-build-system deduplication is particularly valuable in monorepos where multiple build tools coexist. The local layer also enables consistent eviction policies across all cached artifacts, preventing any single build system from monopolizing disk space. And because Fabrik manages uploads to remote storage centrally, you avoid the problem of different build systems each trying to upload the same artifacts independently, competing for bandwidth and potentially creating upload storms during CI runs.

Additionally, Fabrik's local layer provides graceful degradation when remote caches are unavailable. While build systems have local caches, they don't typically queue uploads for later when remote caches are down. Fabrik can detect network failures and queue artifact uploads for retry, ensuring that valuable cache entries eventually make it to shared storage even if they were built during a network outage. This is especially important in CI environments where builds might succeed locally but fail to populate the remote cache for the rest of the team.

### 7. Intelligent Upload Coordination

In CI environments, bandwidth optimization becomes critical when multiple build systems need to upload artifacts to remote storage simultaneously. Without a local layer, each build system independently uploads its artifacts to the remote cache, often resulting in redundant uploads when different build systems produce identical or overlapping artifacts. During large CI runs with parallel jobs, this can create upload storms that saturate network bandwidth and slow down the entire pipeline.

The local layer acts as a coordination point for all uploads from a single machine or CI runner. When multiple build systems produce identical artifacts (identified by content hash), Fabrik uploads only once rather than multiple times. This deduplication happens transparently - each build system thinks it successfully uploaded its artifact, but Fabrik recognizes duplicates and avoids the redundant network transfer. The local layer can also implement batching and prioritization strategies, uploading smaller, frequently-accessed artifacts first while deferring large, rarely-used artifacts to avoid blocking critical cache operations.

For CI environments with mounted volumes provided by services like Namespace or BuildBuddy, the local layer enables a two-tier strategy: write immediately to the fast mounted volume for within-job caching, while queuing uploads to permanent remote storage in the background. This ensures builds complete quickly without waiting for slow uploads to remote storage, while still ensuring artifacts eventually reach the shared team cache. Build systems talking directly to remote storage must choose between speed (skip remote cache) or sharing (wait for upload), but can't optimize for both simultaneously.

### 8. Multi-Backend Resilience

Build infrastructure failures always happen at the worst possible time. A cache server goes down during a critical production deployment. A network partition isolates your regional cache. An S3 bucket becomes temporarily unavailable due to cloud provider issues. In a remote-only architecture, these failures cascade directly to developers and CI pipelines, potentially blocking all work until the infrastructure is restored.

The local layer provides resilience through automatic failover across multiple backends. Fabrik can be configured with a prioritized list of upstream cache sources, each with its own timeout and availability characteristics. When a build requests an artifact, Fabrik tries each backend in order until it finds a hit or exhausts all options. If the primary cache server is down or slow to respond, Fabrik seamlessly falls back to a backup server, then to S3 permanent storage, all without the build system ever knowing there was a problem. This creates defense in depth where no single point of failure can bring caching to a halt.

The resilience benefits extend beyond simple availability. Different backends can be optimized for different use cases. A primary cache might be geographically close for low latency but have limited capacity. A backup cache might be in a different region for redundancy. S3 permanent storage provides unlimited capacity and durability as an ultimate fallback. The local layer can make intelligent routing decisions based on artifact characteristics, choosing fast local storage for small, frequently accessed artifacts and falling back to cheaper, higher-latency storage for large, rarely used items. Build systems remain blissfully unaware of this complexity, seeing only a simple, reliable cache interface that works consistently regardless of underlying infrastructure state.

## Comparison: With vs Without Local Layer

| Capability | Without Local Layer | With Local Layer (Fabrik) |
|------------|-------------------|---------------------------|
| **Configuration** | Complex (multiple endpoints per build system) | Simple (single localhost endpoint) |
| **Attribution** | None | Full user/team tracking |
| **Security** | Credentials distributed to developers | JWT tokens, credentials isolated |
| **Offline Support** | None | Works with local cache |
| **P2P Networks** | Impossible | Supported (future) |
| **Protocol Support** | Server implements all protocols | Server implements one protocol |
| **Failover** | Manual reconfiguration | Automatic fallback |
| **Bandwidth Optimization** | Limited | Smart caching, compression, dedup |
| **Observability** | Basic (S3 logs only) | Rich (per-request logging) |

## Conclusion

The local layer isn't just a performance optimization - it's a **fundamental architectural decision** that enables:

- 🎯 **Simplicity:** Build systems just talk to localhost
- 🔒 **Security:** Zero-trust authentication and authorization
- 📊 **Observability:** Full attribution and audit trails
- 🌐 **Flexibility:** Support for P2P, multi-backend, offline
- 🚀 **Performance:** Smart routing and caching decisions at the edge

By acting as the **narrow waist** between build systems and storage backends, the local layer makes Fabrik adaptable to any environment while keeping build system configuration simple and portable.
