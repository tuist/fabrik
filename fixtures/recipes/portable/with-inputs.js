// Recipe that reads input files and generates output
// This demonstrates using Fabrik.glob() to find input files
// FABRIK output "output/"
// FABRIK cache ttl="7d"

import { mkdirSync, writeFileSync } from 'fs';

console.log("[fabrik] Processing input files...");

// Use Fabrik.glob to find any txt files (will be empty if none exist, which is fine)
const inputFiles = await Fabrik.glob("*.txt");
console.log(`[fabrik] Found ${inputFiles.length} input files`);

// Create output directory
mkdirSync("output", { recursive: true });

// Process each input file (or simulate if none found)
if (inputFiles.length === 0) {
  console.log("[fabrik] No input files found, generating sample output");
  writeFileSync("output/sample.txt", "Sample output generated\n");
} else {
  for (const file of inputFiles) {
    const data = await Fabrik.readFile(file);
    const content = new TextDecoder().decode(data);
    const lines = content.split('\n').length;

    const outputFile = `output/processed-${file}`;
    writeFileSync(outputFile, `Processed: ${content}\nLines: ${lines}`);
  }
}

// Write summary
writeFileSync("output/summary.json", JSON.stringify({
  filesProcessed: inputFiles.length || 1,
  timestamp: new Date().toISOString()
}, null, 2));

console.log(`[fabrik] Processing complete -> output/`);
