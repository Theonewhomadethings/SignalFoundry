//! Mock implementation of MarketDataService for development without API key.

use crate::service::{LiveStream, MarketDataService, ServiceError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use shared::{
    HistoricalRequest, HistoricalResponse, LiveMessage, OhlcvRecord, Schema, TradeRecord,
};
use std::time::Duration;

/// Mock service that generates realistic market data without external API.
pub struct MockService {
    /// Base price for mock data generation (ES futures ~4500-5500 range)
    base_price: i64,
}

impl MockService {
    pub fn new() -> Self {
        // Base price in fixed-point 1e9 format (e.g., 5000.00 = 5000 * 1e9)
        Self {
            base_price: 5_000_000_000_000, // 5000.00
        }
    }

    /// Generate mock trade data for the given time range.
    fn generate_trades(
        &self,
        symbols: &[String],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Vec<TradeRecord> {
        let mut rng = rand::thread_rng();
        let mut trades = Vec::new();
        let mut current_price = self.base_price;

        let duration_ns = (end - start).num_nanoseconds().unwrap_or(0) as u64;
        let num_trades = std::cmp::min(limit as usize, 1000);

        for i in 0..num_trades {
            // Random walk for price
            let price_change: i64 = rng.gen_range(-500_000_000..=500_000_000); // ±0.50
            current_price = (current_price + price_change).max(self.base_price - 50_000_000_000); // Don't go too low

            // Spread trades across the time range
            let time_offset = if num_trades > 1 {
                (duration_ns * i as u64) / (num_trades as u64 - 1)
            } else {
                0
            };
            let ts = start.timestamp_nanos_opt().unwrap_or(0) as u64 + time_offset;

            // Pick a random symbol from the list
            let symbol = symbols[i % symbols.len()].clone();

            trades.push(TradeRecord {
                ts_event_unix_ns: ts,
                symbol,
                price_i64: current_price,
                size_u32: rng.gen_range(1..=50),
            });
        }

        trades
    }

    /// Generate mock OHLCV data for the given time range.
    fn generate_ohlcv(
        &self,
        symbols: &[String],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        bar_duration_secs: i64,
        limit: u32,
    ) -> Vec<OhlcvRecord> {
        let mut rng = rand::thread_rng();
        let mut bars = Vec::new();
        let mut current_price = self.base_price;

        let duration_secs = (end - start).num_seconds();
        let num_bars = std::cmp::min((duration_secs / bar_duration_secs) as usize, limit as usize);

        for i in 0..num_bars {
            for symbol in symbols {
                let bar_start = start + chrono::Duration::seconds(i as i64 * bar_duration_secs);
                let ts = bar_start.timestamp_nanos_opt().unwrap_or(0) as u64;

                let open = current_price;

                // Generate realistic intrabar movement
                let high_delta: i64 = rng.gen_range(0..=2_000_000_000); // Up to +2.00
                let low_delta: i64 = rng.gen_range(0..=2_000_000_000); // Up to -2.00
                let close_delta: i64 = rng.gen_range(-1_000_000_000..=1_000_000_000);

                let high = open + high_delta;
                let low = open - low_delta;
                let close = (open + close_delta).clamp(low, high);

                bars.push(OhlcvRecord {
                    ts_event_unix_ns: ts,
                    symbol: symbol.clone(),
                    open_i64: open,
                    high_i64: high,
                    low_i64: low,
                    close_i64: close,
                    volume_u64: rng.gen_range(100..=10000),
                });

                // Next bar opens at previous close
                current_price = close;
            }
        }

        bars
    }
}

impl Default for MockService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MarketDataService for MockService {
    async fn get_historical(
        &self,
        req: &HistoricalRequest,
    ) -> Result<HistoricalResponse, ServiceError> {
        // Parse schema
        let schema: Schema = req
            .schema
            .parse()
            .map_err(|e: String| ServiceError::InvalidSchema(e))?;

        // Parse timestamps
        let start = DateTime::parse_from_rfc3339(&req.start_rfc3339)
            .map_err(|e| ServiceError::InvalidTimeFormat(format!("start_rfc3339: {}", e)))?
            .with_timezone(&Utc);

        let end = DateTime::parse_from_rfc3339(&req.end_rfc3339)
            .map_err(|e| ServiceError::InvalidTimeFormat(format!("end_rfc3339: {}", e)))?
            .with_timezone(&Utc);

        // Generate mock data based on schema
        match schema {
            Schema::Trades => {
                let data = self.generate_trades(&req.symbols, start, end, req.limit);
                Ok(HistoricalResponse::Trades { data })
            }
            Schema::Ohlcv1S => {
                let data = self.generate_ohlcv(&req.symbols, start, end, 1, req.limit);
                Ok(HistoricalResponse::Ohlcv1S { data })
            }
            Schema::Ohlcv1M => {
                let data = self.generate_ohlcv(&req.symbols, start, end, 60, req.limit);
                Ok(HistoricalResponse::Ohlcv1M { data })
            }
        }
    }

    async fn subscribe_live(
        &self,
        symbols: Vec<String>,
        schema: String,
    ) -> Result<LiveStream, ServiceError> {
        // Validate schema
        let _schema: Schema = schema
            .parse()
            .map_err(|e: String| ServiceError::InvalidSchema(e))?;

        let base_price = self.base_price;
        let symbols_clone = symbols.clone();

        // Create a stream that emits mock trades at random intervals
        // Use StdRng which is Send-safe (unlike thread_rng)
        let stream = async_stream::stream! {
            let mut rng = StdRng::from_entropy();
            let mut current_price = base_price;
            let mut symbol_idx = 0;

            // First, emit a connected message
            yield LiveMessage::Connected {
                symbols: symbols_clone.clone(),
                schema: schema.clone(),
            };

            loop {
                // Random delay between 100-500ms
                let delay_ms = rng.gen_range(100..=500);
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;

                // Random price movement
                let price_change: i64 = rng.gen_range(-250_000_000..=250_000_000); // ±0.25
                current_price = (current_price + price_change).max(base_price - 50_000_000_000);

                let symbol = symbols_clone[symbol_idx % symbols_clone.len()].clone();
                symbol_idx += 1;

                let ts = Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;

                yield LiveMessage::Trade {
                    ts_event_unix_ns: ts,
                    symbol,
                    price_i64: current_price,
                    size_u32: rng.gen_range(1..=25),
                };
            }
        };

        Ok(Box::pin(stream))
    }

    fn name(&self) -> &'static str {
        "MockService"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_mock_historical_trades() {
        let service = MockService::new();
        let req = HistoricalRequest {
            symbols: vec!["ES.FUT".to_string()],
            schema: "trades".to_string(),
            stype_in: "parent".to_string(),
            start_rfc3339: "2024-01-01T00:00:00Z".to_string(),
            end_rfc3339: "2024-01-01T01:00:00Z".to_string(),
            limit: 100,
        };

        let resp = service.get_historical(&req).await.unwrap();
        match resp {
            HistoricalResponse::Trades { data } => {
                assert!(!data.is_empty());
                assert!(data.len() <= 100);
                assert_eq!(data[0].symbol, "ES.FUT");
            }
            _ => panic!("Expected trades response"),
        }
    }

    #[tokio::test]
    async fn test_mock_historical_ohlcv() {
        let service = MockService::new();
        let req = HistoricalRequest {
            symbols: vec!["ES.FUT".to_string()],
            schema: "ohlcv-1m".to_string(),
            stype_in: "parent".to_string(),
            start_rfc3339: "2024-01-01T00:00:00Z".to_string(),
            end_rfc3339: "2024-01-01T01:00:00Z".to_string(),
            limit: 100,
        };

        let resp = service.get_historical(&req).await.unwrap();
        match resp {
            HistoricalResponse::Ohlcv1M { data } => {
                assert!(!data.is_empty());
                // 1 hour = 60 minutes = 60 bars
                assert!(data.len() <= 60);
                assert_eq!(data[0].symbol, "ES.FUT");
                // Verify OHLC relationship
                for bar in &data {
                    assert!(bar.low_i64 <= bar.open_i64);
                    assert!(bar.low_i64 <= bar.close_i64);
                    assert!(bar.high_i64 >= bar.open_i64);
                    assert!(bar.high_i64 >= bar.close_i64);
                }
            }
            _ => panic!("Expected ohlcv-1m response"),
        }
    }

    #[tokio::test]
    async fn test_mock_invalid_schema() {
        let service = MockService::new();
        let req = HistoricalRequest {
            symbols: vec!["ES.FUT".to_string()],
            schema: "invalid".to_string(),
            stype_in: "parent".to_string(),
            start_rfc3339: "2024-01-01T00:00:00Z".to_string(),
            end_rfc3339: "2024-01-01T01:00:00Z".to_string(),
            limit: 100,
        };

        let result = service.get_historical(&req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_invalid_time_format() {
        let service = MockService::new();
        let req = HistoricalRequest {
            symbols: vec!["ES.FUT".to_string()],
            schema: "trades".to_string(),
            stype_in: "parent".to_string(),
            start_rfc3339: "invalid-time".to_string(),
            end_rfc3339: "2024-01-01T01:00:00Z".to_string(),
            limit: 100,
        };

        let result = service.get_historical(&req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_live_stream() {
        let service = MockService::new();
        let stream = service
            .subscribe_live(vec!["ES.FUT".to_string()], "trades".to_string())
            .await
            .unwrap();

        // Take first 3 messages (connected + 2 trades)
        let messages: Vec<_> = stream.take(3).collect().await;

        assert_eq!(messages.len(), 3);

        // First message should be Connected
        match &messages[0] {
            LiveMessage::Connected { symbols, schema } => {
                assert_eq!(symbols, &vec!["ES.FUT".to_string()]);
                assert_eq!(schema, "trades");
            }
            _ => panic!("Expected Connected message first"),
        }

        // Subsequent messages should be trades
        match &messages[1] {
            LiveMessage::Trade { symbol, .. } => {
                assert_eq!(symbol, "ES.FUT");
            }
            _ => panic!("Expected Trade message"),
        }
    }
}
