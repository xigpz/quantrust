import { useState, useEffect, useCallback } from 'react';

const API_BASE = '';

// Types
export interface Portfolio {
  id: string;
  user_id: string;
  name: string;
  description?: string;
  initial_capital: number;
  current_capital: number;
  total_value: number;
  total_return_rate: number;
  positions_value: number;
  positions_count: number;
  created_at: string;
  updated_at: string;
}

export interface PortfolioPosition {
  id: string;
  portfolio_id: string;
  symbol: string;
  name: string;
  quantity: number;
  avg_cost: number;
  current_price: number;
  market_value: number;
  total_profit: number;
  profit_rate: number;
  weight: number;
  first_buy_date?: string;
  last_trade_date?: string;
}

export interface PortfolioTrade {
  id: string;
  portfolio_id: string;
  symbol: string;
  name: string;
  direction: 'buy' | 'sell';
  price: number;
  quantity: number;
  amount: number;
  commission: number;
  position_before?: number;
  position_after?: number;
  weight_before?: number;
  weight_after?: number;
  reason?: string;
  trade_date: string;
  trade_time: string;
}

export interface PortfolioStats {
  total_return: number;
  total_return_rate: number;
  daily_return: number;
  daily_return_rate: number;
  total_trades: number;
  win_trades: number;
  loss_trades: number;
  win_rate: number;
  max_drawdown: number;
  sharpe_ratio: number;
  // 增强指标
  annualized_return: number;
  volatility: number;
  benchmark_return: number;
  alpha: number;
  beta: number;
  position_concentration: number;
  win_streak: number;
  lose_streak: number;
}

export interface MonthlyReturn {
  year: number;
  month: number;
  start_value: number;
  end_value: number;
  monthly_return: number;
  monthly_return_rate: number;
  benchmark_return: number;
}

export interface AnnualReturn {
  year: number;
  start_value: number;
  end_value: number;
  annual_return: number;
  annual_return_rate: number;
  benchmark_return: number;
  trades_count: number;
  win_count: number;
}

export interface CreatePortfolioRequest {
  name: string;
  description?: string;
  initial_capital?: number;
}

export interface BuyStockRequest {
  symbol: string;
  name: string;
  price: number;
  quantity: number;
  reason?: string;
  trade_date?: string;
}

export interface SellStockRequest {
  symbol: string;
  price: number;
  quantity: number;
  reason?: string;
  trade_date?: string;
}

// Hooks
export function usePortfolios() {
  const [portfolios, setPortfolios] = useState<Portfolio[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchPortfolios = useCallback(async () => {
    setLoading(true);
    try {
      const resp = await fetch(`${API_BASE}/api/portfolios`);
      const json = await resp.json();
      if (json.success) {
        setPortfolios(json.data);
      } else {
        setError(json.message);
      }
    } catch (e) {
      setError('获取组合列表失败');
    } finally {
      setLoading(false);
    }
  }, []);

  const createPortfolio = async (req: CreatePortfolioRequest) => {
    try {
      const resp = await fetch(`${API_BASE}/api/portfolios`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(req),
      });
      const json = await resp.json();
      if (json.success) {
        await fetchPortfolios();
        return json.data as Portfolio;
      }
      throw new Error(json.message);
    } catch (e: any) {
      throw new Error(e.message || '创建组合失败');
    }
  };

  const deletePortfolio = async (id: string) => {
    try {
      const resp = await fetch(`${API_BASE}/api/portfolios/${id}`, {
        method: 'DELETE',
      });
      const json = await resp.json();
      if (json.success) {
        await fetchPortfolios();
        return true;
      }
      throw new Error(json.message);
    } catch (e: any) {
      throw new Error(e.message || '删除组合失败');
    }
  };

  useEffect(() => {
    fetchPortfolios();
  }, [fetchPortfolios]);

  return { portfolios, loading, error, refresh: fetchPortfolios, createPortfolio, deletePortfolio };
}

export function usePortfolioDetail(id: string | null) {
  const [portfolio, setPortfolio] = useState<Portfolio | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchPortfolio = useCallback(async () => {
    if (!id) return;
    setLoading(true);
    try {
      const resp = await fetch(`${API_BASE}/api/portfolios/${id}`);
      const json = await resp.json();
      if (json.success) {
        setPortfolio(json.data);
      }
    } catch (e) {
      console.error('获取组合详情失败:', e);
    } finally {
      setLoading(false);
    }
  }, [id]);

  useEffect(() => {
    fetchPortfolio();
  }, [fetchPortfolio]);

  return { portfolio, loading, refresh: fetchPortfolio };
}

