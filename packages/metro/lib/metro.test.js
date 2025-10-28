const { test } = require('node:test');
const assert = require('node:assert/strict');
const { promisify } = require('node:util');
const { gzip: gzipCb, gunzip: gunzipCb } = require('node:zlib');

const gzipAsync = promisify(gzipCb);
const gunzipAsync = promisify(gunzipCb);

const { FabrikStore } = require('./metro.js');

test('FabrikStore - creates store with default options', () => {
  const mockDeps = {
    fetch: async () => ({ ok: true }),
    spawn: () => ({ unref: () => {} }),
    existsSync: () => true,
    binaryPath: '/fake/fabrik',
    devMode: true,
  };

  const store = new FabrikStore({ autoStart: false, _inject: mockDeps });

  assert.ok(store, 'Store should be created');
  assert.equal(typeof store.get, 'function', 'Should have get method');
  assert.equal(typeof store.set, 'function', 'Should have set method');
});

test('FabrikStore - throws if binary not found in production', () => {
  const mockDeps = {
    existsSync: () => false,
    binaryPath: '/fake/fabrik',
    devMode: false,
  };

  assert.throws(
    () => new FabrikStore({ _inject: mockDeps }),
    /Fabrik binary not found/,
    'Should throw when binary missing in production'
  );
});

test('FabrikStore - get() returns null on 404', async () => {
  const mockDeps = {
    fetch: async () => ({ ok: false, status: 404 }),
    spawn: () => ({ unref: () => {} }),
    existsSync: () => true,
    devMode: true,
  };

  const store = new FabrikStore({ autoStart: false, _inject: mockDeps });
  const result = await store.get(Buffer.from('test'));

  assert.equal(result, null, 'Should return null on cache miss');
});

test('FabrikStore - get() decodes gzipped buffer with NULL_BYTE', async () => {
  const testData = Buffer.from('cached data');
  const withNull = Buffer.concat([Buffer.from([0x00]), testData]);
  const gzipped = await gzipAsync(withNull, { level: 9 });

  const mockDeps = {
    fetch: async () => ({
      ok: true,
      arrayBuffer: async () => gzipped.buffer.slice(gzipped.byteOffset, gzipped.byteOffset + gzipped.byteLength),
    }),
    spawn: () => ({ unref: () => {} }),
    existsSync: () => true,
    devMode: true,
  };

  const store = new FabrikStore({ autoStart: false, _inject: mockDeps });
  const result = await store.get(Buffer.from('abc123'));

  assert.ok(Buffer.isBuffer(result), 'Should return a Buffer');
  assert.deepEqual(result, testData, 'Should strip NULL_BYTE and return data');
});

test('FabrikStore - get() decodes gzipped JSON', async () => {
  const testObj = { dependencies: ['a', 'b', 'c'], foo: 'bar' };
  const jsonData = Buffer.from(JSON.stringify(testObj));
  const gzipped = await gzipAsync(jsonData, { level: 9 });

  const mockDeps = {
    fetch: async () => ({
      ok: true,
      arrayBuffer: async () => gzipped.buffer.slice(gzipped.byteOffset, gzipped.byteOffset + gzipped.byteLength),
    }),
    spawn: () => ({ unref: () => {} }),
    existsSync: () => true,
    devMode: true,
  };

  const store = new FabrikStore({ autoStart: false, _inject: mockDeps });
  const result = await store.get(Buffer.from('abc123'));

  assert.deepEqual(result, testObj, 'Should parse JSON correctly');
});

test('FabrikStore - set() gzips buffer with NULL_BYTE', async () => {
  let capturedBody;

  const mockDeps = {
    fetch: async (url, opts) => {
      capturedBody = opts.body;
      return { ok: true };
    },
    spawn: () => ({ unref: () => {} }),
    existsSync: () => true,
    devMode: true,
  };

  const store = new FabrikStore({ autoStart: false, _inject: mockDeps });
  const testValue = Buffer.from('testvalue');
  await store.set(Buffer.from('key'), testValue);

  assert.ok(Buffer.isBuffer(capturedBody), 'Should send gzipped buffer');

  const decompressed = await gunzipAsync(capturedBody);
  assert.equal(decompressed[0], 0x00, 'Should have NULL_BYTE prefix');
  assert.deepEqual(decompressed.slice(1), testValue, 'Should contain data');
});

test('FabrikStore - set() gzips JSON objects', async () => {
  let capturedBody;

  const mockDeps = {
    fetch: async (url, opts) => {
      capturedBody = opts.body;
      return { ok: true };
    },
    spawn: () => ({ unref: () => {} }),
    existsSync: () => true,
    devMode: true,
  };

  const store = new FabrikStore({ autoStart: false, _inject: mockDeps });
  const testObj = { foo: 'bar', num: 123 };
  await store.set(Buffer.from('key'), testObj);

  assert.ok(Buffer.isBuffer(capturedBody), 'Should send gzipped buffer');

  const decompressed = await gunzipAsync(capturedBody);
  const parsed = JSON.parse(decompressed.toString());
  assert.deepEqual(parsed, testObj, 'Should contain JSON');
});

test('FabrikStore - gracefully handles get() errors', async () => {
  const mockDeps = {
    fetch: async () => { throw new Error('Network error'); },
    spawn: () => ({ unref: () => {} }),
    existsSync: () => true,
    devMode: true,
  };

  const store = new FabrikStore({ autoStart: false, _inject: mockDeps });
  const result = await store.get(Buffer.from('test'));

  assert.equal(result, null, 'Should return null on error');
});

test('FabrikStore - gracefully handles set() errors', async () => {
  const mockDeps = {
    fetch: async () => { throw new Error('Network error'); },
    spawn: () => ({ unref: () => {} }),
    existsSync: () => true,
    devMode: true,
  };

  const store = new FabrikStore({ autoStart: false, _inject: mockDeps });

  await assert.doesNotReject(
    () => store.set(Buffer.from('test'), Buffer.from('value')),
    'Should not throw on set error'
  );
});
