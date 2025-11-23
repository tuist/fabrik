// Recipe that demonstrates error handling
// FABRIK cache disable

console.log("Testing error handling...");

const shouldFail = process.env.SHOULD_FAIL === "true";

if (shouldFail) {
  console.error("Intentional error for testing");
  throw new Error("Recipe failed intentionally (SHOULD_FAIL=true)");
}

console.log("Recipe completed successfully (SHOULD_FAIL not set)");
