// Recipe that reads input files and generates output
// FABRIK input "src/**/*.txt"
// FABRIK output "output/"
// FABRIK cache ttl="7d"

import { mkdirSync, writeFileSync, readdirSync, readFileSync } from 'fs';
import { glob } from 'fabrik:fs';

console.log("Processing input files...");

const inputFiles = await glob("src/**/*.txt");
console.log(`Found ${inputFiles.length} input files`);

// Create output directory
mkdirSync("output", { recursive: true });

// Process each input file
let totalLines = 0;
for (const file of inputFiles) {
  const content = readFileSync(file, 'utf-8');
  const lines = content.split('\n').length;
  totalLines += lines;

  const outputFile = file.replace('src/', 'output/processed-');
  writeFileSync(outputFile, `Processed: ${content}\nLines: ${lines}`);
}

// Write summary
writeFileSync("output/summary.txt", `Total files: ${inputFiles.length}\nTotal lines: ${totalLines}`);

console.log(`Processed ${inputFiles.length} files → output/`);
