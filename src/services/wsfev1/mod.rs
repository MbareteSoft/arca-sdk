//! WSFEv1 - Facturación Electrónica
//!
//! Servicio para emisión de comprobantes electrónicos (facturas, notas de crédito/débito).
//!
//! ## Métodos principales
//!
//! | Método | Descripción |
//! |--------|-------------|
//! | [`WsFev1Service::fe_dummy`] | Verificar estado del servicio |
//! | [`WsFev1Service::fe_comp_ultimo_autorizado`] | Último comprobante autorizado |
//! | [`WsFev1Service::fecae_solicitar`] | Solicitar CAE |
//! | [`WsFev1Service::fecae_solicitar_detallado`] | Solicitar CAE con errores detallados |
//! | [`WsFev1Service::fe_comp_consultar`] | Consultar comprobante emitido |
//!
//! ## Ejemplo: Emitir Factura B
//!
//! ```rust,no_run
//! use arca::{
//!     WsFev1Service, FeCabReq, FeDetReq, AlicIva,
//!     cbte_tipos, doc_tipos, condicion_iva, alicuotas_iva,
//!     transport::HttpClient,
//! };
//!
//! # async fn example(token: &str, sign: &str, cuit: u64) -> anyhow::Result<()> {
//! let http = HttpClient::new()?;
//! let wsfev1 = WsFev1Service::new(
//!     "https://wswhomo.afip.gov.ar/wsfev1/service.asmx".to_string(),
//!     http,
//! );
//!
//! // Obtener último número
//! let ultimo = wsfev1.fe_comp_ultimo_autorizado(
//!     token, sign, cuit, 1, cbte_tipos::FACTURA_B
//! ).await?;
//!
//! let cab = FeCabReq {
//!     cant_reg: 1,
//!     pto_vta: 1,
//!     cbte_tipo: cbte_tipos::FACTURA_B,
//! };
//!
//! let det = FeDetReq {
//!     concepto: 1,
//!     doc_tipo: doc_tipos::CONSUMIDOR_FINAL,
//!     doc_nro: 0,
//!     cbte_desde: (ultimo + 1) as u64,
//!     cbte_hasta: (ultimo + 1) as u64,
//!     cbte_fch: "20251226".to_string(),
//!     imp_total: 121.0,
//!     imp_tot_conc: 0.0,
//!     imp_neto: 100.0,
//!     imp_op_ex: 0.0,
//!     imp_trib: 0.0,
//!     imp_iva: 21.0,
//!     fch_serv_desde: None,
//!     fch_serv_hasta: None,
//!     fch_vto_pago: None,
//!     mon_id: "PES".to_string(),
//!     mon_cotiz: 1.0,
//!     condicion_iva_receptor: Some(condicion_iva::CONSUMIDOR_FINAL),
//!     iva: vec![AlicIva {
//!         id: alicuotas_iva::VEINTIUNO_PORCIENTO,
//!         base_imp: 100.0,
//!         importe: 21.0,
//!     }],
//! };
//!
//! let resp = wsfev1.fecae_solicitar(token, sign, cuit, cab, vec![det]).await?;
//! println!("CAE: {}", resp.cae);
//! # Ok(())
//! # }
//! ```
//!
//! ## Constantes disponibles
//!
//! - [`cbte_tipos`] - Tipos de comprobante (FACTURA_A, FACTURA_B, etc.)
//! - [`doc_tipos`] - Tipos de documento (CUIT, DNI, etc.)
//! - [`condicion_iva`] - Condición IVA del receptor
//! - [`alicuotas_iva`] - Alícuotas de IVA
//! - [`monedas`] - Códigos de moneda
//!
//! ## CAEA (Modo Contingencia)
//!
//! Para emitir facturas offline, usar los métodos CAEA:
//!
//! - [`WsFev1Service::fecaea_solicitar`] - Solicitar CAEA anticipado
//! - [`WsFev1Service::fecaea_consultar`] - Consultar CAEA existente
//! - [`WsFev1Service::fecaea_reg_informativo`] - Informar facturas emitidas offline
//! - [`WsFev1Service::fecaea_sin_movimiento_informar`] - Declarar sin movimiento

