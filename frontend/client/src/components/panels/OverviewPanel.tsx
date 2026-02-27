/**
 * OverviewPanel - 市场总览面板
 * Design: 暗夜终端 - 综合仪表盘，指数卡片 + 行情列表
 */
import { useState } from 'react';
import { useMarketOverview, useQuotes, useSectors, formatPrice, formatPercent, formatNumber, getChangeColor } from '@/hooks/useMarketData';
import { LayoutDashboard, TrendingUp, TrendingDown, BarChart3, Activity } from 'lucide-react';
import { useStockClick } from '@/pages/Dashboard';

type SortKey = 'name' | 'price' | 'change_pct' | 'turnover' | 'turnover_rate' | 'pe_ratio';
type SortDir = 'asc' | 'desc';

function IndexCard({ name, price, change, change_pct, volume, turnover, onClick }: {
  name: string; price: number; change: number; change_pct: number; volume: number; turnover: number; onClick?: () => void;
}) {
  const isUp = change_pct > 0;
  const isDown = change_pct < 0;
  return (
    <div
      className={`rounded-lg p-3.5 panel-glow border border-border/50 ${
      isUp ? 'bg-gradient-to-br from-red-950/60 to-red-900/30 border-red-800/30'
      : isDown ? 'bg-gradient-to-br from-green-950/60 to-green-900/30 border-green-800/30'
      : 'bg-card'
    } ${onClick ? 'cursor-pointer hover:border-primary/60 hover:shadow-lg transition-colors' : ''}`}
      onClick={onClick}
    >
      <div className="flex items-center justify-between mb-1.5">
        <span className="text-xs text-gray-400 font-medium">{name}</span>
        {isUp ? <TrendingUp className="w-3.5 h-3.5 text-red-400" /> : isDown ? <TrendingDown className="w-3.5 h-3.5 text-green-400" /> : null}
      </div>
      <div className={`font-mono-data text-xl font-bold ${isUp ? 'text-red-400' : isDown ? 'text-green-400' : 'text-gray-200'}`}>
        {formatPrice(price)}
      </div>
      <div className="flex items-center gap-2 mt-1.5">
        <span className={`font-mono-data text-xs ${isUp ? 'text-red-400/80' : isDown ? 'text-green-400/80' : 'text-gray-400'}`}>
          {change > 0 ? '+' : ''}{change.toFixed(2)}
        </span>
        <span className={`font-mono-data text-xs font-semibold px-1.5 py-0.5 rounded ${
          isUp ? 'bg-red-500/20 text-red-300' : isDown ? 'bg-green-500/20 text-green-300' : 'text-gray-400'
        }`}>
          {formatPercent(change_pct)}
        </span>
      </div>
    </div>
  );
}

