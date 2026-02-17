#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArcaEnvironment {
    Testing,
    Production,
}

impl ArcaEnvironment {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "test" | "testing" | "homo" | "homologacion" => Some(Self::Testing),
            "prod" | "production" | "produccion" => Some(Self::Production),
            _ => None,
        }
    }
}

pub struct Endpoints {
    pub wsaa_login_cms: &'static str,
    pub wsfev1: &'static str,
    pub padron_a13: &'static str,
}

/// Get endpoints for the specified environment
pub fn get(env: ArcaEnvironment) -> Endpoints {
    match env {
        ArcaEnvironment::Testing => Endpoints {
            wsaa_login_cms: "https://wsaahomo.afip.gov.ar/ws/services/LoginCms",
            wsfev1: "https://wswhomo.afip.gov.ar/wsfev1/service.asmx",
            padron_a13: "https://awshomo.afip.gov.ar/sr-padron/webservices/personaServiceA13",
        },
        ArcaEnvironment::Production => Endpoints {
            wsaa_login_cms: "https://wsaa.afip.gov.ar/ws/services/LoginCms",
            wsfev1: "https://servicios1.afip.gov.ar/wsfev1/service.asmx",
            padron_a13: "https://aws.afip.gov.ar/sr-padron/webservices/personaServiceA13",
        },
    }
}
