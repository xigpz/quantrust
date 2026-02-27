/**
 * SectorsPanel - 板块行情面板
 * Design: 暗夜终端 - 板块排行表格
 */
import { useSectors, formatPercent, formatNumber, getChangeColor } from '@/hooks/useMarketData';
import { PieChart, RefreshCw } from 'lucide-react';

export default function SectorsPanel() {
  const { data: sectors, loading, refetch } = useSectors();

  return (
    <div className="flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border sticky top-0 bg-card z-10">
        <div className="flex items-center gap-2">
          <PieChart className="w-4 h-4 text-purple-400" />
          <h2 className="text-sm font-semibold">板块行情</h2>
          <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
            {sectors?.length || 0} 个
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
            <th className="text-left py-2 px-3 font-medium">板块名称</th>
            <th className="text-right py-2 px-2 font-medium">涨跌幅</th>
            <th className="text-right py-2 px-2 font-medium">成交额</th>
            <th className="text-left py-2 px-2 font-medium">领涨股</th>
            <th className="text-right py-2 px-3 font-medium">涨/跌</th>
          </tr>
        </thead>
        <tbody>
          {sectors?.map((sector) => (
            <tr key={sector.code} className="border-b border-border/50 hover:bg-accent/50 transition-colors cursor-pointer">
              <td className="py-1.5 px-3 font-medium">{sector.name}</td>
              <td className={`text-right py-1.5 px-2 font-mono-data font-medium ${getChangeColor(sector.change_pct)}`}>
                {formatPercent(sector.change_pct)}
              </td>
              <td className="text-right py-1.5 px-2 font-mono-data text-muted-foreground">
                {formatNumber(sector.turnover)}
              </td>
              <td className="py-1.5 px-2">
                <div className="flex items-center gap-1">
                  <span className="text-foreground">{sector.leading_stock}</span>
                  <span className={`font-mono-data ${getChangeColor(sector.leading_stock_pct)}`}>
                    {formatPercent(sector.leading_stock_pct)}
                  </span>
                </div>
              </td>
              <td className="text-right py-1.5 px-3">
                <span className="text-up font-mono-data">{sector.up_count}</span>
                <span className="text-muted-foreground mx-0.5">/</span>
                <span className="text-down font-mono-data">{sector.down_count}</span>
              </td>
            </tr>
          ))}
          {(!sectors || sectors.length === 0) && !loading && (
            <tr>
              <td colSpan={5} className="text-center py-8 text-muted-foreground">暂无数据</td>
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
