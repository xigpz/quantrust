/**
 * Sidebar - 左侧导航栏
 * Design: 暗夜终端风格 - 窄侧边栏，图标+文字
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
} from 'lucide-react';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useTheme } from '@/contexts/ThemeContext';

function ThemeToggle() {
  const { theme, toggleTheme } = useTheme();
  
  return (
    <Tooltip delayDuration={200}>
      <TooltipTrigger asChild>
        <button
          onClick={toggleTheme}
          className="w-10 h-10 rounded-md flex items-center justify-center text-muted-foreground hover:text-sidebar-foreground hover:bg-sidebar-accent/50 transition-all duration-150"
        >
          {theme === 'dark' ? <Sun className="w-[18px] h-[18px]" /> : <Moon className="w-[18px] h-[18px]" />}
        </button>
      </TooltipTrigger>
      <TooltipContent side="right" className="text-xs">
        {theme === 'dark' ? '切换亮色' : '切换暗色'}
      </TooltipContent>
    </Tooltip>
  );
}

export type TabId = 'overview' | 'hot' | 'anomaly' | 'momentum' | 'risk' | 'dragon' | 'factor' | 'screener' | 'sectors' | 'flow' | 'limitup' | 'watchlist' | 'backtest' | 'optimize' | 'versions' | 'sim' | 'portfolio' | 'settings';

interface SidebarProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
}

const tabs: { id: TabId; label: string; icon: React.ReactNode }[] = [
  { id: 'overview', label: '市场总览', icon: <LayoutDashboard className="w-[18px] h-[18px]" /> },
  { id: 'hot', label: '热点监测', icon: <Flame className="w-[18px] h-[18px]" /> },
  { id: 'anomaly', label: '异动检测', icon: <Zap className="w-[18px] h-[18px]" /> },
  { id: 'momentum', label: '动量分析', icon: <TrendingUp className="w-[18px] h-[18px]" /> },
  { id: 'risk', label: '风险控制', icon: <Shield className="w-[18px] h-[18px]" /> },
  { id: 'dragon', label: '龙虎榜', icon: <Award className="w-[18px] h-[18px]" /> },
  { id: 'factor', label: '因子库', icon: <Activity className="w-[18px] h-[18px]" /> },
  { id: 'screener', label: '选股器', icon: <Filter className="w-[18px] h-[18px]" /> },
  { id: 'sectors', label: '板块行情', icon: <PieChart className="w-[18px] h-[18px]" /> },
  { id: 'flow', label: '资金流向', icon: <ArrowLeftRight className="w-[18px] h-[18px]" /> },
  { id: 'limitup', label: '涨停监控', icon: <BarChart3 className="w-[18px] h-[18px]" /> },
  { id: 'watchlist', label: '自选股', icon: <Star className="w-[18px] h-[18px]" /> },
  { id: 'backtest', label: '策略回测', icon: <FlaskConical className="w-[18px] h-[18px]" /> },
  { id: 'optimize', label: '参数优化', icon: <Settings2 className="w-[18px] h-[18px]" /> },
  { id: 'versions', label: '版本管理', icon: <GitBranch className="w-[18px] h-[18px]" /> },
  { id: 'sim', label: '模拟交易', icon: <Wallet className="w-[18px] h-[18px]" /> },
  { id: 'portfolio', label: '持仓分析', icon: <Briefcase className="w-[18px] h-[18px]" /> },
];

export default function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  return (
    <aside className="w-14 bg-sidebar border-r border-sidebar-border flex flex-col items-center py-2 shrink-0">
      <nav className="flex flex-col gap-1 flex-1">
        {tabs.map((tab) => (
          <Tooltip key={tab.id} delayDuration={200}>
            <TooltipTrigger asChild>
              <button
                onClick={() => onTabChange(tab.id)}
                className={`
                  w-10 h-10 rounded-md flex items-center justify-center transition-all duration-150
                  ${activeTab === tab.id
                    ? 'bg-sidebar-accent text-sidebar-primary'
                    : 'text-muted-foreground hover:text-sidebar-foreground hover:bg-sidebar-accent/50'
                  }
                `}
              >
                {tab.icon}
              </button>
            </TooltipTrigger>
            <TooltipContent side="right" className="text-xs">
              {tab.label}
            </TooltipContent>
          </Tooltip>
        ))}
      </nav>

      {/* Theme Toggle */}
      <ThemeToggle />

      {/* Settings at bottom */}
      <Tooltip delayDuration={200}>
        <TooltipTrigger asChild>
          <button
            onClick={() => onTabChange('settings')}
            className={`
              w-10 h-10 rounded-md flex items-center justify-center transition-all duration-150
              ${activeTab === 'settings'
                ? 'bg-sidebar-accent text-sidebar-primary'
                : 'text-muted-foreground hover:text-sidebar-foreground hover:bg-sidebar-accent/50'
              }
            `}
          >
            <Settings className="w-[18px] h-[18px]" />
          </button>
        </TooltipTrigger>
        <TooltipContent side="right" className="text-xs">
          设置
        </TooltipContent>
      </Tooltip>
    </aside>
  );
}
