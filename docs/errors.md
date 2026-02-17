# Codigos de Error ARCA

Referencia de codigos de error retornados por los servicios web de ARCA/AFIP.

## Uso

```rust
use arca::errores;

// Obtener descripcion de un codigo
let desc = errores::descripcion(10016);
// "Numero/fecha no corresponde con proximo a autorizar"

// Obtener categoria
let cat = errores::categoria(10016);
// "Auth/Val"

// Verificar si es error CAEA
let es_caea = errores::es_caea(15006);
// true
```

## Errores Comunes

### Validacion de Comprobantes

| Codigo | Descripcion | Solucion |
|--------|-------------|----------|
| 10016 | Numero/fecha no corresponde con proximo a autorizar | Usar `fe_comp_ultimo_autorizado()` para obtener el numero correcto |
| 10048 | ImpTotal no coincide con suma de componentes | Verificar: imp_total = imp_neto + imp_iva + imp_trib + imp_op_ex + imp_tot_conc |
| 10071 | Factura C no debe informar IVA | Dejar `iva: vec![]` vacio para Factura C |
| 10017 | Nro de comprobante registrado | El comprobante ya fue autorizado |
| 10018 | Fecha de comprobante fuera de rango | La fecha debe estar dentro del rango permitido |

### Autenticacion

| Codigo | Descripcion | Solucion |
|--------|-------------|----------|
| 600 | ValidacionDeToken: CUIT no autorizado | Verificar que el certificado esta asociado al CUIT |
| 601 | Token expirado | Renovar el token con `login()` |
| 602 | Token invalido | El token/sign no corresponden al servicio |

### IVA y Receptor

| Codigo | Descripcion | Solucion |
|--------|-------------|----------|
| 10047 | Alicuota de IVA no informada | Agregar detalle en el array `iva` |
| 10063 | Receptor no inscripto para Factura A | El CUIT receptor debe ser Resp. Inscripto |
| 10195 | condicion_iva_receptor obligatorio | Agregar campo desde RG 5616 |
| 39 | condicion_iva_receptor sera obligatorio | Warning - el campo es requerido desde 02/2026 |

### Servicios

| Codigo | Descripcion | Solucion |
|--------|-------------|----------|
| 10019 | Concepto no corresponde a servicio | Para servicios usar concepto 2 o 3 |
| 10020 | Falta fecha servicio desde | Agregar `fch_serv_desde` para servicios |
| 10021 | Falta fecha servicio hasta | Agregar `fch_serv_hasta` para servicios |
| 10022 | Falta fecha vencimiento pago | Agregar `fch_vto_pago` para servicios |

### Moneda

| Codigo | Descripcion | Solucion |
|--------|-------------|----------|
| 10051 | Moneda no valida | Usar codigos validos: PES, DOL, etc. |
| 10052 | Cotizacion incorrecta | Para PES usar 1.0, para otras consultar cotizacion |

### CAEA

| Codigo | Descripcion | Solucion |
|--------|-------------|----------|
| 15001 | CAEA no valido | Verificar el codigo CAEA |
| 15002 | CAEA vencido | El CAEA ya no esta vigente |
| 15003 | Periodo CAEA no corresponde | La quincena/orden no coincide |
| 15006 | CAEA ya informado | El periodo ya fue declarado |

## Lista Completa por Categoria

### Validacion General (10000-10099)

| Codigo | Mensaje |
|--------|---------|
| 10000 | El campo es requerido |
| 10001 | El valor esta fuera de rango |
| 10002 | El formato es invalido |
| 10003 | El campo no puede estar vacio |
| 10004 | El campo excede longitud maxima |
| 10005 | El campo contiene caracteres invalidos |
| 10006 | El tipo de dato es incorrecto |
| 10007 | Valor duplicado |
| 10008 | Referencia invalida |
| 10009 | Combinacion de campos invalida |
| 10010 | Operacion no permitida |

### Comprobantes (10011-10050)

| Codigo | Mensaje |
|--------|---------|
| 10011 | Tipo de comprobante no valido |
| 10012 | Punto de venta no autorizado |
| 10013 | Punto de venta bloqueado |
| 10014 | Rango de numeracion agotado |
| 10015 | Error en secuencia de numeracion |
| 10016 | Numero/fecha no corresponde con proximo a autorizar |
| 10017 | Nro de comprobante registrado |
| 10018 | Fecha de comprobante fuera de rango |
| 10019 | Concepto no corresponde |
| 10020 | Falta FchServDesde |
| 10021 | Falta FchServHasta |
| 10022 | Falta FchVtoPago |

### Importes (10051-10100)

| Codigo | Mensaje |
|--------|---------|
| 10048 | ImpTotal no coincide con suma |
| 10049 | ImpNeto incorrecto |
| 10050 | ImpIVA incorrecto |
| 10051 | Moneda no valida |
| 10052 | Cotizacion incorrecta |

### IVA (10101-10150)

| Codigo | Mensaje |
|--------|---------|
| 10047 | Alicuota de IVA no informada |
| 10070 | Alicuota no corresponde al tipo de comprobante |
| 10071 | Factura C no debe informar IVA |
| 10072 | Base imponible IVA incorrecta |

### Receptor (10151-10200)

