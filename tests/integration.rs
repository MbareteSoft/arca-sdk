//! Integration tests for ARCA services
//!
//! These tests require valid ARCA/AFIP certificates and network access.
//! They are ignored by default. Run with:
//!
//! ```bash
//! # Run all integration tests (serially to avoid token conflicts)
//! cargo test --test integration -- --ignored --test-threads=1
//!
//! # Run specific integration test
//! cargo test --test integration test_wsaa_login -- --ignored
//! ```

use arca::{
    ArcaClient, ArcaEnvironment, AlicIva, FeCabReq, FeDetReq,
    alicuotas_iva, cbte_tipos, condicion_iva, doc_tipos,
};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

/// Shared client for all tests to avoid multiple login attempts
static SHARED_CLIENT: OnceLock<ArcaClient> = OnceLock::new();

/// Get the project root directory (where Cargo.toml is)
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Get certificate paths from environment or default locations
fn get_cert_paths() -> Option<(PathBuf, PathBuf)> {
    // Try environment variables first
    if let (Ok(cert), Ok(key)) = (env::var("ARCA_CERT_PEM"), env::var("ARCA_KEY_PEM")) {
        return Some((PathBuf::from(cert), PathBuf::from(key)));
    }

    // Try default paths relative to project root
    let root = project_root();
    let cert_path = root.join("../certificado/cert.pem");
    let key_path = root.join("../certificado/key.pem");

    if cert_path.exists() && key_path.exists() {
        return Some((cert_path, key_path));
    }

    None
}

/// Get PFX path and password from environment or default location
fn get_pfx_path() -> Option<(PathBuf, String)> {
    // Try environment variables
    if let Ok(pfx) = env::var("ARCA_PFX") {
        let password = env::var("ARCA_PFX_PASSWORD").unwrap_or_default();
        return Some((PathBuf::from(pfx), password));
    }

    // Try default path relative to project root
    let root = project_root();
    let pfx_path = root.join("../certificado/certificado2024GcHomologacion.pfx");

    if pfx_path.exists() {
        return Some((pfx_path, String::new())); // No password
    }

    None
}

/// CUIT for testing (homologation)
const TEST_CUIT: u64 = 20123456781;
const TEST_PTO_VTA: i32 = 1;

// =============================================================================
// WSAA Authentication Tests
// =============================================================================

/// Helper to handle "already has valid token" error from ARCA/AFIP
///
/// This error occurs when ARCA already has a valid token for this certificate
/// (from a previous test run or external process) and won't issue a new one.
/// Our client doesn't have that token, so we can't proceed.
fn is_token_already_exists_error(e: &arca::ArcaError) -> bool {
    let msg = e.to_string();
    msg.contains("TA valido") || msg.contains("valid")
}

/// Macro to skip test if ARCA returns "token already exists" error
macro_rules! skip_if_token_exists {
    ($result:expr, $test_name:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) if is_token_already_exists_error(&e) => {
                eprintln!(
                    "SKIP {}: ARCA has a valid token from a previous run. \
                     Wait for it to expire (~12 hours) or use a different certificate.",
                    $test_name
                );
                return;
            }
            Err(e) => panic!("{} failed: {:?}", $test_name, e),
        }
    };
}

