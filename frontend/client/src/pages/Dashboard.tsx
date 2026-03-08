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
import MomentumPanel from '@/components/panels/MomentumPanel';
import RiskPanel from '@/components/panels/RiskPanel';
import DragonTigerPanel from '@/components/panels/DragonTigerPanel';
import FactorPanel from '@/components/panels/FactorPanel';
import ScreenerPanel from '@/components/panels/ScreenerPanel';
import SectorsPanel from '@/components/panels/SectorsPanel';
import MoneyFlowPanel from '@/components/panels/MoneyFlowPanel';
import LimitUpPanel from '@/components/panels/LimitUpPanel';
import WatchlistPanel from '@/components/panels/WatchlistPanel';
import WatchlistAnalysisPanel from '@/components/panels/WatchlistAnalysisPanel';
import BacktestPanel from '@/components/panels/BacktestPanel';
import OptimizationPanel from '@/components/panels/OptimizationPanel';
import StrategyVersionsPanel from '@/components/panels/StrategyVersionsPanel';
import LeaderboardPanel from '@/components/panels/LeaderboardPanel';
import VisualStrategyEditor from '@/components/panels/VisualStrategyEditor';
import StrategyMarketPanel from '@/components/panels/StrategyMarketPanel';
import RecommendPanel from '@/components/panels/RecommendPanel';
import VirtualTradingPanel from '@/components/panels/VirtualTradingPanel';
import StrategyPanel from '@/components/panels/StrategyPanel';
import NewsPanel from '@/components/panels/NewsPanel';
import WatchlistAnalysisPanel from '@/components/panels/WatchlistAnalysisPanel';
import SimTrading from './SimTrading';
import PortfolioPanel from '@/components/panels/PortfolioPanel';
import SettingsPanel from '@/components/panels/SettingsPanel';
import StockDetailModal from '@/components/StockDetailModal';
import MobileNav from '@/components/MobileNav';
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
  news: NewsPanel,
  watchlist2: WatchlistAnalysisPanel,
  momentum: MomentumPanel,
  risk: RiskPanel,
  dragon: DragonTigerPanel,
  recommend: RecommendPanel,
  virtual: VirtualTradingPanel,
  strategy: StrategyPanel,
  factor: FactorPanel,
  screener: ScreenerPanel,
  sectors: SectorsPanel,
  flow: MoneyFlowPanel,
  limitup: LimitUpPanel,
  watchlist: WatchlistPanel,
  backtest: BacktestPanel,
  optimize: OptimizationPanel,
  versions: StrategyVersionsPanel,
  sim: SimTrading,
  portfolio: PortfolioPanel,
  settings: SettingsPanel,
  leaderboard: LeaderboardPanel,
  visual: VisualStrategyEditor,
  market: StrategyMarketPanel,
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
        {/* Top Market Bar - 移动端隐藏 */}
        <MarketBar wsConnected={connected} isDemo={isDemo} className="hidden md:flex" />

        {/* Main Content */}
        <div className="flex flex-1 overflow-hidden">
          {/* Sidebar - 移动端可折叠 */}
          <Sidebar 
            activeTab={activeTab} 
            onTabChange={setActiveTab} 
            className="hidden md:flex w-44 shrink-0"
          />
          
          {/* 移动端底部导航 */}
          <MobileNav 
            activeTab={activeTab} 
            onTabChange={setActiveTab} 
            className="md:hidden fixed bottom-0 left-0 right-0 z-50"
          />

          {/* Panel Area */}
          <main className="flex-1 overflow-y-auto bg-card/30 pb-16 md:pb-0">
            <ActivePanel />
          </main>
        </div>

        {/* Bottom Status Bar - 移动端隐藏 */}
        <footer className="hidden md:flex h-6 bg-card border-t border-border items-center px-4 text-[10px] text-muted-foreground gap-4 shrink-0">
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
