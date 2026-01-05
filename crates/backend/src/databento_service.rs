//! DataBento integration service.
//!
//! This module provides the real DataBento API integration for
//! historical and live market data.

use crate::service::{LiveStream, MarketDataService, ServiceError};
use async_trait::async_trait;
use databento::{
    dbn::{
        decode::DbnMetadata, Dataset, OhlcvMsg, PitSymbolMap, SType, Schema as DbSchema,
        SymbolIndex, TradeMsg,
    },
    historical::timeseries::GetRangeParams,
    live::Subscription,
    HistoricalClient, LiveClient,
};
use shared::{HistoricalRequest, HistoricalResponse, LiveMessage, OhlcvRecord, Schema, TradeRecord};
use std::num::NonZeroU64;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tracing::{error, info, warn};

/// DataBento service for real market data.
///
/// # Configuration
/// Requires a DataBento API key.
///
/// # Supported Features
/// - Historical trades and OHLCV data
/// - Live streaming trades
///
/// # Dataset
/// Default dataset: `GLBX.MDP3` (CME Globex)
pub struct DatabentoService {
    api_key: String,
    dataset: Dataset,
}

impl DatabentoService {
    /// Create a new DataBento service with the given API key.
    pub fn new(api_key: String) -> Self {
        assert!(!api_key.is_empty(), "API key cannot be empty");

        info!("Initializing DataBento service with GLBX.MDP3 dataset");

        Self {
            api_key,
            dataset: Dataset::GlbxMdp3, // CME Globex
        }
    }

    /// Map our schema string to DataBento's Schema enum.
    fn map_schema(schema: &str) -> Result<DbSchema, ServiceError> {
        match schema {
            "trades" => Ok(DbSchema::Trades),
            "ohlcv-1s" => Ok(DbSchema::Ohlcv1S),
            "ohlcv-1m" => Ok(DbSchema::Ohlcv1M),
            _ => Err(ServiceError::InvalidSchema(format!(
                "Unknown schema: {}. Expected: trades, ohlcv-1s, or ohlcv-1m",
                schema
            ))),
        }
    }

    /// Parse RFC3339 timestamp string to OffsetDateTime.
    fn parse_timestamp(ts: &str) -> Result<OffsetDateTime, ServiceError> {
        OffsetDateTime::parse(ts, &Rfc3339).map_err(|e| {
            ServiceError::InvalidTimeFormat(format!("Invalid RFC3339 timestamp '{}': {}", ts, e))
        })
    }
}

#[async_trait]
impl MarketDataService for DatabentoService {
    async fn get_historical(
        &self,
        req: &HistoricalRequest,
    ) -> Result<HistoricalResponse, ServiceError> {
        info!(
            symbols = ?req.symbols,
            schema = %req.schema,
            start = %req.start_rfc3339,
            end = %req.end_rfc3339,
            limit = req.limit,
            "DataBento historical request"
        );

        // Parse inputs
        let db_schema = Self::map_schema(&req.schema)?;
        let start = Self::parse_timestamp(&req.start_rfc3339)?;
        let end = Self::parse_timestamp(&req.end_rfc3339)?;

        // Parse our schema enum for response building
        let schema: Schema = req
            .schema
            .parse()
            .map_err(|e: String| ServiceError::InvalidSchema(e))?;

        // Build historical client
        let mut client = HistoricalClient::builder()
            .key(self.api_key.clone())
            .map_err(|e| ServiceError::ApiError(format!("Failed to create client: {}", e)))?
            .build()
            .map_err(|e| ServiceError::ApiError(format!("Failed to build client: {}", e)))?;

        // Build request parameters
        let params = GetRangeParams::builder()
            .dataset(self.dataset)
            .date_time_range((start, end))
            .symbols(req.symbols.clone())
            .schema(db_schema)
            .stype_in(SType::Parent) // Using parent symbols like "ES.FUT"
            .limit(NonZeroU64::new(req.limit as u64))
            .build();

        // Fetch data
        let mut decoder = client
            .timeseries()
            .get_range(&params)
            .await
            .map_err(|e| ServiceError::ApiError(format!("API request failed: {}", e)))?;

        // Get symbol map for resolving instrument IDs to symbols
        let symbol_map = decoder
            .metadata()
            .symbol_map_for_date(start.date())
            .map_err(|e| {
                warn!("Failed to get symbol map: {}", e);
                ServiceError::ApiError(format!("Symbol map error: {}", e))
            })?;

        // Process records based on schema
        match schema {
            Schema::Trades => {
                let mut trades = Vec::new();

                while let Some(record) = decoder
                    .decode_record::<TradeMsg>()
                    .await
                    .map_err(|e| ServiceError::ApiError(format!("Decode error: {}", e)))?
                {
                    // Resolve symbol from instrument ID
                    let symbol = symbol_map
                        .get(record.hd.instrument_id)
                        .map(|s: &String| s.to_string())
                        .unwrap_or_else(|| format!("ID:{}", record.hd.instrument_id));

                    trades.push(TradeRecord {
                        ts_event_unix_ns: record.hd.ts_event,
                        symbol,
                        price_i64: record.price,
                        size_u32: record.size,
                    });

                    if trades.len() >= req.limit as usize {
                        break;
                    }
                }

                info!(count = trades.len(), "Fetched trades from DataBento");
                Ok(HistoricalResponse::Trades { data: trades })
            }
            Schema::Ohlcv1S | Schema::Ohlcv1M => {
                let mut bars = Vec::new();

                while let Some(record) = decoder
                    .decode_record::<OhlcvMsg>()
                    .await
                    .map_err(|e| ServiceError::ApiError(format!("Decode error: {}", e)))?
                {
                    let symbol = symbol_map
                        .get(record.hd.instrument_id)
                        .map(|s: &String| s.to_string())
                        .unwrap_or_else(|| format!("ID:{}", record.hd.instrument_id));

                    bars.push(OhlcvRecord {
                        ts_event_unix_ns: record.hd.ts_event,
                        symbol,
                        open_i64: record.open,
                        high_i64: record.high,
                        low_i64: record.low,
                        close_i64: record.close,
                        volume_u64: record.volume,
                    });

                    if bars.len() >= req.limit as usize {
                        break;
                    }
                }

                info!(count = bars.len(), "Fetched OHLCV bars from DataBento");

                match schema {
                    Schema::Ohlcv1S => Ok(HistoricalResponse::Ohlcv1S { data: bars }),
                    Schema::Ohlcv1M => Ok(HistoricalResponse::Ohlcv1M { data: bars }),
                    _ => unreachable!(),
                }
            }
        }
    }

