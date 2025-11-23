// Recipe with dependencies to test dependency resolution
// FABRIK depends "@fixtures/simple-echo.js"
// FABRIK depends "@fixtures/file-generator.js" use-outputs=true
// FABRIK output "final/"

import { mkdirSync, writeFileSync } from 'fs';

console.log("Running dependency chain recipe...");
console.log("Dependencies should have executed first");

// Create final output
mkdirSync("final", { recursive: true });

writeFileSync("final/chain-result.txt",
  "This recipe depends on:\n" +
  "1. simple-echo.js (no outputs)\n" +
  "2. file-generator.js (outputs used as inputs)\n" +
  "\n" +
  "All dependencies completed successfully!"
);

console.log("Dependency chain complete → final/");
