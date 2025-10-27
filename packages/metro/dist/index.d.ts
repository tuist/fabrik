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
export declare class FabrikStore {
    private options;
    private daemon;
    private baseUrl;
    private startPromise;
    constructor(options?: FabrikStoreOptions);
    /**
     * Ensure daemon is started (lazy initialization)
     */
    private ensureDaemon;
    /**
     * Start the Fabrik daemon in the background
     */
    private startDaemon;
    /**
     * Wait for Fabrik daemon to be ready
     */
    private waitForReady;
    /**
     * Get a value from the cache
     * @param key Cache key (content hash)
     */
    get(key: Buffer): Promise<Buffer | null>;
    /**
     * Set a value in the cache
     * @param key Cache key (content hash)
     * @param value Value to cache
     */
    set(key: Buffer, value: Buffer): Promise<void>;
    /**
     * Clear the cache (optional Metro cache store method)
     */
    clear(): Promise<void>;
    /**
     * Stop the Fabrik daemon (call on process exit)
     */
    stop(): void;
}
export default FabrikStore;