pub mod errores;
pub mod requests;
pub mod types;

pub use errores as codigos_error;
pub use requests::{
    build_fe_comp_consultar_req, build_fe_comp_ultimo_autorizado_req, build_fe_dummy_req,
    build_fe_param_get_condicion_iva_receptor_req, build_fe_param_get_cotizacion_req,
    build_fe_param_get_ptos_venta_req, build_fe_param_get_tipos_cbte_req,
    build_fe_param_get_tipos_concepto_req, build_fe_param_get_tipos_doc_req,
    build_fe_param_get_tipos_iva_req, build_fe_param_get_tipos_monedas_req,
    build_fe_param_get_tipos_opcional_req, build_fe_param_get_tipos_tributos_req,
    build_fecae_solicitar_req, build_fecaea_consultar_req, build_fecaea_reg_informativo_req,
    build_fecaea_sin_movimiento_consultar_req, build_fecaea_sin_movimiento_informar_req,
    build_fecaea_solicitar_req,
};
pub use types::{
    ArcaErr, ArcaEvt, ArcaObs, ArcaResponseDetails,
    AlicIva, CaeaRegInformativoResult, CaeaResponse, CaeaSinMovimiento, Cotizacion, FeCabReq,
    FeCaeResponse, FeCaeResponseDetallada, FeCompConsultarResponse, FeDetReq, ParamTipo, PtoVenta,
    // AFIP constants
    alicuotas_iva, cbte_tipos, conceptos, condicion_iva, doc_tipos, monedas,
};

use crate::constants::namespaces::wsfev1::{NAMESPACE, PREFIX};
use crate::error::{ArcaError, Result};
use crate::services::ArcaService;
use crate::transport::{build_soap_action, post_soap, HttpClient};
use crate::xml::{extract_all_blocks, extract_inline_text, extract_tag_text};
use async_trait::async_trait;

// ========== AFIP Response Parsing Helpers ==========

/// Extract all errors from AFIP response
fn parse_arca_errors(resp: &str) -> Vec<ArcaErr> {
    extract_all_blocks(resp, "Err")
        .into_iter()
        .filter_map(|block| {
            let code = extract_inline_text(&block, "Code")?.parse().ok()?;
            let msg = extract_inline_text(&block, "Msg")?;
            Some(ArcaErr { code, msg })
        })
        .collect()
}

/// Extract all observations from AFIP response
fn parse_arca_observaciones(resp: &str) -> Vec<ArcaObs> {
    extract_all_blocks(resp, "Obs")
        .into_iter()
        .filter_map(|block| {
            let code = extract_inline_text(&block, "Code")?.parse().ok()?;
            let msg = extract_inline_text(&block, "Msg")?;
            Some(ArcaObs { code, msg })
        })
        .collect()
}

/// Extract all events from AFIP response
fn parse_arca_events(resp: &str) -> Vec<ArcaEvt> {
    extract_all_blocks(resp, "Evt")
        .into_iter()
        .filter_map(|block| {
            let code = extract_inline_text(&block, "Code")?.parse().ok()?;
            let msg = extract_inline_text(&block, "Msg")?;
            Some(ArcaEvt { code, msg })
        })
        .collect()
}

/// Extract all response details (errors, observations, events)
fn parse_arca_response_details(resp: &str) -> ArcaResponseDetails {
    ArcaResponseDetails {
        errors: parse_arca_errors(resp),
        observaciones: parse_arca_observaciones(resp),
        events: parse_arca_events(resp),
    }
}

/// WSFEv1 service for electronic invoicing
pub struct WsFev1Service {
    endpoint: String,
    http_client: HttpClient,
}

impl WsFev1Service {
    pub fn new(endpoint: String, http_client: HttpClient) -> Self {
        Self {
            endpoint,
            http_client,
        }
    }

