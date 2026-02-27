import { useState, useEffect, useCallback, useRef } from 'react';
import {
  generateMockOverview, generateMockQuotes, generateMockHotStocks,
  generateMockAnomalies, generateMockSectors, generateMockMoneyFlow, generateMockLimitUp,
} from './mockData';

// API base URL
// - In local dev: Vite proxies /api -> http://localhost:8080 (no CORS issues)
// - In production: set VITE_API_BASE to your backend URL
// - When VITE_API_BASE is empty, use relative path (works with Vite proxy)
const API_BASE = import.meta.env.VITE_API_BASE || '';

// Track if backend is available
let backendAvailable: boolean | null = null;

async function checkBackend(): Promise<boolean> {
  if (backendAvailable !== null) return backendAvailable;
  try {
    const res = await fetch(`${API_BASE}/api/health`, { signal: AbortSignal.timeout(2000) });
    backendAvailable = res.ok;
  } catch {
    backendAvailable = false;
  }
  return backendAvailable;
}

interface ApiResponse<T> {
  success: boolean;
  data: T;
  message: string;
}

// Generic fetch hook with mock fallback
function useApi<T>(endpoint: string, mockFn?: () => T, interval?: number) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isDemo, setIsDemo] = useState(false);

  const fetchData = useCallback(async () => {
    const available = await checkBackend();
    if (!available && mockFn) {
      setData(mockFn());
      setIsDemo(true);
      setLoading(false);
      return;
    }
    try {
      const res = await fetch(`${API_BASE}${endpoint}`);
      const json: ApiResponse<T> = await res.json();
      if (json.success) {
        setData(json.data);
        setError(null);
        setIsDemo(false);
      } else {
        setError(json.message);
        if (mockFn) { setData(mockFn()); setIsDemo(true); }
      }
    } catch (err) {
      if (mockFn) {
        setData(mockFn());
        setIsDemo(true);
      } else {
        setError(err instanceof Error ? err.message : 'Network error');
      }
    } finally {
      setLoading(false);
    }
  }, [endpoint, mockFn]);

  useEffect(() => {
    fetchData();
    if (interval) {
      const timer = setInterval(fetchData, interval);
      return () => clearInterval(timer);
    }
  }, [fetchData, interval]);

  return { data, loading, error, isDemo, refetch: fetchData };
}

// Market Overview
export interface IndexQuote {
  name: string;
  code: string;
  price: number;
  change: number;
  change_pct: number;
  volume: number;
  turnover: number;
}

export interface MarketOverview {
  sh_index: IndexQuote;
  sz_index: IndexQuote;
  cyb_index: IndexQuote;
  total_turnover: number;
  up_count: number;
  down_count: number;
  flat_count: number;
  limit_up_count: number;
  limit_down_count: number;
  timestamp: string;
}

export function useMarketOverview() {
  return useApi<MarketOverview>('/api/market/overview', generateMockOverview, 3000);
}

// Stock Quote
export interface StockQuote {
  symbol: string;
  name: string;
  price: number;
  change: number;
  change_pct: number;
  open: number;
  high: number;
  low: number;
  pre_close: number;
  volume: number;
  turnover: number;
  turnover_rate: number;
  amplitude: number;
  pe_ratio: number;
  total_market_cap: number;
  circulating_market_cap: number;
  timestamp: string;
}

export function useQuotes(page = 1, pageSize = 50) {
  return useApi<StockQuote[]>(`/api/quotes?page=${page}&page_size=${pageSize}`, generateMockQuotes, 5000);
}

// Hot Stocks
export interface HotStock {
  symbol: string;
  name: string;
  price: number;
  change_pct: number;
  volume: number;
  turnover: number;
  turnover_rate: number;
  hot_score: number;   // backend field
  score?: number;      // alias for compatibility
  hot_reason: string;
  reason?: string;     // alias for compatibility
  timestamp: string;
}

export function useHotStocks() {
  return useApi<HotStock[]>('/api/hot-stocks', generateMockHotStocks, 5000);
}

// Anomaly Stocks
export interface AnomalyStock {
  symbol: string;
  name: string;
  price: number;
  change_pct: number;
  anomaly_type: string;
  anomaly_score: number;
  description: string;
  volume: number;
  turnover_rate: number;
  timestamp: string;
}

