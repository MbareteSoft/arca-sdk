use crate::error::{ArcaError, Result};
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

/// Sign a message using CMS (PKCS#7) and return base64-encoded result.
/// Uses OpenSSL CLI directly for maximum compatibility with AFIP's WSAA.
///
/// # Arguments
/// * `message` - The XML LoginTicketRequest to sign
/// * `cert_pem` - Certificate in PEM format
/// * `key_pem` - Private key in PEM format
pub fn sign_cms_base64(message: &str, cert_pem: &[u8], key_pem: &[u8]) -> Result<String> {
    // Create temporary files for cert and key
    let mut cert_file = NamedTempFile::new()
        .map_err(|e| ArcaError::Io(format!("Failed to create temp file for cert: {}", e)))?;
    cert_file
        .write_all(cert_pem)
        .map_err(|e| ArcaError::Io(format!("Failed to write cert to temp file: {}", e)))?;

    let mut key_file = NamedTempFile::new()
        .map_err(|e| ArcaError::Io(format!("Failed to create temp file for key: {}", e)))?;
    key_file
        .write_all(key_pem)
        .map_err(|e| ArcaError::Io(format!("Failed to write key to temp file: {}", e)))?;

    let cert_path = cert_file
        .path()
        .to_str()
        .ok_or_else(|| ArcaError::Io("Cert temp file path contains invalid UTF-8".to_string()))?;
    let key_path = key_file
        .path()
        .to_str()
        .ok_or_else(|| ArcaError::Io("Key temp file path contains invalid UTF-8".to_string()))?;

    // Call openssl cms command directly - this produces the exact format AFIP expects
    let mut child = Command::new("openssl")
        .args([
            "cms",
            "-sign",
            "-signer",
            cert_path,
            "-inkey",
            key_path,
            "-nodetach",
            "-outform",
            "PEM",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ArcaError::SigningCli(format!("Failed to spawn openssl: {}", e)))?;

    // Write message to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(message.as_bytes())
            .map_err(|e| ArcaError::Io(format!("Failed to write to openssl stdin: {}", e)))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| ArcaError::SigningCli(format!("Failed to wait for openssl: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ArcaError::SigningCli(format!("CMS signing failed: {}", stderr)));
    }

    // Parse PEM output - extract base64 content between BEGIN/END markers
    let pem_output = String::from_utf8_lossy(&output.stdout);
    let base64_content: String = pem_output
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<Vec<_>>()
        .join("");

    Ok(base64_content)
}

/// Sign a message using CMS (PKCS#7) directly from a PFX/PKCS#12 file.
///
/// This function allows signing without converting the PFX to PEM first.
/// Uses OpenSSL's legacy provider to support older encryption algorithms
/// commonly used in AFIP certificates.
///
/// # Arguments
/// * `message` - The XML LoginTicketRequest to sign
/// * `pfx_data` - The PFX/PKCS#12 file contents
/// * `password` - Password for the PFX (use empty string if none)
///
/// # Example
/// ```rust,no_run
/// use arca::auth::sign_cms_base64_from_pfx;
///
/// let pfx = std::fs::read("certificado.pfx").unwrap();
/// let signed = sign_cms_base64_from_pfx("message", &pfx, "").unwrap();
/// ```
pub fn sign_cms_base64_from_pfx(message: &str, pfx_data: &[u8], password: &str) -> Result<String> {
    // Create temporary file for PFX
    let mut pfx_file = NamedTempFile::new()
        .map_err(|e| ArcaError::Io(format!("Failed to create temp file for PFX: {}", e)))?;
    pfx_file
        .write_all(pfx_data)
        .map_err(|e| ArcaError::Io(format!("Failed to write PFX to temp file: {}", e)))?;

    let pfx_path = pfx_file
        .path()
        .to_str()
        .ok_or_else(|| ArcaError::Io("PFX temp file path contains invalid UTF-8".to_string()))?;

    // Use OpenSSL with legacy provider for older PFX encryption algorithms (RC2-40-CBC)
    // Password passed via stdin to avoid exposure in process list (ps aux)
    let mut child = Command::new("openssl")
        .args([
            "cms",
            "-sign",
            "-signer",
            pfx_path,
            "-inkey",
            pfx_path,
            "-passin",
            "stdin",
            "-nodetach",
            "-outform",
            "PEM",
            "-provider",
            "legacy",
            "-provider",
            "default",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ArcaError::SigningCli(format!("Failed to spawn openssl: {}", e)))?;

    // Write password followed by message to stdin
    // OpenSSL expects password on first line when using -passin stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(format!("{}\n", password).as_bytes())
            .map_err(|e| ArcaError::Io(format!("Failed to write password to openssl stdin: {}", e)))?;
        stdin
            .write_all(message.as_bytes())
            .map_err(|e| ArcaError::Io(format!("Failed to write to openssl stdin: {}", e)))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| ArcaError::SigningCli(format!("Failed to wait for openssl: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ArcaError::SigningCli(format!("CMS signing failed: {}", stderr)));
    }

    // Parse PEM output - extract base64 content between BEGIN/END markers
    let pem_output = String::from_utf8_lossy(&output.stdout);
    let base64_content: String = pem_output
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<Vec<_>>()
        .join("");

    Ok(base64_content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use openssl::asn1::Asn1Time;
    use openssl::hash::MessageDigest;
    use openssl::pkey::PKey;
    use openssl::rsa::Rsa;
    use openssl::x509::{X509NameBuilder, X509};

    fn generate_dummy_cert() -> (Vec<u8>, Vec<u8>) {
        let rsa = Rsa::generate(2048).unwrap();
        let pkey = PKey::from_rsa(rsa).unwrap();

        let mut name = X509NameBuilder::new().unwrap();
        name.append_entry_by_text("CN", "dummy").unwrap();
        let name = name.build();

        let mut builder = X509::builder().unwrap();
        builder.set_version(2).unwrap();
        builder.set_subject_name(&name).unwrap();
        builder.set_issuer_name(&name).unwrap();
        builder.set_pubkey(&pkey).unwrap();
        builder
            .set_not_before(&Asn1Time::days_from_now(0).unwrap())
            .unwrap();
        builder
            .set_not_after(&Asn1Time::days_from_now(365).unwrap())
            .unwrap();
        builder.sign(&pkey, MessageDigest::sha256()).unwrap();
        let cert = builder.build();

        (
            cert.to_pem().unwrap(),
            pkey.private_key_to_pem_pkcs8().unwrap(),
        )
    }

    #[test]
    fn test_sign_cms_base64() {
        let (cert, key) = generate_dummy_cert();
        let message = "test message to sign";
        let result = sign_cms_base64(message, &cert, &key);
        assert!(result.is_ok());
        let signed = result.unwrap();
        assert!(!signed.is_empty());
    }

    #[test]
    fn test_sign_cms_base64_with_invalid_cert() {
        let (_, key) = generate_dummy_cert();
        let result = sign_cms_base64("test", b"invalid cert", &key);
        assert!(result.is_err());
    }
}
