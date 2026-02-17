//! Test de autenticación PFX en Homologación y Producción

use arca::{auth, transport::HttpClient};

const WSAA_HOMO: &str = "https://wsaahomo.afip.gov.ar/ws/services/LoginCms";
const WSAA_PROD: &str = "https://wsaa.afip.gov.ar/ws/services/LoginCms";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let http = HttpClient::new()?;

    // ========================================
    // TEST 1: HOMOLOGACIÓN
    // ========================================
    println!("=== TEST HOMOLOGACIÓN ===\n");

    let pfx_homo = std::fs::read("../certificado/certificado2024GcHomologacion.pfx")?;
    println!("PFX Homologación: {} bytes", pfx_homo.len());

    let ltr = auth::build_login_ticket_request("wsfe");
    let cms = auth::sign_cms_base64_from_pfx(&ltr, &pfx_homo, "")?;
    println!("CMS firmado: {} chars", cms.len());

    let ticket_homo = auth::wsaa_login(&http, WSAA_HOMO, &cms).await?;
    println!("✓ HOMOLOGACIÓN OK");
    println!("  Token: {}...", &ticket_homo.token[..40]);
    println!("  Expira: {}\n", ticket_homo.expiration_time);

    // ========================================
    // TEST 2: PRODUCCIÓN
    // ========================================
    println!("=== TEST PRODUCCIÓN ===\n");

    let pfx_prod = std::fs::read("../certificado/CertificadoProduccion.pfx")?;
    println!("PFX Producción: {} bytes", pfx_prod.len());

    let ltr = auth::build_login_ticket_request("wsfe");

    // El PFX de producción puede tener password - probar sin password primero
    let cms = match auth::sign_cms_base64_from_pfx(&ltr, &pfx_prod, "") {
        Ok(c) => c,
        Err(e) => {
            println!("⚠ PFX requiere password: {}", e);
            println!("  Probando con password común...");
            // Intentar con passwords comunes o pedir al usuario
            return Err(anyhow::anyhow!("PFX de producción requiere password"));
        }
    };
    println!("CMS firmado: {} chars", cms.len());

    let ticket_prod = auth::wsaa_login(&http, WSAA_PROD, &cms).await?;
    println!("✓ PRODUCCIÓN OK");
    println!("  Token: {}...", &ticket_prod.token[..40]);
    println!("  Expira: {}\n", ticket_prod.expiration_time);

    // ========================================
    // RESUMEN
    // ========================================
    println!("========================================");
    println!("✓ Homologación: OK");
    println!("✓ Producción:   OK");
    println!("========================================");

    Ok(())
}
