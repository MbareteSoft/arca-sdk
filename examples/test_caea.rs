//! Test de métodos CAEA (Código de Autorización Electrónico Anticipado)
//!
//! CAEA permite emitir facturas en modo offline (contingencia).
//! Flujo:
//! 1. Solicitar CAEA para un período (quincenal)
//! 2. Emitir facturas offline usando el CAEA
//! 3. Informar las facturas emitidas a ARCA
//! 4. Si no hubo movimiento, informar sin movimiento
//!
//! Ejecutar con:
//! cargo run --example test_caea

use arca::{
    CaeaResponse, WsFev1Service, transport::HttpClient, auth,
};
use chrono::Datelike;
use std::fs;
use std::path::Path;

const CUIT: u64 = 20123456781;
const PTO_VTA: i32 = 1;
const TOKEN_CACHE_FILE: &str = "/tmp/arca_token_cache_homo.json";

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

/// Calcular período y orden actual para CAEA
/// Período: YYYYMM
/// Orden: 1 = primera quincena (días 1-15), 2 = segunda quincena (días 16-fin)
fn get_current_caea_period() -> (i32, i16) {
    let now = chrono::Local::now();
    let periodo = now.format("%Y%m").to_string().parse::<i32>().unwrap();
    let orden = if now.day() <= 15 { 1 } else { 2 };
    (periodo, orden)
}

