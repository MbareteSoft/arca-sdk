// ========== AFIP Constants ==========

/// Concept types (Tipos de Concepto)
pub mod conceptos {
    pub const PRODUCTOS: i32 = 1;
    pub const SERVICIOS: i32 = 2;
    pub const PRODUCTOS_Y_SERVICIOS: i32 = 3;
}

/// Document types (Tipos de Documento)
pub mod doc_tipos {
    pub const CUIT: i32 = 80;
    pub const CUIL: i32 = 86;
    pub const CDI: i32 = 87;
    pub const DNI: i32 = 96;
    pub const PASAPORTE: i32 = 94;
    pub const CONSUMIDOR_FINAL: i32 = 99; // Sin identificar
}

/// Receiver IVA conditions (Condición IVA Receptor - RG 5616)
///
/// These codes identify the IVA condition of the invoice receiver.
/// Each condition determines which invoice types (A, B, C) can be issued.
pub mod condicion_iva {
    /// IVA Responsable Inscripto - Can receive: A/M, C
    pub const IVA_RESPONSABLE_INSCRIPTO: i32 = 1;
    /// IVA Sujeto Exento - Can receive: B, C
    pub const IVA_SUJETO_EXENTO: i32 = 4;
    /// Consumidor Final - Can receive: B, C, Comprobante 49
    pub const CONSUMIDOR_FINAL: i32 = 5;
    /// Responsable Monotributo - Can receive: A/M, C
    pub const RESPONSABLE_MONOTRIBUTO: i32 = 6;
    /// Sujeto No Categorizado - Can receive: B, C
    pub const SUJETO_NO_CATEGORIZADO: i32 = 7;
    /// Proveedor del Exterior - Can receive: B, C
    pub const PROVEEDOR_DEL_EXTERIOR: i32 = 8;
    /// Cliente del Exterior - Can receive: B, C
    pub const CLIENTE_DEL_EXTERIOR: i32 = 9;
    /// IVA Liberado - Ley N° 19.640 - Can receive: B, C
    pub const IVA_LIBERADO_LEY_19640: i32 = 10;
    /// Monotributista Social - Can receive: A/M, C
    pub const MONOTRIBUTISTA_SOCIAL: i32 = 13;
    /// IVA No Alcanzado - Can receive: B, C
    pub const IVA_NO_ALCANZADO: i32 = 15;
    /// Monotributo Trabajador Independiente Promovido - Can receive: A/M, C
    pub const MONOTRIBUTO_TRABAJADOR_INDEPENDIENTE: i32 = 16;

