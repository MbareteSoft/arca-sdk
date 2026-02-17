#!/usr/bin/env python3
"""
ARCA SDK - Python Wrapper

Ejemplo de uso desde Python usando ctypes.

Requisitos:
    - Python 3.7+
    - libaraca.so debe estar en el mismo directorio o en LD_LIBRARY_PATH

Uso:
    python afip_client.py
"""

import ctypes
import json
from datetime import datetime
from pathlib import Path
from typing import Optional, Any

# Cargar la biblioteca
def load_library():
    """Carga la biblioteca ARCA según el sistema operativo."""
    import platform

    lib_name = {
        'Linux': 'libarca.so',
        'Darwin': 'libarca.dylib',
        'Windows': 'arca.dll'
    }.get(platform.system(), 'libarca.so')

    # Buscar en directorio actual o del script
    lib_path = Path(__file__).parent / lib_name
    if not lib_path.exists():
        lib_path = Path('.') / lib_name
    if not lib_path.exists():
        lib_path = Path('../target/release') / lib_name

    return ctypes.CDLL(str(lib_path))

# Cargar la biblioteca
try:
    _lib = load_library()
except OSError as e:
    raise ImportError(f"No se pudo cargar la biblioteca ARCA: {e}")

# Tipos
class ArcaHandle(ctypes.c_void_p):
    """Handle opaco al cliente ARCA."""
    pass

# Configurar funciones
_lib.arca_free_string.argtypes = [ctypes.c_char_p]
_lib.arca_free_string.restype = None

_lib.arca_last_error.argtypes = []
_lib.arca_last_error.restype = ctypes.c_char_p

_lib.arca_client_new_testing.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.POINTER(ArcaHandle)]
_lib.arca_client_new_testing.restype = ctypes.c_int

_lib.arca_client_new_production.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.POINTER(ArcaHandle)]
_lib.arca_client_new_production.restype = ctypes.c_int

_lib.arca_client_new_pfx_testing.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.POINTER(ArcaHandle)]
_lib.arca_client_new_pfx_testing.restype = ctypes.c_int

_lib.arca_client_new_pfx_production.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.POINTER(ArcaHandle)]
_lib.arca_client_new_pfx_production.restype = ctypes.c_int

_lib.arca_client_free.argtypes = [ArcaHandle]
_lib.arca_client_free.restype = None

_lib.arca_login.argtypes = [ArcaHandle, ctypes.c_char_p]
_lib.arca_login.restype = ctypes.c_int

_lib.arca_set_token_cache.argtypes = [ArcaHandle, ctypes.c_char_p]
_lib.arca_set_token_cache.restype = ctypes.c_int

_lib.arca_is_authenticated.argtypes = [ArcaHandle]
_lib.arca_is_authenticated.restype = ctypes.c_int

_lib.arca_fe_dummy.argtypes = [ArcaHandle, ctypes.POINTER(ctypes.c_char_p)]
_lib.arca_fe_dummy.restype = ctypes.c_int

_lib.arca_fe_comp_ultimo_autorizado.argtypes = [ArcaHandle, ctypes.c_uint64, ctypes.c_int32, ctypes.c_int32, ctypes.POINTER(ctypes.c_int64)]
_lib.arca_fe_comp_ultimo_autorizado.restype = ctypes.c_int

_lib.arca_fecae_solicitar.argtypes = [ArcaHandle, ctypes.c_uint64, ctypes.c_char_p, ctypes.POINTER(ctypes.c_char_p)]
_lib.arca_fecae_solicitar.restype = ctypes.c_int

_lib.arca_fe_comp_consultar.argtypes = [ArcaHandle, ctypes.c_uint64, ctypes.c_int32, ctypes.c_uint64, ctypes.c_int32, ctypes.POINTER(ctypes.c_char_p)]
_lib.arca_fe_comp_consultar.restype = ctypes.c_int

_lib.arca_fe_param_get_tipos_cbte.argtypes = [ArcaHandle, ctypes.c_uint64, ctypes.POINTER(ctypes.c_char_p)]
_lib.arca_fe_param_get_tipos_cbte.restype = ctypes.c_int

_lib.arca_fe_param_get_cotizacion.argtypes = [ArcaHandle, ctypes.c_uint64, ctypes.c_char_p, ctypes.POINTER(ctypes.c_double)]
_lib.arca_fe_param_get_cotizacion.restype = ctypes.c_int

