const fs = require('fs');
const path = require('path');

// Create dist directory
const distDir = path.join(__dirname, 'dist');
if (!fs.existsSync(distDir)) {
  fs.mkdirSync(distDir, { recursive: true });
}

// Generate some output
const output = {
  message: 'Build complete!',
  timestamp: new Date().toISOString(),
  platform: process.platform,
};

fs.writeFileSync(
  path.join(distDir, 'output.json'),
  JSON.stringify(output, null, 2)
);

console.log('✅ Build complete!');
console.log('   Output:', path.join(distDir, 'output.json'));
