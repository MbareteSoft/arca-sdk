# ARCA SDK

SDK Rust para integracion con los servicios web de **ARCA** (ex AFIP) de Argentina.

## Caracteristicas

- **Autenticacion WSAA** - Login con certificados digitales (PEM o PFX/PKCS#12)
- **Facturacion Electronica WSFEv1** - Facturas, notas de credito/debito, CAE
- **CAEA** - Modo contingencia (Codigo de Autorizacion Electronico Anticipado)
- **Padron A13** - Consulta de contribuyentes por CUIT
- **DLL/FFI** - Uso desde C#, Python, Delphi y otros lenguajes (22 funciones exportadas)
- **Persistencia de Tokens** - Cache hibrida RAM + disco para evitar lockouts de 12h
- **Thread-safe** - Cliente con `Arc<RwLock<>>` y semaforo para uso concurrente
- **Seguridad de Memoria** - Zeroizacion de credenciales y tokens sensibles con `zeroize`

## Instalacion

```toml
[dependencies]
arca-sdk = { path = "path/to/arca-sdk" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Inicio Rapido

### Rust - Con PFX + Persistencia

```rust
use arca::{ArcaClient, ArcaEnvironment};
use arca::client::token_store::FileSystemTokenStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = FileSystemTokenStore::new("./tokens_cache");

    let client = ArcaClient::builder()
        .environment(ArcaEnvironment::Testing)
        .credentials_pfx(std::fs::read("cert.pfx")?, "password")
        .token_store(store)  // Persiste tokens en disco
        .build()?;

    // Login con cache de 3 niveles: RAM -> Disco -> WSAA
    client.login("wsfe").await?;

    let ultimo = client.get_last_voucher(20123456789, 1, 11).await?;
    println!("Ultimo comprobante: {}", ultimo);

    Ok(())
}
```

### Rust - Emitir Factura

```rust
use arca::{FeCabReq, FeDetReq, cbte_tipos, doc_tipos, condicion_iva};

let cab = FeCabReq { cant_reg: 1, pto_vta: 1, cbte_tipo: cbte_tipos::FACTURA_C };
let det = FeDetReq {
    concepto: 1, doc_tipo: doc_tipos::CONSUMIDOR_FINAL, doc_nro: 0,
    cbte_desde: 1, cbte_hasta: 1,
    cbte_fch: "20260217".to_string(),
    imp_total: 1000.0, imp_neto: 1000.0, imp_iva: 0.0,
    imp_tot_conc: 0.0, imp_op_ex: 0.0, imp_trib: 0.0,
    fch_serv_desde: None, fch_serv_hasta: None, fch_vto_pago: None,
    mon_id: "PES".to_string(), mon_cotiz: 1.0,
    condicion_iva_receptor: Some(condicion_iva::CONSUMIDOR_FINAL),
    iva: vec![],
};

let resp = client.authorize_voucher(cuit, cab, vec![det]).await?;
println!("CAE: {}", resp.cae);
```

## Uso desde otros lenguajes (DLL/FFI)

```bash
# Compilar DLL
cargo build --release  # genera target/release/libarca.so (~7.5 MB)
```

### C#

```csharp
using var client = ArcaClient.ProductionPfx("cert.pfx", "");
client.SetTokenCache("./cache");
client.Login("wsfe");
var ultimo = client.FeCompUltimoAutorizado(cuit, 1, 11);
```

### Python

```python
with ArcaClient.production_pfx("cert.pfx", "") as client:
    client.set_token_cache("./cache")
    client.login("wsfe")
    ultimo = client.fe_comp_ultimo_autorizado(cuit, 1, 11)
```

### Delphi

```pascal
Client := TArcaClient.CreateProductionPfx('cert.pfx', '');
try
  Client.SetTokenCache('./cache');
  Client.Login('wsfe');
  Ultimo := Client.FeCompUltimoAutorizado(Cuit, 1, 11);
finally
  Client.Free;
end;
```

Ver `examples/ffi/` para wrappers completos en C#, Python y Delphi.

## Persistencia de Tokens

WSAA otorga tokens con validez de ~12 horas y rechaza nuevas solicitudes mientras uno este vigente. Sin persistencia, un reinicio del proceso causa un lockout irrecuperable.

El SDK implementa cache de 3 niveles:

```
login("wsfe")
  |-- RAM: token valido en HashMap? -> OK (instantaneo)
  |-- Disco: arca_token_wsfe.json valido? -> cargar a RAM -> OK
  |-- WSAA: login completo -> guardar en RAM + disco -> OK
```

Ver [DOCUMENTACION_PERSISTENCIA.md](DOCUMENTACION_PERSISTENCIA.md) para guia detallada.

## Ejemplos

```bash
cargo run --example test_homologacion    # Suite completa (14 tests)
cargo run --example test_produccion      # Factura C en produccion
cargo run --example test_pfx             # Test con certificado PFX
cargo run --example test_cae_persistence # Test de persistencia de tokens
```

## Tests

```bash
cargo test       # 39 tests unitarios
cargo build      # Verificar compilacion
```

## Documentacion

| Documento | Contenido |
|-----------|-----------|
| [docs/api.md](docs/api.md) | Referencia completa de API |
| [docs/ffi.md](docs/ffi.md) | Guia FFI/DLL con ejemplos |
| [docs/examples.md](docs/examples.md) | Ejemplos detallados Rust |
| [docs/errors.md](docs/errors.md) | Codigos de error ARCA + FFI |
| [DOCUMENTACION_PERSISTENCIA.md](DOCUMENTACION_PERSISTENCIA.md) | Sistema de persistencia de tokens |

## Arquitectura

```
src/
├── auth/           # WSAA: CMS signing, ticket, login
├── client/         # ArcaClient, builder, config, token_store
├── constants/      # Endpoints, defaults, namespaces
├── services/       # WSFEv1, Padron A13, trait ArcaService
├── transport/      # HTTP client, SOAP envelopes
├── xml/            # Parsing de respuestas XML
├── ffi/            # 11 modulos FFI (22 funciones exportadas)
├── error.rs        # ArcaError (10 variantes, thiserror)
└── lib.rs          # Re-exports publicos

include/arca.h      # Header C para consumidores FFI
examples/ffi/       # Wrappers C#, Python, Delphi
```

## Requisitos

- Rust 1.85+ (edition 2024)
- OpenSSL (para firma CMS)
- Certificado digital ARCA/AFIP (`.pem` o `.pfx`)

## Dependencias Principales

| Crate | Version | Uso |
|-------|---------|-----|
| `tokio` | 1.x | Runtime asincrono |
| `reqwest` | 0.13 | Cliente HTTP |
| `quick-xml` | 0.39 | Parsing XML |
| `openssl` | 0.10 (vendored) | Firma CMS |
| `serde` / `serde_json` | 1.x | Serializacion JSON |
| `thiserror` | 2.x | Tipos de error |
| `zeroize` | 1.8 | Limpieza de memoria sensible |
| `tempfile` | 3.x | Archivos temporales para PFX |
| `validator` | 0.20 | Validacion de campos FFI |
| `async-trait` | 0.1 | Traits asincronos (TokenStore) |

## Notas

- **Condicion IVA Receptor**: Obligatorio desde RG 5616, rechazado a partir de 02/2026
- **PFX Legacy**: Soporta RC2-40-CBC con `-provider legacy` para certificados ARCA
- **Token Cache**: Los tokens WSAA duran ~12 horas, con margen configurable (default: 5 min)
- **Seguridad**: Password del PFX se pasa por stdin (no visible en `ps aux`), credenciales se zeroizan al hacer Drop

## Licencia

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribuciones

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
