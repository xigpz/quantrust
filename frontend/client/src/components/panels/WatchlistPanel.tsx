/**
 * WatchlistPanel - 自选股面板
 */
import { useState } from 'react';
import { useWatchlist, removeFromWatchlist, formatPrice, formatPercent, formatNumber, getChangeColor } from '@/hooks/useMarketData';
import { useStockClick } from '@/pages/Dashboard';
import { Star, RefreshCw, Plus, X } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { toast } from 'sonner';

interface WatchlistItem {
  id: string;
  symbol: string;
  name: string;
  group_name: string;
  added_at: string;
  price?: number;
  change?: number;
  change_pct?: number;
  volume?: number;
  turnover?: number;
  turnover_rate?: number;
  sector_name?: string;
}

export default function WatchlistPanel() {
  const { data: watchlist, loading, refetch } = useWatchlist();
  const { openStock } = useStockClick();
  const [removingId, setRemovingId] = useState<string | null>(null);

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border">
        <div className="flex items-center gap-2">
          <Star className="w-4 h-4 text-yellow-400" />
          <h2 className="text-sm font-semibold">自选股</h2>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => toast('搜索股票后可添加到自选', { description: '功能开发中' })}
            className="text-muted-foreground hover:text-foreground transition-colors"
          >
            <Plus className="w-3.5 h-3.5" />
          </button>
          <button onClick={refetch} className="text-muted-foreground hover:text-foreground transition-colors">
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      <ScrollArea className="flex-1">
        {watchlist && watchlist.length > 0 ? (
          <table className="w-full text-xs">
            <thead className="sticky top-0 bg-card z-10">
              <tr className="text-muted-foreground border-b border-border">
                <th className="text-left py-2 px-2 font-medium">股票</th>
                <th className="text-left py-2 px-2 font-medium">板块</th>
                <th className="text-right py-2 px-2 font-medium">现价</th>
                <th className="text-right py-2 px-2 font-medium">涨跌幅</th>
                <th className="text-right py-2 px-2 font-medium">换手率</th>
                <th className="text-right py-2 px-2 font-medium">成交额</th>
                <th className="w-10 py-2 px-1 font-medium" />
              </tr>
            </thead>
            <tbody>
              {watchlist.map((item: WatchlistItem) => (
                <tr
                  key={item.id}
                  className="border-b border-border/50 hover:bg-accent/50 transition-colors cursor-pointer"
                  onClick={() => openStock(item.symbol, item.name)}
                >
                  <td className="py-1.5 px-2">
                    <div className="font-medium">{item.name}</div>
                    <div className="text-[10px] text-muted-foreground font-mono-data">{item.symbol}</div>
                  </td>
                  <td className="py-1.5 px-2 text-muted-foreground">
                    {item.sector_name || '—'}
                  </td>
                  <td className={`text-right py-1.5 px-2 font-mono-data ${item.price ? getChangeColor(item.change || 0) : 'text-muted-foreground'}`}>
                    {item.price != null ? formatPrice(item.price) : '—'}
                  </td>
                  <td className={`text-right py-1.5 px-2 font-mono-data ${item.change_pct ? getChangeColor(item.change_pct) : 'text-muted-foreground'}`}>
                    {item.change_pct != null ? formatPercent(item.change_pct) : '—'}
                  </td>
                  <td className={`text-right py-1.5 px-2 text-muted-foreground`}>
                    {item.turnover_rate != null ? `${item.turnover_rate.toFixed(2)}%` : '—'}
                  </td>
                  <td className="text-right py-1.5 px-2 text-muted-foreground">
                    {item.turnover != null ? formatNumber(item.turnover) : '—'}
                  </td>
                  <td className="text-right py-1.5 px-1">
                    <button
                      disabled={removingId === item.id}
                      onClick={(e) => {
                        e.stopPropagation();
                        (async () => {
                          try {
                            setRemovingId(item.id);
                            const res = await removeFromWatchlist(item.symbol);
                            if (res.success) {
                              toast.success('已移除自选股', { description: `${item.name} (${item.symbol})` });
                              refetch();
                            } else {
                              toast.error('移除自选失败', { description: res.message || '请稍后重试' });
                            }
                          } catch {
                            toast.error('移除自选失败', { description: '网络异常，请检查后端服务' });
                          } finally {
                            setRemovingId(null);
                          }
                        })();
                      }}
                      className="text-muted-foreground hover:text-destructive transition-colors p-1 rounded hover:bg-muted disabled:opacity-60"
                      title="移除自选股"
                    >
                      <X className="w-3.5 h-3.5" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <div className="text-center py-12 text-muted-foreground text-sm">
            <Star className="w-8 h-8 mx-auto mb-2 opacity-20" />
            <p>暂无自选股</p>
            <p className="text-xs mt-1">通过后端API添加自选股</p>
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
