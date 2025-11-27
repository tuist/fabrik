// Recipe that generates output files for cache testing
// FABRIK output "generated/"
// FABRIK cache ttl="30d"

import { mkdirSync, writeFileSync } from 'fs';

console.log("[fabrik] Generating output files...");

// Create output directory
mkdirSync("generated", { recursive: true });

// Generate some test files
writeFileSync("generated/file1.txt", "Content from portable recipe - file 1\n");
writeFileSync("generated/file2.txt", "Content from portable recipe - file 2\n");
writeFileSync("generated/data.json", JSON.stringify({
  generated: true,
  timestamp: new Date().toISOString(),
  recipe: "file-generator.js"
}, null, 2));

console.log("[fabrik] Generated 3 files in generated/");
