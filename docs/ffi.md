# Guia FFI/DLL

La biblioteca se compila como DLL para uso desde otros lenguajes.

## Compilacion

```bash
# Linux - genera target/release/libarca.so
cargo build --release

# Windows (cross-compile)
cargo build --release --target x86_64-pc-windows-gnu
```

### Archivos Generados

```
target/release/
├── libarca.so       # Linux (~7.5 MB con LTO/strip)
└── arca.dll         # Windows

include/
└── arca.h           # Header C

examples/ffi/
├── ArcaClient.cs    # Wrapper C#/.NET
├── arca_client.py   # Wrapper Python
└── ArcaClient.pas   # Wrapper Delphi/Pascal
```

## Arquitectura FFI

La capa FFI se compone de 11 modulos especializados bajo `src/ffi/`:

```
src/ffi/
├── mod.rs           # Exports, URLs estaticas, runtime global Tokio
├── error_codes.rs   # Constantes de codigos de error FFI
├── helpers.rs       # Manejo de errores thread-local, conversion de strings
├── handle.rs        # ArcaHandle (struct opaco para consumidores FFI)
├── client.rs        # Creacion/destruccion de clientes (PEM y PFX)
├── auth.rs          # Login WSAA + cache de tokens persistente
├── wsfev1.rs        # Funciones de facturacion electronica
├── params.rs        # Consultas de parametros
├── padron.rs        # Padron de contribuyentes A13
├── invoice.rs       # Validacion de facturas
└── constants.rs     # Constantes ARCA exportadas (CBTE, DOC, IVA, ALIC)
```

### Caracteristicas clave

- **Runtime global Tokio**: Un unico runtime compartido entre todos los clientes (2 worker threads), usando `OnceLock` para inicializacion lazy
- **Errores thread-local**: Cada thread mantiene su propio estado de `last_error`
- **Servicios cacheados**: `WsFev1Service` y `PadronA13Service` se crean una sola vez por cliente, evitando allocations de URLs en cada llamada
- **Zeroizacion de memoria**: `ArcaHandle` implementa `Drop` para limpiar credenciales y tokens sensibles

## Funciones FFI

### Gestion de Cliente

| Funcion | Descripcion |
|---------|-------------|
| `arca_client_new_pfx_production` | Crear cliente produccion con PFX |
| `arca_client_new_pfx_testing` | Crear cliente testing con PFX |
| `arca_client_new_production` | Crear cliente produccion con PEM |
| `arca_client_new_testing` | Crear cliente testing con PEM |
| `arca_client_free` | Liberar cliente (zeroiza credenciales) |

### Autenticacion

| Funcion | Descripcion |
|---------|-------------|
| `arca_set_token_cache` | Configurar directorio de cache para persistir tokens |
| `arca_login` | Autenticar con WSAA (usa cache RAM -> disco -> WSAA) |
| `arca_is_authenticated` | Verificar autenticacion (retorna 1 o 0) |

### Facturacion (WSFEv1)

| Funcion | Descripcion |
|---------|-------------|
| `arca_fe_dummy` | Health check |
| `arca_fe_comp_ultimo_autorizado` | Ultimo comprobante |
| `arca_fecae_solicitar` | Solicitar CAE |
| `arca_fe_comp_consultar` | Consultar comprobante |

### Parametros

| Funcion | Descripcion |
|---------|-------------|
| `arca_fe_param_get_tipos_cbte` | Tipos de comprobante |
| `arca_fe_param_get_tipos_doc` | Tipos de documento |
| `arca_fe_param_get_tipos_iva` | Alicuotas IVA |
| `arca_fe_param_get_tipos_monedas` | Monedas |
| `arca_fe_param_get_ptos_venta` | Puntos de venta |
| `arca_fe_param_get_cotizacion` | Cotizacion moneda |

### Padron

| Funcion | Descripcion |
|---------|-------------|
| `arca_padron_dummy` | Health check |
| `arca_padron_get_persona` | Consultar CUIT |

### Utilidades

| Funcion | Descripcion |
|---------|-------------|
| `arca_last_error` | Obtener ultimo error (thread-local) |
| `arca_free_string` | Liberar string retornado por la libreria |

