# Fabrik C API Documentation

The Fabrik C API provides a thread-safe interface for integrating Fabrik cache into C/C++ applications and other toolchains.

## Features

- ✅ Thread-safe operations
- ✅ Content-addressed storage
- ✅ Simple error handling
- ✅ Cross-platform (Linux, macOS, Windows)
- ✅ Zero-copy where possible
- ✅ Comprehensive error messages

## Installation

### From Releases

Download the pre-built libraries for your platform from the [releases page](https://github.com/tuist/fabrik/releases):

- **Linux**: `libfabrik.so` + `fabrik.h`
- **macOS**: `libfabrik.dylib` + `fabrik.h`
- **Windows**: `fabrik.dll` + `fabrik.h`

### Building from Source

```bash
# Clone the repository
git clone https://github.com/tuist/fabrik.git
cd fabrik

# Build the library
cargo build --release --lib

# The library will be at:
# - Linux: target/release/libfabrik.so
# - macOS: target/release/libfabrik.dylib
# - Windows: target/release/fabrik.dll

# The header file will be at:
# include/fabrik.h
```

## Quick Start

### Basic Example

```c
#include <fabrik.h>
#include <stdio.h>

int main() {
    // Initialize cache
    FabrikCache *cache = fabrik_cache_init("/tmp/my-cache");
    if (!cache) {
        fprintf(stderr, "Failed to init: %s\n", fabrik_last_error());
        return 1;
    }

    // Store data
    const char *data = "Hello, World!";
    const char *hash = "abc123...";
    if (fabrik_cache_put(cache, hash, (uint8_t*)data, 13) != FABRIK_OK) {
        fprintf(stderr, "Failed to put: %s\n", fabrik_last_error());
    }

    // Retrieve data
    uint8_t buffer[1024];
    size_t bytes_read;
    if (fabrik_cache_get(cache, hash, buffer, sizeof(buffer), &bytes_read) == FABRIK_OK) {
        printf("Retrieved %zu bytes\n", bytes_read);
    }

    // Cleanup
    fabrik_cache_free(cache);
    return 0;
}
```

### Building Your Application

#### GCC/Clang

```bash
gcc -o myapp myapp.c -I/path/to/fabrik/include -L/path/to/fabrik/lib -lfabrik
```

#### CMake

```cmake
cmake_minimum_required(VERSION 3.10)
project(MyApp)

# Find Fabrik
find_library(FABRIK_LIB fabrik PATHS /path/to/fabrik/lib)
include_directories(/path/to/fabrik/include)

add_executable(myapp myapp.c)
target_link_libraries(myapp ${FABRIK_LIB})
```

#### pkg-config

If Fabrik is installed system-wide:

```bash
gcc -o myapp myapp.c $(pkg-config --cflags --libs fabrik)
```

## API Reference

### Types

#### `FabrikCache`

Opaque handle to a cache instance. Must be freed with `fabrik_cache_free()`.

```c
typedef struct FabrikCache FabrikCache;
```

#### Error Codes

```c
#define FABRIK_OK                 0   // Success
#define FABRIK_ERROR             -1   // General error
#define FABRIK_ERROR_NOT_FOUND   -2   // Artifact not found
#define FABRIK_ERROR_INVALID_HASH -3  // Invalid hash format
#define FABRIK_ERROR_IO          -4   // I/O error
```

### Functions

#### `fabrik_cache_init`

Initialize a new cache instance.

```c
FabrikCache* fabrik_cache_init(const char *cache_dir);
```

**Parameters:**
- `cache_dir`: Path to cache directory (NULL-terminated C string)

**Returns:**
- Pointer to `FabrikCache` on success
- `NULL` on error (use `fabrik_last_error()` for details)

**Example:**
```c
FabrikCache *cache = fabrik_cache_init("/home/user/.cache/fabrik");
if (!cache) {
    fprintf(stderr, "Init failed: %s\n", fabrik_last_error());
}
```

---

#### `fabrik_cache_free`

Free a cache instance.

```c
void fabrik_cache_free(FabrikCache *cache);
```

**Parameters:**
- `cache`: Cache instance to free

**Notes:**
- Safe to call with `NULL`
- Must not use the cache after calling this function

---

#### `fabrik_cache_put`

Store an artifact in the cache.

```c
int fabrik_cache_put(
    FabrikCache *cache,
    const char *hash,
    const uint8_t *data,
    size_t data_len
);
```

**Parameters:**
- `cache`: Cache instance
- `hash`: Content hash (SHA256, 64 hex characters)
- `data`: Data to store
- `data_len`: Length of data in bytes

**Returns:**
- `FABRIK_OK` on success
- Error code on failure

**Example:**
```c
const char *data = "artifact content";
const char *hash = "abc123def456...";
int result = fabrik_cache_put(cache, hash, (uint8_t*)data, strlen(data));
if (result != FABRIK_OK) {
    fprintf(stderr, "Put failed: %s\n", fabrik_last_error());
}
```

---

#### `fabrik_cache_get`

Retrieve an artifact from the cache.

```c
int fabrik_cache_get(
    FabrikCache *cache,
    const char *hash,
    uint8_t *output_buffer,
    size_t buffer_size,
    size_t *bytes_written
);
```

**Parameters:**
- `cache`: Cache instance
- `hash`: Content hash to retrieve
- `output_buffer`: Buffer to write data (must be pre-allocated)
- `buffer_size`: Size of output buffer
- `bytes_written`: Output parameter for actual bytes written

**Returns:**
- `FABRIK_OK` on success
- `FABRIK_ERROR_NOT_FOUND` if artifact doesn't exist
- `FABRIK_ERROR` if buffer is too small or other error

**Example:**
```c
uint8_t buffer[4096];
size_t bytes_read;
int result = fabrik_cache_get(cache, hash, buffer, sizeof(buffer), &bytes_read);
if (result == FABRIK_OK) {
    printf("Read %zu bytes\n", bytes_read);
} else if (result == FABRIK_ERROR_NOT_FOUND) {
    printf("Artifact not found\n");
}
```

---

#### `fabrik_cache_exists`

Check if an artifact exists in the cache.

```c
int fabrik_cache_exists(
    FabrikCache *cache,
    const char *hash,
    int *exists
);
```

**Parameters:**
- `cache`: Cache instance
- `hash`: Content hash to check
- `exists`: Output parameter (1 if exists, 0 if not)

**Returns:**
- `FABRIK_OK` on success
- Error code on failure

**Example:**
```c
int exists;
if (fabrik_cache_exists(cache, hash, &exists) == FABRIK_OK) {
    printf("Artifact %s\n", exists ? "exists" : "not found");
}
```

---

#### `fabrik_cache_delete`

Delete an artifact from the cache.

```c
int fabrik_cache_delete(
    FabrikCache *cache,
    const char *hash
);
```

**Parameters:**
- `cache`: Cache instance
- `hash`: Content hash to delete

**Returns:**
- `FABRIK_OK` on success
- Error code on failure

**Example:**
```c
if (fabrik_cache_delete(cache, hash) == FABRIK_OK) {
    printf("Artifact deleted\n");
}
```

---

#### `fabrik_last_error`

Get the last error message for the current thread.

```c
const char* fabrik_last_error(void);
```

**Returns:**
- Pointer to NULL-terminated error string
- `NULL` if no error

**Notes:**
- Error message is valid until next API call
- Do not free the returned pointer
- Thread-local (each thread has its own error)

---

#### `fabrik_version`

Get the library version string.

```c
const char* fabrik_version(void);
```

**Returns:**
- Pointer to version string (e.g., "0.8.1")
- String is statically allocated, do not free

---

## Error Handling

All functions follow a consistent error handling pattern:

```c
int result = fabrik_cache_put(cache, hash, data, len);
if (result != FABRIK_OK) {
    // Get detailed error message
    const char *error = fabrik_last_error();
    fprintf(stderr, "Operation failed: %s\n", error);

    // Check specific error codes
    if (result == FABRIK_ERROR_NOT_FOUND) {
        // Handle not found
    } else if (result == FABRIK_ERROR_IO) {
        // Handle I/O error
    }
}
```

## Thread Safety

The Fabrik C API is **fully thread-safe**:

- Multiple threads can safely access the same cache instance
- Each thread has its own error state (`fabrik_last_error()`)
- No global state or locks required by the caller

```c
// Thread-safe example
void* worker_thread(void *arg) {
    FabrikCache *cache = (FabrikCache*)arg;

    // Safe to call from multiple threads
    int exists;
    fabrik_cache_exists(cache, "hash123", &exists);

    return NULL;
}
```

## Platform-Specific Notes

### Linux

- Library: `libfabrik.so`
- Runtime library path: Set `LD_LIBRARY_PATH` or install to `/usr/lib`

```bash
export LD_LIBRARY_PATH=/path/to/fabrik/lib:$LD_LIBRARY_PATH
./myapp
```

### macOS

- Library: `libfabrik.dylib`
- Runtime library path: Set `DYLD_LIBRARY_PATH` or use `@rpath`

```bash
export DYLD_LIBRARY_PATH=/path/to/fabrik/lib:$DYLD_LIBRARY_PATH
./myapp
```

### Windows

- Library: `fabrik.dll`
- Place DLL in same directory as executable or in system PATH

## Examples

See the [examples/c](../examples/c) directory for complete working examples:

- `example.c`: Comprehensive demonstration of all API functions
- `Makefile`: Build configuration

To run the example:

```bash
cd examples/c
make
make run
```

## Integration Patterns

### Build System Integration

Example of integrating Fabrik into a build system:

```c
#include <fabrik.h>

void check_build_cache(const char *artifact_hash) {
    FabrikCache *cache = fabrik_cache_init(".build/cache");
    if (!cache) return;

    int exists;
    if (fabrik_cache_exists(cache, artifact_hash, &exists) == FABRIK_OK) {
        if (exists) {
            // Skip rebuild, artifact is cached
            uint8_t buffer[1024*1024];  // 1MB buffer
            size_t bytes_read;
            fabrik_cache_get(cache, artifact_hash, buffer, sizeof(buffer), &bytes_read);
            // Use cached artifact...
        }
    }

    fabrik_cache_free(cache);
}
```

### Error Logging Wrapper

```c
#define FABRIK_CHECK(op, msg) \
    if ((op) != FABRIK_OK) { \
        log_error("%s: %s", msg, fabrik_last_error()); \
        goto cleanup; \
    }

int build_with_cache() {
    FabrikCache *cache = fabrik_cache_init("/tmp/cache");
    if (!cache) return -1;

    FABRIK_CHECK(fabrik_cache_put(cache, hash, data, len), "Failed to cache artifact");
    FABRIK_CHECK(fabrik_cache_get(cache, hash, buf, sizeof(buf), &n), "Failed to retrieve");

cleanup:
    fabrik_cache_free(cache);
    return 0;
}
```

## Troubleshooting

### Library Not Found

**Symptom:**
```
error while loading shared libraries: libfabrik.so: cannot open shared object file
```

**Solution:**
- Set `LD_LIBRARY_PATH` (Linux) or `DYLD_LIBRARY_PATH` (macOS)
- Install library to system path
- Use `-rpath` linker flag

### Initialization Fails

**Symptom:**
```c
fabrik_cache_init() returns NULL
```

**Common causes:**
- Invalid cache directory path
- Insufficient permissions
- Disk full

**Solution:**
```c
FabrikCache *cache = fabrik_cache_init("/tmp/cache");
if (!cache) {
    fprintf(stderr, "Init failed: %s\n", fabrik_last_error());
    // Check permissions, disk space, path validity
}
```

### Buffer Too Small

**Symptom:**
```
fabrik_cache_get() returns FABRIK_ERROR
fabrik_last_error() says "Buffer too small"
```

**Solution:**
Use a larger buffer or dynamically allocate based on artifact size.

## Support

- **Documentation**: https://github.com/tuist/fabrik
- **Issues**: https://github.com/tuist/fabrik/issues
- **Discord**: https://discord.gg/tuist

## License

MIT License - see [LICENSE](../LICENSE) file for details.
