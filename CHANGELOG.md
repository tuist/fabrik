# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### 🚀 Features

- **P2P Cache Sharing (Layer 0.5)**: Automatic discovery and sharing of build caches across local networks
  - Zero-configuration peer discovery via mDNS/DNS-SD
  - Secure HMAC-SHA256 authentication with shared secrets
  - User consent system with cross-platform system notifications
  - High-performance parallel peer querying (1-5ms latency)
  - Comprehensive metrics tracking (hits, misses, bandwidth, consent)
  - CLI management commands: `fabrik p2p list`, `status`, `approve`, `deny`, `clear`
  - XDG-compliant consent storage

### 📚 Documentation

- Add comprehensive P2P section to CLAUDE.md with architecture, usage, and security considerations
- Add P2P example configuration file (examples/p2p-config.toml)
- Update README.md with P2P cache sharing key feature
- Update PLAN.md with P2P implementation status

## [0.9.2] - 2025-11-15

### 🐛 Bug Fixes

- **(deps)** Update rust crate home to v0.5.12 (#13) ([3f30b8d](3f30b8d6894538cc9ff4e66aeb85c6891cc299f1))
- **(deps)** Update dependency metro to ^0.83.0 (#26) ([5120c06](5120c06fc4e977cc9fda2388d95447ce630a5e22))

### 📦 Dependency Updates

- **(deps)** Update dependency java to v21.0.9+10.0.lts (#12) ([8a353fa](8a353fab4f1f5e74a8ffac2d582af3f2c1608918))
- **(deps)** Update dependency apple_support to v1.24.4 (#16) ([58bec06](58bec06fbc3ee7b77d74cceda9e287e550d19f0b))
- **(deps)** Update dependency rules_swift to v2.9.0 (#17) ([3c70ffc](3c70ffca986040839f9284dd9c232a6632a35f43))
- **(deps)** Update docker/dockerfile docker tag to v1.20 (#19) ([f7cbbd9](f7cbbd9c4aca4e604137beccc550aa831bbf7cf4))
- **(deps)** Update gradle to v8.14.3 (#20) ([8b62350](8b623509a09df497fd5717e6e829631befc77b02))
- **(deps)** Update npm to v11.6.2 (#21) ([bbc6b42](bbc6b428257b495f6b56da3fdb621e14f85cd6ae))
- **(deps)** Update plugin org.jetbrains.kotlin.jvm to v2.2.21 (#22) ([7af9dc1](7af9dc1f26e3caa86e4a203b30b599e369791f46))
- **(deps)** Update pnpm to v10.22.0 (#23) ([a174aab](a174aabd577838a44ec05948896df9de01ee78c5))
- **(deps)** Update rust crate cbindgen to 0.29 (#24) ([d2f33e1](d2f33e114523747b32d00373390b475f27d4a6ab))
- **(deps)** Update rust docker tag to v1.91 (#25) ([6382b11](6382b11bfaf5c9eaf4b4630fc057c53f74bf8039))
- **(deps)** Update dependency apple_support to v1.24.5 (#27) ([a4d5725](a4d572551e5c36a3d0dd60913fe4102dd0d1b588))

## [0.9.1] - 2025-11-12

### 🚜 Refactor

- Separate CAS and KV commands, improve CLI organization (#11) ([43b82de](43b82de1e95296ab6910ecb0c43521891d422aff))

## [0.9.0] - 2025-11-11

### 🐛 Bug Fixes

- Pin home crate to 0.5.11 to avoid edition 2024 requirement ([6a14676](6a14676eca8af59e1eb611bad39097a666ecff33))
- Use absolute path for DIST_DIR in release workflow ([50f24c2](50f24c2d0fdc19851bd660f2ce44d8f4474588b6))
- Use absolute path for cbindgen.toml in Docker build ([79b5287](79b5287541c220a13d35ff0f5e92c3f2108d14fe))
- Remove non-existent npm-publish job from release workflow ([ee3a647](ee3a647ed967f0b244477a7b7ad6cf62d706a62f))
- Copy cbindgen.toml to Docker build context ([bf34178](bf34178a13d26792dee381d7258fff92aa227a20))

### 🚀 Features

- Add cache CRUD interface with CLI and C FFI library ([c5efa41](c5efa41fe08bf8e10991d4899e15360c63d771df))

## [0.8.1] - 2025-11-10

### Refactor

- Extract TurboRepo token/team generation to shared functions ([6472058](64720586fa2d0e1699db32405b32c8d3b301a9a9))
- Make build tool helpers generic for all build systems ([1d06386](1d0638694559f8c0eca5adabeca07a8d852c0606))

## [0.8.0] - 2025-11-07

### 🐛 Bug Fixes

- Resolve all clippy warnings for CI ([3bdce0d](3bdce0d94650f1fcd24862cda6a020ce079567ae))
- Add platform guards for Unix socket support (Windows compatibility) ([0410468](0410468fa9cf4b98a528bb755fa0a06fa967fe4d))
- Escape Windows backslashes in TOML paths for tests ([10c96cf](10c96cfa5310fa1f356c141cd95ca7b6b3f5edf5))
- Make Xcode build test gracefully skip on missing SDK in CI ([89bd6d2](89bd6d275761ecb343b72038515a4f52f7c2bb78))

### 📚 Documentation

- Add comprehensive user-facing guide for activation workflow ([a23f8b3](a23f8b3ef96cf66a62480b868867d1f36ad01b56))
- Update CLI reference for activation-based workflow ([1f9a077](1f9a077f75b74ce60d18d0dc5887af4922b8ea6c))
- Update Bazel, Gradle, and Nx for activation-based workflow ([29beee8](29beee86627a85b6fd4978e0c7d1020fa8d71cc7))
- Update Metro and Xcode for activation-based workflow ([f735840](f735840a2cebde1937f208b35e927274020f4b3f))
- Finalize activation-based documentation updates ([3b4397b](3b4397b7a08747323e151c8dd9addf857d4ac007))
- Remove redundant setup instructions from build system guides ([f973460](f973460be92984e7b7fa1e937003a15a2441ca9e))
- Remove setup instructions from all build system guides ([0b6ee1b](0b6ee1bb14beb2acc61d95fed2948f64577ffbde))
- Add 'Other Build Systems' section to all build system guides ([bc7b7ec](bc7b7ec168edf6263e13a88bc740e849e94d5c5b))
- Completely rewrite getting-started.md per requirements ([80402de](80402de7e4c1b9ab6234f3e6dd72970139ed5133))
- Use real build system icons in getting-started.md ([20e53ee](20e53ee27b3c4be47422f11ef2735dfcc52fe61a))
- Clarify Bazel .bazelrc doesn't support env var expansion ([c704a86](c704a869f5e90f61c62573a7b0046fe5fabd2bae))
- Simplify Bazel page - remove redundant sections ([1054153](1054153d0117aeef71f2a47c8e169a2c76179799))
- Remove 'Other Build Systems' and 'See Also' sections from all build system pages ([1617423](1617423513dba012446dd17f6c95194cebb705aa))
- Simplify Gradle page to match Bazel simplification ([c2d8a3a](c2d8a3af67076dd34b0b0d14eb9e786d70e404c3))
- Update development section to use mise exec for acceptance tests ([2b4587c](2b4587c09675d4753ab1e1fed9b117fa3cbd8cbe))
- Simplify Metro, Nx, and Xcode pages to match minimal pattern ([39295d2](39295d2723b55836e889efc5729e75a3484b21d1))
- Update Xcode integration to clarify Unix socket limitation ([e2cd3c0](e2cd3c0e1560ca6669a8c0258ee061ee82dda694))
- Update Xcode integration with Unix socket configuration ([3fe156f](3fe156fe3dfc0824cd3c2caa0105af172d807f40))

### 🚀 Features

- Add activation-based daemon architecture (mise-inspired) ([b00b3d0](b00b3d02728448dccef13bd36aadb3aaf8374835))
- Add Unix socket support for Xcode integration ([a1bad76](a1bad769be946c0015e85a3d6159db5d043c175b))
- Implement activate and deactivate commands (partial) ([823a321](823a3216db63df94dc80bc490d690d937c92d08c))
- Implement daemon spawning and state management ([82f2856](82f28568d38a261ffa43ef45faf56ae62845e4e6))
- Implement activation-based daemon with dynamic port allocation ([6a41ecd](6a41ecd1fcd3a3eb55c74ecfab8e178e64f3972f))
- Add doctor command and comprehensive documentation ([c4cee46](c4cee460bffd5677f0739c86624d34ae75ed714e))
- Add fabrik init command and rename config to fabrik.toml ([6e8648a](6e8648a68f794aca7e41e02717a1b47eda549489))
- Add test isolation with FABRIK_STATE_DIR and fix daemon port binding ([4e4c657](4e4c657a029e6231068e9ad8ea3e7dbdd6d299b0))
- Implement Unix socket support for Xcode integration ([b8ddcff](b8ddcff176be030de209df042adb8420f5927077))

### 🚜 Refactor

- Remove wrapper commands in favor of activation-based approach ([c6a2f5c](c6a2f5c16340464a33b83f933443cb5e5dd751f7))

### 🧪 Testing

- Temporarily skip acceptance tests during activation migration ([de65349](de6534929923955fa20f0013fdedba98ce7008fd))

## [0.7.0] - 2025-11-01

### 🐛 Bug Fixes

- Make Nx acceptance test fail on missing cache hits ([b6442a0](b6442a0c5456f10392c4040ebde14757aa838926))
- Repair Nx fixture and configure remote cache properly ([98e6069](98e6069c1413b764cc7178c1352129e30ef045b1))
- Use correct Nx environment variable for self-hosted cache ([4079624](4079624674ca879dea541c31d91fabe6c51920f8))
- Nx uses raw string hashes, not hex-encoded ([3dd30a1](3dd30a18f9ea77c5a9d269451e7d2b62ba738a61))
- Make Nx acceptance test work reliably ([1f0ecfa](1f0ecfae9b644a1c7aa936fa8b22c73d5b6ca51a))
- Force Nx to use remote cache and validate communication ([c55fa77](c55fa77f526aaced5aecb8479e84a77dc6e7dc48))
- Update test assertion to match test architecture ([817fed2](817fed22e67b50e71f2d6a2225253f5195b71360))
- Validate cache hits work across server instances ([4dddc30](4dddc309cc2a6657704cf13ddb4306a83d3c2f86))
- Resolve CI test failures for Nx integration ([2d30a7c](2d30a7c3392b8a582918c47816a15b8236271af7))
- Use npm.cmd on Windows for Nx tests ([9216f43](9216f43b4bd5ecad6f8e1988310ae94b04b87b0f))
- Use cross-platform Node.js commands in Nx fixture for Windows compatibility ([8afc0f5](8afc0f529c3e0585ed4f458d71964d56c4e491fb))

### 📚 Documentation

- Add Nx build system documentation ([1f214a2](1f214a2c6af1c93f79507ee28e3c4103a1395859))
- Add Nx build system documentation and logo ([3d1b316](3d1b3167392e59e6a845e3eccd5f9d7adaf086a0))
- Update Nx documentation to reflect implementation ([320350f](320350f803724799d10eafd6d499387828f0003d))
- Add architecture guide explaining local layer motivations ([c17014c](c17014c04a1c33dd06b085500f71b59bff5e3b0c))
- Update PLAN.md with comprehensive build system support roadmap ([612c67a](612c67aac34c20c73e183e3d8d4987178fb1c302))

### 🚀 Features

- Add Nx build system support ([24f6453](24f64536f0be794de8caa6a2cbfd86830c6cd204))
- Implement Nx wrapper command with comprehensive tests ([1deef56](1deef56958a5c1826e51f741ebe3587a2cf3a0a3))

### 🚜 Refactor

- Consolidate HTTP cache servers into shared HttpServer ([14a31c3](14a31c34705326eb60d9749a0b92c07ff9d2a180))
- Rename handlers to reflect build system binding ([a38c080](a38c08051f8234723606a229fc0ad1129b75c252))

## [0.6.1] - 2025-10-28

### 🐛 Bug Fixes

- Update Cargo.toml version during release build ([6118ff2](6118ff2c340de311883d723cf6dc8cdf657c26ee))
- Commit Cargo.toml version changes to upstream ([b8217d4](b8217d4d5041796a78346bfc7a455607c1e2ce21))

## [0.6.0] - 2025-10-28

### 🐛 Bug Fixes

- Add nodeModulesPaths to Metro config and document fixture status ([6e05308](6e053083c7f94ac3780088ee31089c92557811c7))
- Update Metro integration to use CommonJS and document protocol issue ([2a93d9c](2a93d9c4d909819dbf51a36e62f36252ae9b48c4))
- Implement Metro HttpStore protocol (gzip + NULL_BYTE) ([eac1cfe](eac1cfe815f85a9d0053b425a8f3eddfcde5c9df))
- Resolve Metro module resolution with extraNodeModules ([752dba0](752dba037898ca9c17458163c3b3a29c2bddef45))
- Use pnpm-action instead of mise for Metro tests ([175727a](175727a5019e16e486b37b29bd1b52198399e4a5))
- Remove broken tipi installation from mise config ([4cbe8bb](4cbe8bbbe7055710661b36b74127c430522d2bd5))
- Remove pnpm version from action to use packageManager field ([6dc2675](6dc2675bad9d8363f6ade5b558cf1268c2b64d91))
- Run Metro tests from workspace root ([8f2f8a6](8f2f8a660ff1c0882f4f3f7fa25f2531c9f5e9b6))
- Use explicit test file path instead of glob ([1c13c5a](1c13c5aa8a9dfa183ac554d6c453d18a2e0d9e09))
- Configure NPM authentication properly in release workflow ([fd55a8c](fd55a8c68db27e2f9f9609bf72bc7e70393f4206))

### 📚 Documentation

- Add Metro documentation and NPM publishing to releases ([dea36f9](dea36f9618012364012d0e4a3b62f3ba8ea71774))
- Update fixture README to reflect fully working status ([9a1d637](9a1d637644f1ecc3ef168ce9cfc505040404503a))
- Remove troubleshooting and React Native example from Metro docs ([5828873](58288732aec720d0a15acc78bd03714c3a99cfa4))

### 🚀 Features

- Add cache activity logging to FabrikStore ([7afa86d](7afa86defef98ec3fcb18ac193dfda6ad3ab61df))

### 🚜 Refactor

- Rename package to @tuist/fabrik and update references ([883f456](883f4568e82e7116d3e82ae8bab55d36ebd8967f))
- Restructure package as @tuist/fabrik with /metro subpath ([3575c47](3575c473229394e42f7e3c49a13b3cffa142817a))
- Simplify Metro fixture by removing workspace dependency ([fa91582](fa91582317de7e1cd89bfbebcd1c2a60176fab70))
- Simplify Metro integration to minimal working implementation ([40f8cea](40f8cea40a20280d478c98a77ec8a82fa6a91352))
- Remove debug logging and remaining TypeScript artifacts ([767eb6a](767eb6a59b44ebe86431d49bdb8e32796d09fdeb))

### 🧪 Testing

- Add comprehensive Metro package tests with dependency injection ([5f6f843](5f6f8432ed6360ff762aae3782cfa0cc95799245))

## [0.5.1] - 2025-10-27

### 🐛 Bug Fixes

- Use XDG-compliant cache directory for xcodebuild wrapper ([21f10c9](21f10c99b3e55f81c51e731fcb664283f759e79c))

## [0.5.0] - 2025-10-27

### 🚀 Features

- Implement universal daemon with HTTP and gRPC servers ([1be3d12](1be3d122a34054dcbd2b49783b9583ae1b009c3c))

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


