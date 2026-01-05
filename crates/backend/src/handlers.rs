//! HTTP and WebSocket handlers for the market data API.

use crate::service::{MarketDataService, ServiceError};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use shared::{ErrorResponse, HistoricalRequest};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Application state shared across handlers.
pub struct AppState {
    pub service: Arc<dyn MarketDataService>,
}

/// Health check endpoint.
pub async fn health() -> &'static str {
    "ok"
}

/// Convert ServiceError to HTTP response.
impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ServiceError::InvalidSchema(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServiceError::InvalidTimeFormat(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServiceError::ApiError(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            ServiceError::ConnectionError(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            ServiceError::NotConfigured(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
        };

        let body = Json(ErrorResponse {
            error: message,
            code: status.as_u16(),
        });

        (status, body).into_response()
    }
}

/// POST /api/historical - Fetch historical market data.
pub async fn historical(
    State(state): State<Arc<AppState>>,
    Json(req): Json<HistoricalRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    info!(
        symbols = ?req.symbols,
        schema = %req.schema,
        start = %req.start_rfc3339,
        end = %req.end_rfc3339,
        "Fetching historical data"
    );

    let response = state.service.get_historical(&req).await?;

    Ok(Json(response))
}

/// Query parameters for WebSocket connection.
#[derive(Debug, Deserialize)]
pub struct LiveParams {
    /// Comma-separated list of symbols (default: "ES.FUT")
    #[serde(default = "default_symbols")]
    pub symbols: String,
    /// Schema type (default: "trades")
    #[serde(default = "default_schema")]
    pub schema: String,
    /// Symbol type input (default: "parent") - reserved for DataBento integration
    #[serde(default = "default_stype_in")]
    #[allow(dead_code)]
    pub stype_in: String,
}

fn default_symbols() -> String {
    "ES.FUT".to_string()
}

fn default_schema() -> String {
    "trades".to_string()
}

fn default_stype_in() -> String {
    "parent".to_string()
}

/// GET /ws/live - WebSocket endpoint for live market data.
pub async fn live_ws(
    ws: WebSocketUpgrade,
    Query(params): Query<LiveParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let symbols: Vec<String> = params
        .symbols
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    info!(
        symbols = ?symbols,
        schema = %params.schema,
        "WebSocket connection request"
    );

    ws.on_upgrade(move |socket| handle_live_socket(socket, state, symbols, params.schema))
}

/// Handle an active WebSocket connection.
async fn handle_live_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    symbols: Vec<String>,
    schema: String,
) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to live data
    let stream = match state
        .service
        .subscribe_live(symbols.clone(), schema.clone())
        .await
    {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to subscribe: {}", e);
            let error_msg = serde_json::to_string(&shared::LiveMessage::Error {
                message: e.to_string(),
            })
            .unwrap_or_else(|_| r#"{"type":"error","message":"Unknown error"}"#.to_string());
            let _ = sender.send(Message::Text(error_msg)).await;
            return;
        }
    };

    info!(symbols = ?symbols, schema = %schema, "WebSocket connected");

    // Spawn a task to forward messages from the stream to the WebSocket
    let mut stream = stream;
    let send_task = tokio::spawn(async move {
        while let Some(msg) = stream.next().await {
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    warn!("Failed to serialize message: {}", e);
                }
            }
        }
    });

    // Handle incoming messages (for future use, e.g., ping/pong or resubscription)
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => break,
                Message::Ping(data) => {
                    // Ping is handled automatically by axum
                    tracing::trace!("Received ping: {:?}", data);
                }
                Message::Text(text) => {
                    // Could handle subscription changes here in the future
                    tracing::debug!("Received text: {}", text);
                }
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {
            info!("Send task completed");
        }
        _ = recv_task => {
            info!("Receive task completed (client disconnected)");
        }
    }

    info!(symbols = ?symbols, "WebSocket disconnected");
}
