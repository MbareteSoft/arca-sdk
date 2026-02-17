//! ARCA Constants for FFI
//!
//! Constant functions for voucher types, document types, IVA conditions, and IVA rates.
//! These are exposed as C functions so they can be called from any language.

// ============================================================================
// Constants - Voucher Types (Tipos de Comprobante)
// ============================================================================

/// Factura A
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_CBTE_FACTURA_A() -> i32 { 1 }
/// Nota de Débito A
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_CBTE_NOTA_DEBITO_A() -> i32 { 2 }
/// Nota de Crédito A
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_CBTE_NOTA_CREDITO_A() -> i32 { 3 }
/// Factura B
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_CBTE_FACTURA_B() -> i32 { 6 }
/// Nota de Débito B
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_CBTE_NOTA_DEBITO_B() -> i32 { 7 }
/// Nota de Crédito B
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_CBTE_NOTA_CREDITO_B() -> i32 { 8 }
/// Factura C
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_CBTE_FACTURA_C() -> i32 { 11 }
/// Nota de Débito C
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_CBTE_NOTA_DEBITO_C() -> i32 { 12 }
/// Nota de Crédito C
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_CBTE_NOTA_CREDITO_C() -> i32 { 13 }

// ============================================================================
// Constants - Document Types (Tipos de Documento)
// ============================================================================

/// CUIT
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_DOC_CUIT() -> i32 { 80 }
/// CUIL
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_DOC_CUIL() -> i32 { 86 }
/// CDI
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_DOC_CDI() -> i32 { 87 }
/// DNI
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_DOC_DNI() -> i32 { 96 }
/// Pasaporte
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_DOC_PASAPORTE() -> i32 { 94 }
/// Consumidor Final (Sin identificar)
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_DOC_CONSUMIDOR_FINAL() -> i32 { 99 }

// ============================================================================
// Constants - IVA Conditions (Condiciones frente al IVA)
// ============================================================================

/// IVA Responsable Inscripto
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_IVA_RESPONSABLE_INSCRIPTO() -> i32 { 1 }
/// IVA Sujeto Exento
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_IVA_SUJETO_EXENTO() -> i32 { 4 }
/// Consumidor Final
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_IVA_CONSUMIDOR_FINAL() -> i32 { 5 }
/// Responsable Monotributo
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_IVA_RESPONSABLE_MONOTRIBUTO() -> i32 { 6 }

// ============================================================================
// Constants - IVA Rates (Alícuotas de IVA)
// ============================================================================

/// No Gravado
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_ALIC_NO_GRAVADO() -> i32 { 1 }
/// Exento
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_ALIC_EXENTO() -> i32 { 2 }
/// 0%
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_ALIC_CERO() -> i32 { 3 }
/// 10.5%
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_ALIC_DIEZ_CINCO() -> i32 { 4 }
/// 21%
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_ALIC_VEINTIUNO() -> i32 { 5 }
/// 27%
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_ALIC_VEINTISIETE() -> i32 { 6 }
/// 5%
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_ALIC_CINCO() -> i32 { 8 }
/// 2.5%
#[unsafe(no_mangle)]
pub extern "C" fn ARCA_ALIC_DOS_CINCO() -> i32 { 9 }
