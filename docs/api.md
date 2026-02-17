# API Reference

Referencia completa de la API de ARCA SDK.

## Módulos Principales

### `auth` - Autenticación WSAA

```rust
use arca::auth;

// Construir ticket de login
let login_ticket = auth::build_login_ticket_request("wsfe");

// Firmar con CMS/PKCS#7
let cms = auth::sign_cms_base64(&login_ticket, &cert_pem, &key_pem)?;

// Login con WSAA
let ticket = auth::wsaa_login(&http, wsaa_url, &cms).await?;

// El ticket contiene:
ticket.token       // String - Token de autenticación
ticket.sign        // String - Firma
ticket.expiration_time  // String - Fecha de expiración (RFC3339)
```

### `ArcaClient` - Cliente Alto Nivel

El `ArcaClient` es thread-safe (`Arc<ArcaClientInner>`) y maneja autenticación automática con caché de tokens en 3 niveles (RAM → Disco → WSAA).

```rust
use arca::{ArcaClient, ArcaEnvironment};
use arca::client::token_store::FileSystemTokenStore;

// Builder pattern
let client = ArcaClient::builder()
    .environment(ArcaEnvironment::Testing)  // o Production
    .credentials_pfx(pfx_data, "password")  // PFX/PKCS#12
    // o
    .credentials(cert_pem, key_pem)         // PEM
    .token_store(FileSystemTokenStore::new("./tokens"))  // Persistencia (opcional)
    .http_timeout(Duration::from_secs(30))               // Timeout HTTP (opcional)
    .http_connect_timeout(Duration::from_secs(10))       // Timeout conexión (opcional)
    .token_margin_minutes(5)                             // Margen expiración (opcional)
    .build()?;

// Login automático con caché de tokens
client.login("wsfe").await?;

// Métodos de alto nivel (hacen login automático si es necesario)
let ultimo = client.get_last_voucher(cuit, pto_vta, cbte_tipo).await?;
let resp = client.authorize_voucher(cuit, cab, dets).await?;
let comp = client.get_voucher(cuit, cbte_tipo, cbte_nro, pto_vta).await?;
```

#### Métodos del Builder (`ArcaClientBuilder`)

| Método | Descripción | Requerido |
|--------|-------------|-----------|
| `environment(env)` | Testing o Production | No (default: Testing) |
| `credentials(cert, key)` | Certificado PEM + clave privada | Sí* |
| `credentials_pfx(data, password)` | Certificado PFX/PKCS#12 | Sí* |
| `token_store(store)` | Habilitar persistencia de tokens | No |
| `http_timeout(duration)` | Timeout de requests HTTP | No (default: 30s) |
| `http_connect_timeout(duration)` | Timeout de conexión | No (default: 10s) |
| `token_margin_minutes(min)` | Margen antes de expiración | No (default: 5 min) |

*Se requiere `credentials()` **o** `credentials_pfx()`, no ambos.

#### Métodos de ArcaClient (Alto Nivel)

Todos estos métodos hacen `login("wsfe")` automáticamente si el token no está disponible o expiró.

