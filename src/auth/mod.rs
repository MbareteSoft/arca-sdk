//! Autenticación WSAA (Web Service de Autenticación y Autorización)
//!
//! Este módulo maneja la autenticación con ARCA usando certificados digitales.
//!
//! ## Flujo de autenticación
//!
//! 1. Construir LoginTicketRequest XML con [`build_login_ticket_request`]
//! 2. Firmar con CMS/PKCS#7 usando [`sign_cms_base64`]
//! 3. Enviar a WSAA con [`wsaa_login`]
//! 4. Obtener [`TicketAcceso`] con token y sign
//!
//! ## Ejemplo
//!
//! ```rust,no_run
//! use arca::{auth, transport::HttpClient};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let cert = std::fs::read("cert.pem")?;
//! let key = std::fs::read("key.pem")?;
//! let http = HttpClient::new()?;
//!
//! // 1. Construir request
//! let ltr = auth::build_login_ticket_request("wsfe");
//!
//! // 2. Firmar
//! let cms = auth::sign_cms_base64(&ltr, &cert, &key)?;
//!
//! // 3. Login
//! let ticket = auth::wsaa_login(
//!     &http,
//!     "https://wsaahomo.afip.gov.ar/ws/services/LoginCms",
//!     &cms
//! ).await?;
//!
//! // 4. Usar token y sign
//! println!("Token válido hasta: {}", ticket.expiration_time);
//! # Ok(())
//! # }
//! ```
//!
//! ## Endpoints
//!
//! - Homologación: `https://wsaahomo.afip.gov.ar/ws/services/LoginCms`
//! - Producción: `https://wsaa.afip.gov.ar/ws/services/LoginCms`

pub mod cms;
pub mod ticket;
pub mod wsaa;

pub use cms::{sign_cms_base64, sign_cms_base64_from_pfx};
pub use ticket::TicketAcceso;
pub use wsaa::{build_login_ticket_request, wsaa_login};
