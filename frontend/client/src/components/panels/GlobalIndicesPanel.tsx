/**
 * GlobalIndicesPanel - 全球指数面板
 * 分区域展示：美股、港股、亚洲、欧洲指数
 */
import { useState } from 'react';
import { useGlobalMarketOverview, formatPrice, formatPercent, getChangeColor } from '@/hooks/useMarketData';
import { Globe, TrendingUp, TrendingDown, ChevronDown, ChevronRight, RefreshCw } from 'lucide-react';
import GlobalIndexDetailModal from '@/components/GlobalIndexDetailModal';

interface RegionGroup {
  key: string;
  name: string;
  icon: string;
  indices: typeof indices;
}

export default function GlobalIndicesPanel() {
  const { data, loading, refetch } = useGlobalMarketOverview();
  const [expandedRegions, setExpandedRegions] = useState<Record<string, boolean>>({
    us: true,
    hk: true,
    asia: true,
    eu: true,
  });
  const [selectedIndex, setSelectedIndex] = useState<{ symbol: string; name: string } | null>(null);

  const toggleRegion = (region: string) => {
    setExpandedRegions(prev => ({ ...prev, [region]: !prev[region] }));
  };

  if (loading && !data) {
    return (
      <div className="h-full flex items-center justify-center">
        <RefreshCw className="w-6 h-6 animate-spin text-primary" />
      </div>
    );
  }

  if (!data) {
    return (
      <div className="h-full flex items-center justify-center text-muted-foreground">
        暂无数据
      </div>
    );
  }

  const regions = [
    {
      key: 'us',
      name: '🇺🇸 美股',
      indices: data.us_indices,
      color: 'text-blue-400',
    },
    {
      key: 'hk',
      name: '🇭🇰 港股',
      indices: data.hk_indices,
      color: 'text-orange-400',
    },
    {
      key: 'asia',
      name: '🌏 亚洲',
      indices: data.asia_indices,
      color: 'text-yellow-400',
    },
    {
      key: 'eu',
      name: '🇪🇺 欧洲',
      indices: data.eu_indices,
      color: 'text-purple-400',
    },
  ];

  const commodities = data.commodities || [];

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b bg-card/50">
        <div className="flex items-center gap-2">
          <Globe className="w-4 h-4 text-primary" />
          <h2 className="text-sm font-semibold">全球指数</h2>
        </div>
        <button
          onClick={() => refetch()}
          className="p-1.5 rounded hover:bg-muted/50 transition-colors"
          title="刷新"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        {regions.map(region => {
          if (region.indices.length === 0) return null;

          return (
            <div key={region.key} className="bg-card/30 rounded-lg overflow-hidden">
              {/* Region Header */}
              <button
                onClick={() => toggleRegion(region.key)}
                className="w-full flex items-center justify-between px-3 py-2 hover:bg-muted/30 transition-colors"
              >
                <span className={`text-sm font-medium ${region.color}`}>
                  {region.name}
                </span>
                <div className="flex items-center gap-2">
                  <span className="text-xs text-muted-foreground">
                    {region.indices.length}个指数
                  </span>
                  {expandedRegions[region.key] ? (
                    <ChevronDown className="w-4 h-4 text-muted-foreground" />
                  ) : (
                    <ChevronRight className="w-4 h-4 text-muted-foreground" />
                  )}
                </div>
              </button>

              {/* Indices List */}
              {expandedRegions[region.key] && (
                <div className="border-t border-border/50">
                  {region.indices.map(index => (
                    <div
                      key={index.symbol}
                      className="flex items-center justify-between px-3 py-2 hover:bg-muted/20 transition-colors border-b border-border/30 last:border-b-0 cursor-pointer"
                      onClick={() => setSelectedIndex({ symbol: index.symbol, name: index.name })}
                    >
                      <div className="flex flex-col">
                        <span className="text-sm font-medium">{index.name}</span>
                        <span className="text-[10px] text-muted-foreground font-mono">
                          {index.symbol}
                        </span>
                      </div>
                      <div className="flex items-center gap-3">
                        <span className="font-mono-data text-sm font-semibold">
                          {formatPrice(index.price)}
                        </span>
                        <div className={`flex items-center gap-1 ${getChangeColor(index.change_pct)}`}>
                          {index.change_pct > 0 ? (
                            <TrendingUp className="w-3 h-3" />
                          ) : index.change_pct < 0 ? (
                            <TrendingDown className="w-3 h-3" />
                          ) : null}
                          <span className="font-mono-data text-xs font-medium w-16 text-right">
                            {formatPercent(index.change_pct)}
                          </span>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          );
        })}

        {/* 大宗商品：黄金、石油 */}
        {commodities.length > 0 && (
          <div className="bg-card/30 rounded-lg overflow-hidden">
            <div className="flex items-center justify-between px-3 py-2">
              <span className="text-sm font-medium text-amber-400">🛢️ 大宗商品</span>
              <span className="text-xs text-muted-foreground">
                {commodities.length}个品种
              </span>
            </div>
            <div className="border-t border-border/50">
              {commodities.map(commodity => (
                <div
                  key={commodity.symbol}
                  className="flex items-center justify-between px-3 py-2 hover:bg-muted/20 transition-colors border-b border-border/30 last:border-b-0"
                >
                  <div className="flex flex-col">
                    <span className="text-sm font-medium">{commodity.name}</span>
                    <span className="text-[10px] text-muted-foreground font-mono">
                      {commodity.unit}
                    </span>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className="font-mono-data text-sm font-semibold">
                      {formatPrice(commodity.price)}
                    </span>
                    <div className={`flex items-center gap-1 ${getChangeColor(commodity.change_pct)}`}>
                      {commodity.change_pct > 0 ? (
                        <TrendingUp className="w-3 h-3" />
                      ) : commodity.change_pct < 0 ? (
                        <TrendingDown className="w-3 h-3" />
                      ) : null}
                      <span className="font-mono-data text-xs font-medium w-16 text-right">
                        {formatPercent(commodity.change_pct)}
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Summary */}
        <div className="bg-primary/5 border border-primary/20 rounded-lg p-3">
          <div className="text-xs text-muted-foreground mb-2">市场情绪</div>
          <div className="flex items-center gap-2">
            {data.us_indices.length > 0 && (
              <span className="text-xs">
                美股:
                <span className={getChangeColor(
                  data.us_indices.reduce((sum, i) => sum + i.change_pct, 0) / data.us_indices.length
                )}>
                  {formatPercent(
                    data.us_indices.reduce((sum, i) => sum + i.change_pct, 0) / data.us_indices.length
                  )}
                </span>
              </span>
            )}
            {data.asia_indices.length > 0 && (
              <span className="text-xs">
                亚洲:
                <span className={getChangeColor(
                  data.asia_indices.reduce((sum, i) => sum + i.change_pct, 0) / data.asia_indices.length
                )}>
                  {formatPercent(
                    data.asia_indices.reduce((sum, i) => sum + i.change_pct, 0) / data.asia_indices.length
                  )}
                </span>
              </span>
            )}
            {data.eu_indices.length > 0 && (
              <span className="text-xs">
                欧洲:
                <span className={getChangeColor(
                  data.eu_indices.reduce((sum, i) => sum + i.change_pct, 0) / data.eu_indices.length
                )}>
                  {formatPercent(
                    data.eu_indices.reduce((sum, i) => sum + i.change_pct, 0) / data.eu_indices.length
                  )}
                </span>
              </span>
            )}
          </div>
        </div>
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
