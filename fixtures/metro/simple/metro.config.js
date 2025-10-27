const { FabrikStore } = require('@tuist/fabrik/metro');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '../../..');

module.exports = {
  projectRoot: __dirname,

  watchFolders: [repoRoot],

  resolver: {
    nodeModulesPaths: [
      path.resolve(repoRoot, 'node_modules'),
      path.resolve(__dirname, 'node_modules'),
    ],
  },

  cacheStores: [
    new FabrikStore({
      cacheDir: path.join(__dirname, '.fabrik', 'cache'),
      maxSize: '1GB',
      logLevel: 'info',
    }),
  ],
};
