import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { gzip, gunzip } from 'node:zlib';
import { promisify } from 'node:util';
import { appendFileSync } from 'node:fs';

const gzipAsync = promisify(gzip);
const gunzipAsync = promisify(gunzip);

function log(message) {
  try {
    appendFileSync('/tmp/fabrik-metro.log', `${new Date().toISOString()} ${message}\n`);
  } catch {}
}

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const BINARY_PATH = join(__dirname, '..', 'bin', process.platform === 'win32' ? 'fabrik.exe' : 'fabrik');
const NULL_BYTE = 0x00;
const NULL_BYTE_BUFFER = Buffer.from([NULL_BYTE]);

// Detect development mode: check if we're in a Cargo workspace
function isDevMode() {
  // Check if Cargo.toml exists in parent directories (we're in the Fabrik repo)
  const repoRoot = join(__dirname, '..', '..', '..');
  return existsSync(join(repoRoot, 'Cargo.toml'));
}

const DEV_MODE = isDevMode();

/**
 * Create a Fabrik cache store for Metro
 *
 * @param {Object} options - Configuration options
 * @param {string} [options.cacheDir] - Local cache directory
 * @param {string} [options.upstream] - Upstream Fabrik server URL
 * @param {string} [options.maxSize='5GB'] - Maximum cache size
 * @param {string} [options.token] - Authentication token
 * @param {boolean} [options.autoStart=true] - Auto-start daemon
 * @param {number} [options.port=7070] - Daemon port
 * @param {string} [options.logLevel='info'] - Log level
 * @param {Object} [deps] - Dependencies (for testing)
 * @returns {Object} Metro cache store
 */
