/**
 * SectorsPanel - 板块分析面板
 * Design: 暗夜终端 - 参考东方财富板块中心，包含板块排行、领涨股票、资金流向等
 */
import { useState, useMemo } from 'react';
import { useSectors, useHotStocks, formatPercent, formatNumber, getChangeColor, formatPrice } from '@/hooks/useMarketData';
import { useStockClick } from '@/pages/Dashboard';
import { PieChart, RefreshCw, X, ChevronRight, Loader2, TrendingUp, TrendingDown, ArrowUpDown, BarChart3, Activity } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';

const API_BASE = import.meta.env.VITE_API_BASE || '';

interface SectorStock {
  symbol: string;
  name: string;
  price: number;
  change: number;
  change_pct: number;
}

type SortKey = 'change_pct' | 'turnover' | 'up_count' | 'stock_count' | 'leading_stock_pct';
type SortDir = 'asc' | 'desc';

export default function SectorsPanel() {
  const { data: sectors, loading, refetch } = useSectors();
  const { data: hotStocks } = useHotStocks();
  const { openStock } = useStockClick();
  const [selectedSector, setSelectedSector] = useState<{name: string; code: string; change_pct: number} | null>(null);
  const [sectorStocks, setSectorStocks] = useState<SectorStock[]>([]);
  const [stocksLoading, setStocksLoading] = useState(false);
  const [sortKey, setSortKey] = useState<SortKey>('change_pct');
  const [sortDir, setSortDir] = useState<SortDir>('desc');
  const [viewMode, setViewMode] = useState<'rank' | 'leaders'>('rank');

  const handleSort = (key: SortKey) => {
    if (key === sortKey) {
      setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortKey(key);
      setSortDir(key === 'stock_count' || key === 'up_count' ? 'desc' : 'desc');
    }
  };

  const sortedSectors = useMemo(() => {
    if (!sectors) return [];
    return [...sectors].sort((a, b) => {
      let av = 0, bv = 0;
      switch (sortKey) {
        case 'change_pct':
          av = a.change_pct;
          bv = b.change_pct;
          break;
        case 'turnover':
          av = a.turnover;
          bv = b.turnover;
          break;
        case 'up_count':
          av = a.up_count;
          bv = b.up_count;
          break;
        case 'stock_count':
          av = a.stock_count;
          bv = b.stock_count;
          break;
        case 'leading_stock_pct':
          av = a.leading_stock_pct;
          bv = b.leading_stock_pct;
          break;
      }
      return sortDir === 'asc' ? av - bv : bv - av;
    });
  }, [sectors, sortKey, sortDir]);

  // 获取今日强势板块（涨幅前10）
  const topSectors = useMemo(() => {
    return sortedSectors.slice(0, 10);
  }, [sortedSectors]);

  // 获取资金流入板块
  const moneySectors = useMemo(() => {
    return [...(sectors || [])].sort((a, b) => b.turnover - a.turnover).slice(0, 10);
  }, [sectors]);

  // 获取领涨股票对应的板块
  const leaderSectors = useMemo(() => {
    if (!hotStocks || !sectors) return [];
    return hotStocks.slice(0, 20).map(stock => {
      const sector = sectors.find(s => stock.symbol.startsWith('00') && s.code.includes('cyb') ||
        (stock.symbol.startsWith('60') && s.code.includes('sh') && !s.code.includes('cyb')));
      return { stock, sector };
    }).filter(item => item.sector);
  }, [hotStocks, sectors]);

  const fetchSectorStocks = async (code: string) => {
    setStocksLoading(true);
    try {
      const res = await fetch(`${API_BASE}/api/sectors/${code}/stocks`);
      const data = await res.json();
      if (data.success) {
        setSectorStocks(data.data);
      }
    } catch (e) {
      console.error('Failed to fetch sector stocks:', e);
    } finally {
      setStocksLoading(false);
    }
  };

  const handleSectorClick = (sector: {name: string; code: string; change_pct: number}) => {
    setSelectedSector(sector);
    fetchSectorStocks(sector.code);
  };

  const SortHeader = ({ label, sortKeyName }: { label: string; sortKeyName: SortKey }) => (
    <button
      onClick={() => handleSort(sortKeyName)}
      className="inline-flex items-center gap-1 hover:text-foreground transition-colors"
    >
      {label}
      {sortKey === sortKeyName && (
        <span className="text-[9px]">{sortDir === 'asc' ? '▲' : '▼'}</span>
      )}
    </button>
  );

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border sticky top-0 bg-card z-10">
        <div className="flex items-center gap-2">
          <PieChart className="w-4 h-4 text-purple-400" />
          <h2 className="text-sm font-semibold">板块分析</h2>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setViewMode('rank')}
            className={`px-2 py-1 text-xs rounded ${viewMode === 'rank' ? 'bg-purple-500/20 text-purple-400' : 'text-muted-foreground hover:text-foreground'}`}
          >
            板块排行
          </button>
          <button
            onClick={() => setViewMode('leaders')}
            className={`px-2 py-1 text-xs rounded ${viewMode === 'leaders' ? 'bg-purple-500/20 text-purple-400' : 'text-muted-foreground hover:text-foreground'}`}
          >
            热门板块
          </button>
          <button onClick={refetch} className="text-muted-foreground hover:text-foreground transition-colors ml-2">
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      <ScrollArea className="flex-1">
        {viewMode === 'rank' ? (
          <div className="p-4 space-y-6">
            {/* 今日涨幅榜 */}
            <div>
              <h3 className="text-xs font-semibold text-muted-foreground mb-2 flex items-center gap-1.5">
                <TrendingUp className="w-3 h-3 text-up" /> 今日涨幅榜
              </h3>
              <div className="grid grid-cols-2 gap-2">
                {topSectors.map((sector, idx) => (
                  <div
                    key={sector.code}
                    onClick={() => handleSectorClick(sector)}
                    className={`rounded-lg p-3 border cursor-pointer transition-all hover:shadow-md ${
                      sector.change_pct > 0
                        ? 'dark:bg-red-950/30 dark:border-red-800/30 bg-red-50/50 border-red-200/50'
                        : sector.change_pct < 0
                        ? 'dark:bg-green-950/30 dark:border-green-800/30 bg-green-50/50 border-green-200/50'
                        : 'bg-card border-border'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className={`w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold ${
                          idx < 3 ? 'bg-purple-500/20 text-purple-400' : 'bg-muted text-muted-foreground'
                        }`}>
                          {idx + 1}
                        </span>
                        <span className="font-medium text-sm truncate max-w-[100px]">{sector.name}</span>
                      </div>
                      <span className={`font-mono-data font-semibold ${getChangeColor(sector.change_pct)}`}>
                        {formatPercent(sector.change_pct)}
                      </span>
                    </div>
                    <div className="mt-2 flex items-center justify-between text-[10px] text-muted-foreground">
                      <span>领涨: {sector.leading_stock}</span>
                      <span className={getChangeColor(sector.leading_stock_pct)}>{formatPercent(sector.leading_stock_pct)}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* 资金活跃榜 */}
            <div>
              <h3 className="text-xs font-semibold text-muted-foreground mb-2 flex items-center gap-1.5">
                <BarChart3 className="w-3 h-3 text-yellow-400" /> 资金活跃榜
              </h3>
              <div className="grid grid-cols-2 gap-2">
                {moneySectors.map((sector, idx) => (
                  <div
                    key={sector.code}
                    onClick={() => handleSectorClick(sector)}
                    className="rounded-lg p-3 border border-border bg-card cursor-pointer transition-all hover:shadow-md hover:border-purple-500/30"
                  >
                    <div className="flex items-center justify-between">
                      <span className="font-medium text-sm truncate max-w-[100px]">{sector.name}</span>
                      <span className="text-muted-foreground text-xs">{formatNumber(sector.turnover)}</span>
                    </div>
                    <div className="mt-2 flex items-center justify-between">
                      <span className={`font-mono-data text-sm ${getChangeColor(sector.change_pct)}`}>
                        {formatPercent(sector.change_pct)}
                      </span>
                      <span className="text-[10px] text-muted-foreground">
                        {sector.up_count}↑/{sector.down_count}↓
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* 完整板块排行 */}
            <div>
              <h3 className="text-xs font-semibold text-muted-foreground mb-2 flex items-center gap-1.5">
                <Activity className="w-3 h-3" /> 全部板块
              </h3>
              <table className="w-full text-xs">
                <thead className="sticky top-0 bg-card z-10">
                  <tr className="text-muted-foreground border-b border-border">
                    <th className="text-left py-2 px-2 font-medium w-8">#</th>
                    <th className="text-left py-2 px-2 font-medium">板块名称</th>
                    <th className="text-right py-2 px-2 font-medium">
                      <SortHeader label="涨跌幅" sortKeyName="change_pct" />
                    </th>
                    <th className="text-right py-2 px-2 font-medium">
                      <SortHeader label="成交额" sortKeyName="turnover" />
                    </th>
                    <th className="text-left py-2 px-2 font-medium">领涨股</th>
                    <th className="text-right py-2 px-2 font-medium">
                      <SortHeader label="涨跌" sortKeyName="up_count" />
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {sortedSectors.map((sector, idx) => (
                    <tr
                      key={sector.code}
                      onClick={() => handleSectorClick({ name: sector.name, code: sector.code, change_pct: sector.change_pct })}
                      className="border-b border-border/50 hover:bg-accent/50 transition-colors cursor-pointer"
                    >
                      <td className="py-1.5 px-2 text-muted-foreground">{idx + 1}</td>
                      <td className="py-1.5 px-2 font-medium">{sector.name}</td>
                      <td className={`text-right py-1.5 px-2 font-mono-data font-medium ${getChangeColor(sector.change_pct)}`}>
                        {formatPercent(sector.change_pct)}
                      </td>
                      <td className="text-right py-1.5 px-2 font-mono-data text-muted-foreground">
                        {formatNumber(sector.turnover)}
                      </td>
                      <td className="py-1.5 px-2">
                        <div className="flex items-center gap-1">
                          <span className="text-foreground">{sector.leading_stock}</span>
                          <span className={`font-mono-data text-xs ${getChangeColor(sector.leading_stock_pct)}`}>
                            {formatPercent(sector.leading_stock_pct)}
                          </span>
                        </div>
                      </td>
                      <td className="text-right py-1.5 px-2">
                        <span className="text-up font-mono-data">{sector.up_count}</span>
                        <span className="text-muted-foreground mx-0.5">/</span>
                        <span className="text-down font-mono-data">{sector.down_count}</span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        ) : (
          <div className="p-4 space-y-4">
            {/* 热门板块 */}
            <div>
              <h3 className="text-xs font-semibold text-muted-foreground mb-2">热门板块 - 领涨股票</h3>
              <div className="space-y-2">
                {hotStocks?.slice(0, 15).map((stock, idx) => (
                  <div
                    key={stock.symbol}
                    onClick={() => openStock(stock.symbol, stock.name)}
                    className="flex items-center justify-between p-3 rounded-lg border border-border bg-card hover:bg-accent/50 cursor-pointer transition-colors"
                  >
                    <div className="flex items-center gap-3">
                      <span className={`w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold ${
                        idx < 3 ? 'bg-yellow-500/20 text-yellow-400' : 'bg-muted text-muted-foreground'
                      }`}>
                        {idx + 1}
                      </span>
                      <div>
                        <div className="font-medium text-sm">{stock.name}</div>
                        <div className="text-[10px] text-muted-foreground">{stock.symbol}</div>
                      </div>
                    </div>
                    <div className="text-right">
                      <div className={`font-mono-data font-semibold ${getChangeColor(stock.change_pct)}`}>
                        {formatPercent(stock.change_pct)}
                      </div>
                      <div className="text-[10px] text-muted-foreground">{formatNumber(stock.turnover)}</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </ScrollArea>

      {/* 板块详情弹窗 */}
      {selectedSector && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
          onClick={() => setSelectedSector(null)}
        >
          <div
            className="bg-card border border-border rounded-lg w-[700px] max-h-[80vh] flex flex-col shadow-xl"
            onClick={(e) => e.stopPropagation()}
          >
            {/* 弹窗Header */}
            <div className="flex items-center justify-between px-4 py-3 border-b border-border">
              <div className="flex items-center gap-3">
                <PieChart className="w-4 h-4 text-purple-400" />
                <h3 className="font-semibold">{selectedSector.name}</h3>
                <span className={`font-mono-data font-semibold ${getChangeColor(selectedSector.change_pct)}`}>
                  {formatPercent(selectedSector.change_pct)}
                </span>
                <span className="text-xs text-muted-foreground">
                  {sectorStocks.length} 只股票
                </span>
              </div>
              <button
                onClick={() => { setSelectedSector(null); setSectorStocks([]); }}
                className="text-muted-foreground hover:text-foreground"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* 股票列表 */}
            <ScrollArea className="flex-1">
              {stocksLoading ? (
                <div className="flex items-center justify-center py-12">
                  <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
                </div>
              ) : (
                <table className="w-full text-xs">
                  <thead className="sticky top-0 bg-card">
                    <tr className="text-muted-foreground border-b border-border">
                      <th className="text-left py-2 px-3 font-medium">股票</th>
                      <th className="text-right py-2 px-2 font-medium">现价</th>
                      <th className="text-right py-2 px-2 font-medium">涨跌</th>
                      <th className="text-right py-2 px-3 font-medium">涨跌幅</th>
                      <th className="text-right py-2 px-3 font-medium">成交额</th>
                    </tr>
                  </thead>
                  <tbody>
                    {sectorStocks
                      .sort((a, b) => b.change_pct - a.change_pct)
                      .map((stock, idx) => (
                      <tr
                        key={stock.symbol}
                        className="border-b border-border/50 hover:bg-accent/50 cursor-pointer"
                        onClick={() => openStock(stock.symbol, stock.name)}
                      >
                        <td className="py-2 px-3">
                          <div className="flex items-center gap-2">
                            {idx < 3 && (
                              <span className={`w-4 h-4 rounded-full flex items-center justify-center text-[9px] font-bold ${
                                idx === 0 ? 'bg-yellow-500/20 text-yellow-400' :
                                idx === 1 ? 'bg-gray-400/20 text-gray-400' :
                                'bg-orange-500/20 text-orange-400'
                              }`}>
                                {idx + 1}
                              </span>
                            )}
                            <div>
                              <div className="font-medium">{stock.name}</div>
                              <div className="text-[10px] text-muted-foreground">{stock.symbol}</div>
                            </div>
                          </div>
                        </td>
                        <td className="text-right py-2 px-2 font-mono-data">
                          {formatPrice(stock.price)}
                        </td>
                        <td className={`text-right py-2 px-2 font-mono-data ${getChangeColor(stock.change)}`}>
                          {stock.change >= 0 ? '+' : ''}{stock.change.toFixed(2)}
                        </td>
                        <td className={`text-right py-2 px-3 font-mono-data font-semibold ${getChangeColor(stock.change_pct)}`}>
                          {formatPercent(stock.change_pct)}
                        </td>
                        <td className="text-right py-2 px-3 font-mono-data text-muted-foreground">
                          {formatNumber(stock.change * 10000)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </ScrollArea>
          </div>
        </div>
      )}
    </div>
  );
}
