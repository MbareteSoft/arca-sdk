//! Test CAE with built-in token persistence
//!
//! Ejecutar con:
//! cargo run --example test_cae_persistence

use arca::client::token_store::FileSystemTokenStore;
use arca::{ArcaClient, ArcaEnvironment};
use std::fs;

const CUIT: u64 = 20123456781;
const PTO_VTA: i32 = 1;
const CBTE_TIPO: i32 = 6; // Factura B

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Test de Persistencia de Tokens ===
");

    // 1. Configurar el almacén persistente
    let cache_dir = "./tokens_cache";
    let store = FileSystemTokenStore::new(cache_dir);
    println!("Directorio de cache: {}", cache_dir);

    // 2. Cargar certificados (PEM)
    // Nota: También podrías usar .credentials_pfx(data, password)
    let cert = fs::read("../certificado/cert.pem")?;
    let key = fs::read("../certificado/key.pem")?;

    // 3. Crear el cliente con el almacén de tokens
    let client = ArcaClient::builder()
        .environment(ArcaEnvironment::Testing)
        .credentials(cert, key)
        .token_store(store) // <--- Activa la persistencia
        .build()?;

    // 4. Ejecutar login
    // La primera vez contactará a AFIP.
    // La segunda vez (si el archivo existe y es válido) usará el disco.
    println!("1. Solicitando login/token...");
    client.login("wsfe").await?;
    println!("   ✓ Login exitoso (revisa el directorio '{}' para ver el JSON)
", cache_dir);

    // 5. Realizar una consulta para verificar que el token funciona
    println!("2. Consultando último comprobante para validar el token...");
    let ultimo = client.get_last_voucher(CUIT, PTO_VTA, CBTE_TIPO).await?;
    println!("   Último comprobante: {}
", ultimo);

    println!("=== Test completado exitosamente ===");
    println!("Si vuelves a ejecutar este ejemplo, verás que es más rápido y no");
    println!("genera una nueva firma CMS (a menos que el token haya expirado).");

    Ok(())
}
