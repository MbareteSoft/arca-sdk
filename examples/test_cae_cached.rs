//! Test CAE with token caching
//!
//! This example caches the token to avoid AFIP rate limits in homologación
//!
//! cargo run --example test_cae_cached

use arca::{auth, transport, xml, AlicIva, FeCabReq, FeDetReq};
use std::fs;
use std::path::Path;

const CUIT: u64 = 20123456781;
const PTO_VTA: i32 = 1;
const CBTE_TIPO: i32 = 6; // Factura B

const TOKEN_CACHE_FILE: &str = "/tmp/arca_token_cache.json";

#[derive(serde::Serialize, serde::Deserialize)]
struct CachedToken {
    token: String,
    sign: String,
    expiration_time: String,
}

fn load_cached_token() -> Option<CachedToken> {
    if Path::new(TOKEN_CACHE_FILE).exists() {
        if let Ok(content) = fs::read_to_string(TOKEN_CACHE_FILE) {
            if let Ok(cached) = serde_json::from_str::<CachedToken>(&content) {
                // Check if not expired (simple string comparison)
                let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
                if cached.expiration_time > now {
                    return Some(cached);
                }
            }
        }
    }
    None
}

fn save_token_cache(token: &str, sign: &str, expiration_time: &str) {
    let cached = CachedToken {
        token: token.to_string(),
        sign: sign.to_string(),
        expiration_time: expiration_time.to_string(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&cached) {
        let _ = fs::write(TOKEN_CACHE_FILE, json);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Test CAE con Cache de Token ===\n");

    // Load certs
    let cert = fs::read("../certificado/cert.pem")?;
    let key = fs::read("../certificado/key.pem")?;

    let http_client = transport::HttpClient::new()?;
    let wsaa_url = "https://wsaahomo.afip.gov.ar/ws/services/LoginCms";
    let wsfev1_url = "https://wswhomo.afip.gov.ar/wsfev1/service.asmx";

    // Try to load cached token
    let (token, sign) = if let Some(cached) = load_cached_token() {
        println!("1. Usando token cacheado (expira: {})", cached.expiration_time);
        (cached.token, cached.sign)
    } else {
        println!("1. Autenticando con WSAA...");
        let ltr = auth::build_login_ticket_request("wsfe");
        let cms = auth::sign_cms_base64(&ltr, &cert, &key)?;

        let ta = auth::wsaa_login(&http_client, wsaa_url, &cms).await?;

        // Save to cache
        save_token_cache(&ta.token, &ta.sign, &ta.expiration_time);
        println!("   ✓ Login exitoso, token cacheado");

        (ta.token.clone(), ta.sign.clone())
    };

    // Get last voucher
    println!("\n2. Consultando último comprobante...");
    let ultimo_soap = format!(
        r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:ar="http://ar.gov.afip.dif.FEV1/">
<soapenv:Header/>
<soapenv:Body>
<ar:FECompUltimoAutorizado>
<ar:Auth>
<ar:Token>{}</ar:Token>
<ar:Sign>{}</ar:Sign>
<ar:Cuit>{}</ar:Cuit>
</ar:Auth>
<ar:PtoVta>{}</ar:PtoVta>
<ar:CbteTipo>{}</ar:CbteTipo>
</ar:FECompUltimoAutorizado>
</soapenv:Body>
</soapenv:Envelope>"#,
        token, sign, CUIT, PTO_VTA, CBTE_TIPO
    );

    let resp = transport::post_soap(&http_client, wsfev1_url, &ultimo_soap, Some("http://ar.gov.afip.dif.FEV1/FECompUltimoAutorizado")).await?;

    let ultimo: i32 = xml::extract_tag_text(&resp, "CbteNro")?.parse()?;
    println!("   Último comprobante tipo {} en pto vta {}: {}", CBTE_TIPO, PTO_VTA, ultimo);

    let nuevo = (ultimo + 1) as u64;
    println!("   Nuevo comprobante: {}\n", nuevo);

    // Prepare invoice data
    println!("3. Preparando factura...");
    let fecha_hoy = chrono::Local::now().format("%Y%m%d").to_string();

    let cab = FeCabReq {
        cant_reg: 1,
        pto_vta: PTO_VTA,
        cbte_tipo: CBTE_TIPO,
    };

    let det = FeDetReq {
        concepto: 1,
        doc_tipo: 99, // Consumidor Final
        doc_nro: 0,
        cbte_desde: nuevo,
        cbte_hasta: nuevo,
        cbte_fch: fecha_hoy.clone(),
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
        condicion_iva_receptor: Some(5), // Consumidor Final
        iva: vec![AlicIva {
            id: 5,
            base_imp: 100.0,
            importe: 21.0,
        }],
    };

    println!("   Total: ${:.2}, Neto: ${:.2}, IVA: ${:.2}\n", det.imp_total, det.imp_neto, det.imp_iva);

    // Build CAE request
    println!("4. Solicitando CAE...");
    let cae_soap = format!(
        r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:ar="http://ar.gov.afip.dif.FEV1/">
<soapenv:Header/>
<soapenv:Body>
<ar:FECAESolicitar>
<ar:Auth>
<ar:Token>{}</ar:Token>
<ar:Sign>{}</ar:Sign>
<ar:Cuit>{}</ar:Cuit>
</ar:Auth>
<ar:FeCAEReq>
<ar:FeCabReq>
<ar:CantReg>{}</ar:CantReg>
<ar:PtoVta>{}</ar:PtoVta>
<ar:CbteTipo>{}</ar:CbteTipo>
</ar:FeCabReq>
<ar:FeDetReq>
<ar:FECAEDetRequest>
<ar:Concepto>{}</ar:Concepto>
<ar:DocTipo>{}</ar:DocTipo>
<ar:DocNro>{}</ar:DocNro>
<ar:CbteDesde>{}</ar:CbteDesde>
<ar:CbteHasta>{}</ar:CbteHasta>
<ar:CbteFch>{}</ar:CbteFch>
<ar:ImpTotal>{}</ar:ImpTotal>
<ar:ImpTotConc>{}</ar:ImpTotConc>
<ar:ImpNeto>{}</ar:ImpNeto>
<ar:ImpOpEx>{}</ar:ImpOpEx>
<ar:ImpTrib>{}</ar:ImpTrib>
<ar:ImpIVA>{}</ar:ImpIVA>
<ar:MonId>{}</ar:MonId>
<ar:MonCotiz>{}</ar:MonCotiz>
<ar:CondicionIVAReceptorId>{}</ar:CondicionIVAReceptorId>
<ar:Iva>
<ar:AlicIva>
<ar:Id>{}</ar:Id>
<ar:BaseImp>{}</ar:BaseImp>
<ar:Importe>{}</ar:Importe>
</ar:AlicIva>
</ar:Iva>
</ar:FECAEDetRequest>
</ar:FeDetReq>
</ar:FeCAEReq>
</ar:FECAESolicitar>
</soapenv:Body>
</soapenv:Envelope>"#,
        token, sign, CUIT,
        cab.cant_reg, cab.pto_vta, cab.cbte_tipo,
        det.concepto, det.doc_tipo, det.doc_nro,
        det.cbte_desde, det.cbte_hasta, det.cbte_fch,
        det.imp_total, det.imp_tot_conc, det.imp_neto,
        det.imp_op_ex, det.imp_trib, det.imp_iva,
        det.mon_id, det.mon_cotiz,
        det.condicion_iva_receptor.unwrap_or(5),
        det.iva[0].id, det.iva[0].base_imp, det.iva[0].importe
    );

    let resp = transport::post_soap(&http_client, wsfev1_url, &cae_soap, Some("http://ar.gov.afip.dif.FEV1/FECAESolicitar")).await?;

    // Parse response
    if let Ok(resultado) = xml::extract_tag_text(&resp, "Resultado") {
        if resultado == "R" {
            let code = xml::extract_tag_text(&resp, "Code").unwrap_or_default();
            let msg = xml::extract_tag_text(&resp, "Msg").unwrap_or_default();
            println!("\n   ✗ AFIP rechazó la solicitud");
            println!("   Código: {}", code);
            println!("   Mensaje: {}", msg);
            return Err(anyhow::anyhow!("CAE rejected: {} - {}", code, msg));
        }
    }

    match xml::extract_tag_text(&resp, "CAE") {
        Ok(cae) if !cae.is_empty() => {
            let vto = xml::extract_tag_text(&resp, "CAEFchVto").unwrap_or_default();
            println!("\n   ✓ CAE OBTENIDO EXITOSAMENTE");
            println!("   ================================");
            println!("   CAE: {}", cae);
            println!("   Vencimiento: {}", vto);
            println!("   ================================");
        }
        _ => {
            println!("\n   Respuesta completa:");
            println!("{}", &resp[..resp.len().min(2000)]);
            return Err(anyhow::anyhow!("No CAE in response"));
        }
    }

    println!("\n=== Test completado ===");
    Ok(())
}
