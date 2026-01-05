import { useEffect, useRef, useState } from 'react';
import { connectLive, LiveMessage, TradeRecord } from '../api';

interface LiveStreamProps {
  symbols: string[];
  schema: string;
  onTrade: (trade: TradeRecord) => void;
  onDisconnect: () => void;
}

export function LiveStream({ symbols, schema, onTrade, onDisconnect }: LiveStreamProps) {
  const wsRef = useRef<WebSocket | null>(null);
  const [status, setStatus] = useState<'connecting' | 'connected' | 'error'>('connecting');
  const [messageCount, setMessageCount] = useState(0);

  useEffect(() => {
    setStatus('connecting');
    setMessageCount(0);

    const handleMessage = (msg: LiveMessage) => {
      if (msg.type === 'connected') {
        setStatus('connected');
      } else if (msg.type === 'trade') {
        setMessageCount((prev) => prev + 1);
        onTrade({
          ts_event_unix_ns: msg.ts_event_unix_ns,
          symbol: msg.symbol,
          price_i64: msg.price_i64,
          size_u32: msg.size_u32,
        });
      } else if (msg.type === 'error') {
        console.error('WebSocket error message:', msg.message);
        setStatus('error');
      }
    };

    const handleError = (error: Event) => {
      console.error('WebSocket error:', error);
      setStatus('error');
    };

    const handleClose = () => {
      console.log('WebSocket closed');
      onDisconnect();
    };

    const ws = connectLive(symbols, schema, handleMessage, handleError, handleClose);
    wsRef.current = ws;

    return () => {
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, [symbols, schema, onTrade, onDisconnect]);

  const statusColor = {
    connecting: 'bg-yellow-500',
    connected: 'bg-green-500',
    error: 'bg-red-500',
  }[status];

  const statusText = {
    connecting: 'Connecting...',
    connected: 'Connected',
    error: 'Error',
  }[status];

  return (
    <div className="mt-4 p-3 bg-gray-700/50 rounded-lg">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className={`w-2 h-2 rounded-full ${statusColor} animate-pulse`} />
          <span className="text-sm text-gray-300">{statusText}</span>
        </div>
        <span className="text-sm text-gray-400">{messageCount} msgs</span>
      </div>

      <div className="mt-2 text-xs text-gray-500">
        {symbols.join(', ')} | {schema}
      </div>
    </div>
  );
}
