/**
 * Dashboard - 主仪表盘页面
 * Design: 暗夜终端 - 左侧导航 + 顶部状态栏 + 中央面板区
 */
import { useState } from 'react';
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
import { useWebSocket } from '@/hooks/useMarketData';

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

  const ActivePanel = panelMap[activeTab];

  return (
    <div className="h-screen flex flex-col bg-background overflow-hidden">
      {/* Top Market Bar */}
      <MarketBar wsConnected={connected} isDemo={isDemo} />

      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />

        {/* Panel Area */}
        <main className="flex-1 overflow-hidden bg-card/30">
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
  );
}
