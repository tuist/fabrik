// Recipe demonstrating dependency chain concept
// Note: Dependency resolution is not yet implemented in the runtime
// FABRIK output "final/"

console.log("[fabrik] Running dependency chain recipe...");

// Create final output directory
await Fabrik.exec("mkdir", ["-p", "final"]);

const content =
  "Dependency chain recipe completed.\n" +
  "This demonstrates the concept of recipes depending on other recipes.\n" +
  "\n" +
  "Timestamp: " + new Date().toISOString();

await Fabrik.writeFile("final/chain-result.txt", content);

console.log("[fabrik] Dependency chain complete -> final/");
