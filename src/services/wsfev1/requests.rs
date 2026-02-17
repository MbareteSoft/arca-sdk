use crate::constants::namespaces::wsfev1::{NAMESPACE, PREFIX};
use crate::services::build_auth_block;
use crate::transport::create_soap_envelope;

use super::types::{FeCabReq, FeDetReq};

/// Build the SOAP request for FECompConsultar
pub fn build_fe_comp_consultar_req(
    token: &str,
    sign: &str,
    cuit: u64,
    cbte_tipo: i32,
    cbte_nro: u64,
    pto_vta: i32,
) -> String {
    let auth = build_auth_block(PREFIX, token, sign, cuit);
    let body = format!(
        r#"{}
<{1}:FeCompConsReq>
<{1}:CbteTipo>{2}</{1}:CbteTipo>
<{1}:CbteNro>{3}</{1}:CbteNro>
<{1}:PtoVta>{4}</{1}:PtoVta>
</{1}:FeCompConsReq>"#,
        auth, PREFIX, cbte_tipo, cbte_nro, pto_vta
    );
    create_soap_envelope(NAMESPACE, PREFIX, "FECompConsultar", &body)
}

/// Build the SOAP request for FEDummy (health check)
pub fn build_fe_dummy_req() -> String {
    create_soap_envelope(NAMESPACE, PREFIX, "FEDummy", "")
}

/// Build a generic SOAP request that only requires Auth block
fn build_auth_only_req(token: &str, sign: &str, cuit: u64, method: &str) -> String {
    let auth = build_auth_block(PREFIX, token, sign, cuit);
    create_soap_envelope(NAMESPACE, PREFIX, method, &auth)
}

/// Build the SOAP request for FEParamGetTiposCbte
pub fn build_fe_param_get_tipos_cbte_req(token: &str, sign: &str, cuit: u64) -> String {
    build_auth_only_req(token, sign, cuit, "FEParamGetTiposCbte")
}

/// Build the SOAP request for FEParamGetTiposDoc
pub fn build_fe_param_get_tipos_doc_req(token: &str, sign: &str, cuit: u64) -> String {
    build_auth_only_req(token, sign, cuit, "FEParamGetTiposDoc")
}

/// Build the SOAP request for FEParamGetTiposIva
pub fn build_fe_param_get_tipos_iva_req(token: &str, sign: &str, cuit: u64) -> String {
    build_auth_only_req(token, sign, cuit, "FEParamGetTiposIva")
}

/// Build the SOAP request for FEParamGetTiposMonedas
pub fn build_fe_param_get_tipos_monedas_req(token: &str, sign: &str, cuit: u64) -> String {
    build_auth_only_req(token, sign, cuit, "FEParamGetTiposMonedas")
}

/// Build the SOAP request for FEParamGetTiposTributos
pub fn build_fe_param_get_tipos_tributos_req(token: &str, sign: &str, cuit: u64) -> String {
    build_auth_only_req(token, sign, cuit, "FEParamGetTiposTributos")
}

/// Build the SOAP request for FEParamGetTiposConcepto
pub fn build_fe_param_get_tipos_concepto_req(token: &str, sign: &str, cuit: u64) -> String {
    build_auth_only_req(token, sign, cuit, "FEParamGetTiposConcepto")
}

/// Build the SOAP request for FEParamGetTiposOpcional
pub fn build_fe_param_get_tipos_opcional_req(token: &str, sign: &str, cuit: u64) -> String {
    build_auth_only_req(token, sign, cuit, "FEParamGetTiposOpcional")
}

/// Build the SOAP request for FEParamGetPtosVenta
pub fn build_fe_param_get_ptos_venta_req(token: &str, sign: &str, cuit: u64) -> String {
    build_auth_only_req(token, sign, cuit, "FEParamGetPtosVenta")
}

/// Build the SOAP request for FEParamGetCondicionIvaReceptor
pub fn build_fe_param_get_condicion_iva_receptor_req(
    token: &str,
    sign: &str,
    cuit: u64,
) -> String {
    build_auth_only_req(token, sign, cuit, "FEParamGetCondicionIvaReceptor")
}

