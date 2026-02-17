//! Padrón A13 FFI Functions
//!
//! Taxpayer registry query functions.

use std::os::raw::c_char;

use super::error_codes::*;
use super::handle::ArcaHandle;
use super::helpers::{clear_last_error, set_last_error, json_to_cstring_ptr};
use super::get_runtime;

/// Check Padrón A13 service status (Dummy).
///
/// # Parameters
/// - `handle`: The ARCA client handle
/// - `out_json`: Output pointer for JSON result (must be freed with `arca_free_string`)
///
/// # Returns
/// - `0` on success
/// - Non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_padron_dummy(
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
        handle.padron.dummy(&handle.http).await
    });

    match result {
        Ok(dummy) => {
            let json = serde_json::json!({
                "AppServer": dummy.app_server,
                "AuthServer": dummy.auth_server,
                "DbServer": dummy.db_server
            });
            let ptr = json_to_cstring_ptr(&json);
            if ptr.is_null() {
                return FFI_ERR_CSTRING;
            }
            unsafe { *out_json = ptr; }
            FFI_OK
        }
        Err(e) => {
            set_last_error(format!("Padron Dummy failed: {}", e));
            FFI_ERR_PADRON_DUMMY
        }
    }
}

/// Get taxpayer information by CUIT (GetPersona).
///
/// # Parameters
/// - `handle`: The ARCA client handle
/// - `cuit_representada`: The CUIT of the represented entity (your CUIT)
/// - `cuit_consulta`: The CUIT to query
/// - `out_json`: Output pointer for JSON result (must be freed with `arca_free_string`)
///
/// # Returns
/// - `0` on success
/// - Non-zero on error
///
/// # Note
/// Requires authentication with service "ws_sr_padron_a13"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn arca_padron_get_persona(
    handle: *mut ArcaHandle,
    cuit_representada: u64,
    cuit_consulta: u64,
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
            set_last_error("Not authenticated. Call arca_login with 'ws_sr_padron_a13' first.".to_string());
            return FFI_ERR_NOT_AUTHENTICATED;
        }
    };

    let result = get_runtime().block_on(async {
        handle.padron.get_persona(&handle.http, token, sign, cuit_representada, cuit_consulta).await
    });

    match result {
        Ok(persona) => {
            let json = serde_json::json!({
                "id_persona": persona.persona.id_persona,
                "tipo_persona": persona.persona.tipo_persona,
                "tipo_documento": persona.persona.tipo_documento,
                "numero_documento": persona.persona.numero_documento,
                "nombre": persona.persona.nombre,
                "apellido": persona.persona.apellido,
                "razon_social": persona.persona.razon_social,
                "estado_clave": persona.persona.estado_clave,
                "nombre_completo": persona.persona.nombre_completo(),
                "domicilio": persona.persona.domicilio_fiscal().map(|d| serde_json::json!({
                    "direccion": d.direccion,
                    "calle": d.calle,
                    "numero": d.numero,
                    "localidad": d.localidad,
                    "codigo_postal": d.codigo_postal,
                    "id_provincia": d.id_provincia,
                    "descripcion_provincia": d.descripcion_provincia
                }))
            });
            let ptr = json_to_cstring_ptr(&json);
            if ptr.is_null() {
                return FFI_ERR_CSTRING;
            }
            unsafe { *out_json = ptr; }
            FFI_OK
        }
        Err(e) => {
            set_last_error(format!("GetPersona failed: {}", e));
            FFI_ERR_PADRON_GET_PERSONA
        }
    }
}
