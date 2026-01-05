import { TradeRecord, formatPrice, formatTimestamp } from '../api';

interface TradeTapeProps {
  trades: TradeRecord[];
  title?: string;
}

export function TradeTape({ trades, title = 'Trades' }: TradeTapeProps) {
  if (trades.length === 0) {
    return (
      <div className="bg-gray-800 rounded-lg p-4">
        <h2 className="text-lg font-semibold text-white mb-4">{title}</h2>
        <p className="text-gray-500 text-center py-8">No trades to display</p>
      </div>
    );
  }

  return (
    <div className="bg-gray-800 rounded-lg overflow-hidden">
      <div className="px-4 py-3 border-b border-gray-700">
        <h2 className="text-lg font-semibold text-white">
          {title}
          <span className="text-gray-400 text-sm ml-2">({trades.length} records)</span>
        </h2>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full">
          <thead className="bg-gray-700/50">
            <tr>
              <th className="px-4 py-2 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                Time
              </th>
              <th className="px-4 py-2 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                Symbol
              </th>
              <th className="px-4 py-2 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">
                Price
              </th>
              <th className="px-4 py-2 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">
                Size
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-700">
            {trades.map((trade, idx) => (
              <TradeRow key={`${trade.ts_event_unix_ns}-${idx}`} trade={trade} />
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function TradeRow({ trade }: { trade: TradeRecord }) {
  return (
    <tr className="hover:bg-gray-700/30 transition-colors">
      <td className="px-4 py-2 whitespace-nowrap text-sm text-gray-300 font-mono">
        {formatTimestamp(trade.ts_event_unix_ns)}
      </td>
      <td className="px-4 py-2 whitespace-nowrap text-sm text-white font-medium">
        {trade.symbol}
      </td>
      <td className="px-4 py-2 whitespace-nowrap text-sm text-green-400 text-right font-mono">
        {formatPrice(trade.price_i64)}
      </td>
      <td className="px-4 py-2 whitespace-nowrap text-sm text-gray-300 text-right font-mono">
        {trade.size_u32}
      </td>
    </tr>
  );
}
