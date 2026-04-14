import { useState, useEffect, useCallback, useRef } from 'react';
import {
  generateMockOverview,
  generateMockQuotes,
  generateMockHotStocks,
  generateMockAnomalies,
  generateMockSectors,
  generateMockMoneyFlow,
  generateMockLimitUp,
  generateMockSectorIntradayFlow,
} from './mockData';

// API base URL
// - In local dev: Vite proxies /api -> http://localhost:8080 (no CORS issues)
// - In production: set VITE_API_BASE to your backend URL
// - When VITE_API_BASE is empty, use relative path (works with Vite proxy)
export const API_BASE = import.meta.env.VITE_API_BASE || '';

/**
 * 默认关闭：接口失败或后端不可达时不使用本地假数据，仅展示错误/空状态。
 * 无后端纯前端调试时可设 `VITE_USE_MOCK_FALLBACK=true`。
 */
const USE_MOCK_FALLBACK = import.meta.env.VITE_USE_MOCK_FALLBACK === 'true';

function optionalMock<T>(fn: () => T): (() => T) | undefined {
  return USE_MOCK_FALLBACK ? fn : undefined;
}

// Refresh interval hook with localStorage persistence
const STORAGE_KEY = 'quantrust_refresh_interval';

// Global refresh interval (in seconds)
let globalRefreshInterval = 15;

export function useRefreshInterval() {
  const [refreshInterval, setRefreshInterval] = useState<number>(() => {
    const saved = localStorage.getItem(STORAGE_KEY);
    globalRefreshInterval = saved ? parseInt(saved, 10) : 15;
    return globalRefreshInterval;
  });

  useEffect(() => {
    globalRefreshInterval = refreshInterval;
    localStorage.setItem(STORAGE_KEY, refreshInterval.toString());
  }, [refreshInterval]);

  return { refreshInterval, setRefreshInterval };
}

// Get global refresh interval in milliseconds (for use in hooks)
export function getRefreshIntervalMs(): number {
  return globalRefreshInterval * 1000;
}

/**
 * 后端可用性探测（带短 TTL）。
 * 此前用「只测一次并永久缓存」：若首屏时后端未起或超时，会整页会话一直走模拟数据，即使用户已开盘并启动后端。
 */
let lastBackendCheckAt = 0;
let lastBackendOk = false;
const BACKEND_CHECK_TTL_MS = 12_000;

async function checkBackend(force = false): Promise<boolean> {
  const now = Date.now();
  if (!force && lastBackendCheckAt !== 0 && now - lastBackendCheckAt < BACKEND_CHECK_TTL_MS) {
    return lastBackendOk;
  }
  try {
    const res = await fetch(`${API_BASE}/api/health`, { signal: AbortSignal.timeout(2500) });
    lastBackendOk = res.ok;
  } catch {
    lastBackendOk = false;
  } finally {
    lastBackendCheckAt = Date.now();
  }
  return lastBackendOk;
}

export interface ApiResponse<T> {
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

  const fetchData = useCallback(async (forceHealthCheck = false) => {
    const available = await checkBackend(forceHealthCheck);
    if (!available && mockFn) {
      setData(mockFn());
      setIsDemo(true);
      setLoading(false);
      return;
    }
    try {
      const res = await fetch(`${API_BASE}${endpoint}`);
      const bodyText = await res.text();

      const trimmed = bodyText.trim();
      if (!trimmed) {
        setError(res.ok ? '接口返回空响应' : `HTTP ${res.status} 空响应`);
        setData(null);
        setIsDemo(false);
        return;
      }

      let json: ApiResponse<T> | null = null;
      try {
        json = JSON.parse(trimmed) as ApiResponse<T>;
      } catch {
        // 常见原因：后端崩溃/代理返回了 HTML 或纯文本错误
        setError(
          res.ok
            ? `无法解析接口返回：${trimmed.slice(0, 200)}`
            : `HTTP ${res.status} 且返回非 JSON：${trimmed.slice(0, 200)}`
        );
        setData(null);
        setIsDemo(false);
        return;
      }

      if (json?.success) {
        setData(json.data);
        setError(null);
        setIsDemo(false);
      } else {
        setError(json?.message || (res.ok ? '接口返回失败' : `HTTP ${res.status}`));
        if (mockFn) {
          setData(mockFn());
          setIsDemo(true);
        } else {
          setData(null);
          setIsDemo(false);
        }
      }
    } catch (err) {
      if (mockFn) {
        setData(mockFn());
        setIsDemo(true);
      } else {
        setError(err instanceof Error ? err.message : 'Network error');
        setData(null);
        setIsDemo(false);
      }
    } finally {
      setLoading(false);
    }
  }, [endpoint, mockFn]);

  const refetch = useCallback(() => void fetchData(true), [fetchData]);

  useEffect(() => {
    void fetchData();
    if (interval) {
      const timer = setInterval(() => void fetchData(), interval);
      return () => clearInterval(timer);
    }
  }, [fetchData, interval]);

