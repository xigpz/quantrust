/**
 * Sidebar - 左侧导航栏
 * Design: 暗夜终端风格 - 带文字标签的侧边栏二级菜单
 */
import { useState, useEffect } from 'react';
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
  ChevronDown,
  ChevronRight,
  Layers,
  Search,
  LineChart,
  Newspaper,
  Percent,
  Boxes,
  Brain,
  Clock,
  Globe,
  Pin,
  X,
  MoreHorizontal,
} from 'lucide-react';
import { useTheme } from '@/contexts/ThemeContext';

const FAVORITES_KEY = 'quantrust_favorites';

// Get all available menu items for favorites
const allMenuItems: { id: TabId; label: string; icon: React.ReactNode }[] = [
  { id: 'overview', label: '市场总览', icon: <LayoutDashboard className="w-[18px] h-[18px]" /> },
  { id: 'hot', label: '热点监测', icon: <Flame className="w-[18px] h-[18px]" /> },
  { id: 'anomaly', label: '异动检测', icon: <Zap className="w-[18px] h-[18px]" /> },
  { id: 'limitup', label: '涨停监控', icon: <BarChart3 className="w-[18px] h-[18px]" /> },
  { id: 'sectors', label: '板块行情', icon: <PieChart className="w-[18px] h-[18px]" /> },
  { id: 'sectorflow', label: '板块分时', icon: <Activity className="w-[18px] h-[18px]" /> },
  { id: 'flow', label: '资金流向', icon: <ArrowLeftRight className="w-[18px] h-[18px]" /> },
  { id: 'timing', label: '时机把握', icon: <Clock className="w-[18px] h-[18px]" /> },
  { id: 'watchlist', label: '自选股', icon: <Star className="w-[18px] h-[18px]" /> },
  { id: 'dragon', label: '龙虎榜', icon: <Award className="w-[18px] h-[18px]" /> },
  { id: 'screener', label: '选股器', icon: <Filter className="w-[18px] h-[18px]" /> },
  { id: 'strategy', label: '策略选股', icon: <TrendingUp className="w-[18px] h-[18px]" /> },
  { id: 'factor', label: '因子库', icon: <Boxes className="w-[18px] h-[18px]" /> },
  { id: 'recommend', label: '每日推荐', icon: <Percent className="w-[18px] h-[18px]" /> },
  { id: 'aipattern', label: 'AI形态', icon: <Brain className="w-[18px] h-[18px]" /> },
  { id: 'news', label: '财经新闻', icon: <Newspaper className="w-[18px] h-[18px]" /> },
  { id: 'momentum', label: '动量分析', icon: <TrendingUp className="w-[18px] h-[18px]" /> },
  { id: 'risk', label: '风险控制', icon: <Shield className="w-[18px] h-[18px]" /> },
  { id: 'portfolio', label: '持仓分析', icon: <Briefcase className="w-[18px] h-[18px]" /> },
  { id: 'virtual', label: '虚拟交易', icon: <Wallet className="w-[18px] h-[18px]" /> },
  { id: 'sim', label: '模拟交易', icon: <Wallet className="w-[18px] h-[18px]" /> },
  { id: 'backtest', label: '策略回测', icon: <FlaskConical className="w-[18px] h-[18px]" /> },
  { id: 'optimize', label: '参数优化', icon: <Settings2 className="w-[18px] h-[18px]" /> },
  { id: 'versions', label: '版本管理', icon: <GitBranch className="w-[18px] h-[18px]" /> },
  { id: 'visual', label: '策略画布', icon: <Blocks className="w-[18px] h-[18px]" /> },
  { id: 'leaderboard', label: '排行榜', icon: <Trophy className="w-[18px] h-[18px]" /> },
  { id: 'market', label: '模板市场', icon: <Store className="w-[18px] h-[18px]" /> },
  { id: 'command', label: '全球市场', icon: <Globe className="w-[18px] h-[18px]" /> },
];

export type TabId = 'overview' | 'hot' | 'anomaly' | 'news' | 'momentum' | 'risk' | 'dragon' | 'factor' | 'screener' | 'sectors' | 'sectorflow' | 'flow' | 'limitup' | 'watchlist' | 'backtest' | 'optimize' | 'versions' | 'sim' | 'leaderboard' | 'visual' | 'market' | 'portfolio' | 'settings' | 'recommend' | 'virtual' | 'strategy' | 'aipattern' | 'timing' | 'command';

