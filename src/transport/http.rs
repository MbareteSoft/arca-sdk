use crate::constants::defaults::{HTTP_CONNECT_TIMEOUT, HTTP_TIMEOUT};
use crate::error::{ArcaError, Result};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

/// Thread-safe HTTP client with connection pooling
#[derive(Clone)]
pub struct HttpClient {
    inner: Arc<Client>,
}

impl HttpClient {
    /// Create a new HTTP client with default timeouts
    pub fn new() -> Result<Self> {
        Self::with_config(HTTP_TIMEOUT, HTTP_CONNECT_TIMEOUT)
    }

    /// Create a new HTTP client with custom timeouts
    pub fn with_config(timeout: Duration, connect_timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .connect_timeout(connect_timeout)
            .pool_max_idle_per_host(5)
            .build()
            .map_err(ArcaError::Transport)?;

        Ok(Self {
            inner: Arc::new(client),
        })
    }

    /// Get a reference to the underlying reqwest Client
    pub fn client(&self) -> &Client {
        &self.inner
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_new() {
        let client = HttpClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_http_client_clone() {
        let client1 = HttpClient::new().unwrap();
        let client2 = client1.clone();
        // Both should point to the same Arc
        assert!(Arc::ptr_eq(&client1.inner, &client2.inner));
    }

    #[test]
    fn test_http_client_with_custom_config() {
        let client = HttpClient::with_config(
            Duration::from_secs(60),
            Duration::from_secs(20),
        );
        assert!(client.is_ok());
    }
}
