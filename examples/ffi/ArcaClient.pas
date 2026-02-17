{*
 * ARCA SDK - Delphi/Pascal Wrapper
 *
 * Ejemplo de uso desde Delphi/Free Pascal.
 *
 * Requisitos:
 *   - Delphi 7+ o Free Pascal 3.0+
 *   - libarca.so (Linux) o arca.dll (Windows) en el directorio de la aplicacion
 *
 * Uso:
 *   uses ArcaClient;
 *
 *   var
 *     Client: TArcaClient;
 *   begin
 *     Client := TArcaClient.CreateProductionPfx('certificado.pfx', '');
 *     try
 *       Client.Login('wsfe');
 *       WriteLn('Ultimo: ', Client.FeCompUltimoAutorizado(CUIT, PTO_VTA, CBTE_FACTURA_C));
 *     finally
 *       Client.Free;
 *     end;
 *   end;
 *}

unit ArcaClient;

{$IFDEF FPC}
  {$MODE DELPHI}
{$ENDIF}

interface

uses
  SysUtils, Classes;

const
  {$IFDEF WINDOWS}
  ARCA_LIB = 'arca.dll';
  {$ELSE}
  ARCA_LIB = 'libarca.so';
  {$ENDIF}

  // Tipos de comprobante
  CBTE_FACTURA_A = 1;
  CBTE_NOTA_DEBITO_A = 2;
  CBTE_NOTA_CREDITO_A = 3;
  CBTE_FACTURA_B = 6;
  CBTE_NOTA_DEBITO_B = 7;
  CBTE_NOTA_CREDITO_B = 8;
  CBTE_FACTURA_C = 11;
  CBTE_NOTA_DEBITO_C = 12;
  CBTE_NOTA_CREDITO_C = 13;

  // Tipos de documento
  DOC_CUIT = 80;
  DOC_CUIL = 86;
  DOC_CDI = 87;
  DOC_DNI = 96;
  DOC_PASAPORTE = 94;
  DOC_CONSUMIDOR_FINAL = 99;

  // Condicion IVA
  IVA_RESPONSABLE_INSCRIPTO = 1;
  IVA_SUJETO_EXENTO = 4;
  IVA_CONSUMIDOR_FINAL = 5;
  IVA_RESPONSABLE_MONOTRIBUTO = 6;

  // Alicuotas IVA
  ALIC_NO_GRAVADO = 1;
  ALIC_EXENTO = 2;
  ALIC_CERO = 3;
  ALIC_DIEZ_CINCO = 4;  // 10.5%
  ALIC_VEINTIUNO = 5;   // 21%
  ALIC_VEINTISIETE = 6; // 27%
  ALIC_CINCO = 8;       // 5%
  ALIC_DOS_CINCO = 9;   // 2.5%

type
  TArcaHandle = Pointer;

  EArcaError = class(Exception)
  private
    FCode: Integer;
  public
    constructor Create(const AMessage: string; ACode: Integer);
    property Code: Integer read FCode;
  end;

  TArcaClient = class
  private
    FHandle: TArcaHandle;
    procedure CheckResult(AResult: Integer; const AOperation: string);
    function GetAndFreeString(APtr: PAnsiChar): string;
  public
    constructor CreateTesting(const ACertPath, AKeyPath: string);
    constructor CreateProduction(const ACertPath, AKeyPath: string);
    constructor CreateTestingPfx(const APfxPath, APassword: string);
    constructor CreateProductionPfx(const APfxPath, APassword: string);
    destructor Destroy; override;

    // Autenticacion
    procedure SetTokenCache(const ACacheDir: string);
    procedure Login(const AService: string = 'wsfe');
    function IsAuthenticated: Boolean;

    // WSFEv1
    function FeDummy: string;
    function FeCompUltimoAutorizado(ACUIT: UInt64; APtoVta, ACbteTipo: Integer): Int64;
    function FeCaeSolicitar(ACUIT: UInt64; const AInvoiceJson: string): string;
    function FeCompConsultar(ACUIT: UInt64; ACbteTipo: Integer; ACbteNro: UInt64; APtoVta: Integer): string;
    function FeParamGetTiposCbte(ACUIT: UInt64): string;
    function FeParamGetTiposDoc(ACUIT: UInt64): string;
    function FeParamGetTiposIva(ACUIT: UInt64): string;
    function FeParamGetTiposMonedas(ACUIT: UInt64): string;
    function FeParamGetPtosVenta(ACUIT: UInt64): string;
    function FeParamGetCotizacion(ACUIT: UInt64; const AMonId: string): Double;

    // Padron A13
    function PadronDummy: string;
    function PadronGetPersona(ACUITRepresentada, ACUITConsulta: UInt64): string;
  end;

implementation

// Importaciones de la biblioteca
procedure arca_free_string(s: PAnsiChar); cdecl; external ARCA_LIB;
function arca_last_error: PAnsiChar; cdecl; external ARCA_LIB;

