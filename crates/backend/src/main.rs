//! Market Data Viewer Backend
//!
//! Axum server providing REST and WebSocket APIs for market data.
//! Supports both mock mode (no API key) and live DataBento mode.

mod databento_service;
mod handlers;
mod mock_service;
mod service;

use axum::{
    routing::{get, post},
    Router,
};
use databento_service::DatabentoService;
use handlers::AppState;
use mock_service::MockService;
use service::MarketDataService;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Server configuration.
struct Config {
    host: String,
    port: u16,
    databento_api_key: Option<String>,
}

impl Config {
    fn from_env() -> Self {
        Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3001),
            databento_api_key: std::env::var("DATABENTO_API_KEY").ok(),
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    // Select service based on API key presence
    let service: Arc<dyn MarketDataService> = if let Some(api_key) = config.databento_api_key {
        // Use DataBento service when API key is available
        info!("DATABENTO_API_KEY is set - using DataBento service");
        Arc::new(DatabentoService::new(api_key))
    } else {
        info!("DATABENTO_API_KEY not set - running in MOCK mode");
        info!("Set DATABENTO_API_KEY environment variable to enable live data");
        Arc::new(MockService::new())
    };

    info!("Using service: {}", service.name());

    let state = Arc::new(AppState { service });

    // Configure CORS for local development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/api/health", get(handlers::health))
        .route("/api/historical", post(handlers::historical))
        .route("/ws/live", get(handlers::live_ws))
        .layer(cors)
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid address");

    info!("Starting server on http://{}", addr);
    info!("Health check: http://{}/api/health", addr);
    info!("Historical API: POST http://{}/api/historical", addr);
    info!("Live WebSocket: ws://{}/ws/live", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