| Método | Retorno | Descripción |
|--------|---------|-------------|
| `login(service)` | `Result<()>` | Login WSAA con caché 3 niveles |
| `get_last_voucher(cuit, pto_vta, cbte_tipo)` | `Result<i32>` | Último comprobante autorizado |
| `authorize_voucher(cuit, cab, dets)` | `Result<FeCaeResponse>` | Solicitar CAE |
| `get_voucher(cuit, cbte_tipo, cbte_nro, pto_vta)` | `Result<FeCompConsultarResponse>` | Consultar comprobante emitido |
| `get_tipos_cbte(cuit)` | `Result<Vec<ParamTipo>>` | Tipos de comprobante |
| `get_tipos_doc(cuit)` | `Result<Vec<ParamTipo>>` | Tipos de documento |
| `get_tipos_iva(cuit)` | `Result<Vec<ParamTipo>>` | Alícuotas IVA |
| `get_tipos_monedas(cuit)` | `Result<Vec<ParamTipo>>` | Monedas disponibles |
| `get_tipos_tributos(cuit)` | `Result<Vec<ParamTipo>>` | Tipos de tributos |
| `get_tipos_concepto(cuit)` | `Result<Vec<ParamTipo>>` | Tipos de concepto |
| `get_tipos_opcional(cuit)` | `Result<Vec<ParamTipo>>` | Campos opcionales |
| `get_condicion_iva_receptor(cuit)` | `Result<Vec<ParamTipo>>` | Condiciones IVA receptor |
| `get_ptos_venta(cuit)` | `Result<Vec<PtoVenta>>` | Puntos de venta |
| `get_cotizacion(cuit, mon_id)` | `Result<Cotizacion>` | Cotización de moneda |
| `request_caea(cuit, periodo, orden)` | `Result<CaeaResponse>` | Solicitar CAEA |
| `get_caea(cuit, periodo, orden)` | `Result<CaeaResponse>` | Consultar CAEA |
| `register_caea_invoices(cuit, cab, dets, caea)` | `Result<CaeaRegInformativoResult>` | Informar facturas CAEA |
| `report_caea_no_movement(cuit, pto_vta, caea)` | `Result<CaeaSinMovimiento>` | Declarar sin movimiento |
| `get_caea_no_movement(cuit, caea, pto_vta)` | `Result<Vec<CaeaSinMovimiento>>` | Consultar sin movimiento |

### `WsFev1Service` - Facturación Electrónica (Bajo Nivel)

Para uso directo sin el cliente de alto nivel.

```rust
use arca::{WsFev1Service, transport::HttpClient};

let http = HttpClient::new()?;
let wsfev1 = WsFev1Service::new(
    "https://wswhomo.afip.gov.ar/wsfev1/service.asmx".to_string(),
    http,
);
```

#### Métodos Core

| Método | Descripción | Requiere Auth |
|--------|-------------|---------------|
| `fe_dummy()` | Health check del servicio | No |
| `fe_comp_ultimo_autorizado()` | Último comprobante autorizado | Sí |
| `fecae_solicitar()` | Solicitar CAE | Sí |
| `fecae_solicitar_detallado()` | Solicitar CAE con detalles de errores/obs | Sí |
| `fe_comp_consultar()` | Consultar comprobante emitido | Sí |

#### Métodos de Parámetros

| Método | Descripción |
|--------|-------------|
| `fe_param_get_tipos_cbte()` | Tipos de comprobante |
| `fe_param_get_tipos_doc()` | Tipos de documento |
| `fe_param_get_tipos_iva()` | Alícuotas de IVA |
| `fe_param_get_tipos_monedas()` | Monedas disponibles |
| `fe_param_get_tipos_tributos()` | Tipos de tributos |
| `fe_param_get_tipos_concepto()` | Tipos de concepto |
| `fe_param_get_tipos_opcional()` | Campos opcionales |
| `fe_param_get_condicion_iva_receptor()` | Condiciones IVA |
| `fe_param_get_ptos_venta()` | Puntos de venta |
| `fe_param_get_cotizacion()` | Cotización de moneda |

#### Métodos CAEA

| Método | Descripción |
|--------|-------------|
| `fecaea_solicitar()` | Solicitar CAEA para período |
| `fecaea_consultar()` | Consultar CAEA existente |
| `fecaea_reg_informativo()` | Informar facturas offline |
| `fecaea_sin_movimiento_informar()` | Declarar sin movimiento |
| `fecaea_sin_movimiento_consultar()` | Consultar sin movimiento |

### `PadronA13Service` - Padrón de Contribuyentes

```rust
use arca::PadronA13Service;

// Crear con método de conveniencia
let padron = PadronA13Service::testing();    // Homologación
let padron = PadronA13Service::production(); // Producción

// O con endpoint manual
let padron = PadronA13Service::with_endpoint(
    "https://awshomo.afip.gov.ar/sr-padron/webservices/personaServiceA13"
);
```

