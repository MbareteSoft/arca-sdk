/**
 * AFIP Rust Library - C Header
 *
 * Biblioteca para integración con servicios web de AFIP (Argentina)
 *
 * Uso:
 *   - Incluir este header en tu proyecto C/C++
 *   - Linkear con libafip.so (Linux) / afip.dll (Windows)
 *   - Llamar arca_free_string() para liberar strings retornados
 *   - Llamar arca_client_free() para liberar el cliente
 *
 * Ejemplo básico:
 *   ArcaHandle* client;
 *   int ret = arca_client_new_pfx_production("cert.pfx", "", &client);
 *   if (ret != 0) {
 *       char* error = arca_last_error();
 *       printf("Error: %s\n", error);
 *       arca_free_string(error);
 *       return 1;
 *   }
 *
 *   ret = arca_login(client, "wsfe");
 *   // ... usar el cliente ...
 *   arca_client_free(client);
 */

#ifndef ARCA_H
#define ARCA_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * Tipos
 * ============================================================================ */

/**
 * Handle opaco al cliente AFIP.
 * Crear con afip_client_new_* y liberar con arca_client_free()
 */
typedef struct ArcaHandle ArcaHandle;

/* ============================================================================
 * Manejo de Memoria
 * ============================================================================ */

/**
 * Libera una cadena retornada por funciones AFIP.
 * DEBE llamarse para cada string retornado para evitar memory leaks.
 *
 * @param s Puntero a la cadena a liberar (puede ser NULL)
 */
void arca_free_string(char* s);

/**
 * Obtiene el último mensaje de error.
 *
 * @return Mensaje de error o NULL si no hay error.
 *         El string retornado DEBE ser liberado con arca_free_string()
 */
char* arca_last_error(void);

/* ============================================================================
 * Creación y Destrucción del Cliente
 * ============================================================================ */

/**
 * Crea un cliente AFIP para el ambiente de testing (homologación) usando PEM.
 *
 * @param cert_pem_path Ruta al archivo de certificado PEM
 * @param key_pem_path  Ruta al archivo de clave privada PEM
 * @param out_handle    Puntero donde se almacenará el handle del cliente
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_client_new_testing(
    const char* cert_pem_path,
    const char* key_pem_path,
    ArcaHandle** out_handle
);

/**
 * Crea un cliente AFIP para el ambiente de producción usando PEM.
 *
 * @param cert_pem_path Ruta al archivo de certificado PEM
 * @param key_pem_path  Ruta al archivo de clave privada PEM
 * @param out_handle    Puntero donde se almacenará el handle del cliente
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_client_new_production(
    const char* cert_pem_path,
    const char* key_pem_path,
    ArcaHandle** out_handle
);

/**
 * Crea un cliente AFIP para testing usando PFX/PKCS#12.
 *
 * @param pfx_path     Ruta al archivo PFX
 * @param pfx_password Contraseña del PFX (puede ser cadena vacía "")
 * @param out_handle   Puntero donde se almacenará el handle del cliente
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_client_new_pfx_testing(
    const char* pfx_path,
    const char* pfx_password,
    ArcaHandle** out_handle
);

/**
 * Crea un cliente AFIP para producción usando PFX/PKCS#12.
 *
 * @param pfx_path     Ruta al archivo PFX
 * @param pfx_password Contraseña del PFX (puede ser cadena vacía "")
 * @param out_handle   Puntero donde se almacenará el handle del cliente
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_client_new_pfx_production(
    const char* pfx_path,
    const char* pfx_password,
    ArcaHandle** out_handle
);

/**
 * Libera el cliente AFIP.
 * DEBE llamarse cuando ya no se necesite el cliente.
 *
 * @param handle Handle del cliente a liberar
 */
void arca_client_free(ArcaHandle* handle);

/* ============================================================================
 * Autenticación
 * ============================================================================ */

/**
 * Habilita la persistencia de tokens en disco.
 *
 * Los tokens se guardan/cargan como archivos JSON en el directorio especificado.
 * El directorio se crea automáticamente si no existe.
 * Llamar ANTES de arca_login() para activar la caché persistente.
 *
 * Sin esta llamada, los tokens solo se mantienen en memoria (comportamiento original).
 *
 * @param handle    Handle del cliente AFIP
 * @param cache_dir Ruta al directorio para archivos de caché de tokens
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_set_token_cache(ArcaHandle* handle, const char* cache_dir);

/**
 * Autentica con WSAA para un servicio específico.
 *
 * Si la caché de tokens está habilitada (via arca_set_token_cache),
 * verifica primero en memoria y luego en disco antes de contactar a WSAA.
 *
 * @param handle  Handle del cliente AFIP
 * @param service Nombre del servicio (ej: "wsfe", "ws_sr_padron_a13")
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_login(ArcaHandle* handle, const char* service);

/**
 * Verifica si el cliente está autenticado.
 *
 * @param handle Handle del cliente AFIP
 *
 * @return 1 si está autenticado, 0 si no
 */
int arca_is_authenticated(const ArcaHandle* handle);

