use crate::constants::defaults::{HTTP_CONNECT_TIMEOUT, HTTP_TIMEOUT, TOKEN_EXPIRY_MARGIN_MINUTES};
use crate::constants::endpoints::ArcaEnvironment;
use std::time::Duration;

/// Configuration for ArcaClient
#[derive(Clone)]
pub struct ArcaClientConfig {
    pub environment: ArcaEnvironment,
    pub http_timeout: Duration,
    pub http_connect_timeout: Duration,
    pub token_margin_minutes: i64,
}

impl Default for ArcaClientConfig {
    fn default() -> Self {
        Self {
            environment: ArcaEnvironment::Testing,
            http_timeout: HTTP_TIMEOUT,
            http_connect_timeout: HTTP_CONNECT_TIMEOUT,
            token_margin_minutes: TOKEN_EXPIRY_MARGIN_MINUTES,
        }
    }
}
