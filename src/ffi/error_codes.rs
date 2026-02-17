//! FFI Error Code Constants
//!
//! All error codes returned by FFI functions are defined here.
//! Negative values indicate errors, 0 indicates success.

/// Success
pub const FFI_OK: i32 = 0;

// ============================================================================
// General Errors (-1 to -9)
// ============================================================================

/// Null pointer argument
pub const FFI_ERR_NULL_POINTER: i32 = -1;
/// Invalid UTF-8 string
pub const FFI_ERR_INVALID_UTF8: i32 = -2;
/// File read error
pub const FFI_ERR_FILE_READ: i32 = -3;
/// Runtime creation error
pub const FFI_ERR_RUNTIME: i32 = -4;
/// HTTP client creation error
pub const FFI_ERR_HTTP_CLIENT: i32 = -5;
/// PFX conversion error
pub const FFI_ERR_PFX_CONVERSION: i32 = -6;

// ============================================================================
// Authentication Errors (-10 to -19)
// ============================================================================

/// CMS signing error
pub const FFI_ERR_CMS_SIGNING: i32 = -10;
/// Login failed
pub const FFI_ERR_LOGIN_FAILED: i32 = -11;
/// Not authenticated
pub const FFI_ERR_NOT_AUTHENTICATED: i32 = -12;

// ============================================================================
// WSFEv1 Errors (-20 to -39)
// ============================================================================

/// FE Dummy failed
pub const FFI_ERR_FE_DUMMY: i32 = -20;
/// FE Comp Ultimo Autorizado failed
pub const FFI_ERR_FE_COMP_ULTIMO: i32 = -21;
/// FE CAE Solicitar failed
pub const FFI_ERR_FECAE_SOLICITAR: i32 = -22;
/// FE Comp Consultar failed
pub const FFI_ERR_FE_COMP_CONSULTAR: i32 = -23;
/// FE Param Get Tipos Cbte failed
pub const FFI_ERR_FE_TIPOS_CBTE: i32 = -24;
/// FE Param Get Tipos Doc failed
pub const FFI_ERR_FE_TIPOS_DOC: i32 = -25;
/// FE Param Get Tipos IVA failed
pub const FFI_ERR_FE_TIPOS_IVA: i32 = -26;
/// FE Param Get Tipos Monedas failed
pub const FFI_ERR_FE_TIPOS_MONEDAS: i32 = -27;
/// FE Param Get Ptos Venta failed
pub const FFI_ERR_FE_PTOS_VENTA: i32 = -28;
/// FE Param Get Cotizacion failed
pub const FFI_ERR_FE_COTIZACION: i32 = -29;
/// Invalid JSON
pub const FFI_ERR_INVALID_JSON: i32 = -30;
/// Validation failed (invalid field values)
pub const FFI_ERR_VALIDATION_FAILED: i32 = -31;

// ============================================================================
// Padrón Errors (-40 to -49)
// ============================================================================

/// Padron Dummy failed
pub const FFI_ERR_PADRON_DUMMY: i32 = -40;
/// Padron Get Persona failed
pub const FFI_ERR_PADRON_GET_PERSONA: i32 = -41;

// ============================================================================
// Internal Errors (-99)
// ============================================================================

/// Internal CString conversion error
pub const FFI_ERR_CSTRING: i32 = -99;
