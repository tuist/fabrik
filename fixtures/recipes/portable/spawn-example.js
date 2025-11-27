// Recipe that uses Fabrik.exec to execute commands
// FABRIK output "output/"
// FABRIK cache ttl="1h"

import { mkdirSync, writeFileSync } from 'fs';

console.log("Testing Fabrik.exec functionality...");

// Create output directory
mkdirSync("output", { recursive: true });

// Execute a simple echo command using Fabrik.exec
console.log("Running: echo 'Hello from Fabrik.exec'");
const exitCode = await Fabrik.exec("echo", ["Hello from Fabrik.exec"]);

if (exitCode !== 0) {
  throw new Error(`Command failed with exit code ${exitCode}`);
}

console.log("Command executed successfully");

// Write results
writeFileSync("output/exec-result.txt",
  `Exit code: ${exitCode}\n` +
  `Command: echo\n` +
  `Timestamp: ${new Date().toISOString()}`
);

console.log("Exec test complete -> output/");
