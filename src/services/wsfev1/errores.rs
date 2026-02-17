// Códigos de error y observación AFIP WSFEv1
// Fuente: Manual desarrollador ARCA-COMPG v4.0
//
// Los códigos están organizados por rango:
// - 200-299: FCE MiPyMEs
// - 700-799: Validaciones campos básicos
// - 800-899: Comprobantes asociados
// - 900-999: Tributos y actividades
// - 1000-1099: Opcionales RG específicas
// - 1100-1199: Opcionales valores
// - 1200-1299: CAEA validación
// - 1300-1399: CAEA puntos venta
// - 1400-1499: Validaciones generales
// - 1500-1599: CAEA solicitud
// - 2000-2199: FCE CBU/opcionales
// - 10000-10999: Errores de autorización y validación
// - 11000-11999: Puntos de venta
// - 15000-15999: CAEA período
// - 16000+: CAEA actividades

/// Errores FCE MiPyMEs (200s)
pub mod fce {
    /// FCE A: FchVtoPago obligatorio
    pub const FCE_A_VTO_OBLIGATORIO: i32 = 206;
    /// FCE B: FchVtoPago obligatorio
    pub const FCE_B_VTO_OBLIGATORIO: i32 = 211;
}

/// Validaciones campos básicos (700s)
pub mod campos {
    /// CbteTipo inválido
    pub const CBTE_TIPO_INVALIDO: i32 = 700;
    /// PtoVta inválido
    pub const PTO_VTA_INVALIDO: i32 = 701;
    /// DocNro no registrado en padrón
    pub const DOC_NO_EN_PADRON: i32 = 708;
    /// CAEA/PtoVta inválido
    pub const CAEA_PTO_VTA_INVALIDO: i32 = 709;
    /// ImpTotConc inválido
    pub const IMP_TOT_CONC_INVALIDO: i32 = 717;
    /// Importes no coinciden
    pub const IMPORTES_NO_COINCIDEN: i32 = 724;
    /// CbteFch formato inválido (yyyymmdd)
    pub const CBTE_FCH_FORMATO: i32 = 783;
    /// DocTipo/DocNro inválido
    pub const DOC_TIPO_NRO_INVALIDO: i32 = 788;
}

/// Comprobantes asociados (800s)
pub mod cbte_asoc {
    /// CbteAsoc debe existir en AFIP
    pub const DEBE_EXISTIR: i32 = 801;
    /// Nro CbteAsoc inválido
    pub const NRO_INVALIDO: i32 = 803;
    /// CbteAsoc no encontrado
    pub const NO_ENCONTRADO: i32 = 809;
    /// Monto excede límite Monotributo
    pub const EXCEDE_LIMITE_MONOTRIBUTO: i32 = 816;
    /// DocNro receptor inválido
    pub const DOC_RECEPTOR_INVALIDO: i32 = 819;
}

/// Tributos y actividades (900s)
pub mod tributos {
    /// Desc tributo inválida
    pub const DESC_INVALIDA: i32 = 908;
    /// Actividad requerida para remito
    pub const ACTIVIDAD_REMITO_REQUERIDA: i32 = 995;
}

/// Opcionales RG específicas (1012, 1xxx)
pub mod opcionales_rg {
    /// RG 3368 Doc titular pago
    pub const RG_3368_DOC_TITULAR: i32 = 1012;
    /// Valor opcional inválido
    pub const VALOR_INVALIDO: i32 = 1105;
}

/// CAEA validación (1200s)
pub mod caea_val {
    /// CAEA código inválido
    pub const CODIGO_INVALIDO: i32 = 1200;
    /// CUIT no autorizado CAEA
    pub const CUIT_NO_AUTORIZADO: i32 = 1201;
    /// PtoVta fuera de rango (1-99998)
    pub const PTO_VTA_RANGO: i32 = 1206;
    /// CAEA formato inválido
    pub const FORMATO_INVALIDO: i32 = 1207;
    /// PtoVta no habilitado CAEA
    pub const PTO_VTA_NO_HABILITADO: i32 = 1209;
}

