/**
 * FactorPanel - 因子库面板
 */
import { useState } from 'react';
import { BarChart2, RefreshCw, Search, TrendingUp, TrendingDown, Activity } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';

interface FactorData {
  pe: number;
  pb: number;
  ps: number;
  roe: number;
  roa: number;
  gross_margin: number;
  net_margin: number;
  revenue_growth: number;
  profit_growth: number;
  rsi_14: number;
  macd: number;
  volatility_20: number;
  beta: number;
  debt_ratio: number;
}

function FactorBar({ label, value, min, max, unit = '', goodDirection = 'high' }: {
  label: string;
  value: number;
  min: number;
  max: number;
  unit?: string;
  goodDirection?: 'high' | 'low';
}) {
  const percent = ((value - min) / (max - min)) * 100;
  const isGood = goodDirection === 'high' ? value > (min + max) / 2 : value < (min + max) / 2;
  
  return (
    <div className="mb-3">
      <div className="flex justify-between text-xs mb-1">
        <span className="text-muted-foreground">{label}</span>
        <span className={isGood ? 'text-green-400' : 'text-yellow-400'}>
          {value.toFixed(2)}{unit}
        </span>
      </div>
      <div className="h-2 bg-gray-700 rounded-full overflow-hidden">
        <div 
          className={`h-full ${isGood ? 'bg-green-500' : 'bg-yellow-500'}`}
          style={{ width: `${Math.min(Math.max(percent, 5), 100)}%` }}
        />
      </div>
    </div>
  );
}

