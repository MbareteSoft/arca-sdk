//! FFI (Foreign Function Interface) module for C-compatible exports.
//!
//! This module exposes the ARCA SDK functionality as C-compatible functions
//! that can be called from other languages (C#, Python, Delphi, etc.)
//!
//! # Memory Management
//!
//! - All returned strings must be freed using `arca_free_string()`
//! - All returned JSON must be freed using `arca_free_string()`
//! - The client handle must be freed using `arca_client_free()`
//!
//! # Error Handling
//!
//! Functions return:
//! - `0` on success
//! - Non-zero error code on failure
//! - Use `arca_last_error()` to get the error message
//!
//! # Module Structure
//!
//! - `error_codes`: FFI error code constants
//! - `helpers`: Thread-local error handling and string conversion utilities
//! - `handle`: ArcaHandle opaque structure
//! - `invoice`: Invoice validation types
//! - `client`: Client creation and destruction functions
//! - `auth`: Authentication functions
//! - `wsfev1`: Electronic invoicing functions
//! - `params`: Parameter query functions
//! - `padron`: Taxpayer registry functions
//! - `constants`: ARCA constant values

use std::sync::OnceLock;

// ============================================================================
// Submodules
// ============================================================================

pub mod error_codes;
pub mod helpers;
pub mod handle;
pub mod invoice;
pub mod client;
pub mod auth;
pub mod wsfev1;
pub mod params;
pub mod padron;
pub mod constants;

// ============================================================================
// URL Constants - Static strings to avoid runtime allocations
// ============================================================================

// Testing/Homologación URLs
pub(crate) const WSAA_URL_TESTING: &str = "https://wsaahomo.afip.gov.ar/ws/services/LoginCms";
pub(crate) const WSFEV1_URL_TESTING: &str = "https://wswhomo.afip.gov.ar/wsfev1/service.asmx";
pub(crate) const PADRON_URL_TESTING: &str = "https://awshomo.afip.gov.ar/sr-padron/webservices/personaServiceA13";

// Production URLs
pub(crate) const WSAA_URL_PRODUCTION: &str = "https://wsaa.afip.gov.ar/ws/services/LoginCms";
pub(crate) const WSFEV1_URL_PRODUCTION: &str = "https://servicios1.afip.gov.ar/wsfev1/service.asmx";
pub(crate) const PADRON_URL_PRODUCTION: &str = "https://aws.afip.gov.ar/sr-padron/webservices/personaServiceA13";

// ============================================================================
// Global Tokio Runtime - Shared across all clients
// ============================================================================

static GLOBAL_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub(crate) fn get_runtime() -> &'static tokio::runtime::Runtime {
    GLOBAL_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create global Tokio runtime")
    })
}

// ============================================================================
// Public Re-exports for FFI
// ============================================================================

// Error codes
pub use error_codes::*;

// Handle
pub use handle::ArcaHandle;

// Invoice validation types
pub use invoice::{InvoiceRequest, IvaDetail};

// Client functions
pub use client::{
    arca_free_string,
    arca_last_error,
    arca_client_new_testing,
    arca_client_new_production,
    arca_client_new_pfx_testing,
    arca_client_new_pfx_production,
    arca_client_free,
};

// Authentication functions
pub use auth::{
    arca_login,
    arca_is_authenticated,
    arca_set_token_cache,
};

// WSFEv1 functions
pub use wsfev1::{
    arca_fe_dummy,
    arca_fe_comp_ultimo_autorizado,
    arca_fecae_solicitar,
    arca_fe_comp_consultar,
};

// Parameter functions
pub use params::{
    arca_fe_param_get_tipos_cbte,
    arca_fe_param_get_tipos_doc,
    arca_fe_param_get_tipos_iva,
    arca_fe_param_get_tipos_monedas,
    arca_fe_param_get_ptos_venta,
    arca_fe_param_get_cotizacion,
};

// Padron functions
pub use padron::{
    arca_padron_dummy,
    arca_padron_get_persona,
};

// Constant functions
pub use constants::{
    // Voucher types
    ARCA_CBTE_FACTURA_A,
    ARCA_CBTE_NOTA_DEBITO_A,
    ARCA_CBTE_NOTA_CREDITO_A,
    ARCA_CBTE_FACTURA_B,
    ARCA_CBTE_NOTA_DEBITO_B,
    ARCA_CBTE_NOTA_CREDITO_B,
    ARCA_CBTE_FACTURA_C,
    ARCA_CBTE_NOTA_DEBITO_C,
    ARCA_CBTE_NOTA_CREDITO_C,
    // Document types
    ARCA_DOC_CUIT,
    ARCA_DOC_CUIL,
    ARCA_DOC_CDI,
    ARCA_DOC_DNI,
    ARCA_DOC_PASAPORTE,
    ARCA_DOC_CONSUMIDOR_FINAL,
    // IVA conditions
    ARCA_IVA_RESPONSABLE_INSCRIPTO,
    ARCA_IVA_SUJETO_EXENTO,
    ARCA_IVA_CONSUMIDOR_FINAL,
    ARCA_IVA_RESPONSABLE_MONOTRIBUTO,
    // IVA rates
    ARCA_ALIC_NO_GRAVADO,
    ARCA_ALIC_EXENTO,
    ARCA_ALIC_CERO,
    ARCA_ALIC_DIEZ_CINCO,
    ARCA_ALIC_VEINTIUNO,
    ARCA_ALIC_VEINTISIETE,
    ARCA_ALIC_CINCO,
    ARCA_ALIC_DOS_CINCO,
};
