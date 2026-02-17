//! Invoice Validation Types
//!
//! Typed structs for JSON invoice validation in FFI layer.

use serde::Deserialize;
use validator::{Validate, ValidationError};

/// Validated invoice request structure for FECAESolicitar.
///
/// This struct provides type-safe JSON deserialization with validation rules
/// to catch errors before calling ARCA services.
#[derive(Debug, Deserialize, Validate)]
pub struct InvoiceRequest {
    /// Punto de venta (1-9999)
    #[validate(range(min = 1, max = 9999, message = "pto_vta debe estar entre 1 y 9999"))]
    pub pto_vta: i32,

    /// Tipo de comprobante
    #[validate(custom(function = "validate_cbte_tipo"))]
    pub cbte_tipo: i32,

    /// Concepto: 1=Productos, 2=Servicios, 3=Productos y Servicios
    #[serde(default = "default_concepto")]
    #[validate(range(min = 1, max = 3, message = "concepto debe ser 1, 2 o 3"))]
    pub concepto: i32,

    /// Tipo de documento del receptor
    pub doc_tipo: i32,

    /// Número de documento del receptor
    pub doc_nro: u64,

    /// Número de comprobante desde
    #[validate(range(min = 1, message = "cbte_desde debe ser >= 1"))]
    pub cbte_desde: u64,

    /// Número de comprobante hasta
    #[validate(range(min = 1, message = "cbte_hasta debe ser >= 1"))]
    pub cbte_hasta: u64,

    /// Fecha del comprobante (YYYYMMDD)
    #[validate(length(equal = 8, message = "cbte_fch debe tener 8 caracteres (YYYYMMDD)"))]
    pub cbte_fch: String,

    /// Importe total
    #[validate(range(min = 0.0, message = "imp_total debe ser >= 0"))]
    pub imp_total: f64,

    /// Importe total conceptos no gravados
    #[serde(default)]
    pub imp_tot_conc: f64,

    /// Importe neto gravado
    #[serde(default)]
    pub imp_neto: f64,

    /// Importe operaciones exentas
    #[serde(default)]
    pub imp_op_ex: f64,

    /// Importe tributos
    #[serde(default)]
    pub imp_trib: f64,

    /// Importe IVA
    #[serde(default)]
    pub imp_iva: f64,

    /// Fecha inicio de servicio (YYYYMMDD) - requerido para concepto 2 o 3
    pub fch_serv_desde: Option<String>,

    /// Fecha fin de servicio (YYYYMMDD) - requerido para concepto 2 o 3
    pub fch_serv_hasta: Option<String>,

    /// Fecha vencimiento de pago (YYYYMMDD) - requerido para concepto 2 o 3
    pub fch_vto_pago: Option<String>,

    /// Código de moneda (default: PES)
    #[serde(default = "default_moneda")]
    pub mon_id: String,

    /// Cotización de la moneda (default: 1.0)
    #[serde(default = "default_cotiz")]
    pub mon_cotiz: f64,

    /// Condición IVA del receptor (opcional pero recomendado)
    pub condicion_iva_receptor: Option<i32>,

    /// Alícuotas de IVA
    #[serde(default)]
    pub iva: Vec<IvaDetail>,
}

/// Detalle de alícuota IVA
#[derive(Debug, Deserialize)]
pub struct IvaDetail {
    /// ID de alícuota (3=0%, 4=10.5%, 5=21%, 6=27%, 8=5%, 9=2.5%)
    pub id: i32,
    /// Base imponible
    #[serde(default)]
    pub base_imp: f64,
    /// Importe
    #[serde(default)]
    pub importe: f64,
}

fn default_concepto() -> i32 { 1 }
fn default_moneda() -> String { "PES".to_string() }
fn default_cotiz() -> f64 { 1.0 }

/// Validates that cbte_tipo is a valid voucher type code
fn validate_cbte_tipo(cbte_tipo: i32) -> Result<(), ValidationError> {
    // Valid voucher types:
    // 1=Factura A, 2=Nota Débito A, 3=Nota Crédito A
    // 6=Factura B, 7=Nota Débito B, 8=Nota Crédito B
    // 11=Factura C, 12=Nota Débito C, 13=Nota Crédito C
    // There are more types, but these are the most common
    let valid_types = [1, 2, 3, 6, 7, 8, 11, 12, 13, 51, 52, 53, 201, 202, 203, 206, 207, 208, 211, 212, 213];
    if valid_types.contains(&cbte_tipo) {
        Ok(())
    } else {
        let mut err = ValidationError::new("cbte_tipo_invalido");
        err.message = Some(format!("cbte_tipo {} no es un tipo de comprobante válido", cbte_tipo).into());
        Err(err)
    }
}

impl InvoiceRequest {
    /// Additional business logic validation beyond field-level rules
    pub fn validate_business_rules(&self) -> Result<(), String> {
        // For services (concepto 2 or 3), service dates and payment date are required
        if self.concepto >= 2 {
            if self.fch_serv_desde.is_none() {
                return Err("fch_serv_desde es requerido para concepto de servicios".to_string());
            }
            if self.fch_serv_hasta.is_none() {
                return Err("fch_serv_hasta es requerido para concepto de servicios".to_string());
            }
            if self.fch_vto_pago.is_none() {
                return Err("fch_vto_pago es requerido para concepto de servicios".to_string());
            }
        }

        // cbte_hasta must be >= cbte_desde
        if self.cbte_hasta < self.cbte_desde {
            return Err("cbte_hasta debe ser >= cbte_desde".to_string());
        }

        Ok(())
    }
}