/// Calcular próximo período CAEA (para solicitar con anticipación)
fn get_next_caea_period() -> (i32, i16) {
    let now = chrono::Local::now();
    let day = now.day();
    
    if day <= 15 {
        // Estamos en primera quincena, próximo es segunda quincena del mismo mes
        let periodo = now.format("%Y%m").to_string().parse::<i32>().unwrap();
        (periodo, 2)
    } else {
        // Estamos en segunda quincena, próximo es primera quincena del mes siguiente
        let next_month = if now.month() == 12 {
            chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap()
        } else {
            chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1).unwrap()
        };
        let periodo = next_month.format("%Y%m").to_string().parse::<i32>().unwrap();
        (periodo, 1)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       TEST CAEA - MODO CONTINGENCIA (HOMOLOGACIÓN)           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut tests_ok = 0;
    let mut tests_fail = 0;

    // =========================================================================
    // 1. Cargar certificados
    // =========================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("1. CARGANDO CERTIFICADOS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let cert_path = "../certificado/cert.pem";
    let key_path = "../certificado/key.pem";

    let (cert, key) = match (fs::read(cert_path), fs::read(key_path)) {
        (Ok(c), Ok(k)) => {
            println!("   ✓ Certificados cargados");
            (c, k)
        }
        _ => {
            println!("   ✗ No se encontraron certificados");
            return Ok(());
        }
    };

    let http = HttpClient::new()?;

    // =========================================================================
    // 2. Autenticación WSAA
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("2. AUTENTICACIÓN WSAA");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let (token, sign) = if let Some(cached) = load_cached_token() {
        println!("   Usando token cacheado (expira: {})", cached.expiration_time);
        (cached.token, cached.sign)
    } else {
        println!("   Solicitando nuevo token...");
        let wsaa_url = "https://wsaahomo.afip.gov.ar/ws/services/LoginCms";
        let ltr = auth::build_login_ticket_request("wsfe");
        let cms = auth::sign_cms_base64(&ltr, &cert, &key)?;

        match auth::wsaa_login(&http, wsaa_url, &cms).await {
            Ok(ta) => {
                save_token_cache(&ta.token, &ta.sign, &ta.expiration_time);
                println!("   ✓ Token obtenido");
                (ta.token.clone(), ta.sign.clone())
            }
            Err(e) => {
                println!("   ✗ Error: {}", e);
                return Ok(());
            }
        }
    };

    let wsfev1 = WsFev1Service::new(
        "https://wswhomo.afip.gov.ar/wsfev1/service.asmx".to_string(),
        http.clone(),
    );

    // =========================================================================
    // 3. Información de períodos CAEA
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("3. INFORMACIÓN DE PERÍODOS CAEA");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let (periodo_actual, orden_actual) = get_current_caea_period();
    let (periodo_prox, orden_prox) = get_next_caea_period();
    
    let quincena_actual = if orden_actual == 1 { "1ra (días 1-15)" } else { "2da (días 16-fin)" };
    let quincena_prox = if orden_prox == 1 { "1ra (días 1-15)" } else { "2da (días 16-fin)" };

    println!("   Período actual: {} - {} quincena", periodo_actual, quincena_actual);
    println!("   Próximo período: {} - {} quincena", periodo_prox, quincena_prox);
    println!("\n   NOTA: CAEA se solicita con 5 días de anticipación");

    // =========================================================================
    // 4. Solicitar CAEA (FECAEASolicitar)
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("4. SOLICITAR CAEA (FECAEASolicitar)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("   Solicitando CAEA para período {} orden {}...", periodo_prox, orden_prox);

    let caea_code: Option<String> = match wsfev1.fecaea_solicitar(&token, &sign, CUIT, periodo_prox, orden_prox).await {
        Ok(caea) => {
            println!("\n   ╔═══════════════════════════════════════════════════════════╗");
            println!("   ║              CAEA OBTENIDO                                ║");
            println!("   ╠═══════════════════════════════════════════════════════════╣");
            println!("   ║  CAEA:        {}                          ║", caea.caea);
            println!("   ║  Período:     {}                                    ║", caea.periodo);
            println!("   ║  Orden:       {}                                         ║", caea.orden);
            println!("   ║  Vigencia:    {} a {}                   ║", caea.fch_vig_desde, caea.fch_vig_hasta);
            println!("   ║  Tope Inf:    {}                                ║", caea.fch_tope_inf);
            println!("   ╚═══════════════════════════════════════════════════════════╝");
            tests_ok += 1;
            Some(caea.caea)
        }
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("15006") {
                println!("   ⚠ CAEA ya fue solicitado para este período");
                println!("   Intentando consultar CAEA existente...");
                tests_ok += 1;
                None // Lo consultaremos después
            } else if err_str.contains("15008") {
                println!("   ⚠ Fuera de fecha para solicitar CAEA de este período");
                println!("   (CAEA se solicita entre el día 1 y 5 días antes del período)");
                tests_ok += 1;
                None
            } else {
                println!("   ✗ Error: {}", e);
                tests_fail += 1;
                None
            }
        }
    };

    // =========================================================================
    // 5. Consultar CAEA (FECAEAConsultar)
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("5. CONSULTAR CAEA (FECAEAConsultar)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Siempre consultar el próximo período (el que intentamos solicitar)
    let periodo_consulta = periodo_prox;
    let orden_consulta = orden_prox;

    println!("   Consultando CAEA período {} orden {}...", periodo_consulta, orden_consulta);

    let caea_consultado: Option<CaeaResponse> = match wsfev1.fecaea_consultar(&token, &sign, CUIT, periodo_consulta, orden_consulta).await {
        Ok(caea) => {
            println!("   ✓ CAEA encontrado: {}", caea.caea);
            println!("     Vigencia: {} a {}", caea.fch_vig_desde, caea.fch_vig_hasta);
            tests_ok += 1;
            Some(caea)
        }
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("15007") {
                println!("   ⚠ No existe CAEA para este período");
                println!("   (Normal si nunca se solicitó CAEA para este contribuyente)");
                tests_ok += 1;
            } else {
                println!("   ✗ Error: {}", e);
                tests_fail += 1;
            }
            None
        }
    };

    // =========================================================================
    // 6. Informar Sin Movimiento (FECAEASinMovimientoInformar)
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("6. INFORMAR SIN MOVIMIENTO (FECAEASinMovimientoInformar)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if let Some(caea) = &caea_consultado {
        println!("   Informando sin movimiento para CAEA {}...", caea.caea);
        
        match wsfev1.fecaea_sin_movimiento_informar(&token, &sign, CUIT, PTO_VTA, &caea.caea).await {
            Ok(result) => {
                println!("   ✓ Sin movimiento informado");
                println!("     PtoVta: {}", result.pto_vta);
                println!("     CAEA: {}", result.caea);
                println!("     Fecha proceso: {}", result.fch_proceso);
                tests_ok += 1;
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("15011") {
                    println!("   ⚠ Ya se informó sin movimiento para este CAEA/PtoVta");
                    tests_ok += 1;
                } else if err_str.contains("15010") {
                    println!("   ⚠ CAEA no vigente o ya tiene comprobantes informados");
                    tests_ok += 1;
                } else {
                    println!("   ✗ Error: {}", e);
                    tests_fail += 1;
                }
            }
        }
    } else {
        println!("   ⏭ Saltando (no hay CAEA disponible para probar)");
    }

    // =========================================================================
    // 7. Consultar Sin Movimiento (FECAEASinMovimientoConsultar)
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("7. CONSULTAR SIN MOVIMIENTO (FECAEASinMovimientoConsultar)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if let Some(caea) = &caea_consultado {
        println!("   Consultando sin movimiento para CAEA {}...", caea.caea);
        
        match wsfev1.fecaea_sin_movimiento_consultar(&token, &sign, CUIT, &caea.caea, PTO_VTA).await {
            Ok(results) => {
                if results.is_empty() {
                    println!("   ⚠ No hay registros de sin movimiento");
                } else {
                    println!("   ✓ Registros de sin movimiento: {}", results.len());
                    for r in &results {
                        println!("     - PtoVta: {}, CAEA: {}, Fecha: {}", r.pto_vta, r.caea, r.fch_proceso);
                    }
                }
                tests_ok += 1;
            }
            Err(e) => {
                println!("   ✗ Error: {}", e);
                tests_fail += 1;
            }
        }
    } else {
        println!("   ⏭ Saltando (no hay CAEA disponible para probar)");
    }

    // =========================================================================
    // 8. Ejemplo de registro de comprobantes con CAEA
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("8. EJEMPLO: ESTRUCTURA DE REGISTRO CON CAEA");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("   Para registrar facturas emitidas offline con CAEA:");
    println!("");
    println!("   let cab = FeCabReq {{");
    println!("       cant_reg: 1,");
    println!("       pto_vta: 1,");
    println!("       cbte_tipo: 6, // Factura B");
    println!("   }};");
    println!("");
    println!("   let det = FeDetReq {{");
    println!("       concepto: 1,");
    println!("       doc_tipo: 80, // CUIT");
    println!("       doc_nro: 20111111112,");
    println!("       // ... otros campos ...");
    println!("   }};");
    println!("");
    println!("   // Registrar con CAEA");
    println!("   wsfev1.fecaea_reg_informativo(");
    println!("       &token, &sign, cuit,");
    println!("       cab, vec![det],");
    println!("       \"14123456789012\" // CAEA");
    println!("   ).await;");

    // =========================================================================
    // Resumen
    // =========================================================================
    let total = tests_ok + tests_fail;
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                      RESUMEN CAEA                            ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Tests ejecutados: {:2}                                       ║", total);
    println!("║  ✓ Exitosos:       {:2}                                       ║", tests_ok);
    println!("║  ✗ Fallidos:       {:2}                                       ║", tests_fail);
    println!("╚══════════════════════════════════════════════════════════════╝");

    if tests_fail == 0 {
        println!("\n✅ MÉTODOS CAEA FUNCIONANDO CORRECTAMENTE\n");
    }

    // Documentación adicional
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("DOCUMENTACIÓN CAEA");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("");
    println!("Métodos disponibles:");
    println!("  • fecaea_solicitar()          - Solicitar nuevo CAEA");
    println!("  • fecaea_consultar()          - Consultar CAEA existente");
    println!("  • fecaea_reg_informativo()    - Informar facturas con CAEA");
    println!("  • fecaea_sin_movimiento_informar()   - Declarar sin movimiento");
    println!("  • fecaea_sin_movimiento_consultar()  - Consultar sin movimiento");
    println!("");
    println!("Códigos de error comunes:");
    println!("  • 15006 - CAEA ya solicitado para el período");
    println!("  • 15007 - No existe CAEA para el período consultado");
    println!("  • 15008 - Fuera de fecha para solicitar CAEA");
    println!("  • 15010 - CAEA no vigente o con comprobantes");
    println!("  • 15011 - Ya se informó sin movimiento");
    println!("");

    Ok(())
}
