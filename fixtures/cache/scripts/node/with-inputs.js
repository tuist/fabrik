#!/usr/bin/env -S fabrik run node
//FABRIK input "package.json"
//FABRIK output "analysis.json"
//FABRIK cache ttl="1h"

const fs = require('fs');

console.log('Analyzing package.json...');

const packageData = JSON.parse(fs.readFileSync('package.json', 'utf8'));

const analysis = {
  name: packageData.name || 'unknown',
  hasScripts: !!packageData.scripts,
  scriptCount: packageData.scripts ? Object.keys(packageData.scripts).length : 0,
  hasDependencies: !!packageData.dependencies,
  dependencyCount: packageData.dependencies ? Object.keys(packageData.dependencies).length : 0,
  analyzedAt: new Date().toISOString()
};

fs.writeFileSync('analysis.json', JSON.stringify(analysis, null, 2));
console.log('Analysis complete!');
