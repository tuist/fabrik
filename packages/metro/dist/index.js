"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.FabrikStore = void 0;
const node_child_process_1 = require("node:child_process");
const node_fs_1 = require("node:fs");
const node_path_1 = require("node:path");
const PACKAGE_ROOT = (0, node_path_1.join)(__dirname, '..');
const BINARY_PATH = (0, node_path_1.join)(PACKAGE_ROOT, 'bin', process.platform === 'win32' ? 'fabrik.exe' : 'fabrik');
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
class FabrikStore {
    options;
    daemon = null;
    baseUrl;
    startPromise = null;
    constructor(options = {}) {
        this.options = {
            cacheDir: options.cacheDir || process.env.FABRIK_CACHE_DIR || '/tmp/fabrik-cache',
            upstream: options.upstream || process.env.FABRIK_UPSTREAM || '',
            maxSize: options.maxSize || process.env.FABRIK_MAX_SIZE || '5GB',
            token: options.token || process.env.TUIST_TOKEN || '',
            autoStart: options.autoStart ?? true,
            port: options.port || Number(process.env.FABRIK_PORT) || 7070,
            logLevel: options.logLevel || process.env.FABRIK_LOG_LEVEL || 'info',
        };
        this.baseUrl = `http://localhost:${this.options.port}`;
        // Check if binary exists
        if (!(0, node_fs_1.existsSync)(BINARY_PATH)) {
            throw new Error(`Fabrik binary not found at ${BINARY_PATH}.\n` +
                `The postinstall script may have failed. Try reinstalling:\n` +
                `  npm install @fabrik/metro --force\n` +
                `Or install Fabrik manually:\n` +
                `  mise use -g ubi:tuist/fabrik`);
        }
    }
    /**
     * Ensure daemon is started (lazy initialization)
     */
    async ensureDaemon() {
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
        }
        catch {
            // Not running, start it
        }
        // Start daemon
        this.startPromise = this.startDaemon();
        return this.startPromise;
    }
    /**
     * Start the Fabrik daemon in the background
     */
    async startDaemon() {
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
        this.daemon = (0, node_child_process_1.spawn)(BINARY_PATH, args, {
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
    async waitForReady(maxAttempts = 20) {
        for (let i = 0; i < maxAttempts; i++) {
            try {
                const response = await fetch(`${this.baseUrl}/health`, {
                    signal: AbortSignal.timeout(500)
                });
                if (response.ok) {
                    return;
                }
            }
            catch {
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
    async get(key) {
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
        }
        catch (error) {
            console.warn(`Fabrik cache get failed for key ${key.toString('hex')}:`, error?.message);
            return null; // Graceful degradation on errors
        }
    }
    /**
     * Set a value in the cache
     * @param key Cache key (content hash)
     * @param value Value to cache
     */
    async set(key, value) {
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
        }
        catch (error) {
            console.warn(`Fabrik cache set failed for key ${key.toString('hex')}:`, error?.message);
            // Graceful degradation - don't fail the build if caching fails
        }
    }
    /**
     * Clear the cache (optional Metro cache store method)
     */
    async clear() {
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
        }
        catch (error) {
            console.warn('Fabrik cache clear failed:', error);
        }
    }
    /**
     * Stop the Fabrik daemon (call on process exit)
     */
    stop() {
        if (this.daemon) {
            this.daemon.kill();
            this.daemon = null;
        }
    }
}
exports.FabrikStore = FabrikStore;
// Export for CommonJS compatibility
exports.default = FabrikStore;
