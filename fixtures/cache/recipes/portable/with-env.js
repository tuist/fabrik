// Recipe that uses environment variables for cache keying
// FABRIK env "BUILD_ENV"
// FABRIK env "TARGET"
// FABRIK output "build/"
// FABRIK cache ttl="1d"

import { mkdirSync, writeFileSync } from 'fs';

const buildEnv = process.env.BUILD_ENV || "development";
const target = process.env.TARGET || "default";

console.log(`Building for environment: ${buildEnv}, target: ${target}`);

// Create output directory
mkdirSync("build", { recursive: true });

// Generate environment-specific build
const buildConfig = {
  environment: buildEnv,
  target: target,
  timestamp: new Date().toISOString(),
  features: buildEnv === "production" ? ["optimized", "minified"] : ["debug", "sourcemaps"]
};

writeFileSync("build/config.json", JSON.stringify(buildConfig, null, 2));
writeFileSync("build/env.txt", `ENV=${buildEnv}\nTARGET=${target}`);

console.log(`Build complete for ${buildEnv}/${target} → build/`);