| Método | Descripción |
|--------|-------------|
| `dummy()` | Health check |
| `get_persona()` | Consultar contribuyente por CUIT |
| `get_id_persona_list_by_documento()` | Buscar CUITs por documento |

## Sistema de Persistencia de Tokens

### El Problema

WSAA otorga tokens con validez de ~12 horas. Si solicitas un nuevo token mientras tienes uno vigente, AFIP retorna error. Si tu aplicación se reinicia sin haber guardado el token, quedas bloqueado hasta que expire.

### Solución: Caché Híbrida de 3 Niveles

```
login("wsfe")
  ├─ RAM: token válido en HashMap<RwLock>? → OK (instantáneo)
  ├─ Disco: arca_token_wsfe.json válido? → cargar a RAM → OK
  └─ WSAA: login completo → guardar en RAM + disco → OK
```

### Trait `TokenStore`

```rust
use arca::client::token_store::{TokenStore, FileSystemTokenStore};

// Implementación de filesystem incluida
let store = FileSystemTokenStore::new("./tokens");

// O implementar tu propio store (ej: Redis, base de datos)
#[async_trait]
impl TokenStore for MyCustomStore {
    async fn save_token(&self, service: &str, token: &TicketAcceso) -> Result<()> { ... }
    async fn load_token(&self, service: &str) -> Result<Option<TicketAcceso>> { ... }
}
```

### `FileSystemTokenStore`

Guarda tokens como archivos JSON en el directorio especificado:
- Archivo: `{directorio}/arca_token_{servicio}.json`
- Crea el directorio automáticamente si no existe
- I/O asíncrono con `tokio::fs`
- Errores de disco son silenciosos (fall-through al siguiente nivel)

## Estructuras de Datos

### `TicketAcceso` - Token de Autenticación

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketAcceso {
    pub token: String,            // Token de autenticación
    pub sign: String,             // Firma digital
    pub expiration_time: String,  // Expiración RFC3339
}

// Implementa Drop para zeroizar datos sensibles de memoria
// Métodos:
ta.is_expired()                  // Con margen default (5 min)
ta.is_expired_with_margin(10)    // Con margen custom
```

### `ArcaClientConfig` - Configuración

```rust
pub struct ArcaClientConfig {
    pub environment: ArcaEnvironment,       // Testing / Production
    pub http_timeout: Duration,             // Default: 30s
    pub http_connect_timeout: Duration,     // Default: 10s
    pub token_margin_minutes: i64,          // Default: 5 min
}
```

### `FeCabReq` - Cabecera de Comprobante

```rust
pub struct FeCabReq {
    pub cant_reg: i32,    // Cantidad de registros (normalmente 1)
    pub pto_vta: i32,     // Punto de venta (1-9999)
    pub cbte_tipo: i32,   // Tipo de comprobante (ver constantes)
}
```

### `FeDetReq` - Detalle de Comprobante

```rust
pub struct FeDetReq {
    pub concepto: i32,              // 1=Productos, 2=Servicios, 3=Ambos
    pub doc_tipo: i32,              // Tipo documento receptor
    pub doc_nro: u64,               // Número documento receptor
    pub cbte_desde: u64,            // Número comprobante desde
    pub cbte_hasta: u64,            // Número comprobante hasta
    pub cbte_fch: String,           // Fecha YYYYMMDD
    pub imp_total: f64,             // Importe total
    pub imp_tot_conc: f64,          // No gravado
    pub imp_neto: f64,              // Neto gravado
    pub imp_op_ex: f64,             // Exento
    pub imp_trib: f64,              // Tributos
    pub imp_iva: f64,               // IVA
    pub fch_serv_desde: Option<String>,  // Para servicios
    pub fch_serv_hasta: Option<String>,  // Para servicios
    pub fch_vto_pago: Option<String>,    // Para servicios
    pub mon_id: String,             // Moneda (PES, DOL, etc)
    pub mon_cotiz: f64,             // Cotización
    pub condicion_iva_receptor: Option<i32>,  // Obligatorio desde RG 5616
    pub iva: Vec<AlicIva>,          // Detalle IVA
}
```

### `AlicIva` - Alícuota IVA

```rust
pub struct AlicIva {
    pub id: i32,        // ID alícuota (ver constantes)
    pub base_imp: f64,  // Base imponible
    pub importe: f64,   // Importe IVA
}
```

### `FeCaeResponse` - Respuesta CAE

```rust
pub struct FeCaeResponse {
    pub cae: String,           // Código de Autorización Electrónico
    pub cae_fch_vto: String,   // Vencimiento CAE (YYYYMMDD)
    pub cbte_desde: u64,       // Número desde
    pub cbte_hasta: u64,       // Número hasta
    pub resultado: String,     // A=Aprobado, R=Rechazado, P=Parcial
}
```

### `FeCaeResponseDetallada` - Respuesta con Detalles

```rust
pub struct FeCaeResponseDetallada {
    pub cae: String,
    pub cae_fch_vto: String,
    pub cbte_desde: u64,
    pub cbte_hasta: u64,
    pub resultado: String,
    pub detalles: ArcaResponseDetails,
}

