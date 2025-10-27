import { test, mock } from 'node:test';
import assert from 'node:assert/strict';
import { createFabrikStore } from '../lib/index.js';

test('FabrikStore - creates store with default options', () => {
  const mockDeps = {
    existsSync: () => true, // Binary exists
    fetch: mock.fn(),
    spawn: mock.fn(() => ({
      unref: () => {},
      kill: () => {},
    })),
    binaryPath: '/fake/path/fabrik',
  };

  const store = createFabrikStore({}, mockDeps);

  assert.ok(store, 'Store should be created');
  assert.equal(typeof store.get, 'function', 'Should have get method');
  assert.equal(typeof store.set, 'function', 'Should have set method');
  assert.equal(typeof store.clear, 'function', 'Should have clear method');
  assert.equal(typeof store.stop, 'function', 'Should have stop method');
});

test('FabrikStore - throws if binary not found', () => {
  const mockDeps = {
    existsSync: () => false, // Binary doesn't exist
    fetch: mock.fn(),
    spawn: mock.fn(),
    binaryPath: '/fake/path/fabrik',
  };

  assert.throws(
    () => createFabrikStore({}, mockDeps),
    /Fabrik binary not found/,
    'Should throw error when binary not found'
  );
});

test('FabrikStore - get() returns null on 404', async () => {
  const mockDeps = {
    existsSync: () => true,
    fetch: mock.fn(async () => ({
      ok: false,
      status: 404,
    })),
    spawn: mock.fn(() => ({
      unref: () => {},
      kill: () => {},
    })),
    binaryPath: '/fake/path/fabrik',
  };

  const store = createFabrikStore({ autoStart: false }, mockDeps);
  const result = await store.get(Buffer.from('test'));

  assert.equal(result, null, 'Should return null on cache miss');
});

test('FabrikStore - get() returns buffer on success', async () => {
  const testData = Buffer.from('cached data');

  const mockDeps = {
    existsSync: () => true,
    fetch: mock.fn(async () => ({
      ok: true,
      status: 200,
      arrayBuffer: async () => testData.buffer,
    })),
    spawn: mock.fn(() => ({
      unref: () => {},
      kill: () => {},
    })),
    binaryPath: '/fake/path/fabrik',
  };

  const store = createFabrikStore({ autoStart: false }, mockDeps);
  const result = await store.get(Buffer.from('test'));

  assert.ok(Buffer.isBuffer(result), 'Should return a Buffer');
});

test('FabrikStore - set() sends PUT request', async () => {
  let capturedUrl;
  let capturedBody;

  const mockDeps = {
    existsSync: () => true,
    fetch: mock.fn(async (url, options) => {
      capturedUrl = url;
      capturedBody = options.body;
      return { ok: true };
    }),
    spawn: mock.fn(() => ({
      unref: () => {},
      kill: () => {},
    })),
    binaryPath: '/fake/path/fabrik',
  };

  const store = createFabrikStore({ autoStart: false }, mockDeps);
  const testKey = Buffer.from('testkey');
  const testValue = Buffer.from('testvalue');

  await store.set(testKey, testValue);

  assert.ok(capturedUrl.includes(testKey.toString('hex')), 'URL should contain hash');
  assert.equal(capturedBody, testValue, 'Body should be the value');
});

test('FabrikStore - gracefully handles errors', async () => {
  const mockDeps = {
    existsSync: () => true,
    fetch: mock.fn(async () => {
      throw new Error('Network error');
    }),
    spawn: mock.fn(() => ({
      unref: () => {},
      kill: () => {},
    })),
    binaryPath: '/fake/path/fabrik',
  };

  const store = createFabrikStore({ autoStart: false }, mockDeps);

  // Should not throw, just return null
  const result = await store.get(Buffer.from('test'));
  assert.equal(result, null, 'Should return null on error');

  // Should not throw
  await assert.doesNotReject(
    () => store.set(Buffer.from('test'), Buffer.from('value')),
    'Should not reject on set error'
  );
});