/* ============================================================================
 * WSFEv1 - Facturación Electrónica
 * ============================================================================ */

/**
 * Verifica el estado del servicio WSFEv1 (FEDummy).
 *
 * @param handle   Handle del cliente AFIP
 * @param out_json Puntero donde se almacenará el JSON de respuesta.
 *                 DEBE ser liberado con arca_free_string()
 *
 * Formato JSON:
 * {
 *   "AppServer": "OK",
 *   "AuthServer": "OK",
 *   "DbServer": "OK"
 * }
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_fe_dummy(ArcaHandle* handle, char** out_json);

/**
 * Obtiene el último comprobante autorizado (FECompUltimoAutorizado).
 *
 * @param handle     Handle del cliente AFIP
 * @param cuit       CUIT del contribuyente
 * @param pto_vta    Punto de venta
 * @param cbte_tipo  Tipo de comprobante (usar constantes ARCA_CBTE_*)
 * @param out_numero Puntero donde se almacenará el número
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_fe_comp_ultimo_autorizado(
    ArcaHandle* handle,
    uint64_t cuit,
    int32_t pto_vta,
    int32_t cbte_tipo,
    int64_t* out_numero
);

/**
 * Solicita CAE para una factura (FECAESolicitar).
 *
 * @param handle       Handle del cliente AFIP
 * @param cuit         CUIT del contribuyente
 * @param invoice_json JSON con los datos de la factura
 * @param out_json     Puntero donde se almacenará el JSON de respuesta.
 *                     DEBE ser liberado con arca_free_string()
 *
 * Formato invoice_json:
 * {
 *   "pto_vta": 1,
 *   "cbte_tipo": 11,
 *   "concepto": 1,
 *   "doc_tipo": 99,
 *   "doc_nro": 0,
 *   "cbte_desde": 1,
 *   "cbte_hasta": 1,
 *   "cbte_fch": "20251226",
 *   "imp_total": 1000.00,
 *   "imp_tot_conc": 0.00,
 *   "imp_neto": 1000.00,
 *   "imp_op_ex": 0.00,
 *   "imp_trib": 0.00,
 *   "imp_iva": 0.00,
 *   "mon_id": "PES",
 *   "mon_cotiz": 1.0,
 *   "condicion_iva_receptor": 5,
 *   "iva": []
 * }
 *
 * Formato respuesta:
 * {
 *   "resultado": "A",
 *   "cae": "75524435551136",
 *   "cae_fch_vto": "20260105",
 *   "cbte_desde": 1,
 *   "cbte_hasta": 1,
 *   "errors": [],
 *   "observaciones": [],
 *   "events": []
 * }
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_fecae_solicitar(
    ArcaHandle* handle,
    uint64_t cuit,
    const char* invoice_json,
    char** out_json
);

/**
 * Consulta un comprobante emitido (FECompConsultar).
 *
 * @param handle   Handle del cliente AFIP
 * @param cuit     CUIT del contribuyente
 * @param cbte_tipo Tipo de comprobante
 * @param cbte_nro  Número de comprobante
 * @param pto_vta   Punto de venta
 * @param out_json  Puntero donde se almacenará el JSON de respuesta.
 *                  DEBE ser liberado con arca_free_string()
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_fe_comp_consultar(
    ArcaHandle* handle,
    uint64_t cuit,
    int32_t cbte_tipo,
    uint64_t cbte_nro,
    int32_t pto_vta,
    char** out_json
);

/**
 * Obtiene los tipos de comprobante disponibles (FEParamGetTiposCbte).
 *
 * @param handle   Handle del cliente AFIP
 * @param cuit     CUIT del contribuyente
 * @param out_json Puntero donde se almacenará el JSON de respuesta.
 *                 DEBE ser liberado con arca_free_string()
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_fe_param_get_tipos_cbte(ArcaHandle* handle, uint64_t cuit, char** out_json);

/**
 * Obtiene los tipos de documento disponibles (FEParamGetTiposDoc).
 */
int arca_fe_param_get_tipos_doc(ArcaHandle* handle, uint64_t cuit, char** out_json);

/**
 * Obtiene las alícuotas de IVA disponibles (FEParamGetTiposIva).
 */
int arca_fe_param_get_tipos_iva(ArcaHandle* handle, uint64_t cuit, char** out_json);

/**
 * Obtiene las monedas disponibles (FEParamGetTiposMonedas).
 */
int arca_fe_param_get_tipos_monedas(ArcaHandle* handle, uint64_t cuit, char** out_json);

/**
 * Obtiene los puntos de venta (FEParamGetPtosVenta).
 */
int arca_fe_param_get_ptos_venta(ArcaHandle* handle, uint64_t cuit, char** out_json);

/**
 * Obtiene la cotización de una moneda (FEParamGetCotizacion).
 *
 * @param handle        Handle del cliente AFIP
 * @param cuit          CUIT del contribuyente
 * @param mon_id        ID de la moneda (ej: "DOL", "EUR")
 * @param out_cotizacion Puntero donde se almacenará la cotización
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_fe_param_get_cotizacion(
    ArcaHandle* handle,
    uint64_t cuit,
    const char* mon_id,
    double* out_cotizacion
);

/* ============================================================================
 * Padrón A13 - Registro de Contribuyentes
 * ============================================================================ */