/// CAEA puntos venta (1300s)
pub mod caea_pto {
    /// PtoVta debe ser CAEA habilitado
    pub const PTO_VTA_NO_CAEA: i32 = 1300;
}

/// Validaciones generales (1400s)
pub mod validaciones {
    /// DocNro rango inválido
    pub const DOC_NRO_RANGO: i32 = 1405;
    /// ImpTrib != suma tributos
    pub const IMP_TRIB_NO_COINCIDE: i32 = 1406;
    /// CbteHasta/CbteDesde inválido
    pub const CBTE_RANGO_INVALIDO: i32 = 1416;
    /// DocNro rango para tipo A
    pub const DOC_NRO_RANGO_TIPO_A: i32 = 1421;
    /// DocNro No Categorizado requiere tributo
    pub const NO_CATEGORIZADO_REQ_TRIBUTO: i32 = 1425;
    /// No habilitado tipo A en fecha
    pub const NO_HABILITADO_TIPO_A: i32 = 1429;
    /// Compradores validación
    pub const COMPRADORES_INVALIDO: i32 = 1432;
    /// ImpTotal != ImpNeto + ImpTrib (tipo C)
    pub const IMP_TOTAL_TIPO_C: i32 = 1439;
    /// CbteFchHsGen obligatorio contingencia
    pub const FCH_HS_GEN_OBLIGATORIO: i32 = 1441;
    /// CbteFchHsGen formato inválido
    pub const FCH_HS_GEN_FORMATO: i32 = 1442;
    /// FCE validación general
    pub const FCE_VALIDACION: i32 = 1451;
    /// FCE fecha asociado inválida
    pub const FCE_FECHA_ASOC: i32 = 1456;
    /// FCE cbte asociado inválido
    pub const FCE_CBTE_ASOC: i32 = 1457;
    /// FCE FchVtoPago obligatorio
    pub const FCE_VTO_PAGO: i32 = 1461;
    /// FCE opcionales inválidos
    pub const FCE_OPCIONALES: i32 = 1478;
    /// DocTipo validación
    pub const DOC_TIPO_VALIDACION: i32 = 1489;
    /// CAEA CbtesAsoc validación
    pub const CAEA_CBTES_ASOC: i32 = 1504;
    /// CAEA detalle request
    pub const CAEA_DETALLE_REQ: i32 = 1512;
    /// DocNro validación receptor
    pub const DOC_NRO_RECEPTOR: i32 = 1516;
    /// RG 4004-E Locación
    pub const RG_4004_LOCACION: i32 = 1801;
}

/// CAEA solicitud (15000s)
pub mod caea_sol {
    /// CUIT debe ser Autoimpresor
    pub const CUIT_NO_AUTOIMPRESOR: i32 = 15001;
    /// Periodo formato AAAAMM
    pub const PERIODO_FORMATO: i32 = 15004;
    /// Orden debe ser 1 o 2
    pub const ORDEN_INVALIDO: i32 = 15005;
    /// Validación CUIT
    pub const CUIT_VALIDACION: i32 = 15009;
}

/// FCE CBU/opcionales (2000s)
pub mod fce_opc {
    /// CBU debe ser 22 dígitos
    pub const CBU_FORMATO: i32 = 2101;
    /// CBU/Alias validación
    pub const CBU_ALIAS: i32 = 2102;
}

/// Código especial
pub mod especial {
    /// Valor Libertad (debug/test)
    pub const LIBERTAD: i32 = 3333;
}

