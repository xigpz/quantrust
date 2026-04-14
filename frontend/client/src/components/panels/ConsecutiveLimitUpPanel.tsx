/**
 * ConsecutiveLimitUpPanel - 连板天梯面板
 * Design: 暗夜终端 - 连续涨停股票排行榜
 */
import { useState } from 'react';
import { useConsecutiveLimitUp, formatPrice, formatPercent, formatNumber } from '@/hooks/useMarketData';
import { Trophy, RefreshCw } from 'lucide-react';
import { useStockClick } from '@/pages/Dashboard';

type SortKey = 'name' | 'price' | 'change_pct' | 'consecutive_days' | 'turn_rate' | 'amount';
type SortDir = 'asc' | 'desc';

// 根据连板天数获取颜色
function getConsecutiveBadgeStyle(days: number): { bg: string; text: string } {
  if (days >= 6) return { bg: 'bg-purple-500/20', text: 'text-purple-400' }; // 妖股
  if (days >= 5) return { bg: 'bg-yellow-500/20', text: 'text-yellow-400' }; // 金色
  if (days >= 4) return { bg: 'bg-red-500/20', text: 'text-red-400' };      // 红色
  if (days >= 3) return { bg: 'bg-orange-500/20', text: 'text-orange-400' }; // 橙色
  if (days >= 2) return { bg: 'bg-blue-500/20', text: 'text-blue-400' };     // 蓝色
  return { bg: 'bg-gray-500/20', text: 'text-gray-400' };
}

// 获取连板天数显示文本
function getConsecutiveLabel(days: number): string {
  if (days >= 6) return '妖股';
  if (days >= 5) return '五连板';
  if (days >= 4) return '四连板';
  if (days >= 3) return '三连板';
  if (days >= 2) return '二连板';
  return '首板';
}

export default function ConsecutiveLimitUpPanel() {
  const { data: stocks, loading, refetch } = useConsecutiveLimitUp();
  const { openStock } = useStockClick();

  const [sortKey, setSortKey] = useState<SortKey>('consecutive_days');
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
      case 'consecutive_days':
        av = a.consecutive_days;
        bv = b.consecutive_days;
        break;
      case 'turn_rate':
        av = a.turn_rate;
        bv = b.turn_rate;
        break;
      case 'amount':
        av = a.amount;
        bv = b.amount;
        break;
    }
    if (typeof av === 'string' && typeof bv === 'string') {
      return sortDir === 'asc' ? av.localeCompare(bv) : bv.localeCompare(av);
    }
    const na = Number(av) || 0;
    const nb = Number(bv) || 0;
    return sortDir === 'asc' ? na - nb : nb - na;
  });

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border shrink-0">
        <div className="flex items-center gap-2">
          <Trophy className="w-4 h-4 text-yellow-400" />
          <h2 className="text-sm font-semibold">连板天梯</h2>
          <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
            {stocks?.length || 0} 只
          </span>
        </div>
        <button onClick={refetch} className="text-muted-foreground hover:text-foreground transition-colors">
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {/* Empty State */}
      {!loading && (!stocks || stocks.length === 0) && (
        <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground py-12">
          <Trophy className="w-12 h-12 mb-3 opacity-30" />
          <p className="text-sm">暂无连板股票</p>
          <p className="text-xs mt-1 opacity-60">连板数据需要每日积累</p>
        </div>
      )}

      {/* Loading State */}
      {loading && (
        <div className="flex-1 flex items-center justify-center">
          <RefreshCw className="w-6 h-6 animate-spin text-muted-foreground" />
        </div>
      )}

      {/* Table */}
      {!loading && stocks && stocks.length > 0 && (
        <div className="flex-1 overflow-auto -mx-4 px-4">
          <table className="w-full text-xs min-w-[700px]">
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
            <th className="text-center py-2 px-2 font-medium">
              连板
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
                onClick={() => handleSort('turn_rate')}
                className="inline-flex items-center gap-0.5 hover:text-foreground"
              >
                换手率
                {sortKey === 'turn_rate' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
              </button>
            </th>
            <th className="text-right py-2 px-2 font-medium">
              <button
                type="button"
                onClick={() => handleSort('amount')}
                className="inline-flex items-center gap-0.5 hover:text-foreground"
              >
                成交额
                {sortKey === 'amount' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
              </button>
            </th>
          </tr>
        </thead>
        <tbody>
          {sorted.map((stock, index) => {
            const badgeStyle = getConsecutiveBadgeStyle(stock.consecutive_days);
            return (
              <tr
                key={stock.symbol}
                onClick={() => openStock(stock.symbol, stock.name)}
                className="border-b border-border/50 hover:bg-accent/50 transition-colors cursor-pointer"
              >
                <td className="py-1.5 px-3">
                  <div className="flex flex-col">
                    <div className="flex items-center gap-1.5">
                      <span className="text-[10px] text-muted-foreground">{index + 1}</span>
                      <span className="font-medium text-up">{stock.name}</span>
                    </div>
                    <span className="text-[10px] text-muted-foreground font-mono-data">
                      {stock.symbol}
                    </span>
                    {stock.reason && (
                      <span className="text-[9px] text-muted-foreground/70 mt-0.5">
                        {stock.reason}
                      </span>
                    )}
                  </div>
                </td>
                <td className="py-1.5 px-2 text-center">
                  <span className={`inline-flex items-center justify-center px-1.5 py-0.5 rounded text-[10px] font-medium ${badgeStyle.bg} ${badgeStyle.text}`}>
                    {stock.consecutive_days}板
                  </span>
                  <div className="text-[9px] text-muted-foreground mt-0.5">
                    {getConsecutiveLabel(stock.consecutive_days)}
                  </div>
                </td>
                <td className="text-right py-1.5 px-2 font-mono-data font-medium text-up">
                  {formatPrice(stock.price)}
                </td>
                <td className="text-right py-1.5 px-2 font-mono-data font-medium text-up">
                  {formatPercent(stock.change_pct)}
                </td>
                <td className="text-right py-1.5 px-2 font-mono-data text-muted-foreground">
                  {stock.turn_rate.toFixed(1)}%
                </td>
                <td className="text-right py-1.5 px-2 font-mono-data text-muted-foreground">
                  {formatNumber(stock.amount)}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
        </div>
      )}
    </div>
  );
}