export function useAnomalies() {
  return useApi<AnomalyStock[]>('/api/anomalies', generateMockAnomalies, 5000);
}

// Sectors
export interface SectorInfo {
  name: string;
  code: string;
  change_pct: number;
  turnover: number;
  leading_stock: string;
  leading_stock_pct: number;
  stock_count: number;
  up_count: number;
  down_count: number;
}

export function useSectors() {
  return useApi<SectorInfo[]>('/api/sectors', generateMockSectors, 30000);
}

// Money Flow
export interface MoneyFlow {
  symbol: string;
  name: string;
  main_net_inflow: number;
  super_large_inflow: number;
  large_inflow: number;
  medium_inflow: number;
  small_inflow: number;
  timestamp: string;
}

export function useMoneyFlow() {
  return useApi<MoneyFlow[]>('/api/money-flow', generateMockMoneyFlow, 30000);
}

// Candles
export interface Candle {
  symbol: string;
  timestamp: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  turnover: number;
}

export function useCandles(symbol: string, period = '1d', count = 120) {
  return useApi<Candle[]>(`/api/candles/${symbol}?period=${period}&count=${count}`);
}

// Backtest
export interface BacktestKpis {
  total_return: number;
  annual_return: number;
  max_drawdown: number;
  sharpe_ratio: number;
  sortino_ratio: number;
  win_rate: number;
  profit_loss_ratio: number;
  total_trades: number;
  winning_trades: number;
  losing_trades: number;
}

export interface BacktestResult {
  id: string;
  strategy_id: string;
  kpis: BacktestKpis;
  trades: any[];
  equity_curve: { timestamp: string; equity: number; benchmark: number }[];
  created_at: string;
}

export async function runBacktest(params: {
  symbol: string;
  period?: string;
  count?: number;
  short_ma?: number;
  long_ma?: number;
  initial_capital?: number;
}): Promise<ApiResponse<BacktestResult | null>> {
  const res = await fetch(`${API_BASE}/api/backtest`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(params),
  });
  return res.json();
}

// WebSocket hook
export function useWebSocket() {
  const [connected, setConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<any>(null);
  const [isDemo, setIsDemo] = useState(false);

  useEffect(() => {
    const tryConnect = async () => {
      const available = await checkBackend();
      if (!available) {
        setIsDemo(true);
        return;
      }
      const wsUrl = API_BASE.replace('http', 'ws') + '/ws';
      const ws = new WebSocket(wsUrl);

      ws.onopen = () => {
        setConnected(true);
        setIsDemo(false);
      };
      ws.onmessage = (event) => {
        try { setLastMessage(JSON.parse(event.data)); } catch {}
      };
      ws.onclose = () => setConnected(false);
      ws.onerror = () => { setConnected(false); setIsDemo(true); };

      return () => ws.close();
    };
    tryConnect();
  }, []);

  return { connected, lastMessage, isDemo };
}

// Search
export function useSearch(query: string) {
  return useApi<StockQuote[]>(`/api/search?q=${encodeURIComponent(query)}`);
}

// Limit Up
export function useLimitUp() {
  return useApi<StockQuote[]>('/api/limit-up', generateMockLimitUp, 30000);
}

// Watchlist
export function useWatchlist() {
  return useApi<any[]>('/api/watchlist', undefined, 30000);
}

// Format helpers
export function formatNumber(num: number, decimals = 2): string {
  if (num === 0 || isNaN(num)) return '—';
  if (Math.abs(num) >= 1e8) return (num / 1e8).toFixed(decimals) + '亿';
  if (Math.abs(num) >= 1e4) return (num / 1e4).toFixed(decimals) + '万';
  return num.toFixed(decimals);
}

export function formatPrice(price: number): string {
  if (price === 0 || isNaN(price)) return '—';
  return price.toFixed(2);
}

export function formatPercent(pct: number): string {
  if (isNaN(pct)) return '—';
  const sign = pct > 0 ? '+' : '';
  return `${sign}${pct.toFixed(2)}%`;
}

export function getChangeColor(value: number): string {
  if (value > 0) return 'text-up';
  if (value < 0) return 'text-down';
  return 'text-muted-foreground';
}

export function getChangeBg(value: number): string {
  if (value > 0) return 'bg-up';
  if (value < 0) return 'bg-down';
  return '';
}