    async fn subscribe_live(
        &self,
        symbols: Vec<String>,
        schema: String,
    ) -> Result<LiveStream, ServiceError> {
        info!(
            symbols = ?symbols,
            schema = %schema,
            "DataBento live subscription request"
        );

        let db_schema = Self::map_schema(&schema)?;
        let api_key = self.api_key.clone();
        let dataset = self.dataset;
        let symbols_clone = symbols.clone();

        // Create the live stream
        let stream = async_stream::stream! {
            // First emit connected message
            yield LiveMessage::Connected {
                symbols: symbols_clone.clone(),
                schema: schema.clone(),
            };

            // Build live client
            let client_builder = match LiveClient::builder().key(api_key) {
                Ok(b) => b,
                Err(e) => {
                    error!("Failed to set API key: {}", e);
                    yield LiveMessage::Error {
                        message: format!("Failed to set API key: {}", e),
                    };
                    return;
                }
            };

            // dataset() returns the builder directly (not a Result)
            let client_builder = client_builder.dataset(dataset);

            let mut client = match client_builder.build().await {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to create live client: {}", e);
                    yield LiveMessage::Error {
                        message: format!("Failed to connect: {}", e),
                    };
                    return;
                }
            };

            // Subscribe
            let subscription = Subscription::builder()
                .symbols(symbols_clone.clone())
                .schema(db_schema)
                .stype_in(SType::Parent)
                .build();

            if let Err(e) = client.subscribe(subscription).await {
                error!("Failed to subscribe: {}", e);
                yield LiveMessage::Error {
                    message: format!("Subscription failed: {}", e),
                };
                return;
            }

            // Start receiving
            if let Err(e) = client.start().await {
                error!("Failed to start stream: {}", e);
                yield LiveMessage::Error {
                    message: format!("Failed to start stream: {}", e),
                };
                return;
            }

            // Symbol map for resolving instrument IDs
            let mut symbol_map = PitSymbolMap::new();

            // Stream records
            loop {
                match client.next_record().await {
                    Ok(Some(record)) => {
                        // Update symbol map
                        if let Err(e) = symbol_map.on_record(record) {
                            warn!("Symbol map update failed: {}", e);
                        }

                        // Try to extract as TradeMsg
                        if let Some(trade) = record.get::<TradeMsg>() {
                            let symbol = symbol_map
                                .get_for_rec(trade)
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("ID:{}", trade.hd.instrument_id));

                            yield LiveMessage::Trade {
                                ts_event_unix_ns: trade.hd.ts_event,
                                symbol,
                                price_i64: trade.price,
                                size_u32: trade.size,
                            };
                        }
                    }
                    Ok(None) => {
                        info!("Live stream ended");
                        break;
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        yield LiveMessage::Error {
                            message: format!("Stream error: {}", e),
                        };
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    fn name(&self) -> &'static str {
        "DatabentoService"
    }
}
