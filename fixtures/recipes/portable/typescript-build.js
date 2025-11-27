// Simulated TypeScript build recipe
// FABRIK output "dist/"
// FABRIK cache ttl="1h"

console.log("[fabrik] Building simulated TypeScript project...");

// Simulated TypeScript files
const simulatedFiles = ["src/index.ts", "src/utils.ts", "src/types.ts"];
console.log("[fabrik] Found", simulatedFiles.length, "TypeScript files");

// Create dist directory
await Fabrik.exec("mkdir", ["-p", "dist"]);

// Simulate compilation by creating .js files
for (const tsFile of simulatedFiles) {
  const jsFile = "dist/" + tsFile.replace("src/", "").replace(".ts", ".js");
  const content = "// Compiled from " + tsFile + "\nexport default {};\n";
  await Fabrik.writeFile(jsFile, content);
}

// Generate manifest
const manifest = {
  files: simulatedFiles.length,
  environment: "development",
  timestamp: new Date().toISOString()
};
await Fabrik.writeFile("dist/manifest.json", JSON.stringify(manifest, null, 2));

console.log("[fabrik] Compiled", simulatedFiles.length, "files -> dist/");