## Codigos de Retorno FFI

### General (0 a -9)

| Codigo | Constante | Significado |
|--------|-----------|-------------|
| 0 | `FFI_OK` | Exito |
| -1 | `FFI_ERR_NULL_POINTER` | Puntero nulo |
| -2 | `FFI_ERR_INVALID_UTF8` | String UTF-8 invalido |
| -3 | `FFI_ERR_FILE_READ` | Error leyendo archivo |
| -4 | `FFI_ERR_RUNTIME` | Error creando runtime Tokio |
| -5 | `FFI_ERR_HTTP_CLIENT` | Error creando cliente HTTP |
| -6 | `FFI_ERR_PFX_CONVERSION` | Error convirtiendo PFX a PEM |

### Autenticacion (-10 a -19)

| Codigo | Constante | Significado |
|--------|-----------|-------------|
| -10 | `FFI_ERR_CMS_SIGNING` | Error firmando CMS |
| -11 | `FFI_ERR_LOGIN_FAILED` | Login WSAA fallo |
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
| -31 | `FFI_ERR_VALIDATION_FAILED` | Validacion de campos fallo |

### Padron (-40 a -49)

| Codigo | Constante | Significado |
|--------|-----------|-------------|
| -40 | `FFI_ERR_PADRON_DUMMY` | Error en Padron Dummy |
| -41 | `FFI_ERR_PADRON_GET_PERSONA` | Error consultando persona |

### Interno

| Codigo | Constante | Significado |
|--------|-----------|-------------|
| -99 | `FFI_ERR_CSTRING` | Error interno de conversion CString |

## Constantes Exportadas

La DLL exporta constantes para tipos de comprobante, documento, condicion IVA y alicuotas:

```c
// Tipos de comprobante
ARCA_CBTE_FACTURA_A      // 1
ARCA_CBTE_FACTURA_B      // 6
ARCA_CBTE_FACTURA_C      // 11
// ... etc

// Tipos de documento
ARCA_DOC_CUIT            // 80
ARCA_DOC_CONSUMIDOR_FINAL // 99
// ... etc

// Condicion IVA
ARCA_IVA_RESPONSABLE_INSCRIPTO  // 1
ARCA_IVA_CONSUMIDOR_FINAL       // 5
// ... etc

// Alicuotas IVA
ARCA_ALIC_VEINTIUNO     // 5 (21%)
ARCA_ALIC_DIEZ_CINCO    // 4 (10.5%)
// ... etc
```

## Persistencia de Tokens

WSAA otorga tokens con validez de ~12 horas y rechaza nuevas solicitudes mientras el token anterior este vigente. Si el proceso se reinicia sin haber guardado el token, se produce un **lockout irrecuperable** hasta que expire.

Para evitar esto, llamar a `arca_set_token_cache` **antes** de `arca_login`:

```c
// Habilitar cache de tokens
arca_set_token_cache(handle, "./cache");

// Login con cache de 3 niveles: RAM -> Disco -> WSAA
arca_login(handle, "wsfe");
```

El flujo de `arca_login` con cache habilitado:

1. **RAM** - Si hay token valido en memoria -> retorna OK (instantaneo)
2. **Disco** - Si existe `{cache_dir}/arca_token_{service}.json` y no expiro -> carga a RAM -> OK
3. **WSAA** - Login completo -> guarda en RAM + disco -> OK

Los errores de disco son silenciosos (fall-through al siguiente nivel). Sin llamar a `arca_set_token_cache`, el comportamiento es identico a la version anterior (solo RAM).

## Manejo de Memoria

- **Strings retornados**: Liberar con `arca_free_string()`
- **Cliente**: Liberar con `arca_client_free()` (zeroiza credenciales y tokens)
- **Errores**: Usar `arca_last_error()` para obtener mensaje (thread-local)

---

## Ejemplo C# Completo

