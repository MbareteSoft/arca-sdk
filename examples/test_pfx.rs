//! Test completo con PFX y cache de token
//!
//! cargo run --example test_pfx

use arca::auth::{self, TicketAcceso};
use arca::transport::HttpClient;
use arca::services::WsFev1Service;
use arca::{FeCabReq, FeDetReq};
use std::io::Write;
use std::process::{Command, Stdio};

const CUIT: u64 = 20123456781;
const PTO_VTA: i32 = 1;
const CBTE_FACTURA_C: i32 = 11;

const PFX_PATH: &str = "../certificado/certificado2024GcHomologacion.pfx";
const PFX_PASSWORD: &str = "";
const TOKEN_CACHE: &str = "/tmp/arca_token_wsfe.json";

const WSAA_URL: &str = "https://wsaahomo.afip.gov.ar/ws/services/LoginCms";
const WSFEV1_URL: &str = "https://wswhomo.afip.gov.ar/wsfev1/service.asmx";

// ============================================================================
// PFX -> PEM extraction
// ============================================================================

fn extract_pem_from_pfx(pfx_path: &str, password: &str) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    let pfx_data = std::fs::read(pfx_path)?;
    let mut pfx_file = tempfile::NamedTempFile::new()?;
    pfx_file.write_all(&pfx_data)?;
    let path = pfx_file.path().to_str().unwrap().to_string();

    let cert = run_openssl(&[
        "pkcs12", "-in", &path, "-clcerts", "-nokeys",
        "-passin", "stdin", "-provider", "legacy", "-provider", "default",
    ], password)?;

    let key = run_openssl(&[
        "pkcs12", "-in", &path, "-nocerts", "-nodes",
        "-passin", "stdin", "-provider", "legacy", "-provider", "default",
    ], password)?;

    Ok((cert, key))
}

fn run_openssl(args: &[&str], password: &str) -> anyhow::Result<Vec<u8>> {
    let mut child = Command::new("openssl")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(password.as_bytes())?;
        stdin.write_all(b"\n")?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        anyhow::bail!("OpenSSL failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(output.stdout)
}

// ============================================================================
// Token cache
// ============================================================================

fn load_cached_token() -> Option<TicketAcceso> {
    let content = std::fs::read_to_string(TOKEN_CACHE).ok()?;
    let token: TicketAcceso = serde_json::from_str(&content).ok()?;
    if token.is_expired() {
        println!("   Token cacheado expirado, solicitando nuevo...");
        None
    } else {
        Some(token)
    }
}

fn save_token(token: &TicketAcceso) {
    if let Ok(json) = serde_json::to_string_pretty(token) {
        let _ = std::fs::write(TOKEN_CACHE, json);
    }
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== TEST PFX CON CACHE DE TOKEN ===\n");

    // 1. Extract PEM from PFX
    println!("1. Extrayendo PEM del PFX...");
    let (cert, key) = extract_pem_from_pfx(PFX_PATH, PFX_PASSWORD)?;
    println!("   Cert: {} bytes, Key: {} bytes", cert.len(), key.len());

    let http = HttpClient::new()?;

    // 2. Login (con cache)
    let (token, sign) = if let Some(cached) = load_cached_token() {
        println!("\n2. Usando token cacheado (expira: {})", cached.expiration_time);
        (cached.token.clone(), cached.sign.clone())
    } else {
        println!("\n2. Autenticando con WSAA...");
        let ltr = auth::build_login_ticket_request("wsfe");
        let cms = auth::sign_cms_base64(&ltr, &cert, &key)?;
        let ta = auth::wsaa_login(&http, WSAA_URL, &cms).await?;
        println!("   Token obtenido, expira: {}", ta.expiration_time);
        save_token(&ta);
        println!("   Token guardado en {}", TOKEN_CACHE);
        (ta.token.clone(), ta.sign.clone())
    };

    // 3. FE Dummy
    println!("\n3. FE Dummy...");
    let wsfev1 = WsFev1Service::new(WSFEV1_URL.to_string(), http.clone());
    let (app, auth_srv, db) = wsfev1.fe_dummy().await?;
    println!("   App: {}, Auth: {}, DB: {}", app, auth_srv, db);

    // 4. Ultimo comprobante
    println!("\n4. Ultimo comprobante Factura C...");
    let ultimo = wsfev1.fe_comp_ultimo_autorizado(&token, &sign, CUIT, PTO_VTA, CBTE_FACTURA_C).await?;
    println!("   Ultimo: {}", ultimo);

    let nuevo = (ultimo + 1) as u64;
    println!("   Proximo: {}", nuevo);

    // 5. Solicitar CAE - Factura C $1
    println!("\n5. Solicitando CAE - Factura C $1...");
    let fecha = chrono::Local::now().format("%Y%m%d").to_string();

    let cab = FeCabReq {
        cant_reg: 1,
        pto_vta: PTO_VTA,
        cbte_tipo: CBTE_FACTURA_C,
    };

    let det = FeDetReq {
        concepto: 1,
        doc_tipo: 99,
        doc_nro: 0,
        cbte_desde: nuevo,
        cbte_hasta: nuevo,
        cbte_fch: fecha,
        imp_total: 1.0,
        imp_tot_conc: 0.0,
        imp_neto: 1.0,
        imp_op_ex: 0.0,
        imp_trib: 0.0,
        imp_iva: 0.0,
        fch_serv_desde: None,
        fch_serv_hasta: None,
        fch_vto_pago: None,
        mon_id: "PES".to_string(),
        mon_cotiz: 1.0,
        condicion_iva_receptor: Some(5),
        iva: vec![],
    };

    let resp = wsfev1.fecae_solicitar_detallado(&token, &sign, CUIT, cab, vec![det]).await?;

    if resp.is_approved() {
        println!("   CAE: {}", resp.cae);
        println!("   Vencimiento: {}", resp.cae_fch_vto);
        println!("   Resultado: APROBADO");
    } else {
        println!("   Resultado: {}", resp.resultado);
        for err in &resp.detalles.errors {
            println!("   Error [{}]: {}", err.code, err.msg);
        }
        for obs in &resp.detalles.observaciones {
            println!("   Obs [{}]: {}", obs.code, obs.msg);
        }
    }

    // 6. Consultar el comprobante emitido
    println!("\n6. Consultando comprobante emitido...");
    let comp = wsfev1.fe_comp_consultar(&token, &sign, CUIT, CBTE_FACTURA_C, nuevo, PTO_VTA).await?;
    println!("   CAE: {}", comp.cod_autorizacion);
    println!("   Total: ${}", comp.imp_total);
    println!("   Resultado: {}", comp.resultado);

    println!("\n=== TEST COMPLETADO ===");
    Ok(())
}