  return { data, loading, error, isDemo, refetch };
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
  return useApi<MarketQuote>('/api/market/overview', optionalMock(generateMockOverview), getRefreshIntervalMs());
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
  return useApi<StockQuote[]>(`/api/quotes?page=${page}&page_size=${pageSize}`, optionalMock(generateMockQuotes), getRefreshIntervalMs());
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
  return useApi<HotStock[]>('/api/hot-stocks?page=1&page_size=50', optionalMock(generateMockHotStocks), getRefreshIntervalMs());
}

export function useHotStocksPaged(page = 1, pageSize = 50) {
  const interval = page === 1 ? getRefreshIntervalMs() : undefined;
  return useApi<HotStock[]>(`/api/hot-stocks?page=${page}&page_size=${pageSize}`, optionalMock(generateMockHotStocks), interval);
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
  return useApi<AnomalyStock[]>('/api/anomalies', optionalMock(generateMockAnomalies), getRefreshIntervalMs());
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
  /** 主力净流入（亿元） */
  main_net_inflow?: number;
}

export function useSectors() {
  return useApi<SectorInfo[]>('/api/sectors', optionalMock(generateMockSectors), getRefreshIntervalMs());
}

export interface SectorIntradayPoint {
  t: string;
  v: number;
}

export interface SectorIntradaySeriesItem {
  code: string;
  name: string;
  points: SectorIntradayPoint[];
  last: number;
}

export interface SectorIntradayFlowResponse {
  trade_date: string;
  updated_at: string;
  series: SectorIntradaySeriesItem[];
}

/** 板块主力净流入分时走势（后端随行情扫描聚合；Demo 为模拟曲线） */
export function useSectorIntradayFlow() {
  return useApi<SectorIntradayFlowResponse>(
    '/api/sectors/intraday-flow',
    optionalMock(generateMockSectorIntradayFlow),
    getRefreshIntervalMs(),
  );
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
  return useApi<MoneyFlow[]>('/api/money-flow', optionalMock(generateMockMoneyFlow), getRefreshIntervalMs());
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
      const available = await checkBackend(true);
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
  return useApi<StockQuote[]>('/api/limit-up', optionalMock(generateMockLimitUp), getRefreshIntervalMs());
}

// Watchlist
export function useWatchlist() {
  return useApi<any[]>('/api/watchlist', undefined, getRefreshIntervalMs());
}

export async function addToWatchlist(params: { symbol: string; name: string; group_name?: string }): Promise<ApiResponse<string>> {
  const res = await fetch(`${API_BASE}/api/watchlist`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(params),
  });
  return res.json();
}

export async function removeFromWatchlist(symbol: string): Promise<ApiResponse<string>> {
  const res = await fetch(`${API_BASE}/api/watchlist/${encodeURIComponent(symbol)}`, {
    method: 'DELETE',
  });
  return res.json();
}

// Momentum Strategy
export interface MomentumSignal {
  score: number;
  rsi: number;
  macd_dif: number;
  macd_dea: number;
  macd_hist: number;
  reasons: string[];
}

