import { useState, useCallback } from 'react';
import { SymbolForm } from './components/SymbolForm';
import { TradeTape } from './components/TradeTape';
import { HistoricalChart } from './components/HistoricalChart';
import { LiveStream } from './components/LiveStream';
import {
  fetchHistorical,
  HistoricalRequest,
  HistoricalResponse,
  TradeRecord,
  OhlcvRecord,
} from './api';

type Schema = 'trades' | 'ohlcv-1s' | 'ohlcv-1m';

function App() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [historicalData, setHistoricalData] = useState<HistoricalResponse | null>(null);
  const [isLiveConnected, setIsLiveConnected] = useState(false);
  const [liveTrades, setLiveTrades] = useState<TradeRecord[]>([]);
  const [currentSymbols, setCurrentSymbols] = useState<string[]>(['ES.FUT']);
  const [currentSchema, setCurrentSchema] = useState<Schema>('trades');

  const handleFetchHistorical = useCallback(async (request: HistoricalRequest) => {
    setLoading(true);
    setError(null);
    setHistoricalData(null);

    try {
      const data = await fetchHistorical(request);
      setHistoricalData(data);
      setCurrentSymbols(request.symbols);
      setCurrentSchema(request.schema);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to fetch data');
    } finally {
      setLoading(false);
    }
  }, []);

  const handleLiveConnect = useCallback((symbols: string[], schema: Schema) => {
    setCurrentSymbols(symbols);
    setCurrentSchema(schema);
    setLiveTrades([]);
    setIsLiveConnected(true);
  }, []);

  const handleLiveDisconnect = useCallback(() => {
    setIsLiveConnected(false);
  }, []);

  const handleLiveTrade = useCallback((trade: TradeRecord) => {
    setLiveTrades((prev) => {
      const updated = [trade, ...prev];
      // Keep last 100 trades
      return updated.slice(0, 100);
    });
  }, []);

  // Determine what to show in the main display area
  const showTradeTape = currentSchema === 'trades';
  const showChart = currentSchema === 'ohlcv-1s' || currentSchema === 'ohlcv-1m';

  const historicalTrades = historicalData?.schema === 'trades' ? historicalData.data : [];
  const historicalOhlcv =
    historicalData?.schema === 'ohlcv-1s' || historicalData?.schema === 'ohlcv-1m'
      ? (historicalData.data as OhlcvRecord[])
      : [];

  return (
    <div className="min-h-screen flex flex-col">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 px-6 py-4">
        <h1 className="text-2xl font-bold text-white">
          Market Data Viewer
          <span className="text-gray-400 text-sm ml-2">(DataBento)</span>
        </h1>
      </header>

      {/* Main content */}
      <main className="flex-1 flex flex-col lg:flex-row">
        {/* Controls sidebar */}
        <aside className="w-full lg:w-80 bg-gray-800 border-b lg:border-b-0 lg:border-r border-gray-700 p-4">
          <SymbolForm
            onFetchHistorical={handleFetchHistorical}
            onLiveConnect={handleLiveConnect}
            onLiveDisconnect={handleLiveDisconnect}
            isLiveConnected={isLiveConnected}
            isLoading={loading}
          />

          {/* Live stream component (handles WebSocket) */}
          {isLiveConnected && (
            <LiveStream
              symbols={currentSymbols}
              schema={currentSchema}
              onTrade={handleLiveTrade}
              onDisconnect={handleLiveDisconnect}
            />
          )}
        </aside>

        {/* Display area */}
        <section className="flex-1 p-4 overflow-auto">
          {/* Error display */}
          {error && (
            <div className="bg-red-900/50 border border-red-700 text-red-200 px-4 py-3 rounded mb-4">
              {error}
            </div>
          )}

          {/* Loading state */}
          {loading && (
            <div className="flex items-center justify-center h-64">
              <div className="text-gray-400">Loading...</div>
            </div>
          )}

          {/* Data display */}
          {!loading && !error && (
            <>
              {/* Trade tape for trades schema or live trades */}
              {showTradeTape && (
                <TradeTape
                  trades={isLiveConnected ? liveTrades : historicalTrades}
                  title={isLiveConnected ? 'Live Trades' : 'Historical Trades'}
                />
              )}

              {/* Chart for OHLCV data */}
              {showChart && historicalOhlcv.length > 0 && (
                <HistoricalChart
                  data={historicalOhlcv}
                  schema={currentSchema as 'ohlcv-1s' | 'ohlcv-1m'}
                />
              )}

              {/* Empty state */}
              {!isLiveConnected && !historicalData && (
                <div className="flex items-center justify-center h-64 text-gray-500">
                  <div className="text-center">
                    <p className="text-lg">No data to display</p>
                    <p className="text-sm mt-2">
                      Use the controls to fetch historical data or connect to live stream
                    </p>
                  </div>
                </div>
              )}
            </>
          )}
        </section>
      </main>

      {/* Footer */}
      <footer className="bg-gray-800 border-t border-gray-700 px-6 py-2 text-gray-500 text-sm">
        <div className="flex justify-between">
          <span>Mock Mode Active</span>
          <span>
            {currentSymbols.join(', ')} | {currentSchema}
          </span>
        </div>
      </footer>
    </div>
  );
}

export default App;
