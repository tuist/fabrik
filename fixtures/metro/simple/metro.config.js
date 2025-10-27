const { FabrikStore } = require('@fabrik/metro');
const path = require('node:path');

module.exports = {
  projectRoot: __dirname,

  cacheStores: [
    new FabrikStore({
      cacheDir: path.join(__dirname, '.fabrik', 'cache'),
      maxSize: '1GB',
      logLevel: 'info',
    }),
  ],
};
