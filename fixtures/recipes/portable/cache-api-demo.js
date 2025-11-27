// Demonstration of Fabrik APIs available in portable recipes
// FABRIK cache ttl="1h"

console.log("[fabrik] Demonstrating Fabrik APIs...");

// Example 1: Using Fabrik.glob() to find files
console.log("[fabrik] Finding JavaScript files with Fabrik.glob()...");
const jsFiles = await Fabrik.glob("*.js");
console.log("[fabrik] Found", jsFiles.length, "JavaScript file(s)");

// Example 2: Using Fabrik.exists() to check file existence
console.log("[fabrik] Checking file existence with Fabrik.exists()...");
const selfExists = await Fabrik.exists("cache-api-demo.js");
console.log("[fabrik] cache-api-demo.js exists:", selfExists);

// Example 3: Using Fabrik.hashFile() for content-addressed caching
if (jsFiles.length > 0) {
  console.log("[fabrik] Computing file hash with Fabrik.hashFile()...");
  const hash = await Fabrik.hashFile(jsFiles[0]);
  console.log("[fabrik]", jsFiles[0], "hash:", hash.substring(0, 16) + "...");
}

// Example 4: Using Fabrik.readFile()
console.log("[fabrik] Reading file with Fabrik.readFile()...");
const selfContent = await Fabrik.readFile("cache-api-demo.js");
console.log("[fabrik] Read", selfContent.length, "bytes from cache-api-demo.js");

// Example 5: Using Fabrik.exec() to run commands
console.log("[fabrik] Running command with Fabrik.exec()...");
const exitCode = await Fabrik.exec("echo", ["Hello from Fabrik!"]);
console.log("[fabrik] Command exit code:", exitCode);

// Example 6: Using Fabrik.writeFile() (accepts strings like Node.js)
console.log("[fabrik] Writing file with Fabrik.writeFile()...");
await Fabrik.exec("mkdir", ["-p", "output"]);
const manifest = {
  recipe: "cache-api-demo.js",
  timestamp: new Date().toISOString(),
  apis_demonstrated: [
    "Fabrik.glob()",
    "Fabrik.exists()",
    "Fabrik.hashFile()",
    "Fabrik.readFile()",
    "Fabrik.exec()",
    "Fabrik.writeFile()"
  ]
};
await Fabrik.writeFile("output/manifest.json", JSON.stringify(manifest, null, 2));
console.log("[fabrik] Wrote manifest to output/manifest.json");

console.log("[fabrik] API demonstration complete!");
