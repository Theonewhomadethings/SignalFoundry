//! Shared types for the Market Data Viewer API.
//!
//! These types are used by both the backend and can be serialized to JSON
//! for the frontend.

use serde::{Deserialize, Serialize};

/// Supported schema types for market data queries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Schema {
    Trades,
    #[serde(rename = "ohlcv-1s")]
    Ohlcv1S,
    #[serde(rename = "ohlcv-1m")]
    Ohlcv1M,
}

impl Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Schema::Trades => "trades",
            Schema::Ohlcv1S => "ohlcv-1s",
            Schema::Ohlcv1M => "ohlcv-1m",
        }
    }
}

impl std::str::FromStr for Schema {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "trades" => Ok(Schema::Trades),
            "ohlcv-1s" => Ok(Schema::Ohlcv1S),
            "ohlcv-1m" => Ok(Schema::Ohlcv1M),
            _ => Err(format!(
                "Invalid schema: {}. Expected: trades, ohlcv-1s, or ohlcv-1m",
                s
            )),
        }
    }
}

/// Request for historical market data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalRequest {
    /// Symbols to query (e.g., ["ES.FUT", "CL.FUT"])
    pub symbols: Vec<String>,
    /// Data schema: "trades", "ohlcv-1s", or "ohlcv-1m"
    pub schema: String,
    /// Symbol type input (e.g., "parent", "raw_symbol")
    #[serde(default = "default_stype_in")]
    pub stype_in: String,
    /// Start time in RFC3339 format
    pub start_rfc3339: String,
    /// End time in RFC3339 format
    pub end_rfc3339: String,
    /// Maximum number of records to return
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_stype_in() -> String {
    "parent".to_string()
}

fn default_limit() -> u32 {
    1000
}

/// A single trade record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    /// Timestamp in nanoseconds since Unix epoch
    pub ts_event_unix_ns: u64,
    /// Symbol name
    pub symbol: String,
    /// Price as fixed-point integer (divide by 1e9 for float)
    pub price_i64: i64,
    /// Trade size
    pub size_u32: u32,
}

/// A single OHLCV bar record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvRecord {
    /// Timestamp in nanoseconds since Unix epoch (bar open time)
    pub ts_event_unix_ns: u64,
    /// Symbol name
    pub symbol: String,
    /// Open price as fixed-point integer (divide by 1e9 for float)
    pub open_i64: i64,
    /// High price as fixed-point integer
    pub high_i64: i64,
    /// Low price as fixed-point integer
    pub low_i64: i64,
    /// Close price as fixed-point integer
    pub close_i64: i64,
    /// Volume
    pub volume_u64: u64,
}

/// Response containing historical trade data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesResponse {
    pub schema: String,
    pub data: Vec<TradeRecord>,
}

/// Response containing historical OHLCV data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvResponse {
    pub schema: String,
    pub data: Vec<OhlcvRecord>,
}

/// Unified historical response that can contain either trades or OHLCV data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "schema")]
pub enum HistoricalResponse {
    #[serde(rename = "trades")]
    Trades { data: Vec<TradeRecord> },
    #[serde(rename = "ohlcv-1s")]
    Ohlcv1S { data: Vec<OhlcvRecord> },
    #[serde(rename = "ohlcv-1m")]
    Ohlcv1M { data: Vec<OhlcvRecord> },
}

/// Message sent over WebSocket for live data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LiveMessage {
    #[serde(rename = "trade")]
    Trade {
        ts_event_unix_ns: u64,
        symbol: String,
        price_i64: i64,
        size_u32: u32,
    },
    #[serde(rename = "ohlcv")]
    Ohlcv {
        ts_event_unix_ns: u64,
        symbol: String,
        open_i64: i64,
        high_i64: i64,
        low_i64: i64,
        close_i64: i64,
        volume_u64: u64,
    },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "connected")]
    Connected {
        symbols: Vec<String>,
        schema: String,
    },
}

/// Error response for API errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_parsing() {
        assert_eq!("trades".parse::<Schema>().unwrap(), Schema::Trades);
        assert_eq!("ohlcv-1s".parse::<Schema>().unwrap(), Schema::Ohlcv1S);
        assert_eq!("ohlcv-1m".parse::<Schema>().unwrap(), Schema::Ohlcv1M);
        assert!("invalid".parse::<Schema>().is_err());
    }

    #[test]
    fn test_schema_as_str() {
        assert_eq!(Schema::Trades.as_str(), "trades");
        assert_eq!(Schema::Ohlcv1S.as_str(), "ohlcv-1s");
        assert_eq!(Schema::Ohlcv1M.as_str(), "ohlcv-1m");
    }

    #[test]
    fn test_historical_request_serialization() {
        let req = HistoricalRequest {
            symbols: vec!["ES.FUT".to_string()],
            schema: "trades".to_string(),
            stype_in: "parent".to_string(),
            start_rfc3339: "2022-06-10T14:30:00Z".to_string(),
            end_rfc3339: "2022-06-10T14:40:00Z".to_string(),
            limit: 1000,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("ES.FUT"));
        assert!(json.contains("trades"));
    }

    #[test]
    fn test_live_message_serialization() {
        let msg = LiveMessage::Trade {
            ts_event_unix_ns: 1234567890,
            symbol: "ES.FUT".to_string(),
            price_i64: 4_500_000_000_000,
            size_u32: 10,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"trade\""));
    }

    #[test]
    fn test_historical_response_serialization() {
        let resp = HistoricalResponse::Trades {
            data: vec![TradeRecord {
                ts_event_unix_ns: 1234567890,
                symbol: "ES.FUT".to_string(),
                price_i64: 4_500_000_000_000,
                size_u32: 10,
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"schema\":\"trades\""));
    }
}
