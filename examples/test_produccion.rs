//! Test de producción AFIP
//!
//! Emite una Factura C de $1 en producción.
//!
//! Ejecutar con:
//! cargo run --example test_produccion

use arca::{
    FeCabReq, FeDetReq, WsFev1Service, cbte_tipos, condicion_iva, doc_tipos,
    transport::HttpClient, auth,
};
use std::fs;
use std::path::Path;

const CUIT: u64 = 20299879624;  // CUIT del certificado de producción
const PTO_VTA: i32 = 2;  // Punto de venta 2
const TOKEN_CACHE_FILE: &str = "/tmp/arca_token_cache_prod.json";

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

/// Convertir PFX a PEM usando OpenSSL CLI
fn convert_pfx_to_pem(pfx_data: &[u8], password: &str) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    use std::io::Write;
    use std::process::Command;
    use tempfile::NamedTempFile;

    let mut pfx_file = NamedTempFile::new()?;
    pfx_file.write_all(pfx_data)?;
    let pfx_path = pfx_file.path().to_str().unwrap();

    // Extraer certificado
    let cert_output = Command::new("openssl")
        .args([
            "pkcs12",
            "-in", pfx_path,
            "-clcerts",
            "-nokeys",
            "-passin", &format!("pass:{}", password),
            "-provider", "legacy",
            "-provider", "default",
        ])
        .output()?;

    if !cert_output.status.success() {
        let stderr = String::from_utf8_lossy(&cert_output.stderr);
        anyhow::bail!("Error extrayendo certificado: {}", stderr);
    }

    // Extraer clave privada
    let key_output = Command::new("openssl")
        .args([
            "pkcs12",
            "-in", pfx_path,
            "-nocerts",
            "-nodes",
            "-passin", &format!("pass:{}", password),
            "-provider", "legacy",
            "-provider", "default",
        ])
        .output()?;

    if !key_output.status.success() {
        let stderr = String::from_utf8_lossy(&key_output.stderr);
        anyhow::bail!("Error extrayendo clave: {}", stderr);
    }

    Ok((cert_output.stdout, key_output.stdout))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          TEST DE PRODUCCIÓN - FACTURA C $1                   ║");
    println!("║              ⚠️  AMBIENTE REAL                                ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // =========================================================================
    // 1. Cargar certificado PFX de producción
    // =========================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("1. CARGANDO CERTIFICADO DE PRODUCCIÓN");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let pfx_path = "../certificado/CertificadoProduccion.pfx";
    let pfx_data = match fs::read(pfx_path) {
        Ok(data) => {
            println!("   ✓ Certificado PFX cargado: {}", pfx_path);
            data
        }
        Err(e) => {
            println!("   ✗ Error cargando certificado: {}", e);
            return Ok(());
        }
    };

    // Convertir PFX a PEM para firmar
    println!("   Convirtiendo PFX a PEM...");
    let (cert_pem, key_pem) = match convert_pfx_to_pem(&pfx_data, "") {
        Ok((c, k)) => {
            println!("   ✓ Certificado convertido correctamente");
            (c, k)
        }
        Err(e) => {
            println!("   ✗ Error convirtiendo PFX: {}", e);
            return Ok(());
        }
    };

    let http = HttpClient::new()?;

    // =========================================================================
    // 2. Autenticación WSAA
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("2. AUTENTICACIÓN WSAA (PRODUCCIÓN)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("   Entorno: PRODUCCIÓN");
    println!("   CUIT: {}", CUIT);
    println!("   Punto de Venta: {}", PTO_VTA);

    // Intentar usar token cacheado primero
    let (token, sign) = if let Some(cached) = load_cached_token() {
        println!("   Usando token cacheado (expira: {})", cached.expiration_time);
        (cached.token, cached.sign)
    } else {
        println!("   Solicitando nuevo token...");

        let wsaa_url = "https://wsaa.afip.gov.ar/ws/services/LoginCms";
        let ltr = auth::build_login_ticket_request("wsfe");
        let cms = auth::sign_cms_base64(&ltr, &cert_pem, &key_pem)?;

        match auth::wsaa_login(&http, wsaa_url, &cms).await {
            Ok(ta) => {
                save_token_cache(&ta.token, &ta.sign, &ta.expiration_time);
                println!("   ✓ Token obtenido y cacheado");
                (ta.token.clone(), ta.sign.clone())
            }
            Err(e) => {
                println!("   ✗ Error de autenticación: {}", e);
                return Ok(());
            }
        }
    };

    println!("   ✓ Autenticación exitosa");

    // Crear servicio WSFEv1 para producción
    let wsfev1 = WsFev1Service::new(
        "https://servicios1.afip.gov.ar/wsfev1/service.asmx".to_string(),
        http.clone(),
    );

    // =========================================================================
    // 3. FE Dummy (verificar conectividad)
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("3. VERIFICAR CONECTIVIDAD (FE DUMMY)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    match wsfev1.fe_dummy().await {
        Ok((app, auth_srv, db)) => {
            println!("   App Server: {}", app);
            println!("   Auth Server: {}", auth_srv);
            println!("   DB Server: {}", db);
            if app == "OK" && auth_srv == "OK" && db == "OK" {
                println!("   ✓ Servidores AFIP operativos");
            } else {
                println!("   ⚠ Algunos servidores pueden tener problemas");
            }
        }
        Err(e) => {
            println!("   ✗ Error: {}", e);
            return Ok(());
        }
    }

    // =========================================================================
    // 4. Obtener último comprobante
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("4. ÚLTIMO COMPROBANTE AUTORIZADO");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let cbte_tipo = cbte_tipos::FACTURA_C; // 11
    let ultimo = match wsfev1.fe_comp_ultimo_autorizado(&token, &sign, CUIT, PTO_VTA, cbte_tipo).await {
        Ok(n) => {
            println!("   Tipo: Factura C ({})", cbte_tipo);
            println!("   Punto de Venta: {}", PTO_VTA);
            println!("   Último autorizado: {}", n);
            n
        }
        Err(e) => {
            println!("   ✗ Error: {}", e);
            return Ok(());
        }
    };

    // =========================================================================
    // 5. Emitir Factura C de $1
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("5. EMITIR FACTURA C - $1.00");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let nuevo = (ultimo + 1) as u64;
    let fecha_hoy = chrono::Local::now().format("%Y%m%d").to_string();

    let cab = FeCabReq {
        cant_reg: 1,
        pto_vta: PTO_VTA,
        cbte_tipo,
    };

    // Factura C para Monotributo: No lleva IVA discriminado
    // imp_total = imp_neto (todo en neto)
    let det = FeDetReq {
        concepto: 1, // Productos
        doc_tipo: doc_tipos::CONSUMIDOR_FINAL, // 99
        doc_nro: 0,
        cbte_desde: nuevo,
        cbte_hasta: nuevo,
        cbte_fch: fecha_hoy.clone(),
        imp_total: 1.0,      // Total $1
        imp_tot_conc: 0.0,   // No gravado
        imp_neto: 1.0,       // Neto = Total (no hay IVA en Factura C)
        imp_op_ex: 0.0,      // Exento
        imp_trib: 0.0,       // Tributos
        imp_iva: 0.0,        // Sin IVA (Monotributo)
        fch_serv_desde: None,
        fch_serv_hasta: None,
        fch_vto_pago: None,
        mon_id: "PES".to_string(),
        mon_cotiz: 1.0,
        condicion_iva_receptor: Some(condicion_iva::CONSUMIDOR_FINAL), // Requerido desde 2025
        iva: vec![],         // Sin alícuotas IVA
    };

    println!("   Comprobante: Factura C {}-{}",
        format!("{:05}", PTO_VTA),
        format!("{:08}", nuevo)
    );
    println!("   Fecha: {}", fecha_hoy);
    println!("   Total: $1.00");
    println!("   Cliente: Consumidor Final");
    println!();
    println!("   Emitiendo...");

    match wsfev1.fecae_solicitar_detallado(&token, &sign, CUIT, cab, vec![det]).await {
        Ok(resp) => {
            println!();
            println!("   Resultado: {}", resp.resultado);

            // Mostrar errores si hay
            if !resp.detalles.errors.is_empty() {
                println!("\n   ERRORES:");
                for err in &resp.detalles.errors {
                    println!("   - [{}] {}", err.code, err.msg);
                }
            }

            // Mostrar observaciones si hay
            if !resp.detalles.observaciones.is_empty() {
                println!("\n   OBSERVACIONES:");
                for obs in &resp.detalles.observaciones {
                    println!("   - [{}] {}", obs.code, obs.msg);
                }
            }

            // Mostrar eventos si hay
            if !resp.detalles.events.is_empty() {
                println!("\n   EVENTOS:");
                for ev in &resp.detalles.events {
                    println!("   - [{}] {}", ev.code, ev.msg);
                }
            }

            if resp.resultado == "A" {
                println!();
                println!("   ╔═══════════════════════════════════════════════════════╗");
                println!("   ║              FACTURA AUTORIZADA                       ║");
                println!("   ╠═══════════════════════════════════════════════════════╣");
                println!("   ║  Tipo:        Factura C                               ║");
                println!("   ║  Número:      {}-{}                    ║",
                    format!("{:05}", PTO_VTA),
                    format!("{:08}", nuevo)
                );
                println!("   ║  CAE:         {}                   ║", resp.cae);
                println!("   ║  Vencimiento: {}                           ║", resp.cae_fch_vto);
                println!("   ║  Total:       $1.00                                   ║");
                println!("   ╚═══════════════════════════════════════════════════════╝");
                println!();
                println!("   ✓ FACTURA EMITIDA EXITOSAMENTE EN PRODUCCIÓN");
            } else {
                println!();
                println!("   ✗ Factura rechazada por AFIP");
            }
        }
        Err(e) => {
            println!();
            println!("   ✗ Error emitiendo factura: {}", e);
            return Ok(());
        }
    }

    // =========================================================================
    // 6. Consultar comprobante emitido
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("6. VERIFICAR COMPROBANTE EMITIDO");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    match wsfev1.fe_comp_consultar(&token, &sign, CUIT, cbte_tipo, nuevo, PTO_VTA).await {
        Ok(comp) => {
            println!("   Comprobante encontrado en AFIP:");
            println!("   - Número: {}-{}", format!("{:05}", PTO_VTA), format!("{:08}", nuevo));
            println!("   - Resultado: {}", comp.resultado);
            println!("   - CAE: {}", comp.cod_autorizacion);
            println!("   - Vencimiento CAE: {}", comp.fch_vto);
            println!("   - Total: ${:.2}", comp.imp_total);
            println!();
            println!("   ✓ Comprobante verificado correctamente");
        }
        Err(e) => {
            println!("   ✗ Error consultando: {}", e);
        }
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║              TEST DE PRODUCCIÓN COMPLETADO                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
