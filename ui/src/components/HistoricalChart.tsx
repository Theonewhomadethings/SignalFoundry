import { useEffect, useRef } from 'react';
import { createChart, ColorType, CrosshairMode } from 'lightweight-charts';
import type { Time } from 'lightweight-charts';
import { OhlcvRecord, formatPriceNumber } from '../api';

interface HistoricalChartProps {
  data: OhlcvRecord[];
  schema: 'ohlcv-1s' | 'ohlcv-1m';
}

export function HistoricalChart({ data, schema }: HistoricalChartProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!containerRef.current || data.length === 0) return;

    // Create chart
    const chart = createChart(containerRef.current, {
      layout: {
        background: { type: ColorType.Solid, color: '#1f2937' },
        textColor: '#9ca3af',
      },
      grid: {
        vertLines: { color: '#374151' },
        horzLines: { color: '#374151' },
      },
      crosshair: {
        mode: CrosshairMode.Normal,
      },
      rightPriceScale: {
        borderColor: '#374151',
      },
      timeScale: {
        borderColor: '#374151',
        timeVisible: true,
        secondsVisible: schema === 'ohlcv-1s',
      },
      width: containerRef.current.clientWidth,
      height: 400,
    });

    // Add candlestick series (v5 API)
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const candlestickSeries = (chart as any).addCandlestickSeries({
      upColor: '#22c55e',
      downColor: '#ef4444',
      borderDownColor: '#ef4444',
      borderUpColor: '#22c55e',
      wickDownColor: '#ef4444',
      wickUpColor: '#22c55e',
    });

    // Convert data to lightweight-charts format
    const chartData = data.map((bar) => ({
      time: (bar.ts_event_unix_ns / 1e9) as Time, // Convert ns to seconds (Unix timestamp)
      open: formatPriceNumber(bar.open_i64),
      high: formatPriceNumber(bar.high_i64),
      low: formatPriceNumber(bar.low_i64),
      close: formatPriceNumber(bar.close_i64),
    }));

    // Sort by time ascending
    chartData.sort((a, b) => (a.time as number) - (b.time as number));

    candlestickSeries.setData(chartData);
    chart.timeScale().fitContent();

    // Handle resize
    const handleResize = () => {
      if (containerRef.current) {
        chart.applyOptions({
          width: containerRef.current.clientWidth,
        });
      }
    };

    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      chart.remove();
    };
  }, [data, schema]);

  if (data.length === 0) {
    return (
      <div className="bg-gray-800 rounded-lg p-4">
        <h2 className="text-lg font-semibold text-white mb-4">OHLCV Chart</h2>
        <p className="text-gray-500 text-center py-8">No OHLCV data to display</p>
      </div>
    );
  }

  const symbol = data[0]?.symbol || 'Unknown';
  const schemaLabel = schema === 'ohlcv-1s' ? '1-Second' : '1-Minute';

  return (
    <div className="bg-gray-800 rounded-lg overflow-hidden">
      <div className="px-4 py-3 border-b border-gray-700">
        <h2 className="text-lg font-semibold text-white">
          {symbol} - {schemaLabel} Chart
          <span className="text-gray-400 text-sm ml-2">({data.length} bars)</span>
        </h2>
      </div>
      <div ref={containerRef} className="p-2" />
    </div>
  );
}
