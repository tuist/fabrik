// Recipe that uses spawn to execute commands
// FABRIK output "output/"
// FABRIK cache ttl="1h"

import { spawn } from 'child_process';
import { mkdirSync, writeFileSync } from 'fs';

console.log("Testing spawn functionality...");

// Create output directory
mkdirSync("output", { recursive: true });

// Execute a simple echo command
console.log("Running: echo 'Hello from spawn'");
const result = await spawn("echo", ["Hello from spawn"]);

if (result.exitCode !== 0) {
  throw new Error(`Command failed with exit code ${result.exitCode}`);
}

console.log("Command executed successfully");

// Write results
writeFileSync("output/spawn-result.txt",
  `Exit code: ${result.exitCode}\n` +
  `Command: echo\n` +
  `Timestamp: ${new Date().toISOString()}`
);

console.log("Spawn test complete → output/");
