//! Test with PFX using native Rust API
//!
//! Ejecutar con:
//! cargo run --example test_pfx_native

use arca::{ArcaClient, ArcaEnvironment};
use std::fs;

fn main() -> anyhow::Result<()> {
    // We need tokio runtime for the async methods later, 
    // but the builder itself is synchronous.
    let rt = tokio::runtime::Runtime::new()?;
    
    rt.block_on(async {
        println!("=== Test con PFX (API Nativa) ===
");

        // 1. Leer datos del PFX
        let pfx_path = "../certificado/certificado2024GcHomologacion.pfx";
        println!("1. Cargando PFX desde: {}", pfx_path);
        let pfx_data = fs::read(pfx_path)?;

        // 2. Crear el cliente usando .credentials_pfx()
        // El password suele estar vacío en certificados de AFIP para homologación.
        println!("2. Creando cliente...");
        let client = ArcaClient::builder()
            .environment(ArcaEnvironment::Testing)
            .credentials_pfx(pfx_data, "")
            .build()?;
        println!("   ✓ Cliente creado exitosamente
");

        // 3. Probar login
        println!("3. Probando login en WSAA...");
        match client.login("wsfe").await {
            Ok(_) => println!("   ✓ Login exitoso
"),
            Err(e) => {
                println!("   ✗ Error en login: {}", e);
                return Err(e.into());
            }
        }

        // 4. Probar una consulta simple
        println!("4. Consultando último comprobante...");
        let cuit = 20123456781;
        match client.get_last_voucher(cuit, 1, 11).await {
            Ok(nro) => println!("   Último comprobante (Factura C): {}
", nro),
            Err(e) => println!("   ✗ Error al consultar: {}
", e),
        }

        println!("=== Test completado ===");
        Ok(())
    })
}
