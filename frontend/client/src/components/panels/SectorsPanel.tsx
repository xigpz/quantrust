/**
 * SectorsPanel - 板块行情面板
 * Design: 暗夜终端 - 板块排行表格 + 详情弹窗
 */
import { useState, useEffect, useContext } from 'react';
import { useSectors, formatPercent, formatNumber, getChangeColor } from '@/hooks/useMarketData';
import { useStockClick } from '@/pages/Dashboard';
import { PieChart, RefreshCw, X, ChevronRight, Loader2 } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';

const API_BASE = import.meta.env.VITE_API_BASE || '';

interface SectorStock {
  symbol: string;
  name: string;
  price: number;
  change: number;
  change_pct: number;
}

export default function SectorsPanel() {
  const { data: sectors, loading, refetch } = useSectors();
  const { openStock } = useStockClick();
  const [selectedSector, setSelectedSector] = useState<{name: string; code: string} | null>(null);
  const [sectorStocks, setSectorStocks] = useState<SectorStock[]>([]);
  const [stocksLoading, setStocksLoading] = useState(false);

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

  const handleSectorClick = (sector: {name: string; code: string}) => {
    setSelectedSector(sector);
    fetchSectorStocks(sector.code);
  };

  return (
    <div className="flex flex-col h-full">
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
      <div className="flex-1 overflow-auto">
        <table className="w-full text-xs">
          <thead className="sticky top-0 bg-card z-10">
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
              <tr 
                key={sector.code} 
                onClick={() => handleSectorClick({ name: sector.name, code: sector.code })}
                className="border-b border-border/50 hover:bg-accent/50 transition-colors cursor-pointer"
              >
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

      {/* 板块详情弹窗 */}
      {selectedSector && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-card border border-border rounded-lg w-[600px] max-h-[80vh] flex flex-col shadow-xl">
            {/* 弹窗Header */}
            <div className="flex items-center justify-between px-4 py-3 border-b border-border">
              <div className="flex items-center gap-2">
                <PieChart className="w-4 h-4 text-purple-400" />
                <h3 className="font-semibold">{selectedSector.name}</h3>
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
                    </tr>
                  </thead>
                  <tbody>
                    {sectorStocks.map((stock) => (
                      <tr 
                        key={stock.symbol} 
                        className="border-b border-border/50 hover:bg-accent/50 cursor-pointer"
                        onClick={() => openStock(stock.symbol, stock.name)}
                      >
                        <td className="py-2 px-3">
                          <div className="font-medium">{stock.name}</div>
                          <div className="text-[10px] text-muted-foreground">{stock.symbol}</div>
                        </td>
                        <td className="text-right py-2 px-2 font-mono-data">
                          {stock.price.toFixed(2)}
                        </td>
                        <td className={`text-right py-2 px-2 font-mono-data ${getChangeColor(stock.change)}`}>
                          {stock.change >= 0 ? '+' : ''}{stock.change.toFixed(2)}
                        </td>
                        <td className={`text-right py-2 px-3 font-mono-data font-medium ${getChangeColor(stock.change_pct)}`}>
                          {formatPercent(stock.change_pct)}
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