export default function OverviewPanel() {
  const { data: overview } = useMarketOverview();
  const { data: quotes } = useQuotes(1, 100);
  const { data: sectors } = useSectors();
  const { openStock } = useStockClick();

  const [sortKey, setSortKey] = useState<SortKey>('change_pct');
  const [sortDir, setSortDir] = useState<SortDir>('desc');

  const handleSort = (key: SortKey) => {
    if (key === sortKey) {
      setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortKey(key);
      setSortDir(key === 'name' ? 'asc' : 'desc');
    }
  };

  const sortedQuotes = (quotes ?? []).slice().sort((a, b) => {
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
      case 'pe_ratio':
        av = a.pe_ratio;
        bv = b.pe_ratio;
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
    <div className="p-4 space-y-4">
      {/* Header */}
      <div className="flex items-center gap-2 pb-2 border-b border-border">
        <LayoutDashboard className="w-4 h-4 text-primary" />
        <h2 className="text-sm font-semibold">市场总览</h2>
      </div>

      {/* Index Cards */}
      {overview && (
        <div className="grid grid-cols-3 gap-3">
          <IndexCard
            {...overview.sh_index}
            onClick={() => openStock(`${overview.sh_index.code}.SH`, overview.sh_index.name)}
          />
          <IndexCard
            {...overview.sz_index}
            onClick={() => openStock(`${overview.sz_index.code}.SZ`, overview.sz_index.name)}
          />
          <IndexCard
            {...overview.cyb_index}
            onClick={() => openStock(`${overview.cyb_index.code}.SZ`, overview.cyb_index.name)}
          />
        </div>
      )}

      {/* Market Stats */}
      {overview && (
        <div className="grid grid-cols-4 gap-2">
          {[
            { label: '涨停', value: overview.limit_up_count, color: 'text-up' },
            { label: '跌停', value: overview.limit_down_count, color: 'text-down' },
            { label: '上涨', value: overview.up_count, color: 'text-up' },
            { label: '下跌', value: overview.down_count, color: 'text-down' },
          ].map((stat) => (
            <div key={stat.label} className="bg-card rounded-lg p-2.5 border border-border/50 text-center">
              <div className="text-[10px] text-muted-foreground mb-0.5">{stat.label}</div>
              <div className={`font-mono-data text-sm font-bold ${stat.color}`}>
                {stat.value || '—'}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Top Sectors */}
      {sectors && sectors.length > 0 && (
        <div>
          <h3 className="text-xs font-semibold text-muted-foreground mb-2 flex items-center gap-1.5">
            <Activity className="w-3 h-3" /> 热门板块
          </h3>
          <div className="grid grid-cols-2 gap-2">
            {sectors.slice(0, 6).map((s) => (
              <div key={s.code} className="bg-card rounded-lg px-3 py-2 border border-border/50 flex items-center justify-between">
                <span className="text-xs font-medium truncate">{s.name}</span>
                <span className={`font-mono-data text-xs font-medium ${getChangeColor(s.change_pct)}`}>
                  {formatPercent(s.change_pct)}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Quotes Table */}
      <div>
        <h3 className="text-xs font-semibold text-muted-foreground mb-2 flex items-center gap-1.5">
          <BarChart3 className="w-3 h-3" /> 全市场行情
          <span className="text-[10px] normal-case font-normal ml-1">（点击查看详情）</span>
        </h3>
        <table className="w-full text-xs">
          <thead className="sticky top-0 bg-card/95 z-10">
            <tr className="text-muted-foreground border-b border-border">
              <th className="text-left py-1.5 px-2 font-medium">
                <button
                  type="button"
                  onClick={() => handleSort('name')}
                  className="inline-flex items-center gap-0.5 hover:text-foreground"
                >
                  股票
                  {sortKey === 'name' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
                </button>
              </th>
              <th className="text-right py-1.5 px-2 font-medium">
                <button
                  type="button"
                  onClick={() => handleSort('price')}
                  className="inline-flex items-center gap-0.5 hover:text-foreground"
                >
                  最新价
                  {sortKey === 'price' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
                </button>
              </th>
              <th className="text-right py-1.5 px-2 font-medium">
                <button
                  type="button"
                  onClick={() => handleSort('change_pct')}
                  className="inline-flex items-center gap-0.5 hover:text-foreground"
                >
                  涨跌幅
                  {sortKey === 'change_pct' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
                </button>
              </th>
              <th className="text-right py-1.5 px-2 font-medium">
                <button
                  type="button"
                  onClick={() => handleSort('turnover')}
                  className="inline-flex items-center gap-0.5 hover:text-foreground"
                >
                  成交额
                  {sortKey === 'turnover' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
                </button>
              </th>
              <th className="text-right py-1.5 px-2 font-medium">
                <button
                  type="button"
                  onClick={() => handleSort('turnover_rate')}
                  className="inline-flex items-center gap-0.5 hover:text-foreground"
                >
                  换手率
                  {sortKey === 'turnover_rate' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
                </button>
              </th>
              <th className="text-right py-1.5 px-2 font-medium">
                <button
                  type="button"
                  onClick={() => handleSort('pe_ratio')}
                  className="inline-flex items-center gap-0.5 hover:text-foreground"
                >
                  市盈率
                  {sortKey === 'pe_ratio' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
                </button>
              </th>
            </tr>
          </thead>
          <tbody>
            {sortedQuotes.map((q) => (
              <tr
                key={q.symbol}
                onClick={() => openStock(q.symbol, q.name)}
                className="border-b border-border/30 hover:bg-accent/50 transition-colors cursor-pointer"
              >
                <td className="py-1 px-2">
                  <span className="font-medium">{q.name}</span>
                  <span className="text-[10px] text-muted-foreground ml-1 font-mono-data">{q.symbol}</span>
                </td>
                <td className={`text-right py-1 px-2 font-mono-data ${getChangeColor(q.change_pct)}`}>
                  {formatPrice(q.price)}
                </td>
                <td className={`text-right py-1 px-2 font-mono-data font-medium ${getChangeColor(q.change_pct)}`}>
                  {formatPercent(q.change_pct)}
                </td>
                <td className="text-right py-1 px-2 font-mono-data text-muted-foreground">
                  {formatNumber(q.turnover)}
                </td>
                <td className="text-right py-1 px-2 font-mono-data text-muted-foreground">
                  {q.turnover_rate.toFixed(1)}%
                </td>
                <td className="text-right py-1 px-2 font-mono-data text-muted-foreground">
                  {q.pe_ratio > 0 ? q.pe_ratio.toFixed(1) : '—'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
