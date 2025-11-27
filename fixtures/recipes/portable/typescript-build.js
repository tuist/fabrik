// Simulated TypeScript build recipe
// This demonstrates a portable recipe that generates output files
// FABRIK output "dist/"
// FABRIK cache ttl="1h"

import { mkdirSync, writeFileSync } from 'fs';

console.log("[fabrik] Building simulated TypeScript project...");

// Simulate finding TypeScript files
const simulatedFiles = [
  "src/index.ts",
  "src/utils.ts",
  "src/types.ts"
];
console.log(`[fabrik] Found ${simulatedFiles.length} TypeScript files`);

// Create dist directory
mkdirSync("dist", { recursive: true });

// Simulate compilation by creating .js files
for (const tsFile of simulatedFiles) {
  const jsFile = tsFile.replace('src/', 'dist/').replace('.ts', '.js');
  const content = `// Compiled from ${tsFile}\nexport default {};\n`;

  writeFileSync(jsFile, content);
}

// Generate manifest
const manifest = {
  files: simulatedFiles.length,
  environment: "development",
  timestamp: new Date().toISOString()
};
writeFileSync("dist/manifest.json", JSON.stringify(manifest, null, 2));

console.log(`[fabrik] Compiled ${simulatedFiles.length} files -> dist/`);
