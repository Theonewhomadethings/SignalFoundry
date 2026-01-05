//! MarketDataService trait defining the interface for market data providers.

use async_trait::async_trait;
use shared::{HistoricalRequest, HistoricalResponse, LiveMessage};
use std::pin::Pin;
use tokio_stream::Stream;

/// Error type for service operations.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)] // Some variants reserved for DataBento integration
pub enum ServiceError {
    #[error("Invalid schema: {0}")]
    InvalidSchema(String),
    #[error("Invalid time format: {0}")]
    InvalidTimeFormat(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Not configured: {0}")]
    NotConfigured(String),
}

/// A stream of live market data messages.
pub type LiveStream = Pin<Box<dyn Stream<Item = LiveMessage> + Send>>;

/// Trait defining the interface for market data services.
/// Implemented by both MockService and DatabentoService.
#[async_trait]
pub trait MarketDataService: Send + Sync {
    /// Fetch historical market data.
    async fn get_historical(
        &self,
        req: &HistoricalRequest,
    ) -> Result<HistoricalResponse, ServiceError>;

    /// Subscribe to live market data.
    /// Returns a stream of LiveMessage that can be forwarded to WebSocket clients.
    async fn subscribe_live(
        &self,
        symbols: Vec<String>,
        schema: String,
    ) -> Result<LiveStream, ServiceError>;

    /// Get the name of this service (for logging).
    fn name(&self) -> &'static str;
}
