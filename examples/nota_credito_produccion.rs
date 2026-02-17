//! Emitir Nota de Crédito C en PRODUCCIÓN
//!
//! CUIDADO: Este ejemplo emite comprobantes REALES con validez fiscal
//!
//! OPENSSL_CONF=openssl_arca.cnf cargo run --example nota_credito_produccion

use arca::{auth, transport, xml, conceptos, condicion_iva, doc_tipos};
use std::fs;
use std::path::Path;

const CUIT: u64 = 20299879624;  // NexTSolucionesInformaticas
const PTO_VTA: i32 = 2;         // Punto de venta habilitado para WebService
const CBTE_TIPO: i32 = 13;      // Nota de Crédito C (Monotributo)

// Comprobante a anular
const CBTE_ASOC_TIPO: i32 = 11;       // Factura C
const CBTE_ASOC_PTO_VTA: i32 = 2;
const CBTE_ASOC_NRO: u64 = 175;
const CBTE_ASOC_CUIT: u64 = 20299879624;
const CBTE_ASOC_FECHA: &str = "20251226";  // Fecha de la factura original

const IMP_TOTAL: f64 = 1.00;  // Mismo importe que la factura original

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
    println!("║     NOTA DE CRÉDITO C EN PRODUCCIÓN                      ║");
    println!("║     CUIT: {} - NexTSolucionesInformaticas      ║", CUIT);
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // Cargar certificados de PRODUCCIÓN
    let cert = fs::read("../certificado/cert_prod.pem")?;
    let key = fs::read("../certificado/key_prod.pem")?;

    let http_client = transport::HttpClient::new()?;

    // URLs de PRODUCCIÓN
    let wsaa_url = "https://wsaa.afip.gov.ar/ws/services/LoginCms";
    let wsfev1_url = "https://servicios1.afip.gov.ar/wsfev1/service.asmx";

    // Login con cache
    let (token, sign) = if let Some(cached) = load_cached_token() {
        println!("1. Usando token cacheado (expira: {})", cached.expiration_time);
        (cached.token, cached.sign)
    } else {
        println!("1. Autenticando con WSAA PRODUCCIÓN...");
        let ltr = auth::build_login_ticket_request("wsfe");
        let cms = auth::sign_cms_base64(&ltr, &cert, &key)?;

        let ta = auth::wsaa_login(&http_client, wsaa_url, &cms).await?;
        save_token_cache(&ta.token, &ta.sign, &ta.expiration_time);
        println!("   ✓ Login exitoso");

        (ta.token.clone(), ta.sign.clone())
    };

    // Obtener último comprobante NC C
    println!("\n2. Consultando última Nota de Crédito C...");
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
    println!("   Última NC C en pto vta {}: {}", PTO_VTA, ultimo);

    let nuevo = (ultimo + 1) as u64;
    println!("   Nueva NC C a emitir: {}\n", nuevo);

    // Preparar Nota de Crédito
    println!("3. Preparando Nota de Crédito C...");
    let fecha_hoy = chrono::Local::now().format("%Y%m%d").to_string();

    println!("   ┌─────────────────────────────────────────────┐");
    println!("   │ NOTA DE CRÉDITO C - ANULACIÓN               │");
    println!("   ├─────────────────────────────────────────────┤");
    println!("   │ Comprobante asociado:                       │");
    println!("   │   Factura C {:05}-{:08}                 │", CBTE_ASOC_PTO_VTA, CBTE_ASOC_NRO);
    println!("   │   Fecha: {}                            │", CBTE_ASOC_FECHA);
    println!("   ├─────────────────────────────────────────────┤");
    println!("   │ TOTAL A ACREDITAR: ${:.2}                    │", IMP_TOTAL);
    println!("   │ Fecha NC: {}                            │", fecha_hoy);
    println!("   └─────────────────────────────────────────────┘\n");

    // Solicitar CAE para NC
    println!("4. Solicitando CAE para Nota de Crédito...");
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
<ar:CantReg>1</ar:CantReg>
<ar:PtoVta>{}</ar:PtoVta>
<ar:CbteTipo>{}</ar:CbteTipo>
</ar:FeCabReq>
<ar:FeDetReq>
<ar:FECAEDetRequest>
<ar:Concepto>{}</ar:Concepto>
<ar:DocTipo>{}</ar:DocTipo>
<ar:DocNro>0</ar:DocNro>
<ar:CbteDesde>{}</ar:CbteDesde>
<ar:CbteHasta>{}</ar:CbteHasta>
<ar:CbteFch>{}</ar:CbteFch>
<ar:ImpTotal>{}</ar:ImpTotal>
<ar:ImpTotConc>0</ar:ImpTotConc>
<ar:ImpNeto>{}</ar:ImpNeto>
<ar:ImpOpEx>0</ar:ImpOpEx>
<ar:ImpTrib>0</ar:ImpTrib>
<ar:ImpIVA>0</ar:ImpIVA>
<ar:MonId>PES</ar:MonId>
<ar:MonCotiz>1</ar:MonCotiz>
<ar:CondicionIVAReceptorId>{}</ar:CondicionIVAReceptorId>
<ar:CbtesAsoc>
<ar:CbteAsoc>
<ar:Tipo>{}</ar:Tipo>
<ar:PtoVta>{}</ar:PtoVta>
<ar:Nro>{}</ar:Nro>
<ar:Cuit>{}</ar:Cuit>
<ar:CbteFch>{}</ar:CbteFch>
</ar:CbteAsoc>
</ar:CbtesAsoc>
</ar:FECAEDetRequest>
</ar:FeDetReq>
</ar:FeCAEReq>
</ar:FECAESolicitar>
</soapenv:Body>
</soapenv:Envelope>"#,
        token, sign, CUIT,
        PTO_VTA, CBTE_TIPO,
        conceptos::PRODUCTOS,
        doc_tipos::CONSUMIDOR_FINAL,
        nuevo, nuevo, fecha_hoy,
        IMP_TOTAL,
        IMP_TOTAL,
        condicion_iva::CONSUMIDOR_FINAL,
        // Comprobante asociado (factura a anular)
        CBTE_ASOC_TIPO,
        CBTE_ASOC_PTO_VTA,
        CBTE_ASOC_NRO,
        CBTE_ASOC_CUIT,
        CBTE_ASOC_FECHA
    );

    let resp = transport::post_soap(&http_client, wsfev1_url, &cae_soap, Some("http://ar.gov.afip.dif.FEV1/FECAESolicitar")).await?;

    // Verificar resultado
    if let Ok(resultado) = xml::extract_tag_text(&resp, "Resultado") {
        if resultado == "R" {
            println!("\n   ✗ AFIP rechazó la solicitud");

            // Extraer todos los errores
            let mut pos = 0;
            while let Some(err_start) = resp[pos..].find("<Err>") {
                let err_start = pos + err_start;
                if let Some(err_end) = resp[err_start..].find("</Err>") {
                    let err_block = &resp[err_start..err_start + err_end + 6];
                    if let (Ok(code), Ok(msg)) = (
                        xml::extract_tag_text(err_block, "Code"),
                        xml::extract_tag_text(err_block, "Msg"),
                    ) {
                        println!("   Error {}: {}", code, msg);
                    }
                    pos = err_start + err_end + 6;
                } else {
                    break;
                }
            }

            // Mostrar observaciones también
            let mut pos = 0;
            while let Some(obs_start) = resp[pos..].find("<Obs>") {
                let obs_start = pos + obs_start;
                if let Some(obs_end) = resp[obs_start..].find("</Obs>") {
                    let obs_block = &resp[obs_start..obs_start + obs_end + 6];
                    if let (Ok(code), Ok(msg)) = (
                        xml::extract_tag_text(obs_block, "Code"),
                        xml::extract_tag_text(obs_block, "Msg"),
                    ) {
                        println!("   Obs {}: {}", code, msg);
                    }
                    pos = obs_start + obs_end + 6;
                } else {
                    break;
                }
            }

            return Err(anyhow::anyhow!("NC rejected by AFIP"));
        }
    }

    match xml::extract_tag_text(&resp, "CAE") {
        Ok(cae) if !cae.is_empty() => {
            let vto = xml::extract_tag_text(&resp, "CAEFchVto").unwrap_or_default();
            println!("\n   ╔═══════════════════════════════════════════════╗");
            println!("   ║     ✓ NOTA DE CRÉDITO EMITIDA                  ║");
            println!("   ╠═══════════════════════════════════════════════╣");
            println!("   ║  CAE: {}                  ║", cae);
            println!("   ║  Vencimiento: {}                       ║", vto);
            println!("   ║  NC C: {:05}-{:08}                     ║", PTO_VTA, nuevo);
            println!("   ║  Anula Factura C: {:05}-{:08}          ║", CBTE_ASOC_PTO_VTA, CBTE_ASOC_NRO);
            println!("   ╚═══════════════════════════════════════════════╝");
        }
        _ => {
            println!("\n   Respuesta AFIP:");
            println!("{}", &resp[..resp.len().min(2000)]);
            return Err(anyhow::anyhow!("No CAE in response"));
        }
    }

    println!("\n=== Nota de Crédito emitida exitosamente en PRODUCCIÓN ===");
    Ok(())
}