_lib.arca_padron_dummy.argtypes = [ArcaHandle, ctypes.POINTER(ctypes.c_char_p)]
_lib.arca_padron_dummy.restype = ctypes.c_int

_lib.arca_padron_get_persona.argtypes = [ArcaHandle, ctypes.c_uint64, ctypes.c_uint64, ctypes.POINTER(ctypes.c_char_p)]
_lib.arca_padron_get_persona.restype = ctypes.c_int

# Constantes
_lib.ARCA_CBTE_FACTURA_A.restype = ctypes.c_int
_lib.ARCA_CBTE_FACTURA_B.restype = ctypes.c_int
_lib.ARCA_CBTE_FACTURA_C.restype = ctypes.c_int
_lib.ARCA_CBTE_NOTA_DEBITO_A.restype = ctypes.c_int
_lib.ARCA_CBTE_NOTA_DEBITO_B.restype = ctypes.c_int
_lib.ARCA_CBTE_NOTA_DEBITO_C.restype = ctypes.c_int
_lib.ARCA_CBTE_NOTA_CREDITO_A.restype = ctypes.c_int
_lib.ARCA_CBTE_NOTA_CREDITO_B.restype = ctypes.c_int
_lib.ARCA_CBTE_NOTA_CREDITO_C.restype = ctypes.c_int
_lib.ARCA_DOC_CUIT.restype = ctypes.c_int
_lib.ARCA_DOC_DNI.restype = ctypes.c_int
_lib.ARCA_DOC_CONSUMIDOR_FINAL.restype = ctypes.c_int
_lib.ARCA_IVA_RESPONSABLE_INSCRIPTO.restype = ctypes.c_int
_lib.ARCA_IVA_CONSUMIDOR_FINAL.restype = ctypes.c_int
_lib.ARCA_IVA_RESPONSABLE_MONOTRIBUTO.restype = ctypes.c_int
_lib.ARCA_ALIC_VEINTIUNO.restype = ctypes.c_int


class CbteTipos:
    """Tipos de comprobante"""
    FACTURA_A = _lib.ARCA_CBTE_FACTURA_A()
    FACTURA_B = _lib.ARCA_CBTE_FACTURA_B()
    FACTURA_C = _lib.ARCA_CBTE_FACTURA_C()
    NOTA_DEBITO_A = _lib.ARCA_CBTE_NOTA_DEBITO_A()
    NOTA_DEBITO_B = _lib.ARCA_CBTE_NOTA_DEBITO_B()
    NOTA_DEBITO_C = _lib.ARCA_CBTE_NOTA_DEBITO_C()
    NOTA_CREDITO_A = _lib.ARCA_CBTE_NOTA_CREDITO_A()
    NOTA_CREDITO_B = _lib.ARCA_CBTE_NOTA_CREDITO_B()
    NOTA_CREDITO_C = _lib.ARCA_CBTE_NOTA_CREDITO_C()


class DocTipos:
    """Tipos de documento"""
    CUIT = _lib.ARCA_DOC_CUIT()
    DNI = _lib.ARCA_DOC_DNI()
    CONSUMIDOR_FINAL = _lib.ARCA_DOC_CONSUMIDOR_FINAL()


class CondicionIva:
    """Condiciones IVA del receptor"""
    RESPONSABLE_INSCRIPTO = _lib.ARCA_IVA_RESPONSABLE_INSCRIPTO()
    CONSUMIDOR_FINAL = _lib.ARCA_IVA_CONSUMIDOR_FINAL()
    RESPONSABLE_MONOTRIBUTO = _lib.ARCA_IVA_RESPONSABLE_MONOTRIBUTO()


class AlicuotasIva:
    """Alícuotas de IVA"""
    VEINTIUNO = _lib.ARCA_ALIC_VEINTIUNO()


class ArcaError(Exception):
    """Excepción para errores de ARCA"""
    def __init__(self, message: str, code: int):
        super().__init__(message)
        self.code = code


def _get_last_error() -> Optional[str]:
    """Obtiene el último mensaje de error."""
    ptr = _lib.arca_last_error()
    if ptr:
        error = ptr.decode('utf-8')
        _lib.arca_free_string(ptr)
        return error
    return None


def _check_result(result: int, operation: str):
    """Verifica el resultado de una operación."""
    if result != 0:
        error = _get_last_error() or f"Error code: {result}"
        raise ArcaError(f"{operation} failed: {error}", result)


