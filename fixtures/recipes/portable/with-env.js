// Recipe that demonstrates environment variable handling
// FABRIK output "build/"
// FABRIK cache ttl="1d"

import { mkdirSync, writeFileSync } from 'fs';

// Note: QuickJS may not have process.env, so provide defaults
const buildEnv = "development";
const target = "default";

console.log(`[fabrik] Building for environment: ${buildEnv}, target: ${target}`);

// Create output directory
mkdirSync("build", { recursive: true });

// Generate environment-specific build
const buildConfig = {
  environment: buildEnv,
  target: target,
  timestamp: new Date().toISOString(),
  features: ["debug", "sourcemaps"]
};

writeFileSync("build/config.json", JSON.stringify(buildConfig, null, 2));
writeFileSync("build/env.txt", `ENV=${buildEnv}\nTARGET=${target}`);

console.log(`[fabrik] Build complete for ${buildEnv}/${target} -> build/`);