    /// Health check - FEDummy
    pub async fn fe_dummy(&self) -> Result<(String, String, String)> {
        let soap = build_fe_dummy_req();
        let action = build_soap_action(NAMESPACE, "FEDummy");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;

        Ok((
            extract_tag_text(&resp, "AppServer")
                .or(extract_tag_text(&resp, "appserver"))
                .unwrap_or_default(),
            extract_tag_text(&resp, "AuthServer")
                .or(extract_tag_text(&resp, "authserver"))
                .unwrap_or_default(),
            extract_tag_text(&resp, "DbServer")
                .or(extract_tag_text(&resp, "dbserver"))
                .unwrap_or_default(),
        ))
    }

    /// Get last authorized voucher number
    pub async fn fe_comp_ultimo_autorizado(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
        pto_vta: i32,
        cbte_tipo: i32,
    ) -> Result<i32> {
        let soap = build_fe_comp_ultimo_autorizado_req(token, sign, cuit, pto_vta, cbte_tipo);
        let action = build_soap_action(NAMESPACE, "FECompUltimoAutorizado");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;

        let cbte_nro = extract_tag_text(&resp, "CbteNro").map_err(|e| ArcaError::Xml(e.to_string()))?;
        cbte_nro
            .parse()
            .map_err(|_| ArcaError::Xml("Failed to parse CbteNro".to_string()))
    }

    /// Request CAE authorization for invoices
    pub async fn fecae_solicitar(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
        cab: FeCabReq,
        dets: Vec<FeDetReq>,
    ) -> Result<FeCaeResponse> {
        let soap = build_fecae_solicitar_req(token, sign, cuit, &cab, dets);
        let action = build_soap_action(NAMESPACE, "FECAESolicitar");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;

        // Check for AFIP errors first
        if let Ok(resultado) = extract_tag_text(&resp, "Resultado") {
            if resultado == "R" {
                // Rejected - extract error details
                let mut error_msg = String::from("AFIP rejected the request");
                if let Ok(code) = extract_tag_text(&resp, "Code") {
                    error_msg.push_str(&format!(" (Code: {})", code));
                }
                if let Ok(msg) = extract_tag_text(&resp, "Msg") {
                    error_msg.push_str(&format!(": {}", msg));
                }
                // Also check for Observaciones
                if let Ok(obs) = extract_tag_text(&resp, "Obs") {
                    error_msg.push_str(&format!(" | Obs: {}", obs));
                }
                return Err(ArcaError::Xml(error_msg));
            }
        }

        // Try to extract CAE - if not found, include more context
        let cae = match extract_tag_text(&resp, "CAE") {
            Ok(c) if !c.is_empty() => c,
            _ => {
                // No CAE found - check for errors in response
                let mut error_msg = String::from("No CAE in response");
                if let Ok(code) = extract_tag_text(&resp, "Code") {
                    error_msg.push_str(&format!(" | Code: {}", code));
                }
                if let Ok(msg) = extract_tag_text(&resp, "Msg") {
                    error_msg.push_str(&format!(" | Msg: {}", msg));
                }
                // Include first 500 chars of response for debugging
                if resp.len() > 100 {
                    error_msg.push_str(&format!(" | Response: {}...", &resp[..500.min(resp.len())]));
                }
                return Err(ArcaError::Xml(error_msg));
            }
        };

        let cae_fch_vto = extract_tag_text(&resp, "CAEFchVto").unwrap_or_default();

        Ok(FeCaeResponse { cae, cae_fch_vto })
    }

    /// Request CAE authorization with detailed response including errors/observations
    ///
    /// This version returns all details from AFIP including:
    /// - Errors (if rejected)
    /// - Observations (warnings)
    /// - Events (system events)
    ///
    /// Use this method when you need to handle partial approvals or want to
    /// see observations even on successful requests.
    pub async fn fecae_solicitar_detallado(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
        cab: FeCabReq,
        dets: Vec<FeDetReq>,
    ) -> Result<FeCaeResponseDetallada> {
        let soap = build_fecae_solicitar_req(token, sign, cuit, &cab, dets);
        let action = build_soap_action(NAMESPACE, "FECAESolicitar");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;

        // Parse response details (errors, observations, events)
        let detalles = parse_arca_response_details(&resp);

        // Extract the result and CAE info from FECAEDetResponse
        let resultado = extract_tag_text(&resp, "Resultado").unwrap_or_default();
        let cae = extract_tag_text(&resp, "CAE").unwrap_or_default();
        let cae_fch_vto = extract_tag_text(&resp, "CAEFchVto").unwrap_or_default();
        let cbte_desde = extract_tag_text(&resp, "CbteDesde")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let cbte_hasta = extract_tag_text(&resp, "CbteHasta")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let fch_proceso = extract_tag_text(&resp, "FchProceso").unwrap_or_default();

        Ok(FeCaeResponseDetallada {
            resultado,
            cae,
            cae_fch_vto,
            cbte_desde,
            cbte_hasta,
            fch_proceso,
            detalles,
        })
    }