def _get_and_free_string(ptr: ctypes.c_char_p) -> Optional[str]:
    """Obtiene una cadena y la libera."""
    if ptr:
        result = ptr.value.decode('utf-8') if ptr.value else None
        _lib.arca_free_string(ptr)
        return result
    return None


class ArcaClient:
    """
    Cliente para integración con ARCA.

    Uso:
        # Crear cliente
        client = ArcaClient.production_pfx("certificado.pfx", "")

        # Autenticar
        client.login("wsfe")

        # Usar servicios
        ultimo = client.fe_comp_ultimo_autorizado(cuit, pto_vta, CbteTipos.FACTURA_C)
    """

    def __init__(self, handle: ArcaHandle):
        self._handle = handle

    @classmethod
    def testing(cls, cert_path: str, key_path: str) -> 'ArcaClient':
        """Crea un cliente para el ambiente de testing con certificados PEM."""
        handle = ArcaHandle()
        result = _lib.arca_client_new_testing(
            cert_path.encode('utf-8'),
            key_path.encode('utf-8'),
            ctypes.byref(handle)
        )
        _check_result(result, "Create testing client")
        return cls(handle)

    @classmethod
    def production(cls, cert_path: str, key_path: str) -> 'ArcaClient':
        """Crea un cliente para el ambiente de producción con certificados PEM."""
        handle = ArcaHandle()
        result = _lib.arca_client_new_production(
            cert_path.encode('utf-8'),
            key_path.encode('utf-8'),
            ctypes.byref(handle)
        )
        _check_result(result, "Create production client")
        return cls(handle)

    @classmethod
    def testing_pfx(cls, pfx_path: str, password: str = "") -> 'ArcaClient':
        """Crea un cliente para testing con certificado PFX."""
        handle = ArcaHandle()
        result = _lib.arca_client_new_pfx_testing(
            pfx_path.encode('utf-8'),
            password.encode('utf-8'),
            ctypes.byref(handle)
        )
        _check_result(result, "Create testing PFX client")
        return cls(handle)

    @classmethod
    def production_pfx(cls, pfx_path: str, password: str = "") -> 'ArcaClient':
        """Crea un cliente para producción con certificado PFX."""
        handle = ArcaHandle()
        result = _lib.arca_client_new_pfx_production(
            pfx_path.encode('utf-8'),
            password.encode('utf-8'),
            ctypes.byref(handle)
        )
        _check_result(result, "Create production PFX client")
        return cls(handle)

    def __del__(self):
        """Libera el cliente."""
        if hasattr(self, '_handle') and self._handle:
            _lib.arca_client_free(self._handle)

    def __enter__(self):
        return self

    def __exit__(self, *args):
        if self._handle:
            _lib.arca_client_free(self._handle)
            self._handle = None

    def set_token_cache(self, cache_dir: str) -> None:
        """Configura el directorio de cache para persistir tokens WSAA en disco.
        Llamar antes de login(). Evita lockouts de 12h si el proceso se reinicia."""
        result = _lib.arca_set_token_cache(self._handle, cache_dir.encode('utf-8'))
        _check_result(result, "SetTokenCache")

    def login(self, service: str = "wsfe") -> None:
        """Autentica con WSAA para un servicio específico."""
        result = _lib.arca_login(self._handle, service.encode('utf-8'))
        _check_result(result, "Login")

    @property
    def is_authenticated(self) -> bool:
        """Verifica si el cliente está autenticado."""
        return _lib.arca_is_authenticated(self._handle) == 1

    def fe_dummy(self) -> dict:
        """Verifica el estado del servicio WSFEv1."""
        out_json = ctypes.c_char_p()
        result = _lib.arca_fe_dummy(self._handle, ctypes.byref(out_json))
        _check_result(result, "FEDummy")
        json_str = _get_and_free_string(out_json)
        return json.loads(json_str) if json_str else {}

    def fe_comp_ultimo_autorizado(self, cuit: int, pto_vta: int, cbte_tipo: int) -> int:
        """Obtiene el último comprobante autorizado."""
        out_numero = ctypes.c_int64()
        result = _lib.arca_fe_comp_ultimo_autorizado(
            self._handle, cuit, pto_vta, cbte_tipo, ctypes.byref(out_numero)
        )
        _check_result(result, "FECompUltimoAutorizado")
        return out_numero.value

    def fecae_solicitar(self, cuit: int, invoice: dict) -> dict:
        """Solicita CAE para una factura."""
        invoice_json = json.dumps(invoice).encode('utf-8')
        out_json = ctypes.c_char_p()
        result = _lib.arca_fecae_solicitar(
            self._handle, cuit, invoice_json, ctypes.byref(out_json)
        )
        _check_result(result, "FECAESolicitar")
        json_str = _get_and_free_string(out_json)
        return json.loads(json_str) if json_str else {}

    def fe_comp_consultar(self, cuit: int, cbte_tipo: int, cbte_nro: int, pto_vta: int) -> dict:
        """Consulta un comprobante emitido."""
        out_json = ctypes.c_char_p()
        result = _lib.arca_fe_comp_consultar(
            self._handle, cuit, cbte_tipo, cbte_nro, pto_vta, ctypes.byref(out_json)
        )
        _check_result(result, "FECompConsultar")
        json_str = _get_and_free_string(out_json)
        return json.loads(json_str) if json_str else {}

    def fe_param_get_tipos_cbte(self, cuit: int) -> list:
        """Obtiene los tipos de comprobante disponibles."""
        out_json = ctypes.c_char_p()
        result = _lib.arca_fe_param_get_tipos_cbte(self._handle, cuit, ctypes.byref(out_json))
        _check_result(result, "FEParamGetTiposCbte")
        json_str = _get_and_free_string(out_json)
        return json.loads(json_str) if json_str else []

    def fe_param_get_cotizacion(self, cuit: int, mon_id: str) -> float:
        """Obtiene la cotización de una moneda."""
        out_cotizacion = ctypes.c_double()
        result = _lib.arca_fe_param_get_cotizacion(
            self._handle, cuit, mon_id.encode('utf-8'), ctypes.byref(out_cotizacion)
        )
        _check_result(result, "FEParamGetCotizacion")
        return out_cotizacion.value

    def padron_dummy(self) -> dict:
        """Verifica el estado del servicio Padrón A13."""
        out_json = ctypes.c_char_p()
        result = _lib.arca_padron_dummy(self._handle, ctypes.byref(out_json))
        _check_result(result, "PadronDummy")
        json_str = _get_and_free_string(out_json)
        return json.loads(json_str) if json_str else {}

    def padron_get_persona(self, cuit_representada: int, cuit_consulta: int) -> dict:
        """Consulta datos de una persona por CUIT."""
        out_json = ctypes.c_char_p()
        result = _lib.arca_padron_get_persona(
            self._handle, cuit_representada, cuit_consulta, ctypes.byref(out_json)
        )
        _check_result(result, "GetPersona")
        json_str = _get_and_free_string(out_json)
        return json.loads(json_str) if json_str else {}