function arca_client_new_testing(cert_path, key_path: PAnsiChar; out handle: TArcaHandle): Integer; cdecl; external ARCA_LIB;
function arca_client_new_production(cert_path, key_path: PAnsiChar; out handle: TArcaHandle): Integer; cdecl; external ARCA_LIB;
function arca_client_new_pfx_testing(pfx_path, password: PAnsiChar; out handle: TArcaHandle): Integer; cdecl; external ARCA_LIB;
function arca_client_new_pfx_production(pfx_path, password: PAnsiChar; out handle: TArcaHandle): Integer; cdecl; external ARCA_LIB;
procedure arca_client_free(handle: TArcaHandle); cdecl; external ARCA_LIB;

function arca_set_token_cache(handle: TArcaHandle; cache_dir: PAnsiChar): Integer; cdecl; external ARCA_LIB;
function arca_login(handle: TArcaHandle; service: PAnsiChar): Integer; cdecl; external ARCA_LIB;
function arca_is_authenticated(handle: TArcaHandle): Integer; cdecl; external ARCA_LIB;

function arca_fe_dummy(handle: TArcaHandle; out out_json: PAnsiChar): Integer; cdecl; external ARCA_LIB;
function arca_fe_comp_ultimo_autorizado(handle: TArcaHandle; cuit: UInt64; pto_vta, cbte_tipo: Integer; out out_numero: Int64): Integer; cdecl; external ARCA_LIB;
function arca_fecae_solicitar(handle: TArcaHandle; cuit: UInt64; invoice_json: PAnsiChar; out out_json: PAnsiChar): Integer; cdecl; external ARCA_LIB;
function arca_fe_comp_consultar(handle: TArcaHandle; cuit: UInt64; cbte_tipo: Integer; cbte_nro: UInt64; pto_vta: Integer; out out_json: PAnsiChar): Integer; cdecl; external ARCA_LIB;
function arca_fe_param_get_tipos_cbte(handle: TArcaHandle; cuit: UInt64; out out_json: PAnsiChar): Integer; cdecl; external ARCA_LIB;
function arca_fe_param_get_tipos_doc(handle: TArcaHandle; cuit: UInt64; out out_json: PAnsiChar): Integer; cdecl; external ARCA_LIB;
function arca_fe_param_get_tipos_iva(handle: TArcaHandle; cuit: UInt64; out out_json: PAnsiChar): Integer; cdecl; external ARCA_LIB;
function arca_fe_param_get_tipos_monedas(handle: TArcaHandle; cuit: UInt64; out out_json: PAnsiChar): Integer; cdecl; external ARCA_LIB;
function arca_fe_param_get_ptos_venta(handle: TArcaHandle; cuit: UInt64; out out_json: PAnsiChar): Integer; cdecl; external ARCA_LIB;
function arca_fe_param_get_cotizacion(handle: TArcaHandle; cuit: UInt64; mon_id: PAnsiChar; out out_cotizacion: Double): Integer; cdecl; external ARCA_LIB;

function arca_padron_dummy(handle: TArcaHandle; out out_json: PAnsiChar): Integer; cdecl; external ARCA_LIB;
function arca_padron_get_persona(handle: TArcaHandle; cuit_representada, cuit_consulta: UInt64; out out_json: PAnsiChar): Integer; cdecl; external ARCA_LIB;

{ EArcaError }

constructor EArcaError.Create(const AMessage: string; ACode: Integer);
begin
  inherited Create(AMessage);
  FCode := ACode;
end;

{ TArcaClient }

procedure TArcaClient.CheckResult(AResult: Integer; const AOperation: string);
var
  ErrorPtr: PAnsiChar;
  ErrorMsg: string;
begin
  if AResult <> 0 then
  begin
    ErrorPtr := arca_last_error;
    if ErrorPtr <> nil then
    begin
      ErrorMsg := string(ErrorPtr);
      arca_free_string(ErrorPtr);
    end
    else
      ErrorMsg := Format('Error code: %d', [AResult]);

    raise EArcaError.Create(Format('%s failed: %s', [AOperation, ErrorMsg]), AResult);
  end;
end;

function TArcaClient.GetAndFreeString(APtr: PAnsiChar): string;
begin
  if APtr <> nil then
  begin
    Result := string(APtr);
    arca_free_string(APtr);
  end
  else
    Result := '';
end;

constructor TArcaClient.CreateTesting(const ACertPath, AKeyPath: string);
begin
  inherited Create;
  CheckResult(
    arca_client_new_testing(PAnsiChar(AnsiString(ACertPath)), PAnsiChar(AnsiString(AKeyPath)), FHandle),
    'Create testing client'
  );
end;

constructor TArcaClient.CreateProduction(const ACertPath, AKeyPath: string);
begin
  inherited Create;
  CheckResult(
    arca_client_new_production(PAnsiChar(AnsiString(ACertPath)), PAnsiChar(AnsiString(AKeyPath)), FHandle),
    'Create production client'
  );
end;

constructor TArcaClient.CreateTestingPfx(const APfxPath, APassword: string);
begin
  inherited Create;
  CheckResult(
    arca_client_new_pfx_testing(PAnsiChar(AnsiString(APfxPath)), PAnsiChar(AnsiString(APassword)), FHandle),
    'Create testing PFX client'
  );