```csharp
using System;
using System.Runtime.InteropServices;
using System.Text.Json;

public class ArcaClient : IDisposable
{
    private IntPtr _handle;

    [DllImport("libarca.so")]
    private static extern int arca_client_new_pfx_production(
        string pfxPath, string password, out IntPtr handle);

    [DllImport("libarca.so")]
    private static extern int arca_client_new_pfx_testing(
        string pfxPath, string password, out IntPtr handle);

    [DllImport("libarca.so")]
    private static extern void arca_client_free(IntPtr handle);

    [DllImport("libarca.so")]
    private static extern int arca_set_token_cache(IntPtr handle, string cacheDir);

    [DllImport("libarca.so")]
    private static extern int arca_login(IntPtr handle, string service);

    [DllImport("libarca.so")]
    private static extern int arca_is_authenticated(IntPtr handle);

    [DllImport("libarca.so")]
    private static extern int arca_fe_comp_ultimo_autorizado(
        IntPtr handle, ulong cuit, int ptoVta, int cbteTipo, out long numero);

    [DllImport("libarca.so")]
    private static extern int arca_fecae_solicitar(
        IntPtr handle, ulong cuit, string invoiceJson, out IntPtr result);

    [DllImport("libarca.so")]
    private static extern IntPtr arca_last_error();

    [DllImport("libarca.so")]
    private static extern void arca_free_string(IntPtr str);

    // Factory methods
    public static ArcaClient ProductionPfx(string pfxPath, string password)
    {
        var client = new ArcaClient();
        int result = arca_client_new_pfx_production(pfxPath, password, out client._handle);
        if (result != 0)
            throw new Exception(GetLastError());
        return client;
    }

    public static ArcaClient TestingPfx(string pfxPath, string password)
    {
        var client = new ArcaClient();
        int result = arca_client_new_pfx_testing(pfxPath, password, out client._handle);
        if (result != 0)
            throw new Exception(GetLastError());
        return client;
    }

    public void SetTokenCache(string cacheDir)
    {
        int result = arca_set_token_cache(_handle, cacheDir);
        if (result != 0)
            throw new Exception(GetLastError());
    }

    public void Login(string service)
    {
        int result = arca_login(_handle, service);
        if (result != 0)
            throw new Exception(GetLastError());
    }

    public bool IsAuthenticated => arca_is_authenticated(_handle) == 1;

    public long FeCompUltimoAutorizado(ulong cuit, int ptoVta, int cbteTipo)
    {
        int result = arca_fe_comp_ultimo_autorizado(_handle, cuit, ptoVta, cbteTipo, out long numero);
        if (result != 0)
            throw new Exception(GetLastError());
        return numero;
    }

    public JsonDocument FeCaeSolicitar(ulong cuit, object invoice)
    {
        string json = JsonSerializer.Serialize(invoice);
        int result = arca_fecae_solicitar(_handle, cuit, json, out IntPtr resultPtr);
        if (result != 0)
            throw new Exception(GetLastError());

        string resultJson = Marshal.PtrToStringAnsi(resultPtr);
        arca_free_string(resultPtr);
        return JsonDocument.Parse(resultJson);
    }

    private static string GetLastError()
    {
        IntPtr ptr = arca_last_error();
        if (ptr == IntPtr.Zero) return "Unknown error";
        string msg = Marshal.PtrToStringAnsi(ptr);
        arca_free_string(ptr);
        return msg;
    }

    public void Dispose()
    {
        if (_handle != IntPtr.Zero)
        {
            arca_client_free(_handle);
            _handle = IntPtr.Zero;
        }
    }
}

// Uso
class Program
{
    const ulong CUIT = 20123456789;
    const int PTO_VTA = 1;

    static void Main()
    {
        using var client = ArcaClient.ProductionPfx("certificado.pfx", "");
        client.SetTokenCache("./cache");  // Habilitar persistencia
        client.Login("wsfe");

        long ultimo = client.FeCompUltimoAutorizado(CUIT, PTO_VTA, 11);
        Console.WriteLine($"Ultimo comprobante: {ultimo}");

        var invoice = new {
            pto_vta = PTO_VTA,
            cbte_tipo = 11,
            concepto = 1,
            doc_tipo = 99,
            doc_nro = 0,
            cbte_desde = ultimo + 1,
            cbte_hasta = ultimo + 1,
            cbte_fch = DateTime.Now.ToString("yyyyMMdd"),
            imp_total = 1000.0,
            imp_tot_conc = 0.0,
            imp_neto = 1000.0,
            imp_op_ex = 0.0,
            imp_trib = 0.0,
            imp_iva = 0.0,
            mon_id = "PES",
            mon_cotiz = 1.0,
            condicion_iva_receptor = 5,
            iva = new object[] { }
        };

        var result = client.FeCaeSolicitar(CUIT, invoice);
        Console.WriteLine($"CAE: {result.RootElement.GetProperty("cae")}");
    }
}
```

