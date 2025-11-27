// Recipe that demonstrates environment variable handling
// FABRIK output "build/"
// FABRIK cache ttl="1d"

// Note: QuickJS doesn't have process.env, so we use defaults
const buildEnv = "development";
const target = "default";

console.log("[fabrik] Building for environment:", buildEnv + ", target:", target);

// Generate environment-specific build (parent directories created automatically)
const buildConfig = {
  environment: buildEnv,
  target: target,
  timestamp: new Date().toISOString(),
  features: ["debug", "sourcemaps"]
};

await Fabrik.writeFile("build/config.json", JSON.stringify(buildConfig, null, 2));
await Fabrik.writeFile("build/env.txt", "ENV=" + buildEnv + "\nTARGET=" + target);

console.log("[fabrik] Build complete for", buildEnv + "/" + target, "-> build/");
