/**
 * LimitUpPanel - 涨停监控面板
 * Design: 暗夜终端 - 涨停股监控
 */
import { useLimitUp, formatPrice, formatPercent, formatNumber } from '@/hooks/useMarketData';
import { BarChart3, RefreshCw } from 'lucide-react';
import { useStockClick } from '@/pages/Dashboard';

export default function LimitUpPanel() {
  const { data: stocks, loading, refetch } = useLimitUp();
  const { openStock } = useStockClick();

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
            <th className="text-left py-2 px-3 font-medium">股票</th>
            <th className="text-right py-2 px-2 font-medium">涨停价</th>
            <th className="text-right py-2 px-2 font-medium">涨幅</th>
            <th className="text-right py-2 px-2 font-medium">成交额</th>
            <th className="text-right py-2 px-3 font-medium">换手率</th>
          </tr>
        </thead>
        <tbody>
          {stocks?.map((stock) => (
            <tr
              key={stock.symbol}
              onClick={() => openStock(stock.symbol, stock.name)}
              className="border-b border-border/50 hover:bg-accent/50 transition-colors cursor-pointer"
            >
              <td className="py-1.5 px-3">
                <div className="flex flex-col">
                  <span className="font-medium text-up">{stock.name}</span>
                  <span className="text-[10px] text-muted-foreground font-mono-data">{stock.symbol}</span>
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