---

## Ejemplo Python Completo

```python
import ctypes
import json
from datetime import datetime
from contextlib import contextmanager

class ArcaClient:
    def __init__(self, handle, lib):
        self._handle = handle
        self._lib = lib

    @classmethod
    def _load_lib(cls):
        lib = ctypes.CDLL("./libarca.so")

        # Configurar tipos de retorno
        lib.arca_last_error.restype = ctypes.c_char_p
        lib.arca_is_authenticated.restype = ctypes.c_int

        return lib

    @classmethod
    def production_pfx(cls, pfx_path, password):
        lib = cls._load_lib()
        handle = ctypes.c_void_p()
        result = lib.arca_client_new_pfx_production(
            pfx_path.encode(), password.encode(), ctypes.byref(handle))
        if result != 0:
            raise Exception(cls._get_error(lib))
        return cls(handle, lib)

    @classmethod
    def testing_pfx(cls, pfx_path, password):
        lib = cls._load_lib()
        handle = ctypes.c_void_p()
        result = lib.arca_client_new_pfx_testing(
            pfx_path.encode(), password.encode(), ctypes.byref(handle))
        if result != 0:
            raise Exception(cls._get_error(lib))
        return cls(handle, lib)

    @staticmethod
    def _get_error(lib):
        err = lib.arca_last_error()
        return err.decode() if err else "Unknown error"

    def set_token_cache(self, cache_dir):
        result = self._lib.arca_set_token_cache(self._handle, cache_dir.encode())
        if result != 0:
            raise Exception(self._get_error(self._lib))

    def login(self, service):
        result = self._lib.arca_login(self._handle, service.encode())
        if result != 0:
            raise Exception(self._get_error(self._lib))

    @property
    def is_authenticated(self):
        return self._lib.arca_is_authenticated(self._handle) == 1

    def fe_comp_ultimo_autorizado(self, cuit, pto_vta, cbte_tipo):
        numero = ctypes.c_int64()
        result = self._lib.arca_fe_comp_ultimo_autorizado(
            self._handle, cuit, pto_vta, cbte_tipo, ctypes.byref(numero))
        if result != 0:
            raise Exception(self._get_error(self._lib))
        return numero.value

    def fecae_solicitar(self, cuit, invoice):
        invoice_json = json.dumps(invoice).encode()
        result_ptr = ctypes.c_char_p()
        result = self._lib.arca_fecae_solicitar(
            self._handle, cuit, invoice_json, ctypes.byref(result_ptr))
        if result != 0:
            raise Exception(self._get_error(self._lib))

        result_json = result_ptr.value.decode()
        self._lib.arca_free_string(result_ptr)
        return json.loads(result_json)

    def close(self):
        if self._handle:
            self._lib.arca_client_free(self._handle)
            self._handle = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()


# Constantes
class CbteTipos:
    FACTURA_A = 1
    FACTURA_B = 6
    FACTURA_C = 11
    NOTA_CREDITO_C = 13

class DocTipos:
    CUIT = 80
    DNI = 96
    CONSUMIDOR_FINAL = 99

class CondicionIva:
    RESPONSABLE_INSCRIPTO = 1
    CONSUMIDOR_FINAL = 5
    MONOTRIBUTO = 6


# Uso
if __name__ == "__main__":
    CUIT = 20123456789
    PTO_VTA = 1

    with ArcaClient.production_pfx("certificado.pfx", "") as client:
        client.set_token_cache("./cache")  # Habilitar persistencia
        client.login("wsfe")

        ultimo = client.fe_comp_ultimo_autorizado(CUIT, PTO_VTA, CbteTipos.FACTURA_C)
        print(f"Ultimo comprobante: {ultimo}")

        invoice = {
            "pto_vta": PTO_VTA,
            "cbte_tipo": CbteTipos.FACTURA_C,
            "concepto": 1,
            "doc_tipo": DocTipos.CONSUMIDOR_FINAL,
            "doc_nro": 0,
            "cbte_desde": ultimo + 1,
            "cbte_hasta": ultimo + 1,
            "cbte_fch": datetime.now().strftime("%Y%m%d"),
            "imp_total": 1000.0,
            "imp_tot_conc": 0.0,
            "imp_neto": 1000.0,
            "imp_op_ex": 0.0,
            "imp_trib": 0.0,
            "imp_iva": 0.0,
            "mon_id": "PES",
            "mon_cotiz": 1.0,
            "condicion_iva_receptor": CondicionIva.CONSUMIDOR_FINAL,
            "iva": []
        }

        result = client.fecae_solicitar(CUIT, invoice)
        print(f"CAE: {result['cae']}")
        print(f"Vencimiento: {result['cae_fch_vto']}")
```

