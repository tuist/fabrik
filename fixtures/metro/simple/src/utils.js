/**
 * A simple utility function to demonstrate Metro bundling
 */
function greet(name) {
  return `Hello, ${name}!`;
}

/**
 * Calculate the sum of an array of numbers
 */
function sum(numbers) {
  return numbers.reduce((acc, num) => acc + num, 0);
}

/**
 * A more complex function to ensure Metro transforms it
 */
async function fetchData(url) {
  const response = await fetch(url);
  return response.json();
}

module.exports = { greet, sum, fetchData };
