use arca::{auth, transport, PadronA13Service};
use std::fs;

const CUIT: u64 = 20123456781;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Test Padrón A13 - Homologación ===\n");

    let cert = fs::read("../certificado/cert.pem")?;
    let key = fs::read("../certificado/key.pem")?;
    let http = transport::HttpClient::new()?;

    // WSAA homologación
    println!("1. Autenticando...");
    let ltr = auth::build_login_ticket_request("ws_sr_padron_a13");
    let cms = auth::sign_cms_base64(&ltr, &cert, &key)?;
    let ta = auth::wsaa_login(&http, "https://wsaahomo.afip.gov.ar/ws/services/LoginCms", &cms).await?;
    println!("   ✓ Token obtenido");

    // Padrón homologación
    let padron = PadronA13Service::testing();

    println!("\n2. Dummy...");
    let dummy = padron.dummy(&http).await?;
    println!("   App: {}, DB: {}", dummy.app_server, dummy.db_server);

    println!("\n3. Consultando CUIT {}...", CUIT);
    let result = padron.get_persona(&http, &ta.token, &ta.sign, CUIT, CUIT).await?;
    let p = &result.persona;

    println!("\n   CUIT: {}", p.id_persona);
    println!("   Nombre: {}", p.nombre_completo());
    println!("   Tipo: {:?}", p.tipo_persona);
    println!("   Estado: {:?}", p.estado_clave);
    if let Some(ref act) = p.descripcion_actividad_principal {
        println!("   Actividad: {}", act);
    }
    if let Some(dom) = p.domicilio_fiscal() {
        if let Some(ref dir) = dom.direccion {
            println!("   Domicilio: {}", dir);
        }
        if let Some(ref prov) = dom.descripcion_provincia {
            println!("   Provincia: {}", prov);
        }
    }

    println!("\n=== OK ===");
    Ok(())
}
