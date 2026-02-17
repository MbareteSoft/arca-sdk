// ws_sr_padron_a13 - Consulta a Padrón Alcance 13
//
// Este servicio permite consultar datos de contribuyentes AFIP.
// Requiere certificado digital y autorización previa.

mod requests;
mod types;

pub use types::*;

use crate::error::ArcaError;
use crate::transport::HttpClient;
use crate::xml;

/// Endpoints para ws_sr_padron_a13
pub mod endpoints {
    /// Testing (homologación)
    pub const TESTING: &str = "https://awshomo.afip.gov.ar/sr-padron/webservices/personaServiceA13";
    /// Producción
    pub const PRODUCTION: &str = "https://aws.afip.gov.ar/sr-padron/webservices/personaServiceA13";
}

/// Servicio de consulta al Padrón Alcance 13
pub struct PadronA13Service {
    endpoint: String,
}

impl PadronA13Service {
    /// Crear servicio para testing
    pub fn testing() -> Self {
        Self {
            endpoint: endpoints::TESTING.to_string(),
        }
    }

    /// Crear servicio para producción
    pub fn production() -> Self {
        Self {
            endpoint: endpoints::PRODUCTION.to_string(),
        }
    }

    /// Crear servicio con endpoint personalizado
    pub fn with_endpoint(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
        }
    }

    /// Health check del servicio
    pub async fn dummy(&self, http: &HttpClient) -> Result<DummyReturn, ArcaError> {
        let req = requests::build_dummy_req();
        let resp = crate::transport::post_soap(http, &self.endpoint, &req, None).await?;

        Ok(DummyReturn {
            app_server: xml::extract_tag_text(&resp, "appserver").unwrap_or_default(),
            auth_server: xml::extract_tag_text(&resp, "authserver").unwrap_or_default(),
            db_server: xml::extract_tag_text(&resp, "dbserver").unwrap_or_default(),
        })
    }

    /// Consultar datos de una persona por CUIT
    pub async fn get_persona(
        &self,
        http: &HttpClient,
        token: &str,
        sign: &str,
        cuit_representada: u64,
        id_persona: u64,
    ) -> Result<PersonaReturn, ArcaError> {
        let req = requests::build_get_persona_req(token, sign, cuit_representada, id_persona);
        let resp = crate::transport::post_soap(http, &self.endpoint, &req, None).await?;

        // Check for errors
        if let Ok(fault) = xml::extract_tag_text(&resp, "faultstring") {
            return Err(ArcaError::Service {
                code: "SOAP_FAULT".to_string(),
                message: fault,
            });
        }

        // Parse persona
        let persona = parse_persona(&resp)?;

        Ok(PersonaReturn {
            persona,
            fecha_hora: xml::extract_tag_text(&resp, "fechaHora").unwrap_or_default(),
            servidor: xml::extract_tag_text(&resp, "servidor").unwrap_or_default(),
        })
    }

    /// Buscar CUITs por tipo y número de documento
    pub async fn get_id_persona_list_by_documento(
        &self,
        http: &HttpClient,
        token: &str,
        sign: &str,
        cuit_representada: u64,
        tipo_documento: &str,
        numero_documento: &str,
    ) -> Result<IdPersonaListReturn, ArcaError> {
        let req = requests::build_get_id_persona_list_req(
            token,
            sign,
            cuit_representada,
            tipo_documento,
            numero_documento,
        );
        let resp = crate::transport::post_soap(http, &self.endpoint, &req, None).await?;

        // Check for errors
        if let Ok(fault) = xml::extract_tag_text(&resp, "faultstring") {
            return Err(ArcaError::Service {
                code: "SOAP_FAULT".to_string(),
                message: fault,
            });
        }

        // Parse id list
        let id_personas = parse_id_persona_list(&resp);

        Ok(IdPersonaListReturn {
            id_personas,
            fecha_hora: xml::extract_tag_text(&resp, "fechaHora").unwrap_or_default(),
            servidor: xml::extract_tag_text(&resp, "servidor").unwrap_or_default(),
        })
    }
}

