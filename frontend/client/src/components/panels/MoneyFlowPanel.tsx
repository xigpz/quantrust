/**
 * MoneyFlowPanel - 资金流向面板
 * Design: 暗夜终端 - 资金流向排行
 */
import { useMoneyFlow, formatNumber, getChangeColor } from '@/hooks/useMarketData';
import { ArrowLeftRight, RefreshCw } from 'lucide-react';
import { useStockClick } from '@/pages/Dashboard';

export default function MoneyFlowPanel() {
  const { data: flows, loading, refetch } = useMoneyFlow();
  const { openStock } = useStockClick();

  return (
    <div className="flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border sticky top-0 bg-card z-10">
        <div className="flex items-center gap-2">
          <ArrowLeftRight className="w-4 h-4 text-cyan-400" />
          <h2 className="text-sm font-semibold">资金流向</h2>
          <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
            {flows?.length || 0} 只
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
            <th className="text-right py-2 px-2 font-medium">主力净流入</th>
            <th className="text-right py-2 px-2 font-medium">超大单</th>
            <th className="text-right py-2 px-2 font-medium">大单</th>
            <th className="text-right py-2 px-2 font-medium">中单</th>
            <th className="text-right py-2 px-3 font-medium">小单</th>
          </tr>
        </thead>
        <tbody>
          {flows?.map((flow) => (
            <tr
              key={flow.symbol}
              onClick={() => openStock(flow.symbol, flow.name)}
              className="border-b border-border/50 hover:bg-accent/50 transition-colors cursor-pointer"
            >
              <td className="py-1.5 px-3">
                <div className="flex flex-col">
                  <span className="font-medium">{flow.name}</span>
                  <span className="text-[10px] text-muted-foreground font-mono-data">{flow.symbol}</span>
                </div>
              </td>
              <td className={`text-right py-1.5 px-2 font-mono-data font-medium ${getChangeColor(flow.main_net_inflow)}`}>
                {formatNumber(flow.main_net_inflow)}
              </td>
              <td className={`text-right py-1.5 px-2 font-mono-data ${getChangeColor(flow.super_large_inflow)}`}>
                {formatNumber(flow.super_large_inflow)}
              </td>
              <td className={`text-right py-1.5 px-2 font-mono-data ${getChangeColor(flow.large_inflow)}`}>
                {formatNumber(flow.large_inflow)}
              </td>
              <td className={`text-right py-1.5 px-2 font-mono-data ${getChangeColor(flow.medium_inflow)}`}>
                {formatNumber(flow.medium_inflow)}
              </td>
              <td className={`text-right py-1.5 px-3 font-mono-data ${getChangeColor(flow.small_inflow)}`}>
                {formatNumber(flow.small_inflow)}
              </td>
            </tr>
          ))}
          {(!flows || flows.length === 0) && !loading && (
            <tr>
              <td colSpan={6} className="text-center py-8 text-muted-foreground">暂无数据</td>
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