    /// Retrieve an existing invoice by its type, number and point of sale
    ///
    /// This is critical for error recovery - if a CAE request times out,
    /// you can use this method to check if the invoice was actually authorized.
    pub async fn fe_comp_consultar(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
        cbte_tipo: i32,
        cbte_nro: u64,
        pto_vta: i32,
    ) -> Result<FeCompConsultarResponse> {
        let soap = build_fe_comp_consultar_req(token, sign, cuit, cbte_tipo, cbte_nro, pto_vta);
        let action = build_soap_action(NAMESPACE, "FECompConsultar");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;

        // Parse required fields
        let parse_i32 = |tag: &str| -> Result<i32> {
            extract_tag_text(&resp, tag)
                .map_err(|e| ArcaError::Xml(e.to_string()))?
                .parse()
                .map_err(|_| ArcaError::Xml(format!("Failed to parse {} as i32", tag)))
        };

        let parse_u64 = |tag: &str| -> Result<u64> {
            extract_tag_text(&resp, tag)
                .map_err(|e| ArcaError::Xml(e.to_string()))?
                .parse()
                .map_err(|_| ArcaError::Xml(format!("Failed to parse {} as u64", tag)))
        };

        let parse_f64 = |tag: &str| -> Result<f64> {
            extract_tag_text(&resp, tag)
                .map_err(|e| ArcaError::Xml(e.to_string()))?
                .parse()
                .map_err(|_| ArcaError::Xml(format!("Failed to parse {} as f64", tag)))
        };

        let parse_string = |tag: &str| -> Result<String> {
            extract_tag_text(&resp, tag).map_err(|e| ArcaError::Xml(e.to_string()))
        };

        let parse_optional_string = |tag: &str| -> Option<String> {
            extract_tag_text(&resp, tag).ok()
        };

        Ok(FeCompConsultarResponse {
            doc_tipo: parse_i32("DocTipo")?,
            doc_nro: parse_u64("DocNro")?,
            cbte_tipo: parse_i32("CbteTipo")?,
            pto_vta: parse_i32("PtoVta")?,
            cbte_desde: parse_u64("CbteDesde")?,
            cbte_hasta: parse_u64("CbteHasta")?,
            cbte_fch: parse_string("CbteFch")?,
            imp_total: parse_f64("ImpTotal")?,
            imp_tot_conc: parse_f64("ImpTotConc")?,
            imp_neto: parse_f64("ImpNeto")?,
            imp_op_ex: parse_f64("ImpOpEx")?,
            imp_trib: parse_f64("ImpTrib")?,
            imp_iva: parse_f64("ImpIVA")?,
            concepto: parse_i32("Concepto")?,
            mon_id: parse_string("MonId")?,
            mon_cotiz: parse_f64("MonCotiz")?,
            resultado: parse_string("Resultado")?,
            cod_autorizacion: parse_string("CodAutorizacion")?,
            emision_tipo: parse_string("EmisionTipo")?,
            fch_vto: parse_string("FchVto")?,
            fch_serv_desde: parse_optional_string("FchServDesde"),
            fch_serv_hasta: parse_optional_string("FchServHasta"),
            fch_vto_pago: parse_optional_string("FchVtoPago"),
            fch_proceso: parse_string("FchProceso")?,
        })
    }

    // ========== FEParamGet* methods ==========