/// Errores autorización/validación (10000s)
pub mod auth {
    /// Número/fecha de comprobante no corresponde con el próximo a autorizar
    pub const NUMERO_FECHA_NO_CORRESPONDE: i32 = 10016;
    /// Factura C no debe informar IVA
    pub const FACTURA_C_SIN_IVA: i32 = 10071;
    /// Opcionales Id=7 inválido
    pub const OPCIONAL_ID_7: i32 = 10094;
    /// Opcionales RG 3368 doc
    pub const RG_3368_DOC: i32 = 10099;
    /// Validación general
    pub const VALIDACION_GENERAL: i32 = 10101;
    /// Opcionales RG múltiple
    pub const OPCIONAL_RG_MULTIPLE: i32 = 10112;
    /// RG 3368 padrón AFIP
    pub const RG_3368_PADRON: i32 = 10115;
    /// CbteAsoc receptor
    pub const CBTE_ASOC_RECEPTOR: i32 = 10122;
    /// RG 4004-E locador
    pub const RG_4004_LOCADOR: i32 = 10128;
    /// Comprador DocTipo obligatorio
    pub const COMPRADOR_DOC_TIPO: i32 = 10135;
    /// Comprador tipos habilitados
    pub const COMPRADOR_TIPOS: i32 = 10137;
    /// Comprador DocNro obligatorio
    pub const COMPRADOR_DOC_NRO: i32 = 10138;
    /// Comprador porcentaje obligatorio
    pub const COMPRADOR_PORCENTAJE: i32 = 10142;
    /// Comprador porcentaje formato
    pub const COMPRADOR_PORCENTAJE_FMT: i32 = 10143;
    /// Comprador porcentaje suma 100%
    pub const COMPRADOR_SUMA_100: i32 = 10147;
    /// Comprador en padrón activo
    pub const COMPRADOR_PADRON: i32 = 10148;
    /// Comprador IVA activo
    pub const COMPRADOR_IVA: i32 = 10149;
    /// FCE NC/ND anulación
    pub const FCE_ANULACION: i32 = 10154;
    /// FCE fecha asociado obligatoria
    pub const FCE_FECHA_ASOC_OBL: i32 = 10158;
    /// FCE validación débito/crédito
    pub const FCE_DEBITO_CREDITO: i32 = 10159;
}

/// Errores validación adicionales (10175+)
pub mod validacion {
    /// Validación importes
    pub const IMPORTES: i32 = 10175;
    /// Fecha servicio
    pub const FECHA_SERVICIO: i32 = 10180;
    /// Concepto validación
    pub const CONCEPTO: i32 = 10183;
    /// IVA validación
    pub const IVA: i32 = 10184;
    /// Tributos validación
    pub const TRIBUTOS: i32 = 10188;
    /// Opcionales validación
    pub const OPCIONALES: i32 = 10189;
    /// Remito electrónico
    pub const REMITO_ELECTRONICO: i32 = 10205;
    /// Actividad AFIP
    pub const ACTIVIDAD_AFIP: i32 = 10208;
    /// ImpTotal validación
    pub const IMP_TOTAL: i32 = 10212;
    /// MonCotiz validación
    pub const MON_COTIZ: i32 = 10219;
    /// CondicionIVAReceptor
    pub const CONDICION_IVA_RECEPTOR: i32 = 10222;
    /// Receptor validación
    pub const RECEPTOR: i32 = 10230;
    /// Periodo fiscal
    pub const PERIODO_FISCAL: i32 = 10234;
    /// CBU FCE
    pub const CBU_FCE: i32 = 10238;
    /// Alias CBU FCE
    pub const ALIAS_CBU_FCE: i32 = 10239;
    /// DocNro receptor
    pub const DOC_NRO_RECEPTOR: i32 = 10242;
}

/// Punto de venta (11000)
pub mod pto_vta {
    /// PtoVta no habilitado
    pub const NO_HABILITADO: i32 = 11000;
}

/// CAEA actividades (16000s)
pub mod caea_act {
    /// Actividad CAEA requerida
    pub const ACTIVIDAD_REQUERIDA: i32 = 16001;
}

// =============================================================================
// Funciones de ayuda
// =============================================================================

