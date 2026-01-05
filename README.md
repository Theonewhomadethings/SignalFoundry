# Market Data Viewer

A real-time market data viewer built with Rust (Axum) backend and React frontend. Displays historical and live market data from DataBento API.

## Features

- **Historical Data**: Fetch trades and OHLCV bars for any time range
- **Live Streaming**: Real-time trade updates via WebSocket
- **Mock Mode**: Works without a DataBento API key for development
- **Charting**: Interactive candlestick charts (TradingView lightweight-charts)
- **Trade Tape**: Real-time trade log display

## Quick Start

### Prerequisites

- Rust (1.70+)
- Node.js (18+)
- npm

### How to Run

create a .env and add api key or try export DATABENTO_API_KEY="your_api_key_here"
```bash
# Terminal 1: Start the backend
source .env
. ~/.cargo/env && cargo run -p backend

# Terminal 2: Start the frontend
cd ui
npm install
npm run dev
```

Then open http://localhost:5173

## Project Structure

```
market-data-viewer/
├── Cargo.toml              # Rust workspace
├── crates/
│   ├── shared/             # Shared types (API request/response)
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── backend/            # Axum server
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs           # Server entry point
│           ├── handlers.rs       # HTTP/WebSocket handlers
│           ├── service.rs        # MarketDataService trait
│           ├── mock_service.rs   # Mock implementation
│           └── databento_service.rs  # DataBento integration (stub)
├── ui/                     # React frontend
│   ├── package.json
│   ├── src/
│   │   ├── App.tsx
│   │   ├── api.ts          # API utilities
│   │   └── components/
│   │       ├── SymbolForm.tsx
│   │       ├── TradeTape.tsx
│   │       ├── HistoricalChart.tsx
│   │       └── LiveStream.tsx
│   └── ...
├── .env.example            # Environment template
└── README.md
```

## API Endpoints

### REST

- `GET /api/health` - Health check
- `POST /api/historical` - Fetch historical data

**Request:**
```json
{
  "symbols": ["ES.FUT"],
  "schema": "trades",
  "start_rfc3339": "2024-01-01T00:00:00Z",
  "end_rfc3339": "2024-01-01T01:00:00Z",
  "limit": 100
}
```

**Schema options:** `trades`, `ohlcv-1s`, `ohlcv-1m`

### WebSocket

- `GET /ws/live?symbols=ES.FUT&schema=trades` - Live data stream

## Configuration

| Environment Variable | Description | Default |
|---------------------|-------------|---------|
| `DATABENTO_API_KEY` | DataBento API key (optional) | Mock mode |
| `HOST` | Server host | `127.0.0.1` |
| `PORT` | Server port | `3001` |

## Development

### Backend

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --workspace --all-targets -- -D warnings

# Run tests
cargo test --workspace
```

### Frontend

```bash
cd ui

# Install dependencies
npm install

# Development server
npm run dev

# Build for production
npm run build
```

## Tech Stack

### Backend
- Rust 2021 edition
- Axum (HTTP/WebSocket server)
- Tokio (async runtime)
- Serde (JSON serialization)
- Tracing (logging)

### Frontend
- React 18
- TypeScript
- Vite
- Tailwind CSS
- TradingView lightweight-charts

## License

MIT
