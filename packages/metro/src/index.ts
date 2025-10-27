import { spawn, ChildProcess } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

const PACKAGE_ROOT = join(__dirname, '..');
const BINARY_PATH = join(
  PACKAGE_ROOT,
  'bin',
  process.platform === 'win32' ? 'fabrik.exe' : 'fabrik'
);

export interface FabrikStoreOptions {
  /**
   * Directory for local cache storage
   * @default process.env.FABRIK_CACHE_DIR || '/tmp/fabrik-cache'
   */
  cacheDir?: string;

  /**
   * Upstream Fabrik server URL (e.g., 'grpc://cache.tuist.io:7070')
   */
  upstream?: string;

  /**
   * Maximum cache size (e.g., '5GB', '500MB')
   * @default '5GB'
   */
  maxSize?: string;

  /**
   * Authentication token for upstream cache
   */
  token?: string;

  /**
   * Automatically start Fabrik daemon if not running
   * @default true
   */
  autoStart?: boolean;

  /**
   * Port for Fabrik daemon
   * @default 7070
   */
  port?: number;

  /**
   * Log level for Fabrik daemon
   * @default 'info'
   */
  logLevel?: 'debug' | 'info' | 'warn' | 'error';
}

/**
 * Metro cache store implementation for Fabrik
 *
 * This store integrates Metro with Fabrik's multi-layer build cache.
 * It automatically manages the Fabrik daemon lifecycle and provides
 * transparent caching through Fabrik's protocol.
 *
 * @example
 * ```javascript
 * // metro.config.js
 * const { FabrikStore } = require('@fabrik/metro');
 *
 * module.exports = {
 *   cacheStores: [
 *     new FabrikStore({
 *       cacheDir: '.fabrik/cache',
 *       upstream: 'grpc://cache.tuist.io:7070',
 *       maxSize: '5GB',
 *     }),
 *   ],
 * };
 * ```
 */
export class FabrikStore {
  private options: Required<FabrikStoreOptions>;
  private daemon: ChildProcess | null = null;
  private baseUrl: string;
  private startPromise: Promise<void> | null = null;

  constructor(options: FabrikStoreOptions = {}) {
    this.options = {
      cacheDir: options.cacheDir || process.env.FABRIK_CACHE_DIR || '/tmp/fabrik-cache',
      upstream: options.upstream || process.env.FABRIK_UPSTREAM || '',
      maxSize: options.maxSize || process.env.FABRIK_MAX_SIZE || '5GB',
      token: options.token || process.env.TUIST_TOKEN || '',
      autoStart: options.autoStart ?? true,
      port: options.port || Number(process.env.FABRIK_PORT) || 7070,
      logLevel: (options.logLevel as any) || process.env.FABRIK_LOG_LEVEL || 'info',
    };

    this.baseUrl = `http://localhost:${this.options.port}`;

    // Check if binary exists
    if (!existsSync(BINARY_PATH)) {
      throw new Error(
        `Fabrik binary not found at ${BINARY_PATH}.\n` +
        `The postinstall script may have failed. Try reinstalling:\n` +
        `  npm install @fabrik/metro --force\n` +
        `Or install Fabrik manually:\n` +
        `  mise use -g ubi:tuist/fabrik`
      );
    }
  }

  /**
   * Ensure daemon is started (lazy initialization)
   */
  private async ensureDaemon(): Promise<void> {
    if (!this.options.autoStart) {
      return; // User manages daemon themselves
    }

    if (this.startPromise) {
      return this.startPromise; // Already starting
    }

    // Check if daemon is already running
    try {
      const response = await fetch(`${this.baseUrl}/health`, {
        signal: AbortSignal.timeout(1000)
      });
      if (response.ok) {
        return; // Already running
      }
    } catch {
      // Not running, start it
    }

    // Start daemon
    this.startPromise = this.startDaemon();
    return this.startPromise;
  }

  /**
   * Start the Fabrik daemon in the background
   */
  private async startDaemon(): Promise<void> {
    const args = [
      'daemon',
      '--config-cache-dir', this.options.cacheDir,
      '--config-max-cache-size', this.options.maxSize,
      '--config-log-level', this.options.logLevel,
    ];

    if (this.options.upstream) {
      args.push('--config-upstream', this.options.upstream);
    }

    if (this.options.token) {
      args.push('--config-jwt-token', this.options.token);
    }

    // Start daemon as detached process
    this.daemon = spawn(BINARY_PATH, args, {
      detached: true,
      stdio: 'ignore',
    });

    // Unref so parent process can exit
    this.daemon.unref();

    // Wait for daemon to be ready
    await this.waitForReady();
  }

  /**
   * Wait for Fabrik daemon to be ready
   */
  private async waitForReady(maxAttempts = 20): Promise<void> {
    for (let i = 0; i < maxAttempts; i++) {
      try {
        const response = await fetch(`${this.baseUrl}/health`, {
          signal: AbortSignal.timeout(500)
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
   * Get a value from the cache
   * @param key Cache key (content hash)
   */
  async get(key: Buffer): Promise<Buffer | null> {
    try {
      // Ensure daemon is running
      await this.ensureDaemon();

      const hash = key.toString('hex');
      const response = await fetch(`${this.baseUrl}/api/v1/artifacts/${hash}`, {
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
    } catch (error: any) {
      console.warn(`Fabrik cache get failed for key ${key.toString('hex')}:`, error?.message);
      return null; // Graceful degradation on errors
    }
  }

  /**
   * Set a value in the cache
   * @param key Cache key (content hash)
   * @param value Value to cache
   */
  async set(key: Buffer, value: Buffer): Promise<void> {
    try {
      // Ensure daemon is running
      await this.ensureDaemon();

      const hash = key.toString('hex');
      const response = await fetch(`${this.baseUrl}/api/v1/artifacts/${hash}`, {
        method: 'PUT',
        body: value,
        headers: {
          'Content-Type': 'application/octet-stream',
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to set cache: ${response.statusText}`);
      }
    } catch (error: any) {
      console.warn(`Fabrik cache set failed for key ${key.toString('hex')}:`, error?.message);
      // Graceful degradation - don't fail the build if caching fails
    }
  }

  /**
   * Clear the cache (optional Metro cache store method)
   */
  async clear(): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/api/v1/admin/clear`, {
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
      console.warn('Fabrik cache clear failed:', error);
    }
  }

  /**
   * Stop the Fabrik daemon (call on process exit)
   */
  stop(): void {
    if (this.daemon) {
      this.daemon.kill();
      this.daemon = null;
    }
  }
}

// Export for CommonJS compatibility
export default FabrikStore;
