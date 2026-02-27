/**
 * AnomalyPanel - 异动股票检测面板
 * Design: 暗夜终端 - 告警式布局，异动类型标签
 */
import { useAnomalies, formatPrice, formatPercent, getChangeColor } from '@/hooks/useMarketData';
import { Zap, RefreshCw, AlertTriangle, TrendingUp, TrendingDown, ArrowUpCircle, ArrowDownCircle } from 'lucide-react';
import { useStockClick } from '@/pages/Dashboard';

function getAnomalyIcon(type: string) {
  switch (type) {
    case 'LimitUp': return <ArrowUpCircle className="w-3.5 h-3.5 text-up" />;
    case 'LimitDown': return <ArrowDownCircle className="w-3.5 h-3.5 text-down" />;
    case 'PriceSurge': return <TrendingUp className="w-3.5 h-3.5 text-up" />;
    case 'PriceDrop': return <TrendingDown className="w-3.5 h-3.5 text-down" />;
    case 'VolumeSpike': return <AlertTriangle className="w-3.5 h-3.5 text-warning" />;
    default: return <Zap className="w-3.5 h-3.5 text-info" />;
  }
}

function getAnomalyLabel(type: string): string {
  const labels: Record<string, string> = {
    VolumeSpike: '量能突增',
    PriceSurge: '急速拉升',
    PriceDrop: '急速下跌',
    LimitUp: '涨停',
    LimitDown: '跌停',
    LimitUpOpen: '涨停打开',
    LimitDownOpen: '跌停打开',
    LargeOrder: '大单异动',
    TurnoverSpike: '换手突增',
    GapUp: '跳空高开',
    GapDown: '跳空低开',
    BreakResistance: '突破压力',
    BreakSupport: '跌破支撑',
    BoardRush: '板块异动',
  };
  return labels[type] || type;
}

function getAnomalyBadgeColor(type: string): string {
  if (['LimitUp', 'PriceSurge', 'GapUp', 'BreakResistance'].includes(type)) {
    return 'bg-up/15 text-red-300';
  }
  if (['LimitDown', 'PriceDrop', 'GapDown', 'BreakSupport'].includes(type)) {
    return 'bg-down/15 text-green-300';
  }
  return 'bg-yellow-500/15 text-yellow-300';
}

export default function AnomalyPanel() {
  const { data: anomalies, loading, refetch } = useAnomalies();
  const { openStock } = useStockClick();

  const formatAnomalyTime = (ts?: string): string => {
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
          <Zap className="w-4 h-4 text-yellow-400" />
          <h2 className="text-sm font-semibold">异动检测</h2>
          <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
            {anomalies?.length || 0} 条
          </span>
        </div>
        <button onClick={refetch} className="text-muted-foreground hover:text-foreground transition-colors">
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {/* Anomaly List */}
      <div className="divide-y divide-border/50">
        {anomalies?.map((item, i) => (
          <div
            key={`${item.symbol}-${item.anomaly_type}-${i}`}
            onClick={() => openStock(item.symbol, item.name)}
            className="px-4 py-2.5 hover:bg-accent/50 transition-colors cursor-pointer"
          >
            <div className="flex items-center justify-between mb-1">
              <div className="flex items-center gap-2">
                {getAnomalyIcon(item.anomaly_type)}
                <span className="font-medium text-sm">{item.name}</span>
                <span className="text-[10px] text-muted-foreground font-mono-data">{item.symbol}</span>
              </div>
              <div className="flex items-center gap-2">
                <span className={`font-mono-data text-sm font-medium ${getChangeColor(item.change_pct)}`}>
                  {formatPercent(item.change_pct)}
                </span>
                <span className={`font-mono-data text-xs ${getChangeColor(item.change_pct)}`}>
                  {formatPrice(item.price)}
                </span>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span className={`text-[10px] px-1.5 py-0.5 rounded ${getAnomalyBadgeColor(item.anomaly_type)}`}>
                  {getAnomalyLabel(item.anomaly_type)}
                </span>
                <span className="text-[10px] text-muted-foreground">{item.description}</span>
                {item.timestamp && (
                  <span className="text-[10px] text-muted-foreground font-mono-data">
                    {formatAnomalyTime(item.timestamp)}
                  </span>
                )}
              </div>
              <div className="flex items-center gap-1">
                <div className="w-10 h-1 bg-muted rounded-full overflow-hidden">
                  <div
                    className="h-full rounded-full"
                    style={{
                      width: `${Math.min(item.anomaly_score, 100)}%`,
                      background: item.anomaly_score > 80
                        ? '#ef4444'
                        : item.anomaly_score > 50
                        ? '#f97316'
                        : '#3b82f6',
                    }}
                  />
                </div>
              </div>
            </div>
          </div>
        ))}
        {(!anomalies || anomalies.length === 0) && !loading && (
          <div className="text-center py-12 text-muted-foreground text-sm">
            <Zap className="w-8 h-8 mx-auto mb-2 opacity-30" />
            暂无异动（非交易时段）
          </div>
        )}
        {loading && (
          <div className="text-center py-8 text-muted-foreground">
            <RefreshCw className="w-4 h-4 animate-spin mx-auto" />
          </div>
        )}
      </div>
    </div>
  );
}
