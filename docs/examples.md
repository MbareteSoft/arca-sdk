# Ejemplos Detallados

Ejemplos completos de uso de ARCA SDK en Rust.

## Cliente de Alto Nivel (Recomendado)

### Con PFX + Persistencia de Tokens

```rust
use arca::{ArcaClient, ArcaEnvironment};
use arca::client::token_store::FileSystemTokenStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pfx = std::fs::read("certificado.pfx")?;

    // Crear store para persistencia de tokens
    let store = FileSystemTokenStore::new("./tokens_cache");

    let client = ArcaClient::builder()
        .environment(ArcaEnvironment::Testing)
        .credentials_pfx(pfx, "")  // Password vacio si no tiene
        .token_store(store)         // Persistencia en disco
        .build()?;

    // Login automatico con cache de 3 niveles:
    // 1. RAM (HashMap) -> 2. Disco (arca_token_wsfe.json) -> 3. WSAA
    client.login("wsfe").await?;

    // Obtener ultimo comprobante (login automatico si es necesario)
    let ultimo = client.get_last_voucher(20123456789, 1, 11).await?;
    println!("Ultimo comprobante: {}", ultimo);

    Ok(())
}
```

### Con PEM (Certificado + Clave Privada)

```rust
use arca::{ArcaClient, ArcaEnvironment};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cert = std::fs::read("certificado.pem")?;
    let key = std::fs::read("clave_privada.pem")?;

    let client = ArcaClient::builder()
        .environment(ArcaEnvironment::Testing)
        .credentials(cert, key)
        .build()?;

    client.login("wsfe").await?;
    println!("Autenticado!");

    Ok(())
}
```

## Autenticacion de Bajo Nivel

### Usando auth directamente

```rust
use arca::{auth, transport::HttpClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cert = std::fs::read("certificado.pem")?;
    let key = std::fs::read("clave_privada.pem")?;
    let http = HttpClient::new()?;

    let wsaa_url = "https://wsaahomo.afip.gov.ar/ws/services/LoginCms";
    let login_ticket = auth::build_login_ticket_request("wsfe");
    let cms = auth::sign_cms_base64(&login_ticket, &cert, &key)?;
    let ticket = auth::wsaa_login(&http, wsaa_url, &cms).await?;

    println!("Token obtenido!");
    println!("Expira: {}", ticket.expiration_time);

    Ok(())
}
```

## Facturacion Electronica

### Factura C (Monotributo a Consumidor Final)

```rust
use arca::{
    ArcaClient, ArcaEnvironment, FeCabReq, FeDetReq,
    cbte_tipos, doc_tipos, condicion_iva,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ArcaClient::builder()
        .environment(ArcaEnvironment::Testing)
        .credentials_pfx(std::fs::read("cert.pfx")?, "")
        .build()?;

    let cuit: u64 = 20123456789;
    let pto_vta = 1;

    // Obtener ultimo numero autorizado
    let ultimo = client.get_last_voucher(cuit, pto_vta, cbte_tipos::FACTURA_C).await?;
    let nuevo = (ultimo + 1) as u64;
    let fecha = chrono::Local::now().format("%Y%m%d").to_string();

    let cab = FeCabReq {
        cant_reg: 1,
        pto_vta,
        cbte_tipo: cbte_tipos::FACTURA_C,
    };

    // Factura C NO discrimina IVA
    let det = FeDetReq {
        concepto: 1,                              // Productos
        doc_tipo: doc_tipos::CONSUMIDOR_FINAL,    // 99
        doc_nro: 0,
        cbte_desde: nuevo,
        cbte_hasta: nuevo,
        cbte_fch: fecha,
        imp_total: 1000.0,
        imp_tot_conc: 0.0,
        imp_neto: 1000.0,                         // Total = Neto
        imp_op_ex: 0.0,
        imp_trib: 0.0,
        imp_iva: 0.0,                             // Sin IVA discriminado
        fch_serv_desde: None,
        fch_serv_hasta: None,
        fch_vto_pago: None,
        mon_id: "PES".to_string(),
        mon_cotiz: 1.0,
        condicion_iva_receptor: Some(condicion_iva::CONSUMIDOR_FINAL),
        iva: vec![],                              // Vacio para Factura C
    };

    let resp = client.authorize_voucher(cuit, cab, vec![det]).await?;

    println!("Factura C autorizada!");
    println!("CAE: {}", resp.cae);
    println!("Vencimiento: {}", resp.cae_fch_vto);

    Ok(())
}
```