export function usePortfolioPositions(id: string | null) {
  const [positions, setPositions] = useState<PortfolioPosition[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchPositions = useCallback(async () => {
    if (!id) return;
    setLoading(true);
    try {
      const resp = await fetch(`${API_BASE}/api/portfolios/${id}/positions`);
      const json = await resp.json();
      if (json.success) {
        setPositions(json.data);
      }
    } catch (e) {
      console.error('获取持仓失败:', e);
    } finally {
      setLoading(false);
    }
  }, [id]);

  useEffect(() => {
    fetchPositions();
    const interval = setInterval(fetchPositions, 30000); // 30秒刷新
    return () => clearInterval(interval);
  }, [fetchPositions]);

  return { positions, loading, refresh: fetchPositions };
}

export function usePortfolioTrades(id: string | null) {
  const [trades, setTrades] = useState<PortfolioTrade[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchTrades = useCallback(async () => {
    if (!id) return;
    setLoading(true);
    try {
      const resp = await fetch(`${API_BASE}/api/portfolios/${id}/trades?page_size=50`);
      const json = await resp.json();
      if (json.success) {
        setTrades(json.data);
      }
    } catch (e) {
      console.error('获取调仓记录失败:', e);
    } finally {
      setLoading(false);
    }
  }, [id]);

  useEffect(() => {
    fetchTrades();
  }, [fetchTrades]);

  return { trades, loading, refresh: fetchTrades };
}

export function usePortfolioStats(id: string | null) {
  const [stats, setStats] = useState<PortfolioStats | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchStats = useCallback(async () => {
    if (!id) return;
    setLoading(true);
    try {
      const resp = await fetch(`${API_BASE}/api/portfolios/${id}/stats`);
      const json = await resp.json();
      if (json.success) {
        setStats(json.data);
      }
    } catch (e) {
      console.error('获取统计信息失败:', e);
    } finally {
      setLoading(false);
    }
  }, [id]);

  useEffect(() => {
    fetchStats();
  }, [fetchStats]);

  return { stats, loading, refresh: fetchStats };
}

export function useBuyStock() {
  const buyStock = async (portfolioId: string, req: BuyStockRequest) => {
    const resp = await fetch(`${API_BASE}/api/portfolios/${portfolioId}/buy`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });
    const json = await resp.json();
    if (!json.success) {
      throw new Error(json.message);
    }
    return json.data as PortfolioTrade;
  };

  return { buyStock };
}

export function useSellStock() {
  const sellStock = async (portfolioId: string, req: SellStockRequest) => {
    const resp = await fetch(`${API_BASE}/api/portfolios/${portfolioId}/sell`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });
    const json = await resp.json();
    if (!json.success) {
      throw new Error(json.message);
    }
    return json.data as PortfolioTrade;
  };

  return { sellStock };
}

export function useMonthlyReturns(id: string | null) {
  const [monthlyReturns, setMonthlyReturns] = useState<MonthlyReturn[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchMonthlyReturns = useCallback(async () => {
    if (!id) return;
    setLoading(true);
    try {
      const resp = await fetch(`${API_BASE}/api/portfolios/${id}/returns/monthly`);
      const json = await resp.json();
      if (json.success) {
        setMonthlyReturns(json.data);
      }
    } catch (e) {
      console.error('获取月度收益失败:', e);
    } finally {
      setLoading(false);
    }
  }, [id]);

  useEffect(() => {
    fetchMonthlyReturns();
  }, [fetchMonthlyReturns]);

  return { monthlyReturns, loading, refresh: fetchMonthlyReturns };
}

export function useAnnualReturns(id: string | null) {
  const [annualReturns, setAnnualReturns] = useState<AnnualReturn[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchAnnualReturns = useCallback(async () => {
    if (!id) return;
    setLoading(true);
    try {
      const resp = await fetch(`${API_BASE}/api/portfolios/${id}/returns/annual`);
      const json = await resp.json();
      if (json.success) {
        setAnnualReturns(json.data);
      }
    } catch (e) {
      console.error('获取年度收益失败:', e);
    } finally {
      setLoading(false);
    }
  }, [id]);

  useEffect(() => {
    fetchAnnualReturns();
  }, [fetchAnnualReturns]);

  return { annualReturns, loading, refresh: fetchAnnualReturns };
}