/// Devuelve descripción corta del error (máx ~40 chars)
pub fn descripcion(codigo: i32) -> &'static str {
    match codigo {
        // FCE MiPyMEs
        206 => "FCE A: FchVtoPago obligatorio",
        211 => "FCE B: FchVtoPago obligatorio",

        // Campos básicos
        700 => "CbteTipo inválido",
        701 => "PtoVta inválido",
        708 => "DocNro no en padrón AFIP",
        709 => "CAEA/PtoVta inválido",
        717 => "ImpTotConc inválido",
        724 => "Importes no coinciden",
        783 => "CbteFch formato yyyymmdd",
        788 => "DocTipo/DocNro inválido",

        // Comprobantes asociados
        801 => "CbteAsoc debe existir",
        803 => "Nro CbteAsoc inválido",
        809 => "CbteAsoc no encontrado",
        816 => "Excede límite Monotributo",
        819 => "DocNro receptor inválido",

        // Tributos
        908 => "Desc tributo inválida",
        995 => "Actividad remito requerida",

        // Opcionales
        1012 => "RG 3368: Doc titular pago",
        1105 => "Valor opcional inválido",

        // CAEA validación
        1200 => "CAEA código inválido",
        1201 => "CUIT no autorizado CAEA",
        1206 => "PtoVta fuera rango 1-99998",
        1207 => "CAEA formato inválido",
        1209 => "PtoVta no habilitado CAEA",

        // CAEA puntos venta
        1300 => "PtoVta no es CAEA",

        // Validaciones generales
        1405 => "DocNro fuera de rango",
        1406 => "ImpTrib != suma tributos",
        1416 => "CbteDesde/Hasta inválido",
        1421 => "DocNro rango tipo A",
        1425 => "No Categ. requiere tributo",
        1429 => "No habilitado tipo A",
        1432 => "Compradores inválido",
        1439 => "ImpTotal tipo C incorrecto",
        1441 => "FchHsGen oblig. contingencia",
        1442 => "FchHsGen formato inválido",
        1451 => "FCE validación",
        1456 => "FCE fecha asociado",
        1457 => "FCE cbte asociado",
        1461 => "FCE FchVtoPago obligatorio",
        1478 => "FCE opcionales inválidos",
        1489 => "DocTipo validación",
        1504 => "CAEA CbtesAsoc validación",
        1512 => "CAEA detalle request",
        1516 => "DocNro receptor validación",
        1801 => "RG 4004-E locación",

        // CAEA solicitud
        15001 => "CUIT no es Autoimpresor",
        15004 => "Periodo formato AAAAMM",
        15005 => "Orden debe ser 1 o 2",
        15009 => "CUIT validación CAEA",

        // FCE opcionales
        2101 => "CBU debe ser 22 dígitos",
        2102 => "CBU/Alias validación",

        // Especial
        3333 => "Valor Libertad (debug)",

        // Auth/validación 10000s
        10016 => "Número/fecha no corresponde con próximo a autorizar",
        10071 => "Factura C: IVA no debe informarse",
        10094 => "Opcional Id=7 inválido",
        10099 => "RG 3368 documento",
        10101 => "Validación general",
        10112 => "Opcional RG múltiple",
        10115 => "RG 3368 padrón AFIP",
        10122 => "CbteAsoc receptor",
        10128 => "RG 4004-E locador",
        10135 => "Comprador DocTipo oblig.",
        10137 => "Comprador tipos 80,86,87",
        10138 => "Comprador DocNro oblig.",
        10142 => "Comprador % obligatorio",
        10143 => "Comprador % formato",
        10147 => "Comprador % suma 100",
        10148 => "Comprador no en padrón",
        10149 => "Comprador IVA no activo",
        10154 => "FCE NC/ND anulación",
        10158 => "FCE fecha asoc. oblig.",
        10159 => "FCE débito/crédito",
        10175 => "Importes validación",
        10180 => "Fecha servicio",
        10183 => "Concepto validación",
        10184 => "IVA validación",
        10188 => "Tributos validación",
        10189 => "Opcionales validación",
        10205 => "Remito electrónico",
        10208 => "Actividad AFIP",
        10212 => "ImpTotal validación",
        10219 => "MonCotiz validación",
        10222 => "CondicionIVAReceptor",
        10230 => "Receptor validación",
        10234 => "Periodo fiscal",
        10238 => "CBU FCE inválido",
        10239 => "Alias CBU FCE",
        10242 => "DocNro receptor",

        // Punto de venta
        11000 => "PtoVta no habilitado",

        // CAEA actividades
        16001 => "Actividad CAEA requerida",

        _ => "Error desconocido",
    }
}

