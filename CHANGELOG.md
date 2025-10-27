# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0] - 2025-10-27

### 🚀 Features

- Implement HTTP cache server for Metro and Gradle ([50d057a](50d057a69d0393b90e298b5056f71c1560a678fd))

## [0.3.0] - 2025-10-27

### 🚀 Features

- Add @fabrik/metro NPM package for Metro bundler integration ([752b03b](752b03b1d342b0967e9d77d19b2cd6d67e16c4e0))
- Add development mode to @fabrik/metro using cargo run ([0adc976](0adc97610d718ac088ad931fa03ac644c2b8e60a))

### 🚜 Refactor

- Convert @fabrik/metro to vanilla JS with dependency injection ([bc8160f](bc8160fc8ebe646b0b559d3d310f3f5090eb05e4))

## [0.2.2] - 2025-10-27

### 📚 Documentation

- Add build system logos to sidebar ([f401674](f401674a67656c23082854d41e915bc729aa60ae))
- Use image files for build system logos with fixed spacing ([46dc1e1](46dc1e1b08b4fb06f65933688d8c39d9cb6ac558))

## [0.2.1] - 2025-10-27

### 📚 Documentation

- Add Docker registry links to README and release notes ([49386bc](49386bc80e9900d144047a295a400382b98840f8))

## [0.2.0] - 2025-10-27

### ⚡ Performance

- Optimize filesystem storage for high concurrency ([6ce2117](6ce2117f68b54794b8c804b409899ba699b3e5df))

### 🎨 Styling

- Apply rustfmt formatting ([f73c501](f73c5011df7e428de538cf7339a7cbe7d491ae7b))
- Apply rustfmt formatting ([57f4e91](57f4e919f119352133bcc9d8af93562d9e3dee42))

### 🐛 Bug Fixes

- Add platform guards for Unix-specific Xcode code ([26e7606](26e7606272dccf73b31cefd4d96ecaea4ee64c67))
- Prevent xcode test compilation on non-macOS platforms ([62c0614](62c0614b1cffb913b131d3b6acec66805fc007ac))
- Ensure gRPC server cleanup on all exit paths ([12b2ae3](12b2ae3ecab24189042833c94278ecf8b8ecfb6b))
- Install protobuf compiler for macOS and Windows builds ([af2bcf3](af2bcf3d9b8fe48e9ed91501e4a1fcaf193ebe2f))
- Resolve protobuf compilation issues in Docker and cross builds ([7baebf2](7baebf2fdce02d7bad083ad47a341ce9765c7c08))
- Use apt-get instead of apk for musl cross builds ([ac64483](ac644830eceae96cb55aca9c5c8c7ffe420ecd38))
- Remove optional keyword from proto3 syntax ([97ee04e](97ee04e7caedcf73389d8fcacd82a7594698f90d))
- Prevent mise from overriding Rust toolchain targets ([947ec68](947ec6818011246624e8102e7688c7db955243da))
- Install protoc directly in CI, remove mise-action ([540c078](540c0781a3b897086b879438765426cd46a845ee))
- Use correct 'osx' prefix for macOS protoc downloads ([8a46962](8a46962ffc865235981355e997af11423df39cec))
- Correct Windows protoc download filename ([ca35aed](ca35aed0daab048202c4e52838b75049f8840acd))
- Use x86_64 protoc for all cross-compilation targets ([f741dd6](f741dd6d914b739df8a21e45824af2673241ec5b))
- Convert protoc PATH to Windows format for PowerShell compatibility ([4711c0c](4711c0c07293b7abcaab77434d00ee9ab121d204))
- Resolve Docker multi-platform build failure with QEMU ([19f6cbb](19f6cbb3b3c75bf32eb584072b1586f6e65bdf42))
- Remove APT cache mounts to resolve QEMU emulation failures ([2d66ec8](2d66ec8ef260871ca86f7ce343d6ec1a76d4f2ab))
- Build only linux/amd64 to avoid QEMU emulation issues ([5021f35](5021f3540ff553f1261801d8547afe472f4f9ad0))
- Exclude Docker build artifacts from release downloads ([fa9b5e6](fa9b5e61891790299f748f949ee8f6dea975a244))

### 📚 Documentation

- Document GitHub Actions cache API limitation ([08ac688](08ac6880652e0eaa61a6316ee35d2e9b056f7ea0))

### 🚀 Features