    /// Returns the description for a given condition code
    pub fn descripcion(codigo: i32) -> &'static str {
        match codigo {
            1 => "IVA Responsable Inscripto",
            4 => "IVA Sujeto Exento",
            5 => "Consumidor Final",
            6 => "Responsable Monotributo",
            7 => "Sujeto No Categorizado",
            8 => "Proveedor del Exterior",
            9 => "Cliente del Exterior",
            10 => "IVA Liberado – Ley N° 19.640",
            13 => "Monotributista Social",
            15 => "IVA No Alcanzado",
            16 => "Monotributo Trabajador Independiente Promovido",
            _ => "Desconocido",
        }
    }

    /// Returns true if the receiver can receive invoice type A or M
    pub fn puede_recibir_a_m(codigo: i32) -> bool {
        matches!(codigo, 1 | 6 | 13 | 16)
    }

    /// Returns true if the receiver can receive invoice type B
    pub fn puede_recibir_b(codigo: i32) -> bool {
        matches!(codigo, 4 | 5 | 7 | 8 | 9 | 10 | 15)
    }

    /// Returns true if the receiver can receive invoice type C
    pub fn puede_recibir_c(codigo: i32) -> bool {
        // All conditions can receive C
        matches!(codigo, 1 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 13 | 15 | 16)
    }

    /// Returns true if the receiver can receive invoice type 49 (Comprobante de Compra de Bienes Usados)
    pub fn puede_recibir_49(codigo: i32) -> bool {
        codigo == 5 // Only Consumidor Final
    }

    /// Validates if a receiver condition is compatible with a given invoice type
    /// Returns Ok(()) if valid, Err with message if invalid
    pub fn validar_comprobante(condicion: i32, cbte_tipo: i32) -> Result<(), &'static str> {
        use super::cbte_tipos::*;

        let es_tipo_a_m = matches!(cbte_tipo, FACTURA_A | NOTA_DEBITO_A | NOTA_CREDITO_A);
        let es_tipo_b = matches!(cbte_tipo, FACTURA_B | NOTA_DEBITO_B | NOTA_CREDITO_B);
        let es_tipo_c = matches!(cbte_tipo, FACTURA_C | NOTA_DEBITO_C | NOTA_CREDITO_C);
        let es_tipo_49 = cbte_tipo == 49;

        if es_tipo_a_m && !puede_recibir_a_m(condicion) {
            return Err("El receptor no puede recibir comprobantes tipo A/M");
        }
        if es_tipo_b && !puede_recibir_b(condicion) {
            return Err("El receptor no puede recibir comprobantes tipo B");
        }
        if es_tipo_c && !puede_recibir_c(condicion) {
            return Err("El receptor no puede recibir comprobantes tipo C");
        }
        if es_tipo_49 && !puede_recibir_49(condicion) {
            return Err("El receptor no puede recibir comprobante tipo 49");
        }

        Ok(())
    }
}

/// IVA rates (Alícuotas de IVA)
pub mod alicuotas_iva {
    pub const NO_GRAVADO: i32 = 1;
    pub const EXENTO: i32 = 2;
    pub const CERO_PORCIENTO: i32 = 3;
    pub const DIEZ_CINCO_PORCIENTO: i32 = 4;  // 10.5%
    pub const VEINTIUNO_PORCIENTO: i32 = 5;   // 21%
    pub const VEINTISIETE_PORCIENTO: i32 = 6; // 27%
    pub const CINCO_PORCIENTO: i32 = 8;       // 5%
    pub const DOS_CINCO_PORCIENTO: i32 = 9;   // 2.5%
}

/// Invoice types (Tipos de Comprobante)
pub mod cbte_tipos {
    pub const FACTURA_A: i32 = 1;
    pub const NOTA_DEBITO_A: i32 = 2;
    pub const NOTA_CREDITO_A: i32 = 3;
    pub const FACTURA_B: i32 = 6;
    pub const NOTA_DEBITO_B: i32 = 7;
    pub const NOTA_CREDITO_B: i32 = 8;
    pub const FACTURA_C: i32 = 11;
    pub const NOTA_DEBITO_C: i32 = 12;
    pub const NOTA_CREDITO_C: i32 = 13;
}

/// Currency types (Tipos de Moneda)
pub mod monedas {
    pub const PESOS: &str = "PES";
    pub const DOLAR: &str = "DOL";
    pub const EURO: &str = "060";
    pub const REAL: &str = "012";
}

// ========== Structs ==========

/// Header for invoice request
#[derive(Debug, Clone)]
pub struct FeCabReq {
    pub cant_reg: i32,
    pub pto_vta: i32,
    pub cbte_tipo: i32,
}

/// IVA allocation detail
#[derive(Debug, Clone)]
pub struct AlicIva {
    pub id: i32,
    pub base_imp: f64,
    pub importe: f64,
}