| Codigo | Mensaje |
|--------|---------|
| 10063 | Receptor no inscripto para Factura A |
| 10064 | CUIT receptor invalido |
| 10065 | Tipo documento receptor no valido |
| 10066 | Numero documento receptor invalido |
| 10195 | condicion_iva_receptor obligatorio |

### CAEA (15001-15100)

| Codigo | Mensaje |
|--------|---------|
| 15001 | CAEA no valido |
| 15002 | CAEA vencido |
| 15003 | Periodo CAEA no corresponde |
| 15004 | CAEA no autorizado para punto de venta |
| 15005 | CAEA ya utilizado |
| 15006 | CAEA ya informado |

### Autenticacion (600-699)

| Codigo | Mensaje |
|--------|---------|
| 600 | ValidacionDeToken: CUIT no autorizado |
| 601 | Token expirado |
| 602 | Token invalido |
| 603 | Firma invalida |
| 604 | Servicio no autorizado |

## Tipos de Error Rust (`ArcaError`)

| Variante | Origen | Descripcion |
|----------|--------|-------------|
| `Auth(String)` | WSAA | Error de autenticacion |
| `Transport(reqwest::Error)` | HTTP | Error de red/conexion |
| `Xml(String)` | Parser | Error parseando respuesta XML |
| `Signing(ErrorStack)` | OpenSSL lib | Error firmando CMS (binding) |
| `SigningCli(String)` | OpenSSL CLI | Error ejecutando openssl (proceso) |
| `Service { code, message }` | ARCA | Error devuelto por el servicio web |
| `Config(String)` | Builder/PFX | Error de configuracion o conversion PFX |
| `NoToken` | Client | Token expirado o no disponible |
| `InvalidEnvironment(String)` | Config | Ambiente no reconocido |
| `Io(String)` | Sistema | Error de I/O (archivos temporales) |

## Codigos FFI

Los codigos retornados por las funciones FFI de la DLL:

### General (0 a -9)

| Codigo | Constante | Significado |
|--------|-----------|-------------|
| 0 | `FFI_OK` | Exito |
| -1 | `FFI_ERR_NULL_POINTER` | Puntero nulo |
| -2 | `FFI_ERR_INVALID_UTF8` | String UTF-8 invalido |
| -3 | `FFI_ERR_FILE_READ` | Error leyendo archivo |
| -4 | `FFI_ERR_RUNTIME` | Error creando runtime Tokio |
| -5 | `FFI_ERR_HTTP_CLIENT` | Error creando cliente HTTP |
| -6 | `FFI_ERR_PFX_CONVERSION` | Error convirtiendo PFX |

### Autenticacion (-10 a -19)

| Codigo | Constante | Significado |
|--------|-----------|-------------|
| -10 | `FFI_ERR_CMS_SIGNING` | Error firmando CMS |
| -11 | `FFI_ERR_LOGIN_FAILED` | Login fallo |
| -12 | `FFI_ERR_NOT_AUTHENTICATED` | No autenticado |

### WSFEv1 (-20 a -39)

| Codigo | Constante | Significado |
|--------|-----------|-------------|
| -20 | `FFI_ERR_FE_DUMMY` | Error en FE Dummy |
| -21 | `FFI_ERR_FE_COMP_ULTIMO` | Error obteniendo ultimo comprobante |
| -22 | `FFI_ERR_FECAE_SOLICITAR` | Error solicitando CAE |
| -23 | `FFI_ERR_FE_COMP_CONSULTAR` | Error consultando comprobante |
| -24 | `FFI_ERR_FE_TIPOS_CBTE` | Error obteniendo tipos cbte |
| -25 | `FFI_ERR_FE_TIPOS_DOC` | Error obteniendo tipos doc |
| -26 | `FFI_ERR_FE_TIPOS_IVA` | Error obteniendo tipos IVA |
| -27 | `FFI_ERR_FE_TIPOS_MONEDAS` | Error obteniendo monedas |
| -28 | `FFI_ERR_FE_PTOS_VENTA` | Error obteniendo puntos de venta |
| -29 | `FFI_ERR_FE_COTIZACION` | Error obteniendo cotizacion |
| -30 | `FFI_ERR_INVALID_JSON` | JSON invalido |
| -31 | `FFI_ERR_VALIDATION_FAILED` | Validacion fallo |

### Padron (-40 a -49)

| Codigo | Constante | Significado |
|--------|-----------|-------------|
| -40 | `FFI_ERR_PADRON_DUMMY` | Error en Padron Dummy |
| -41 | `FFI_ERR_PADRON_GET_PERSONA` | Error consultando persona |

### Interno

| Codigo | Constante | Significado |
|--------|-----------|-------------|
| -99 | `FFI_ERR_CSTRING` | Error interno de conversion CString |

## Observaciones vs Errores

ARCA distingue entre:

- **Errores**: Rechazan el comprobante
- **Observaciones**: Warnings que no impiden la autorizacion

```rust
// Codigo 39 es observacion, no error
// "condicion_iva_receptor sera obligatorio a partir de 01/02/2026"
if resp.detalles.observaciones.iter().any(|o| o.code == 39) {
    println!("Warning: Agregar condicion_iva_receptor pronto");
}
```
