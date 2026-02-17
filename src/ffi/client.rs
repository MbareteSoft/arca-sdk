//! FFI Client Creation Functions
//!
//! Functions for creating and destroying ARCA client handles.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::services::{PadronA13Service, WsFev1Service};
use crate::transport::HttpClient;

use super::error_codes::*;
use super::handle::ArcaHandle;
use super::helpers::{clear_last_error, set_last_error};
use super::{get_runtime, WSAA_URL_TESTING, WSFEV1_URL_TESTING, PADRON_URL_TESTING,
            WSAA_URL_PRODUCTION, WSFEV1_URL_PRODUCTION, PADRON_URL_PRODUCTION};

// ============================================================================
// Memory Management
// ============================================================================

/// Free a string returned by ARCA functions.
///
/// # Safety
/// The pointer must have been returned by an ARCA function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}

/// Get the last error message.
///
/// Returns NULL if no error occurred.
/// The returned string must be freed with `arca_free_string()`.
#[unsafe(no_mangle)]
pub extern "C" fn arca_last_error() -> *mut c_char {
    use super::helpers::{get_last_error, string_to_cstring_ptr};
    use std::ptr;

    match get_last_error() {
        Some(err) => string_to_cstring_ptr(&err),
        None => ptr::null_mut(),
    }
}

// ============================================================================
// Client Creation - Testing Environment
// ============================================================================

/// Create a new ARCA client for the testing (homologación) environment.
///
/// # Parameters
/// - `cert_pem`: Path to the certificate PEM file
/// - `key_pem`: Path to the private key PEM file
/// - `out_handle`: Output pointer for the client handle
///
/// # Returns
/// - `0` on success
/// - Non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_client_new_testing(
    cert_pem_path: *const c_char,
    key_pem_path: *const c_char,
    out_handle: *mut *mut ArcaHandle,
) -> i32 {
    clear_last_error();

    if cert_pem_path.is_null() || key_pem_path.is_null() || out_handle.is_null() {
        set_last_error("Null pointer argument".to_string());
        return FFI_ERR_NULL_POINTER;
    }

    // Initialize global runtime
    let _ = get_runtime();

    let cert_path = match unsafe { CStr::from_ptr(cert_pem_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid cert path: {}", e));
            return FFI_ERR_INVALID_UTF8;
        }
    };

    let key_path = match unsafe { CStr::from_ptr(key_pem_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid key path: {}", e));
            return FFI_ERR_INVALID_UTF8;
        }
    };

    let cert_pem = match std::fs::read(cert_path) {
        Ok(data) => data,
        Err(e) => {
            set_last_error(format!("Failed to read certificate: {}", e));
            return FFI_ERR_FILE_READ;
        }
    };

    let key_pem = match std::fs::read(key_path) {
        Ok(data) => data,
        Err(e) => {
            set_last_error(format!("Failed to read private key: {}", e));
            return FFI_ERR_FILE_READ;
        }
    };

    let http = match HttpClient::new() {
        Ok(h) => h,
        Err(e) => {
            set_last_error(format!("Failed to create HTTP client: {}", e));
            return FFI_ERR_HTTP_CLIENT;
        }
    };

    // Create cached services (URL allocation happens once here)
    let wsfev1 = WsFev1Service::new(WSFEV1_URL_TESTING.to_string(), http.clone());
    let padron = PadronA13Service::with_endpoint(PADRON_URL_TESTING);

    let handle = Box::new(ArcaHandle {
        http,
        wsaa_url: WSAA_URL_TESTING,
        cert_pem,
        key_pem,
        token: None,
        sign: None,
        expiration_time: None,
        wsfev1,
        padron,
        token_store: None,
    });

    unsafe { *out_handle = Box::into_raw(handle); }
    FFI_OK
}

// ============================================================================
// Client Creation - Production Environment
// ============================================================================

