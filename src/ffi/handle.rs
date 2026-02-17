//! ARCA Client Handle
//!
//! Opaque handle structure for FFI consumers.

use crate::client::token_store::FileSystemTokenStore;
use crate::services::{PadronA13Service, WsFev1Service};
use crate::transport::HttpClient;
use zeroize::Zeroize;

/// Opaque handle to the ARCA client.
///
/// Implements Drop to ensure sensitive credentials (certificate, private key,
/// token, and sign) are zeroized from memory when the handle is freed.
///
/// Services are cached to avoid URL string allocations on every FFI call.
pub struct ArcaHandle {
    pub(crate) http: HttpClient,
    pub(crate) wsaa_url: &'static str,
    pub(crate) cert_pem: Vec<u8>,
    pub(crate) key_pem: Vec<u8>,
    pub(crate) token: Option<String>,
    pub(crate) sign: Option<String>,
    pub(crate) expiration_time: Option<String>,
    // Cached services - allocated once at client creation
    pub(crate) wsfev1: WsFev1Service,
    pub(crate) padron: PadronA13Service,
    // Token persistence (None = disabled, Some = enabled via arca_set_token_cache)
    pub(crate) token_store: Option<FileSystemTokenStore>,
}

impl Drop for ArcaHandle {
    fn drop(&mut self) {
        // Zeroize all sensitive credential data
        self.cert_pem.zeroize();
        self.key_pem.zeroize();
        if let Some(ref mut token) = self.token {
            token.zeroize();
        }
        if let Some(ref mut sign) = self.sign {
            sign.zeroize();
        }
    }
}