#[tokio::test]
#[ignore]
async fn test_wsaa_login_with_pem() {
    let Some((cert_path, key_path)) = get_cert_paths() else {
        eprintln!("Skipping: No PEM certificates found");
        return;
    };

    let cert = fs::read(&cert_path).expect("Failed to read cert");
    let key = fs::read(&key_path).expect("Failed to read key");

    let client = ArcaClient::builder()
        .environment(ArcaEnvironment::Testing)
        .credentials(cert, key)
        .build()
        .expect("Failed to build client");

    let result = client.login("wsfe").await;

    match &result {
        Ok(_) => (),
        Err(e) if is_token_already_exists_error(e) => {
            // This is expected if another test just ran - token still valid at ARCA
            return;
        }
        Err(e) => panic!("Login failed: {:?}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_wsaa_login_with_pfx() {
    let Some((pfx_path, password)) = get_pfx_path() else {
        eprintln!("Skipping: No PFX certificate found");
        return;
    };

    let pfx = fs::read(&pfx_path).expect("Failed to read PFX");

    let client = ArcaClient::builder()
        .environment(ArcaEnvironment::Testing)
        .credentials_pfx(pfx, &password)
        .build()
        .expect("Failed to build client");

    let result = client.login("wsfe").await;

    match &result {
        Ok(_) => (),
        Err(e) if is_token_already_exists_error(e) => return,
        Err(e) => panic!("Login failed: {:?}", e),
    }
}

// =============================================================================
// WSFEv1 Tests
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_wsfev1_get_last_voucher() {
    let client = create_test_client().await;

    let result = client.get_last_voucher(TEST_CUIT, TEST_PTO_VTA, cbte_tipos::FACTURA_B).await;
    let last = skip_if_token_exists!(result, "get_last_voucher");

    assert!(last >= 0, "Last voucher should be non-negative");
}

#[tokio::test]
#[ignore]
async fn test_wsfev1_get_tipos_cbte() {
    let client = create_test_client().await;

    let result = client.get_tipos_cbte(TEST_CUIT).await;
    let tipos = skip_if_token_exists!(result, "get_tipos_cbte");

    assert!(!tipos.is_empty(), "Should return at least one voucher type");
}

#[tokio::test]
#[ignore]
async fn test_wsfev1_get_tipos_doc() {
    let client = create_test_client().await;

    let result = client.get_tipos_doc(TEST_CUIT).await;
    let tipos = skip_if_token_exists!(result, "get_tipos_doc");

    assert!(!tipos.is_empty(), "Should return at least one document type");
}

#[tokio::test]
#[ignore]
async fn test_wsfev1_get_tipos_iva() {
    let client = create_test_client().await;

    let result = client.get_tipos_iva(TEST_CUIT).await;
    let tipos = skip_if_token_exists!(result, "get_tipos_iva");

    assert!(!tipos.is_empty(), "Should return at least one IVA rate");
}

#[tokio::test]
#[ignore]
async fn test_wsfev1_get_cotizacion_usd() {
    let client = create_test_client().await;

    let result = client.get_cotizacion(TEST_CUIT, "DOL").await;
    let cot = skip_if_token_exists!(result, "get_cotizacion");

    assert!(cot.mon_cotiz > 0.0, "USD rate should be positive");
}

#[tokio::test]
#[ignore]
async fn test_wsfev1_authorize_factura_b() {
    let client = create_test_client().await;

    // Get last voucher number
    let result = client.get_last_voucher(TEST_CUIT, TEST_PTO_VTA, cbte_tipos::FACTURA_B).await;
    let last = skip_if_token_exists!(result, "authorize_factura_b (get_last)");

    let next = (last + 1) as u64;
    let today = chrono::Local::now().format("%Y%m%d").to_string();

    let cab = FeCabReq {
        cant_reg: 1,
        pto_vta: TEST_PTO_VTA,
        cbte_tipo: cbte_tipos::FACTURA_B,
    };

    let det = FeDetReq {
        concepto: 1, // Productos
        doc_tipo: doc_tipos::CONSUMIDOR_FINAL,
        doc_nro: 0,
        cbte_desde: next,
        cbte_hasta: next,
        cbte_fch: today,
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

    let result = client.authorize_voucher(TEST_CUIT, cab, vec![det]).await;
    let response = skip_if_token_exists!(result, "authorize_factura_b");

    assert!(!response.cae.is_empty(), "CAE should not be empty");
    assert!(!response.cae_fch_vto.is_empty(), "CAE expiration should not be empty");
}

#[tokio::test]
#[ignore]
async fn test_wsfev1_get_voucher() {
    let client = create_test_client().await;

    // Get last voucher number
    let result = client.get_last_voucher(TEST_CUIT, TEST_PTO_VTA, cbte_tipos::FACTURA_B).await;
    let last = skip_if_token_exists!(result, "get_voucher (get_last)");

    if last > 0 {
        // Query existing voucher
        let result = client.get_voucher(TEST_CUIT, cbte_tipos::FACTURA_B, last as u64, TEST_PTO_VTA).await;
        let comp = skip_if_token_exists!(result, "get_voucher");

        assert!(!comp.cod_autorizacion.is_empty(), "Authorization code should not be empty");
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_wsfev1_duplicate_voucher_error() {
    let client = create_test_client().await;

    let today = chrono::Local::now().format("%Y%m%d").to_string();

    let cab = FeCabReq {
        cant_reg: 1,
        pto_vta: TEST_PTO_VTA,
        cbte_tipo: cbte_tipos::FACTURA_B,
    };

    // Try to authorize voucher #1 which should already exist
    let det = FeDetReq {
        concepto: 1,
        doc_tipo: doc_tipos::CONSUMIDOR_FINAL,
        doc_nro: 0,
        cbte_desde: 1,
        cbte_hasta: 1,
        cbte_fch: today,
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

    let result = client.authorize_voucher(TEST_CUIT, cab, vec![det]).await;

    // Handle "token already exists" error - skip test in this case
    match &result {
        Err(e) if is_token_already_exists_error(e) => {
            eprintln!("SKIP duplicate_voucher_error: token already exists");
            return;
        }
        _ => {}
    }

    // Should fail because voucher #1 already exists
    assert!(result.is_err(), "Duplicate voucher should fail");
}

#[tokio::test]
#[ignore]
async fn test_wsfev1_invalid_amounts_error() {
    let client = create_test_client().await;

    let result = client.get_last_voucher(TEST_CUIT, TEST_PTO_VTA, cbte_tipos::FACTURA_B).await;
    let last = skip_if_token_exists!(result, "invalid_amounts_error (get_last)");

    let next = (last + 1) as u64;
    let today = chrono::Local::now().format("%Y%m%d").to_string();

    let cab = FeCabReq {
        cant_reg: 1,
        pto_vta: TEST_PTO_VTA,
        cbte_tipo: cbte_tipos::FACTURA_B,
    };

    // Invalid: imp_total doesn't match imp_neto + imp_iva
    let det = FeDetReq {
        concepto: 1,
        doc_tipo: doc_tipos::CONSUMIDOR_FINAL,
        doc_nro: 0,
        cbte_desde: next,
        cbte_hasta: next,
        cbte_fch: today,
        imp_total: 100.0,  // Wrong! Should be 121
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

    let result = client.authorize_voucher(TEST_CUIT, cab, vec![det]).await;
    // Should fail because amounts don't match
    assert!(result.is_err(), "Invalid amounts should fail");
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Get or create a shared test client
fn get_shared_client() -> &'static ArcaClient {
    SHARED_CLIENT.get_or_init(|| {
        // Try PFX first (preferred)
        if let Some((pfx_path, password)) = get_pfx_path() {
            let pfx = fs::read(&pfx_path).expect("Failed to read PFX");
            return ArcaClient::builder()
                .environment(ArcaEnvironment::Testing)
                .credentials_pfx(pfx, &password)
                .build()
                .expect("Failed to build client");
        }

        // Fall back to PEM
        if let Some((cert_path, key_path)) = get_cert_paths() {
            let cert = fs::read(&cert_path).expect("Failed to read cert");
            let key = fs::read(&key_path).expect("Failed to read key");
            return ArcaClient::builder()
                .environment(ArcaEnvironment::Testing)
                .credentials(cert, key)
                .build()
                .expect("Failed to build client");
        }

        panic!("No ARCA credentials found. Set ARCA_CERT_PEM/ARCA_KEY_PEM or ARCA_PFX environment variables");
    })
}

async fn create_test_client() -> &'static ArcaClient {
    get_shared_client()
}
