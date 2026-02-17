// ws_sr_padron_a13 - Padrón Alcance 13
// Consulta de datos de contribuyentes AFIP

/// Domicilio del contribuyente
#[derive(Debug, Clone, Default)]
pub struct Domicilio {
    pub tipo_domicilio: Option<String>,
    pub calle: Option<String>,
    pub numero: Option<String>,
    pub piso: Option<String>,
    pub oficina_dpto_local: Option<String>,
    pub manzana: Option<String>,
    pub sector: Option<String>,
    pub torre: Option<String>,
    pub dato_adicional: Option<String>,
    pub tipo_dato_adicional: Option<String>,
    pub codigo_postal: Option<String>,
    pub localidad: Option<String>,
    pub descripcion_provincia: Option<String>,
    pub id_provincia: Option<i32>,
    pub direccion: Option<String>,
    pub estado_domicilio: Option<String>,
}

/// Datos de una persona (física o jurídica)
#[derive(Debug, Clone, Default)]
pub struct Persona {
    /// CUIT/CUIL de la persona
    pub id_persona: u64,
    /// Tipo de persona: "FISICA" o "JURIDICA"
    pub tipo_persona: Option<String>,
    /// Tipo de clave: "CUIT", "CUIL", "CDI"
    pub tipo_clave: Option<String>,
    /// Estado de la clave: "ACTIVO", "INACTIVO"
    pub estado_clave: Option<String>,
    /// Nombre (persona física)
    pub nombre: Option<String>,
    /// Apellido (persona física)
    pub apellido: Option<String>,
    /// Razón social (persona jurídica)
    pub razon_social: Option<String>,
    /// Tipo de documento
    pub tipo_documento: Option<String>,
    /// Número de documento
    pub numero_documento: Option<String>,
    /// Fecha de nacimiento (persona física) YYYY-MM-DD
    pub fecha_nacimiento: Option<String>,
    /// Fecha de fallecimiento YYYY-MM-DD
    pub fecha_fallecimiento: Option<String>,
    /// Fecha de contrato social (persona jurídica) YYYY-MM-DD
    pub fecha_contrato_social: Option<String>,
    /// Forma jurídica (SRL, SA, etc)
    pub forma_juridica: Option<String>,
    /// ID de actividad principal
    pub id_actividad_principal: Option<String>,
    /// Descripción de actividad principal
    pub descripcion_actividad_principal: Option<String>,
    /// Período de actividad principal
    pub periodo_actividad_principal: Option<String>,
    /// Mes de cierre fiscal
    pub mes_cierre: Option<i32>,
    /// Domicilios
    pub domicilios: Vec<Domicilio>,
    /// Claves inactivas asociadas
    pub claves_inactivas: Vec<u64>,
}

impl Persona {
    /// Devuelve el nombre completo (nombre + apellido para física, razón social para jurídica)
    pub fn nombre_completo(&self) -> String {
        if let Some(ref razon) = self.razon_social {
            if !razon.is_empty() {
                return razon.clone();
            }
        }

        let nombre = self.nombre.as_deref().unwrap_or("");
        let apellido = self.apellido.as_deref().unwrap_or("");

        if !nombre.is_empty() && !apellido.is_empty() {
            format!("{} {}", apellido, nombre)
        } else if !apellido.is_empty() {
            apellido.to_string()
        } else {
            nombre.to_string()
        }
    }

    /// Devuelve true si la clave está activa
    pub fn esta_activo(&self) -> bool {
        self.estado_clave.as_deref() == Some("ACTIVO")
    }

    /// Devuelve true si es persona física
    pub fn es_persona_fisica(&self) -> bool {
        self.tipo_persona.as_deref() == Some("FISICA")
    }

    /// Devuelve true si es persona jurídica
    pub fn es_persona_juridica(&self) -> bool {
        self.tipo_persona.as_deref() == Some("JURIDICA")
    }

    /// Devuelve el domicilio fiscal (tipo "FISCAL" o el primero disponible)
    pub fn domicilio_fiscal(&self) -> Option<&Domicilio> {
        self.domicilios
            .iter()
            .find(|d| d.tipo_domicilio.as_deref() == Some("FISCAL"))
            .or_else(|| self.domicilios.first())
    }
}

/// Respuesta del método getPersona
#[derive(Debug, Clone)]
pub struct PersonaReturn {
    pub persona: Persona,
    pub fecha_hora: String,
    pub servidor: String,
}

/// Respuesta del método getIdPersonaListByDocumento
#[derive(Debug, Clone)]
pub struct IdPersonaListReturn {
    pub id_personas: Vec<u64>,
    pub fecha_hora: String,
    pub servidor: String,
}

/// Respuesta del método dummy
#[derive(Debug, Clone)]
pub struct DummyReturn {
    pub app_server: String,
    pub auth_server: String,
    pub db_server: String,
}
