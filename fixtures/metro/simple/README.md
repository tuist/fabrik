# Metro Simple Fixture

A simple test fixture demonstrating Metro bundler integration with Fabrik caching.

## Status

✅ **Fully Working** - Metro + Fabrik cache integration complete!

**What's Working:**
- ✅ Metro HttpStore protocol (gzip + NULL_BYTE)
- ✅ Fabrik daemon lifecycle management
- ✅ Cache hits/misses working correctly
- ✅ pnpm monorepo module resolution
- ✅ Development mode (cargo run) and production mode

## Usage

```bash
# Run bundle
pnpm run bundle

# Run with debug logging
FABRIK_DEBUG=1 pnpm run bundle

# Check cache logs
tail -f /tmp/fabrik-metro.log
```

## Verified Caching

First run (cache misses):
```
2025-10-27T18:03:04.221Z Miss: 83c0970e
2025-10-27T18:03:04.397Z Set: 83c0970e (945b)
2025-10-27T18:03:04.543Z Miss: 69431b14
2025-10-27T18:03:04.545Z Set: 4f27a9a7 (552b)
2025-10-27T18:03:04.754Z Set: 69431b14 (5091b)
```

Second run (cache hits):
```
2025-10-27T18:03:12.404Z Hit: 83c0970e
2025-10-27T18:03:12.409Z Hit: 4f27a9a7
2025-10-27T18:03:12.410Z Hit: 69431b14
```

This confirms Metro → Fabrik caching is working perfectly!