# Ejemplo de uso
if __name__ == "__main__":
    CUIT = 20299879624
    PTO_VTA = 2

    try:
        # Crear cliente con PFX
        with ArcaClient.production_pfx("certificado.pfx", "") as client:
            # Habilitar cache de tokens (evita lockouts de 12h)
            client.set_token_cache("./cache")

            # Autenticar
            client.login("wsfe")
            print("Login exitoso!")

            # Verificar servicio
            dummy = client.fe_dummy()
            print(f"AppServer: {dummy['AppServer']}")

            # Obtener último comprobante
            ultimo = client.fe_comp_ultimo_autorizado(CUIT, PTO_VTA, CbteTipos.FACTURA_C)
            print(f"Último comprobante: {ultimo}")

            # Emitir factura
            invoice = {
                "pto_vta": PTO_VTA,
                "cbte_tipo": CbteTipos.FACTURA_C,
                "concepto": 1,
                "doc_tipo": DocTipos.CONSUMIDOR_FINAL,
                "doc_nro": 0,
                "cbte_desde": ultimo + 1,
                "cbte_hasta": ultimo + 1,
                "cbte_fch": datetime.now().strftime("%Y%m%d"),
                "imp_total": 1.0,
                "imp_tot_conc": 0.0,
                "imp_neto": 1.0,
                "imp_op_ex": 0.0,
                "imp_trib": 0.0,
                "imp_iva": 0.0,
                "mon_id": "PES",
                "mon_cotiz": 1.0,
                "condicion_iva_receptor": CondicionIva.CONSUMIDOR_FINAL,
                "iva": []
            }

            result = client.fecae_solicitar(CUIT, invoice)
            print(f"Resultado: {result['resultado']}")
            print(f"CAE: {result['cae']}")
            print(f"Vencimiento: {result['cae_fch_vto']}")

    except ArcaError as e:
        print(f"Error AFIP [{e.code}]: {e}")