### Factura B (Responsable Inscripto a Consumidor Final)

```rust
use arca::{FeCabReq, FeDetReq, AlicIva, cbte_tipos, doc_tipos, condicion_iva, alicuotas_iva};

// Factura B por $1210 (neto $1000 + IVA 21%)
let cab = FeCabReq {
    cant_reg: 1,
    pto_vta: 1,
    cbte_tipo: cbte_tipos::FACTURA_B,
};

let det = FeDetReq {
    concepto: 1,
    doc_tipo: doc_tipos::CONSUMIDOR_FINAL,
    doc_nro: 0,
    cbte_desde: nuevo,
    cbte_hasta: nuevo,
    cbte_fch: fecha,
    imp_total: 1210.0,                            // Neto + IVA
    imp_tot_conc: 0.0,
    imp_neto: 1000.0,
    imp_op_ex: 0.0,
    imp_trib: 0.0,
    imp_iva: 210.0,                               // 21% de 1000
    fch_serv_desde: None,
    fch_serv_hasta: None,
    fch_vto_pago: None,
    mon_id: "PES".to_string(),
    mon_cotiz: 1.0,
    condicion_iva_receptor: Some(condicion_iva::CONSUMIDOR_FINAL),
    iva: vec![AlicIva {
        id: alicuotas_iva::VEINTIUNO_PORCIENTO,
        base_imp: 1000.0,
        importe: 210.0,
    }],
};

let resp = client.authorize_voucher(cuit, cab, vec![det]).await?;
```

### Factura A (Responsable Inscripto a Responsable Inscripto)

```rust
// Factura A requiere CUIT del receptor
let det = FeDetReq {
    concepto: 1,
    doc_tipo: doc_tipos::CUIT,                    // 80
    doc_nro: 30123456789,                         // CUIT del receptor
    cbte_desde: nuevo,
    cbte_hasta: nuevo,
    cbte_fch: fecha,
    imp_total: 1210.0,
    imp_tot_conc: 0.0,
    imp_neto: 1000.0,
    imp_op_ex: 0.0,
    imp_trib: 0.0,
    imp_iva: 210.0,
    fch_serv_desde: None,
    fch_serv_hasta: None,
    fch_vto_pago: None,
    mon_id: "PES".to_string(),
    mon_cotiz: 1.0,
    condicion_iva_receptor: Some(condicion_iva::IVA_RESPONSABLE_INSCRIPTO),
    iva: vec![AlicIva {
        id: alicuotas_iva::VEINTIUNO_PORCIENTO,
        base_imp: 1000.0,
        importe: 210.0,
    }],
};
```

### Factura de Servicios

```rust
// Para servicios (concepto 2 o 3), los campos de fecha son obligatorios
let det = FeDetReq {
    concepto: 2,                                  // Servicios
    doc_tipo: doc_tipos::CONSUMIDOR_FINAL,
    doc_nro: 0,
    cbte_desde: nuevo,
    cbte_hasta: nuevo,
    cbte_fch: "20260217".to_string(),
    imp_total: 1000.0,
    imp_tot_conc: 0.0,
    imp_neto: 1000.0,
    imp_op_ex: 0.0,
    imp_trib: 0.0,
    imp_iva: 0.0,
    fch_serv_desde: Some("20260101".to_string()), // Inicio del servicio
    fch_serv_hasta: Some("20260131".to_string()), // Fin del servicio
    fch_vto_pago: Some("20260215".to_string()),   // Vencimiento de pago
    mon_id: "PES".to_string(),
    mon_cotiz: 1.0,
    condicion_iva_receptor: Some(condicion_iva::CONSUMIDOR_FINAL),
    iva: vec![],
};
```

