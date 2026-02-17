//! WSFEv1 FFI Functions
//!
//! Electronic invoicing functions (FEDummy, FECompUltimoAutorizado, FECAESolicitar, FECompConsultar).

use std::ffi::CStr;
use std::os::raw::c_char;

use validator::Validate;

use crate::{FeCabReq, FeDetReq, AlicIva};

use super::error_codes::*;
use super::handle::ArcaHandle;
use super::helpers::{clear_last_error, set_last_error, json_to_cstring_ptr};
use super::invoice::InvoiceRequest;
use super::get_runtime;

/// Check WSFEv1 service status (FEDummy).
///
/// # Parameters
/// - `handle`: The ARCA client handle
/// - `out_json`: Output pointer for JSON result (must be freed with `arca_free_string`)
///
/// # Returns
/// - `0` on success
/// - Non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_fe_dummy(
    handle: *mut ArcaHandle,
    out_json: *mut *mut c_char,
) -> i32 {
    clear_last_error();

    if handle.is_null() || out_json.is_null() {
        set_last_error("Null pointer argument".to_string());
        return FFI_ERR_NULL_POINTER;
    }

    let handle = unsafe { &*handle };

    let result = get_runtime().block_on(async {
        handle.wsfev1.fe_dummy().await
    });

    match result {
        Ok((app, auth, db)) => {
            let json = serde_json::json!({
                "AppServer": app,
                "AuthServer": auth,
                "DbServer": db
            });
            let ptr = json_to_cstring_ptr(&json);
            if ptr.is_null() {
                return FFI_ERR_CSTRING;
            }
            unsafe { *out_json = ptr; }
            FFI_OK
        }
        Err(e) => {
            set_last_error(format!("FEDummy failed: {}", e));
            FFI_ERR_FE_DUMMY
        }
    }
}

/// Get the last authorized voucher number (FECompUltimoAutorizado).
///
/// # Parameters
/// - `handle`: The ARCA client handle
/// - `cuit`: The CUIT number
/// - `pto_vta`: Point of sale number
/// - `cbte_tipo`: Voucher type (see cbte_tipos constants)
/// - `out_numero`: Output pointer for the voucher number
///
/// # Returns
/// - `0` on success
/// - Non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_fe_comp_ultimo_autorizado(
    handle: *mut ArcaHandle,
    cuit: u64,
    pto_vta: i32,
    cbte_tipo: i32,
    out_numero: *mut i64,
) -> i32 {
    clear_last_error();

    if handle.is_null() || out_numero.is_null() {
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
        handle.wsfev1.fe_comp_ultimo_autorizado(token, sign, cuit, pto_vta, cbte_tipo).await
    });

    match result {
        Ok(n) => {
            unsafe { *out_numero = n as i64; }
            FFI_OK
        }
        Err(e) => {
            set_last_error(format!("FECompUltimoAutorizado failed: {}", e));
            FFI_ERR_FE_COMP_ULTIMO
        }
    }
}