    /// Helper to parse a list of ParamTipo from AFIP response
    fn parse_param_tipo_list(resp: &str, container_tag: &str) -> Vec<ParamTipo> {
        extract_all_blocks(resp, container_tag)
            .into_iter()
            .filter_map(|block| {
                let id = extract_inline_text(&block, "Id")?.parse().ok()?;
                let desc = extract_inline_text(&block, "Desc")?;
                let fch_desde = extract_inline_text(&block, "FchDesde");
                let fch_hasta = extract_inline_text(&block, "FchHasta");
                Some(ParamTipo {
                    id,
                    desc,
                    fch_desde,
                    fch_hasta,
                })
            })
            .collect()
    }

    /// Get voucher types (FEParamGetTiposCbte)
    pub async fn fe_param_get_tipos_cbte(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
    ) -> Result<Vec<ParamTipo>> {
        let soap = build_fe_param_get_tipos_cbte_req(token, sign, cuit);
        let action = build_soap_action(NAMESPACE, "FEParamGetTiposCbte");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;
        Ok(Self::parse_param_tipo_list(&resp, "CbteTipo"))
    }

    /// Get document types (FEParamGetTiposDoc)
    pub async fn fe_param_get_tipos_doc(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
    ) -> Result<Vec<ParamTipo>> {
        let soap = build_fe_param_get_tipos_doc_req(token, sign, cuit);
        let action = build_soap_action(NAMESPACE, "FEParamGetTiposDoc");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;
        Ok(Self::parse_param_tipo_list(&resp, "DocTipo"))
    }

    /// Get IVA types (FEParamGetTiposIva)
    pub async fn fe_param_get_tipos_iva(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
    ) -> Result<Vec<ParamTipo>> {
        let soap = build_fe_param_get_tipos_iva_req(token, sign, cuit);
        let action = build_soap_action(NAMESPACE, "FEParamGetTiposIva");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;
        Ok(Self::parse_param_tipo_list(&resp, "IvaTipo"))
    }

    /// Get currency types (FEParamGetTiposMonedas)
    pub async fn fe_param_get_tipos_monedas(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
    ) -> Result<Vec<ParamTipo>> {
        let soap = build_fe_param_get_tipos_monedas_req(token, sign, cuit);
        let action = build_soap_action(NAMESPACE, "FEParamGetTiposMonedas");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;
        Ok(Self::parse_param_tipo_list(&resp, "Moneda"))
    }

    /// Get tax types (FEParamGetTiposTributos)
    pub async fn fe_param_get_tipos_tributos(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
    ) -> Result<Vec<ParamTipo>> {
        let soap = build_fe_param_get_tipos_tributos_req(token, sign, cuit);
        let action = build_soap_action(NAMESPACE, "FEParamGetTiposTributos");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;
        Ok(Self::parse_param_tipo_list(&resp, "TributoTipo"))
    }

    /// Get concept types (FEParamGetTiposConcepto)
    pub async fn fe_param_get_tipos_concepto(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
    ) -> Result<Vec<ParamTipo>> {
        let soap = build_fe_param_get_tipos_concepto_req(token, sign, cuit);
        let action = build_soap_action(NAMESPACE, "FEParamGetTiposConcepto");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;
        Ok(Self::parse_param_tipo_list(&resp, "ConceptoTipo"))
    }

    /// Get optional field types (FEParamGetTiposOpcional)
    pub async fn fe_param_get_tipos_opcional(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
    ) -> Result<Vec<ParamTipo>> {
        let soap = build_fe_param_get_tipos_opcional_req(token, sign, cuit);
        let action = build_soap_action(NAMESPACE, "FEParamGetTiposOpcional");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;
        Ok(Self::parse_param_tipo_list(&resp, "OpcionalTipo"))
    }

    /// Get receiver IVA conditions (FEParamGetCondicionIvaReceptor)
    pub async fn fe_param_get_condicion_iva_receptor(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
    ) -> Result<Vec<ParamTipo>> {
        let soap = build_fe_param_get_condicion_iva_receptor_req(token, sign, cuit);
        let action = build_soap_action(NAMESPACE, "FEParamGetCondicionIvaReceptor");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;
        Ok(Self::parse_param_tipo_list(&resp, "CondicionIvaReceptor"))
    }

