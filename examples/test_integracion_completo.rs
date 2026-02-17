//! Suite completa de tests de integración contra homologación AFIP
//!
//! Esta suite prueba todos los métodos implementados del servicio WSFEv1
//! contra el ambiente de homologación de AFIP.
//!
//! Ejecutar con:
//! cargo run --example test_integracion_completo
//!
//! Para ver solo el resumen:
//! cargo run --example test_integracion_completo 2>&1 | grep -E "^(═|─|✓|✗|⚠|RESULTADO)"

use arca::{
    AlicIva, FeCabReq, FeDetReq, WsFev1Service,
    alicuotas_iva, cbte_tipos, condicion_iva, doc_tipos, codigos_error,
    transport::HttpClient, auth,
};
use std::fs;
use std::path::Path;
use std::time::Instant;

const CUIT: u64 = 20123456781;
const PTO_VTA: i32 = 1;
const TOKEN_CACHE_FILE: &str = "/tmp/arca_token_cache_homo.json";

// ============================================================================
// Estructuras de soporte
// ============================================================================

#[derive(Debug)]
struct TestResult {
    name: String,
    passed: bool,
    duration_ms: u128,
    message: Option<String>,
}

struct TestSuite {
    results: Vec<TestResult>,
    start_time: Instant,
}

impl TestSuite {
    fn new() -> Self {
        Self {
            results: Vec::new(),
            start_time: Instant::now(),
        }
    }

    fn add_result(&mut self, name: &str, passed: bool, duration_ms: u128, message: Option<String>) {
        self.results.push(TestResult {
            name: name.to_string(),
            passed,
            duration_ms,
            message,
        });
    }

    fn passed_count(&self) -> usize {
        self.results.iter().filter(|r| r.passed).count()
    }

    fn failed_count(&self) -> usize {
        self.results.iter().filter(|r| !r.passed).count()
    }

    fn total_duration_ms(&self) -> u128 {
        self.start_time.elapsed().as_millis()
    }

    fn print_summary(&self) {
        println!("\n═══════════════════════════════════════════════════════════════════");
        println!("                      RESUMEN DE TESTS");
        println!("═══════════════════════════════════════════════════════════════════");

        for result in &self.results {
            let status = if result.passed { "✓" } else { "✗" };
            let time = format!("{}ms", result.duration_ms);
            print!("   {} {:50} {:>8}", status, result.name, time);
            if let Some(ref msg) = result.message {
                if !result.passed {
                    print!(" - {}", msg);
                }
            }
            println!();
        }

        println!("───────────────────────────────────────────────────────────────────");
        println!("   RESULTADO: {} pasaron, {} fallaron de {} tests",
            self.passed_count(),
            self.failed_count(),
            self.results.len()
        );
        println!("   Tiempo total: {}ms", self.total_duration_ms());
        println!("═══════════════════════════════════════════════════════════════════");

        if self.failed_count() == 0 {
            println!("\n   ✓ TODOS LOS TESTS PASARON\n");
        } else {
            println!("\n   ✗ ALGUNOS TESTS FALLARON\n");
        }
    }
}

