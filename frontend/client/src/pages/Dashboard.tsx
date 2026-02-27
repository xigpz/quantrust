/**
 * Dashboard - 主仪表盘页面
 * Design: 暗夜终端 - 左侧导航 + 顶部状态栏 + 中央面板区
 */
import { useState, createContext, useContext } from 'react';
import MarketBar from '@/components/MarketBar';
import Sidebar, { type TabId } from '@/components/Sidebar';
import OverviewPanel from '@/components/panels/OverviewPanel';
import HotStocksPanel from '@/components/panels/HotStocksPanel';
import AnomalyPanel from '@/components/panels/AnomalyPanel';
import SectorsPanel from '@/components/panels/SectorsPanel';
import MoneyFlowPanel from '@/components/panels/MoneyFlowPanel';
import LimitUpPanel from '@/components/panels/LimitUpPanel';
import WatchlistPanel from '@/components/panels/WatchlistPanel';
import BacktestPanel from '@/components/panels/BacktestPanel';
import SettingsPanel from '@/components/panels/SettingsPanel';
import StockDetailModal from '@/components/StockDetailModal';
import { useWebSocket } from '@/hooks/useMarketData';

// Context: 让子面板可以触发股票详情弹窗
interface StockClickCtx {
  openStock: (symbol: string, name?: string) => void;
}
export const StockClickContext = createContext<StockClickCtx>({ openStock: () => {} });
export function useStockClick() { return useContext(StockClickContext); }

const panelMap: Record<TabId, React.ComponentType> = {
  overview: OverviewPanel,
  hot: HotStocksPanel,
  anomaly: AnomalyPanel,
  sectors: SectorsPanel,
  flow: MoneyFlowPanel,
  limitup: LimitUpPanel,
  watchlist: WatchlistPanel,
  backtest: BacktestPanel,
  settings: SettingsPanel,
};

export default function Dashboard() {
  const [activeTab, setActiveTab] = useState<TabId>('overview');
  const { connected, isDemo } = useWebSocket();

  // 股票详情弹窗状态
  const [selectedStock, setSelectedStock] = useState<{ symbol: string; name?: string } | null>(null);

  const ActivePanel = panelMap[activeTab];

  return (
    <StockClickContext.Provider value={{ openStock: (symbol, name) => setSelectedStock({ symbol, name }) }}>
      <div className="h-screen flex flex-col bg-background overflow-hidden">
        {/* Top Market Bar */}
        <MarketBar wsConnected={connected} isDemo={isDemo} />

        {/* Main Content */}
        <div className="flex flex-1 overflow-hidden">
          {/* Sidebar */}
          <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />

          {/* Panel Area - overflow-y-auto 让面板内容可以滚动 */}
          <main className="flex-1 overflow-y-auto bg-card/30">
            <ActivePanel />
          </main>
        </div>

        {/* Bottom Status Bar */}
        <footer className="h-6 bg-card border-t border-border flex items-center px-4 text-[10px] text-muted-foreground gap-4 shrink-0">
          <span className="flex items-center gap-1">
            <div className={`w-1.5 h-1.5 rounded-full ${connected ? 'bg-green-500 pulse-dot' : 'bg-yellow-500'}`} />
            {connected ? '数据连接正常' : isDemo ? 'Demo 模式 — 启动后端后自动切换实时数据' : '等待连接...'}
          </span>
          <span>数据源: 东方财富</span>
          <span>刷新间隔: 15s</span>
          <div className="flex-1" />
          <span>QuantRust v0.1.0</span>
        </footer>
      </div>

      {/* 股票详情弹窗 */}
      <StockDetailModal
        symbol={selectedStock?.symbol ?? null}
        name={selectedStock?.name}
        onClose={() => setSelectedStock(null)}
      />
    </StockClickContext.Provider>
  );
}