- Add storage dependencies for Xcode cache server ([7a657eb](7a657eb76f0ef9d90de53d4c76f4e1deec3c52dc))
- Add XCBBuildService proto files and gRPC dependencies ([9bc73fa](9bc73fa3254ff6d921380d0b743e109e38d2a48c))
- Implement Xcode cache server with CAS and KeyValue services ([e535f2e](e535f2e233e236d2d337109f7c80134356d997c3))
- Implement fabrik exec with Unix socket support for Xcode ([6c8ce0b](6c8ce0b650ea497a306c4a7855bf739f36a73aad))
- Implement fabrik xcodebuild command with 100% cache hit rate! ([6fefe2f](6fefe2f9e1e2076e80e4041404254bde0eaa8540))
- Add database migrations for cache metadata schema ([e2af003](e2af0032d238da459c75afab9e3170be2fbb2563))
- Add GitHub Actions cache auto-detection and storage backend ([888e981](888e98101dc1e9e2bff9fbee20439cfa9b0f1a58))
- Add storage backend detection diagnostics ([1df8014](1df8014472c7c087e5601e5228721e6450bdef57))
- Add GitHub Actions cache integration test ([fd0b59d](fd0b59dd6c0235f79f8672104e04ffb228c5f0b7))
- Support ACTIONS_RESULTS_URL for GitHub Actions cache ([1829eaa](1829eaa5295c84d1c850ae3515d8654345ae0975))
- Add Docker containerization support to release process ([798f180](798f1805894f0a672b0604c36ab39e204c98b1fb))
- Use Mise to ensure consistent protoc version across all platforms ([1f7a1b2](1f7a1b2729245b4e206acd20ad59f43e31cecae7))
- Add concurrency control to cancel outdated workflow runs ([d3fb700](d3fb70015d5e767d00bf540445d73ae0ef6bbc98))

### 🚜 Refactor

- Extract common config args into shared struct ([ae5733d](ae5733d9e752df48c00dd13e8d53652b4cbbb80f))
- Use persistent tokio runtime in GitHub Actions storage ([4cfec26](4cfec26b17af71b98e045e46290a9c930319567a))
- Remove GitHub Actions cache storage backend ([748b55a](748b55a955e4cb752ef4b9edc14132d708466ae3))

### 🧪 Testing

- Add Xcode cache server acceptance test ([bd2a85c](bd2a85ced2fd49d3ebf698e49ca6cd5d3d101c12))
- Enable xcode_acceptance test by default ([8b90633](8b906337ac627ea7a99d1e7c164e1a6ce49433e4))
- Add verbose logging support to acceptance test ([503a992](503a9920d9f21ea3b130a932a5b699a1d53102d6))

## [0.1.3] - 2025-10-24

### 🐛 Bug Fixes

- Correct README description and add Tuist hosting note ([276ac06](276ac061e95a0bb11d886ae323a46e49771f6f4a))
- Remove "tenant-agnostic" from README description ([d281773](d28177374233652065b11b6e3d0fb32b3cfda505))

### 📚 Documentation

- Update README - remove tagline and fix Tuist URL ([04eb02d](04eb02deafc3caf7e7a23f2ce2f8f5d6fa21a85e))

## [0.1.2] - 2025-10-24

### 📚 Documentation

- Simplify README and add Mise installation instructions ([03492f0](03492f0c1d5c5c4469fab92e0827aa5b2ae34da4))

## [0.1.1] - 2025-10-24

### 🐛 Bug Fixes

- Remove version header from release notes body ([af8f907](af8f9078e922d57ccf30c253409a48ddc00b1dcd))

## [0.1.0] - 2025-10-24

### 🐛 Bug Fixes

- Use v-prefixed initial tag in cliff.toml ([23e89bd](23e89bd67d2e192cf32b158cecfc5f43498027de))
- Use cross-rs instead of cargo-zigbuild for cross-compilation ([4204918](420491843c396b2dfb42d48a6f72fbe0e34938b7))
- Resolve CI linting issues ([439e29b](439e29ba82f4a69ed5c438855fa1aecdba70e83b))
- Correct git-cliff --strip argument in release workflow ([f33191d](f33191dc13de7c62a7217f5d3d38d6791c9b7393))

### 🚀 Features

- Add CI/CD pipeline with automated releases ([d6f1f59](d6f1f590f1cceb0b02b43b524710ee5f4eafbb81))
- Add observability APIs, CLI/config system, and Bazel fixture ([aed8fce](aed8fceee7ba3a7517dfcd540cae3c460a4845e3))


