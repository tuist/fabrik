// Recipe demonstrating dependency chain concept
// Note: Dependency resolution is not yet implemented in the runtime
// FABRIK output "final/"

import { mkdirSync, writeFileSync } from 'fs';

console.log("[fabrik] Running dependency chain recipe...");

// Create final output
mkdirSync("final", { recursive: true });

writeFileSync("final/chain-result.txt",
  "Dependency chain recipe completed.\n" +
  "This demonstrates the concept of recipes depending on other recipes.\n" +
  "\n" +
  "Timestamp: " + new Date().toISOString()
);

console.log("[fabrik] Dependency chain complete -> final/");
