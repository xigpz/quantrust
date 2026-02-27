/**
 * HotStocksPanel - 热点股票监测面板
 * Design: 暗夜终端 - 紧凑数据表格，热度评分可视化
 */
import { useHotStocks, formatPrice, formatPercent, formatNumber, getChangeColor } from '@/hooks/useMarketData';
import { Flame, RefreshCw } from 'lucide-react';
import { useStockClick } from '@/pages/Dashboard';

export default function HotStocksPanel() {
  const { data: hotStocks, loading, refetch } = useHotStocks();
  const { openStock } = useStockClick();

  return (
    <div className="flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border sticky top-0 bg-card z-10">
        <div className="flex items-center gap-2">
          <Flame className="w-4 h-4 text-orange-400" />
          <h2 className="text-sm font-semibold">热点股票</h2>
          <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
            {hotStocks?.length || 0} 只
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
            <th className="text-left py-2 px-3 font-medium">股票</th>
            <th className="text-right py-2 px-2 font-medium">最新价</th>
            <th className="text-right py-2 px-2 font-medium">涨跌幅</th>
            <th className="text-right py-2 px-2 font-medium">成交额</th>
            <th className="text-right py-2 px-2 font-medium">换手率</th>
            <th className="text-right py-2 px-3 font-medium">热度</th>
          </tr>
        </thead>
        <tbody>
          {hotStocks?.map((stock) => (
            <tr
              key={stock.symbol}
              onClick={() => openStock(stock.symbol, stock.name)}
              className="border-b border-border/50 hover:bg-accent/50 transition-colors cursor-pointer"
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
          {(!hotStocks || hotStocks.length === 0) && !loading && (
            <tr>
              <td colSpan={6} className="text-center py-8 text-muted-foreground">
                暂无数据（非交易时段）
              </td>
            </tr>
          )}
          {loading && (
            <tr>
              <td colSpan={6} className="text-center py-8 text-muted-foreground">
                <RefreshCw className="w-4 h-4 animate-spin mx-auto" />
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
