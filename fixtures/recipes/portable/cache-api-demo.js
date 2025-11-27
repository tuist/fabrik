// Demonstration of Fabrik cache APIs
// NOTE: runCached and needsRun from fabrik:cache are planned but not yet implemented
// This recipe demonstrates the currently available Fabrik global APIs
//
// FABRIK output "build-output/"
// FABRIK cache ttl="1h"

import { mkdirSync, writeFileSync, readFileSync, existsSync } from 'fs';

console.log("[fabrik] Demonstrating Fabrik cache-related APIs...");

// Create output directory
mkdirSync("build-output", { recursive: true });

// Example 1: Using Fabrik.glob() to find files
console.log("[fabrik] Finding source files with Fabrik.glob()...");
const sourceFiles = await Fabrik.glob("*.js");
console.log(`[fabrik] Found ${sourceFiles.length} JavaScript files`);

// Example 2: Using Fabrik.hashFile() for content-addressed caching
console.log("[fabrik] Computing file hashes with Fabrik.hashFile()...");
const fileHashes = {};
for (const file of sourceFiles.slice(0, 3)) { // Hash first 3 files
  try {
    const hash = await Fabrik.hashFile(file);
    fileHashes[file] = hash.substring(0, 16); // First 16 chars for readability
    console.log(`[fabrik] ${file}: ${fileHashes[file]}...`);
  } catch (e) {
    console.log(`[fabrik] Could not hash ${file}: ${e.message}`);
  }
}

// Example 3: Using Fabrik.exists() to check file existence
console.log("[fabrik] Checking file existence with Fabrik.exists()...");
const configExists = await Fabrik.exists("config.json");
console.log(`[fabrik] config.json exists: ${configExists}`);

// Example 4: Using Fabrik.readFile() and Fabrik.writeFile()
console.log("[fabrik] Testing Fabrik.readFile() and Fabrik.writeFile()...");

// Write a test file using Fabrik API
const testData = new TextEncoder().encode("Hello from Fabrik!\n");
await Fabrik.writeFile("build-output/fabrik-test.txt", testData);

// Read it back
const readData = await Fabrik.readFile("build-output/fabrik-test.txt");
const decoded = new TextDecoder().decode(readData);
console.log(`[fabrik] Read back: ${decoded.trim()}`);

// Example 5: Using Fabrik.exec() to run commands
console.log("[fabrik] Running command with Fabrik.exec()...");
const exitCode = await Fabrik.exec("echo", ["Build completed successfully"]);
console.log(`[fabrik] Command exit code: ${exitCode}`);

// Write build manifest
const manifest = {
  recipe: "cache-api-demo.js",
  timestamp: new Date().toISOString(),
  sourceFiles: sourceFiles.length,
  hashedFiles: Object.keys(fileHashes).length,
  fileHashes: fileHashes,
  apis_demonstrated: [
    "Fabrik.glob()",
    "Fabrik.hashFile()",
    "Fabrik.exists()",
    "Fabrik.readFile()",
    "Fabrik.writeFile()",
    "Fabrik.exec()"
  ]
};

writeFileSync("build-output/manifest.json", JSON.stringify(manifest, null, 2));
console.log("[fabrik] Build manifest written to build-output/manifest.json");

console.log("[fabrik] Cache API demonstration complete!");