### Multiples Alicuotas IVA

```rust
// Factura con productos a diferentes tasas de IVA
let det = FeDetReq {
    concepto: 1,
    doc_tipo: doc_tipos::CUIT,
    doc_nro: 30123456789,
    cbte_desde: nuevo,
    cbte_hasta: nuevo,
    cbte_fch: fecha,
    imp_total: 2480.0,                            // 1000 + 210 + 1000 + 270
    imp_tot_conc: 0.0,
    imp_neto: 2000.0,                             // 1000 + 1000
    imp_op_ex: 0.0,
    imp_trib: 0.0,
    imp_iva: 480.0,                               // 210 + 270
    fch_serv_desde: None,
    fch_serv_hasta: None,
    fch_vto_pago: None,
    mon_id: "PES".to_string(),
    mon_cotiz: 1.0,
    condicion_iva_receptor: Some(condicion_iva::IVA_RESPONSABLE_INSCRIPTO),
    iva: vec![
        AlicIva {
            id: alicuotas_iva::VEINTIUNO_PORCIENTO,   // 21%
            base_imp: 1000.0,
            importe: 210.0,
        },
        AlicIva {
            id: alicuotas_iva::VEINTISIETE_PORCIENTO, // 27%
            base_imp: 1000.0,
            importe: 270.0,
        },
    ],
};
```

## Manejo de Errores Detallado

```rust
use arca::WsFev1Service;

// Usando el servicio de bajo nivel con respuesta detallada
let resp = wsfev1.fecae_solicitar_detallado(
    &token, &sign, cuit, cab, vec![det]
).await?;

if resp.is_approved() {
    println!("Aprobado!");
    println!("CAE: {}", resp.cae);
    println!("Vencimiento: {}", resp.cae_fch_vto);
} else if resp.is_rejected() {
    println!("Rechazado");

    // Errores
    for err in &resp.detalles.errors {
        println!("Error [{}]: {}", err.code, err.msg);

        // Descripcion extendida del catalogo
        let desc = arca::errores::descripcion(err.code as i32);
        println!("  -> {}", desc);
    }

    // Observaciones (pueden ser warnings)
    for obs in &resp.detalles.observaciones {
        println!("Obs [{}]: {}", obs.code, obs.msg);
    }
} else if resp.is_partial() {
    println!("Parcialmente aprobado");
}
```

## Consultas

### Ultimo Comprobante Autorizado

```rust
// Con cliente de alto nivel
let ultimo = client.get_last_voucher(cuit, pto_vta, cbte_tipos::FACTURA_C).await?;
println!("Ultimo comprobante: {}", ultimo);

// Con servicio de bajo nivel
let ultimo = wsfev1.fe_comp_ultimo_autorizado(
    &token, &sign, cuit, pto_vta, cbte_tipos::FACTURA_C
).await?;
```

### Consultar Comprobante Emitido

```rust
// Con cliente de alto nivel
let comp = client.get_voucher(cuit, cbte_tipos::FACTURA_C, 123, pto_vta).await?;
println!("Fecha: {}", comp.cbte_fch);
println!("Total: ${}", comp.imp_total);
println!("CAE: {}", comp.cod_autorizacion);

// Con servicio de bajo nivel
let comp = wsfev1.fe_comp_consultar(
    &token, &sign, cuit, cbte_tipos::FACTURA_C, 123, pto_vta
).await?;
```

### Tipos de Comprobante Disponibles

```rust
let tipos = client.get_tipos_cbte(cuit).await?;
for t in tipos {
    println!("{}: {} (desde: {})", t.id, t.desc, t.fch_desde);
}
```

### Cotizacion de Moneda