/// Create a new ARCA client for the production environment.
///
/// # Parameters
/// - `cert_pem`: Path to the certificate PEM file
/// - `key_pem`: Path to the private key PEM file
/// - `out_handle`: Output pointer for the client handle
///
/// # Returns
/// - `0` on success
/// - Non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_client_new_production(
    cert_pem_path: *const c_char,
    key_pem_path: *const c_char,
    out_handle: *mut *mut ArcaHandle,
) -> i32 {
    clear_last_error();

    if cert_pem_path.is_null() || key_pem_path.is_null() || out_handle.is_null() {
        set_last_error("Null pointer argument".to_string());
        return FFI_ERR_NULL_POINTER;
    }

    let _ = get_runtime();

    let cert_path = match unsafe { CStr::from_ptr(cert_pem_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid cert path: {}", e));
            return FFI_ERR_INVALID_UTF8;
        }
    };

    let key_path = match unsafe { CStr::from_ptr(key_pem_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid key path: {}", e));
            return FFI_ERR_INVALID_UTF8;
        }
    };

    let cert_pem = match std::fs::read(cert_path) {
        Ok(data) => data,
        Err(e) => {
            set_last_error(format!("Failed to read certificate: {}", e));
            return FFI_ERR_FILE_READ;
        }
    };

    let key_pem = match std::fs::read(key_path) {
        Ok(data) => data,
        Err(e) => {
            set_last_error(format!("Failed to read private key: {}", e));
            return FFI_ERR_FILE_READ;
        }
    };

    let http = match HttpClient::new() {
        Ok(h) => h,
        Err(e) => {
            set_last_error(format!("Failed to create HTTP client: {}", e));
            return FFI_ERR_HTTP_CLIENT;
        }
    };

    // Create cached services (URL allocation happens once here)
    let wsfev1 = WsFev1Service::new(WSFEV1_URL_PRODUCTION.to_string(), http.clone());
    let padron = PadronA13Service::with_endpoint(PADRON_URL_PRODUCTION);

    let handle = Box::new(ArcaHandle {
        http,
        wsaa_url: WSAA_URL_PRODUCTION,
        cert_pem,
        key_pem,
        token: None,
        sign: None,
        expiration_time: None,
        wsfev1,
        padron,
        token_store: None,
    });

    unsafe { *out_handle = Box::into_raw(handle); }
    FFI_OK
}

// ============================================================================
// Client Creation - PFX Format
// ============================================================================

/// Create a new ARCA client from PFX/PKCS#12 certificate for testing environment.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_client_new_pfx_testing(
    pfx_path: *const c_char,
    pfx_password: *const c_char,
    out_handle: *mut *mut ArcaHandle,
) -> i32 {
    unsafe { arca_client_new_pfx_internal(pfx_path, pfx_password, out_handle, false) }
}

/// Create a new ARCA client from PFX/PKCS#12 certificate for production environment.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_client_new_pfx_production(
    pfx_path: *const c_char,
    pfx_password: *const c_char,
    out_handle: *mut *mut ArcaHandle,
) -> i32 {
    unsafe { arca_client_new_pfx_internal(pfx_path, pfx_password, out_handle, true) }
}

