//! FFI Authentication Functions
//!
//! WSAA login, authentication status, and token cache functions.

use std::ffi::CStr;
use std::os::raw::c_char;

use crate::auth;
use crate::auth::TicketAcceso;
use crate::client::token_store::{FileSystemTokenStore, TokenStore};

use super::error_codes::*;
use super::handle::ArcaHandle;
use super::helpers::{clear_last_error, set_last_error};
use super::get_runtime;

/// Enable persistent token caching to a directory.
///
/// Tokens will be saved to and loaded from JSON files in the specified directory.
/// The directory is created automatically if it does not exist.
/// Call this BEFORE `arca_login()` to enable cache-aware authentication.
///
/// Without this call, tokens are stored only in memory (original behavior).
///
/// # Parameters
/// - `handle`: The ARCA client handle
/// - `cache_dir`: Path to the directory for token cache files
///
/// # Returns
/// - `0` on success
/// - Non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_set_token_cache(
    handle: *mut ArcaHandle,
    cache_dir: *const c_char,
) -> i32 {
    clear_last_error();

    if handle.is_null() || cache_dir.is_null() {
        set_last_error("Null pointer argument".to_string());
        return FFI_ERR_NULL_POINTER;
    }

    let handle = unsafe { &mut *handle };
    let dir_str = match unsafe { CStr::from_ptr(cache_dir) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid cache directory path: {}", e));
            return FFI_ERR_INVALID_UTF8;
        }
    };

    handle.token_store = Some(FileSystemTokenStore::new(dir_str));
    FFI_OK
}

/// Authenticate with WSAA for a specific service.
///
/// Uses a 3-tier cache strategy:
/// 1. **RAM**: If a valid token exists in memory, returns immediately
/// 2. **Disk**: If token cache is enabled (via `arca_set_token_cache`), loads from disk
/// 3. **WSAA**: Contacts ARCA/AFIP to get a new token, saves to both RAM and disk
///
/// # Parameters
/// - `handle`: The ARCA client handle
/// - `service`: The service to authenticate for (e.g., "wsfe", "ws_sr_padron_a13")
///
/// # Returns
/// - `0` on success
/// - Non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_login(
    handle: *mut ArcaHandle,
    service: *const c_char,
) -> i32 {
    clear_last_error();

    if handle.is_null() || service.is_null() {
        set_last_error("Null pointer argument".to_string());
        return FFI_ERR_NULL_POINTER;
    }

    let handle = unsafe { &mut *handle };
    let service_str = match unsafe { CStr::from_ptr(service) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid service name: {}", e));
            return FFI_ERR_INVALID_UTF8;
        }
    };

    // --- TIER 1: Check in-memory token ---
    if let (Some(token), Some(sign), Some(exp)) =
        (&handle.token, &handle.sign, &handle.expiration_time)
    {
        let ta = TicketAcceso {
            token: token.clone(),
            sign: sign.clone(),
            expiration_time: exp.clone(),
        };
        if !ta.is_expired() {
            return FFI_OK;
        }
        // ta dropped here → zeroizes cloned values
    }

    // --- TIER 2: Check persistent cache ---
    if let Some(ref store) = handle.token_store {
        let load_result = get_runtime().block_on(async {
            store.load_token(service_str).await
        });

        if let Ok(Some(ta)) = load_result {
            if !ta.is_expired() {
                handle.token = Some(ta.token.clone());
                handle.sign = Some(ta.sign.clone());
                handle.expiration_time = Some(ta.expiration_time.clone());
                return FFI_OK;
            }
        }
        // Disk errors are silently ignored → fall through to WSAA
    }

    // --- TIER 3: Contact WSAA ---
    let ltr = auth::build_login_ticket_request(service_str);
    let cms = match auth::sign_cms_base64(&ltr, &handle.cert_pem, &handle.key_pem) {
        Ok(c) => c,
        Err(e) => {
            set_last_error(format!("Failed to sign CMS: {}", e));
            return FFI_ERR_CMS_SIGNING;
        }
    };

    let result = get_runtime().block_on(async {
        auth::wsaa_login(&handle.http, &handle.wsaa_url, &cms).await
    });

    match result {
        Ok(ticket) => {
            handle.token = Some(ticket.token.clone());
            handle.sign = Some(ticket.sign.clone());
            handle.expiration_time = Some(ticket.expiration_time.clone());

            // Persist to disk if cache is enabled (best-effort)
            if let Some(ref store) = handle.token_store {
                let _ = get_runtime().block_on(async {
                    store.save_token(service_str, &ticket).await
                });
            }

            FFI_OK
        }
        Err(e) => {
            set_last_error(format!("Login failed: {}", e));
            FFI_ERR_LOGIN_FAILED
        }
    }
}

/// Check if the client is authenticated.
///
/// # Parameters
/// - `handle`: The ARCA client handle
///
/// # Returns
/// - `1` if authenticated
/// - `0` if not authenticated
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_is_authenticated(handle: *const ArcaHandle) -> i32 {
    if handle.is_null() {
        return 0;
    }
    let handle = unsafe { &*handle };
    if handle.token.is_some() && handle.sign.is_some() {
        1
    } else {
        0
    }
}
