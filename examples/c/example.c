/**
 * Fabrik Cache C API Example
 *
 * This example demonstrates how to use the Fabrik C API to:
 * - Initialize a cache
 * - Store artifacts
 * - Check existence
 * - Retrieve artifacts
 * - Delete artifacts
 * - Handle errors
 *
 * Build:
 *   gcc -o example example.c -I../../include -L../../target/release -lfabrik
 *
 * Run:
 *   LD_LIBRARY_PATH=../../target/release ./example
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../../../include/fabrik.h"

#define CHECK_ERROR(result, operation)                                         \
    if ((result) != FABRIK_OK) {                                              \
        fprintf(stderr, "[ERROR] %s failed: %s\n", operation,                 \
                fabrik_last_error());                                          \
        fabrik_cache_free(cache);                                             \
        return 1;                                                              \
    }

int main() {
    printf("=== Fabrik Cache C API Example ===\n\n");
    printf("Library version: %s\n\n", fabrik_version());

    // Initialize cache
    printf("1. Initializing cache...\n");
    FabrikCache *cache = fabrik_cache_init("/tmp/fabrik-c-example");
    if (!cache) {
        fprintf(stderr, "[ERROR] Failed to initialize cache: %s\n",
                fabrik_last_error());
        return 1;
    }
    printf("   ✓ Cache initialized\n\n");

    // Put artifact
    printf("2. Storing artifact...\n");
    const char *test_data = "Hello from Fabrik C API!";
    const char *test_hash =
        "abc123def456789abc123def456789abc123def456789abc123def456789abc1";

    int result = fabrik_cache_put(cache, test_hash, (const uint8_t *)test_data,
                                   strlen(test_data));
    CHECK_ERROR(result, "Put artifact");
    printf("   ✓ Artifact stored: %s\n", test_hash);
    printf("   Size: %zu bytes\n\n", strlen(test_data));

    // Check existence
    printf("3. Checking existence...\n");
    int exists = 0;
    result = fabrik_cache_exists(cache, test_hash, &exists);
    CHECK_ERROR(result, "Check existence");
    printf("   ✓ Artifact exists: %s\n\n", exists ? "yes" : "no");

    // Get artifact
    printf("4. Retrieving artifact...\n");
    uint8_t buffer[1024];
    size_t bytes_read = 0;
    result = fabrik_cache_get(cache, test_hash, buffer, sizeof(buffer),
                               &bytes_read);
    CHECK_ERROR(result, "Get artifact");

    // Null-terminate for printing
    buffer[bytes_read] = '\0';
    printf("   ✓ Retrieved %zu bytes\n", bytes_read);
    printf("   Content: %s\n\n", buffer);

    // Verify content
    if (strcmp((char *)buffer, test_data) == 0) {
        printf("   ✓ Content verification passed\n\n");
    } else {
        fprintf(stderr, "   ✗ Content verification failed\n\n");
        fabrik_cache_free(cache);
        return 1;
    }

    // Delete artifact
    printf("5. Deleting artifact...\n");
    result = fabrik_cache_delete(cache, test_hash);
    CHECK_ERROR(result, "Delete artifact");
    printf("   ✓ Artifact deleted\n\n");

    // Verify deletion
    printf("6. Verifying deletion...\n");
    exists = 0;
    result = fabrik_cache_exists(cache, test_hash, &exists);
    CHECK_ERROR(result, "Check existence after deletion");
    printf("   ✓ Artifact exists after deletion: %s\n\n", exists ? "yes" : "no");

    if (!exists) {
        printf("   ✓ Deletion verified\n\n");
    } else {
        fprintf(stderr, "   ✗ Deletion verification failed\n\n");
        fabrik_cache_free(cache);
        return 1;
    }

    // Test error handling - try to get non-existent artifact
    printf("7. Testing error handling...\n");
    result = fabrik_cache_get(cache, "nonexistent", buffer, sizeof(buffer),
                               &bytes_read);
    if (result == FABRIK_ERROR_NOT_FOUND) {
        printf("   ✓ Correctly returned NOT_FOUND error\n");
        printf("   Error message: %s\n\n", fabrik_last_error());
    } else {
        fprintf(stderr, "   ✗ Expected NOT_FOUND error\n\n");
    }

    // Cleanup
    printf("8. Cleaning up...\n");
    fabrik_cache_free(cache);
    printf("   ✓ Cache freed\n\n");

    printf("=== All tests passed! ===\n");
    return 0;
}