/// Devuelve true si el código es una observación (no bloquea CAE)
pub fn es_observacion(codigo: i32) -> bool {
    matches!(
        codigo,
        1300..=1399 | 1405..=1499 | 1500..=1599 | 2101..=2199
    )
}

/// Devuelve true si el código es de CAEA
pub fn es_caea(codigo: i32) -> bool {
    matches!(
        codigo,
        1200..=1209 | 1300..=1399 | 1500..=1599 | 15001..=15009 | 16001
    )
}

/// Devuelve true si el código es de FCE MiPyMEs
pub fn es_fce(codigo: i32) -> bool {
    matches!(
        codigo,
        206 | 211 | 1451 | 1456 | 1457 | 1461 | 1478 | 2101 | 2102 | 10154 | 10158 | 10159
            | 10238
            | 10239
    )
}

/// Devuelve true si el código es de compradores
pub fn es_compradores(codigo: i32) -> bool {
    matches!(
        codigo,
        1432 | 10135 | 10137 | 10138 | 10142 | 10143 | 10147 | 10148 | 10149
    )
}

/// Devuelve true si el código es de RG específica (3368, 4004-E, etc)
pub fn es_rg_especifica(codigo: i32) -> bool {
    matches!(
        codigo,
        1012 | 1801 | 10094 | 10099 | 10112 | 10115 | 10122..=10132
    )
}

/// Devuelve la categoría del error
pub fn categoria(codigo: i32) -> &'static str {
    match codigo {
        200..=299 => "FCE",
        700..=799 => "Campos",
        800..=899 => "CbteAsoc",
        900..=999 => "Tributos",
        1000..=1099 => "Opcionales",
        1100..=1199 => "Valores",
        1200..=1299 => "CAEA",
        1300..=1399 => "CAEA PtoVta",
        1400..=1499 => "Validación",
        1500..=1599 | 15001..=15009 => "CAEA Sol.",
        2000..=2199 => "FCE Opc.",
        3333 => "Debug",
        10000..=10999 => "Auth/Val",
        11000..=11999 => "PtoVta",
        16000..=16999 => "CAEA Act.",
        _ => "Otro",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descripcion_conocidos() {
        assert_eq!(descripcion(700), "CbteTipo inválido");
        assert_eq!(descripcion(801), "CbteAsoc debe existir");
        assert_eq!(descripcion(1200), "CAEA código inválido");
        assert_eq!(descripcion(10147), "Comprador % suma 100");
    }

    #[test]
    fn test_descripcion_desconocido() {
        assert_eq!(descripcion(99999), "Error desconocido");
    }

    #[test]
    fn test_es_fce() {
        assert!(es_fce(206));
        assert!(es_fce(211));
        assert!(es_fce(1461));
        assert!(!es_fce(700));
    }

    #[test]
    fn test_es_caea() {
        assert!(es_caea(1200));
        assert!(es_caea(15001));
        assert!(es_caea(16001));
        assert!(!es_caea(700));
    }

    #[test]
    fn test_categoria() {
        assert_eq!(categoria(700), "Campos");
        assert_eq!(categoria(1200), "CAEA");
        assert_eq!(categoria(10100), "Auth/Val");
    }
}
