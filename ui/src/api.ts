/**
 * API utilities for communicating with the backend.
 */

// Types matching backend shared crate
export interface HistoricalRequest {
  symbols: string[];
  schema: 'trades' | 'ohlcv-1s' | 'ohlcv-1m';
  stype_in?: string;
  start_rfc3339: string;
  end_rfc3339: string;
  limit?: number;
}

export interface TradeRecord {
  ts_event_unix_ns: number;
  symbol: string;
  price_i64: number;
  size_u32: number;
}

export interface OhlcvRecord {
  ts_event_unix_ns: number;
  symbol: string;
  open_i64: number;
  high_i64: number;
  low_i64: number;
  close_i64: number;
  volume_u64: number;
}

export type HistoricalResponse =
  | { schema: 'trades'; data: TradeRecord[] }
  | { schema: 'ohlcv-1s'; data: OhlcvRecord[] }
  | { schema: 'ohlcv-1m'; data: OhlcvRecord[] };

export type LiveMessage =
  | { type: 'trade'; ts_event_unix_ns: number; symbol: string; price_i64: number; size_u32: number }
  | { type: 'ohlcv'; ts_event_unix_ns: number; symbol: string; open_i64: number; high_i64: number; low_i64: number; close_i64: number; volume_u64: number }
  | { type: 'error'; message: string }
  | { type: 'connected'; symbols: string[]; schema: string };

// Price conversion utilities
// DataBento uses fixed-point 1e-9 format
const PRICE_SCALE = 1e9;

export function formatPrice(priceI64: number): string {
  return (priceI64 / PRICE_SCALE).toFixed(2);
}

export function formatPriceNumber(priceI64: number): number {
  return priceI64 / PRICE_SCALE;
}

// Timestamp conversion
export function formatTimestamp(tsNs: number): string {
  const date = new Date(tsNs / 1e6); // ns to ms
  const base = date.toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
  // Add milliseconds manually
  const ms = date.getMilliseconds().toString().padStart(3, '0');
  return `${base}.${ms}`;
}

export function formatTimestampFull(tsNs: number): string {
  const date = new Date(tsNs / 1e6);
  return date.toISOString();
}

// API functions
export async function fetchHealth(): Promise<string> {
  const response = await fetch('/api/health');
  return response.text();
}

export async function fetchHistorical(request: HistoricalRequest): Promise<HistoricalResponse> {
  const response = await fetch('/api/historical', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to fetch historical data');
  }

  return response.json();
}

// WebSocket connection for live data
export function connectLive(
  symbols: string[],
  schema: string,
  onMessage: (msg: LiveMessage) => void,
  onError: (error: Event) => void,
  onClose: () => void
): WebSocket {
  const params = new URLSearchParams({
    symbols: symbols.join(','),
    schema,
  });

  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${window.location.host}/ws/live?${params}`;

  const ws = new WebSocket(wsUrl);

  ws.onmessage = (event) => {
    try {
      const msg = JSON.parse(event.data) as LiveMessage;
      onMessage(msg);
    } catch (e) {
      console.error('Failed to parse WebSocket message:', e);
    }
  };

  ws.onerror = onError;
  ws.onclose = onClose;

  return ws;
}