---

## Ejemplo Delphi/Pascal Completo

```pascal
unit ArcaClient;

interface

uses
  SysUtils, Classes;

const
  CBTE_FACTURA_A = 1;
  CBTE_FACTURA_B = 6;
  CBTE_FACTURA_C = 11;

  DOC_CUIT = 80;
  DOC_DNI = 96;
  DOC_CONSUMIDOR_FINAL = 99;

  COND_IVA_RESPONSABLE_INSCRIPTO = 1;
  COND_IVA_CONSUMIDOR_FINAL = 5;
  COND_IVA_MONOTRIBUTO = 6;

type
  TArcaClient = class
  private
    FHandle: Pointer;
    function GetLastError: string;
    function GetIsAuthenticated: Boolean;
  public
    constructor CreateProductionPfx(const PfxPath, Password: string);
    constructor CreateTestingPfx(const PfxPath, Password: string);
    destructor Destroy; override;

    procedure SetTokenCache(const CacheDir: string);
    procedure Login(const Service: string);
    function FeCompUltimoAutorizado(Cuit: UInt64; PtoVta, CbteTipo: Integer): Int64;
    function FeCaeSolicitar(Cuit: UInt64; const InvoiceJson: string): string;

    property IsAuthenticated: Boolean read GetIsAuthenticated;
  end;

implementation

// FFI imports
function arca_client_new_pfx_production(PfxPath, Password: PAnsiChar;
  out Handle: Pointer): Integer; cdecl; external 'libarca.so';
function arca_client_new_pfx_testing(PfxPath, Password: PAnsiChar;
  out Handle: Pointer): Integer; cdecl; external 'libarca.so';
procedure arca_client_free(Handle: Pointer); cdecl; external 'libarca.so';
function arca_set_token_cache(Handle: Pointer;
  CacheDir: PAnsiChar): Integer; cdecl; external 'libarca.so';
function arca_login(Handle: Pointer; Service: PAnsiChar): Integer; cdecl; external 'libarca.so';
function arca_is_authenticated(Handle: Pointer): Integer; cdecl; external 'libarca.so';
function arca_fe_comp_ultimo_autorizado(Handle: Pointer; Cuit: UInt64;
  PtoVta, CbteTipo: Integer; out Numero: Int64): Integer; cdecl; external 'libarca.so';
function arca_fecae_solicitar(Handle: Pointer; Cuit: UInt64;
  InvoiceJson: PAnsiChar; out ResultJson: PAnsiChar): Integer; cdecl; external 'libarca.so';
function arca_last_error: PAnsiChar; cdecl; external 'libarca.so';
procedure arca_free_string(S: PAnsiChar); cdecl; external 'libarca.so';

{ TArcaClient }

constructor TArcaClient.CreateProductionPfx(const PfxPath, Password: string);
var
  Result: Integer;
begin
  inherited Create;
  Result := arca_client_new_pfx_production(
    PAnsiChar(AnsiString(PfxPath)),
    PAnsiChar(AnsiString(Password)),
    FHandle);
  if Result <> 0 then
    raise Exception.Create(GetLastError);
end;

constructor TArcaClient.CreateTestingPfx(const PfxPath, Password: string);
var
  Result: Integer;
begin
  inherited Create;
  Result := arca_client_new_pfx_testing(
    PAnsiChar(AnsiString(PfxPath)),
    PAnsiChar(AnsiString(Password)),
    FHandle);
  if Result <> 0 then
    raise Exception.Create(GetLastError);
end;

destructor TArcaClient.Destroy;
begin
  if Assigned(FHandle) then
    arca_client_free(FHandle);
  inherited;
end;

procedure TArcaClient.SetTokenCache(const CacheDir: string);
var
  Res: Integer;
begin
  Res := arca_set_token_cache(FHandle, PAnsiChar(AnsiString(CacheDir)));
  if Res <> 0 then
    raise Exception.Create(GetLastError);
end;

function TArcaClient.GetLastError: string;
var
  Err: PAnsiChar;
begin
  Err := arca_last_error;
  if Err = nil then
    Result := 'Unknown error'
  else
  begin
    Result := string(AnsiString(Err));
    arca_free_string(Err);
  end;
end;

function TArcaClient.GetIsAuthenticated: Boolean;
begin
  Result := arca_is_authenticated(FHandle) = 1;
end;

procedure TArcaClient.Login(const Service: string);
var
  Res: Integer;
begin
  Res := arca_login(FHandle, PAnsiChar(AnsiString(Service)));
  if Res <> 0 then
    raise Exception.Create(GetLastError);
end;

function TArcaClient.FeCompUltimoAutorizado(Cuit: UInt64; PtoVta, CbteTipo: Integer): Int64;
var
  Res: Integer;
begin
  Res := arca_fe_comp_ultimo_autorizado(FHandle, Cuit, PtoVta, CbteTipo, Result);
  if Res <> 0 then
    raise Exception.Create(GetLastError);
end;

function TArcaClient.FeCaeSolicitar(Cuit: UInt64; const InvoiceJson: string): string;
var
  Res: Integer;
  ResultPtr: PAnsiChar;
begin
  Res := arca_fecae_solicitar(FHandle, Cuit, PAnsiChar(AnsiString(InvoiceJson)), ResultPtr);
  if Res <> 0 then
    raise Exception.Create(GetLastError);

  Result := string(AnsiString(ResultPtr));
  arca_free_string(ResultPtr);
end;

end.

// Uso
program TestArca;
uses
  SysUtils, ArcaClient;
var
  Client: TArcaClient;
  Ultimo: Int64;
  InvoiceJson, ResultJson: string;
begin
  Client := TArcaClient.CreateProductionPfx('certificado.pfx', '');
  try
    Client.SetTokenCache('./cache');  // Habilitar persistencia
    Client.Login('wsfe');

    Ultimo := Client.FeCompUltimoAutorizado(20123456789, 1, CBTE_FACTURA_C);
    WriteLn('Ultimo: ', Ultimo);

    InvoiceJson := Format(
      '{"pto_vta":1,"cbte_tipo":11,"concepto":1,' +
      '"doc_tipo":99,"doc_nro":0,"cbte_desde":%d,"cbte_hasta":%d,' +
      '"cbte_fch":"%s","imp_total":1000,"imp_tot_conc":0,"imp_neto":1000,' +
      '"imp_op_ex":0,"imp_trib":0,"imp_iva":0,"mon_id":"PES","mon_cotiz":1,' +
      '"condicion_iva_receptor":5,"iva":[]}',
      [Ultimo + 1, Ultimo + 1, FormatDateTime('yyyymmdd', Now)]);

    ResultJson := Client.FeCaeSolicitar(20123456789, InvoiceJson);
    WriteLn('Resultado: ', ResultJson);
  finally
    Client.Free;
  end;
end.
```