end;

constructor TArcaClient.CreateProductionPfx(const APfxPath, APassword: string);
begin
  inherited Create;
  CheckResult(
    arca_client_new_pfx_production(PAnsiChar(AnsiString(APfxPath)), PAnsiChar(AnsiString(APassword)), FHandle),
    'Create production PFX client'
  );
end;

destructor TArcaClient.Destroy;
begin
  if FHandle <> nil then
    arca_client_free(FHandle);
  inherited;
end;

procedure TArcaClient.SetTokenCache(const ACacheDir: string);
begin
  CheckResult(arca_set_token_cache(FHandle, PAnsiChar(AnsiString(ACacheDir))), 'SetTokenCache');
end;

procedure TArcaClient.Login(const AService: string);
begin
  CheckResult(arca_login(FHandle, PAnsiChar(AnsiString(AService))), 'Login');
end;

function TArcaClient.IsAuthenticated: Boolean;
begin
  Result := arca_is_authenticated(FHandle) = 1;
end;

function TArcaClient.FeDummy: string;
var
  OutJson: PAnsiChar;
begin
  CheckResult(arca_fe_dummy(FHandle, OutJson), 'FEDummy');
  Result := GetAndFreeString(OutJson);
end;

function TArcaClient.FeCompUltimoAutorizado(ACUIT: UInt64; APtoVta, ACbteTipo: Integer): Int64;
begin
  CheckResult(
    arca_fe_comp_ultimo_autorizado(FHandle, ACUIT, APtoVta, ACbteTipo, Result),
    'FECompUltimoAutorizado'
  );
end;

function TArcaClient.FeCaeSolicitar(ACUIT: UInt64; const AInvoiceJson: string): string;
var
  OutJson: PAnsiChar;
begin
  CheckResult(
    arca_fecae_solicitar(FHandle, ACUIT, PAnsiChar(AnsiString(AInvoiceJson)), OutJson),
    'FECAESolicitar'
  );
  Result := GetAndFreeString(OutJson);
end;

function TArcaClient.FeCompConsultar(ACUIT: UInt64; ACbteTipo: Integer; ACbteNro: UInt64; APtoVta: Integer): string;
var
  OutJson: PAnsiChar;
begin
  CheckResult(
    arca_fe_comp_consultar(FHandle, ACUIT, ACbteTipo, ACbteNro, APtoVta, OutJson),
    'FECompConsultar'
  );
  Result := GetAndFreeString(OutJson);
end;

function TArcaClient.FeParamGetTiposCbte(ACUIT: UInt64): string;
var
  OutJson: PAnsiChar;
begin
  CheckResult(arca_fe_param_get_tipos_cbte(FHandle, ACUIT, OutJson), 'FEParamGetTiposCbte');
  Result := GetAndFreeString(OutJson);
end;

function TArcaClient.FeParamGetTiposDoc(ACUIT: UInt64): string;
var
  OutJson: PAnsiChar;
begin
  CheckResult(arca_fe_param_get_tipos_doc(FHandle, ACUIT, OutJson), 'FEParamGetTiposDoc');
  Result := GetAndFreeString(OutJson);
end;

function TArcaClient.FeParamGetTiposIva(ACUIT: UInt64): string;
var
  OutJson: PAnsiChar;
begin
  CheckResult(arca_fe_param_get_tipos_iva(FHandle, ACUIT, OutJson), 'FEParamGetTiposIva');
  Result := GetAndFreeString(OutJson);
end;

function TArcaClient.FeParamGetTiposMonedas(ACUIT: UInt64): string;
var
  OutJson: PAnsiChar;
begin
  CheckResult(arca_fe_param_get_tipos_monedas(FHandle, ACUIT, OutJson), 'FEParamGetTiposMonedas');
  Result := GetAndFreeString(OutJson);
end;

function TArcaClient.FeParamGetPtosVenta(ACUIT: UInt64): string;
var
  OutJson: PAnsiChar;
begin
  CheckResult(arca_fe_param_get_ptos_venta(FHandle, ACUIT, OutJson), 'FEParamGetPtosVenta');
  Result := GetAndFreeString(OutJson);
end;

function TArcaClient.FeParamGetCotizacion(ACUIT: UInt64; const AMonId: string): Double;
begin
  CheckResult(
    arca_fe_param_get_cotizacion(FHandle, ACUIT, PAnsiChar(AnsiString(AMonId)), Result),
    'FEParamGetCotizacion'
  );
end;

function TArcaClient.PadronDummy: string;
var
  OutJson: PAnsiChar;
begin
  CheckResult(arca_padron_dummy(FHandle, OutJson), 'PadronDummy');
  Result := GetAndFreeString(OutJson);
end;

function TArcaClient.PadronGetPersona(ACUITRepresentada, ACUITConsulta: UInt64): string;
var
  OutJson: PAnsiChar;
begin
  CheckResult(
    arca_padron_get_persona(FHandle, ACUITRepresentada, ACUITConsulta, OutJson),
    'GetPersona'
  );
  Result := GetAndFreeString(OutJson);
end;

end.
