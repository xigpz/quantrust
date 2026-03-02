/**
 * MarketBar - 顶部市场概览状态栏
 * Design: 暗夜终端风格 - 紧凑的大盘指数实时展示
 */
import { useMarketOverview, formatPrice, formatPercent, getChangeColor } from '@/hooks/useMarketData';
import { Activity, TrendingUp, TrendingDown, Wifi, WifiOff } from 'lucide-react';

interface MarketBarProps {
  wsConnected: boolean;
  isDemo?: boolean;
}

export default function MarketBar({ wsConnected, isDemo }: MarketBarProps) {
  const { data: overview } = useMarketOverview();

  const indices = overview ? [
    { ...overview.sh_index, label: '上证' },
    { ...overview.sz_index, label: '深证' },
    { ...overview.cyb_index, label: '创业板' },
  ] : [];

  return (
    <header className="h-10 bg-card border-b border-border flex items-center px-4 gap-6 shrink-0">
      {/* Logo */}
      <div className="flex items-center gap-2 mr-2">
        <Activity className="w-4 h-4 text-primary" />
        <span className="font-semibold text-sm tracking-tight">QuantRust</span>
      </div>

      {/* Index Quotes */}
      <div className="flex items-center gap-5">
        {indices.map((idx) => (
          <div key={idx.code} className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">{idx.label}</span>
            <span className={`font-mono-data text-sm font-medium ${getChangeColor(idx.change_pct)}`}>
              {formatPrice(idx.price)}
            </span>
            <span className={`font-mono-data text-xs ${getChangeColor(idx.change_pct)}`}>
              {formatPercent(idx.change_pct)}
            </span>
            {idx.change_pct > 0 ? (
              <TrendingUp className="w-3 h-3 text-up" />
            ) : idx.change_pct < 0 ? (
              <TrendingDown className="w-3 h-3 text-down" />
            ) : null}
          </div>
        ))}
      </div>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Market Stats */}
      {overview && (
        <div className="flex items-center gap-4 text-xs text-muted-foreground">
          <span>
            涨 <span className="text-up font-mono-data">{overview.up_count ?? '—'}</span>
          </span>
          <span>
            跌 <span className="text-down font-mono-data">{overview.down_count ?? '—'}</span>
          </span>
          <span>
            平 <span className="font-mono-data">{overview.flat_count ?? '—'}</span>
          </span>
        </div>
      )}

      {/* Demo badge */}
      {isDemo && (
        <span className="text-[9px] bg-yellow-500/20 text-yellow-400 px-1.5 py-0.5 rounded font-medium tracking-wide">
          DEMO
        </span>
      )}

      {/* Connection Status */}
      <div className="flex items-center gap-1.5">
        {wsConnected ? (
          <>
            <div className="w-1.5 h-1.5 rounded-full bg-green-500 pulse-dot" />
            <Wifi className="w-3 h-3 text-green-500" />
          </>
        ) : (
          <>
            <div className="w-1.5 h-1.5 rounded-full bg-yellow-500" />
            <WifiOff className="w-3 h-3 text-yellow-500" />
          </>
        )}
        <span className="text-[10px] text-muted-foreground ml-1">
          {new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
        </span>
      </div>
    </header>
  );
}
