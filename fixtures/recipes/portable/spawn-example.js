// Recipe that uses Fabrik.exec to execute commands
// FABRIK cache ttl="1h"

console.log("[fabrik] Testing Fabrik.exec functionality...");

// Execute a simple echo command using Fabrik.exec
console.log("[fabrik] Running: echo 'Hello from Fabrik.exec'");
const exitCode = await Fabrik.exec("echo", ["Hello from Fabrik.exec"]);

if (exitCode !== 0) {
  throw new Error(`Command failed with exit code ${exitCode}`);
}

console.log("[fabrik] Command executed successfully with exit code:", exitCode);
console.log("[fabrik] Exec test complete!");
