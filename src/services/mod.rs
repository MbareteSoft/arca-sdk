pub mod common;
pub mod padron_a13;
pub mod wsfev1;

pub use common::build_auth_block;
pub use padron_a13::PadronA13Service;
pub use wsfev1::WsFev1Service;

use async_trait::async_trait;

/// Trait for ARCA services that require authentication
///
/// Implement this trait for each ARCA web service to provide
/// a consistent interface and enable easy extension.
#[async_trait]
pub trait ArcaService: Send + Sync {
    /// Service identifier used for WSAA login (e.g., "wsfe", "ws_sr_padron_a13")
    fn service_id(&self) -> &'static str;

    /// The service endpoint URL
    fn endpoint(&self) -> &str;

    /// XML namespace for this service
    fn namespace(&self) -> &'static str;

    /// XML prefix for this service
    fn prefix(&self) -> &'static str;
}
