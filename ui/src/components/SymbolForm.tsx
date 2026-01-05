import { useState, FormEvent } from 'react';
import { HistoricalRequest } from '../api';

type Schema = 'trades' | 'ohlcv-1s' | 'ohlcv-1m';

interface SymbolFormProps {
  onFetchHistorical: (request: HistoricalRequest) => void;
  onLiveConnect: (symbols: string[], schema: Schema) => void;
  onLiveDisconnect: () => void;
  isLiveConnected: boolean;
  isLoading: boolean;
}

// Helper to get default time range (last hour)
function getDefaultTimeRange(): { start: string; end: string } {
  const now = new Date();
  const oneHourAgo = new Date(now.getTime() - 60 * 60 * 1000);

  // Format as datetime-local value (YYYY-MM-DDTHH:mm)
  const formatForInput = (date: Date) => {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    return `${year}-${month}-${day}T${hours}:${minutes}`;
  };

  return {
    start: formatForInput(oneHourAgo),
    end: formatForInput(now),
  };
}

export function SymbolForm({
  onFetchHistorical,
  onLiveConnect,
  onLiveDisconnect,
  isLiveConnected,
  isLoading,
}: SymbolFormProps) {
  const defaultRange = getDefaultTimeRange();

  const [symbols, setSymbols] = useState('ES.FUT');
  const [schema, setSchema] = useState<Schema>('trades');
  const [startTime, setStartTime] = useState(defaultRange.start);
  const [endTime, setEndTime] = useState(defaultRange.end);
  const [limit, setLimit] = useState(100);

  const handleFetchHistorical = (e: FormEvent) => {
    e.preventDefault();

    const symbolList = symbols
      .split(',')
      .map((s) => s.trim())
      .filter((s) => s.length > 0);

    if (symbolList.length === 0) {
      return;
    }

    // Convert local datetime to RFC3339
    const startRfc3339 = new Date(startTime).toISOString();
    const endRfc3339 = new Date(endTime).toISOString();

    onFetchHistorical({
      symbols: symbolList,
      schema,
      start_rfc3339: startRfc3339,
      end_rfc3339: endRfc3339,
      limit,
    });
  };

  const handleLiveToggle = () => {
    if (isLiveConnected) {
      onLiveDisconnect();
    } else {
      const symbolList = symbols
        .split(',')
        .map((s) => s.trim())
        .filter((s) => s.length > 0);

      if (symbolList.length > 0) {
        onLiveConnect(symbolList, schema);
      }
    }
  };

  return (
    <form onSubmit={handleFetchHistorical} className="space-y-4">
      <h2 className="text-lg font-semibold text-white mb-4">Query Parameters</h2>

      {/* Symbols */}
      <div>
        <label htmlFor="symbols" className="block text-sm font-medium text-gray-300 mb-1">
          Symbols (comma-separated)
        </label>
        <input
          id="symbols"
          type="text"
          value={symbols}
          onChange={(e) => setSymbols(e.target.value)}
          className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="ES.FUT, CL.FUT"
        />
      </div>

      {/* Schema */}
      <div>
        <label htmlFor="schema" className="block text-sm font-medium text-gray-300 mb-1">
          Schema
        </label>
        <select
          id="schema"
          value={schema}
          onChange={(e) => setSchema(e.target.value as Schema)}
          className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value="trades">Trades</option>
          <option value="ohlcv-1s">OHLCV 1-Second</option>
          <option value="ohlcv-1m">OHLCV 1-Minute</option>
        </select>
      </div>

      {/* Time range */}
      <div className="grid grid-cols-2 gap-2">
        <div>
          <label htmlFor="start" className="block text-sm font-medium text-gray-300 mb-1">
            Start
          </label>
          <input
            id="start"
            type="datetime-local"
            value={startTime}
            onChange={(e) => setStartTime(e.target.value)}
            className="w-full px-2 py-2 bg-gray-700 border border-gray-600 rounded-md text-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label htmlFor="end" className="block text-sm font-medium text-gray-300 mb-1">
            End
          </label>
          <input
            id="end"
            type="datetime-local"
            value={endTime}
            onChange={(e) => setEndTime(e.target.value)}
            className="w-full px-2 py-2 bg-gray-700 border border-gray-600 rounded-md text-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

      {/* Limit */}
      <div>
        <label htmlFor="limit" className="block text-sm font-medium text-gray-300 mb-1">
          Limit
        </label>
        <input
          id="limit"
          type="number"
          min={1}
          max={10000}
          value={limit}
          onChange={(e) => setLimit(parseInt(e.target.value) || 100)}
          className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>

      {/* Buttons */}
      <div className="flex gap-2 pt-2">
        <button
          type="submit"
          disabled={isLoading || isLiveConnected}
          className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white font-medium rounded-md transition-colors"
        >
          {isLoading ? 'Loading...' : 'Fetch Historical'}
        </button>
      </div>

      <div className="flex gap-2">
        <button
          type="button"
          onClick={handleLiveToggle}
          disabled={isLoading}
          className={`flex-1 px-4 py-2 font-medium rounded-md transition-colors ${
            isLiveConnected
              ? 'bg-red-600 hover:bg-red-700 text-white'
              : 'bg-green-600 hover:bg-green-700 text-white'
          } disabled:bg-gray-600 disabled:cursor-not-allowed`}
        >
          {isLiveConnected ? 'Disconnect' : 'Connect Live'}
        </button>
      </div>
    </form>
  );
}
