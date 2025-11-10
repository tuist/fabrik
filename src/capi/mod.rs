//! C API (FFI) for Fabrik cache
//!
//! This module provides a C-compatible API for integrating Fabrik into other toolchains.
//! The API is thread-safe and designed for use in C/C++ applications.
//!
//! # Memory Management
//!
//! - All strings returned by the API must be freed using `fabrik_free_string()`
//! - Error messages are owned by the caller and must be freed
//! - Cache handles must be freed using `fabrik_cache_free()`
//!
//! # Error Handling
//!
//! All functions return error codes:
//! - 0: Success
//! - Non-zero: Error (use `fabrik_last_error()` to get error message)

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::Mutex;

use crate::storage::{FilesystemStorage, Storage};

// Thread-local error storage
thread_local! {
    static LAST_ERROR: Mutex<Option<CString>> = const { Mutex::new(None) };
}

/// Opaque handle to a Fabrik cache instance
#[repr(C)]
pub struct FabrikCache {
    storage: FilesystemStorage,
}

/// Result codes
pub const FABRIK_OK: c_int = 0;
pub const FABRIK_ERROR: c_int = -1;
pub const FABRIK_ERROR_NOT_FOUND: c_int = -2;
pub const FABRIK_ERROR_INVALID_HASH: c_int = -3;
pub const FABRIK_ERROR_IO: c_int = -4;

/// Store an error message in thread-local storage
fn set_last_error(err: impl std::fmt::Display) {
    let err_msg = format!("{}", err);
    if let Ok(c_string) = CString::new(err_msg) {
        LAST_ERROR.with(|last| {
            *last.lock().unwrap() = Some(c_string);
        });
    }
}

/// Clear the last error
fn clear_last_error() {
    LAST_ERROR.with(|last| {
        *last.lock().unwrap() = None;
    });
}

/// Initialize a new Fabrik cache instance
///
/// # Arguments
/// * `cache_dir` - Path to cache directory (NULL-terminated C string)
///
/// # Returns
/// * Pointer to FabrikCache on success
/// * NULL on error (use `fabrik_last_error()` to get error message)
///
/// # Safety
/// * `cache_dir` must be a valid NULL-terminated C string
/// * Returned pointer must be freed with `fabrik_cache_free()`
#[no_mangle]
pub unsafe extern "C" fn fabrik_cache_init(cache_dir: *const c_char) -> *mut FabrikCache {
    clear_last_error();

    if cache_dir.is_null() {
        set_last_error("cache_dir is NULL");
        return ptr::null_mut();
    }

    let cache_dir_str = match CStr::from_ptr(cache_dir).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in cache_dir: {}", e));
            return ptr::null_mut();
        }
    };

    match FilesystemStorage::new(cache_dir_str) {
        Ok(storage) => Box::into_raw(Box::new(FabrikCache { storage })),
        Err(e) => {
            set_last_error(format!("Failed to initialize cache: {}", e));
            ptr::null_mut()
        }
    }
}

/// Free a Fabrik cache instance
///
/// # Safety
/// * `cache` must be a valid pointer returned by `fabrik_cache_init()`
/// * Must not be used after calling this function
#[no_mangle]
pub unsafe extern "C" fn fabrik_cache_free(cache: *mut FabrikCache) {
    if !cache.is_null() {
        let _ = Box::from_raw(cache);
    }
}

/// Get an artifact from the cache
///
/// # Arguments
/// * `cache` - Cache instance
/// * `hash` - Content hash (NULL-terminated C string)
/// * `output_buffer` - Buffer to write data (must be pre-allocated)
/// * `buffer_size` - Size of output buffer
/// * `bytes_written` - Output: actual bytes written
///
/// # Returns
/// * `FABRIK_OK` on success
/// * `FABRIK_ERROR_NOT_FOUND` if artifact not found
/// * `FABRIK_ERROR` on other errors
///
/// # Safety
/// * All pointers must be valid
/// * `output_buffer` must have at least `buffer_size` bytes allocated
#[no_mangle]
pub unsafe extern "C" fn fabrik_cache_get(
    cache: *mut FabrikCache,
    hash: *const c_char,
    output_buffer: *mut u8,
    buffer_size: usize,
    bytes_written: *mut usize,
) -> c_int {
    clear_last_error();

    if cache.is_null() || hash.is_null() || output_buffer.is_null() || bytes_written.is_null() {
        set_last_error("NULL pointer argument");
        return FABRIK_ERROR;
    }

    let cache = &(*cache);
    let hash_str = match CStr::from_ptr(hash).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in hash: {}", e));
            return FABRIK_ERROR;
        }
    };

    match cache.storage.get(hash_str.as_bytes()) {
        Ok(Some(data)) => {
            if data.len() > buffer_size {
                set_last_error(format!(
                    "Buffer too small: need {} bytes, have {}",
                    data.len(),
                    buffer_size
                ));
                return FABRIK_ERROR;
            }

            ptr::copy_nonoverlapping(data.as_ptr(), output_buffer, data.len());
            *bytes_written = data.len();
            FABRIK_OK
        }
        Ok(None) => {
            set_last_error(format!("Artifact not found: {}", hash_str));
            FABRIK_ERROR_NOT_FOUND
        }
        Err(e) => {
            set_last_error(format!("Failed to get artifact: {}", e));
            FABRIK_ERROR
        }
    }
}

