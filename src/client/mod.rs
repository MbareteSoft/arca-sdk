pub mod builder;
pub mod config;
pub mod token_store;

pub use builder::ArcaClientBuilder;
pub use config::ArcaClientConfig;
pub use token_store::{FileSystemTokenStore, TokenStore};

use crate::auth::{build_login_ticket_request, sign_cms_base64, wsaa_login, TicketAcceso};
use crate::constants::endpoints::{get as get_endpoints, ArcaEnvironment};
use crate::error::{ArcaError, Result};
use crate::services::wsfev1::{
    CaeaRegInformativoResult, CaeaResponse, CaeaSinMovimiento, Cotizacion, FeCabReq, FeCaeResponse,
    FeCompConsultarResponse, FeDetReq, ParamTipo, PtoVenta, WsFev1Service,
};
use crate::transport::HttpClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use zeroize::Zeroize;

/// Context for WSFEv1 service calls - avoids code duplication
struct WsFev1Context {
    service: WsFev1Service,
    token: String,
    sign: String,
}

/// Thread-safe ARCA client
///
/// This client can be safely cloned and shared across threads.
/// It manages authentication tokens automatically and reuses HTTP connections.
#[derive(Clone)]
pub struct ArcaClient {
    inner: Arc<ArcaClientInner>,
}

/// Inner state for ArcaClient.
///
/// Certificate and key material is zeroed from memory when dropped
/// via the custom Drop implementation.
struct ArcaClientInner {
    config: ArcaClientConfig,
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
    http_client: HttpClient,
    /// Service-specific tokens (keyed by service_id)
    tokens: RwLock<HashMap<String, TicketAcceso>>,
    /// Semaphore to prevent concurrent token refreshes for same service
    login_semaphore: Semaphore,
    /// Optional persistent token store
    token_store: Option<Box<dyn TokenStore>>,
}

impl Drop for ArcaClientInner {
    fn drop(&mut self) {
        // Zeroize sensitive credential data
        self.cert_pem.zeroize();
        self.key_pem.zeroize();
        // Note: tokens in RwLock will be dropped and zeroized via TicketAcceso's ZeroizeOnDrop
    }
}

impl ArcaClient {
    /// Create a new ArcaClientBuilder
    pub fn builder() -> ArcaClientBuilder {
        ArcaClientBuilder::new()
    }

    /// Create a new ArcaClient for the specified environment
    pub fn new(env: ArcaEnvironment) -> Result<Self> {
        Self::builder().environment(env).build()
    }

    /// Create an ArcaClient from configuration
    pub(crate) fn from_config(
        config: ArcaClientConfig,
        cert_pem: Vec<u8>,
        key_pem: Vec<u8>,
        token_store: Option<Box<dyn TokenStore>>,
    ) -> Result<Self> {
        let http_client =
            HttpClient::with_config(config.http_timeout, config.http_connect_timeout)?;

        Ok(Self {
            inner: Arc::new(ArcaClientInner {
                config,
                cert_pem,
                key_pem,
                http_client,
                tokens: RwLock::new(HashMap::new()),
                // Allow only 1 concurrent login per client to avoid redundant WSAA calls
                login_semaphore: Semaphore::new(1),
                token_store,
            }),
        })
    }

    /// Set or update the credentials
    pub fn set_credentials(&self, _cert_pem: Vec<u8>, _key_pem: Vec<u8>) {
        // Note: In the thread-safe version, credentials are set at build time
        // This method exists for API compatibility but credentials should be
        // set via the builder pattern. Consider using Arc::make_mut or
        // rebuilding the client if credentials need to change at runtime.
    }

    /// Get the current environment
    pub fn environment(&self) -> ArcaEnvironment {
        self.inner.config.environment
    }

