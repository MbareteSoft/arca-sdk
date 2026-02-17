//! Consultar Padrón Alcance 13 - PRODUCCIÓN
//!
//! OPENSSL_CONF=openssl_arca.cnf cargo run --example consultar_padron -- 20299879624

use arca::{auth, transport, PadronA13Service};
use std::env;
use std::fs;
use std::path::Path;

const CUIT_REPRESENTADA: u64 = 20299879624;  // Tu CUIT
const TOKEN_CACHE_FILE: &str = "/tmp/arca_padron_token_cache.json";

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
    // Obtener CUIT a consultar del argumento
    let args: Vec<String> = env::args().collect();
    let cuit_consultar: u64 = if args.len() > 1 {
        args[1].parse().expect("CUIT inválido")
    } else {
        println!("Uso: cargo run --example consultar_padron -- <CUIT>");
        println!("Ejemplo: cargo run --example consultar_padron -- 20299879624");
        return Ok(());
    };

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║     CONSULTA PADRÓN A13 - PRODUCCIÓN                     ║");
    println!("║     Consultando CUIT: {}                       ║", cuit_consultar);
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // Cargar certificados
    let cert = fs::read("../certificado/cert_prod.pem")?;
    let key = fs::read("../certificado/key_prod.pem")?;

    let http_client = transport::HttpClient::new()?;

    // URLs de producción
    let wsaa_url = "https://wsaa.afip.gov.ar/ws/services/LoginCms";

    // Login con cache (servicio: ws_sr_padron_a13)
    let (token, sign) = if let Some(cached) = load_cached_token() {
        println!("1. Usando token cacheado (expira: {})", cached.expiration_time);
        (cached.token, cached.sign)
    } else {
        println!("1. Autenticando con WSAA para ws_sr_padron_a13...");
        let ltr = auth::build_login_ticket_request("ws_sr_padron_a13");
        let cms = auth::sign_cms_base64(&ltr, &cert, &key)?;

        let ta = auth::wsaa_login(&http_client, wsaa_url, &cms).await?;
        save_token_cache(&ta.token, &ta.sign, &ta.expiration_time);
        println!("   ✓ Login exitoso");

        (ta.token.clone(), ta.sign.clone())
    };

    // Crear servicio de padrón
    let padron = PadronA13Service::production();

    // Primero verificar que el servicio esté activo
    println!("\n2. Verificando servicio...");
    match padron.dummy(&http_client).await {
        Ok(dummy) => {
            println!("   App Server: {}", dummy.app_server);
            println!("   Auth Server: {}", dummy.auth_server);
            println!("   DB Server: {}", dummy.db_server);
        }
        Err(e) => {
            println!("   ⚠ Error en dummy: {}", e);
        }
    }

    // Consultar persona
    println!("\n3. Consultando CUIT {}...", cuit_consultar);
    match padron.get_persona(&http_client, &token, &sign, CUIT_REPRESENTADA, cuit_consultar).await {
        Ok(result) => {
            let p = &result.persona;

            println!("\n   ╔═══════════════════════════════════════════════════════╗");
            println!("   ║  DATOS DEL CONTRIBUYENTE                              ║");
            println!("   ╠═══════════════════════════════════════════════════════╣");
            println!("   ║  CUIT: {}                                   ║", p.id_persona);
            println!("   ║  Nombre: {:<42} ║", p.nombre_completo());
            println!("   ║  Tipo: {:<44} ║", p.tipo_persona.as_deref().unwrap_or("-"));
            println!("   ║  Estado: {:<42} ║", p.estado_clave.as_deref().unwrap_or("-"));

            if let Some(ref actividad) = p.descripcion_actividad_principal {
                let act_short = if actividad.len() > 40 {
                    format!("{}...", &actividad[..37])
                } else {
                    actividad.clone()
                };
                println!("   ║  Actividad: {:<39} ║", act_short);
            }

            if let Some(dom) = p.domicilio_fiscal() {
                println!("   ╠═══════════════════════════════════════════════════════╣");
                println!("   ║  DOMICILIO FISCAL                                     ║");
                if let Some(ref dir) = dom.direccion {
                    let dir_short = if dir.len() > 42 {
                        format!("{}...", &dir[..39])
                    } else {
                        dir.clone()
                    };
                    println!("   ║  {:<52} ║", dir_short);
                } else if let Some(ref calle) = dom.calle {
                    let nro = dom.numero.as_deref().unwrap_or("");
                    println!("   ║  {} {}                                   ║", calle, nro);
                }
                if let Some(ref loc) = dom.localidad {
                    let prov = dom.descripcion_provincia.as_deref().unwrap_or("");
                    let cp = dom.codigo_postal.as_deref().unwrap_or("");
                    println!("   ║  {} - {} ({})             ║", loc, prov, cp);
                }
            }

            println!("   ╚═══════════════════════════════════════════════════════╝");

            // Info adicional
            if p.es_persona_fisica() {
                if let Some(ref fecha) = p.fecha_nacimiento {
                    println!("\n   Fecha nacimiento: {}", fecha);
                }
            } else {
                if let Some(ref forma) = p.forma_juridica {
                    println!("\n   Forma jurídica: {}", forma);
                }
            }

            if !p.claves_inactivas.is_empty() {
                println!("   Claves inactivas asociadas: {:?}", p.claves_inactivas);
            }

            println!("\n   Servidor: {} - {}", result.servidor, result.fecha_hora);
        }
        Err(e) => {
            println!("\n   ✗ Error consultando padrón: {}", e);
        }
    }

    Ok(())
}
