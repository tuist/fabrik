const { FabrikStore } = require('@tuist/fabrik/metro');
const path = require('node:path');

module.exports = {
  projectRoot: __dirname,

  watchFolders: [
    path.resolve(__dirname, '../../..'),  // Watch the entire monorepo
  ],

  cacheStores: [
    FabrikStore({
      cacheDir: path.join(__dirname, '.fabrik', 'cache'),
      maxSize: '1GB',
      logLevel: 'info',
    }),
  ],
};