/// Build the SOAP request for FEParamGetCotizacion
pub fn build_fe_param_get_cotizacion_req(
    token: &str,
    sign: &str,
    cuit: u64,
    mon_id: &str,
) -> String {
    let auth = build_auth_block(PREFIX, token, sign, cuit);
    let body = format!(
        "{}\n<{}:MonId>{}</{}:MonId>",
        auth, PREFIX, mon_id, PREFIX
    );
    create_soap_envelope(NAMESPACE, PREFIX, "FEParamGetCotizacion", &body)
}

// ========== CAEA Request Builders ==========

/// Build the SOAP request for FECAEASolicitar
pub fn build_fecaea_solicitar_req(
    token: &str,
    sign: &str,
    cuit: u64,
    periodo: i32,
    orden: i16,
) -> String {
    let auth = build_auth_block(PREFIX, token, sign, cuit);
    let body = format!(
        r#"{}
<{1}:Periodo>{2}</{1}:Periodo>
<{1}:Orden>{3}</{1}:Orden>"#,
        auth, PREFIX, periodo, orden
    );
    create_soap_envelope(NAMESPACE, PREFIX, "FECAEASolicitar", &body)
}

/// Build the SOAP request for FECAEAConsultar
pub fn build_fecaea_consultar_req(
    token: &str,
    sign: &str,
    cuit: u64,
    periodo: i32,
    orden: i16,
) -> String {
    let auth = build_auth_block(PREFIX, token, sign, cuit);
    let body = format!(
        r#"{}
<{1}:Periodo>{2}</{1}:Periodo>
<{1}:Orden>{3}</{1}:Orden>"#,
        auth, PREFIX, periodo, orden
    );
    create_soap_envelope(NAMESPACE, PREFIX, "FECAEAConsultar", &body)
}

/// Build the SOAP request for FECAEASinMovimientoInformar
pub fn build_fecaea_sin_movimiento_informar_req(
    token: &str,
    sign: &str,
    cuit: u64,
    pto_vta: i32,
    caea: &str,
) -> String {
    let auth = build_auth_block(PREFIX, token, sign, cuit);
    let body = format!(
        r#"{}
<{1}:PtoVta>{2}</{1}:PtoVta>
<{1}:CAEA>{3}</{1}:CAEA>"#,
        auth, PREFIX, pto_vta, caea
    );
    create_soap_envelope(NAMESPACE, PREFIX, "FECAEASinMovimientoInformar", &body)
}

/// Build the SOAP request for FECAEASinMovimientoConsultar
pub fn build_fecaea_sin_movimiento_consultar_req(
    token: &str,
    sign: &str,
    cuit: u64,
    caea: &str,
    pto_vta: i32,
) -> String {
    let auth = build_auth_block(PREFIX, token, sign, cuit);
    let body = format!(
        r#"{}
<{1}:CAEA>{2}</{1}:CAEA>
<{1}:PtoVta>{3}</{1}:PtoVta>"#,
        auth, PREFIX, caea, pto_vta
    );
    create_soap_envelope(NAMESPACE, PREFIX, "FECAEASinMovimientoConsultar", &body)
}