/// Request CAE for an invoice (FECAESolicitar).
///
/// # Parameters
/// - `handle`: The ARCA client handle
/// - `cuit`: The CUIT number
/// - `invoice_json`: JSON string with invoice data
/// - `out_json`: Output pointer for JSON result (must be freed with `arca_free_string`)
///
/// # Returns
/// - `0` on success
/// - Non-zero on error
///
/// # Invoice JSON Format
/// ```json
/// {
///     "pto_vta": 1,
///     "cbte_tipo": 11,
///     "concepto": 1,
///     "doc_tipo": 99,
///     "doc_nro": 0,
///     "cbte_desde": 1,
///     "cbte_hasta": 1,
///     "cbte_fch": "20251226",
///     "imp_total": 1000.00,
///     "imp_tot_conc": 0.00,
///     "imp_neto": 1000.00,
///     "imp_op_ex": 0.00,
///     "imp_trib": 0.00,
///     "imp_iva": 0.00,
///     "mon_id": "PES",
///     "mon_cotiz": 1.0,
///     "condicion_iva_receptor": 5,
///     "iva": []
/// }
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_fecae_solicitar(
    handle: *mut ArcaHandle,
    cuit: u64,
    invoice_json: *const c_char,
    out_json: *mut *mut c_char,
) -> i32 {
    clear_last_error();

    if handle.is_null() || invoice_json.is_null() || out_json.is_null() {
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

    let json_str = match unsafe { CStr::from_ptr(invoice_json) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid JSON string: {}", e));
            return FFI_ERR_INVALID_UTF8;
        }
    };

    // Parse JSON into typed invoice structure with serde
    let invoice: InvoiceRequest = match serde_json::from_str(json_str) {
        Ok(inv) => inv,
        Err(e) => {
            set_last_error(format!("JSON parse error: {}", e));
            return FFI_ERR_INVALID_JSON;
        }
    };

    // Validate field-level rules
    if let Err(validation_errors) = invoice.validate() {
        let errors: Vec<String> = validation_errors
            .field_errors()
            .iter()
            .map(|(field, errs)| {
                let messages: Vec<String> = errs.iter()
                    .map(|e| e.message.as_ref().map(|m| m.to_string()).unwrap_or_else(|| e.code.to_string()))
                    .collect();
                format!("{}: {}", field, messages.join(", "))
            })
            .collect();
        set_last_error(format!("Validation failed: {}", errors.join("; ")));
        return FFI_ERR_VALIDATION_FAILED;
    }

    // Validate business rules
    if let Err(business_err) = invoice.validate_business_rules() {
        set_last_error(format!("Validation failed: {}", business_err));
        return FFI_ERR_VALIDATION_FAILED;
    }

    // Build request from validated invoice
    let cab = FeCabReq {
        cant_reg: 1,
        pto_vta: invoice.pto_vta,
        cbte_tipo: invoice.cbte_tipo,
    };

    let det = FeDetReq {
        concepto: invoice.concepto,
        doc_tipo: invoice.doc_tipo,
        doc_nro: invoice.doc_nro,
        cbte_desde: invoice.cbte_desde,
        cbte_hasta: invoice.cbte_hasta,
        cbte_fch: invoice.cbte_fch,
        imp_total: invoice.imp_total,
        imp_tot_conc: invoice.imp_tot_conc,
        imp_neto: invoice.imp_neto,
        imp_op_ex: invoice.imp_op_ex,
        imp_trib: invoice.imp_trib,
        imp_iva: invoice.imp_iva,
        fch_serv_desde: invoice.fch_serv_desde,
        fch_serv_hasta: invoice.fch_serv_hasta,
        fch_vto_pago: invoice.fch_vto_pago,
        mon_id: invoice.mon_id,
        mon_cotiz: invoice.mon_cotiz,
        condicion_iva_receptor: invoice.condicion_iva_receptor,
        iva: invoice.iva.into_iter().map(|i| AlicIva {
            id: i.id,
            base_imp: i.base_imp,
            importe: i.importe,
        }).collect(),
    };

    let result = get_runtime().block_on(async {
        handle.wsfev1.fecae_solicitar_detallado(token, sign, cuit, cab, vec![det]).await
    });

    match result {
        Ok(resp) => {
            let json = serde_json::json!({
                "resultado": resp.resultado,
                "cae": resp.cae,
                "cae_fch_vto": resp.cae_fch_vto,
                "cbte_desde": resp.cbte_desde,
                "cbte_hasta": resp.cbte_hasta,
                "errors": resp.detalles.errors.iter().map(|e| serde_json::json!({
                    "code": e.code,
                    "msg": e.msg
                })).collect::<Vec<_>>(),
                "observaciones": resp.detalles.observaciones.iter().map(|o| serde_json::json!({
                    "code": o.code,
                    "msg": o.msg
                })).collect::<Vec<_>>(),
                "events": resp.detalles.events.iter().map(|e| serde_json::json!({
                    "code": e.code,
                    "msg": e.msg
                })).collect::<Vec<_>>()
            });
            let ptr = json_to_cstring_ptr(&json);
            if ptr.is_null() {
                return FFI_ERR_CSTRING;
            }
            unsafe { *out_json = ptr; }
            FFI_OK
        }
        Err(e) => {
            set_last_error(format!("FECAESolicitar failed: {}", e));
            FFI_ERR_FECAE_SOLICITAR
        }
    }
}

/// Consult an issued voucher (FECompConsultar).
///
/// # Parameters
/// - `handle`: The ARCA client handle
/// - `cuit`: The CUIT number
/// - `cbte_tipo`: Voucher type
/// - `cbte_nro`: Voucher number
/// - `pto_vta`: Point of sale number
/// - `out_json`: Output pointer for JSON result (must be freed with `arca_free_string`)
///
/// # Returns
/// - `0` on success
/// - Non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_fe_comp_consultar(
    handle: *mut ArcaHandle,
    cuit: u64,
    cbte_tipo: i32,
    cbte_nro: u64,
    pto_vta: i32,
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
        handle.wsfev1.fe_comp_consultar(token, sign, cuit, cbte_tipo, cbte_nro, pto_vta).await
    });

    match result {
        Ok(comp) => {
            let json = serde_json::json!({
                "resultado": comp.resultado,
                "cod_autorizacion": comp.cod_autorizacion,
                "fch_vto": comp.fch_vto,
                "fch_proceso": comp.fch_proceso,
                "cbte_desde": comp.cbte_desde,
                "cbte_hasta": comp.cbte_hasta,
                "cbte_fch": comp.cbte_fch,
                "imp_total": comp.imp_total,
                "imp_neto": comp.imp_neto,
                "imp_iva": comp.imp_iva,
                "imp_trib": comp.imp_trib,
                "imp_op_ex": comp.imp_op_ex,
                "imp_tot_conc": comp.imp_tot_conc,
                "doc_tipo": comp.doc_tipo,
                "doc_nro": comp.doc_nro,
                "concepto": comp.concepto,
                "mon_id": comp.mon_id,
                "mon_cotiz": comp.mon_cotiz
            });
            let ptr = json_to_cstring_ptr(&json);
            if ptr.is_null() {
                return FFI_ERR_CSTRING;
            }
            unsafe { *out_json = ptr; }
            FFI_OK
        }
        Err(e) => {
            set_last_error(format!("FECompConsultar failed: {}", e));
            FFI_ERR_FE_COMP_CONSULTAR
        }
    }
}
