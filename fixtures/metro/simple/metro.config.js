const { FabrikStore } = require('@tuist/fabrik/metro');
const path = require('node:path');
const { readdirSync } = require('node:fs');

const repoRoot = path.resolve(__dirname, '../../..');

// Get all packages from pnpm node_modules
const pnpmModules = path.resolve(repoRoot, 'node_modules/.pnpm');
const extraNodeModules = {};

try {
  const pnpmPackages = readdirSync(pnpmModules);
  pnpmPackages.forEach((pkg) => {
    if (pkg.startsWith('metro-runtime@')) {
      const modulePath = path.join(pnpmModules, pkg, 'node_modules');
      const metroRuntimePath = path.join(modulePath, 'metro-runtime');
      extraNodeModules['metro-runtime'] = metroRuntimePath;
    }
  });
} catch (err) {
  console.warn('Could not read pnpm modules:', err.message);
}

module.exports = {
  projectRoot: __dirname,
  watchFolders: [repoRoot],
  resolver: {
    nodeModulesPaths: [
      path.resolve(repoRoot, 'node_modules'),
    ],
    extraNodeModules,
  },
  cacheStores: [
    new FabrikStore({
      cacheDir: path.join(__dirname, '.fabrik', 'cache'),
      maxSize: '1GB',
      logLevel: 'info',
    }),
  ],
};
