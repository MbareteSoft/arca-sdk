//! WSFEv1 Parameter Query FFI Functions
//!
//! Functions for querying ARCA parameter tables (types, currencies, etc.).

use std::ffi::CStr;
use std::os::raw::c_char;

use super::error_codes::*;
use super::handle::ArcaHandle;
use super::helpers::{clear_last_error, set_last_error, serialize_to_cstring_ptr};
use super::get_runtime;

// ============================================================================
// Helper macros for WSFEv1 param methods that return JSON arrays
// ============================================================================

/// Macro for FFI param functions that return Vec<ParamTipo> as JSON.
/// Handles: null checks, authentication, service creation, async runtime, error handling.
macro_rules! impl_fe_param_tipos {
    ($fn_name:ident, $method:ident, $err_code:expr, $err_msg:literal) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $fn_name(
            handle: *mut ArcaHandle,
            cuit: u64,
            out_json: *mut *mut c_char,
        ) -> i32 {
            clear_last_error();

            if handle.is_null() || out_json.is_null() {
                set_last_error("Null pointer argument".to_string());
                return FFI_ERR_NULL_POINTER;
            }

            let handle = unsafe { &*handle };

            let (token, sign) = match (&handle.token, &handle.sign) {
                (Some(t), Some(s)) => (t.as_str(), s.as_str()),
                _ => {
                    set_last_error("Not authenticated. Call arca_login first.".to_string());
                    return FFI_ERR_NOT_AUTHENTICATED;
                }
            };

            let result = get_runtime().block_on(async {
                handle.wsfev1.$method(token, sign, cuit).await
            });

            match result {
                Ok(tipos) => {
                    let json: Vec<_> = tipos.iter().map(|t| serde_json::json!({
                        "id": t.id,
                        "desc": t.desc,
                        "fch_desde": t.fch_desde,
                        "fch_hasta": t.fch_hasta
                    })).collect();
                    let ptr = serialize_to_cstring_ptr(&json);
                    if ptr.is_null() {
                        return FFI_ERR_CSTRING;
                    }
                    unsafe { *out_json = ptr; }
                    FFI_OK
                }
                Err(e) => {
                    set_last_error(format!("{}: {}", $err_msg, e));
                    $err_code
                }
            }
        }
    };
}

/// Macro for FFI param functions that return Vec<PtoVenta> as JSON.
macro_rules! impl_fe_param_ptos_venta {
    ($fn_name:ident, $method:ident, $err_code:expr, $err_msg:literal) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $fn_name(
            handle: *mut ArcaHandle,
            cuit: u64,
            out_json: *mut *mut c_char,
        ) -> i32 {
            clear_last_error();

            if handle.is_null() || out_json.is_null() {
                set_last_error("Null pointer argument".to_string());
                return FFI_ERR_NULL_POINTER;
            }

            let handle = unsafe { &*handle };

            let (token, sign) = match (&handle.token, &handle.sign) {
                (Some(t), Some(s)) => (t.as_str(), s.as_str()),
                _ => {
                    set_last_error("Not authenticated. Call arca_login first.".to_string());
                    return FFI_ERR_NOT_AUTHENTICATED;
                }
            };

            let result = get_runtime().block_on(async {
                handle.wsfev1.$method(token, sign, cuit).await
            });

            match result {
                Ok(ptos) => {
                    let json: Vec<_> = ptos.iter().map(|p| serde_json::json!({
                        "nro": p.nro,
                        "emision_tipo": p.emision_tipo,
                        "bloqueado": p.bloqueado,
                        "fch_baja": p.fch_baja
                    })).collect();
                    let ptr = serialize_to_cstring_ptr(&json);
                    if ptr.is_null() {
                        return FFI_ERR_CSTRING;
                    }
                    unsafe { *out_json = ptr; }
                    FFI_OK
                }
                Err(e) => {
                    set_last_error(format!("{}: {}", $err_msg, e));
                    $err_code
                }
            }
        }
    };
}

// FEParamGetTiposCbte - Get available voucher types
impl_fe_param_tipos!(arca_fe_param_get_tipos_cbte, fe_param_get_tipos_cbte, FFI_ERR_FE_TIPOS_CBTE, "FEParamGetTiposCbte failed");

// FEParamGetTiposDoc - Get available document types
impl_fe_param_tipos!(arca_fe_param_get_tipos_doc, fe_param_get_tipos_doc, FFI_ERR_FE_TIPOS_DOC, "FEParamGetTiposDoc failed");

// FEParamGetTiposIva - Get available IVA rates
impl_fe_param_tipos!(arca_fe_param_get_tipos_iva, fe_param_get_tipos_iva, FFI_ERR_FE_TIPOS_IVA, "FEParamGetTiposIva failed");

// FEParamGetTiposMonedas - Get available currencies
impl_fe_param_tipos!(arca_fe_param_get_tipos_monedas, fe_param_get_tipos_monedas, FFI_ERR_FE_TIPOS_MONEDAS, "FEParamGetTiposMonedas failed");

// FEParamGetPtosVenta - Get points of sale
impl_fe_param_ptos_venta!(arca_fe_param_get_ptos_venta, fe_param_get_ptos_venta, FFI_ERR_FE_PTOS_VENTA, "FEParamGetPtosVenta failed");

/// Get currency exchange rate (FEParamGetCotizacion).
///
/// # Parameters
/// - `handle`: The ARCA client handle
/// - `cuit`: The CUIT number
/// - `mon_id`: Currency ID (e.g., "DOL" for USD, "PES" for ARS)
/// - `out_cotizacion`: Output pointer for the exchange rate
///
/// # Returns
/// - `0` on success
/// - Non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_fe_param_get_cotizacion(
    handle: *mut ArcaHandle,
    cuit: u64,
    mon_id: *const c_char,
    out_cotizacion: *mut f64,
) -> i32 {
    clear_last_error();

    if handle.is_null() || mon_id.is_null() || out_cotizacion.is_null() {
        set_last_error("Null pointer argument".to_string());
        return FFI_ERR_NULL_POINTER;
    }

    let handle = unsafe { &*handle };

    let (token, sign) = match (&handle.token, &handle.sign) {
        (Some(t), Some(s)) => (t.as_str(), s.as_str()),
        _ => {
            set_last_error("Not authenticated. Call arca_login first.".to_string());
            return FFI_ERR_NOT_AUTHENTICATED;
        }
    };

    let mon_id_str = match unsafe { CStr::from_ptr(mon_id) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid currency ID: {}", e));
            return FFI_ERR_INVALID_UTF8;
        }
    };

    let result = get_runtime().block_on(async {
        handle.wsfev1.fe_param_get_cotizacion(token, sign, cuit, mon_id_str).await
    });

    match result {
        Ok(cot) => {
            unsafe { *out_cotizacion = cot.mon_cotiz; }
            FFI_OK
        }
        Err(e) => {
            set_last_error(format!("FEParamGetCotizacion failed: {}", e));
            FFI_ERR_FE_COTIZACION
        }
    }
}