/**
 * Verifica el estado del servicio Padrón A13 (Dummy).
 *
 * @param handle   Handle del cliente AFIP
 * @param out_json Puntero donde se almacenará el JSON de respuesta.
 *                 DEBE ser liberado con arca_free_string()
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_padron_dummy(ArcaHandle* handle, char** out_json);

/**
 * Consulta datos de una persona por CUIT (GetPersona).
 * NOTA: Requiere autenticación previa con servicio "ws_sr_padron_a13"
 *
 * @param handle           Handle del cliente AFIP
 * @param cuit_representada CUIT que representa (usualmente el mismo que consulta)
 * @param cuit_consulta     CUIT a consultar
 * @param out_json          Puntero donde se almacenará el JSON de respuesta.
 *                          DEBE ser liberado con arca_free_string()
 *
 * @return 0 en éxito, código de error negativo en falla
 */
int arca_padron_get_persona(
    ArcaHandle* handle,
    uint64_t cuit_representada,
    uint64_t cuit_consulta,
    char** out_json
);

/* ============================================================================
 * Constantes - Tipos de Comprobante
 * ============================================================================ */

int ARCA_CBTE_FACTURA_A(void);      /* 1 */
int ARCA_CBTE_NOTA_DEBITO_A(void);  /* 2 */
int ARCA_CBTE_NOTA_CREDITO_A(void); /* 3 */
int ARCA_CBTE_FACTURA_B(void);      /* 6 */
int ARCA_CBTE_NOTA_DEBITO_B(void);  /* 7 */
int ARCA_CBTE_NOTA_CREDITO_B(void); /* 8 */
int ARCA_CBTE_FACTURA_C(void);      /* 11 */
int ARCA_CBTE_NOTA_DEBITO_C(void);  /* 12 */
int ARCA_CBTE_NOTA_CREDITO_C(void); /* 13 */

/* ============================================================================
 * Constantes - Tipos de Documento
 * ============================================================================ */

int ARCA_DOC_CUIT(void);             /* 80 */
int ARCA_DOC_CUIL(void);             /* 86 */
int ARCA_DOC_CDI(void);              /* 87 */
int ARCA_DOC_DNI(void);              /* 96 */
int ARCA_DOC_PASAPORTE(void);        /* 94 */
int ARCA_DOC_CONSUMIDOR_FINAL(void); /* 99 */

/* ============================================================================
 * Constantes - Condición IVA
 * ============================================================================ */

int ARCA_IVA_RESPONSABLE_INSCRIPTO(void); /* 1 */
int ARCA_IVA_SUJETO_EXENTO(void);         /* 4 */
int ARCA_IVA_CONSUMIDOR_FINAL(void);      /* 5 */
int ARCA_IVA_RESPONSABLE_MONOTRIBUTO(void); /* 6 */

/* ============================================================================
 * Constantes - Alícuotas IVA
 * ============================================================================ */

int ARCA_ALIC_NO_GRAVADO(void);   /* 1 */
int ARCA_ALIC_EXENTO(void);       /* 2 */
int ARCA_ALIC_CERO(void);         /* 3 */
int ARCA_ALIC_DIEZ_CINCO(void);   /* 4 - 10.5% */
int ARCA_ALIC_VEINTIUNO(void);    /* 5 - 21% */
int ARCA_ALIC_VEINTISIETE(void);  /* 6 - 27% */
int ARCA_ALIC_CINCO(void);        /* 8 - 5% */
int ARCA_ALIC_DOS_CINCO(void);    /* 9 - 2.5% */

/* ============================================================================
 * Códigos de Error
 * ============================================================================ */

/*
 * Errores generales:
 *   -1  = Argumento nulo
 *   -2  = Cadena inválida (UTF-8)
 *   -3  = Error leyendo archivo
 *   -4  = Error creando runtime
 *   -5  = Error creando cliente HTTP
 *   -6  = Error convirtiendo PFX
 *
 * Errores de autenticación:
 *   -10 = Error firmando CMS
 *   -11 = Login fallido
 *   -12 = No autenticado
 *
 * Errores WSFEv1:
 *   -20 = FEDummy fallido
 *   -21 = FECompUltimoAutorizado fallido
 *   -22 = FECAESolicitar fallido
 *   -23 = FECompConsultar fallido
 *   -24 = FEParamGetTiposCbte fallido
 *   -25 = FEParamGetTiposDoc fallido
 *   -26 = FEParamGetTiposIva fallido
 *   -27 = FEParamGetTiposMonedas fallido
 *   -28 = FEParamGetPtosVenta fallido
 *   -29 = FEParamGetCotizacion fallido
 *   -30 = Error parseando JSON de factura
 *
 * Errores Padrón:
 *   -40 = Padron Dummy fallido
 *   -41 = GetPersona fallido
 */

#ifdef __cplusplus
}
#endif

#endif /* ARCA_H */
