// Recipe that reads input files and generates output
// FABRIK output "output/"
// FABRIK cache ttl="7d"

console.log("[fabrik] Processing input files...");

// Use Fabrik.glob to find any txt files (will be empty if none exist)
const inputFiles = await Fabrik.glob("*.txt");
console.log("[fabrik] Found", inputFiles.length, "input files");

// Create output directory
await Fabrik.exec("mkdir", ["-p", "output"]);

// Process each input file (or simulate if none found)
if (inputFiles.length === 0) {
  console.log("[fabrik] No input files found, generating sample output");
  await Fabrik.writeFile("output/sample.txt", "Sample output generated\n");
} else {
  for (const file of inputFiles) {
    const data = await Fabrik.readFile(file);
    const lines = data.length; // Count bytes as proxy for content size

    const outputFile = "output/processed-" + file;
    await Fabrik.writeFile(outputFile, "Processed: " + file + "\nBytes: " + lines);
  }
}

// Write summary
const summary = {
  filesProcessed: inputFiles.length || 1,
  timestamp: new Date().toISOString()
};
await Fabrik.writeFile("output/summary.json", JSON.stringify(summary, null, 2));

console.log("[fabrik] Processing complete -> output/");
