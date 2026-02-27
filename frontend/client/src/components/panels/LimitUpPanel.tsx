/**
 * LimitUpPanel - 涨停监控面板
 * Design: 暗夜终端 - 涨停股监控
 */
import { useState } from 'react';
import { useLimitUp, formatPrice, formatPercent, formatNumber } from '@/hooks/useMarketData';
import { BarChart3, RefreshCw } from 'lucide-react';
import { useStockClick } from '@/pages/Dashboard';

type SortKey = 'name' | 'price' | 'change_pct' | 'turnover' | 'turnover_rate' | 'timestamp';
type SortDir = 'asc' | 'desc';

export default function LimitUpPanel() {
  const { data: stocks, loading, refetch } = useLimitUp();
  const { openStock } = useStockClick();

  const [sortKey, setSortKey] = useState<SortKey>('turnover');
  const [sortDir, setSortDir] = useState<SortDir>('desc');

  const handleSort = (key: SortKey) => {
    if (key === sortKey) {
      setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortKey(key);
      setSortDir(key === 'name' ? 'asc' : 'desc');
    }
  };

  const sorted = (stocks ?? []).slice().sort((a, b) => {
    let av: number | string = 0;
    let bv: number | string = 0;
    switch (sortKey) {
      case 'name':
        av = a.name;
        bv = b.name;
        break;
      case 'price':
        av = a.price;
        bv = b.price;
        break;
      case 'change_pct':
        av = a.change_pct;
        bv = b.change_pct;
        break;
      case 'turnover':
        av = a.turnover;
        bv = b.turnover;
        break;
      case 'turnover_rate':
        av = a.turnover_rate;
        bv = b.turnover_rate;
        break;
      case 'timestamp':
        av = a.timestamp ?? '';
        bv = b.timestamp ?? '';
        break;
    }
    if (typeof av === 'string' && typeof bv === 'string') {
      return sortDir === 'asc' ? av.localeCompare(bv) : bv.localeCompare(av);
    }
    const na = Number(av) || 0;
    const nb = Number(bv) || 0;
    return sortDir === 'asc' ? na - nb : nb - na;
  });

  const formatTime = (ts?: string): string => {
    if (!ts) return '';
    const d = new Date(ts);
    if (Number.isNaN(d.getTime())) return '';
    return d.toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

  return (
    <div className="flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border sticky top-0 bg-card z-10">
        <div className="flex items-center gap-2">
          <BarChart3 className="w-4 h-4 text-red-400" />
          <h2 className="text-sm font-semibold">涨停监控</h2>
          <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
            {stocks?.length || 0} 只
          </span>
        </div>
        <button onClick={refetch} className="text-muted-foreground hover:text-foreground transition-colors">
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {/* Table */}
      <table className="w-full text-xs">
        <thead className="sticky top-[41px] bg-card z-10">
          <tr className="text-muted-foreground border-b border-border">
            <th className="text-left py-2 px-3 font-medium">
              <button
                type="button"
                onClick={() => handleSort('name')}
                className="inline-flex items-center gap-0.5 hover:text-foreground"
              >
                股票
                {sortKey === 'name' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
              </button>
            </th>
            <th className="text-right py-2 px-2 font-medium">
              <button
                type="button"
                onClick={() => handleSort('price')}
                className="inline-flex items-center gap-0.5 hover:text-foreground"
              >
                涨停价
                {sortKey === 'price' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
              </button>
            </th>
            <th className="text-right py-2 px-2 font-medium">
              <button
                type="button"
                onClick={() => handleSort('change_pct')}
                className="inline-flex items-center gap-0.5 hover:text-foreground"
              >
                涨幅
                {sortKey === 'change_pct' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
              </button>
            </th>
            <th className="text-right py-2 px-2 font-medium">
              <button
                type="button"
                onClick={() => handleSort('turnover')}
                className="inline-flex items-center gap-0.5 hover:text-foreground"
              >
                成交额
                {sortKey === 'turnover' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
              </button>
            </th>
            <th className="text-right py-2 px-3 font-medium">
              <button
                type="button"
                onClick={() => handleSort('turnover_rate')}
                className="inline-flex items-center gap-0.5 hover:text-foreground"
              >
                换手率
                {sortKey === 'turnover_rate' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
              </button>
            </th>
          </tr>
        </thead>
        <tbody>
          {sorted.map((stock) => (
            <tr
              key={stock.symbol}
              onClick={() => openStock(stock.symbol, stock.name)}
              className="border-b border-border/50 hover:bg-accent/50 transition-colors cursor-pointer"
            >
              <td className="py-1.5 px-3">
                <div className="flex flex-col">
                  <span className="font-medium text-up">{stock.name}</span>
                  <span className="text-[10px] text-muted-foreground font-mono-data">
                    {stock.symbol}
                    {stock.timestamp && (
                      <span className="ml-1 text-[10px]">
                        {formatTime(stock.timestamp)}
                      </span>
                    )}
                  </span>
                </div>
              </td>
              <td className="text-right py-1.5 px-2 font-mono-data font-medium text-up">
                {formatPrice(stock.price)}
              </td>
              <td className="text-right py-1.5 px-2 font-mono-data font-medium text-up">
                {formatPercent(stock.change_pct)}
              </td>
              <td className="text-right py-1.5 px-2 font-mono-data text-muted-foreground">
                {formatNumber(stock.turnover)}
              </td>
              <td className="text-right py-1.5 px-3 font-mono-data text-muted-foreground">
                {stock.turnover_rate.toFixed(1)}%
              </td>
            </tr>
          ))}
          {(!stocks || stocks.length === 0) && !loading && (
            <tr>
              <td colSpan={5} className="text-center py-8 text-muted-foreground">暂无涨停股</td>
            </tr>
          )}
          {loading && (
            <tr>
              <td colSpan={5} className="text-center py-8 text-muted-foreground">
                <RefreshCw className="w-4 h-4 animate-spin mx-auto" />
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
