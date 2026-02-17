//! Transporte HTTP y SOAP
//!
//! Este módulo proporciona la infraestructura de comunicación con los servicios ARCA.
//!
//! ## Componentes
//!
//! - [`HttpClient`] - Cliente HTTP thread-safe con connection pooling
//! - [`create_soap_envelope`] - Constructor de envelopes SOAP
//! - [`post_soap`] - Envío de requests SOAP
//!
//! ## Ejemplo
//!
//! ```rust,no_run
//! use arca::transport::{HttpClient, create_soap_envelope, post_soap};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let http = HttpClient::new()?;
//!
//! let envelope = create_soap_envelope(
//!     "http://ar.gov.afip.dif.FEV1/",
//!     "ar",
//!     "FEDummy",
//!     "",
//! );
//!
//! let response = post_soap(
//!     &http,
//!     "https://wswhomo.afip.gov.ar/wsfev1/service.asmx",
//!     &envelope,
//!     Some("http://ar.gov.afip.dif.FEV1/FEDummy"),
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Manejo de errores SOAP
//!
//! AFIP retorna HTTP 500 para SOAP faults. El módulo maneja esto
//! automáticamente, extrayendo el mensaje de error del fault.

pub mod http;
pub mod soap;

pub use http::HttpClient;
pub use soap::{build_soap_action, create_soap_envelope, post_soap};
