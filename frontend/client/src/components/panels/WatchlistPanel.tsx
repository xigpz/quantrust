/**
 * WatchlistPanel - 自选股面板
 */
import { useWatchlist, formatPrice, formatPercent, getChangeColor } from '@/hooks/useMarketData';
import { Star, RefreshCw, Plus } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { toast } from 'sonner';

export default function WatchlistPanel() {
  const { data: watchlist, loading, refetch } = useWatchlist();

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
                <th className="text-left py-2 px-3 font-medium">股票</th>
                <th className="text-right py-2 px-2 font-medium">代码</th>
                <th className="text-right py-2 px-3 font-medium">分组</th>
              </tr>
            </thead>
            <tbody>
              {watchlist.map((item: any) => (
                <tr key={item.id} className="border-b border-border/50 hover:bg-accent/50 transition-colors cursor-pointer">
                  <td className="py-1.5 px-3 font-medium">{item.name}</td>
                  <td className="text-right py-1.5 px-2 font-mono-data text-muted-foreground">{item.symbol}</td>
                  <td className="text-right py-1.5 px-3 text-muted-foreground">{item.group_name}</td>
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
