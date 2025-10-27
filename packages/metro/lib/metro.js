const { spawn } = require('node:child_process');
const { existsSync, appendFileSync } = require('node:fs');
const { join } = require('node:path');
const { promisify } = require('node:util');
const { gzip: gzipCb, gunzip: gunzipCb } = require('node:zlib');

const gzip = promisify(gzipCb);
const gunzip = promisify(gunzipCb);

const BINARY_PATH = join(__dirname, '..', 'bin', process.platform === 'win32' ? 'fabrik.exe' : 'fabrik');
const NULL_BYTE_BUFFER = Buffer.from([0x00]);
const DEV_MODE = existsSync(join(__dirname, '..', '..', '..', 'Cargo.toml'));

function log(message) {
  if (process.env.FABRIK_DEBUG) {
    try {
      appendFileSync('/tmp/fabrik-metro.log', `${new Date().toISOString()} ${message}\n`);
    } catch {}
  }
}

/**
 * Create a Fabrik cache store for Metro
 *
 * @param {Object} options - Configuration options
 * @param {string} [options.cacheDir] - Cache directory (default: /tmp/fabrik-cache)
 * @param {string} [options.upstream] - Upstream Fabrik server URL
 * @param {string} [options.maxSize] - Max cache size (default: 5GB)
 * @param {string} [options.token] - Auth token
 * @param {boolean} [options.autoStart] - Auto-start daemon (default: true)
 * @param {number} [options.port] - Daemon port (default: 7070)
 * @param {string} [options.logLevel] - Log level (default: info)
 */
function FabrikStore(options = {}) {
  const config = {
    cacheDir: options.cacheDir || process.env.FABRIK_CACHE_DIR || '/tmp/fabrik-cache',
    upstream: options.upstream || process.env.FABRIK_UPSTREAM,
    maxSize: options.maxSize || process.env.FABRIK_MAX_SIZE || '5GB',
    token: options.token || process.env.TUIST_TOKEN,
    autoStart: options.autoStart ?? true,
    port: options.port || Number(process.env.FABRIK_PORT) || 7070,
    logLevel: options.logLevel || process.env.FABRIK_LOG_LEVEL || 'info',
  };

  const baseUrl = `http://localhost:${config.port}`;
  let daemon = null;
  let startPromise = null;

  // Validate binary exists in production mode
  if (!DEV_MODE && !existsSync(BINARY_PATH)) {
    throw new Error(
      `Fabrik binary not found at ${BINARY_PATH}. ` +
      `Run: npm install @tuist/fabrik --force`
    );
  }

  async function ensureDaemon() {
    if (!config.autoStart) return;
    if (startPromise) return startPromise;

    // Check if already running
    try {
      const res = await fetch(`${baseUrl}/health`, { signal: AbortSignal.timeout(1000) });
      if (res.ok) return;
    } catch {}

    // Start daemon
    startPromise = (async () => {
      const args = [
        'daemon',
        '--config-cache-dir', config.cacheDir,
        '--config-max-cache-size', config.maxSize,
        '--config-log-level', config.logLevel,
        '--config-http-port', String(config.port),
      ];

      if (config.upstream) args.push('--config-upstream', config.upstream);
      if (config.token) args.push('--config-jwt-token', config.token);

      if (DEV_MODE) {
        const repoRoot = join(__dirname, '..', '..', '..');
        daemon = spawn('cargo', ['run', '--', ...args], {
          detached: true,
          stdio: 'inherit',
          cwd: repoRoot,
        });
      } else {
        daemon = spawn(BINARY_PATH, args, {
          detached: true,
          stdio: 'ignore',
        });
      }

      daemon.unref();

      // Wait for ready
      for (let i = 0; i < 20; i++) {
        try {
          const res = await fetch(`${baseUrl}/health`, { signal: AbortSignal.timeout(500) });
          if (res.ok) return;
        } catch {}
        await new Promise(r => setTimeout(r, 300));
      }
    })();

    return startPromise;
  }

  async function get(key) {
    try {
      await ensureDaemon();
      const hash = key.toString('hex');
      const res = await fetch(`${baseUrl}/api/v1/artifacts/${hash}`);

      if (!res.ok) {
        if (res.status === 404) {
          log(`Miss: ${hash.slice(0, 8)}`);
          return null;
        }
        throw new Error(`Get failed: ${res.statusText}`);
      }

      log(`Hit: ${hash.slice(0, 8)}`);

      // Gunzip and decode Metro's HttpStore protocol
      const gzipped = Buffer.from(await res.arrayBuffer());
      const buffer = await gunzip(gzipped);

      // NULL_BYTE prefix = raw buffer, otherwise JSON
      return buffer[0] === 0x00 ? buffer.slice(1) : JSON.parse(buffer.toString());
    } catch (err) {
      log(`Get error: ${err.message}`);
      return null; // Graceful degradation
    }
  }

  async function set(key, value) {
    try {
      await ensureDaemon();
      const hash = key.toString('hex');

      // Encode per Metro's HttpStore protocol: gzip(NULL_BYTE + buffer) or gzip(JSON)
      const data = Buffer.isBuffer(value)
        ? Buffer.concat([NULL_BYTE_BUFFER, value])
        : Buffer.from(JSON.stringify(value));
      const gzipped = await gzip(data, { level: 9 });

      const res = await fetch(`${baseUrl}/api/v1/artifacts/${hash}`, {
        method: 'PUT',
        body: gzipped,
        headers: { 'Content-Type': 'application/octet-stream' },
      });

      if (!res.ok) throw new Error(`Set failed: ${res.statusText}`);
      log(`Set: ${hash.slice(0, 8)} (${gzipped.length}b)`);
    } catch (err) {
      log(`Set error: ${err.message}`);
    }
  }

  return { get, set };
}

module.exports = { FabrikStore };
