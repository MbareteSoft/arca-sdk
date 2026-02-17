//! Test CAE en homologación
//!
//! Ejecutar con:
//! cargo run --example test_cae

use arca::{ArcaClient, ArcaEnvironment, AlicIva, FeCabReq, FeDetReq};
use std::fs;

const CUIT: u64 = 20123456781;
const PTO_VTA: i32 = 1; // Punto de venta - ajustar si es necesario
const CBTE_TIPO: i32 = 6; // 6 = Factura B, 11 = Factura C

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Test CAE en Homologación ===\n");

    // Cargar certificados
    let cert_path = "../certificado/cert.pem";
    let key_path = "../certificado/key.pem";

    println!("Cargando certificados...");
    let cert = fs::read(cert_path)?;
    let key = fs::read(key_path)?;

    // Crear cliente en entorno de testing (homologación)
    let client = ArcaClient::builder()
        .environment(ArcaEnvironment::Testing)
        .credentials(cert, key)
        .build()?;

    println!("Entorno: {:?}", client.environment());
    println!("CUIT: {}\n", CUIT);

    // 1. Login (WSAA)
    println!("1. Autenticando con WSAA...");
    client.login("wsfe").await?;
    println!("   ✓ Login exitoso\n");

    // 2. Obtener último comprobante autorizado
    println!("2. Consultando último comprobante...");
    let ultimo = client.get_last_voucher(CUIT, PTO_VTA, CBTE_TIPO).await?;
    println!("   Último comprobante tipo {} en pto vta {}: {}\n", CBTE_TIPO, PTO_VTA, ultimo);

    let nuevo_cbte = (ultimo + 1) as u64;
    println!("   Nuevo comprobante a emitir: {}\n", nuevo_cbte);

    // 3. Preparar factura para consumidor final
    println!("3. Preparando factura...");

    let fecha_hoy = chrono::Local::now().format("%Y%m%d").to_string();

    let cab = FeCabReq {
        cant_reg: 1,
        pto_vta: PTO_VTA,
        cbte_tipo: CBTE_TIPO,
    };

    // Consumidor final: DocTipo=99, DocNro=0
    let det = FeDetReq {
        concepto: 1, // 1=Productos, 2=Servicios, 3=Productos y Servicios
        doc_tipo: 99, // 99 = Consumidor Final
        doc_nro: 0,
        cbte_desde: nuevo_cbte,
        cbte_hasta: nuevo_cbte,
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
        condicion_iva_receptor: Some(5), // 5 = Consumidor Final (mandatory since RG 5616)
        iva: vec![AlicIva {
            id: 5, // 5 = 21%
            base_imp: 100.0,
            importe: 21.0,
        }],
    };

    println!("   Fecha: {}", fecha_hoy);
    println!("   Comprobante: {} - {}", nuevo_cbte, nuevo_cbte);
    println!("   Total: ${:.2}", det.imp_total);
    println!("   Neto: ${:.2} + IVA: ${:.2}\n", det.imp_neto, det.imp_iva);

    // 4. Solicitar CAE
    println!("4. Solicitando CAE...");
    match client.authorize_voucher(CUIT, cab, vec![det]).await {
        Ok(response) => {
            println!("\n   ✓ CAE OBTENIDO EXITOSAMENTE");
            println!("   ================================");
            println!("   CAE: {}", response.cae);
            println!("   Vencimiento: {}", response.cae_fch_vto);
            println!("   ================================\n");
        }
        Err(e) => {
            println!("\n   ✗ Error al solicitar CAE:");
            println!("   {}\n", e);
            return Err(e.into());
        }
    }

    // 5. Verificar consultando el comprobante
    println!("5. Verificando comprobante emitido...");
    match client.get_voucher(CUIT, CBTE_TIPO, nuevo_cbte, PTO_VTA).await {
        Ok(comp) => {
            println!("   ✓ Comprobante verificado");
            println!("   Resultado: {}", comp.resultado);
            println!("   CAE: {}", comp.cod_autorizacion);
            println!("   Vto: {}", comp.fch_vto);
        }
        Err(e) => {
            println!("   Error al verificar: {}", e);
        }
    }

    println!("\n=== Test completado ===");
    Ok(())
}
