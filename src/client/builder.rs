use super::config::ArcaClientConfig;
use super::ArcaClient;
use super::token_store::TokenStore;
use crate::constants::endpoints::ArcaEnvironment;
use crate::error::{ArcaError, Result};
use std::time::Duration;
use zeroize::Zeroize;

/// Builder for creating ArcaClient instances
///
/// Implements Drop to ensure sensitive credentials (PEM keys, PFX data,
/// and PFX password) are zeroed from memory when the builder is dropped.
pub struct ArcaClientBuilder {
    config: ArcaClientConfig,
    cert_pem: Option<Vec<u8>>,
    key_pem: Option<Vec<u8>>,
    pfx_data: Option<Vec<u8>>,
    pfx_password: Option<String>,
    token_store: Option<Box<dyn TokenStore>>,
}

impl Drop for ArcaClientBuilder {
    fn drop(&mut self) {
        if let Some(ref mut cert) = self.cert_pem {
            cert.zeroize();
        }
        if let Some(ref mut key) = self.key_pem {
            key.zeroize();
        }
        if let Some(ref mut pfx) = self.pfx_data {
            pfx.zeroize();
        }
        if let Some(ref mut password) = self.pfx_password {
            password.zeroize();
        }
    }
}

impl ArcaClientBuilder {
    pub fn new() -> Self {
        Self {
            config: ArcaClientConfig::default(),
            cert_pem: None,
            key_pem: None,
            pfx_data: None,
            pfx_password: None,
            token_store: None,
        }
    }

    /// Set the ARCA environment (Testing or Production)
    pub fn environment(mut self, env: ArcaEnvironment) -> Self {
        self.config.environment = env;
        self
    }

    /// Set the certificate and private key for authentication (PEM format)
    pub fn credentials(mut self, cert_pem: Vec<u8>, key_pem: Vec<u8>) -> Self {
        self.cert_pem = Some(cert_pem);
        self.key_pem = Some(key_pem);
        self
    }

    /// Set the certificate from a PFX/PKCS#12 file
    ///
    /// The PFX will be converted to PEM format internally.
    /// Use an empty string for password if the PFX is not encrypted.
    pub fn credentials_pfx(mut self, pfx_data: Vec<u8>, password: &str) -> Self {
        self.pfx_data = Some(pfx_data);
        self.pfx_password = Some(password.to_string());
        self
    }

    /// Set a persistent token store for caching access tickets
    pub fn token_store<T: TokenStore + 'static>(mut self, store: T) -> Self {
        self.token_store = Some(Box::new(store));
        self
    }

    /// Set the HTTP request timeout
    pub fn http_timeout(mut self, timeout: Duration) -> Self {
        self.config.http_timeout = timeout;
        self
    }

    /// Set the HTTP connection timeout
    pub fn http_connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.http_connect_timeout = timeout;
        self
    }

    /// Set the token expiration margin in minutes
    pub fn token_margin_minutes(mut self, minutes: i64) -> Self {
        self.config.token_margin_minutes = minutes;
        self
    }

    /// Build the ArcaClient
    pub fn build(mut self) -> Result<ArcaClient> {
        // If PFX is provided, convert to PEM
        // Use .take() to move values out while leaving None (which will be zeroized in drop)
        let (cert_pem, key_pem) = if let (Some(pfx_data), Some(password)) = (self.pfx_data.take(), self.pfx_password.take()) {
            convert_pfx_to_pem(&pfx_data, &password)?
        } else {
            (self.cert_pem.take().unwrap_or_default(), self.key_pem.take().unwrap_or_default())
        };

        let config = std::mem::take(&mut self.config);
        let token_store = self.token_store.take();
        
        ArcaClient::from_config(config, cert_pem, key_pem, token_store)
    }
}

/// Convert PFX/PKCS#12 to PEM format
///
/// Uses OpenSSL CLI with legacy provider to support older encryption algorithms
/// like RC2-40-CBC commonly used in AFIP certificates.
/// Password is passed via stdin to avoid exposure in process list (ps aux).
fn convert_pfx_to_pem(pfx_data: &[u8], password: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use tempfile::NamedTempFile;

    // Create temp file for PFX
    let mut pfx_file = NamedTempFile::new()
        .map_err(|e| ArcaError::Config(format!("Failed to create temp file: {}", e)))?;
    pfx_file.write_all(pfx_data)
        .map_err(|e| ArcaError::Config(format!("Failed to write PFX: {}", e)))?;

    let pfx_path = pfx_file
        .path()
        .to_str()
        .ok_or_else(|| ArcaError::Config("PFX temp file path contains invalid UTF-8".to_string()))?;

    // Extract certificate using OpenSSL CLI with legacy provider
    // Password passed via stdin to avoid exposure in process list
    let mut cert_child = Command::new("openssl")
        .args([
            "pkcs12",
            "-in", pfx_path,
            "-clcerts",
            "-nokeys",
            "-passin", "stdin",
            "-provider", "legacy",
            "-provider", "default",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ArcaError::Config(format!("Failed to spawn openssl: {}", e)))?;

    if let Some(mut stdin) = cert_child.stdin.take() {
        // Add newline to password as openssl expects it
        let pwd = format!("{}\n", password);
        stdin.write_all(pwd.as_bytes())
            .map_err(|e| ArcaError::Config(format!("Failed to write password: {}", e)))?;
    }

    let cert_output = cert_child.wait_with_output()
        .map_err(|e| ArcaError::Config(format!("Failed to wait for openssl: {}", e)))?;

    if !cert_output.status.success() {
        let stderr = String::from_utf8_lossy(&cert_output.stderr);
        return Err(ArcaError::Config(format!("Failed to extract certificate: {}", stderr)));
    }

    // Extract private key using OpenSSL CLI with legacy provider
    // Password passed via stdin to avoid exposure in process list
    let mut key_child = Command::new("openssl")
        .args([
            "pkcs12",
            "-in", pfx_path,
            "-nocerts",
            "-nodes",
            "-passin", "stdin",
            "-provider", "legacy",
            "-provider", "default",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ArcaError::Config(format!("Failed to spawn openssl: {}", e)))?;

    if let Some(mut stdin) = key_child.stdin.take() {
        // Add newline to password as openssl expects it
        let pwd = format!("{}\n", password);
        stdin.write_all(pwd.as_bytes())
            .map_err(|e| ArcaError::Config(format!("Failed to write password: {}", e)))?;
    }

    let key_output = key_child.wait_with_output()
        .map_err(|e| ArcaError::Config(format!("Failed to wait for openssl: {}", e)))?;

    if !key_output.status.success() {
        let stderr = String::from_utf8_lossy(&key_output.stderr);
        return Err(ArcaError::Config(format!("Failed to extract private key: {}", stderr)));
    }

    Ok((cert_output.stdout, key_output.stdout))
}

impl Default for ArcaClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}