/// Parse persona from SOAP response
fn parse_persona(resp: &str) -> Result<Persona, ArcaError> {
    // Find persona block
    let persona_start = resp.find("<persona>").or_else(|| resp.find(":persona>"));
    if persona_start.is_none() {
        return Err(ArcaError::Xml("No persona block found in response".to_string()));
    }

    let mut persona = Persona::default();

    persona.id_persona = xml::extract_tag_text(resp, "idPersona")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    persona.tipo_persona = xml::extract_tag_text(resp, "tipoPersona").ok();
    persona.tipo_clave = xml::extract_tag_text(resp, "tipoClave").ok();
    persona.estado_clave = xml::extract_tag_text(resp, "estadoClave").ok();
    persona.nombre = xml::extract_tag_text(resp, "nombre").ok();
    persona.apellido = xml::extract_tag_text(resp, "apellido").ok();
    persona.razon_social = xml::extract_tag_text(resp, "razonSocial").ok();
    persona.tipo_documento = xml::extract_tag_text(resp, "tipoDocumento").ok();
    persona.numero_documento = xml::extract_tag_text(resp, "numeroDocumento").ok();
    persona.fecha_nacimiento = xml::extract_tag_text(resp, "fechaNacimiento").ok();
    persona.fecha_fallecimiento = xml::extract_tag_text(resp, "fechaFallecimiento").ok();
    persona.fecha_contrato_social = xml::extract_tag_text(resp, "fechaContratoSocial").ok();
    persona.forma_juridica = xml::extract_tag_text(resp, "formaJuridica").ok();
    persona.id_actividad_principal = xml::extract_tag_text(resp, "idActividadPrincipal").ok();
    persona.descripcion_actividad_principal = xml::extract_tag_text(resp, "descripcionActividadPrincipal").ok();
    persona.periodo_actividad_principal = xml::extract_tag_text(resp, "periodoActividadPrincipal").ok();
    persona.mes_cierre = xml::extract_tag_text(resp, "mesCierre")
        .ok()
        .and_then(|s| s.parse().ok());

    // Parse domicilios
    persona.domicilios = parse_domicilios(resp);

    // Parse claves inactivas
    persona.claves_inactivas = parse_claves_inactivas(resp);

    Ok(persona)
}

/// Parse domicilios from response using unified XML helpers
fn parse_domicilios(resp: &str) -> Vec<Domicilio> {
    xml::extract_all_blocks(resp, "domicilio")
        .into_iter()
        .map(|block| {
            Domicilio {
                tipo_domicilio: xml::extract_inline_text(&block, "tipoDomicilio"),
                calle: xml::extract_inline_text(&block, "calle"),
                numero: xml::extract_inline_text(&block, "numero"),
                piso: xml::extract_inline_text(&block, "piso"),
                oficina_dpto_local: xml::extract_inline_text(&block, "oficinaDptoLocal"),
                manzana: xml::extract_inline_text(&block, "manzana"),
                sector: xml::extract_inline_text(&block, "sector"),
                torre: xml::extract_inline_text(&block, "torre"),
                dato_adicional: xml::extract_inline_text(&block, "datoAdicional"),
                tipo_dato_adicional: xml::extract_inline_text(&block, "tipoDatoAdicional"),
                codigo_postal: xml::extract_inline_text(&block, "codigoPostal"),
                localidad: xml::extract_inline_text(&block, "localidad"),
                descripcion_provincia: xml::extract_inline_text(&block, "descripcionProvincia"),
                id_provincia: xml::parse_inline(&block, "idProvincia"),
                direccion: xml::extract_inline_text(&block, "direccion"),
                estado_domicilio: xml::extract_inline_text(&block, "estadoDomicilio"),
            }
        })
        .collect()
}

/// Parse claves inactivas from response using unified XML helpers
fn parse_claves_inactivas(resp: &str) -> Vec<u64> {
    xml::extract_all_blocks(resp, "claveInactivaAsociada")
        .into_iter()
        .filter_map(|block| block.trim().parse().ok())
        .collect()
}

/// Parse list of idPersona from response using unified XML helpers
fn parse_id_persona_list(resp: &str) -> Vec<u64> {
    xml::extract_all_blocks(resp, "idPersona")
        .into_iter()
        .filter_map(|block| block.trim().parse().ok())
        .collect()
}