export default function FactorPanel() {
  const [symbol, setSymbol] = useState('600519');
  const [searchInput, setSearchInput] = useState('600519');
  const [data, setData] = useState<FactorData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadData = async () => {
    if (!symbol.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`/api/factors/${encodeURIComponent(symbol)}`);
      const json = await res.json();
      if (json.success) {
        setData(json.data);
      } else {
        setError(json.message);
      }
    } catch (e) {
      setError('加载失败');
    }
    setLoading(false);
  };

  const handleSearch = () => {
    if (searchInput.trim()) {
      setSymbol(searchInput.trim().toUpperCase());
    }
  };

  // 自动加载
  useState(() => {
    loadData();
  });

  const getRSIColor = (rsi: number) => {
    if (rsi < 30) return 'text-green-400';
    if (rsi > 70) return 'text-red-400';
    return 'text-yellow-400';
  };

  const getRSILabel = (rsi: number) => {
    if (rsi < 30) return '超卖';
    if (rsi > 70) return '超买';
    return '中性';
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-2 border-b border-border">
        <div className="flex items-center gap-2">
          <Activity className="w-4 h-4 text-cyan-400" />
          <h2 className="text-sm font-medium">因子库</h2>
        </div>
        <button onClick={loadData} className="text-muted-foreground hover:text-foreground transition-colors">
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {/* 搜索框 */}
      <div className="px-4 py-3 border-b border-border">
        <div className="flex gap-2">
          <input
            type="text"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value.toUpperCase())}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="输入股票代码"
            className="flex-1 bg-background border border-border rounded px-3 py-1.5 text-sm focus:outline-none focus:ring-1 focus:ring-primary"
          />
          <button
            onClick={handleSearch}
            className="bg-primary text-primary-foreground px-3 py-1.5 rounded text-sm hover:bg-primary/90"
          >
            <Search className="w-4 h-4" />
          </button>
        </div>
      </div>

      <ScrollArea className="flex-1">
        {error ? (
          <div className="p-4 text-center text-muted-foreground text-sm">
            {error}
          </div>
        ) : !data ? (
          <div className="p-4 text-center text-muted-foreground text-sm">
            {loading ? '加载中...' : '输入股票代码查看因子数据'}
          </div>
        ) : (
          <div className="p-4 space-y-4">
            {/* 估值因子 */}
            <div className="bg-card rounded-lg p-4 border border-border">
              <div className="flex items-center gap-2 mb-3">
                <BarChart2 className="w-4 h-4 text-blue-400" />
                <span className="text-sm font-medium">估值因子</span>
              </div>
              <FactorBar label="市盈率 (PE)" value={data.pe} min={0} max={100} unit='倍' goodDirection="low" />
              <FactorBar label="市净率 (PB)" value={data.pb} min={0} max={20} unit='倍' goodDirection="low" />
              <FactorBar label="市销率 (PS)" value={data.ps} min={0} max={30} unit='倍' goodDirection="low" />
            </div>

            {/* 盈利因子 */}
            <div className="bg-card rounded-lg p-4 border border-border">
              <div className="flex items-center gap-2 mb-3">
                <TrendingUp className="w-4 h-4 text-green-400" />
                <span className="text-sm font-medium">盈利因子</span>
              </div>
              <FactorBar label="净资产收益率 (ROE)" value={data.roe} min={0} max={30} unit='%' goodDirection="high" />
              <FactorBar label="总资产收益率 (ROA)" value={data.roa} min={0} max={20} unit='%' goodDirection="high" />
              <FactorBar label="毛利率" value={data.gross_margin} min={0} max={80} unit='%' goodDirection="high" />
              <FactorBar label="净利率" value={data.net_margin} min={0} max={40} unit='%' goodDirection="high" />
            </div>

            {/* 成长因子 */}
            <div className="bg-card rounded-lg p-4 border border-border">
              <div className="flex items-center gap-2 mb-3">
                <TrendingUp className="w-4 h-4 text-purple-400" />
                <span className="text-sm font-medium">成长因子</span>
              </div>
              <FactorBar label="营收增长" value={data.revenue_growth} min={-50} max={100} unit='%' goodDirection="high" />
              <FactorBar label="净利润增长" value={data.profit_growth} min={-50} max={100} unit='%' goodDirection="high" />
            </div>

            {/* 技术因子 */}
            <div className="bg-card rounded-lg p-4 border border-border">
              <div className="flex items-center gap-2 mb-3">
                <Activity className="w-4 h-4 text-yellow-400" />
                <span className="text-sm font-medium">技术因子</span>
              </div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-xs text-muted-foreground">RSI (14)</span>
                <span className={`text-sm font-mono ${getRSIColor(data.rsi_14)}`}>
                  {data.rsi_14.toFixed(1)} - {getRSILabel(data.rsi_14)}
                </span>
              </div>
              <div className="h-2 bg-gray-700 rounded-full overflow-hidden mb-3">
                <div 
                  className={`h-full ${data.rsi_14 < 30 ? 'bg-green-500' : data.rsi_14 > 70 ? 'bg-red-500' : 'bg-yellow-500'}`}
                  style={{ width: `${Math.min(data.rsi_14, 100)}%` }}
                />
              </div>
              
              <div className="flex justify-between items-center mb-2">
                <span className="text-xs text-muted-foreground">MACD</span>
                <span className={`text-sm font-mono ${data.macd >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                  {data.macd >= 0 ? '+' : ''}{data.macd.toFixed(4)}
                </span>
              </div>
              
              <FactorBar label="20日波动率" value={data.volatility_20} min={0} max={10} unit='%' goodDirection="low" />
              <FactorBar label="Beta" value={data.beta} min={0.5} max={2} unit='' goodDirection="low" />
            </div>

            {/* 风险因子 */}
            <div className="bg-card rounded-lg p-4 border border-border">
              <div className="flex items-center gap-2 mb-3">
                <TrendingDown className="w-4 h-4 text-red-400" />
                <span className="text-sm font-medium">风险因子</span>
              </div>
              <FactorBar label="资产负债率" value={data.debt_ratio} min={0} max={100} unit='%' goodDirection="low" />
            </div>

            <div className="text-xs text-muted-foreground text-center">
              因子数据仅供参考，不构成投资建议
            </div>
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