interface SidebarProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  className?: string;
}

interface MenuItem {
  id: TabId;
  label: string;
  icon: React.ReactNode;
}

interface MenuGroup {
  id: string;
  label: string;
  icon: React.ReactNode;
  items: MenuItem[];
}

const menuGroups: MenuGroup[] = [
  {
    id: 'market',
    label: '市场监控',
    icon: <LineChart className="w-[18px] h-[18px]" />,
    items: [
      { id: 'overview', label: '市场总览', icon: <LayoutDashboard className="w-[18px] h-[18px]" /> },
      { id: 'hot', label: '热点监测', icon: <Flame className="w-[18px] h-[18px]" /> },
      { id: 'anomaly', label: '异动检测', icon: <Zap className="w-[18px] h-[18px]" /> },
      { id: 'limitup', label: '涨停监控', icon: <BarChart3 className="w-[18px] h-[18px]" /> },
      { id: 'sectors', label: '板块行情', icon: <PieChart className="w-[18px] h-[18px]" /> },
      { id: 'sectorflow', label: '板块分时', icon: <Activity className="w-[18px] h-[18px]" /> },
      { id: 'flow', label: '资金流向', icon: <ArrowLeftRight className="w-[18px] h-[18px]" /> },
      { id: 'timing', label: '时机把握', icon: <Clock className="w-[18px] h-[18px]" /> },
      { id: 'command', label: '全球市场', icon: <Globe className="w-[18px] h-[18px]" /> },
    ],
  },
  {
    id: 'quotes',
    label: '行情数据',
    icon: <Layers className="w-[18px] h-[18px]" />,
    items: [
      { id: 'watchlist', label: '自选股', icon: <Star className="w-[18px] h-[18px]" /> },
      { id: 'dragon', label: '龙虎榜', icon: <Award className="w-[18px] h-[18px]" /> },
    ],
  },
  {
    id: 'selection',
    label: '选股工具',
    icon: <Search className="w-[18px] h-[18px]" />,
    items: [
      { id: 'screener', label: '选股器', icon: <Filter className="w-[18px] h-[18px]" /> },
      { id: 'strategy', label: '策略选股', icon: <TrendingUp className="w-[18px] h-[18px]" /> },
      { id: 'factor', label: '因子库', icon: <Boxes className="w-[18px] h-[18px]" /> },
      { id: 'recommend', label: '每日推荐', icon: <Percent className="w-[18px] h-[18px]" /> },
      { id: 'aipattern', label: 'AI形态', icon: <Brain className="w-[18px] h-[18px]" /> },
    ],
  },
  {
    id: 'analysis',
    label: '分析工具',
    icon: <LineChart className="w-[18px] h-[18px]" />,
    items: [
      { id: 'news', label: '财经新闻', icon: <Newspaper className="w-[18px] h-[18px]" /> },
      { id: 'momentum', label: '动量分析', icon: <TrendingUp className="w-[18px] h-[18px]" /> },
      { id: 'risk', label: '风险控制', icon: <Shield className="w-[18px] h-[18px]" /> },
      { id: 'portfolio', label: '持仓分析', icon: <Briefcase className="w-[18px] h-[18px]" /> },
    ],
  },
  {
    id: 'trading',
    label: '交易模拟',
    icon: <Wallet className="w-[18px] h-[18px]" />,
    items: [
      { id: 'virtual', label: '虚拟交易', icon: <Wallet className="w-[18px] h-[18px]" /> },
      { id: 'sim', label: '模拟交易', icon: <Wallet className="w-[18px] h-[18px]" /> },
    ],
  },
  {
    id: 'strategy',
    label: '策略开发',
    icon: <FlaskConical className="w-[18px] h-[18px]" />,
    items: [
      { id: 'backtest', label: '策略回测', icon: <FlaskConical className="w-[18px] h-[18px]" /> },
      { id: 'optimize', label: '参数优化', icon: <Settings2 className="w-[18px] h-[18px]" /> },
      { id: 'versions', label: '版本管理', icon: <GitBranch className="w-[18px] h-[18px]" /> },
      { id: 'visual', label: '策略画布', icon: <Blocks className="w-[18px] h-[18px]" /> },
    ],
  },
  {
    id: 'other',
    label: '其他',
    icon: <Trophy className="w-[18px] h-[18px]" />,
    items: [
      { id: 'leaderboard', label: '排行榜', icon: <Trophy className="w-[18px] h-[18px]" /> },
      { id: 'market', label: '模板市场', icon: <Store className="w-[18px] h-[18px]" /> },
    ],
  },
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
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set(['market']));
  const [favorites, setFavorites] = useState<TabId[]>(() => {
    try {
      const saved = localStorage.getItem(FAVORITES_KEY);
      return saved ? JSON.parse(saved) : ['overview', 'hot', 'anomaly'];
    } catch {
      return ['overview', 'hot', 'anomaly'];
    }
  });
  const [showFavoritesPicker, setShowFavoritesPicker] = useState(false);

  // Persist favorites to localStorage
  useEffect(() => {
    localStorage.setItem(FAVORITES_KEY, JSON.stringify(favorites));
  }, [favorites]);

  const toggleGroup = (groupId: string) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(groupId)) {
        next.delete(groupId);
      } else {
        next.add(groupId);
      }
      return next;
    });
  };

  const addToFavorites = (id: TabId) => {
    if (!favorites.includes(id)) {
      setFavorites([...favorites, id]);
    }
  };

  const removeFromFavorites = (id: TabId) => {
    setFavorites(favorites.filter(f => f !== id));
  };

  const isFavorite = (id: TabId) => favorites.includes(id);

  const getMenuItem = (id: TabId) => allMenuItems.find(item => item.id === id);

  // 检查当前激活的 tab 属于哪个分组
  const getGroupForTab = (tabId: TabId): string | null => {
    for (const group of menuGroups) {
      if (group.items.some((item) => item.id === tabId)) {
        return group.id;
      }
    }
    return null;
  };

  const activeGroup = getGroupForTab(activeTab);

  // Get favorite menu items
  const favoriteItems = favorites.map(id => getMenuItem(id)).filter(Boolean) as { id: TabId; label: string; icon: React.ReactNode }[];

  return (
    <aside className={`w-52 bg-sidebar border-r border-sidebar-border flex flex-col py-2 shrink-0 ${className}`}>
      {/* 常用菜单 */}
      <div className="px-2 mb-1">
        <div className="flex items-center justify-between mb-1">
          <span className="text-xs font-medium text-muted-foreground">常用</span>
          <button
            onClick={() => setShowFavoritesPicker(true)}
            className="p-1 rounded hover:bg-sidebar-accent/50 text-muted-foreground hover:text-sidebar-foreground"
            title="管理常用菜单"
          >
            <MoreHorizontal className="w-3.5 h-3.5" />
          </button>
        </div>
        <div className="space-y-0.5">
          {favoriteItems.slice(0, 5).map((item) => (
            <div
              key={item.id}
              className={`
                group w-full h-8 px-1 rounded-md flex items-center gap-1 transition-all duration-150 text-xs
                ${activeTab === item.id
                  ? 'bg-sidebar-accent text-sidebar-primary font-medium'
                  : 'text-muted-foreground hover:text-sidebar-foreground hover:bg-sidebar-accent/50'
                }
              `}
            >
              <button
                onClick={() => onTabChange(item.id)}
                className="flex-1 h-full px-2 rounded-md flex items-center gap-2 text-left"
              >
                <span className="shrink-0">{item.icon}</span>
                <span className="truncate">{item.label}</span>
              </button>
              <button
                onClick={() => removeFromFavorites(item.id)}
                className="p-0.5 rounded hover:bg-sidebar-accent/50 opacity-0 group-hover:opacity-100"
                aria-label={`从常用中移除 ${item.label}`}
                title={`从常用中移除 ${item.label}`}
              >
                <X className="w-3 h-3" />
              </button>
            </div>
          ))}
          {favoriteItems.length === 0 && (
            <button
              onClick={() => setShowFavoritesPicker(true)}
              className="w-full h-8 px-3 rounded-md flex items-center gap-2 text-xs text-muted-foreground hover:text-sidebar-foreground hover:bg-sidebar-accent/50"
            >
              <Pin className="w-[14px] h-[14px]" />
              <span>添加常用功能</span>
            </button>
          )}
        </div>
      </div>

      {/* Divider */}
      <div className="border-t border-sidebar-border mx-2 my-1"></div>

      <nav className="flex flex-col gap-1 px-2 flex-1 overflow-y-auto">
        {menuGroups.map((group) => {
          const isExpanded = expandedGroups.has(group.id);
          const isActive = activeGroup === group.id;

          return (
            <div key={group.id} className="mb-1">
              {/* 分组标题 */}
              <button
                onClick={() => toggleGroup(group.id)}
                className={`
                  w-full h-8 px-2 rounded-md flex items-center gap-2 transition-all duration-150
                  ${isActive ? 'bg-sidebar-accent/50' : 'hover:bg-sidebar-accent/30'}
                `}
              >
                {isExpanded ? (
                  <ChevronDown className="w-3.5 h-3.5 text-muted-foreground" />
                ) : (
                  <ChevronRight className="w-3.5 h-3.5 text-muted-foreground" />
                )}
                <span className="text-xs font-medium text-muted-foreground">{group.label}</span>
              </button>

              {/* 分组菜单项 */}
              {isExpanded && (
                <div className="mt-1 ml-2 space-y-0.5">
                  {group.items.map((item) => (
                    <button
                      key={item.id}
                      onClick={() => onTabChange(item.id)}
                      className={`
                        w-full h-8 px-3 rounded-md flex items-center gap-2 transition-all duration-150 text-xs
                        ${activeTab === item.id
                          ? 'bg-sidebar-accent text-sidebar-primary font-medium'
                          : 'text-muted-foreground hover:text-sidebar-foreground hover:bg-sidebar-accent/50'
                        }
                      `}
                    >
                      <span className="shrink-0">{item.icon}</span>
                      <span className="truncate">{item.label}</span>
                      <Pin
                        className={`w-3 h-3 ml-auto shrink-0 cursor-pointer transition-colors ${
                          isFavorite(item.id)
                            ? 'text-primary opacity-100'
                            : 'opacity-0 hover:opacity-100'
                        }`}
                        onClick={(e) => {
                          e.stopPropagation();
                          if (isFavorite(item.id)) {
                            removeFromFavorites(item.id);
                          } else {
                            addToFavorites(item.id);
                          }
                        }}
                      />
                    </button>
                  ))}
                </div>
              )}
            </div>
          );
        })}
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

      {/* 常用菜单选择弹窗 */}
      {showFavoritesPicker && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-sidebar border border-sidebar-border rounded-lg shadow-xl w-80 max-h-[70vh] flex flex-col">
            <div className="flex items-center justify-between p-4 border-b border-sidebar-border">
              <h3 className="text-sm font-medium">管理常用菜单</h3>
              <button
                onClick={() => setShowFavoritesPicker(false)}
                className="p-1 rounded hover:bg-sidebar-accent/50"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
            <div className="flex-1 overflow-y-auto p-2">
              <div className="space-y-1">
                {allMenuItems.map((item) => {
                  const isFav = isFavorite(item.id);
                  return (
                    <button
                      key={item.id}
                      onClick={() => {
                        if (isFav) {
                          removeFromFavorites(item.id);
                        } else {
                          addToFavorites(item.id);
                        }
                      }}
                      className={`
                        w-full h-9 px-3 rounded-md flex items-center gap-2 text-xs transition-colors
                        ${isFav
                          ? 'bg-primary/20 text-primary'
                          : 'text-muted-foreground hover:bg-sidebar-accent/50 hover:text-sidebar-foreground'
                        }
                      `}
                    >
                      <span className="shrink-0">{item.icon}</span>
                      <span className="flex-1 text-left">{item.label}</span>
                      {isFav ? (
                        <Pin className="w-3.5 h-3.5 text-primary" />
                      ) : (
                        <span className="text-[10px] text-muted-foreground">右键添加</span>
                      )}
                    </button>
                  );
                })}
              </div>
            </div>
            <div className="p-3 border-t border-sidebar-border">
              <button
                onClick={() => setShowFavoritesPicker(false)}
                className="w-full h-9 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90"
              >
                完成
              </button>
            </div>
          </div>
        </div>
      )}
    </aside>
  );
}