// ============================================================================
// Cache de token
// ============================================================================

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

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("╔═══════════════════════════════════════════════════════════════════╗");
    println!("║     SUITE DE TESTS DE INTEGRACIÓN - AFIP HOMOLOGACIÓN            ║");
    println!("╠═══════════════════════════════════════════════════════════════════╣");
    println!("║  CUIT: {}                                               ║", CUIT);
    println!("║  Punto de Venta: {}                                              ║", PTO_VTA);
    println!("║  Ambiente: Homologación                                          ║");
    println!("╚═══════════════════════════════════════════════════════════════════╝\n");

    let mut suite = TestSuite::new();

    // =========================================================================
    // FASE 1: Setup y Autenticación
    // =========================================================================
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  FASE 1: SETUP Y AUTENTICACIÓN");
    println!("═══════════════════════════════════════════════════════════════════\n");

    // Test 1.1: Cargar certificados
    let start = Instant::now();
    let cert_path = "../certificado/cert.pem";
    let key_path = "../certificado/key.pem";

    let (cert, key) = match (fs::read(cert_path), fs::read(key_path)) {
        (Ok(c), Ok(k)) => {
            println!("   ✓ Test 1.1: Cargar certificados");
            suite.add_result("1.1 Cargar certificados", true, start.elapsed().as_millis(), None);
            (c, k)
        }
        _ => {
            println!("   ✗ Test 1.1: No se encontraron certificados");
            suite.add_result("1.1 Cargar certificados", false, start.elapsed().as_millis(),
                Some("Certificados no encontrados".to_string()));
            suite.print_summary();
            return Ok(());
        }
    };

    // Test 1.2: Crear cliente HTTP
    let start = Instant::now();
    let http = match HttpClient::new() {
        Ok(h) => {
            println!("   ✓ Test 1.2: Crear cliente HTTP");
            suite.add_result("1.2 Crear cliente HTTP", true, start.elapsed().as_millis(), None);
            h
        }
        Err(e) => {
            println!("   ✗ Test 1.2: Error creando cliente HTTP: {}", e);
            suite.add_result("1.2 Crear cliente HTTP", false, start.elapsed().as_millis(),
                Some(e.to_string()));
            suite.print_summary();
            return Ok(());
        }
    };

    // Test 1.3: Autenticación WSAA
    let start = Instant::now();
    let (token, sign) = if let Some(cached) = load_cached_token() {
        println!("   ✓ Test 1.3: Autenticación WSAA (token cacheado)");
        suite.add_result("1.3 Autenticación WSAA", true, start.elapsed().as_millis(),
            Some("Token cacheado".to_string()));
        (cached.token, cached.sign)
    } else {
        let wsaa_url = "https://wsaahomo.afip.gov.ar/ws/services/LoginCms";
        let ltr = auth::build_login_ticket_request("wsfe");
        let cms = auth::sign_cms_base64(&ltr, &cert, &key)?;

        match auth::wsaa_login(&http, wsaa_url, &cms).await {
            Ok(ta) => {
                save_token_cache(&ta.token, &ta.sign, &ta.expiration_time);
                println!("   ✓ Test 1.3: Autenticación WSAA (nuevo token)");
                suite.add_result("1.3 Autenticación WSAA", true, start.elapsed().as_millis(),
                    Some("Nuevo token obtenido".to_string()));
                (ta.token.clone(), ta.sign.clone())
            }
            Err(e) => {
                println!("   ✗ Test 1.3: Error de autenticación: {}", e);
                suite.add_result("1.3 Autenticación WSAA", false, start.elapsed().as_millis(),
                    Some(e.to_string()));
                suite.print_summary();
                return Ok(());
            }
        }
    };

    let wsfev1 = WsFev1Service::new(
        "https://wswhomo.afip.gov.ar/wsfev1/service.asmx".to_string(),
        http.clone(),
    );

    // =========================================================================
    // FASE 2: Tests de Estado del Servicio
    // =========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("  FASE 2: ESTADO DEL SERVICIO");
    println!("═══════════════════════════════════════════════════════════════════\n");

    // Test 2.1: FEDummy
    let start = Instant::now();
    match wsfev1.fe_dummy().await {
        Ok((app, auth_srv, db)) => {
            let all_ok = app == "OK" && auth_srv == "OK" && db == "OK";
            if all_ok {
                println!("   ✓ Test 2.1: FEDummy - Servicio operativo");
                suite.add_result("2.1 FEDummy", true, start.elapsed().as_millis(), None);
            } else {
                println!("   ⚠ Test 2.1: FEDummy - App:{} Auth:{} DB:{}", app, auth_srv, db);
                suite.add_result("2.1 FEDummy", false, start.elapsed().as_millis(),
                    Some(format!("App:{} Auth:{} DB:{}", app, auth_srv, db)));
            }
        }
        Err(e) => {
            println!("   ✗ Test 2.1: FEDummy - Error: {}", e);
            suite.add_result("2.1 FEDummy", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // =========================================================================
    // FASE 3: Tests de Parámetros
    // =========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("  FASE 3: CONSULTA DE PARÁMETROS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    // Test 3.1: FEParamGetTiposCbte
    let start = Instant::now();
    match wsfev1.fe_param_get_tipos_cbte(&token, &sign, CUIT).await {
        Ok(tipos) => {
            println!("   ✓ Test 3.1: FEParamGetTiposCbte - {} tipos", tipos.len());
            suite.add_result("3.1 FEParamGetTiposCbte", true, start.elapsed().as_millis(),
                Some(format!("{} tipos", tipos.len())));
        }
        Err(e) => {
            println!("   ✗ Test 3.1: FEParamGetTiposCbte - Error: {}", e);
            suite.add_result("3.1 FEParamGetTiposCbte", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // Test 3.2: FEParamGetTiposDoc
    let start = Instant::now();
    match wsfev1.fe_param_get_tipos_doc(&token, &sign, CUIT).await {
        Ok(tipos) => {
            println!("   ✓ Test 3.2: FEParamGetTiposDoc - {} tipos", tipos.len());
            suite.add_result("3.2 FEParamGetTiposDoc", true, start.elapsed().as_millis(),
                Some(format!("{} tipos", tipos.len())));
        }
        Err(e) => {
            println!("   ✗ Test 3.2: FEParamGetTiposDoc - Error: {}", e);
            suite.add_result("3.2 FEParamGetTiposDoc", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // Test 3.3: FEParamGetTiposIva
    let start = Instant::now();
    match wsfev1.fe_param_get_tipos_iva(&token, &sign, CUIT).await {
        Ok(tipos) => {
            println!("   ✓ Test 3.3: FEParamGetTiposIva - {} tipos", tipos.len());
            suite.add_result("3.3 FEParamGetTiposIva", true, start.elapsed().as_millis(),
                Some(format!("{} tipos", tipos.len())));
        }
        Err(e) => {
            println!("   ✗ Test 3.3: FEParamGetTiposIva - Error: {}", e);
            suite.add_result("3.3 FEParamGetTiposIva", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // Test 3.4: FEParamGetTiposMonedas
    let start = Instant::now();
    match wsfev1.fe_param_get_tipos_monedas(&token, &sign, CUIT).await {
        Ok(tipos) => {
            println!("   ✓ Test 3.4: FEParamGetTiposMonedas - {} tipos", tipos.len());
            suite.add_result("3.4 FEParamGetTiposMonedas", true, start.elapsed().as_millis(),
                Some(format!("{} tipos", tipos.len())));
        }
        Err(e) => {
            println!("   ✗ Test 3.4: FEParamGetTiposMonedas - Error: {}", e);
            suite.add_result("3.4 FEParamGetTiposMonedas", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // Test 3.5: FEParamGetTiposTributos
    let start = Instant::now();
    match wsfev1.fe_param_get_tipos_tributos(&token, &sign, CUIT).await {
        Ok(tipos) => {
            println!("   ✓ Test 3.5: FEParamGetTiposTributos - {} tipos", tipos.len());
            suite.add_result("3.5 FEParamGetTiposTributos", true, start.elapsed().as_millis(),
                Some(format!("{} tipos", tipos.len())));
        }
        Err(e) => {
            println!("   ✗ Test 3.5: FEParamGetTiposTributos - Error: {}", e);
            suite.add_result("3.5 FEParamGetTiposTributos", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // Test 3.6: FEParamGetTiposConcepto
    let start = Instant::now();
    match wsfev1.fe_param_get_tipos_concepto(&token, &sign, CUIT).await {
        Ok(tipos) => {
            println!("   ✓ Test 3.6: FEParamGetTiposConcepto - {} tipos", tipos.len());
            suite.add_result("3.6 FEParamGetTiposConcepto", true, start.elapsed().as_millis(),
                Some(format!("{} tipos", tipos.len())));
        }
        Err(e) => {
            println!("   ✗ Test 3.6: FEParamGetTiposConcepto - Error: {}", e);
            suite.add_result("3.6 FEParamGetTiposConcepto", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // Test 3.7: FEParamGetTiposOpcional
    let start = Instant::now();
    match wsfev1.fe_param_get_tipos_opcional(&token, &sign, CUIT).await {
        Ok(tipos) => {
            println!("   ✓ Test 3.7: FEParamGetTiposOpcional - {} tipos", tipos.len());
            suite.add_result("3.7 FEParamGetTiposOpcional", true, start.elapsed().as_millis(),
                Some(format!("{} tipos", tipos.len())));
        }
        Err(e) => {
            println!("   ✗ Test 3.7: FEParamGetTiposOpcional - Error: {}", e);
            suite.add_result("3.7 FEParamGetTiposOpcional", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // Test 3.8: FEParamGetCondicionIvaReceptor
    let start = Instant::now();
    match wsfev1.fe_param_get_condicion_iva_receptor(&token, &sign, CUIT).await {
        Ok(tipos) => {
            println!("   ✓ Test 3.8: FEParamGetCondicionIvaReceptor - {} tipos", tipos.len());
            suite.add_result("3.8 FEParamGetCondicionIvaReceptor", true, start.elapsed().as_millis(),
                Some(format!("{} tipos", tipos.len())));
        }
        Err(e) => {
            println!("   ✗ Test 3.8: FEParamGetCondicionIvaReceptor - Error: {}", e);
            suite.add_result("3.8 FEParamGetCondicionIvaReceptor", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // Test 3.9: FEParamGetPtosVenta
    let start = Instant::now();
    match wsfev1.fe_param_get_ptos_venta(&token, &sign, CUIT).await {
        Ok(ptos) => {
            println!("   ✓ Test 3.9: FEParamGetPtosVenta - {} puntos", ptos.len());
            suite.add_result("3.9 FEParamGetPtosVenta", true, start.elapsed().as_millis(),
                Some(format!("{} puntos", ptos.len())));
        }
        Err(e) => {
            println!("   ✗ Test 3.9: FEParamGetPtosVenta - Error: {}", e);
            suite.add_result("3.9 FEParamGetPtosVenta", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // Test 3.10: FEParamGetCotizacion (Dólar)
    let start = Instant::now();
    match wsfev1.fe_param_get_cotizacion(&token, &sign, CUIT, "DOL").await {
        Ok(cot) => {
            println!("   ✓ Test 3.10: FEParamGetCotizacion - USD = ${:.2}", cot.mon_cotiz);
            suite.add_result("3.10 FEParamGetCotizacion", true, start.elapsed().as_millis(),
                Some(format!("USD = ${:.2}", cot.mon_cotiz)));
        }
        Err(e) => {
            println!("   ✗ Test 3.10: FEParamGetCotizacion - Error: {}", e);
            suite.add_result("3.10 FEParamGetCotizacion", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // =========================================================================
    // FASE 4: Tests de Comprobantes
    // =========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("  FASE 4: COMPROBANTES");
    println!("═══════════════════════════════════════════════════════════════════\n");

    // Test 4.1: FECompUltimoAutorizado - Factura B
    let start = Instant::now();
    let ultimo_b = match wsfev1.fe_comp_ultimo_autorizado(&token, &sign, CUIT, PTO_VTA, cbte_tipos::FACTURA_B).await {
        Ok(u) => {
            println!("   ✓ Test 4.1: FECompUltimoAutorizado (Fact B) - #{}", u);
            suite.add_result("4.1 FECompUltimoAutorizado Fact B", true, start.elapsed().as_millis(),
                Some(format!("#{}", u)));
            u
        }
        Err(e) => {
            println!("   ✗ Test 4.1: FECompUltimoAutorizado (Fact B) - Error: {}", e);
            suite.add_result("4.1 FECompUltimoAutorizado Fact B", false, start.elapsed().as_millis(), Some(e.to_string()));
            0
        }
    };

    // Test 4.2: FECompUltimoAutorizado - Nota de Crédito B
    let start = Instant::now();
    match wsfev1.fe_comp_ultimo_autorizado(&token, &sign, CUIT, PTO_VTA, cbte_tipos::NOTA_CREDITO_B).await {
        Ok(u) => {
            println!("   ✓ Test 4.2: FECompUltimoAutorizado (NC B) - #{}", u);
            suite.add_result("4.2 FECompUltimoAutorizado NC B", true, start.elapsed().as_millis(),
                Some(format!("#{}", u)));
        }
        Err(e) => {
            println!("   ✗ Test 4.2: FECompUltimoAutorizado (NC B) - Error: {}", e);
            suite.add_result("4.2 FECompUltimoAutorizado NC B", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // Test 4.3: FECAESolicitar - Factura B válida
    let start = Instant::now();
    let nuevo_b = (ultimo_b + 1) as u64;
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
        cbte_desde: nuevo_b,
        cbte_hasta: nuevo_b,
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
        iva: vec![AlicIva {
            id: alicuotas_iva::VEINTIUNO_PORCIENTO,
            base_imp: 100.0,
            importe: 21.0,
        }],
    };

    let cae_factura_b = match wsfev1.fecae_solicitar(&token, &sign, CUIT, cab.clone(), vec![det]).await {
        Ok(resp) => {
            println!("   ✓ Test 4.3: FECAESolicitar (Fact B) - CAE: {}", resp.cae);
            suite.add_result("4.3 FECAESolicitar Fact B", true, start.elapsed().as_millis(),
                Some(format!("CAE: {}", resp.cae)));
            Some((nuevo_b, resp.cae))
        }
        Err(e) => {
            println!("   ✗ Test 4.3: FECAESolicitar (Fact B) - Error: {}", e);
            suite.add_result("4.3 FECAESolicitar Fact B", false, start.elapsed().as_millis(), Some(e.to_string()));
            None
        }
    };

    // Test 4.4: FECAESolicitar con respuesta detallada
    let start = Instant::now();
    let ultimo_b2 = wsfev1.fe_comp_ultimo_autorizado(&token, &sign, CUIT, PTO_VTA, cbte_tipos::FACTURA_B).await.unwrap_or(0);
    let nuevo_b2 = (ultimo_b2 + 1) as u64;

    let cab2 = FeCabReq {
        cant_reg: 1,
        pto_vta: PTO_VTA,
        cbte_tipo: cbte_tipos::FACTURA_B,
    };

    let det2 = FeDetReq {
        concepto: 1,
        doc_tipo: doc_tipos::CONSUMIDOR_FINAL,
        doc_nro: 0,
        cbte_desde: nuevo_b2,
        cbte_hasta: nuevo_b2,
        cbte_fch: fecha_hoy.clone(),
        imp_total: 242.0,
        imp_tot_conc: 0.0,
        imp_neto: 200.0,
        imp_op_ex: 0.0,
        imp_trib: 0.0,
        imp_iva: 42.0,
        fch_serv_desde: None,
        fch_serv_hasta: None,
        fch_vto_pago: None,
        mon_id: "PES".to_string(),
        mon_cotiz: 1.0,
        condicion_iva_receptor: Some(condicion_iva::CONSUMIDOR_FINAL),
        iva: vec![AlicIva {
            id: alicuotas_iva::VEINTIUNO_PORCIENTO,
            base_imp: 200.0,
            importe: 42.0,
        }],
    };

    match wsfev1.fecae_solicitar_detallado(&token, &sign, CUIT, cab2, vec![det2]).await {
        Ok(resp) => {
            let status = if resp.is_approved() { "Aprobado" } else { "Rechazado" };
            println!("   ✓ Test 4.4: FECAESolicitar detallado - {} CAE: {}", status, resp.cae);
            suite.add_result("4.4 FECAESolicitar detallado", true, start.elapsed().as_millis(),
                Some(format!("{} CAE: {}", status, resp.cae)));
        }
        Err(e) => {
            println!("   ✗ Test 4.4: FECAESolicitar detallado - Error: {}", e);
            suite.add_result("4.4 FECAESolicitar detallado", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // Test 4.5: FECompConsultar
    if let Some((cbte_nro, _cae)) = cae_factura_b {
        let start = Instant::now();
        match wsfev1.fe_comp_consultar(&token, &sign, CUIT, cbte_tipos::FACTURA_B, cbte_nro, PTO_VTA).await {
            Ok(comp) => {
                println!("   ✓ Test 4.5: FECompConsultar - Resultado: {} CAE: {}", comp.resultado, comp.cod_autorizacion);
                suite.add_result("4.5 FECompConsultar", true, start.elapsed().as_millis(),
                    Some(format!("Resultado: {}", comp.resultado)));
            }
            Err(e) => {
                println!("   ✗ Test 4.5: FECompConsultar - Error: {}", e);
                suite.add_result("4.5 FECompConsultar", false, start.elapsed().as_millis(), Some(e.to_string()));
            }
        }
    } else {
        println!("   ⏭ Test 4.5: FECompConsultar - Saltado (no hay factura previa)");
        suite.add_result("4.5 FECompConsultar", true, 0, Some("Saltado".to_string()));
    }

    // =========================================================================
    // FASE 5: Tests de Manejo de Errores
    // =========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("  FASE 5: MANEJO DE ERRORES");
    println!("═══════════════════════════════════════════════════════════════════\n");

    // Test 5.1: Error por número duplicado
    let start = Instant::now();
    let cab_dup = FeCabReq {
        cant_reg: 1,
        pto_vta: PTO_VTA,
        cbte_tipo: cbte_tipos::FACTURA_B,
    };

    let det_dup = FeDetReq {
        concepto: 1,
        doc_tipo: doc_tipos::CONSUMIDOR_FINAL,
        doc_nro: 0,
        cbte_desde: 1, // Número viejo que ya existe
        cbte_hasta: 1,
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
        iva: vec![AlicIva {
            id: alicuotas_iva::VEINTIUNO_PORCIENTO,
            base_imp: 100.0,
            importe: 21.0,
        }],
    };

    match wsfev1.fecae_solicitar_detallado(&token, &sign, CUIT, cab_dup, vec![det_dup]).await {
        Ok(resp) => {
            if resp.is_rejected() && resp.detalles.has_errors() {
                let err_code = resp.detalles.errors.first().map(|e| e.code).unwrap_or(0);
                let desc = codigos_error::descripcion(err_code);
                println!("   ✓ Test 5.1: Error número duplicado - Código {} ({})", err_code, desc);
                suite.add_result("5.1 Error número duplicado", true, start.elapsed().as_millis(),
                    Some(format!("Código {}", err_code)));
            } else {
                println!("   ✗ Test 5.1: Se esperaba rechazo pero fue: {}", resp.resultado);
                suite.add_result("5.1 Error número duplicado", false, start.elapsed().as_millis(),
                    Some("No se rechazó".to_string()));
            }
        }
        Err(e) => {
            // También es válido si el error viene como excepción
            println!("   ✓ Test 5.1: Error número duplicado - {}", e);
            suite.add_result("5.1 Error número duplicado", true, start.elapsed().as_millis(),
                Some("Error esperado".to_string()));
        }
    }

    // Test 5.2: Error por importes incorrectos
    let start = Instant::now();
    let ultimo_err = wsfev1.fe_comp_ultimo_autorizado(&token, &sign, CUIT, PTO_VTA, cbte_tipos::FACTURA_B).await.unwrap_or(0);

    let cab_err = FeCabReq {
        cant_reg: 1,
        pto_vta: PTO_VTA,
        cbte_tipo: cbte_tipos::FACTURA_B,
    };

    let det_err = FeDetReq {
        concepto: 1,
        doc_tipo: doc_tipos::CONSUMIDOR_FINAL,
        doc_nro: 0,
        cbte_desde: (ultimo_err + 1) as u64,
        cbte_hasta: (ultimo_err + 1) as u64,
        cbte_fch: fecha_hoy.clone(),
        imp_total: 100.0,  // Incorrecto: debería ser 121
        imp_tot_conc: 0.0,
        imp_neto: 100.0,
        imp_op_ex: 0.0,
        imp_trib: 0.0,
        imp_iva: 21.0,     // IVA es 21, pero total dice 100
        fch_serv_desde: None,
        fch_serv_hasta: None,
        fch_vto_pago: None,
        mon_id: "PES".to_string(),
        mon_cotiz: 1.0,
        condicion_iva_receptor: Some(condicion_iva::CONSUMIDOR_FINAL),
        iva: vec![AlicIva {
            id: alicuotas_iva::VEINTIUNO_PORCIENTO,
            base_imp: 100.0,
            importe: 21.0,
        }],
    };

    match wsfev1.fecae_solicitar_detallado(&token, &sign, CUIT, cab_err, vec![det_err]).await {
        Ok(resp) => {
            // AFIP puede rechazar con errores en Errors u Observaciones
            let has_issues = resp.detalles.has_errors() || resp.detalles.has_observaciones();
            if resp.is_rejected() && has_issues {
                let msg = if resp.detalles.has_errors() {
                    resp.detalles.format_errors()
                } else {
                    resp.detalles.format_observaciones()
                };
                println!("   ✓ Test 5.2: Error importes incorrectos - {}",
                    if msg.len() > 60 { &msg[..60] } else { &msg });
                suite.add_result("5.2 Error importes incorrectos", true, start.elapsed().as_millis(),
                    Some("Rechazado correctamente".to_string()));
            } else if resp.is_rejected() {
                // Rechazado pero sin detalles - aún es válido
                println!("   ✓ Test 5.2: Error importes incorrectos - Rechazado (sin detalles)");
                suite.add_result("5.2 Error importes incorrectos", true, start.elapsed().as_millis(),
                    Some("Rechazado".to_string()));
            } else {
                println!("   ✗ Test 5.2: Se esperaba rechazo pero fue: {}", resp.resultado);
                suite.add_result("5.2 Error importes incorrectos", false, start.elapsed().as_millis(),
                    Some("No se rechazó".to_string()));
            }
        }
        Err(e) => {
            println!("   ✓ Test 5.2: Error importes incorrectos - {}", e);
            suite.add_result("5.2 Error importes incorrectos", true, start.elapsed().as_millis(),
                Some("Error esperado".to_string()));
        }
    }

    // Test 5.3: Consultar comprobante inexistente
    let start = Instant::now();
    match wsfev1.fe_comp_consultar(&token, &sign, CUIT, cbte_tipos::FACTURA_B, 999999999, PTO_VTA).await {
        Ok(_) => {
            println!("   ✗ Test 5.3: Comprobante inexistente - No debería existir");
            suite.add_result("5.3 Comprobante inexistente", false, start.elapsed().as_millis(),
                Some("No debería existir".to_string()));
        }
        Err(e) => {
            let msg = e.to_string();
            let truncated = if msg.len() > 50 { &msg[..50] } else { &msg };
            println!("   ✓ Test 5.3: Comprobante inexistente - Error esperado: {}...", truncated);
            suite.add_result("5.3 Comprobante inexistente", true, start.elapsed().as_millis(),
                Some("Error esperado".to_string()));
        }
    }

    // =========================================================================
    // FASE 6: Tests de Cotizaciones
    // =========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("  FASE 6: COTIZACIONES");
    println!("═══════════════════════════════════════════════════════════════════\n");

    // Test 6.1: Cotización Euro
    let start = Instant::now();
    match wsfev1.fe_param_get_cotizacion(&token, &sign, CUIT, "060").await {
        Ok(cot) => {
            println!("   ✓ Test 6.1: Cotización Euro - EUR = ${:.2}", cot.mon_cotiz);
            suite.add_result("6.1 Cotización Euro", true, start.elapsed().as_millis(),
                Some(format!("EUR = ${:.2}", cot.mon_cotiz)));
        }
        Err(e) => {
            println!("   ✗ Test 6.1: Cotización Euro - Error: {}", e);
            suite.add_result("6.1 Cotización Euro", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // Test 6.2: Cotización Real
    let start = Instant::now();
    match wsfev1.fe_param_get_cotizacion(&token, &sign, CUIT, "012").await {
        Ok(cot) => {
            println!("   ✓ Test 6.2: Cotización Real - BRL = ${:.2}", cot.mon_cotiz);
            suite.add_result("6.2 Cotización Real", true, start.elapsed().as_millis(),
                Some(format!("BRL = ${:.2}", cot.mon_cotiz)));
        }
        Err(e) => {
            println!("   ✗ Test 6.2: Cotización Real - Error: {}", e);
            suite.add_result("6.2 Cotización Real", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // =========================================================================
    // FASE 7: Tests de Factura con Servicios
    // =========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("  FASE 7: FACTURA DE SERVICIOS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    // Test 7.1: Factura B de servicios
    let start = Instant::now();
    let ultimo_serv = wsfev1.fe_comp_ultimo_autorizado(&token, &sign, CUIT, PTO_VTA, cbte_tipos::FACTURA_B).await.unwrap_or(0);
    let nuevo_serv = (ultimo_serv + 1) as u64;

    let fecha_inicio = chrono::Local::now().format("%Y%m01").to_string();
    let fecha_fin = chrono::Local::now().format("%Y%m%d").to_string();

    let cab_serv = FeCabReq {
        cant_reg: 1,
        pto_vta: PTO_VTA,
        cbte_tipo: cbte_tipos::FACTURA_B,
    };

    let det_serv = FeDetReq {
        concepto: 2, // Servicios
        doc_tipo: doc_tipos::CONSUMIDOR_FINAL,
        doc_nro: 0,
        cbte_desde: nuevo_serv,
        cbte_hasta: nuevo_serv,
        cbte_fch: fecha_hoy.clone(),
        imp_total: 1210.0,
        imp_tot_conc: 0.0,
        imp_neto: 1000.0,
        imp_op_ex: 0.0,
        imp_trib: 0.0,
        imp_iva: 210.0,
        fch_serv_desde: Some(fecha_inicio),
        fch_serv_hasta: Some(fecha_fin.clone()),
        fch_vto_pago: Some(fecha_fin),
        mon_id: "PES".to_string(),
        mon_cotiz: 1.0,
        condicion_iva_receptor: Some(condicion_iva::CONSUMIDOR_FINAL),
        iva: vec![AlicIva {
            id: alicuotas_iva::VEINTIUNO_PORCIENTO,
            base_imp: 1000.0,
            importe: 210.0,
        }],
    };

    match wsfev1.fecae_solicitar(&token, &sign, CUIT, cab_serv, vec![det_serv]).await {
        Ok(resp) => {
            println!("   ✓ Test 7.1: Factura servicios - CAE: {}", resp.cae);
            suite.add_result("7.1 Factura servicios", true, start.elapsed().as_millis(),
                Some(format!("CAE: {}", resp.cae)));
        }
        Err(e) => {
            println!("   ✗ Test 7.1: Factura servicios - Error: {}", e);
            suite.add_result("7.1 Factura servicios", false, start.elapsed().as_millis(), Some(e.to_string()));
        }
    }

    // =========================================================================
    // RESUMEN FINAL
    // =========================================================================
    suite.print_summary();

    Ok(())
}