```rust
let cot = client.get_cotizacion(cuit, "DOL").await?;
println!("Cotizacion USD: ${}", cot.mon_cotiz);
```

## Persistencia de Tokens

### Patron de Produccion Recomendado

```rust
use arca::{ArcaClient, ArcaEnvironment};
use arca::client::token_store::FileSystemTokenStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Crear store (el directorio se crea automaticamente)
    let store = FileSystemTokenStore::new("./tokens_cache");

    // 2. Crear cliente con persistencia
    let client = ArcaClient::builder()
        .environment(ArcaEnvironment::Production)
        .credentials_pfx(std::fs::read("certificado.pfx")?, "password")
        .token_store(store)
        .build()?;

    // 3. Primera ejecucion: contacta WSAA, guarda en RAM + disco
    //    Ejecuciones siguientes: lee del disco si el token es valido
    client.login("wsfe").await?;

    // 4. Usar normalmente
    let ultimo = client.get_last_voucher(20123456781, 1, 6).await?;
    println!("Ultimo comprobante: {}", ultimo);

    Ok(())
}
```

### Archivos generados

```
./tokens_cache/
├── arca_token_wsfe.json             # Token para facturacion
└── arca_token_ws_sr_padron_a13.json # Token para padron (si se usa)
```

Cada archivo contiene:
```json
{
  "token": "PD94bWwg...",
  "sign": "WkNERWJ4...",
  "expiration_time": "2026-02-17T23:59:59-03:00"
}
```

## Padron de Contribuyentes

```rust
use arca::{ArcaClient, ArcaEnvironment, PadronA13Service};
use arca::transport::HttpClient;

// Con servicio de bajo nivel
let http = HttpClient::new()?;
let padron = PadronA13Service::testing();

// Health check (sin autenticacion)
let dummy = padron.dummy(&http).await?;
println!("App: {}, Auth: {}, DB: {}", dummy.app_server, dummy.auth_server, dummy.db_server);

// Consultar CUIT (requiere autenticacion con ws_sr_padron_a13)
let persona = padron.get_persona(
    &http, &token, &sign, mi_cuit, cuit_consulta
).await?;

println!("Nombre: {}", persona.persona.nombre_completo());
println!("Estado: {}", persona.persona.estado_clave);
```

## CAEA (Modo Contingencia)

```rust
// Solicitar CAEA para la primera quincena de febrero 2026
// Parametros: cuit, periodo (YYYYMM), orden (1=primera quincena, 2=segunda)
let caea = client.request_caea(cuit, 202602, 1).await?;

println!("CAEA: {}", caea.caea);
println!("Vencimiento: {}", caea.fch_vto_caea);

// Consultar CAEA existente
let caea = client.get_caea(cuit, 202602, 1).await?;

// Registrar facturas emitidas offline con CAEA
let resp = client.register_caea_invoices(cuit, cab, vec![det], &caea.caea).await?;

// Declarar periodo sin movimiento si no hubo ventas
client.report_caea_no_movement(cuit, pto_vta, &caea.caea).await?;

// Consultar si un periodo fue declarado sin movimiento
let sin_mov = client.get_caea_no_movement(cuit, &caea.caea, pto_vta).await?;
```

## Ejemplos Incluidos

El directorio `examples/` contiene tests ejecutables:

```bash
# Suite completa de homologacion (14 tests)
cargo run --example test_homologacion

# Factura C en produccion ($1)
cargo run --example test_produccion

# Test con certificado PFX (homologacion)
cargo run --example test_pfx

# Test con PFX en ambos entornos
cargo run --example test_pfx_ambos

# Test de persistencia de tokens
cargo run --example test_cae_persistence

# Debug WSAA (solo autenticacion)
cargo run --example debug_wsaa

# Debug CAE (facturacion con detalles)
cargo run --example debug_cae

# Consultar padron dummy
cargo run --example test_padron_dummy

# Nota de credito en produccion
cargo run --example nota_credito_produccion

# Suite completa de integracion
cargo run --example test_integracion_completo
```