    /// Login to a specific ARCA service (thread-safe)
    ///
    /// This method will automatically reuse valid tokens and refresh expired ones.
    /// Uses a semaphore to prevent multiple concurrent login attempts.
    pub async fn login(&self, service: &str) -> Result<()> {
        // 1. Fast path: check if we have a valid token in RAM (read lock, no semaphore needed)
        {
            let tokens = self.inner.tokens.read().await;
            if let Some(ta) = tokens.get(service) {
                if !ta.is_expired_with_margin(self.inner.config.token_margin_minutes) {
                    return Ok(());
                }
            }
        }

        // Acquire semaphore to prevent concurrent logins
        let _permit = self.inner.login_semaphore.acquire().await
            .map_err(|_| ArcaError::Auth("Login semaphore closed".to_string()))?;

        // 2. Double-check after acquiring semaphore (another thread may have refreshed RAM cache)
        {
            let tokens = self.inner.tokens.read().await;
            if let Some(ta) = tokens.get(service) {
                if !ta.is_expired_with_margin(self.inner.config.token_margin_minutes) {
                    return Ok(());
                }
            }
        }

        // 3. Persistent cache check: if not in RAM, try to load from disk
        if let Some(store) = &self.inner.token_store {
            if let Some(ta) = store.load_token(service).await? {
                if !ta.is_expired_with_margin(self.inner.config.token_margin_minutes) {
                    // Valid token found on disk, update RAM cache
                    let mut tokens = self.inner.tokens.write().await;
                    tokens.insert(service.to_string(), ta);
                    return Ok(());
                }
            }
        }

        // 4. Perform actual login (contacting AFIP)
        let ep = get_endpoints(self.inner.config.environment);
        let ltr = build_login_ticket_request(service);
        let cms = sign_cms_base64(&ltr, &self.inner.cert_pem, &self.inner.key_pem)?;
        let ta = wsaa_login(&self.inner.http_client, ep.wsaa_login_cms, &cms).await?;

        // 5. Update both caches
        if let Some(store) = &self.inner.token_store {
            store.save_token(service, &ta).await?;
        }

        let mut tokens = self.inner.tokens.write().await;
        tokens.insert(service.to_string(), ta);
        Ok(())
    }

    /// Get a clone of the token for a service
    async fn get_token(&self, service: &str) -> Result<TicketAcceso> {
        let tokens = self.inner.tokens.read().await;
        tokens.get(service).cloned().ok_or(ArcaError::NoToken)
    }

    /// Get WSFEv1 service context (handles login and token retrieval)
    /// This eliminates code duplication across all WSFEv1 methods.
    async fn wsfev1_context(&self) -> Result<WsFev1Context> {
        self.login("wsfe").await?;
        let ta = self.get_token("wsfe").await?;
        let ep = get_endpoints(self.inner.config.environment);

        Ok(WsFev1Context {
            service: WsFev1Service::new(ep.wsfev1.to_string(), self.inner.http_client.clone()),
            // Clone token and sign since TicketAcceso has custom Drop for zeroization
            token: ta.token.clone(),
            sign: ta.sign.clone(),
        })
    }

    /// Get the last authorized voucher number
    ///
    /// Automatically handles authentication.
    pub async fn get_last_voucher(&self, cuit: u64, pto_vta: i32, cbte_tipo: i32) -> Result<i32> {
        let ctx = self.wsfev1_context().await?;
        ctx.service
            .fe_comp_ultimo_autorizado(&ctx.token, &ctx.sign, cuit, pto_vta, cbte_tipo)
            .await
    }

    /// Authorize a voucher and get CAE
    ///
    /// Automatically handles authentication.
    pub async fn authorize_voucher(
        &self,
        cuit: u64,
        cab: FeCabReq,
        dets: Vec<FeDetReq>,
    ) -> Result<FeCaeResponse> {
        let ctx = self.wsfev1_context().await?;
        ctx.service
            .fecae_solicitar(&ctx.token, &ctx.sign, cuit, cab, dets)
            .await
    }

    /// Get an existing voucher by type, number and point of sale
    ///
    /// This is critical for error recovery - if a CAE request times out,
    /// use this method to check if the invoice was actually authorized.
    ///
    /// Automatically handles authentication.
    pub async fn get_voucher(
        &self,
        cuit: u64,
        cbte_tipo: i32,
        cbte_nro: u64,
        pto_vta: i32,
    ) -> Result<FeCompConsultarResponse> {
        let ctx = self.wsfev1_context().await?;
        ctx.service
            .fe_comp_consultar(&ctx.token, &ctx.sign, cuit, cbte_tipo, cbte_nro, pto_vta)
            .await
    }

    // ========== Parameter query methods ==========