pub struct ArcaResponseDetails {
    pub errors: Vec<ArcaErr>,
    pub observaciones: Vec<ArcaObs>,
    pub eventos: Vec<ArcaEvt>,
}

// Métodos de conveniencia:
resp.is_approved()   // resultado == "A"
resp.is_rejected()   // resultado == "R"
resp.is_partial()    // resultado == "P"
```

### `FeCompConsultarResponse` - Comprobante Consultado

```rust
pub struct FeCompConsultarResponse {
    pub resultado: String,
    pub cod_autorizacion: String,  // CAE
    pub fch_vto: String,          // Vencimiento
    pub imp_total: f64,
    pub imp_neto: f64,
    pub imp_iva: f64,
    pub imp_trib: f64,
    pub imp_op_ex: f64,
    pub imp_tot_conc: f64,
    pub cbte_fch: String,
}
```

### `CaeaResponse` - Respuesta CAEA

```rust
pub struct CaeaResponse {
    pub caea: String,
    pub periodo: i32,
    pub orden: i16,
    pub fch_tope_inf: String,
    pub fch_vto_caea: String,
}
```

### Tipos de Parámetros

```rust
pub struct ParamTipo {
    pub id: i32,
    pub desc: String,
    pub fch_desde: String,
    pub fch_hasta: String,
}

pub struct PtoVenta {
    pub nro: i32,
    pub emision_tipo: String,
    pub bloqueado: String,     // "S" o "N"
    pub fch_baja: String,
}

pub struct Cotizacion {
    pub mon_id: String,
    pub mon_cotiz: f64,
    pub fch_cotiz: String,
}
```

## Constantes

### Tipos de Comprobante (`cbte_tipos`)

```rust
use arca::cbte_tipos;

FACTURA_A       // 1
NOTA_DEBITO_A   // 2
NOTA_CREDITO_A  // 3
FACTURA_B       // 6
NOTA_DEBITO_B   // 7
NOTA_CREDITO_B  // 8
FACTURA_C       // 11
NOTA_DEBITO_C   // 12
NOTA_CREDITO_C  // 13
```

### Tipos de Documento (`doc_tipos`)

```rust
use arca::doc_tipos;

CUIT              // 80
CUIL              // 86
CDI               // 87
DNI               // 96
PASAPORTE         // 94
CONSUMIDOR_FINAL  // 99
```

### Condición IVA (`condicion_iva`)

```rust
use arca::condicion_iva;