export function useMomentum(symbol: string) {
  return useApi<MomentumSignal>(`/api/momentum/${encodeURIComponent(symbol)}`);
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

// ============ Global Market Types ============

export interface GlobalIndexQuote {
  symbol: string;
  name: string;
  price: number;
  change: number;
  change_pct: number;
  volume?: number;
  turnover?: number;
  timestamp?: string;
}

export interface CommodityQuote {
  name: string;
  symbol: string;
  price: number;
  change: number;
  change_pct: number;
  unit: string;
}

export interface ForexQuote {
  pair: string;
  price: number;
  change: number;
  change_pct: number;
}

export interface CryptoQuote {
  name: string;
  symbol: string;
  price: number;
  change_24h: number;
  market_cap: string;
  volume_24h: string;
}

export interface GlobalMarketOverview {
  us_indices: GlobalIndexQuote[];
  hk_indices: GlobalIndexQuote[];
  asia_indices: GlobalIndexQuote[];
  eu_indices: GlobalIndexQuote[];
  commodities: CommodityQuote[];
  forex: ForexQuote[];
  crypto: CryptoQuote[];
}

// Mock data for global market
function generateMockGlobalMarket(): GlobalMarketOverview {
  return {
    us_indices: [
      { symbol: '^GSPC', name: '标普500', price: 5234.18, change: 15.29, change_pct: 0.29 },
      { symbol: '^DJI', name: '道琼斯', price: 39127.80, change: 45.20, change_pct: 0.12 },
      { symbol: '^IXIC', name: '纳斯达克', price: 16277.46, change: -28.13, change_pct: -0.17 },
      { symbol: '^RUT', name: '罗素2000', price: 2068.32, change: 8.45, change_pct: 0.41 },
    ],
    hk_indices: [
      { symbol: '^HSI', name: '恒生指数', price: 17201.27, change: 128.70, change_pct: 0.75 },
      { symbol: '^HSCE', name: '国企指数', price: 5949.12, change: 42.36, change_pct: 0.72 },
      { symbol: '^HTECH', name: '恒生科技', price: 3429.58, change: -15.82, change_pct: -0.46 },
    ],
    asia_indices: [
      { symbol: '^N225', name: '日经225', price: 38923.89, change: 156.24, change_pct: 0.40 },
      { symbol: '^KS11', name: '韩国综合', price: 2743.82, change: -12.45, change_pct: -0.45 },
      { symbol: '^STI', name: '新加坡海峡', price: 3276.48, change: 5.12, change_pct: 0.16 },
    ],
    eu_indices: [
      { symbol: '^GDAXI', name: '德国DAX', price: 18567.23, change: 89.45, change_pct: 0.48 },
      { symbol: '^FCHI', name: '法国CAC', price: 8130.05, change: 32.18, change_pct: 0.40 },
      { symbol: '^FTSE', name: '英国富时', price: 8075.92, change: -15.33, change_pct: -0.19 },
    ],
    commodities: [
      { name: '黄金', symbol: 'GC=F', price: 2334.50, change: 8.20, change_pct: 0.35, unit: '美元/盎司' },
      { name: '白银', symbol: 'SI=F', price: 27.42, change: 0.18, change_pct: 0.66, unit: '美元/盎司' },
      { name: 'WTI原油', symbol: 'CL=F', price: 78.56, change: -0.82, change_pct: -1.03, unit: '美元/桶' },
      { name: '布伦特原油', symbol: 'BZ=F', price: 82.34, change: -0.65, change_pct: -0.78, unit: '美元/桶' },
    ],
    forex: [
      { pair: 'USD/CNY', price: 7.2456, change: -0.0023, change_pct: -0.03 },
      { pair: 'EUR/USD', price: 1.0834, change: 0.0012, change_pct: 0.11 },
      { pair: 'USD/JPY', price: 151.23, change: -0.34, change_pct: -0.22 },
    ],
    crypto: [
      { name: 'Bitcoin', symbol: 'BTC', price: 67842.50, change_24h: 2.34, market_cap: '$1.33T', volume_24h: '$28.5B' },
      { name: 'Ethereum', symbol: 'ETH', price: 3456.78, change_24h: 1.87, market_cap: '$415.2B', volume_24h: '$12.3B' },
      { name: 'Solana', symbol: 'SOL', price: 178.92, change_24h: 3.45, market_cap: '$78.9B', volume_24h: '$3.2B' },
    ],
  };
}

// Global Market Overview hook - fetches all global market data
export function useGlobalMarketOverview() {
  const [data, setData] = useState<GlobalMarketOverview | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    try {
      const available = await checkBackend();
      if (!available) {
        if (USE_MOCK_FALLBACK) {
          setData(generateMockGlobalMarket());
          setError(null);
        } else {
          setData(null);
          setError('无法连接后端，请确认服务已启动且 /api/health 可访问');
        }
        setLoading(false);
        return;
      }

      // 避免接口偶发返回空 body/非 JSON 导致 res.json() 直接抛错
      const safeReadJson = async <T,>(res: Response | null): Promise<T | null> => {
        if (!res || !res.ok) return null;
        const txt = await res.text();
        const trimmed = txt.trim();
        if (!trimmed) return null;
        try {
          return JSON.parse(trimmed) as T;
        } catch {
          return null;
        }
      };

      // Fetch all global market data in parallel
      const [usRes, hkRes, commoditiesRes, forexRes, cryptoRes] = await Promise.all([
        fetch(`${API_BASE}/api/global/indices`).catch(() => null),
        fetch(`${API_BASE}/api/global/hk/indices`).catch(() => null),
        fetch(`${API_BASE}/api/global/commodities`).catch(() => null),
        fetch(`${API_BASE}/api/global/forex`).catch(() => null),
        fetch(`${API_BASE}/api/global/crypto`).catch(() => null),
      ]);

      const [us_indices, hk_indices, commodities, forex, crypto] = await Promise.all([
        safeReadJson<any>(usRes),
        safeReadJson<any>(hkRes),
        safeReadJson<any>(commoditiesRes),
        safeReadJson<any>(forexRes),
        safeReadJson<any>(cryptoRes),
      ]);

      setData({
        us_indices: us_indices || [],
        hk_indices: hk_indices || [],
        asia_indices: [], // Not exposed by backend yet
        eu_indices: [],   // Not exposed by backend yet
        commodities: commodities?.data || commodities || [],
        forex: forex?.data || forex || [],
        crypto: crypto?.data || crypto || [],
      });
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch global market data');
      if (USE_MOCK_FALLBACK) {
        setData(generateMockGlobalMarket());
      } else {
        setData(null);
      }
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, getRefreshIntervalMs());
    return () => clearInterval(interval);
  }, [fetchData]);

  return { data, loading, error, refetch: fetchData };
}
