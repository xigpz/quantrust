/**
 * HotStocksPanel - 热点股票监测面板
 * Design: 暗夜终端 - 紧凑数据表格，热度评分可视化，支持滚动刷新
 */
import { useEffect, useState, useRef, useCallback } from 'react';
import { useHotStocksPaged, formatPrice, formatPercent, formatNumber, getChangeColor } from '@/hooks/useMarketData';
import { Flame, RefreshCw } from 'lucide-react';
import { useStockClick } from '@/pages/Dashboard';

type SortKey = 'name' | 'price' | 'change_pct' | 'turnover' | 'turnover_rate' | 'hot_score';
type SortDir = 'asc' | 'desc';

export default function HotStocksPanel() {
  const PAGE_SIZE = 50;
  const [page, setPage] = useState(1);
  const [items, setItems] = useState<any[]>([]);
  const [lastPageSize, setLastPageSize] = useState<number>(0);
  const scrollRef = useRef<HTMLDivElement>(null);
  const isScrolling = useRef(false);

  const { data: hotStocks, loading, refetch } = useHotStocksPaged(page, PAGE_SIZE);
  const { openStock } = useStockClick();

  const [sortKey, setSortKey] = useState<SortKey>('hot_score');
  const [sortDir, setSortDir] = useState<SortDir>('desc');

  useEffect(() => {
    const list = hotStocks ?? [];
    setLastPageSize(list.length);
    if (page === 1) {
      setItems(list);
      return;
    }
    if (list.length === 0) return;
    setItems((prev) => {
      const seen = new Set(prev.map((x: any) => x.symbol));
      const next = [...prev];
      for (const x of list as any[]) {
        if (!seen.has(x.symbol)) {
          next.push(x);
          seen.add(x.symbol);
        }
      }
      return next;
    });
  }, [hotStocks, page]);

  const handleSort = (key: SortKey) => {
    if (key === sortKey) {
      setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortKey(key);
      setSortDir(key === 'name' ? 'asc' : 'desc');
    }
  };

  const sorted = (items ?? []).slice().sort((a, b) => {
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
      case 'hot_score':
        av = a.hot_score;
        bv = b.hot_score;
        break;
    }
    if (typeof av === 'string' && typeof bv === 'string') {
      return sortDir === 'asc' ? av.localeCompare(bv) : bv.localeCompare(av);
    }
    const na = Number(av) || 0;
    const nb = Number(bv) || 0;
    return sortDir === 'asc' ? na - nb : nb - na;
  });

  const hasMore = lastPageSize === PAGE_SIZE;

  // 滚动刷新
  const handleScroll = useCallback(() => {
    if (!scrollRef.current || loading || !hasMore || isScrolling.current) return;

    const { scrollTop, scrollHeight, clientHeight } = scrollRef.current;

    // 滚动到底部附近时刷新
    if (scrollTop + clientHeight >= scrollHeight - 100) {
      isScrolling.current = true;
      setPage((p) => p + 1);
      // 重置滚动状态
      setTimeout(() => {
        isScrolling.current = false;
      }, 1000);
    }

    // 滚动到顶部附近时刷新第一页
    if (scrollTop < 50) {
      isScrolling.current = true;
      setPage(1);
      setItems([]);
      refetch();
      setTimeout(() => {
        isScrolling.current = false;
      }, 1000);
    }
  }, [loading, hasMore, refetch]);

  return (
    <div className="flex flex-col h-full overflow-y-auto" ref={scrollRef} onScroll={handleScroll}>
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border sticky top-0 bg-card z-10 shrink-0">
        <div className="flex items-center gap-2">
          <Flame className="w-4 h-4 text-orange-400" />
          <h2 className="text-sm font-semibold">热点股票</h2>
          <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
            {items?.length || 0} 只
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            disabled={!hasMore || loading}
            onClick={() => setPage((p) => p + 1)}
            className="text-[10px] text-muted-foreground hover:text-foreground transition-colors disabled:opacity-50"
            title="加载更多"
          >
            加载更多
          </button>
          <button
            type="button"
            onClick={() => {
              setPage(1);
              setItems([]);
              refetch();
            }}
            className="text-muted-foreground hover:text-foreground transition-colors"
            title="刷新"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>
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
                最新价
                {sortKey === 'price' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
              </button>
            </th>
            <th className="text-right py-2 px-2 font-medium">
              <button
                type="button"
                onClick={() => handleSort('change_pct')}
                className="inline-flex items-center gap-0.5 hover:text-foreground"
              >
                涨跌幅
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
            <th className="text-right py-2 px-2 font-medium">
              <button
                type="button"
                onClick={() => handleSort('turnover_rate')}
                className="inline-flex items-center gap-0.5 hover:text-foreground"
              >
                换手率
                {sortKey === 'turnover_rate' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
              </button>
            </th>
            <th className="text-left py-2 px-2 font-medium">
              所属板块
            </th>
            <th className="text-right py-2 px-3 font-medium">
              <button
                type="button"
                onClick={() => handleSort('hot_score')}
                className="inline-flex items-center gap-0.5 hover:text-foreground"
              >
                热度
                {sortKey === 'hot_score' && <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>}
              </button>
            </th>
          </tr>
        </thead>
        <tbody>
          {sorted.map((stock) => (
            <tr
              key={stock.symbol}
              onClick={() => openStock(stock.symbol, stock.name)}
              className="border-b border-border/50 hover:bg-muted/60 transition-colors cursor-pointer"
            >
              <td className="py-1.5 px-3">
                <div className="flex flex-col">
                  <span className="font-medium text-foreground">{stock.name}</span>
                  <span className="text-[10px] text-muted-foreground font-mono-data">{stock.symbol}</span>
                </div>
              </td>
              <td className={`text-right py-1.5 px-2 font-mono-data font-medium ${getChangeColor(stock.change_pct)}`}>
                {formatPrice(stock.price)}
              </td>
              <td className={`text-right py-1.5 px-2 font-mono-data font-medium ${getChangeColor(stock.change_pct)}`}>
                {formatPercent(stock.change_pct)}
              </td>
              <td className="text-right py-1.5 px-2 font-mono-data text-muted-foreground">
                {formatNumber(stock.turnover)}
              </td>
              <td className="text-right py-1.5 px-2 font-mono-data text-muted-foreground">
                {stock.turnover_rate.toFixed(1)}%
              </td>
              <td className="py-1.5 px-2">
                <div className="flex flex-col">
                  <span className="text-[10px] text-foreground truncate max-w-[80px]" title={stock.sector_name}>
                    {stock.sector_name || '-'}
                  </span>
                  <span className={`text-[9px] font-mono-data ${getChangeColor(stock.sector_change_pct)}`}>
                    {stock.sector_change_pct ? formatPercent(stock.sector_change_pct) : '-'}
                  </span>
                </div>
              </td>
              <td className="text-right py-1.5 px-3">
                <div className="flex items-center justify-end gap-1.5">
                  <div className="w-16 h-1.5 bg-muted rounded-full overflow-hidden">
                    <div
                      className="h-full rounded-full transition-all duration-500"
                      style={{
                        width: `${Math.min(stock.hot_score, 100)}%`,
                        background: stock.hot_score > 70
                          ? 'linear-gradient(90deg, #f97316, #ef4444)'
                          : stock.hot_score > 40
                          ? 'linear-gradient(90deg, #eab308, #f97316)'
                          : 'linear-gradient(90deg, #3b82f6, #6366f1)',
                      }}
                    />
                  </div>
                  <span className="font-mono-data text-[10px] text-muted-foreground w-6 text-right">
                    {stock.hot_score.toFixed(0)}
                  </span>
                </div>
              </td>
            </tr>
          ))}
          {(!items || items.length === 0) && !loading && (
            <tr>
              <td colSpan={7} className="text-center py-8 text-muted-foreground">
                暂无数据（非交易时段）
              </td>
            </tr>
          )}
          {loading && (
            <tr>
              <td colSpan={7} className="text-center py-8 text-muted-foreground">
                <RefreshCw className="w-4 h-4 animate-spin mx-auto" />
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
