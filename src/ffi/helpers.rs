//! FFI Helper Functions
//!
//! Thread-local error handling and string conversion utilities.

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use std::sync::Mutex;

// ============================================================================
// Thread-local Error Handling
// ============================================================================

thread_local! {
    static LAST_ERROR: Mutex<Option<String>> = const { Mutex::new(None) };
}

pub fn set_last_error(err: String) {
    LAST_ERROR.with(|e| {
        // Handle poisoned mutex gracefully - clear poison and set error
        let mut guard = e.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = Some(err);
    });
}

pub fn clear_last_error() {
    LAST_ERROR.with(|e| {
        // Handle poisoned mutex gracefully
        let mut guard = e.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = None;
    });
}

pub fn get_last_error() -> Option<String> {
    LAST_ERROR.with(|e| {
        let guard = e.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.clone()
    })
}

// ============================================================================
// String Conversion Helpers
// ============================================================================

/// Safely convert a Rust string to a C string pointer.
/// Returns null_mut() if conversion fails (sets last error).
pub fn string_to_cstring_ptr(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(cstr) => cstr.into_raw(),
        Err(e) => {
            set_last_error(format!("CString conversion failed (null byte in string): {}", e));
            ptr::null_mut()
        }
    }
}

/// Safely convert JSON value to a C string pointer.
/// Returns null_mut() if conversion fails.
pub fn json_to_cstring_ptr(value: &serde_json::Value) -> *mut c_char {
    string_to_cstring_ptr(&value.to_string())
}

/// Safely serialize to JSON and convert to C string pointer.
pub fn serialize_to_cstring_ptr<T: serde::Serialize>(value: &T) -> *mut c_char {
    match serde_json::to_string(value) {
        Ok(s) => string_to_cstring_ptr(&s),
        Err(e) => {
            set_last_error(format!("JSON serialization failed: {}", e));
            ptr::null_mut()
        }
    }
}
