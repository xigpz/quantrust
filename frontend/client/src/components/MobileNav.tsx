import { TabId } from '@/components/Sidebar';
import { LayoutDashboard, TrendingUp, Wallet, Star, BarChart3 } from 'lucide-react';

interface MobileNavProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  className?: string;
}

// 移动端底部导航 - 只显示最常用的4个功能
const mobileTabs: { id: TabId; label: string; icon: React.ReactNode }[] = [
  { id: 'overview', label: '行情', icon: <LayoutDashboard className="w-5 h-5" /> },
  { id: 'hot', label: '热点', icon: <BarChart3 className="w-5 h-5" /> },
  { id: 'strategy', label: '选股', icon: <TrendingUp className="w-5 h-5" /> },
  { id: 'virtual', label: '交易', icon: <Wallet className="w-5 h-5" /> },
  { id: 'watchlist', label: '自选', icon: <Star className="w-5 h-5" /> },
];

export default function MobileNav({ activeTab, onTabChange, className = '' }: MobileNavProps) {
  return (
    <nav className={`flex bg-card border-t border-border ${className}`}>
      {mobileTabs.map((tab) => (
        <button
          key={tab.id}
          onClick={() => onTabChange(tab.id)}
          className={`flex-1 flex flex-col items-center justify-center py-2 px-1 text-[10px] transition-colors ${
            activeTab === tab.id
              ? 'text-primary bg-primary/10'
              : 'text-muted-foreground'
          }`}
        >
          {tab.icon}
          <span className="mt-1">{tab.label}</span>
        </button>
      ))}
    </nav>
  );
}
