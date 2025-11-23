// Simulated TypeScript build recipe
// FABRIK input "src/**/*.ts"
// FABRIK input "tsconfig.json"
// FABRIK output "dist/"
// FABRIK env "NODE_ENV"

import { mkdirSync, writeFileSync, existsSync } from 'fs';
import { glob } from 'fabrik:fs';

console.log("Building TypeScript project...");

// Find TypeScript files
const tsFiles = await glob("src/**/*.ts");
console.log(`Found ${tsFiles.length} TypeScript files`);

// Create dist directory
mkdirSync("dist", { recursive: true });

// Simulate compilation by creating .js files
for (const tsFile of tsFiles) {
  const jsFile = tsFile.replace('src/', 'dist/').replace('.ts', '.js');
  const content = `// Compiled from ${tsFile}\nexport default {};`;

  // Ensure parent directory exists
  const dir = jsFile.substring(0, jsFile.lastIndexOf('/'));
  mkdirSync(dir, { recursive: true });

  writeFileSync(jsFile, content);
}

// Generate manifest
const manifest = {
  files: tsFiles.length,
  environment: process.env.NODE_ENV || "development",
  timestamp: new Date().toISOString()
};
writeFileSync("dist/manifest.json", JSON.stringify(manifest, null, 2));

console.log(`Compiled ${tsFiles.length} files → dist/`);