/// Detail for invoice request
#[derive(Debug, Clone)]
pub struct FeDetReq {
    pub concepto: i32,
    pub doc_tipo: i32,
    pub doc_nro: u64,
    pub cbte_desde: u64,
    pub cbte_hasta: u64,
    pub cbte_fch: String,
    pub imp_total: f64,
    pub imp_tot_conc: f64,
    pub imp_neto: f64,
    pub imp_op_ex: f64,
    pub imp_trib: f64,
    pub imp_iva: f64,
    pub fch_serv_desde: Option<String>,
    pub fch_serv_hasta: Option<String>,
    pub fch_vto_pago: Option<String>,
    pub mon_id: String,
    pub mon_cotiz: f64,
    /// Receiver's IVA condition (mandatory since RG 5616)
    /// Common values: 1=IVA Responsable Inscripto, 5=Consumidor Final, 6=Monotributo
    pub condicion_iva_receptor: Option<i32>,
    pub iva: Vec<AlicIva>,
}

/// Response from CAE authorization (basic)
#[derive(Debug, Clone)]
pub struct FeCaeResponse {
    pub cae: String,
    pub cae_fch_vto: String,
}

/// Detailed response from CAE authorization with errors/observations
#[derive(Debug, Clone)]
pub struct FeCaeResponseDetallada {
    /// Processing result ("A"=Approved, "R"=Rejected, "P"=Partial)
    pub resultado: String,
    /// CAE code (empty if rejected)
    pub cae: String,
    /// CAE expiration date (empty if rejected)
    pub cae_fch_vto: String,
    /// Invoice number from
    pub cbte_desde: u64,
    /// Invoice number to
    pub cbte_hasta: u64,
    /// Processing date
    pub fch_proceso: String,
    /// Errors, observations and events from AFIP
    pub detalles: ArcaResponseDetails,
}

impl FeCaeResponseDetallada {
    /// Returns true if the invoice was approved (result "A")
    pub fn is_approved(&self) -> bool {
        self.resultado == "A"
    }

    /// Returns true if the invoice was rejected (result "R")
    pub fn is_rejected(&self) -> bool {
        self.resultado == "R"
    }

    /// Returns true if there were partial results (result "P")
    pub fn is_partial(&self) -> bool {
        self.resultado == "P"
    }

    /// Convert to basic FeCaeResponse (only if approved)
    pub fn to_basic(&self) -> Option<FeCaeResponse> {
        if self.is_approved() && !self.cae.is_empty() {
            Some(FeCaeResponse {
                cae: self.cae.clone(),
                cae_fch_vto: self.cae_fch_vto.clone(),
            })
        } else {
            None
        }
    }
}

/// Generic parameter type (used by most FEParamGet* methods)
#[derive(Debug, Clone)]
pub struct ParamTipo {
    pub id: i32,
    pub desc: String,
    pub fch_desde: Option<String>,
    pub fch_hasta: Option<String>,
}

/// Point of sale information
#[derive(Debug, Clone)]
pub struct PtoVenta {
    pub nro: i32,
    pub emision_tipo: String,
    pub bloqueado: String,
    pub fch_baja: Option<String>,
}

/// Currency exchange rate
#[derive(Debug, Clone)]
pub struct Cotizacion {
    pub mon_id: String,
    pub mon_cotiz: f64,
    pub fch_cotiz: String,
}

// ========== CAEA Types ==========

/// CAEA (Código de Autorización Electrónico Anticipado) response
#[derive(Debug, Clone)]
pub struct CaeaResponse {
    /// The CAEA code
    pub caea: String,
    /// Period (YYYYMM)
    pub periodo: String,
    /// Order within the period (1=first half, 2=second half)
    pub orden: i32,
    /// From date for the CAEA validity (YYYYMMDD)
    pub fch_vig_desde: String,
    /// To date for the CAEA validity (YYYYMMDD)
    pub fch_vig_hasta: String,
    /// Date to report invoices (YYYYMMDD)
    pub fch_tope_inf: String,
    /// Processing date
    pub fch_proceso: String,
}

/// CAEA informative registration result
#[derive(Debug, Clone)]
pub struct CaeaRegInformativoResult {
    /// The CAEA code used
    pub caea: String,
    /// Processing result ("A"=Approved, "R"=Rejected)
    pub resultado: String,
    /// Invoice number from
    pub cbte_desde: u64,
    /// Invoice number to
    pub cbte_hasta: u64,
}

