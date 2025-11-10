#!/usr/bin/env -S fabrik run node
//FABRIK input "src/**/*.js"
//FABRIK output "dist/"
//FABRIK cache ttl="7d"

const fs = require('fs');
const path = require('path');

console.log('Building JavaScript project...');

// Create dist directory
if (!fs.existsSync('dist')) {
  fs.mkdirSync('dist', { recursive: true });
}

// Write build manifest
const manifest = {
  buildTime: new Date().toISOString(),
  nodeVersion: process.version,
  platform: process.platform,
  files: []
};

// Process source files (if they exist)
if (fs.existsSync('src')) {
  const files = fs.readdirSync('src').filter(f => f.endsWith('.js'));

  files.forEach(file => {
    const srcPath = path.join('src', file);
    const distPath = path.join('dist', file);

    // Simple "minification" - just copy and add a header
    const content = fs.readFileSync(srcPath, 'utf8');
    const minified = `/* Built at ${new Date().toISOString()} */\n${content}`;

    fs.writeFileSync(distPath, minified);
    manifest.files.push(file);
    console.log(`Processed: ${file}`);
  });
}

fs.writeFileSync('dist/manifest.json', JSON.stringify(manifest, null, 2));
console.log('Build complete!');