    /// Get points of sale (FEParamGetPtosVenta)
    pub async fn fe_param_get_ptos_venta(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
    ) -> Result<Vec<PtoVenta>> {
        let soap = build_fe_param_get_ptos_venta_req(token, sign, cuit);
        let action = build_soap_action(NAMESPACE, "FEParamGetPtosVenta");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;

        let ptos = extract_all_blocks(&resp, "PtoVenta")
            .into_iter()
            .filter_map(|block| {
                let nro = extract_inline_text(&block, "Nro")?.parse().ok()?;
                let emision_tipo = extract_inline_text(&block, "EmisionTipo")?;
                let bloqueado = extract_inline_text(&block, "Bloqueado")?;
                let fch_baja = extract_inline_text(&block, "FchBaja");
                Some(PtoVenta {
                    nro,
                    emision_tipo,
                    bloqueado,
                    fch_baja,
                })
            })
            .collect();
        Ok(ptos)
    }

    /// Get currency exchange rate (FEParamGetCotizacion)
    pub async fn fe_param_get_cotizacion(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
        mon_id: &str,
    ) -> Result<Cotizacion> {
        let soap = build_fe_param_get_cotizacion_req(token, sign, cuit, mon_id);
        let action = build_soap_action(NAMESPACE, "FEParamGetCotizacion");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;

        let mon_cotiz = extract_tag_text(&resp, "MonCotiz")
            .map_err(|e| ArcaError::Xml(e.to_string()))?
            .parse()
            .map_err(|_| ArcaError::Xml("Failed to parse MonCotiz".to_string()))?;

        let mon_id = extract_tag_text(&resp, "MonId")
            .map_err(|e| ArcaError::Xml(e.to_string()))?;

        let fch_cotiz = extract_tag_text(&resp, "FchCotiz")
            .map_err(|e| ArcaError::Xml(e.to_string()))?;

        Ok(Cotizacion {
            mon_id,
            mon_cotiz,
            fch_cotiz,
        })
    }

    // ========== CAEA Methods ==========

    /// Request a CAEA for a specific period (FECAEASolicitar)
    ///
    /// Period format: YYYYMM (e.g., 202501 for January 2025)
    /// Order: 1 for first half of month, 2 for second half
    pub async fn fecaea_solicitar(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
        periodo: i32,
        orden: i16,
    ) -> Result<CaeaResponse> {
        let soap = build_fecaea_solicitar_req(token, sign, cuit, periodo, orden);
        let action = build_soap_action(NAMESPACE, "FECAEASolicitar");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;

        Self::parse_caea_response(&resp)
    }

    /// Consult a CAEA for a specific period (FECAEAConsultar)
    pub async fn fecaea_consultar(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
        periodo: i32,
        orden: i16,
    ) -> Result<CaeaResponse> {
        let soap = build_fecaea_consultar_req(token, sign, cuit, periodo, orden);
        let action = build_soap_action(NAMESPACE, "FECAEAConsultar");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;

        Self::parse_caea_response(&resp)
    }

    /// Helper to parse CAEA response
    fn parse_caea_response(resp: &str) -> Result<CaeaResponse> {
        // First check for AFIP errors
        if let Ok(err_code) = extract_tag_text(resp, "Code") {
            if let Ok(err_msg) = extract_tag_text(resp, "Msg") {
                return Err(ArcaError::Xml(format!("AFIP Error {}: {}", err_code, err_msg)));
            }
        }

        let parse_string = |tag: &str| -> Result<String> {
            extract_tag_text(resp, tag).map_err(|e| ArcaError::Xml(e.to_string()))
        };

        let parse_i32 = |tag: &str| -> Result<i32> {
            extract_tag_text(resp, tag)
                .map_err(|e| ArcaError::Xml(e.to_string()))?
                .parse()
                .map_err(|_| ArcaError::Xml(format!("Failed to parse {} as i32", tag)))
        };

        Ok(CaeaResponse {
            caea: parse_string("CAEA")?,
            periodo: parse_string("Periodo")?,
            orden: parse_i32("Orden")?,
            fch_vig_desde: parse_string("FchVigDesde")?,
            fch_vig_hasta: parse_string("FchVigHasta")?,
            fch_tope_inf: parse_string("FchTopeInf")?,
            fch_proceso: parse_string("FchProceso")?,
        })
    }

