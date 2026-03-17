export type StockDetailChartModeId =
  | 'pre'
  | 'time'
  | 'fiveDay'
  | 'dayK'
  | 'weekK'
  | 'monthK'
  | 'm5'
  | 'm15'
  | 'm30'
  | 'm60';

export type StockDetailChartKind = 'intraday' | 'kline';

export interface StockDetailChartMode {
  id: StockDetailChartModeId;
  label: string;
  kind: StockDetailChartKind;
  intradayRange?: 'pre' | '1d' | '5d';
  candlePeriod?: '1d' | '1w' | '1M' | '5m' | '15m' | '30m' | '60m';
  candleCount?: number;
}

export const STOCK_DETAIL_CHART_MODES: StockDetailChartMode[] = [
  { id: 'pre', label: '盘前', kind: 'intraday', intradayRange: 'pre' },
  { id: 'time', label: '分时', kind: 'intraday', intradayRange: '1d' },
  { id: 'fiveDay', label: '5日', kind: 'intraday', intradayRange: '5d' },
  { id: 'dayK', label: '日K', kind: 'kline', candlePeriod: '1d', candleCount: 240 },
  { id: 'weekK', label: '周K', kind: 'kline', candlePeriod: '1w', candleCount: 240 },
  { id: 'monthK', label: '月K', kind: 'kline', candlePeriod: '1M', candleCount: 240 },
  { id: 'm5', label: '5分', kind: 'kline', candlePeriod: '5m', candleCount: 240 },
  { id: 'm15', label: '15分', kind: 'kline', candlePeriod: '15m', candleCount: 240 },
  { id: 'm30', label: '30分', kind: 'kline', candlePeriod: '30m', candleCount: 240 },
  { id: 'm60', label: '60分', kind: 'kline', candlePeriod: '60m', candleCount: 240 },
];

export function getStockDetailChartMode(id: StockDetailChartModeId) {
  return STOCK_DETAIL_CHART_MODES.find((item) => item.id === id) ?? STOCK_DETAIL_CHART_MODES[1];
}