function createFabrikStore(options = {}, deps = {}) {
  const config = {
    cacheDir: options.cacheDir || process.env.FABRIK_CACHE_DIR || '/tmp/fabrik-cache',
    upstream: options.upstream || process.env.FABRIK_UPSTREAM || '',
    maxSize: options.maxSize || process.env.FABRIK_MAX_SIZE || '5GB',
    token: options.token || process.env.TUIST_TOKEN || '',
    autoStart: options.autoStart ?? true,
    port: options.port || Number(process.env.FABRIK_PORT) || 7070,
    logLevel: options.logLevel || process.env.FABRIK_LOG_LEVEL || 'info',
  };

  const baseUrl = `http://localhost:${config.port}`;

  // Dependency injection for testing
  const {
    fetch: fetchFn = globalThis.fetch,
    spawn: spawnFn = spawn,
    existsSync: existsSyncFn = existsSync,
    binaryPath = BINARY_PATH,
    devMode = DEV_MODE,
  } = deps;

  // Check if binary exists (skip in dev mode, we'll use cargo run)
  if (!devMode && !existsSyncFn(binaryPath)) {
    throw new Error(
      `Fabrik binary not found at ${binaryPath}.\n` +
      `The postinstall script may have failed. Try reinstalling:\n` +
      `  npm install @tuist/fabrik --force\n` +
      `Or install Fabrik manually:\n` +
      `  mise use -g ubi:tuist/fabrik`
    );
  }

  if (devMode) {
    console.log('[Fabrik Metro] Development mode detected - will use cargo run');
  }

  let daemon = null;
  let startPromise = null;

  /**
   * Ensure daemon is running (lazy initialization)
   */
  async function ensureDaemon() {
    if (!config.autoStart) {
      return; // User manages daemon
    }

    if (startPromise) {
      return startPromise; // Already starting
    }

    // Check if already running
    try {
      const response = await fetchFn(`${baseUrl}/health`, {
        signal: AbortSignal.timeout(1000),
      });
      if (response.ok) {
        return; // Already running
      }
    } catch {
      // Not running, start it
    }

    startPromise = startDaemon();
    return startPromise;
  }

  /**
   * Start the Fabrik daemon
   */
  async function startDaemon() {
    let command;
    let args;

    if (devMode) {
      // Development mode: use cargo run
      const repoRoot = join(__dirname, '..', '..', '..');
      command = 'cargo';
      args = [
        'run',
        '--',
        'daemon',
        '--config-cache-dir', config.cacheDir,
        '--config-max-cache-size', config.maxSize,
        '--config-log-level', config.logLevel,
        '--config-http-port', String(config.port),
      ];

      if (config.upstream) {
        args.push('--config-upstream', config.upstream);
      }

      if (config.token) {
        args.push('--config-jwt-token', config.token);
      }

      console.log('[Fabrik Metro] Starting daemon with: cargo run -- daemon ...');

      daemon = spawnFn(command, args, {
        detached: true,
        stdio: 'inherit', // Show output in dev mode
        cwd: repoRoot,
      });
    } else {
      // Production mode: use downloaded binary
      args = [
        'daemon',
        '--config-cache-dir', config.cacheDir,
        '--config-max-cache-size', config.maxSize,
        '--config-log-level', config.logLevel,
        '--config-http-port', String(config.port),
      ];

      if (config.upstream) {
        args.push('--config-upstream', config.upstream);
      }

      if (config.token) {
        args.push('--config-jwt-token', config.token);
      }

      daemon = spawnFn(binaryPath, args, {
        detached: true,
        stdio: 'ignore',
      });
    }

    daemon.unref();

    // Wait for daemon to be ready
    await waitForReady();
  }

  /**
   * Wait for daemon to be ready
   */
  async function waitForReady(maxAttempts = 20) {
    for (let i = 0; i < maxAttempts; i++) {
      try {
        const response = await fetchFn(`${baseUrl}/health`, {
          signal: AbortSignal.timeout(500),
        });
        if (response.ok) {
          return;
        }
      } catch {
        // Not ready yet
      }
      await new Promise(resolve => setTimeout(resolve, 300));
    }
    console.warn('Fabrik daemon took too long to start, continuing anyway...');
  }

  /**
   * Get value from cache
   */
  async function get(key) {
    try {
      await ensureDaemon();

      const hash = key.toString('hex');
      const response = await fetchFn(`${baseUrl}/api/v1/artifacts/${hash}`, {
        method: 'GET',
      });

      if (!response.ok) {
        if (response.status === 404) {
          log(`Cache miss: ${hash.substring(0, 12)}...`);
          return null; // Cache miss
        }
        throw new Error(`Failed to get from cache: ${response.statusText}`);
      }

      log(`Cache HIT: ${hash.substring(0, 12)}...`);

      // Metro stores data as gzipped, Fabrik returns it as-is
      const arrayBuffer = await response.arrayBuffer();
      const gzippedBuffer = Buffer.from(arrayBuffer);

      try {
        // Try to gunzip (Metro's HttpStore sends gzipped data)
        const buffer = await gunzipAsync(gzippedBuffer);

        // Check for NULL_BYTE prefix (indicates raw buffer vs JSON)
        if (buffer.length > 0 && buffer[0] === NULL_BYTE) {
          // Raw buffer - strip NULL_BYTE prefix
          return buffer.slice(1);
        } else {
          // JSON data - parse and return
          return JSON.parse(buffer.toString('utf8'));
        }
      } catch (gunzipError) {
        // Data is not gzipped (shouldn't happen with Metro, but handle gracefully)
        // Assume it's raw buffer data
        return gzippedBuffer;
      }
    } catch (error) {
      console.warn(`Fabrik cache get failed:`, error?.message);
      return null;
    }
  }

  /**
   * Set value in cache
   */
  async function set(key, value) {
    try {
      await ensureDaemon();

      const hash = key.toString('hex');

      // Prepare data according to Metro's HttpStore protocol
      let dataToCompress;
      if (Buffer.isBuffer(value)) {
        // For buffers: prepend NULL_BYTE, then gzip
        dataToCompress = Buffer.concat([NULL_BYTE_BUFFER, value]);
      } else {
        // For non-buffers (objects): JSON stringify, then gzip
        dataToCompress = Buffer.from(JSON.stringify(value), 'utf8');
      }

      // Gzip the data
      const gzippedData = await gzipAsync(dataToCompress, { level: 9 });

      const response = await fetchFn(`${baseUrl}/api/v1/artifacts/${hash}`, {
        method: 'PUT',
        body: gzippedData,
        headers: {
          'Content-Type': 'application/octet-stream',
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to set cache: ${response.statusText}`);
      }

      log(`Cached: ${hash.substring(0, 12)}... (${gzippedData.length} bytes)`);
    } catch (error) {
      console.warn(`[Fabrik] Cache set failed:`, error?.message);
    }
  }

  /**
   * Clear cache
   */
  async function clear() {
    try {
      const response = await fetchFn(`${baseUrl}/api/v1/admin/clear`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ confirm: true }),
      });

      if (!response.ok) {
        throw new Error(`Failed to clear cache: ${response.statusText}`);
      }
    } catch (error) {
      console.warn('Fabrik cache clear failed:', error?.message);
    }
  }

  /**
   * Stop daemon
   */
  function stop() {
    if (daemon) {
      daemon.kill();
      daemon = null;
    }
  }

  return { get, set, clear, stop };
}

// Named exports
export { createFabrikStore, createFabrikStore as FabrikStore };

// Default export
export default createFabrikStore;