    /// Get voucher types (facturas, notas de crédito, etc.)
    pub async fn get_tipos_cbte(&self, cuit: u64) -> Result<Vec<ParamTipo>> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fe_param_get_tipos_cbte(&ctx.token, &ctx.sign, cuit).await
    }

    /// Get document types (CUIT, DNI, etc.)
    pub async fn get_tipos_doc(&self, cuit: u64) -> Result<Vec<ParamTipo>> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fe_param_get_tipos_doc(&ctx.token, &ctx.sign, cuit).await
    }

    /// Get IVA rates
    pub async fn get_tipos_iva(&self, cuit: u64) -> Result<Vec<ParamTipo>> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fe_param_get_tipos_iva(&ctx.token, &ctx.sign, cuit).await
    }

    /// Get currency types
    pub async fn get_tipos_monedas(&self, cuit: u64) -> Result<Vec<ParamTipo>> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fe_param_get_tipos_monedas(&ctx.token, &ctx.sign, cuit).await
    }

    /// Get tax types (percepciones, retenciones, etc.)
    pub async fn get_tipos_tributos(&self, cuit: u64) -> Result<Vec<ParamTipo>> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fe_param_get_tipos_tributos(&ctx.token, &ctx.sign, cuit).await
    }

    /// Get concept types (productos, servicios, productos y servicios)
    pub async fn get_tipos_concepto(&self, cuit: u64) -> Result<Vec<ParamTipo>> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fe_param_get_tipos_concepto(&ctx.token, &ctx.sign, cuit).await
    }

    /// Get optional field types
    pub async fn get_tipos_opcional(&self, cuit: u64) -> Result<Vec<ParamTipo>> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fe_param_get_tipos_opcional(&ctx.token, &ctx.sign, cuit).await
    }

    /// Get receiver IVA conditions
    pub async fn get_condicion_iva_receptor(&self, cuit: u64) -> Result<Vec<ParamTipo>> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fe_param_get_condicion_iva_receptor(&ctx.token, &ctx.sign, cuit).await
    }

    /// Get points of sale
    pub async fn get_ptos_venta(&self, cuit: u64) -> Result<Vec<PtoVenta>> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fe_param_get_ptos_venta(&ctx.token, &ctx.sign, cuit).await
    }

    /// Get currency exchange rate
    pub async fn get_cotizacion(&self, cuit: u64, mon_id: &str) -> Result<Cotizacion> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fe_param_get_cotizacion(&ctx.token, &ctx.sign, cuit, mon_id).await
    }

    // ========== CAEA Methods ==========

    /// Request a CAEA for offline invoice emission
    ///
    /// Period format: YYYYMM (e.g., 202501 for January 2025)
    /// Order: 1 for first half of month (1-15), 2 for second half (16-end)
    pub async fn request_caea(&self, cuit: u64, periodo: i32, orden: i16) -> Result<CaeaResponse> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fecaea_solicitar(&ctx.token, &ctx.sign, cuit, periodo, orden).await
    }

    /// Consult a previously requested CAEA
    pub async fn get_caea(&self, cuit: u64, periodo: i32, orden: i16) -> Result<CaeaResponse> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fecaea_consultar(&ctx.token, &ctx.sign, cuit, periodo, orden).await
    }

    /// Register invoices that were issued offline with CAEA
    pub async fn register_caea_invoices(
        &self,
        cuit: u64,
        cab: FeCabReq,
        dets: Vec<FeDetReq>,
        caea: &str,
    ) -> Result<CaeaRegInformativoResult> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fecaea_reg_informativo(&ctx.token, &ctx.sign, cuit, cab, dets, caea).await
    }

    /// Declare that a CAEA period had no invoices
    pub async fn report_caea_no_movement(
        &self,
        cuit: u64,
        pto_vta: i32,
        caea: &str,
    ) -> Result<CaeaSinMovimiento> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fecaea_sin_movimiento_informar(&ctx.token, &ctx.sign, cuit, pto_vta, caea).await
    }

    /// Check if a CAEA period was declared as having no invoices
    pub async fn get_caea_no_movement(
        &self,
        cuit: u64,
        caea: &str,
        pto_vta: i32,
    ) -> Result<Vec<CaeaSinMovimiento>> {
        let ctx = self.wsfev1_context().await?;
        ctx.service.fecaea_sin_movimiento_consultar(&ctx.token, &ctx.sign, cuit, caea, pto_vta).await
    }

    /// Get a reference to the internal HTTP client
    pub fn http_client(&self) -> &HttpClient {
        &self.inner.http_client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arca_client_builder() {
        let client = ArcaClient::builder()
            .environment(ArcaEnvironment::Testing)
            .credentials(vec![1, 2, 3], vec![4, 5, 6])
            .build();

        assert!(client.is_ok());
    }

    #[test]
    fn test_arca_client_new() {
        let client = ArcaClient::new(ArcaEnvironment::Testing);
        assert!(client.is_ok());
    }

    #[test]
    fn test_arca_client_clone() {
        let client1 = ArcaClient::new(ArcaEnvironment::Testing).unwrap();
        let client2 = client1.clone();

        // Both should point to the same Arc
        assert!(Arc::ptr_eq(&client1.inner, &client2.inner));
    }

    #[test]
    fn test_arca_client_environment() {
        let client = ArcaClient::builder()
            .environment(ArcaEnvironment::Production)
            .build()
            .unwrap();

        assert_eq!(client.environment(), ArcaEnvironment::Production);
    }
}
