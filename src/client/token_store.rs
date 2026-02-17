use crate::auth::TicketAcceso;
use crate::error::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;

/// Trait for persistent token storage
#[async_trait]
pub trait TokenStore: Send + Sync {
    /// Save a token for a specific service
    async fn save_token(&self, service: &str, token: &TicketAcceso) -> Result<()>;
    /// Load a token for a specific service
    async fn load_token(&self, service: &str) -> Result<Option<TicketAcceso>>;
}

/// Token storage implementation using the file system
pub struct FileSystemTokenStore {
    base_path: PathBuf,
}

impl FileSystemTokenStore {
    /// Create a new FileSystemTokenStore at the specified path
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            base_path: path.into(),
        }
    }

    fn get_file_path(&self, service: &str) -> PathBuf {
        self.base_path.join(format!("arca_token_{}.json", service))
    }
}

#[async_trait]
impl TokenStore for FileSystemTokenStore {
    async fn save_token(&self, service: &str, token: &TicketAcceso) -> Result<()> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path).await.map_err(|e| {
                crate::error::ArcaError::Config(format!("Failed to create token directory: {}", e))
            })?;
        }

        let file_path = self.get_file_path(service);
        let json = serde_json::to_string(token).map_err(|e| {
            crate::error::ArcaError::Config(format!("Failed to serialize token: {}", e))
        })?;

        fs::write(file_path, json).await.map_err(|e| {
            crate::error::ArcaError::Config(format!("Failed to write token file: {}", e))
        })?;

        Ok(())
    }

    async fn load_token(&self, service: &str) -> Result<Option<TicketAcceso>> {
        let file_path = self.get_file_path(service);
        if !file_path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(file_path).await.map_err(|e| {
            crate::error::ArcaError::Config(format!("Failed to read token file: {}", e))
        })?;

        let token: TicketAcceso = serde_json::from_str(&json).map_err(|e| {
            crate::error::ArcaError::Config(format!("Failed to deserialize token: {}", e))
        })?;

        Ok(Some(token))
    }
}
