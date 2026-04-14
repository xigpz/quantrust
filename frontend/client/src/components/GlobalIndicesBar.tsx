/**
 * GlobalIndicesBar - 全球指数展示条
 * 显示日经、韩股、美股、德国DAX等国际指数
 */
import { useState } from 'react';
import { useGlobalMarketOverview, formatPrice, formatPercent, getChangeColor } from '@/hooks/useMarketData';
import { TrendingUp, TrendingDown, Globe } from 'lucide-react';
import GlobalIndexDetailModal from '@/components/GlobalIndexDetailModal';

interface GlobalIndicesBarProps {
  className?: string;
}

export default function GlobalIndicesBar({ className = '' }: GlobalIndicesBarProps) {
  const { data, loading } = useGlobalMarketOverview();
  const [selectedIndex, setSelectedIndex] = useState<{ symbol: string; name: string } | null>(null);

  if (loading || !data) {
    return (
      <div className={`h-8 bg-card/50 border-b border-border flex items-center px-4 gap-6 ${className}`}>
        <Globe className="w-3 h-3 text-muted-foreground" />
        <span className="text-xs text-muted-foreground">加载全球指数...</span>
      </div>
    );
  }

  // 合并所有国际指数
  const allIndices = [
    ...data.us_indices.map(i => ({ ...i, region: '🇺🇸 美股' })),
    ...data.hk_indices.map(i => ({ ...i, region: '🇭🇰 港股' })),
    ...data.asia_indices.map(i => ({ ...i, region: '🌏 亚洲' })),
    ...data.eu_indices.map(i => ({ ...i, region: '🇪🇺 欧洲' })),
  ];

  if (allIndices.length === 0) {
    return null;
  }

  return (
    <div className={`bg-card/50 border-b border-border ${className}`}>
      <div className="flex items-center gap-1 px-4 py-1.5 overflow-x-auto scrollbar-thin">
        <Globe className="w-3 h-3 text-primary shrink-0 mr-1" />

        {allIndices.map((idx) => (
          <div
            key={idx.symbol}
            className="flex items-center gap-1.5 px-2 py-0.5 rounded hover:bg-muted/50 shrink-0 cursor-pointer"
            title={`${idx.region} ${idx.name}`}
            onClick={() => setSelectedIndex({ symbol: idx.symbol, name: idx.name })}
          >
            <span className="text-[10px] text-muted-foreground whitespace-nowrap">
              {idx.name}
            </span>
            <span className={`font-mono-data text-xs font-semibold ${getChangeColor(idx.change_pct)}`}>
              {formatPrice(idx.price)}
            </span>
            <span className={`font-mono-data text-[10px] font-medium ${getChangeColor(idx.change_pct)} flex items-center gap-0.5`}>
              {idx.change_pct > 0 ? (
                <TrendingUp className="w-2.5 h-2.5 text-up" />
              ) : idx.change_pct < 0 ? (
                <TrendingDown className="w-2.5 h-2.5 text-down" />
              ) : null}
              {formatPercent(idx.change_pct)}
            </span>
          </div>
        ))}
      </div>

      {/* 详情弹窗 */}
      <GlobalIndexDetailModal
        symbol={selectedIndex?.symbol || null}
        name={selectedIndex?.name || null}
        onClose={() => setSelectedIndex(null)}
      />
    </div>
  );
}
