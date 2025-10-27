const { greet, sum, fetchData } = require('./utils.js');

/**
 * Main entry point for the Metro bundle
 * This demonstrates Metro bundling and transformation with Fabrik caching
 */

console.log(greet('Metro'));
console.log('Sum of [1, 2, 3, 4, 5]:', sum([1, 2, 3, 4, 5]));

// Async operation to demonstrate transformation
async function main() {
  console.log('Metro bundler with Fabrik caching is working!');

  // Example of modern JS features that Metro will transform
  const numbers = [1, 2, 3, 4, 5];
  const doubled = numbers.map(n => n * 2);
  const [first, ...rest] = doubled;

  console.log('First:', first);
  console.log('Rest:', rest);
}

main().catch(console.error);

module.exports = { greet, sum, fetchData };
