/// SOAP envelope namespace (standard)
pub const SOAP_ENVELOPE_NS: &str = "http://schemas.xmlsoap.org/soap/envelope/";

/// WSAA (Web Service de Autenticación y Autorización) namespace
pub mod wsaa {
    pub const NAMESPACE: &str = "http://wsaa.view.sua.dvadac.desein.afip.gov";
    pub const PREFIX: &str = "ws";
}

/// WSFEv1 (Factura Electrónica v1) namespace
pub mod wsfev1 {
    pub const NAMESPACE: &str = "http://ar.gov.afip.dif.FEV1/";
    pub const PREFIX: &str = "ar";
}
