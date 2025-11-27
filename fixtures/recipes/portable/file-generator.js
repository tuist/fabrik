// Recipe that generates output files for cache testing
// FABRIK output "generated/"
// FABRIK cache ttl="30d"

console.log("[fabrik] Generating output files...");

// Write files using Fabrik API (accepts strings like Node.js)
// Parent directories are created automatically
await Fabrik.writeFile("generated/file1.txt", "Content from portable recipe - file 1\n");
await Fabrik.writeFile("generated/file2.txt", "Content from portable recipe - file 2\n");

const dataJson = JSON.stringify({
  generated: true,
  timestamp: new Date().toISOString(),
  recipe: "file-generator.js"
}, null, 2);
await Fabrik.writeFile("generated/data.json", dataJson);

console.log("[fabrik] Generated 3 files in generated/");