    /// Register invoices issued with CAEA (FECAEARegInformativo)
    ///
    /// Used to inform AFIP about invoices that were issued offline with a CAEA.
    pub async fn fecaea_reg_informativo(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
        cab: FeCabReq,
        dets: Vec<FeDetReq>,
        caea: &str,
    ) -> Result<CaeaRegInformativoResult> {
        let soap = build_fecaea_reg_informativo_req(token, sign, cuit, &cab, dets, caea);
        let action = build_soap_action(NAMESPACE, "FECAEARegInformativo");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;

        let caea = extract_tag_text(&resp, "CAEA")
            .map_err(|e| ArcaError::Xml(e.to_string()))?;
        let resultado = extract_tag_text(&resp, "Resultado")
            .map_err(|e| ArcaError::Xml(e.to_string()))?;
        let cbte_desde = extract_tag_text(&resp, "CbteDesde")
            .map_err(|e| ArcaError::Xml(e.to_string()))?
            .parse()
            .map_err(|_| ArcaError::Xml("Failed to parse CbteDesde".to_string()))?;
        let cbte_hasta = extract_tag_text(&resp, "CbteHasta")
            .map_err(|e| ArcaError::Xml(e.to_string()))?
            .parse()
            .map_err(|_| ArcaError::Xml("Failed to parse CbteHasta".to_string()))?;

        Ok(CaeaRegInformativoResult {
            caea,
            resultado,
            cbte_desde,
            cbte_hasta,
        })
    }

    /// Inform AFIP that a CAEA period had no invoices (FECAEASinMovimientoInformar)
    pub async fn fecaea_sin_movimiento_informar(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
        pto_vta: i32,
        caea: &str,
    ) -> Result<CaeaSinMovimiento> {
        let soap = build_fecaea_sin_movimiento_informar_req(token, sign, cuit, pto_vta, caea);
        let action = build_soap_action(NAMESPACE, "FECAEASinMovimientoInformar");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;

        let pto_vta = extract_tag_text(&resp, "PtoVta")
            .map_err(|e| ArcaError::Xml(e.to_string()))?
            .parse()
            .map_err(|_| ArcaError::Xml("Failed to parse PtoVta".to_string()))?;
        let caea = extract_tag_text(&resp, "CAEA")
            .map_err(|e| ArcaError::Xml(e.to_string()))?;
        let fch_proceso = extract_tag_text(&resp, "FchProceso")
            .map_err(|e| ArcaError::Xml(e.to_string()))?;

        Ok(CaeaSinMovimiento {
            pto_vta,
            caea,
            fch_proceso,
        })
    }

    /// Check if a CAEA period was declared as having no invoices (FECAEASinMovimientoConsultar)
    pub async fn fecaea_sin_movimiento_consultar(
        &self,
        token: &str,
        sign: &str,
        cuit: u64,
        caea: &str,
        pto_vta: i32,
    ) -> Result<Vec<CaeaSinMovimiento>> {
        let soap = build_fecaea_sin_movimiento_consultar_req(token, sign, cuit, caea, pto_vta);
        let action = build_soap_action(NAMESPACE, "FECAEASinMovimientoConsultar");
        let resp = post_soap(&self.http_client, &self.endpoint, &soap, Some(&action)).await?;

        let results = extract_all_blocks(&resp, "SinMovimiento")
            .into_iter()
            .filter_map(|block| {
                let pto_vta = extract_inline_text(&block, "PtoVta")?.parse().ok()?;
                let caea = extract_inline_text(&block, "CAEA")?;
                let fch_proceso = extract_inline_text(&block, "FchProceso")?;
                Some(CaeaSinMovimiento {
                    pto_vta,
                    caea,
                    fch_proceso,
                })
            })
            .collect();

        Ok(results)
    }
}

#[async_trait]
impl ArcaService for WsFev1Service {
    fn service_id(&self) -> &'static str {
        "wsfe"
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn namespace(&self) -> &'static str {
        NAMESPACE
    }

    fn prefix(&self) -> &'static str {
        PREFIX
    }
}
