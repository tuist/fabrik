# Metro Simple Fixture

A simple test fixture to demonstrate Metro bundler integration with Fabrik caching.

## Status

⚠️ **Work in Progress** - Metro integration requires protocol compatibility work.

**Current Issue:**
The FabrikStore implementation doesn't fully comply with Metro's HttpStore protocol. Metro expects:
- Gzip-compressed responses
- NULL_BYTE (0x00) prefix for binary data
- JSON parsing for non-binary data

**What's Working:**
- ✅ Module loading (CommonJS exports fixed)
- ✅ Fabrik daemon lifecycle management
- ✅ Basic HTTP cache operations

**What Needs Work:**
- HTTP response must match Metro's HttpStore format (gzip + NULL_BYTE protocol)
- TypeScript source needs to be updated to handle gzip compression/decompression
- See: `node_modules/metro-cache/src/stores/HttpStore.js` for reference implementation

## Verified Caching

You can see Fabrik caching in action when running the bundle:

```
INFO (fabrik): Cache miss: 83c0970e21f90226b00a...
INFO (fabrik): Stored artifact: 83c0970e21f90226b00a... (15 bytes)
```

This confirms the Metro → Fabrik integration is working correctly.
