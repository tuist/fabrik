import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const BINARY_PATH = join(__dirname, '..', 'bin', process.platform === 'win32' ? 'fabrik.exe' : 'fabrik');

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
      `  npm install @fabrik/metro --force\n` +
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
          return null; // Cache miss
        }
        throw new Error(`Failed to get from cache: ${response.statusText}`);
      }

      const arrayBuffer = await response.arrayBuffer();
      return Buffer.from(arrayBuffer);
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
      const response = await fetchFn(`${baseUrl}/api/v1/artifacts/${hash}`, {
        method: 'PUT',
        body: value,
        headers: {
          'Content-Type': 'application/octet-stream',
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to set cache: ${response.statusText}`);
      }
    } catch (error) {
      console.warn(`Fabrik cache set failed:`, error?.message);
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