/// Build the SOAP request for FECAEARegInformativo
pub fn build_fecaea_reg_informativo_req(
    token: &str,
    sign: &str,
    cuit: u64,
    cab: &FeCabReq,
    dets: Vec<FeDetReq>,
    caea: &str,
) -> String {
    let auth = build_auth_block(PREFIX, token, sign, cuit);

    let mut dets_xml = String::new();
    for det in dets {
        let mut iva_xml = String::new();
        if !det.iva.is_empty() {
            iva_xml.push_str(&format!("<{}:Iva>", PREFIX));
            for alic in &det.iva {
                iva_xml.push_str(&format!(
                    r#"<{0}:AlicIva>
<{0}:Id>{1}</{0}:Id>
<{0}:BaseImp>{2}</{0}:BaseImp>
<{0}:Importe>{3}</{0}:Importe>
</{0}:AlicIva>"#,
                    PREFIX, alic.id, alic.base_imp, alic.importe
                ));
            }
            iva_xml.push_str(&format!("</{}:Iva>", PREFIX));
        }

        let optional_dates = format!(
            "{}{}{}",
            det.fch_serv_desde
                .as_ref()
                .map(|s| format!("<{}:FchServDesde>{}</{}:FchServDesde>", PREFIX, s, PREFIX))
                .unwrap_or_default(),
            det.fch_serv_hasta
                .as_ref()
                .map(|s| format!("<{}:FchServHasta>{}</{}:FchServHasta>", PREFIX, s, PREFIX))
                .unwrap_or_default(),
            det.fch_vto_pago
                .as_ref()
                .map(|s| format!("<{}:FchVtoPago>{}</{}:FchVtoPago>", PREFIX, s, PREFIX))
                .unwrap_or_default()
        );

        // CondicionIVAReceptorId is mandatory since RG 5616
        let condicion_iva = det
            .condicion_iva_receptor
            .map(|id| format!("<{}:CondicionIVAReceptorId>{}</{}:CondicionIVAReceptorId>", PREFIX, id, PREFIX))
            .unwrap_or_default();

        dets_xml.push_str(&format!(
            r#"<{0}:FECAEADetRequest>
<{0}:Concepto>{1}</{0}:Concepto>
<{0}:DocTipo>{2}</{0}:DocTipo>
<{0}:DocNro>{3}</{0}:DocNro>
<{0}:CbteDesde>{4}</{0}:CbteDesde>
<{0}:CbteHasta>{5}</{0}:CbteHasta>
<{0}:CbteFch>{6}</{0}:CbteFch>
<{0}:ImpTotal>{7}</{0}:ImpTotal>
<{0}:ImpTotConc>{8}</{0}:ImpTotConc>
<{0}:ImpNeto>{9}</{0}:ImpNeto>
<{0}:ImpOpEx>{10}</{0}:ImpOpEx>
<{0}:ImpTrib>{11}</{0}:ImpTrib>
<{0}:ImpIVA>{12}</{0}:ImpIVA>
{13}
<{0}:MonId>{14}</{0}:MonId>
<{0}:MonCotiz>{15}</{0}:MonCotiz>
{16}
<{0}:CAEA>{17}</{0}:CAEA>
{18}
</{0}:FECAEADetRequest>"#,
            PREFIX,
            det.concepto,
            det.doc_tipo,
            det.doc_nro,
            det.cbte_desde,
            det.cbte_hasta,
            det.cbte_fch,
            det.imp_total,
            det.imp_tot_conc,
            det.imp_neto,
            det.imp_op_ex,
            det.imp_trib,
            det.imp_iva,
            optional_dates,
            det.mon_id,
            det.mon_cotiz,
            condicion_iva,
            caea,
            iva_xml
        ));
    }

    let body = format!(
        r#"{}
<{1}:FeCAEARegInfReq>
<{1}:FeCabReq>
<{1}:CantReg>{2}</{1}:CantReg>
<{1}:PtoVta>{3}</{1}:PtoVta>
<{1}:CbteTipo>{4}</{1}:CbteTipo>
</{1}:FeCabReq>
<{1}:FeDetReq>
{5}
</{1}:FeDetReq>
</{1}:FeCAEARegInfReq>"#,
        auth, PREFIX, cab.cant_reg, cab.pto_vta, cab.cbte_tipo, dets_xml
    );

    create_soap_envelope(NAMESPACE, PREFIX, "FECAEARegInformativo", &body)
}

/// Build the SOAP request for FECompUltimoAutorizado
pub fn build_fe_comp_ultimo_autorizado_req(
    token: &str,
    sign: &str,
    cuit: u64,
    pto_vta: i32,
    cbte_tipo: i32,
) -> String {
    let auth = build_auth_block(PREFIX, token, sign, cuit);
    let body = format!(
        r#"{}
<{1}:PtoVta>{2}</{1}:PtoVta>
<{1}:CbteTipo>{3}</{1}:CbteTipo>"#,
        auth, PREFIX, pto_vta, cbte_tipo
    );
    create_soap_envelope(NAMESPACE, PREFIX, "FECompUltimoAutorizado", &body)
}

