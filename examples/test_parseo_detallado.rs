//! Test de parseo detallado de respuestas AFIP
//!
//! Demuestra el uso de FeCaeResponseDetallada que incluye:
//! - Errores (código y mensaje)
//! - Observaciones (advertencias)
//! - Eventos del sistema
//!
//! Ejecutar con:
//! cargo run --example test_parseo_detallado

use arca::{
    FeCabReq, FeDetReq, WsFev1Service, cbte_tipos, condicion_iva, doc_tipos,
    transport::HttpClient, auth, codigos_error,
};
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       TEST PARSEO DETALLADO DE RESPUESTAS AFIP              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

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
    // 2. Autenticación
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
        http,
    );

    // =========================================================================
    // 3. Test 1: Factura válida con respuesta detallada
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("3. TEST: FACTURA VÁLIDA (RESPUESTA DETALLADA)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let ultimo = wsfev1.fe_comp_ultimo_autorizado(&token, &sign, CUIT, PTO_VTA, cbte_tipos::FACTURA_B).await?;
    let nuevo = (ultimo + 1) as u64;
    let fecha_hoy = chrono::Local::now().format("%Y%m%d").to_string();

    let cab = FeCabReq {
        cant_reg: 1,
        pto_vta: PTO_VTA,
        cbte_tipo: cbte_tipos::FACTURA_B,
    };

    let det = FeDetReq {
        concepto: 1,
        doc_tipo: doc_tipos::CONSUMIDOR_FINAL,
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
        condicion_iva_receptor: Some(condicion_iva::CONSUMIDOR_FINAL),
        iva: vec![arca::AlicIva {
            id: arca::alicuotas_iva::VEINTIUNO_PORCIENTO,
            base_imp: 100.0,
            importe: 21.0,
        }],
    };

    match wsfev1.fecae_solicitar_detallado(&token, &sign, CUIT, cab, vec![det]).await {
        Ok(resp) => {
            println!("\n   RESPUESTA DETALLADA:");
            println!("   ┌────────────────────────────────────────────────────────────┐");
            println!("   │ Resultado:     {}                                          │", resp.resultado);
            println!("   │ CAE:           {}                          │", resp.cae);
            println!("   │ Vencimiento:   {}                                  │", resp.cae_fch_vto);
            println!("   │ Cbte Desde:    {}                                       │", resp.cbte_desde);
            println!("   │ Cbte Hasta:    {}                                       │", resp.cbte_hasta);
            println!("   │ Fch Proceso:   {}                       │", resp.fch_proceso);
            println!("   └────────────────────────────────────────────────────────────┘");

            // Mostrar errores si hay
            if resp.detalles.has_errors() {
                println!("\n   ERRORES ({}):", resp.detalles.errors.len());
                for err in &resp.detalles.errors {
                    let desc = codigos_error::descripcion(err.code);
                    println!("   ✗ [{}] {} ({})", err.code, err.msg, desc);
                }
            } else {
                println!("\n   ✓ Sin errores");
            }

            // Mostrar observaciones si hay
            if resp.detalles.has_observaciones() {
                println!("\n   OBSERVACIONES ({}):", resp.detalles.observaciones.len());
                for obs in &resp.detalles.observaciones {
                    let desc = codigos_error::descripcion(obs.code);
                    println!("   ⚠ [{}] {} ({})", obs.code, obs.msg, desc);
                }
            } else {
                println!("   ✓ Sin observaciones");
            }

            // Mostrar eventos si hay
            if resp.detalles.has_events() {
                println!("\n   EVENTOS ({}):", resp.detalles.events.len());
                for evt in &resp.detalles.events {
                    println!("   ℹ [{}] {}", evt.code, evt.msg);
                }
            } else {
                println!("   ✓ Sin eventos");
            }

            // Verificar aprobación
            println!("\n   Estado: {}", if resp.is_approved() {
                "✓ APROBADO"
            } else if resp.is_rejected() {
                "✗ RECHAZADO"
            } else {
                "⚠ PARCIAL"
            });
        }
        Err(e) => {
            println!("   ✗ Error: {}", e);
        }
    }

    // =========================================================================
    // 4. Test 2: Factura con error (número duplicado)
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("4. TEST: FACTURA CON ERROR (NÚMERO DUPLICADO)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Intentar crear factura con número duplicado (el mismo que acabamos de crear)
    let cab_dup = FeCabReq {
        cant_reg: 1,
        pto_vta: PTO_VTA,
        cbte_tipo: cbte_tipos::FACTURA_B,
    };

    let det_dup = FeDetReq {
        concepto: 1,
        doc_tipo: doc_tipos::CONSUMIDOR_FINAL,
        doc_nro: 0,
        cbte_desde: nuevo, // DUPLICADO!
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
        condicion_iva_receptor: Some(condicion_iva::CONSUMIDOR_FINAL),
        iva: vec![arca::AlicIva {
            id: arca::alicuotas_iva::VEINTIUNO_PORCIENTO,
            base_imp: 100.0,
            importe: 21.0,
        }],
    };

    println!("   Intentando emitir factura duplicada (cbte #{})...", nuevo);

    match wsfev1.fecae_solicitar_detallado(&token, &sign, CUIT, cab_dup, vec![det_dup]).await {
        Ok(resp) => {
            println!("\n   RESPUESTA DETALLADA:");
            println!("   Resultado: {}", resp.resultado);

            if resp.detalles.has_errors() {
                println!("\n   ERRORES DETECTADOS ({}):", resp.detalles.errors.len());
                for err in &resp.detalles.errors {
                    let desc = codigos_error::descripcion(err.code);
                    let cat = codigos_error::categoria(err.code);
                    println!("   ┌─────────────────────────────────────────────────────┐");
                    println!("   │ Código:     {}                                      ", err.code);
                    println!("   │ Mensaje:    {}", err.msg);
                    println!("   │ Descripción: {}", desc);
                    println!("   │ Categoría:  {}", cat);
                    println!("   └─────────────────────────────────────────────────────┘");
                }
            }

            if resp.detalles.has_observaciones() {
                println!("\n   OBSERVACIONES ({}):", resp.detalles.observaciones.len());
                for obs in &resp.detalles.observaciones {
                    let desc = codigos_error::descripcion(obs.code);
                    println!("   ⚠ [{}] {} -> {}", obs.code, obs.msg, desc);
                }
            }

            println!("\n   Usando format_errors(): {}", resp.detalles.format_errors());
        }
        Err(e) => {
            println!("   Error general: {}", e);
        }
    }

    // =========================================================================
    // 5. Resumen
    // =========================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("RESUMEN DE FUNCIONALIDAD");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("   Nuevos tipos disponibles:");
    println!("   • ArcaErr          - Error individual (code, msg)");
    println!("   • ArcaObs          - Observación individual (code, msg)");
    println!("   • ArcaEvt          - Evento individual (code, msg)");
    println!("   • ArcaResponseDetails - Colección de errores/obs/eventos");
    println!("   • FeCaeResponseDetallada - Respuesta CAE con detalles");
    println!();
    println!("   Métodos de ArcaResponseDetails:");
    println!("   • has_errors()         - ¿Hay errores?");
    println!("   • has_observaciones()  - ¿Hay observaciones?");
    println!("   • has_events()         - ¿Hay eventos?");
    println!("   • format_errors()      - String con todos los errores");
    println!("   • format_observaciones() - String con todas las obs");
    println!("   • format_events()      - String con todos los eventos");
    println!();
    println!("   Métodos de FeCaeResponseDetallada:");
    println!("   • is_approved()  - Resultado = 'A'");
    println!("   • is_rejected()  - Resultado = 'R'");
    println!("   • is_partial()   - Resultado = 'P'");
    println!("   • to_basic()     - Convertir a FeCaeResponse simple");
    println!();

    Ok(())
}
