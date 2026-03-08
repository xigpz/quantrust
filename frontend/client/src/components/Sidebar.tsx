/**
 * Sidebar - 左侧导航栏
 * Design: 暗夜终端风格 - 带文字标签的侧边栏
 */
import {
  LayoutDashboard,
  Flame,
  Zap,
  BarChart3,
  PieChart,
  ArrowLeftRight,
  Star,
  FlaskConical,
  Settings,
  Sun,
  Moon,
  Wallet,
  TrendingUp,
  Shield,
  Award,
  Filter,
  Activity,
  Briefcase,
  Settings2,
  GitBranch,
  Trophy,
  Blocks,
  Store,
} from 'lucide-react';
import { useTheme } from '@/contexts/ThemeContext';

export type TabId = 'overview' | 'hot' | 'anomaly' | 'news' | 'momentum' | 'risk' | 'dragon' | 'factor' | 'screener' | 'sectors' | 'flow' | 'limitup' | 'watchlist' | 'backtest' | 'optimize' | 'versions' | 'sim' | 'leaderboard' | 'visual' | 'market' | 'portfolio' | 'settings' | 'recommend' | 'virtual' | 'strategy';

interface SidebarProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  className?: string;
}

const tabs: { id: TabId; label: string; icon: React.ReactNode }[] = [
  { id: 'overview', label: '市场总览', icon: <LayoutDashboard className="w-[18px] h-[18px]" /> },
  { id: 'hot', label: '热点监测', icon: <Flame className="w-[18px] h-[18px]" /> },
  { id: 'anomaly', label: '异动检测', icon: <Zap className="w-[18px] h-[18px]" /> },
  { id: 'news', label: '财经新闻', icon: <Activity className="w-[18px] h-[18px]" /> },
  { id: 'momentum', label: '动量分析', icon: <TrendingUp className="w-[18px] h-[18px]" /> },
  { id: 'risk', label: '风险控制', icon: <Shield className="w-[18px] h-[18px]" /> },
  { id: 'dragon', label: '龙虎榜', icon: <Award className="w-[18px] h-[18px]" /> },
  { id: 'recommend', label: '每日推荐', icon: <TrendingUp className="w-[18px] h-[18px]" /> },
  { id: 'virtual', label: '虚拟交易', icon: <Wallet className="w-[18px] h-[18px]" /> },
  { id: 'strategy', label: '策略选股', icon: <TrendingUp className="w-[18px] h-[18px]" /> },
  { id: 'factor', label: '因子库', icon: <Activity className="w-[18px] h-[18px]" /> },
  { id: 'screener', label: '选股器', icon: <Filter className="w-[18px] h-[18px]" /> },
  { id: 'sectors', label: '板块行情', icon: <PieChart className="w-[18px] h-[18px]" /> },
  { id: 'flow', label: '资金流向', icon: <ArrowLeftRight className="w-[18px] h-[18px]" /> },
  { id: 'limitup', label: '涨停监控', icon: <BarChart3 className="w-[18px] h-[18px]" /> },
  { id: 'watchlist2', label: '自选分析', icon: <Star className="w-[18px] h-[18px]" /> },
  { id: 'backtest', label: '策略回测', icon: <FlaskConical className="w-[18px] h-[18px]" /> },
  { id: 'optimize', label: '参数优化', icon: <Settings2 className="w-[18px] h-[18px]" /> },
  { id: 'versions', label: '版本管理', icon: <GitBranch className="w-[18px] h-[18px]" /> },
  { id: 'sim', label: '模拟交易', icon: <Wallet className="w-[18px] h-[18px]" /> },
  { id: 'leaderboard', label: '排行榜', icon: <Trophy className="w-[18px] h-[18px]" /> },
  { id: 'visual', label: '策略画布', icon: <Blocks className="w-[18px] h-[18px]" /> },
  { id: 'market', label: '模板市场', icon: <Store className="w-[18px] h-[18px]" /> },
  { id: 'portfolio', label: '持仓分析', icon: <Briefcase className="w-[18px] h-[18px]" /> },
];

function ThemeToggle() {
  const { theme, toggleTheme } = useTheme();
  
  return (
    <button
      onClick={toggleTheme}
      className="w-full h-10 px-3 rounded-md flex items-center gap-3 text-muted-foreground hover:text-sidebar-foreground hover:bg-sidebar-accent/50 transition-all duration-150"
    >
      {theme === 'dark' ? <Sun className="w-[18px] h-[18px]" /> : <Moon className="w-[18px] h-[18px]" />}
      <span className="text-sm">{theme === 'dark' ? '亮色模式' : '暗色模式'}</span>
    </button>
  );
}

export default function Sidebar({ activeTab, onTabChange, className = '' }: SidebarProps) {
  return (
    <aside className={`w-44 bg-sidebar border-r border-sidebar-border flex flex-col py-2 shrink-0 ${className}`}>
      <nav className="flex flex-col gap-1 px-2 flex-1">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => onTabChange(tab.id)}
            className={`
              w-full h-10 px-3 rounded-md flex items-center gap-3 transition-all duration-150
              ${activeTab === tab.id
                ? 'bg-sidebar-accent text-sidebar-primary'
                : 'text-muted-foreground hover:text-sidebar-foreground hover:bg-sidebar-accent/50'
              }
            `}
          >
            {tab.icon}
            <span className="text-sm">{tab.label}</span>
          </button>
        ))}
      </nav>

      {/* Divider */}
      <div className="border-t border-sidebar-border mx-2 my-2"></div>

      {/* Theme Toggle */}
      <div className="px-2">
        <ThemeToggle />
      </div>

      {/* Settings at bottom */}
      <div className="px-2">
        <button
          onClick={() => onTabChange('settings')}
          className={`
            w-full h-10 px-3 rounded-md flex items-center gap-3 transition-all duration-150
            ${activeTab === 'settings'
              ? 'bg-sidebar-accent text-sidebar-primary'
              : 'text-muted-foreground hover:text-sidebar-foreground hover:bg-sidebar-accent/50'
            }
          `}
        >
          <Settings className="w-[18px] h-[18px]" />
          <span className="text-sm">设置</span>
        </button>
      </div>
    </aside>
  );
}