/// Build the SOAP request for FECAESolicitar
pub fn build_fecae_solicitar_req(
    token: &str,
    sign: &str,
    cuit: u64,
    cab: &FeCabReq,
    dets: Vec<FeDetReq>,
) -> String {
    let auth = build_auth_block(PREFIX, token, sign, cuit);

    let mut dets_xml = String::new();
    for det in dets {
        let mut iva_xml = String::new();
        if !det.iva.is_empty() {
            iva_xml.push_str(&format!("<{}:Iva>", PREFIX));
            for alic in det.iva {
                iva_xml.push_str(&format!(
                    r#"<{0}:AlicIva>
<{0}:Id>{1}</{0}:Id>
<{0}:BaseImp>{2}</{0}:BaseImp>
<{0}:Importe>{3}</{0}:Importe>
</{0}:AlicIva>"#,
                    PREFIX, alic.id, alic.base_imp, alic.importe
                ));
            }
            iva_xml.push_str(&format!("</{}:Iva>", PREFIX));
        }

        let optional_dates = format!(
            "{}{}{}",
            det.fch_serv_desde
                .map(|s| format!("<{}:FchServDesde>{}</{}:FchServDesde>", PREFIX, s, PREFIX))
                .unwrap_or_default(),
            det.fch_serv_hasta
                .map(|s| format!("<{}:FchServHasta>{}</{}:FchServHasta>", PREFIX, s, PREFIX))
                .unwrap_or_default(),
            det.fch_vto_pago
                .map(|s| format!("<{}:FchVtoPago>{}</{}:FchVtoPago>", PREFIX, s, PREFIX))
                .unwrap_or_default()
        );

        // CondicionIVAReceptorId is mandatory since RG 5616
        let condicion_iva = det
            .condicion_iva_receptor
            .map(|id| format!("<{}:CondicionIVAReceptorId>{}</{}:CondicionIVAReceptorId>", PREFIX, id, PREFIX))
            .unwrap_or_default();

        dets_xml.push_str(&format!(
            r#"<{0}:FECAEDetRequest>
<{0}:Concepto>{1}</{0}:Concepto>
<{0}:DocTipo>{2}</{0}:DocTipo>
<{0}:DocNro>{3}</{0}:DocNro>
<{0}:CbteDesde>{4}</{0}:CbteDesde>
<{0}:CbteHasta>{5}</{0}:CbteHasta>
<{0}:CbteFch>{6}</{0}:CbteFch>
<{0}:ImpTotal>{7}</{0}:ImpTotal>
<{0}:ImpTotConc>{8}</{0}:ImpTotConc>
<{0}:ImpNeto>{9}</{0}:ImpNeto>
<{0}:ImpOpEx>{10}</{0}:ImpOpEx>
<{0}:ImpTrib>{11}</{0}:ImpTrib>
<{0}:ImpIVA>{12}</{0}:ImpIVA>
{13}
<{0}:MonId>{14}</{0}:MonId>
<{0}:MonCotiz>{15}</{0}:MonCotiz>
{16}
{17}
</{0}:FECAEDetRequest>"#,
            PREFIX,
            det.concepto,
            det.doc_tipo,
            det.doc_nro,
            det.cbte_desde,
            det.cbte_hasta,
            det.cbte_fch,
            det.imp_total,
            det.imp_tot_conc,
            det.imp_neto,
            det.imp_op_ex,
            det.imp_trib,
            det.imp_iva,
            optional_dates,
            det.mon_id,
            det.mon_cotiz,
            condicion_iva,
            iva_xml
        ));
    }

    let body = format!(
        r#"{}
<{1}:FeCAEReq>
<{1}:FeCabReq>
<{1}:CantReg>{2}</{1}:CantReg>
<{1}:PtoVta>{3}</{1}:PtoVta>
<{1}:CbteTipo>{4}</{1}:CbteTipo>
</{1}:FeCabReq>
<{1}:FeDetReq>
{5}
</{1}:FeDetReq>
</{1}:FeCAEReq>"#,
        auth, PREFIX, cab.cant_reg, cab.pto_vta, cab.cbte_tipo, dets_xml
    );

    create_soap_envelope(NAMESPACE, PREFIX, "FECAESolicitar", &body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::wsfev1::types::AlicIva;

    #[test]
    fn test_build_fe_dummy_req() {
        let req = build_fe_dummy_req();
        assert!(req.contains("<ar:FEDummy>"));
        assert!(req.contains("</ar:FEDummy>"));
    }

    #[test]
    fn test_build_fe_comp_ultimo_autorizado_req() {
        let req = build_fe_comp_ultimo_autorizado_req("token", "sign", 20111111112, 1, 1);

        assert!(req.contains("<ar:FECompUltimoAutorizado>"));
        assert!(req.contains("<ar:Token>token</ar:Token>"));
        assert!(req.contains("<ar:Sign>sign</ar:Sign>"));
        assert!(req.contains("<ar:Cuit>20111111112</ar:Cuit>"));
        assert!(req.contains("<ar:PtoVta>1</ar:PtoVta>"));
        assert!(req.contains("<ar:CbteTipo>1</ar:CbteTipo>"));
    }

    #[test]
    fn test_build_fe_comp_consultar_req() {
        let req = build_fe_comp_consultar_req("token", "sign", 20111111112, 1, 100, 1);

        assert!(req.contains("<ar:FECompConsultar>"));
        assert!(req.contains("<ar:Token>token</ar:Token>"));
        assert!(req.contains("<ar:Sign>sign</ar:Sign>"));
        assert!(req.contains("<ar:Cuit>20111111112</ar:Cuit>"));
        assert!(req.contains("<ar:CbteTipo>1</ar:CbteTipo>"));
        assert!(req.contains("<ar:CbteNro>100</ar:CbteNro>"));
        assert!(req.contains("<ar:PtoVta>1</ar:PtoVta>"));
    }

    #[test]
    fn test_build_fe_param_get_tipos_cbte_req() {
        let req = build_fe_param_get_tipos_cbte_req("token", "sign", 20111111112);
        assert!(req.contains("<ar:FEParamGetTiposCbte>"));
        assert!(req.contains("<ar:Token>token</ar:Token>"));
    }

    #[test]
    fn test_build_fe_param_get_cotizacion_req() {
        let req = build_fe_param_get_cotizacion_req("token", "sign", 20111111112, "DOL");
        assert!(req.contains("<ar:FEParamGetCotizacion>"));
        assert!(req.contains("<ar:MonId>DOL</ar:MonId>"));
    }

    #[test]
    fn test_build_fecaea_solicitar_req() {
        let req = build_fecaea_solicitar_req("token", "sign", 20111111112, 202501, 1);
        assert!(req.contains("<ar:FECAEASolicitar>"));
        assert!(req.contains("<ar:Periodo>202501</ar:Periodo>"));
        assert!(req.contains("<ar:Orden>1</ar:Orden>"));
    }

    #[test]
    fn test_build_fecaea_consultar_req() {
        let req = build_fecaea_consultar_req("token", "sign", 20111111112, 202501, 2);
        assert!(req.contains("<ar:FECAEAConsultar>"));
        assert!(req.contains("<ar:Periodo>202501</ar:Periodo>"));
        assert!(req.contains("<ar:Orden>2</ar:Orden>"));
    }

    #[test]
    fn test_build_fecaea_sin_movimiento_informar_req() {
        let req = build_fecaea_sin_movimiento_informar_req(
            "token",
            "sign",
            20111111112,
            1,
            "12345678901234",
        );
        assert!(req.contains("<ar:FECAEASinMovimientoInformar>"));
        assert!(req.contains("<ar:PtoVta>1</ar:PtoVta>"));
        assert!(req.contains("<ar:CAEA>12345678901234</ar:CAEA>"));
    }

    #[test]
    fn test_build_fecae_solicitar_req() {
        let cab = FeCabReq {
            cant_reg: 1,
            pto_vta: 1,
            cbte_tipo: 1,
        };
        let iva = vec![AlicIva {
            id: 5,
            base_imp: 100.0,
            importe: 21.0,
        }];
        let det = FeDetReq {
            concepto: 1,
            doc_tipo: 80,
            doc_nro: 20111111112,
            cbte_desde: 1,
            cbte_hasta: 1,
            cbte_fch: "20230101".to_string(),
            imp_total: 121.0,
            imp_tot_conc: 0.0,
            imp_neto: 100.0,
            imp_op_ex: 0.0,
            imp_trib: 0.0,
            imp_iva: 21.0,
            fch_serv_desde: None,
            fch_serv_hasta: None,
            fch_vto_pago: None,
            mon_id: "PES".to_string(),
            mon_cotiz: 1.0,
            condicion_iva_receptor: Some(1), // IVA Responsable Inscripto
            iva,
        };

        let req = build_fecae_solicitar_req("token", "sign", 20111111112, &cab, vec![det]);

        assert!(req.contains("<ar:FECAESolicitar>"));
        assert!(req.contains("<ar:ImpTotal>121</ar:ImpTotal>"));
        assert!(req.contains("<ar:ImpIVA>21</ar:ImpIVA>"));
        assert!(req.contains("<ar:BaseImp>100</ar:BaseImp>"));
        assert!(req.contains("<ar:CondicionIVAReceptorId>1</ar:CondicionIVAReceptorId>"));
    }
}