unsafe fn arca_client_new_pfx_internal(
    pfx_path: *const c_char,
    pfx_password: *const c_char,
    out_handle: *mut *mut ArcaHandle,
    production: bool,
) -> i32 {
    clear_last_error();

    if pfx_path.is_null() || pfx_password.is_null() || out_handle.is_null() {
        set_last_error("Null pointer argument".to_string());
        return FFI_ERR_NULL_POINTER;
    }

    let _ = get_runtime();

    let path = match unsafe { CStr::from_ptr(pfx_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid PFX path: {}", e));
            return FFI_ERR_INVALID_UTF8;
        }
    };

    let password = match unsafe { CStr::from_ptr(pfx_password) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid password: {}", e));
            return FFI_ERR_INVALID_UTF8;
        }
    };

    let pfx_data = match std::fs::read(path) {
        Ok(data) => data,
        Err(e) => {
            set_last_error(format!("Failed to read PFX file: {}", e));
            return FFI_ERR_FILE_READ;
        }
    };

    // Convert PFX to PEM using OpenSSL CLI
    let (cert_pem, key_pem) = match convert_pfx_to_pem(&pfx_data, password) {
        Ok((c, k)) => (c, k),
        Err(e) => {
            set_last_error(format!("Failed to convert PFX: {}", e));
            return FFI_ERR_PFX_CONVERSION;
        }
    };

    let http = match HttpClient::new() {
        Ok(h) => h,
        Err(e) => {
            set_last_error(format!("Failed to create HTTP client: {}", e));
            return FFI_ERR_HTTP_CLIENT;
        }
    };

    let (wsaa_url, wsfev1_url, padron_url) = if production {
        (WSAA_URL_PRODUCTION, WSFEV1_URL_PRODUCTION, PADRON_URL_PRODUCTION)
    } else {
        (WSAA_URL_TESTING, WSFEV1_URL_TESTING, PADRON_URL_TESTING)
    };

    // Create cached services (URL allocation happens once here)
    let wsfev1 = WsFev1Service::new(wsfev1_url.to_string(), http.clone());
    let padron = PadronA13Service::with_endpoint(padron_url);

    let handle = Box::new(ArcaHandle {
        http,
        wsaa_url,
        cert_pem,
        key_pem,
        token: None,
        sign: None,
        expiration_time: None,
        wsfev1,
        padron,
        token_store: None,
    });

    unsafe { *out_handle = Box::into_raw(handle); }
    FFI_OK
}

/// Convert PFX/PKCS#12 to PEM format using OpenSSL CLI.
/// Password is passed via stdin to avoid exposure in process list.
fn convert_pfx_to_pem(pfx_data: &[u8], password: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use tempfile::NamedTempFile;

    let mut pfx_file = NamedTempFile::new().map_err(|e| e.to_string())?;
    pfx_file.write_all(pfx_data).map_err(|e| e.to_string())?;
    let pfx_path = pfx_file
        .path()
        .to_str()
        .ok_or_else(|| "PFX temp file path contains invalid UTF-8".to_string())?;

    // Extract certificate
    let mut cert_child = Command::new("openssl")
        .args([
            "pkcs12",
            "-in", pfx_path,
            "-clcerts",
            "-nokeys",
            "-passin", "stdin",
            "-provider", "legacy",
            "-provider", "default",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn openssl: {}", e))?;

    if let Some(mut stdin) = cert_child.stdin.take() {
        // Write password followed by newline - openssl needs newline to know password is complete
        stdin.write_all(password.as_bytes())
            .map_err(|e| format!("Failed to write password: {}", e))?;
        stdin.write_all(b"\n")
            .map_err(|e| format!("Failed to write newline: {}", e))?;
    }

    let cert_output = cert_child.wait_with_output()
        .map_err(|e| format!("Failed to wait for openssl: {}", e))?;

    if !cert_output.status.success() {
        return Err(format!(
            "OpenSSL cert extraction failed: {}",
            String::from_utf8_lossy(&cert_output.stderr)
        ));
    }

    // Extract private key
    let mut key_child = Command::new("openssl")
        .args([
            "pkcs12",
            "-in", pfx_path,
            "-nocerts",
            "-nodes",
            "-passin", "stdin",
            "-provider", "legacy",
            "-provider", "default",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn openssl: {}", e))?;

    if let Some(mut stdin) = key_child.stdin.take() {
        // Write password followed by newline - openssl needs newline to know password is complete
        stdin.write_all(password.as_bytes())
            .map_err(|e| format!("Failed to write password: {}", e))?;
        stdin.write_all(b"\n")
            .map_err(|e| format!("Failed to write newline: {}", e))?;
    }

    let key_output = key_child.wait_with_output()
        .map_err(|e| format!("Failed to wait for openssl: {}", e))?;

    if !key_output.status.success() {
        return Err(format!(
            "OpenSSL key extraction failed: {}",
            String::from_utf8_lossy(&key_output.stderr)
        ));
    }

    Ok((cert_output.stdout, key_output.stdout))
}

// ============================================================================
// Client Destruction
// ============================================================================

/// Free the ARCA client handle.
///
/// # Safety
/// The handle must have been created by one of the `afip_client_new_*` functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_client_free(handle: *mut ArcaHandle) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle)); }
    }
}
