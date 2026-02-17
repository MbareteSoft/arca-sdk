//! Consultar puntos de venta habilitados
//!
//! OPENSSL_CONF=openssl_arca.cnf cargo run --example consultar_ptos_venta

use arca::{auth, transport, xml};
use std::fs;
use std::path::Path;

const CUIT: u64 = 20299879624;

const TOKEN_CACHE_FILE: &str = "/tmp/arca_prod_token_cache.json";

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
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║     CONSULTAR PUNTOS DE VENTA - PRODUCCIÓN               ║");
    println!("║     CUIT: {}                                   ║", CUIT);
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let cert = fs::read("../certificado/cert_prod.pem")?;
    let key = fs::read("../certificado/key_prod.pem")?;

    let http_client = transport::HttpClient::new()?;

    let wsaa_url = "https://wsaa.afip.gov.ar/ws/services/LoginCms";
    let wsfev1_url = "https://servicios1.afip.gov.ar/wsfev1/service.asmx";

    // Login
    let (token, sign) = if let Some(cached) = load_cached_token() {
        println!("Usando token cacheado (expira: {})", cached.expiration_time);
        (cached.token, cached.sign)
    } else {
        println!("Autenticando con WSAA PRODUCCIÓN...");
        let ltr = auth::build_login_ticket_request("wsfe");
        let cms = auth::sign_cms_base64(&ltr, &cert, &key)?;
        let ta = auth::wsaa_login(&http_client, wsaa_url, &cms).await?;
        save_token_cache(&ta.token, &ta.sign, &ta.expiration_time);
        println!("Login exitoso");
        (ta.token.clone(), ta.sign.clone())
    };

    // Consultar puntos de venta
    println!("\nConsultando puntos de venta habilitados...\n");

    let soap = format!(
        r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:ar="http://ar.gov.afip.dif.FEV1/">
<soapenv:Header/>
<soapenv:Body>
<ar:FEParamGetPtosVenta>
<ar:Auth>
<ar:Token>{}</ar:Token>
<ar:Sign>{}</ar:Sign>
<ar:Cuit>{}</ar:Cuit>
</ar:Auth>
</ar:FEParamGetPtosVenta>
</soapenv:Body>
</soapenv:Envelope>"#,
        token, sign, CUIT
    );

    let resp = transport::post_soap(&http_client, wsfev1_url, &soap, Some("http://ar.gov.afip.dif.FEV1/FEParamGetPtosVenta")).await?;

    // Mostrar respuesta formateada
    if resp.contains("<PtoVta>") {
        println!("Puntos de venta encontrados:\n");

        let mut pos = 0;
        while let Some(start) = resp[pos..].find("<PtoVta>") {
            let start = pos + start;
            if let Some(end) = resp[start..].find("</PtoVta>") {
                let pv_block = &resp[start..start + end + 9];

                let nro = xml::extract_tag_text(pv_block, "Nro").unwrap_or_default();
                let emision_tipo = xml::extract_tag_text(pv_block, "EmisionTipo").unwrap_or_default();
                let bloqueado = xml::extract_tag_text(pv_block, "Bloqueado").unwrap_or_default();
                let fch_baja = xml::extract_tag_text(pv_block, "FchBaja").unwrap_or_else(|_| "Activo".to_string());

                let tipo_desc = match emision_tipo.as_str() {
                    "CAE" => "WebService (RECE)",
                    "CAEA" => "CAEA",
                    _ => &emision_tipo,
                };

                println!("  Punto de Venta: {}", nro);
                println!("  Tipo Emisión:   {} ({})", emision_tipo, tipo_desc);
                println!("  Bloqueado:      {}", if bloqueado == "N" { "No" } else { "Sí" });
                println!("  Estado:         {}", fch_baja);
                println!();

                pos = start + end + 9;
            } else {
                break;
            }
        }
    } else if resp.contains("<Errors>") {
        println!("Error consultando puntos de venta:");
        if let Ok(msg) = xml::extract_tag_text(&resp, "Msg") {
            println!("  {}", msg);
        }
    } else {
        println!("No se encontraron puntos de venta habilitados");
        println!("\nRespuesta AFIP:");
        println!("{}", &resp[..resp.len().min(2000)]);
    }

    Ok(())
}
