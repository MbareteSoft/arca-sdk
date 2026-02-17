//! Tipos de error para la biblioteca ARCA SDK
//!
//! Este módulo define los errores que pueden ocurrir durante la integración con ARCA/AFIP.
//!
//! ## Tipos de error
//!
//! | Variante | Descripción |
//! |----------|-------------|
//! | [`ArcaError::Auth`] | Error de autenticación WSAA |
//! | [`ArcaError::Transport`] | Error de red/HTTP |
//! | [`ArcaError::Xml`] | Error parseando respuesta XML |
//! | [`ArcaError::Signing`] | Error firmando con OpenSSL |
//! | [`ArcaError::Service`] | Error devuelto por servicio AFIP |
//! | [`ArcaError::NoToken`] | Token expirado o no disponible |
//!
//! ## Ejemplo
//!
//! ```rust
//! use arca::{ArcaError, Result};
//!
//! fn process_invoice() -> Result<()> {
//!     // Si falta el token
//!     if false {
//!         return Err(ArcaError::NoToken);
//!     }
//!     Ok(())
//! }
//!
//! match process_invoice() {
//!     Ok(_) => println!("Procesado"),
//!     Err(ArcaError::NoToken) => println!("Debe autenticarse primero"),
//!     Err(e) => println!("Error: {}", e),
//! }
//! ```

use thiserror::Error;

/// Tipos de error para operaciones AFIP
#[derive(Error, Debug)]
pub enum ArcaError {
    /// Error de autenticación WSAA
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Error de red/HTTP
    #[error("HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),

    /// Error parseando respuesta XML
    #[error("XML parsing error: {0}")]
    Xml(String),

    /// Error de la librería OpenSSL
    #[error("CMS signing error: {0}")]
    Signing(#[from] openssl::error::ErrorStack),

    /// Error ejecutando OpenSSL CLI (usado para firma CMS)
    #[error("OpenSSL CLI error: {0}")]
    SigningCli(String),

    /// Error devuelto por servicio AFIP
    #[error("Service error: {code} - {message}")]
    Service { code: String, message: String },

    /// Error de configuración
    #[error("Configuration error: {0}")]
    Config(String),

    /// Token expirado o no disponible
    #[error("Token expired or not available")]
    NoToken,

    /// Ambiente inválido
    #[error("Invalid environment: {0}")]
    InvalidEnvironment(String),

    /// Error de I/O (archivos temporales, etc.)
    #[error("I/O error: {0}")]
    Io(String),
}

pub type Result<T> = std::result::Result<T, ArcaError>;