/// CAEA period without movement
#[derive(Debug, Clone)]
pub struct CaeaSinMovimiento {
    /// Point of sale
    pub pto_vta: i32,
    /// The CAEA code
    pub caea: String,
    /// Processing date
    pub fch_proceso: String,
}

// ========== AFIP Response Details ==========

/// AFIP Error detail
#[derive(Debug, Clone, Default)]
pub struct ArcaErr {
    /// Error code
    pub code: i32,
    /// Error message
    pub msg: String,
}

/// AFIP Observation detail (warning or informational message)
#[derive(Debug, Clone, Default)]
pub struct ArcaObs {
    /// Observation code
    pub code: i32,
    /// Observation message
    pub msg: String,
}

/// AFIP Event detail (system event)
#[derive(Debug, Clone, Default)]
pub struct ArcaEvt {
    /// Event code
    pub code: i32,
    /// Event message
    pub msg: String,
}

/// Collection of errors, observations and events from AFIP response
#[derive(Debug, Clone, Default)]
pub struct ArcaResponseDetails {
    /// Errors - these indicate the operation was rejected
    pub errors: Vec<ArcaErr>,
    /// Observations - warnings or informational messages
    pub observaciones: Vec<ArcaObs>,
    /// Events - system events
    pub events: Vec<ArcaEvt>,
}

impl ArcaResponseDetails {
    /// Returns true if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns true if there are any observations
    pub fn has_observaciones(&self) -> bool {
        !self.observaciones.is_empty()
    }

    /// Returns true if there are any events
    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }

    /// Returns a formatted string with all errors
    pub fn format_errors(&self) -> String {
        self.errors
            .iter()
            .map(|e| format!("[{}] {}", e.code, e.msg))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Returns a formatted string with all observations
    pub fn format_observaciones(&self) -> String {
        self.observaciones
            .iter()
            .map(|o| format!("[{}] {}", o.code, o.msg))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Returns a formatted string with all events
    pub fn format_events(&self) -> String {
        self.events
            .iter()
            .map(|e| format!("[{}] {}", e.code, e.msg))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

/// Response from FECompConsultar - contains full invoice data
#[derive(Debug, Clone)]
pub struct FeCompConsultarResponse {
    /// Document type code
    pub doc_tipo: i32,
    /// Document number
    pub doc_nro: u64,
    /// Invoice type code
    pub cbte_tipo: i32,
    /// Point of sale
    pub pto_vta: i32,
    /// Invoice number from
    pub cbte_desde: u64,
    /// Invoice number to
    pub cbte_hasta: u64,
    /// Invoice date (YYYYMMDD)
    pub cbte_fch: String,
    /// Total amount
    pub imp_total: f64,
    /// Non-taxable amount
    pub imp_tot_conc: f64,
    /// Net taxable amount
    pub imp_neto: f64,
    /// Exempt operations amount
    pub imp_op_ex: f64,
    /// Other taxes amount
    pub imp_trib: f64,
    /// IVA amount
    pub imp_iva: f64,
    /// Concept code (1=Products, 2=Services, 3=Both)
    pub concepto: i32,
    /// Currency code
    pub mon_id: String,
    /// Exchange rate
    pub mon_cotiz: f64,
    /// Processing result ("A"=Approved, "R"=Rejected)
    pub resultado: String,
    /// CAE number
    pub cod_autorizacion: String,
    /// Authorization type ("E"=CAE, "A"=CAEA)
    pub emision_tipo: String,
    /// CAE expiration date (YYYYMMDD)
    pub fch_vto: String,
    /// Service from date (for concepts 2 or 3)
    pub fch_serv_desde: Option<String>,
    /// Service to date (for concepts 2 or 3)
    pub fch_serv_hasta: Option<String>,
    /// Payment due date
    pub fch_vto_pago: Option<String>,
    /// Processing date
    pub fch_proceso: String,
}
