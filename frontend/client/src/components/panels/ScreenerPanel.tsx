/**
 * ScreenerPanel - 选股器面板
 */
import { useState } from 'react';
import { Filter, RefreshCw, Search, TrendingUp, TrendingDown } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { toast } from 'sonner';

interface ScreenerReq {
  min_pe?: number;
  max_pe?: number;
  min_pb?: number;
  max_pb?: number;
  min_roe?: number;
  min_growth?: number;
  min_volume?: number;
  change_pct_min?: number;
  limit?: number;
}

interface StockQuote {
  symbol: string;
  name: string;
  price: number;
  change: number;
  change_pct: number;
  volume: number;
  turnover: number;
  pe_ratio: number;
}

function formatNumber(num: number): string {
  if (num >= 1e8) return (num / 1e8).toFixed(2) + '亿';
  if (num >= 1e4) return (num / 1e4).toFixed(2) + '万';
  return num.toFixed(0);
}

export default function ScreenerPanel() {
  const [filters, setFilters] = useState<ScreenerReq>({
    min_pe: 0,
    max_pe: 50,
    change_pct_min: 0,
    min_volume: 10000000,
    limit: 30,
  });
  
  const [results, setResults] = useState<StockQuote[]>([]);
  const [loading, setLoading] = useState(false);
  const [hasSearched, setHasSearched] = useState(false);

  const updateFilter = (key: keyof ScreenerReq, value: number | undefined) => {
    setFilters(prev => ({ ...prev, [key]: value }));
  };

  const runScreener = async () => {
    setLoading(true);
    setHasSearched(true);
    try {
      const res = await fetch('/api/screener', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(filters),
      });
      const data = await res.json();
      if (data.success) {
        setResults(data.data);
        toast.success(`找到 ${data.data.length} 只股票`);
      } else {
        toast.error(data.message);
      }
    } catch (e) {
      toast.error('选股失败');
    }
    setLoading(false);
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-2 border-border">
        <div className="flex items-center gap-2">
          <Filter className="w-4 h-4 text-purple-400" />
          <h2 className="text-sm font-medium">选股器</h2>
        </div>
        <button onClick={runScreener} disabled={loading} className="text-muted-foreground hover:text-foreground transition-colors">
          <Search className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {/* 筛选条件 */}
      <div className="px-4 py-3 border-b border-border space-y-3">
        <div className="grid grid-cols-2 gap-3">
          {/* 市盈率 */}
          <div>
            <label className="text-xs text-muted-foreground">市盈率 (PE)</label>
            <div className="flex gap-1 mt-1">
              <input
                type="number"
                placeholder="最小"
                value={filters.min_pe ?? ''}
                onChange={(e) => updateFilter('min_pe', e.target.value ? parseFloat(e.target.value) : undefined)}
                className="w-full bg-background border border-border rounded px-2 py-1 text-xs"
              />
              <input
                type="number"
                placeholder="最大"
                value={filters.max_pe ?? ''}
                onChange={(e) => updateFilter('max_pe', e.target.value ? parseFloat(e.target.value) : undefined)}
                className="w-full bg-background border border-border rounded px-2 py-1 text-xs"
              />
            </div>
          </div>

          {/* 涨跌幅 */}
          <div>
            <label className="text-xs text-muted-foreground">涨跌幅 ≥</label>
            <div className="flex items-center mt-1">
              <input
                type="number"
                step="0.1"
                value={filters.change_pct_min ?? ''}
                onChange={(e) => updateFilter('change_pct_min', e.target.value ? parseFloat(e.target.value) : undefined)}
                className="w-full bg-background border border-border rounded px-2 py-1 text-xs"
              />
              <span className="ml-1 text-xs">%</span>
            </div>
          </div>

          {/* 成交量 */}
          <div>
            <label className="text-xs text-muted-foreground">成交量 ≥</label>
            <div className="flex items-center mt-1">
              <input
                type="number"
                value={(filters.min_volume ?? 0) / 10000}
                onChange={(e) => updateFilter('min_volume', e.target.value ? parseFloat(e.target.value) * 10000 : undefined)}
                className="w-full bg-background border border-border rounded px-2 py-1 text-xs"
              />
              <span className="ml-1 text-xs">万</span>
            </div>
          </div>

          {/* 返回数量 */}
          <div>
            <label className="text-xs text-muted-foreground">返回数量</label>
            <select
              value={filters.limit ?? 30}
              onChange={(e) => updateFilter('limit', parseInt(e.target.value))}
              className="w-full mt-1 bg-background border border-border rounded px-2 py-1 text-xs"
            >
              <option value={10}>10只</option>
              <option value={30}>30只</option>
              <option value={50}>50只</option>
              <option value={100}>100只</option>
            </select>
          </div>
        </div>

        <button
          onClick={runScreener}
          disabled={loading}
          className="w-full bg-purple-600 hover:bg-purple-700 text-white py-2 px-4 rounded flex items-center justify-center gap-2 disabled:opacity-50"
        >
          <Filter className="w-4 h-4" />
          {loading ? '选股中...' : '开始选股'}
        </button>
      </div>

      <ScrollArea className="flex-1">
        {!hasSearched ? (
          <div className="p-4 text-center text-muted-foreground text-sm">
            设置筛选条件后点击"开始选股"
          </div>
        ) : results.length === 0 ? (
          <div className="p-4 text-center text-muted-foreground text-sm">
            没有符合条件的股票
          </div>
        ) : (
          <div className="p-2">
            <table className="w-full text-xs">
              <thead className="sticky top-0 bg-card z-10">
                <tr className="text-muted-foreground border-b border-border">
                  <th className="text-left py-2 px-2 font-medium">股票</th>
                  <th className="text-right py-2 px-2 font-medium">现价</th>
                  <th className="text-right py-2 px-2 font-medium">涨跌幅</th>
                  <th className="text-right py-2 px-2 font-medium">成交量</th>
                </tr>
              </thead>
              <tbody>
                {results.map((stock, idx) => (
                  <tr key={idx} className="border-b border-border/50 hover:bg-accent/50 transition-colors cursor-pointer">
                    <td className="py-2 px-2">
                      <div className="font-medium">{stock.name}</div>
                      <div className="text-muted-foreground text-xs">{stock.symbol}</div>
                    </td>
                    <td className="text-right py-2 px-2 font-mono">
                      {stock.price > 0 ? stock.price.toFixed(2) : '—'}
                    </td>
                    <td className={`text-right py-2 px-2 font-mono flex items-center justify-end gap-1 ${stock.change_pct >= 0 ? 'text-up' : 'text-down'}`}>
                      {stock.change_pct >= 0 ? <TrendingUp className="w-3 h-3" /> : <TrendingDown className="w-3 h-3" />}
                      {stock.change_pct > 0 ? '+' : ''}{stock.change_pct.toFixed(2)}%
                    </td>
                    <td className="text-right py-2 px-2 font-mono text-muted-foreground">
                      {formatNumber(stock.volume)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
