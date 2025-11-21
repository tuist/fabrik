// Example recipe demonstrating runCached() and needsRun() APIs
//
// This recipe shows how to use Fabrik's cache APIs for content-addressed caching

import { spawn } from 'child_process';
import { runCached, needsRun } from 'fabrik:cache';
import { glob } from 'fabrik:fs';

console.log("[fabrik] Building Rust project with caching...");

// Example 1: Use runCached() for automatic caching
await runCached(
  async () => {
    console.log("[fabrik] Running cargo build...");
    const result = await spawn("cargo", ["build", "--release"]);
    if (result.exitCode !== 0) {
      throw new Error("Build failed!");
    }
    console.log("[fabrik] Build completed!");
  },
  {
    inputs: ["src/**/*.rs", "Cargo.toml", "Cargo.lock"],
    outputs: ["target/release/"],
    env: ["RUSTFLAGS"],
    hashMethod: "content"
  }
);

// Example 2: Use needsRun() for conditional logic
const needsTest = await needsRun({
  inputs: ["src/**/*.rs", "tests/**/*.rs"],
  env: ["RUST_TEST_THREADS"]
});

if (needsTest) {
  console.log("[fabrik] Running tests (inputs changed)...");
  const testResult = await spawn("cargo", ["test"]);
  if (testResult.exitCode !== 0) {
    throw new Error("Tests failed!");
  }
} else {
  console.log("[fabrik] Skipping tests (no changes detected)");
}

// Example 3: Using cache directory override
await runCached(
  async () => {
    console.log("[fabrik] Running documentation build...");
    await spawn("cargo", ["doc", "--no-deps"]);
  },
  {
    inputs: ["src/**/*.rs"],
    outputs: ["target/doc/"],
    cacheDir: ".fabrik/doc-cache"  // Custom cache directory
  }
);

console.log("[fabrik] All tasks completed!");
