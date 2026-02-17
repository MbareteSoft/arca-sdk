//! # ARCA SDK
//!
//! Biblioteca para integración con servicios web de ARCA (Argentina).
//!
//! ## Características principales
//!
//! - **Autenticación WSAA** - Login con certificados digitales
//! - **Facturación WSFEv1** - Emisión de facturas electrónicas
//! - **CAEA** - Modo contingencia (offline)
//! - **Manejo de errores** - Parseo detallado de respuestas AFIP
//!
//! ## Ejemplo básico
//!
//! ```rust,no_run
//! use arca::{WsFev1Service, FeCabReq, FeDetReq, cbte_tipos, transport::HttpClient, auth};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Cargar certificados
//!     let cert = std::fs::read("cert.pem")?;
//!     let key = std::fs::read("key.pem")?;
//!     let http = HttpClient::new()?;
//!
//!     // Autenticar
//!     let ltr = auth::build_login_ticket_request("wsfe");
//!     let cms = auth::sign_cms_base64(&ltr, &cert, &key)?;
//!     let ticket = auth::wsaa_login(&http, "https://wsaahomo.afip.gov.ar/ws/services/LoginCms", &cms).await?;
//!
//!     // Crear servicio WSFEv1
//!     let wsfev1 = WsFev1Service::new(
//!         "https://wswhomo.afip.gov.ar/wsfev1/service.asmx".to_string(),
//!         http,
//!     );
//!
//!     // Consultar último comprobante
//!     let ultimo = wsfev1.fe_comp_ultimo_autorizado(
//!         &ticket.token, &ticket.sign, 20123456789, 1, cbte_tipos::FACTURA_B
//!     ).await?;
//!
//!     println!("Último comprobante: {}", ultimo);
//!     Ok(())
//! }
//! ```
//!
//! ## Módulos
//!
//! - [`auth`] - Autenticación WSAA (login, firma CMS)
//! - [`services::wsfev1`] - Facturación electrónica
//! - [`transport`] - Cliente HTTP y SOAP
//! - [`error`] - Tipos de error

// Modular structure
pub mod auth;
pub mod client;
pub mod constants;
pub mod error;
pub mod ffi;
pub mod services;
pub mod transport;
pub mod xml;

// Re-exports for convenience
pub use client::{ArcaClient, ArcaClientBuilder, ArcaClientConfig};
pub use constants::endpoints::ArcaEnvironment;
pub use error::{ArcaError, Result};
pub use services::wsfev1::{
    // Response detail types
    ArcaErr, ArcaEvt, ArcaObs, ArcaResponseDetails,
    // Main types
    AlicIva, CaeaRegInformativoResult, CaeaResponse, CaeaSinMovimiento, Cotizacion, FeCabReq,
    FeCaeResponse, FeCaeResponseDetallada, FeCompConsultarResponse, FeDetReq, ParamTipo, PtoVenta,
    // AFIP constants
    alicuotas_iva, cbte_tipos, codigos_error, conceptos, condicion_iva, doc_tipos, errores,
    monedas,
};
pub use services::padron_a13::{
    Domicilio, DummyReturn as PadronDummy, IdPersonaListReturn, PadronA13Service, Persona,
    PersonaReturn,
};
pub use services::{build_auth_block, ArcaService, WsFev1Service};
pub use transport::HttpClient;