/// Put an artifact into the cache
///
/// # Arguments
/// * `cache` - Cache instance
/// * `hash` - Content hash (NULL-terminated C string)
/// * `data` - Data to store
/// * `data_len` - Length of data in bytes
///
/// # Returns
/// * `FABRIK_OK` on success
/// * `FABRIK_ERROR` on error
///
/// # Safety
/// * All pointers must be valid
/// * `data` must have at least `data_len` bytes
#[no_mangle]
pub unsafe extern "C" fn fabrik_cache_put(
    cache: *mut FabrikCache,
    hash: *const c_char,
    data: *const u8,
    data_len: usize,
) -> c_int {
    clear_last_error();

    if cache.is_null() || hash.is_null() || data.is_null() {
        set_last_error("NULL pointer argument");
        return FABRIK_ERROR;
    }

    let cache = &(*cache);
    let hash_str = match CStr::from_ptr(hash).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in hash: {}", e));
            return FABRIK_ERROR;
        }
    };

    let data_slice = std::slice::from_raw_parts(data, data_len);

    match cache.storage.put(hash_str.as_bytes(), data_slice) {
        Ok(_) => FABRIK_OK,
        Err(e) => {
            set_last_error(format!("Failed to put artifact: {}", e));
            FABRIK_ERROR
        }
    }
}

/// Check if an artifact exists in the cache
///
/// # Arguments
/// * `cache` - Cache instance
/// * `hash` - Content hash (NULL-terminated C string)
/// * `exists` - Output: 1 if exists, 0 if not
///
/// # Returns
/// * `FABRIK_OK` on success
/// * `FABRIK_ERROR` on error
///
/// # Safety
/// * All pointers must be valid
#[no_mangle]
pub unsafe extern "C" fn fabrik_cache_exists(
    cache: *mut FabrikCache,
    hash: *const c_char,
    exists: *mut c_int,
) -> c_int {
    clear_last_error();

    if cache.is_null() || hash.is_null() || exists.is_null() {
        set_last_error("NULL pointer argument");
        return FABRIK_ERROR;
    }

    let cache = &(*cache);
    let hash_str = match CStr::from_ptr(hash).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in hash: {}", e));
            return FABRIK_ERROR;
        }
    };

    match cache.storage.exists(hash_str.as_bytes()) {
        Ok(result) => {
            *exists = if result { 1 } else { 0 };
            FABRIK_OK
        }
        Err(e) => {
            set_last_error(format!("Failed to check existence: {}", e));
            FABRIK_ERROR
        }
    }
}

/// Delete an artifact from the cache
///
/// # Arguments
/// * `cache` - Cache instance
/// * `hash` - Content hash (NULL-terminated C string)
///
/// # Returns
/// * `FABRIK_OK` on success
/// * `FABRIK_ERROR` on error
///
/// # Safety
/// * All pointers must be valid
#[no_mangle]
pub unsafe extern "C" fn fabrik_cache_delete(
    cache: *mut FabrikCache,
    hash: *const c_char,
) -> c_int {
    clear_last_error();

    if cache.is_null() || hash.is_null() {
        set_last_error("NULL pointer argument");
        return FABRIK_ERROR;
    }

    let cache = &(*cache);
    let hash_str = match CStr::from_ptr(hash).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in hash: {}", e));
            return FABRIK_ERROR;
        }
    };

    match cache.storage.delete(hash_str.as_bytes()) {
        Ok(_) => FABRIK_OK,
        Err(e) => {
            set_last_error(format!("Failed to delete artifact: {}", e));
            FABRIK_ERROR
        }
    }
}

/// Get the last error message
///
/// # Returns
/// * Pointer to NULL-terminated error string
/// * NULL if no error
///
/// # Safety
/// * Returned string is valid until next API call
/// * Do not free the returned pointer
#[no_mangle]
pub extern "C" fn fabrik_last_error() -> *const c_char {
    LAST_ERROR.with(|last| {
        last.lock()
            .unwrap()
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(ptr::null())
    })
}

/// Free a string allocated by the Fabrik library
///
/// # Safety
/// * `s` must be a string allocated by a Fabrik API function
#[no_mangle]
pub unsafe extern "C" fn fabrik_free_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

/// Get the library version
///
/// # Returns
/// * Pointer to NULL-terminated version string
///
/// # Safety
/// * Returned string is statically allocated, do not free
#[no_mangle]
pub extern "C" fn fabrik_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}