IVA_RESPONSABLE_INSCRIPTO  // 1
IVA_SUJETO_EXENTO          // 4
CONSUMIDOR_FINAL           // 5
RESPONSABLE_MONOTRIBUTO    // 6
```

### Alícuotas IVA (`alicuotas_iva`)

```rust
use arca::alicuotas_iva;

NO_GRAVADO            // 1
EXENTO                // 2
CERO_PORCIENTO        // 3
DIEZ_CINCO_PORCIENTO  // 4 (10.5%)
VEINTIUNO_PORCIENTO   // 5 (21%)
VEINTISIETE_PORCIENTO // 6 (27%)
CINCO_PORCIENTO       // 8 (5%)
DOS_CINCO_PORCIENTO   // 9 (2.5%)
```

## Endpoints

### Homologación (Testing)

```
WSAA:       https://wsaahomo.afip.gov.ar/ws/services/LoginCms
WSFEv1:     https://wswhomo.afip.gov.ar/wsfev1/service.asmx
Padrón A13: https://awshomo.afip.gov.ar/sr-padron/webservices/personaServiceA13
```

### Producción

```
WSAA:       https://wsaa.afip.gov.ar/ws/services/LoginCms
WSFEv1:     https://servicios1.afip.gov.ar/wsfev1/service.asmx
Padrón A13: https://aws.afip.gov.ar/sr-padron/webservices/personaServiceA13
```

## Manejo de Errores

```rust
use arca::ArcaError;

match result {
    Ok(response) => { /* ... */ }
    Err(ArcaError::Auth(msg)) => { /* Error de autenticación WSAA */ }
    Err(ArcaError::Transport(e)) => { /* Error de red/HTTP (reqwest::Error) */ }
    Err(ArcaError::Xml(msg)) => { /* Error parseando XML */ }
    Err(ArcaError::Signing(e)) => { /* Error OpenSSL (openssl::error::ErrorStack) */ }
    Err(ArcaError::SigningCli(msg)) => { /* Error OpenSSL CLI */ }
    Err(ArcaError::Service { code, message }) => { /* Error de servicio ARCA */ }
    Err(ArcaError::Config(msg)) => { /* Error de configuración */ }
    Err(ArcaError::NoToken) => { /* Token no disponible */ }
    Err(ArcaError::InvalidEnvironment(msg)) => { /* Ambiente inválido */ }
    Err(ArcaError::Io(msg)) => { /* Error de I/O */ }
}
```

### Variantes de `ArcaError`

| Variante | Origen | Descripción |
|----------|--------|-------------|
| `Auth(String)` | WSAA | Error de autenticación |
| `Transport(reqwest::Error)` | HTTP | Error de red/conexión |
| `Xml(String)` | Parser | Error parseando respuesta XML |
| `Signing(ErrorStack)` | OpenSSL lib | Error firmando CMS |
| `SigningCli(String)` | OpenSSL CLI | Error ejecutando openssl |
| `Service { code, message }` | ARCA | Error devuelto por el servicio |
| `Config(String)` | Builder | Error de configuración o PFX |
| `NoToken` | Client | Token expirado o no disponible |
| `InvalidEnvironment(String)` | Config | Ambiente no reconocido |
| `Io(String)` | Sistema | Error de I/O (archivos, temp) |

## Seguridad de Memoria

La biblioteca utiliza la crate `zeroize` para limpiar datos sensibles de la memoria:

- **`TicketAcceso`**: Token y sign se zeroizan al hacer Drop
- **`ArcaClientInner`**: Certificado PEM y clave privada se zeroizan al hacer Drop
- **`ArcaClientBuilder`**: PFX data y password se zeroizan al hacer Drop
- **`ArcaHandle` (FFI)**: Credenciales y tokens se zeroizan al hacer Drop

Ver [errors.md](errors.md) para códigos de error específicos de ARCA.
Ver [DOCUMENTACION_PERSISTENCIA.md](../DOCUMENTACION_PERSISTENCIA.md) para guía detallada de persistencia.
