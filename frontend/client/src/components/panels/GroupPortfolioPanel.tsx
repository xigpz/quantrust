/**
 * GroupPortfolioPanel - 模拟组合全功能版 V4
 * 参考东方财富组合功能，增强个人组合和板块分析
 * 优化：增加sparkline、雷达图、生命周期可视化、实时模拟等
 */
import { useEffect, useState, useCallback, useMemo, useRef } from 'react';
import {
  Trophy, Star, RefreshCw, TrendingUp, TrendingDown, Plus, Minus, X,
  BarChart2, Activity, Target, Users, Crown, ArrowUpDown, Filter,
  Eye, EyeOff, Info, PieChart, Briefcase, DollarSign, Percent,
  Calendar, Clock, GitCompare, Flame, Shield, Zap, ArrowUp, ArrowDown,
  ChevronDown, ChevronRight, LayoutGrid, List, TrendingFlat, Play,
  Pause, Settings2, Maximize2, Minimize2
} from 'lucide-react';
import { useStockClick } from '@/pages/Dashboard';
import {
  ChartContainer,
} from '@/components/ui/chart';
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  BarChart,
  Bar,
  Cell,
  Legend,
  PieChart as RechartsPie,
  Pie,
  RadialBarChart,
  RadialBar,
  RadarChart,
  Radar,
  PolarGrid,
  PolarAngleAxis,
  PolarRadiusAxis,
  ReferenceLine,
  Rectangle,
} from 'recharts';

// ==================== 类型定义 ====================

interface Portfolio {
  id: string;
  name: string;
  owner: string;
  total_return: number;
  win_rate: number;
  recent_returns: number[];
  follower_count: number;
  created_at: string;
  win_count?: number;
  lose_count?: number;
  annualized_return?: number;
  max_drawdown?: number;
  volatility?: number;
  sharpe_ratio?: number;
  total_trades?: number;
  // 板块信息
  sector_allocation?: SectorAllocation[];
  // 我的组合
  is_mine?: boolean;
  // 标签
  tags?: string[];
}

interface SectorAllocation {
  name: string;
  percentage: number;
  change?: number;
}

interface PortfolioDetail {
  portfolio: Portfolio;
  daily_return: number;
  return_5d: number;
  return_20d: number;
  return_60d: number;
  return_250d: number;
  max_drawdown: number;
  hold_position_sum: string;
  // 额外数据
  total_capital?: number;
  available_capital?: number;
  positions_count?: number;
  sector_allocation?: SectorAllocation[];
  asset_distribution?: AssetDistribution[];
  recent_activities?: ActivityItem[];
  equity_curve?: EquityPoint[];
}

interface AssetDistribution {
  name: string;
  value: number;
  color: string;
}

interface ActivityItem {
  type: 'buy' | 'sell' | 'dividend' | 'split';
  symbol: string;
  name: string;
  time: string;
  price?: number;
  quantity?: number;
  amount?: number;
  note?: string;
}

interface EquityPoint {
  date: string;
  value: number;
  benchmark?: number;
}

// 实时价格更新类型
interface LivePrice {
  symbol: string;
  price: number;
  change_pct: number;
  timestamp: number;
}

interface PortfolioHolding {
  symbol: string;
  name: string;
  price: number;
  change_pct: number;
  cost: number;
  profit_ratio: number;
  shares: number;
  hold_days: number;
  // 板块信息
  sector?: string;
  // 生命周期
  lifecycle_stage?: '建仓' | '拉升' | '持有' | '成熟' | '减仓';
}

interface TransferRecord {
  symbol: string;
  name: string;
  action: string;
  position_before: number;
  position_after: number;
  price: number;
  amount?: number;
  time: string;
  profit?: number;
}

interface HoldingChange {
  symbol: string;
  name: string;
  change_type: 'add' | 'reduce' | 'clear';
  shares: number;
  price: number;
  time: string;
}

// ==================== 工具函数 ====================

// 渲染小型sparkline SVG
const renderSparkline = (data: number[], width: number = 50, height: number = 18) => {
  if (!data || data.length < 2) return <span className="text-muted-foreground text-[9px]">--</span>;
  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;
  const stepX = width / (data.length - 1);
  const points = data.map((v, i) => `${i * stepX},${height - ((v - min) / range) * height}`).join(' ');
  const isPositive = data[data.length - 1] >= data[0];
  const lineColor = isPositive ? '#ef4444' : '#22c55e';
  return (
    <svg width={width} height={height} className="inline-block align-middle">
      <polyline
        points={points}
        fill="none"
        stroke={lineColor}
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
};

const generateEquityCurve = (baseReturn: number, days: number = 180) => {
  const data: EquityPoint[] = [];
  let value = 100;
  const dailyVol = Math.abs(baseReturn) / 100 / Math.sqrt(252) * 1.5;

  for (let i = 0; i < days; i++) {
    const randomReturn = (Math.random() - 0.48) * dailyVol * 100;
    value = value * (1 + randomReturn / 100);
    const date = new Date(Date.now() - (days - i) * 24 * 60 * 60 * 1000);
    data.push({
      date: `${date.getMonth() + 1}/${date.getDate()}`,
      value: Math.round(value * 100) / 100,
      benchmark: 100 + (i * (baseReturn / days)),
    });
  }
  return data;
};

const calculateScore = (portfolio: Portfolio): number => {
  const returnScore = Math.min(portfolio.total_return / 100, 40);
  const winRateScore = (portfolio.win_rate - 50) * 0.4;
  const followerScore = Math.min(portfolio.follower_count / 200, 10);
  const tradesScore = Math.min((portfolio.win_count || 0) / 50, 10);
  return Math.round((returnScore + winRateScore + followerScore + tradesScore) * 10) / 10;
};

// ==================== 板块颜色 ====================
const SECTOR_COLORS: Record<string, string> = {
  '科技': '#3b82f6',
  '医药': '#10b981',
  '消费': '#f59e0b',
  '金融': '#8b5cf6',
  '新能源': '#22c55e',
  '军工': '#ef4444',
  '半导体': '#06b6d4',
  'AI': '#ec4899',
  '房地产': '#f97316',
  '基建': '#84cc16',
  '教育': '#a855f7',
  '传媒': '#14b8a6',
  '环保': '#22d3d1',
  'default': '#6b7280',
};

const getSectorColor = (sector: string): string => {
  for (const key of Object.keys(SECTOR_COLORS)) {
    if (sector.includes(key)) return SECTOR_COLORS[key];
  }
  return SECTOR_COLORS['default'];
};

// ==================== 模拟数据 ====================

const generateMockPortfolios = (): Portfolio[] => [
  // ===== 热门排行榜 =====
  { id: "260640100000017705", name: "泰来战略5", owner: "泰来大福利", total_return: 24.32, win_rate: 75.5, recent_returns: [1.2, -0.5, 2.1, 0.8, -1.2], follower_count: 128, created_at: "2024-01-15", win_count: 18, lose_count: 6, annualized_return: 28.5, max_drawdown: -8.5, volatility: 15.2, sharpe_ratio: 1.52, total_trades: 24, tags: ["价值投资", "长线"], sector_allocation: [{ name: "消费", percentage: 35 }, { name: "医药", percentage: 25 }, { name: "金融", percentage: 20 }, { name: "科技", percentage: 20 }] },
  { id: "252710200000054426", name: "趋势追击者", owner: "别样人生", total_return: 64.37, win_rate: 68.2, recent_returns: [0.5, 1.8, -0.3, 3.2, 1.5], follower_count: 86, created_at: "2023-06-20", win_count: 45, lose_count: 21, annualized_return: 45.2, max_drawdown: -15.3, volatility: 22.8, sharpe_ratio: 1.68, total_trades: 66, tags: ["趋势跟踪", "短线"], sector_allocation: [{ name: "科技", percentage: 40 }, { name: "新能源", percentage: 30 }, { name: "半导体", percentage: 30 }] },
  { id: "251730200000152651", name: "价值成长", owner: "我逢买必涨", total_return: 139.57, win_rate: 82.1, recent_returns: [2.5, 1.2, 0.8, 3.5, -0.5], follower_count: 215, created_at: "2023-03-10", win_count: 28, lose_count: 6, annualized_return: 52.3, max_drawdown: -12.8, volatility: 18.5, sharpe_ratio: 2.45, total_trades: 34, tags: ["成长股", "中线"], sector_allocation: [{ name: "AI", percentage: 45 }, { name: "半导体", percentage: 35 }, { name: "科技", percentage: 20 }] },
  { id: "260010100000098241", name: "科技龙头", owner: "科创先锋", total_return: 88.45, win_rate: 71.3, recent_returns: [-0.8, 2.3, 1.5, 0.9, 2.8], follower_count: 156, created_at: "2023-09-05", win_count: 52, lose_count: 21, annualized_return: 38.6, max_drawdown: -18.2, volatility: 28.5, sharpe_ratio: 1.18, total_trades: 73, tags: ["科技", "龙头"], sector_allocation: [{ name: "科技", percentage: 50 }, { name: "AI", percentage: 30 }, { name: "半导体", percentage: 20 }] },
  { id: "261520100000187654", name: "消费升级", owner: "内需之王", total_return: 45.82, win_rate: 65.8, recent_returns: [0.3, -0.2, 1.8, 0.5, -0.9], follower_count: 92, created_at: "2024-02-28", win_count: 35, lose_count: 18, annualized_return: 55.2, max_drawdown: -9.8, volatility: 12.3, sharpe_ratio: 3.85, total_trades: 53, tags: ["消费", "价值"], sector_allocation: [{ name: "消费", percentage: 60 }, { name: "医药", percentage: 25 }, { name: "基建", percentage: 15 }] },
  { id: "260980100000123789", name: "新能源周期", owner: "绿能天下", total_return: 156.23, win_rate: 78.5, recent_returns: [3.2, 1.5, 2.8, -1.2, 4.1], follower_count: 287, created_at: "2022-11-15", win_count: 68, lose_count: 19, annualized_return: 42.8, max_drawdown: -22.5, volatility: 32.5, sharpe_ratio: 1.15, total_trades: 87, tags: ["新能源", "周期股"], sector_allocation: [{ name: "新能源", percentage: 55 }, { name: "科技", percentage: 25 }, { name: "环保", percentage: 20 }] },
  { id: "252220200000087432", name: "低估蓝筹", owner: "稳健收益", total_return: 32.15, win_rate: 72.4, recent_returns: [0.8, 0.3, -0.1, 1.2, 0.5], follower_count: 143, created_at: "2024-01-08", win_count: 25, lose_count: 10, annualized_return: 35.8, max_drawdown: -6.5, volatility: 8.5, sharpe_ratio: 3.52, total_trades: 35, tags: ["蓝筹", "低风险"], sector_allocation: [{ name: "金融", percentage: 40 }, { name: "消费", percentage: 30 }, { name: "基建", percentage: 30 }] },
  { id: "251890200000165432", name: "半导体机会", owner: "芯片之巅", total_return: 98.76, win_rate: 69.5, recent_returns: [1.8, -0.5, 2.2, 3.1, 0.8], follower_count: 178, created_at: "2023-04-22", win_count: 41, lose_count: 18, annualized_return: 35.2, max_drawdown: -20.8, volatility: 35.8, sharpe_ratio: 0.85, total_trades: 59, tags: ["半导体", "高波动"], sector_allocation: [{ name: "半导体", percentage: 65 }, { name: "科技", percentage: 25 }, { name: "AI", percentage: 10 }] },
  { id: "260550100000234567", name: "医药创新", owner: "健康使者", total_return: 72.38, win_rate: 74.2, recent_returns: [1.5, 0.8, 2.2, -0.5, 1.2], follower_count: 165, created_at: "2023-07-15", win_count: 38, lose_count: 13, annualized_return: 32.5, max_drawdown: -11.2, volatility: 16.8, sharpe_ratio: 1.62, total_trades: 51, tags: ["医药", "创新"], sector_allocation: [{ name: "医药", percentage: 70 }, { name: "科技", percentage: 20 }, { name: "消费", percentage: 10 }] },
  { id: "261050100000198765", name: "军工装备", owner: "国防先锋", total_return: 58.92, win_rate: 66.8, recent_returns: [2.8, -1.2, 0.5, 1.8, 2.5], follower_count: 98, created_at: "2023-11-20", win_count: 42, lose_count: 21, annualized_return: 28.5, max_drawdown: -14.5, volatility: 25.2, sharpe_ratio: 0.95, total_trades: 63, tags: ["军工", "国防"], sector_allocation: [{ name: "军工", percentage: 60 }, { name: "科技", percentage: 25 }, { name: "新能源", percentage: 15 }] },
  { id: "252350100000287654", name: "AI人工智能", owner: "智能时代", total_return: 185.45, win_rate: 79.3, recent_returns: [4.2, 2.5, 3.8, 1.2, 5.5], follower_count: 325, created_at: "2023-02-10", win_count: 75, lose_count: 20, annualized_return: 68.5, max_drawdown: -25.8, volatility: 42.5, sharpe_ratio: 1.45, total_trades: 95, tags: ["AI", "高科技"], sector_allocation: [{ name: "AI", percentage: 50 }, { name: "科技", percentage: 30 }, { name: "半导体", percentage: 20 }] },
  { id: "252650100000398765", name: "量子科技", owner: "未来已来", total_return: 125.68, win_rate: 76.5, recent_returns: [3.5, 2.8, 1.5, 4.2, 2.8], follower_count: 198, created_at: "2023-05-12", win_count: 58, lose_count: 18, annualized_return: 48.5, max_drawdown: -18.5, volatility: 32.8, sharpe_ratio: 1.28, total_trades: 76, tags: ["量子", "前沿"], sector_allocation: [{ name: "科技", percentage: 45 }, { name: "AI", percentage: 35 }, { name: "半导体", percentage: 20 }] },
  { id: "261120100000423456", name: "碳中和", owner: "绿色金融", total_return: 85.32, win_rate: 73.8, recent_returns: [1.8, 2.2, 0.8, 3.2, 1.5], follower_count: 142, created_at: "2023-09-18", win_count: 45, lose_count: 16, annualized_return: 38.2, max_drawdown: -12.5, volatility: 20.5, sharpe_ratio: 1.58, total_trades: 61, tags: ["碳中和", "环保"], sector_allocation: [{ name: "新能源", percentage: 40 }, { name: "环保", percentage: 35 }, { name: "基建", percentage: 25 }] },
  { id: "260890100000456789", name: "机器人", owner: "智造未来", total_return: 112.75, win_rate: 77.2, recent_returns: [2.8, 1.5, 3.5, 2.2, 4.5], follower_count: 185, created_at: "2023-06-25", win_count: 55, lose_count: 16, annualized_return: 45.8, max_drawdown: -16.2, volatility: 28.8, sharpe_ratio: 1.38, total_trades: 71, tags: ["机器人", "智能制造"], sector_allocation: [{ name: "科技", percentage: 50 }, { name: "AI", percentage: 30 }, { name: "新能源", percentage: 20 }] },
  { id: "260230100000512345", name: "云计算", owner: "数据先锋", total_return: 78.45, win_rate: 70.5, recent_returns: [2.2, 1.5, 0.8, 2.5, 1.8], follower_count: 155, created_at: "2023-08-15", win_count: 48, lose_count: 20, annualized_return: 35.5, max_drawdown: -14.8, volatility: 22.5, sharpe_ratio: 1.32, total_trades: 68, tags: ["云计算", "SaaS"], sector_allocation: [{ name: "科技", percentage: 55 }, { name: "AI", percentage: 25 }, { name: "基建", percentage: 20 }] },
  { id: "261340100000534567", name: "生物医药", owner: "创新药神", total_return: 95.85, win_rate: 75.8, recent_returns: [2.5, 1.8, 3.2, 1.2, 2.8], follower_count: 172, created_at: "2023-07-22", win_count: 52, lose_count: 17, annualized_return: 42.8, max_drawdown: -13.5, volatility: 19.8, sharpe_ratio: 1.85, total_trades: 69, tags: ["生物医药", "创新"], sector_allocation: [{ name: "医药", percentage: 75 }, { name: "科技", percentage: 15 }, { name: "消费", percentage: 10 }] },
  { id: "252410100000567890", name: "新材料", owner: "材料达人", total_return: 68.25, win_rate: 69.8, recent_returns: [1.8, 0.5, 2.5, 1.2, 2.2], follower_count: 88, created_at: "2023-12-05", win_count: 35, lose_count: 15, annualized_return: 58.5, max_drawdown: -10.2, volatility: 18.5, sharpe_ratio: 2.68, total_trades: 50, tags: ["新材料", "化工"], sector_allocation: [{ name: "科技", percentage: 35 }, { name: "新能源", percentage: 35 }, { name: "基建", percentage: 30 }] },
  { id: "260670100000589012", name: "5G通信", owner: "通信专家", total_return: 52.38, win_rate: 67.5, recent_returns: [1.2, 2.5, 0.8, 1.8, 0.5], follower_count: 105, created_at: "2024-01-18", win_count: 28, lose_count: 14, annualized_return: 68.5, max_drawdown: -8.5, volatility: 15.2, sharpe_ratio: 3.85, total_trades: 42, tags: ["5G", "通信"], sector_allocation: [{ name: "科技", percentage: 45 }, { name: "基建", percentage: 35 }, { name: "新能源", percentage: 20 }] },

  // ===== 我的组合（个人创建） =====
  { id: "my_001", name: "我的成长组合", owner: "我", total_return: 35.68, win_rate: 72.5, recent_returns: [1.5, 0.8, 2.2, 1.2, 0.5], follower_count: 0, created_at: "2024-02-01", win_count: 15, lose_count: 6, annualized_return: 48.5, max_drawdown: -12.5, volatility: 18.2, sharpe_ratio: 2.25, total_trades: 21, is_mine: true, tags: ["个人", "成长"], sector_allocation: [{ name: "科技", percentage: 40 }, { name: "AI", percentage: 30 }, { name: "医药", percentage: 30 }] },
  { id: "my_002", name: "稳健理财", owner: "我", total_return: 12.35, win_rate: 78.2, recent_returns: [0.5, 0.3, 0.8, 0.2, 0.6], follower_count: 0, created_at: "2024-01-15", win_count: 18, lose_count: 5, annualized_return: 18.5, max_drawdown: -5.2, volatility: 6.8, sharpe_ratio: 2.85, total_trades: 23, is_mine: true, tags: ["个人", "稳健"], sector_allocation: [{ name: "金融", percentage: 50 }, { name: "消费", percentage: 30 }, { name: "基建", percentage: 20 }] },

  // ===== 负收益组合（用于对比） =====
  { id: "260780100000312345", name: "元宇宙", owner: "虚拟世界", total_return: -12.35, win_rate: 45.2, recent_returns: [-2.5, -1.8, 0.5, -3.2, -0.8], follower_count: 45, created_at: "2023-08-05", win_count: 18, lose_count: 22, annualized_return: -8.5, max_drawdown: -35.2, volatility: 48.5, sharpe_ratio: -0.35, total_trades: 40, tags: ["元宇宙", "高风险"], sector_allocation: [{ name: "传媒", percentage: 50 }, { name: "科技", percentage: 30 }, { name: "AI", percentage: 20 }] },
  { id: "252980100000489012", name: "数字货币", owner: "区块链王", total_return: -25.68, win_rate: 38.5, recent_returns: [-4.5, -2.8, 1.2, -5.5, -3.2], follower_count: 32, created_at: "2023-10-08", win_count: 12, lose_count: 19, annualized_return: -35.2, max_drawdown: -55.8, volatility: 65.5, sharpe_ratio: -0.72, total_trades: 31, tags: ["数字货币", "高波动"], sector_allocation: [{ name: "科技", percentage: 100 }] },
];

// ==================== 主组件 ====================

export default function GroupPortfolioPanel() {
  const [portfolios, setPortfolios] = useState<Portfolio[]>([]);
  const [following, setFollowing] = useState<Set<string>>(new Set());
  const [selectedPortfolio, setSelectedPortfolio] = useState<Portfolio | null>(null);
  const [portfolioDetail, setPortfolioDetail] = useState<PortfolioDetail | null>(null);
  const [holdings, setHoldings] = useState<PortfolioHolding[]>([]);
  const [transferRecords, setTransferRecords] = useState<TransferRecord[]>([]);
  const [changes, setChanges] = useState<HoldingChange[]>([]);
  const [loading, setLoading] = useState(false);
  const [view, setView] = useState<'rank' | 'mine' | 'follow' | 'detail' | 'compare'>('rank');
  const [rankType, setRankType] = useState<'return' | 'annualized' | 'win_rate' | 'followers' | 'score' | 'dd'>('return');
  const [detailTab, setDetailTab] = useState<'holdings' | 'transfers' | 'stats' | 'sector' | 'lifecycle'>('holdings');
  const [notifications, setNotifications] = useState<{ id: string; message: string; type: string }[]>([]);
  const [isLive, setIsLive] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());
  const { openStock } = useStockClick();

  // 实时价格模拟
  useEffect(() => {
    if (!isLive) return;
    const interval = setInterval(() => {
      setHoldings(prev => prev.map(h => ({
        ...h,
        price: h.price * (1 + (Math.random() - 0.5) * 0.002),
        change_pct: h.change_pct + (Math.random() - 0.5) * 0.2,
      })));
      setLastUpdate(new Date());
    }, 3000);
    return () => clearInterval(interval);
  }, [isLive]);

  // 获取组合列表
  const fetchPortfolios = useCallback(async () => {
    setLoading(true);
    try {
      const resp = await fetch('/api/group/portfolios?page=1&page_size=50');
      const json = await resp.json();
      if (json.success && json.data.length > 0) {
        setPortfolios(json.data);
      } else {
        setPortfolios(generateMockPortfolios());
      }
    } catch (e) {
      setPortfolios(generateMockPortfolios());
    } finally {
      setLoading(false);
    }
  }, []);

  // 获取组合详情
  const fetchPortfolioDetail = useCallback(async (portfolio: Portfolio) => {
    const detail: PortfolioDetail = {
      portfolio,
      daily_return: portfolio.recent_returns?.[0] || 0,
      return_5d: portfolio.recent_returns?.slice(0, 5).reduce((a, b) => a + b, 0) || 0,
      return_20d: portfolio.recent_returns?.slice(0, 20).reduce((a, b) => a + b, 0) || 0,
      return_60d: portfolio.total_return * 0.6,
      return_250d: portfolio.total_return,
      max_drawdown: portfolio.max_drawdown || -10,
      hold_position_sum: '85%',
      total_capital: 1000000,
      available_capital: 150000,
      positions_count: portfolio.sector_allocation?.length || 5,
      sector_allocation: portfolio.sector_allocation,
      equity_curve: generateEquityCurve(portfolio.total_return, 180),
      recent_activities: generateMockActivities(),
    };
    setPortfolioDetail(detail);
  }, []);

  // 获取持仓
  const fetchHoldings = useCallback(async (portfolioId: string) => {
    const mockHoldings = generateMockHoldings(portfolioId);
    setHoldings(mockHoldings);
  }, []);

  // 获取调仓记录
  const fetchTransferRecords = useCallback(async (portfolioId: string) => {
    try {
      const resp = await fetch(`/api/group/transfers/${portfolioId}`);
      const json = await resp.json();
      if (json.success && json.data.length > 0) {
        setTransferRecords(json.data);
      } else {
        setTransferRecords(generateMockTransfers(portfolioId));
      }
    } catch (e) {
      setTransferRecords(generateMockTransfers(portfolioId));
    }
  }, []);

  // 关注组合
  const followPortfolio = async (id: string) => {
    setFollowing((prev) => new Set([...prev, id]));
    addNotification(`已关注组合`, 'follow');
  };

  // 取消关注
  const unfollowPortfolio = async (id: string) => {
    setFollowing((prev) => {
      const next = new Set(prev);
      next.delete(id);
      return next;
    });
    addNotification(`已取消关注`, 'unfollow');
  };

  // 查看组合详情
  const viewPortfolioDetail = async (portfolio: Portfolio) => {
    setSelectedPortfolio(portfolio);
    setView('detail');
    await Promise.all([
      fetchPortfolioDetail(portfolio),
      fetchHoldings(portfolio.id),
      fetchTransferRecords(portfolio.id),
    ]);
  };

  // 添加通知
  const addNotification = (message: string, type: string) => {
    const id = Date.now().toString();
    setNotifications((prev) => [...prev, { id, message, type }]);
    setTimeout(() => setNotifications((prev) => prev.filter((n) => n.id !== id)), 5000);
  };

  // 排序后的组合列表
  const sortedPortfolios = useMemo(() => {
    const filtered = portfolios.filter(p => {
      if (view === 'mine') return p.is_mine;
      if (view === 'follow') return following.has(p.id);
      return true;
    });

    return [...filtered].sort((a, b) => {
      switch (rankType) {
        case 'return': return b.total_return - a.total_return;
        case 'annualized': return (b.annualized_return || 0) - (a.annualized_return || 0);
        case 'win_rate': return b.win_rate - a.win_rate;
        case 'followers': return b.follower_count - a.follower_count;
        case 'score': return calculateScore(b) - calculateScore(a);
        case 'dd': return (b.max_drawdown || 0) - (a.max_drawdown || 0); // 回撤越小越好
        default: return 0;
      }
    });
  }, [portfolios, view, rankType, following]);

  // 板块统计数据
  const sectorStats = useMemo(() => {
    const stats: Record<string, { count: number; avgReturn: number }> = {};
    portfolios.forEach(p => {
      p.sector_allocation?.forEach(s => {
        if (!stats[s.name]) stats[s.name] = { count: 0, avgReturn: 0 };
        stats[s.name].count++;
        stats[s.name].avgReturn += p.total_return;
      });
    });
    return Object.entries(stats).map(([name, data]) => ({
      name,
      count: data.count,
      avgReturn: data.avgReturn / data.count,
      color: getSectorColor(name),
    })).sort((a, b) => b.count - a.count);
  }, [portfolios]);

  const formatPercent = (val: number) => `${val >= 0 ? '+' : ''}${val.toFixed(2)}%`;
  const getReturnColor = (val: number) => val >= 0 ? 'text-red-500' : 'text-green-500';
  const getScoreColor = (score: number) => score >= 30 ? 'text-green-400' : score >= 20 ? 'text-yellow-400' : 'text-red-400';

  useEffect(() => {
    fetchPortfolios();
  }, [fetchPortfolios]);

  // ==================== 渲染函数 ====================

  // 排行榜视图
  const renderRankView = () => (
    <>
      {/* 排行榜Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b bg-muted/30 shrink-0">
        <div className="flex items-center gap-2">
          <Trophy className="w-4 h-4 text-yellow-400" />
          <h2 className="text-sm font-semibold">组合排行榜</h2>
          <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
            {sortedPortfolios.length} 个
          </span>
        </div>
        <div className="flex items-center gap-1">
          <button onClick={() => setView('compare')} className="p-1.5 hover:bg-muted rounded" title="对比">
            <GitCompare className="w-3.5 h-3.5" />
          </button>
          <button onClick={fetchPortfolios} className="p-1.5 hover:bg-muted rounded" title="刷新">
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {/* 排行榜类型切换 */}
      <div className="flex border-b">
        {[
          { key: 'return', label: '总收益', icon: TrendingUp },
          { key: 'annualized', label: '年化', icon: TrendingUp },
          { key: 'win_rate', label: '胜率', icon: Target },
          { key: 'followers', label: '关注', icon: Users },
          { key: 'score', label: '综合', icon: Crown },
          { key: 'dd', label: '抗跌', icon: Shield },
        ].map(({ key, label, icon: Icon }) => (
          <button
            key={key}
            onClick={() => setRankType(key as typeof rankType)}
            className={`flex-1 flex items-center justify-center gap-1.5 py-2 text-xs font-medium transition-colors ${
              rankType === key ? 'border-b-2 border-primary text-primary bg-primary/5' : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            <Icon className="w-3 h-3" />
            {label}
          </button>
        ))}
      </div>

      {/* 板块分布概览 */}
      <div className="px-4 py-2 border-b bg-muted/20">
        <div className="flex items-center gap-2 mb-2">
          <PieChart className="w-3 h-3 text-muted-foreground" />
          <span className="text-[10px] text-muted-foreground">板块分布</span>
        </div>
        <div className="flex flex-wrap gap-1">
          {sectorStats.slice(0, 6).map(s => (
            <button
              key={s.name}
              className="flex items-center gap-1 px-2 py-0.5 rounded text-[10px] bg-muted hover:bg-muted/80 transition-colors"
              style={{ borderLeft: `2px solid ${s.color}` }}
            >
              <span>{s.name}</span>
              <span className="text-muted-foreground">({s.count})</span>
            </button>
          ))}
        </div>
      </div>

      {/* 排行榜列表 */}
      <div className="flex-1 overflow-y-auto">
        <table className="w-full text-xs">
          <thead className="text-muted-foreground bg-muted/50 sticky top-0">
            <tr>
              <th className="text-left px-3 py-2 font-medium w-8">#</th>
              <th className="text-left px-2 py-2 font-medium">组合</th>
              <th className="text-center px-2 py-2 font-medium w-16">走势</th>
              <th className="text-right px-2 py-2 font-medium">总收益</th>
              <th className="text-right px-2 py-2 font-medium">胜率</th>
              <th className="text-right px-2 py-2 font-medium">关注</th>
              <th className="text-center px-2 py-2 font-medium">标签</th>
            </tr>
          </thead>
          <tbody>
            {sortedPortfolios.map((portfolio, idx) => {
              const rank = idx + 1;
              const score = calculateScore(portfolio);
              return (
                <tr
                  key={portfolio.id}
                  className="border-b border-border/50 hover:bg-muted/30 cursor-pointer transition-colors"
                  onClick={() => viewPortfolioDetail(portfolio)}
                >
                  <td className="px-3 py-2.5">
                    {rank <= 3 ? (
                      <div className={`w-5 h-5 rounded flex items-center justify-center text-[10px] font-bold ${
                        rank === 1 ? 'bg-yellow-500 text-black' :
                        rank === 2 ? 'bg-gray-400 text-black' :
                        'bg-amber-700 text-white'
                      }`}>{rank}</div>
                    ) : (
                      <span className="text-muted-foreground">{rank}</span>
                    )}
                  </td>
                  <td className="px-2 py-2.5">
                    <div className="flex items-center gap-1">
                      <span className="font-medium">{portfolio.name}</span>
                      {portfolio.is_mine && <span className="px-1 py-0.5 bg-blue-500/20 text-blue-400 text-[8px] rounded">我</span>}
                      {following.has(portfolio.id) && <Star className="w-3 h-3 text-yellow-400 fill-yellow-400" />}
                    </div>
                    <div className="text-[10px] text-muted-foreground flex items-center gap-1">
                      <span>{portfolio.owner}</span>
                      <span className="text-muted-foreground/50">|</span>
                      <span>{portfolio.created_at}</span>
                    </div>
                  </td>
                  <td className="px-2 py-2.5">
                    {renderSparkline(portfolio.recent_returns)}
                  </td>
                  <td className={`text-right px-2 py-2.5 font-mono ${getReturnColor(portfolio.total_return)}`}>
                    <div className="font-medium">{formatPercent(portfolio.total_return)}</div>
                    <div className="text-[10px] text-muted-foreground">
                      年化 {formatPercent(portfolio.annualized_return || 0)}
                    </div>
                  </td>
                  <td className="text-right px-2 py-2.5">
                    <span className={portfolio.win_rate >= 70 ? 'text-green-400' : portfolio.win_rate >= 50 ? 'text-yellow-400' : 'text-red-400'}>
                      {portfolio.win_rate.toFixed(1)}%
                    </span>
                    <div className="text-[10px] text-muted-foreground">
                      {portfolio.win_count}胜/{portfolio.lose_count}负
                    </div>
                  </td>
                  <td className="text-right px-2 py-2.5 text-muted-foreground">
                    {portfolio.follower_count}
                  </td>
                  <td className="px-2 py-2.5">
                    <div className="flex flex-wrap gap-0.5 justify-center">
                      {portfolio.tags?.slice(0, 2).map(tag => (
                        <span key={tag} className="px-1 py-0.5 bg-muted text-[9px] rounded">
                          {tag}
                        </span>
                      ))}
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </>
  );

  // 我的组合视图
  const renderMineView = () => (
    <>
      <div className="flex items-center justify-between px-4 py-2.5 border-b bg-blue-500/10 shrink-0">
        <div className="flex items-center gap-2">
          <Briefcase className="w-4 h-4 text-blue-400" />
          <h2 className="text-sm font-semibold">我的组合</h2>
        </div>
        <button className="flex items-center gap-1 px-2 py-1 bg-blue-500 text-white text-xs rounded hover:bg-blue-600">
          <Plus className="w-3 h-3" />
          新建
        </button>
      </div>

      {/* 资产概览 */}
      <div className="grid grid-cols-4 gap-2 p-4 border-b bg-muted/30">
        <div className="text-center">
          <div className="text-[10px] text-muted-foreground">总资产</div>
          <div className="text-sm font-mono font-bold">¥1,235,680</div>
        </div>
        <div className="text-center">
          <div className="text-[10px] text-muted-foreground">今日收益</div>
          <div className="text-sm font-mono font-bold text-red-500">+¥8,450</div>
        </div>
        <div className="text-center">
          <div className="text-[10px] text-muted-foreground">持仓仓位</div>
          <div className="text-sm font-mono font-bold">78.5%</div>
        </div>
        <div className="text-center">
          <div className="text-[10px] text-muted-foreground">组合数量</div>
          <div className="text-sm font-mono font-bold">2</div>
        </div>
      </div>

      {/* 我的组合列表 */}
      <div className="flex-1 overflow-y-auto">
        {sortedPortfolios.filter(p => p.is_mine).map((portfolio) => (
          <div
            key={portfolio.id}
            className="p-4 border-b border-border/50 hover:bg-muted/30 cursor-pointer transition-colors"
            onClick={() => viewPortfolioDetail(portfolio)}
          >
            <div className="flex items-start justify-between mb-2">
              <div>
                <div className="flex items-center gap-2">
                  <span className="font-medium">{portfolio.name}</span>
                  <span className="px-1.5 py-0.5 bg-blue-500/20 text-blue-400 text-[10px] rounded">
                    {portfolio.tags?.join(',')}
                  </span>
                </div>
                <div className="text-[10px] text-muted-foreground mt-0.5">
                  创建于 {portfolio.created_at}
                </div>
              </div>
              <div className="text-right">
                <div className={`text-lg font-mono font-bold ${getReturnColor(portfolio.total_return)}`}>
                  {formatPercent(portfolio.total_return)}
                </div>
                <div className="text-[10px] text-muted-foreground">
                  ¥{(1000000 * (1 + portfolio.total_return / 100)).toLocaleString()}
                </div>
              </div>
            </div>

            {/* 简易收益曲线 */}
            <div className="h-8 mt-2">
              <ChartContainer config={{}}>
                <AreaChart data={generateEquityCurve(portfolio.total_return, 30).map((d, i) => ({
                  ...d,
                  v: d.value,
                }))}>
                  <Area type="monotone" dataKey="v" stroke={portfolio.total_return >= 0 ? '#ef4444' : '#22c55e'} fill={portfolio.total_return >= 0 ? '#ef444420' : '#22c55e20'} strokeWidth={1.5} />
                </AreaChart>
              </ChartContainer>
            </div>

            {/* 板块分布 */}
            <div className="flex items-center gap-2 mt-2">
              <span className="text-[10px] text-muted-foreground">板块:</span>
              <div className="flex gap-1">
                {portfolio.sector_allocation?.slice(0, 3).map(s => (
                  <span key={s.name} className="text-[10px] px-1 py-0.5 rounded" style={{ backgroundColor: getSectorColor(s.name) + '20', color: getSectorColor(s.name) }}>
                    {s.name} {s.percentage}%
                  </span>
                ))}
              </div>
            </div>
          </div>
        ))}

        {sortedPortfolios.filter(p => p.is_mine).length === 0 && (
          <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <Briefcase className="w-12 h-12 opacity-30 mb-3" />
            <p>暂无个人组合</p>
            <button className="mt-3 flex items-center gap-1 px-3 py-1.5 bg-primary text-primary-foreground text-xs rounded hover:bg-primary/90">
              <Plus className="w-3 h-3" />
              创建第一个组合
            </button>
          </div>
        )}
      </div>
    </>
  );

  // 详情视图
  const renderDetailView = () => (
    <>
      {/* 详情Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b bg-muted/30 shrink-0">
        <div className="flex items-center gap-2">
          <button onClick={() => setView('rank')} className="p-1 hover:bg-muted rounded">
            <TrendingDown className="w-4 h-4" />
          </button>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-sm font-semibold">{selectedPortfolio?.name}</h2>
              {selectedPortfolio?.is_mine && <span className="px-1.5 py-0.5 bg-blue-500/20 text-blue-400 text-[10px] rounded">我的</span>}
            </div>
            <div className="text-[10px] text-muted-foreground">
              by {selectedPortfolio?.owner} | 创建于 {selectedPortfolio?.created_at}
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <span className={`text-sm font-mono font-bold ${getReturnColor(selectedPortfolio?.total_return || 0)}`}>
            {formatPercent(selectedPortfolio?.total_return || 0)}
          </span>
          <button
            onClick={() => following.has(selectedPortfolio?.id || '') ? unfollowPortfolio(selectedPortfolio?.id || '') : followPortfolio(selectedPortfolio?.id || '')}
            className={`p-1.5 rounded ${following.has(selectedPortfolio?.id || '') ? 'text-yellow-400' : 'text-muted-foreground hover:text-yellow-400'}`}
          >
            <Star className={`w-4 h-4 ${following.has(selectedPortfolio?.id || '') ? 'fill-current' : ''}`} />
          </button>
          <button
            onClick={() => setIsLive(!isLive)}
            className={`p-1.5 rounded ${isLive ? 'text-green-400 bg-green-400/20' : 'text-muted-foreground hover:text-green-400'}`}
            title={isLive ? '实时更新中...' : '开启实时更新'}
          >
            {isLive ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4" />}
          </button>
        </div>
      </div>

      {/* 收益概览 */}
      {portfolioDetail && (
        <div className="grid grid-cols-5 gap-1 px-4 py-2 border-b">
          {[
            { label: '日收益', value: portfolioDetail.daily_return },
            { label: '5日', value: portfolioDetail.return_5d },
            { label: '20日', value: portfolioDetail.return_20d },
            { label: '60日', value: portfolioDetail.return_60d },
            { label: '250日', value: portfolioDetail.return_250d },
          ].map(item => (
            <div key={item.label} className="text-center bg-muted/30 py-1.5 rounded">
              <div className="text-[9px] text-muted-foreground">{item.label}</div>
              <div className={`text-xs font-mono font-medium ${getReturnColor(item.value)}`}>
                {formatPercent(item.value)}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Tab切换 */}
      <div className="flex border-b">
        {[
          { key: 'holdings', label: '持仓', icon: Briefcase },
          { key: 'transfers', label: '调仓', icon: ArrowUpDown },
          { key: 'stats', label: '统计', icon: BarChart2 },
          { key: 'sector', label: '板块', icon: PieChart },
          { key: 'lifecycle', label: '周期', icon: Clock },
        ].map(({ key, label, icon: Icon }) => (
          <button
            key={key}
            onClick={() => setDetailTab(key as typeof detailTab)}
            className={`flex-1 flex items-center justify-center gap-1 py-2 text-xs font-medium transition-colors ${
              detailTab === key ? 'border-b-2 border-primary text-primary' : 'text-muted-foreground'
            }`}
          >
            <Icon className="w-3 h-3" />
            {label}
          </button>
        ))}
      </div>

      {/* 持仓Tab */}
      {detailTab === 'holdings' && (
        <div className="flex-1 overflow-y-auto">
          <table className="w-full text-xs">
            <thead className="text-muted-foreground bg-muted/50 sticky top-0">
              <tr>
                <th className="text-left px-2 py-2 font-medium">股票</th>
                <th className="text-right px-2 py-2 font-medium">现价</th>
                <th className="text-right px-2 py-2 font-medium">涨跌幅</th>
                <th className="text-right px-2 py-2 font-medium">成本</th>
                <th className="text-right px-2 py-2 font-medium">盈亏</th>
              </tr>
            </thead>
            <tbody>
              {holdings.map((h) => (
                <tr key={h.symbol} className="border-b border-border/50 hover:bg-muted/30 cursor-pointer" onClick={() => openStock(h.symbol, h.name)}>
                  <td className="px-2 py-2">
                    <div className="font-medium">{h.name}</div>
                    <div className="text-[10px] text-muted-foreground">{h.symbol}</div>
                  </td>
                  <td className="text-right px-2 py-2 font-mono">¥{h.price.toFixed(2)}</td>
                  <td className={`text-right px-2 py-2 font-mono ${getReturnColor(h.change_pct)}`}>{formatPercent(h.change_pct)}</td>
                  <td className="text-right px-2 py-2 font-mono">¥{h.cost.toFixed(2)}</td>
                  <td className={`text-right px-2 py-2 font-mono ${getReturnColor(h.profit_ratio)}`}>{formatPercent(h.profit_ratio)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* 调仓Tab - 时间轴曲线 */}
      {detailTab === 'transfers' && (
        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {/* 时间轴曲线图 */}
          <div className="bg-card border rounded-lg p-3">
            <div className="text-xs font-medium mb-3 flex items-center gap-2">
              <ArrowUpDown className="w-3 h-3" /> 调仓时间轴
              <span className="text-[9px] text-muted-foreground ml-auto">红↑买 绿↓卖</span>
            </div>
            <ChartContainer config={{}} className="h-56">
              <BarChart data={transferRecords.map((r, idx) => ({
                date: r.time.split(' ')[0].slice(5),
                fullTime: r.time,
                value: r.action === '买' ? r.price : -r.price,
                action: r.action,
                name: r.name,
                symbol: r.symbol,
                price: r.price,
                shares: r.shares,
              })).reverse()}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                <XAxis dataKey="date" tick={{ fontSize: 9 }} tickLine={false} />
                <YAxis tick={{ fontSize: 9 }} tickLine={false} domain={['auto', 'auto']} />
                <Tooltip
                  contentStyle={{ background: 'var(--background)', border: '1px solid var(--border)', borderRadius: '8px', fontSize: '12px' }}
                  formatter={(value: any, color: string, props: any) => {
                    const d = props.payload;
                    return [
                      <span key="tip" className="flex flex-col gap-1">
                        <span className="font-medium">{d.name} ({d.symbol})</span>
                        <span className={d.action === '买' ? 'text-red-400' : 'text-green-400'}>
                          {d.action === '买' ? '买入' : '卖出'} ¥{d.price.toFixed(2)} × {d.shares}
                        </span>
                        <span className="text-muted-foreground text-[10px]">{d.fullTime}</span>
                      </span>,
                      ''
                    ];
                  }}
                />
                <ReferenceLine y={0} stroke="var(--muted-foreground)" strokeDasharray="3 3" />
                <Bar
                  dataKey="value"
                  radius={[4, 4, 0, 0]}
                  maxBarSize={40}
                  shape={(props: any) => {
                    const { x, y, width, height, payload } = props;
                    const isBuy = payload.action === '买';
                    const fill = isBuy ? '#ef4444' : '#22c55e';
                    return <Rectangle x={x} y={y} width={width} height={height} fill={fill} radius={[4, 4, 0, 0]} />;
                  }}
                />
              </BarChart>
            </ChartContainer>
          </div>

          {/* 图例说明 */}
          <div className="flex items-center justify-center gap-4 text-[10px]">
            <div className="flex items-center gap-1">
              <div className="w-3 h-3 rounded bg-red-500" />
              <span className="text-red-400">买入 (上方)</span>
            </div>
            <div className="flex items-center gap-1">
              <div className="w-3 h-3 rounded bg-green-500" />
              <span className="text-green-400">卖出 (下方)</span>
            </div>
          </div>

          {/* 调仓列表 */}
          <div className="bg-card border rounded-lg">
            <div className="px-3 py-2 border-b bg-muted/30">
              <div className="text-xs font-medium">调仓明细</div>
            </div>
            <div className="divide-y">
              {transferRecords.map((r, idx) => (
                <div
                  key={idx}
                  className="flex items-center gap-3 px-3 py-2.5 hover:bg-muted/30 cursor-pointer transition-colors"
                  onClick={() => openStock(r.symbol, r.name)}
                >
                  {/* 操作图标 */}
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center ${
                    r.action === '买' ? 'bg-red-500/20 text-red-400' : 'bg-green-500/20 text-green-400'
                  }`}>
                    {r.action === '买' ? <ArrowUp className="w-4 h-4" /> : <ArrowDown className="w-4 h-4" />}
                  </div>

                  {/* 股票信息 */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-sm truncate">{r.name}</span>
                      <span className="text-[10px] text-muted-foreground">{r.symbol}</span>
                    </div>
                    <div className="text-[10px] text-muted-foreground">{r.time}</div>
                  </div>

                  {/* 价格数量 */}
                  <div className="text-right">
                    <div className="font-mono text-sm">¥{r.price.toFixed(2)}</div>
                    <div className="text-[10px] text-muted-foreground">
                      {r.action === '买' ? '买入' : '卖出'} {r.shares}股
                    </div>
                  </div>

                  {/* 金额 */}
                  <div className="text-right w-20">
                    <div className={`font-mono text-sm font-medium ${r.action === '买' ? 'text-red-400' : 'text-green-400'}`}>
                      {r.action === '买' ? '-' : '+'}¥{((r.price * (r.shares || 0))).toFixed(0)}
                    </div>
                    <div className="text-[9px] text-muted-foreground">
                      总额
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* 统计Tab */}
      {detailTab === 'stats' && portfolioDetail && (
        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {/* 收益曲线 */}
          <div className="bg-card border rounded-lg p-3">
            <div className="flex items-center justify-between mb-2">
              <div className="text-xs font-medium">收益走势 (vs 沪深300)</div>
              {isLive && (
                <div className="flex items-center gap-1 text-[9px] text-green-400">
                  <span className="w-1.5 h-1.5 bg-green-400 rounded-full animate-pulse" />
                  {lastUpdate.toLocaleTimeString()}
                </div>
              )}
            </div>
            <ChartContainer config={{}} className="h-44">
              <AreaChart data={portfolioDetail.equity_curve || []}>
                <defs>
                  <linearGradient id="equityGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                <XAxis dataKey="date" tick={{ fontSize: 9 }} tickLine={false} />
                <YAxis tick={{ fontSize: 9 }} tickLine={false} />
                <Tooltip contentStyle={{ background: 'var(--background)', border: '1px solid var(--border)', borderRadius: '8px', fontSize: '12px' }} />
                <Area type="monotone" dataKey="value" stroke="#3b82f6" fill="url(#equityGradient)" name="组合" strokeWidth={2} />
                <Line type="monotone" dataKey="benchmark" stroke="#ef4444" strokeDasharray="5 5" strokeWidth={1} name="沪深300" />
              </AreaChart>
            </ChartContainer>
          </div>

          {/* 收益指标 */}
          <div className="bg-card border rounded-lg p-3">
            <div className="text-xs font-medium mb-2 flex items-center gap-1">
              <TrendingUp className="w-3 h-3" /> 收益指标
            </div>
            <div className="grid grid-cols-4 gap-2">
              <div className="bg-muted/30 p-2 rounded text-center">
                <div className="text-[9px] text-muted-foreground">总收益</div>
                <div className={`text-sm font-mono font-bold ${getReturnColor(selectedPortfolio?.total_return || 0)}`}>
                  {formatPercent(selectedPortfolio?.total_return || 0)}
                </div>
              </div>
              <div className="bg-muted/30 p-2 rounded text-center">
                <div className="text-[9px] text-muted-foreground">年化</div>
                <div className={`text-sm font-mono font-bold ${getReturnColor(selectedPortfolio?.annualized_return || 0)}`}>
                  {formatPercent(selectedPortfolio?.annualized_return || 0)}
                </div>
              </div>
              <div className="bg-muted/30 p-2 rounded text-center">
                <div className="text-[9px] text-muted-foreground">近60日</div>
                <div className={`text-sm font-mono font-bold ${getReturnColor(portfolioDetail.return_60d)}`}>
                  {formatPercent(portfolioDetail.return_60d)}
                </div>
              </div>
              <div className="bg-muted/30 p-2 rounded text-center">
                <div className="text-[9px] text-muted-foreground">近250日</div>
                <div className={`text-sm font-mono font-bold ${getReturnColor(portfolioDetail.return_250d)}`}>
                  {formatPercent(portfolioDetail.return_250d)}
                </div>
              </div>
            </div>
          </div>

          {/* 风险指标 */}
          <div className="bg-card border rounded-lg p-3">
            <div className="text-xs font-medium mb-2 flex items-center gap-1">
              <Shield className="w-3 h-3" /> 风险指标
            </div>
            <div className="grid grid-cols-4 gap-2">
              <div className="bg-muted/30 p-2 rounded text-center">
                <div className="text-[9px] text-muted-foreground">最大回撤</div>
                <div className="text-sm font-mono font-bold text-red-400">
                  {formatPercent(selectedPortfolio?.max_drawdown || 0)}
                </div>
              </div>
              <div className="bg-muted/30 p-2 rounded text-center">
                <div className="text-[9px] text-muted-foreground">波动率</div>
                <div className="text-sm font-mono font-bold">
                  {formatPercent(selectedPortfolio?.volatility || 0)}
                </div>
              </div>
              <div className="bg-muted/30 p-2 rounded text-center">
                <div className="text-[9px] text-muted-foreground">夏普比率</div>
                <div className={`text-sm font-mono font-bold ${(selectedPortfolio?.sharpe_ratio || 0) >= 1 ? 'text-green-400' : 'text-yellow-400'}`}>
                  {(selectedPortfolio?.sharpe_ratio || 0).toFixed(2)}
                </div>
              </div>
              <div className="bg-muted/30 p-2 rounded text-center">
                <div className="text-[9px] text-muted-foreground">胜率</div>
                <div className={`text-sm font-mono font-bold ${(selectedPortfolio?.win_rate || 0) >= 70 ? 'text-green-400' : 'text-yellow-400'}`}>
                  {selectedPortfolio?.win_rate?.toFixed(1)}%
                </div>
              </div>
            </div>
          </div>

          {/* 交易统计 */}
          <div className="bg-card border rounded-lg p-3">
            <div className="text-xs font-medium mb-2 flex items-center gap-1">
              <Activity className="w-3 h-3" /> 交易统计
            </div>
            <div className="grid grid-cols-3 gap-2">
              <div className="bg-muted/30 p-2 rounded text-center">
                <div className="text-[9px] text-muted-foreground">总交易</div>
                <div className="text-sm font-mono font-bold">{selectedPortfolio?.total_trades || 0}笔</div>
              </div>
              <div className="bg-muted/30 p-2 rounded text-center">
                <div className="text-[9px] text-muted-foreground">盈利</div>
                <div className="text-sm font-mono font-bold text-green-400">{selectedPortfolio?.win_count || 0}笔</div>
              </div>
              <div className="bg-muted/30 p-2 rounded text-center">
                <div className="text-[9px] text-muted-foreground">亏损</div>
                <div className="text-sm font-mono font-bold text-red-400">{selectedPortfolio?.lose_count || 0}笔</div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 板块Tab */}
      {detailTab === 'sector' && portfolioDetail?.sector_allocation && (
        <div className="flex-1 overflow-y-auto p-4">
          <div className="bg-card border rounded-lg p-4">
            <div className="text-xs font-medium mb-3">板块配置</div>
            <div className="flex gap-4">
              {/* 饼图 */}
              <div className="w-1/2">
                <ChartContainer config={{}}>
                  <RechartsPie>
                    <Pie
                      data={portfolioDetail.sector_allocation.map(s => ({
                        name: s.name,
                        value: s.percentage,
                      }))}
                      cx="50%"
                      cy="50%"
                      innerRadius={40}
                      outerRadius={70}
                      paddingAngle={2}
                      dataKey="value"
                    >
                      {portfolioDetail.sector_allocation.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={getSectorColor(entry.name)} />
                      ))}
                    </Pie>
                    <Tooltip contentStyle={{ background: 'var(--background)', border: '1px solid var(--border)', borderRadius: '8px', fontSize: '12px' }} />
                  </RechartsPie>
                </ChartContainer>
              </div>
              {/* 列表 */}
              <div className="flex-1 space-y-2">
                {portfolioDetail.sector_allocation.map(s => (
                  <div key={s.name} className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <div className="w-2 h-2 rounded-full" style={{ backgroundColor: getSectorColor(s.name) }} />
                      <span className="text-xs">{s.name}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-mono">{s.percentage}%</span>
                      <div className="w-16 h-1.5 bg-muted rounded-full overflow-hidden">
                        <div className="h-full rounded-full" style={{ width: `${s.percentage}%`, backgroundColor: getSectorColor(s.name) }} />
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 生命周期Tab */}
      {detailTab === 'lifecycle' && (
        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {holdings.map(h => {
            const lifecycleColors: Record<string, string> = {
              '建仓': 'bg-blue-500',
              '拉升': 'bg-green-500',
              '持有': 'bg-yellow-500',
              '成熟': 'bg-purple-500',
              '减仓': 'bg-red-500',
            };
            return (
              <div key={h.symbol} className="bg-card border rounded-lg p-3">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <div>
                      <div className="font-medium text-sm">{h.name}</div>
                      <div className="text-[10px] text-muted-foreground">{h.symbol}</div>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="text-right">
                      <div className={`text-sm font-mono font-bold ${getReturnColor(h.profit_ratio)}`}>
                        {formatPercent(h.profit_ratio)}
                      </div>
                      <div className="text-[9px] text-muted-foreground">
                        ¥{((h.price - h.cost) * h.shares).toFixed(0)}
                      </div>
                    </div>
                    <span className={`px-2 py-0.5 rounded text-[10px] text-white ${lifecycleColors[h.lifecycle_stage || '持有']}`}>
                      {h.lifecycle_stage || '持有'}
                    </span>
                  </div>
                </div>

                {/* 生命周期进度条 */}
                <div className="mb-3">
                  <div className="flex items-center justify-between text-[9px] text-muted-foreground mb-1">
                    <span>建仓</span><span>拉升</span><span>持有</span><span>成熟</span><span>减仓</span>
                  </div>
                  <div className="relative h-2 bg-muted rounded-full overflow-hidden">
                    {/* 进度条背景段 */}
                    <div className="absolute inset-0 flex">
                      {[0, 1, 2, 3, 4].map(i => (
                        <div key={i} className="flex-1 border-r border-background last:border-r-0" />
                      ))}
                    </div>
                    {/* 当前位置 */}
                    <div
                      className={`absolute top-0 bottom-0 ${lifecycleColors[h.lifecycle_stage || '持有'].replace('bg-', 'bg-opacity-60 bg-')}`}
                      style={{
                        left: `${(h.hold_days < 30 ? 0 : h.hold_days < 60 ? 20 : h.hold_days < 120 ? 40 : h.hold_days < 180 ? 60 : 80)}%`,
                        width: '20%'
                      }}
                    />
                  </div>
                  <div className="text-[9px] text-muted-foreground mt-1 text-center">
                    已持有 <span className="font-mono text-foreground">{h.hold_days}</span> 天
                  </div>
                </div>

                <div className="grid grid-cols-4 gap-2 text-xs">
                  <div className="bg-muted/30 p-1.5 rounded">
                    <div className="text-[9px] text-muted-foreground">持仓</div>
                    <div className="font-mono text-[10px]">{h.shares.toFixed(0)}股</div>
                  </div>
                  <div className="bg-muted/30 p-1.5 rounded">
                    <div className="text-[9px] text-muted-foreground">成本价</div>
                    <div className="font-mono text-[10px]">¥{h.cost.toFixed(2)}</div>
                  </div>
                  <div className="bg-muted/30 p-1.5 rounded">
                    <div className="text-[9px] text-muted-foreground">现价</div>
                    <div className="font-mono text-[10px]">¥{h.price.toFixed(2)}</div>
                  </div>
                  <div className="bg-muted/30 p-1.5 rounded">
                    <div className="text-[9px] text-muted-foreground">市值</div>
                    <div className="font-mono text-[10px]">¥{(h.price * h.shares).toFixed(0)}</div>
                  </div>
                </div>

                {h.sector && (
                  <div className="mt-2 flex items-center gap-1">
                    <span className="text-[9px] px-1.5 py-0.5 rounded" style={{ backgroundColor: getSectorColor(h.sector) + '20', color: getSectorColor(h.sector) }}>
                      {h.sector}
                    </span>
                  </div>
                )}
              </div>
            );
          })}

          {/* 相关组合推荐 */}
          <div className="border-t pt-3 mt-3">
            <div className="text-xs font-medium mb-2 flex items-center gap-1">
              <Users className="w-3 h-3" /> 相关组合
            </div>
            <div className="grid grid-cols-2 gap-2">
              {portfolios
                .filter(p => p.id !== selectedPortfolio?.id && p.sector_allocation?.some(s =>
                  selectedPortfolio?.sector_allocation?.some(ss => ss.name === s.name)
                ))
                .slice(0, 4)
                .map(p => (
                  <div
                    key={p.id}
                    className="bg-muted/30 p-2 rounded cursor-pointer hover:bg-muted/50 transition-colors"
                    onClick={() => viewPortfolioDetail(p)}
                  >
                    <div className="text-[10px] font-medium truncate">{p.name}</div>
                    <div className="flex items-center justify-between mt-1">
                      <span className={`text-[10px] font-mono ${getReturnColor(p.total_return)}`}>
                        {formatPercent(p.total_return)}
                      </span>
                      <span className="text-[9px] text-muted-foreground">{p.win_rate.toFixed(0)}%胜</span>
                    </div>
                  </div>
                ))}
            </div>
          </div>
        </div>
      )}
    </>
  );

  // 对比视图
  const renderCompareView = () => {
    const comparePortfolios = sortedPortfolios.slice(0, 5);
    // 雷达图数据 - 标准化到0-100
    const radarData = useMemo(() => {
      return comparePortfolios.map(p => ({
        name: p.name.slice(0, 5),
        // 收益 (越高越好，直接用)
        收益: Math.min(Math.max(p.total_return + 30, 0), 100),
        // 胜率 (直接用)
        胜率: p.win_rate,
        // 夏普比率 (标准化，0-2 -> 0-100)
        夏普: Math.min(Math.max((p.sharpe_ratio || 0) * 40, 0), 100),
        // 抗跌 (回撤越小越好，0到-50 -> 100到0)
        抗跌: Math.min(Math.max((p.max_drawdown || 0) + 60, 0), 100),
        // 稳定性 (波动率越小越好，0-50 -> 100-0)
        稳定: Math.max(100 - (p.volatility || 0) * 2, 0),
        // 人气 (关注人数标准化)
        人气: Math.min((p.follower_count / 3), 100),
      }));
    }, [comparePortfolios]);

    return (
      <>
        <div className="flex items-center justify-between px-4 py-2.5 border-b shrink-0">
          <div className="flex items-center gap-2">
            <button onClick={() => setView('rank')} className="p-1 hover:bg-muted rounded">
              <TrendingDown className="w-4 h-4" />
            </button>
            <h2 className="text-sm font-semibold">组合对比</h2>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {/* 雷达图综合对比 */}
          <div className="bg-card border rounded-lg p-3">
            <div className="text-xs font-medium mb-3">综合能力雷达图</div>
            <ChartContainer config={{}} className="h-64">
              <RadarChart cx="50%" cy="50%" outerRadius="70%">
                <PolarGrid stroke="var(--border)" />
                <PolarAngleAxis dataKey="name" tick={{ fontSize: 9 }} />
                <PolarRadiusAxis angle={30} domain={[0, 100]} tick={{ fontSize: 8 }} />
                {comparePortfolios.map((p, idx) => {
                  const colors = ['#3b82f6', '#ef4444', '#22c55e', '#f59e0b', '#8b5cf6'];
                  return (
                    <Radar
                      key={p.id}
                      name={p.name.slice(0, 5)}
                      dataKey={p.name.slice(0, 5)}
                      stroke={colors[idx]}
                      fill={colors[idx]}
                      fillOpacity={0.15}
                    />
                  );
                })}
                <Legend wrapperStyle={{ fontSize: 9 }} />
                <Tooltip contentStyle={{ background: 'var(--background)', border: '1px solid var(--border)', borderRadius: '8px', fontSize: '12px' }} />
              </RadarChart>
            </ChartContainer>
          </div>

          {/* 收益对比柱状图 */}
          <div className="bg-card border rounded-lg p-3">
            <div className="text-xs font-medium mb-3">收益对比</div>
            <ChartContainer config={{}} className="h-40">
              <BarChart data={comparePortfolios.map(p => ({
                name: p.name.slice(0, 5),
                总收益: p.total_return,
                年化: p.annualized_return || 0,
              }))}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                <XAxis dataKey="name" tick={{ fontSize: 10 }} />
                <YAxis tick={{ fontSize: 10 }} />
                <Tooltip contentStyle={{ background: 'var(--background)', border: '1px solid var(--border)', borderRadius: '8px', fontSize: '12px' }} />
                <Bar dataKey="总收益" fill="#ef4444" radius={[4, 4, 0, 0]} />
                <Bar dataKey="年化" fill="#3b82f6" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ChartContainer>
          </div>

          {/* 胜率与风险对比 */}
          <div className="grid grid-cols-2 gap-3">
            <div className="bg-card border rounded-lg p-3">
              <div className="text-xs font-medium mb-2">胜率对比</div>
              <ChartContainer config={{}} className="h-32">
                <BarChart data={comparePortfolios.map(p => ({
                  name: p.name.slice(0, 4),
                  胜率: p.win_rate,
                }))} layout="vertical">
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                  <XAxis type="number" domain={[0, 100]} tick={{ fontSize: 9 }} />
                  <YAxis type="category" dataKey="name" tick={{ fontSize: 8 }} width={35} />
                  <Tooltip contentStyle={{ background: 'var(--background)', border: '1px solid var(--border)', borderRadius: '8px', fontSize: '12px' }} />
                  <Bar dataKey="胜率" fill="#22c55e" radius={[0, 4, 4, 0]} />
                </BarChart>
              </ChartContainer>
            </div>
            <div className="bg-card border rounded-lg p-3">
              <div className="text-xs font-medium mb-2">最大回撤</div>
              <ChartContainer config={{}} className="h-32">
                <BarChart data={comparePortfolios.map(p => ({
                  name: p.name.slice(0, 4),
                  回撤: p.max_drawdown || 0,
                }))} layout="vertical">
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                  <XAxis type="number" domain={[-60, 0]} tick={{ fontSize: 9 }} />
                  <YAxis type="category" dataKey="name" tick={{ fontSize: 8 }} width={35} />
                  <Tooltip contentStyle={{ background: 'var(--background)', border: '1px solid var(--border)', borderRadius: '8px', fontSize: '12px' }} />
                  <Bar dataKey="回撤" fill="#f59e0b" radius={[0, 4, 4, 0]} />
                </BarChart>
              </ChartContainer>
            </div>
          </div>

          {/* 板块分布对比 */}
          <div className="bg-card border rounded-lg p-3">
            <div className="text-xs font-medium mb-3">板块配置对比</div>
            <div className="grid grid-cols-3 gap-2">
              {comparePortfolios.slice(0, 3).map(p => (
                <div key={p.id} className="bg-muted/30 p-2 rounded">
                  <div className="text-[10px] font-medium mb-2 truncate">{p.name}</div>
                  <div className="space-y-1">
                    {p.sector_allocation?.slice(0, 3).map(s => (
                      <div key={s.name} className="flex items-center gap-1 text-[9px]">
                        <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: getSectorColor(s.name) }} />
                        <span className="flex-1 truncate">{s.name}</span>
                        <span className="text-muted-foreground">{s.percentage}%</span>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </>
    );
  };

  return (
    <div className="flex flex-col h-full relative">
      {/* 通知 */}
      <div className="absolute top-2 right-2 z-50 space-y-1 max-w-xs">
        {notifications.map(n => (
          <div key={n.id} className={`px-3 py-2 rounded text-xs shadow-lg animate-in slide-in-from-right ${
            n.type === 'follow' ? 'bg-blue-900/90 text-blue-200' : 'bg-gray-900/90 text-gray-200'
          }`}>
            {n.message}
          </div>
        ))}
      </div>

      {/* 顶部视图切换 */}
      <div className="flex border-b bg-muted/50 shrink-0">
        {[
          { key: 'rank', label: '排行榜', icon: Trophy },
          { key: 'mine', label: '我的组合', icon: Briefcase },
          { key: 'follow', label: '关注', icon: Star },
        ].map(({ key, label, icon: Icon }) => (
          <button
            key={key}
            onClick={() => { setView(key as typeof view); }}
            className={`flex-1 flex items-center justify-center gap-1.5 py-2 text-xs font-medium transition-colors ${
              view === key ? 'bg-background border-b-2 border-primary text-primary' : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            <Icon className="w-3.5 h-3.5" />
            {label}
            {key === 'follow' && following.size > 0 && (
              <span className="ml-1 px-1 py-0.5 bg-primary/20 text-primary text-[10px] rounded-full">
                {following.size}
              </span>
            )}
          </button>
        ))}
      </div>

      {/* 主内容 */}
      {view === 'rank' && renderRankView()}
      {view === 'mine' && renderMineView()}
      {view === 'follow' && (
        <div className="flex-1 flex flex-col">
          <div className="flex items-center justify-between px-4 py-2.5 border-b shrink-0">
            <div className="flex items-center gap-2">
              <Star className="w-4 h-4 text-yellow-400" />
              <h2 className="text-sm font-semibold">关注的组合</h2>
            </div>
          </div>
          <div className="flex-1 overflow-y-auto">
            {sortedPortfolios.filter(p => following.has(p.id)).map(p => (
              <div
                key={p.id}
                className="p-4 border-b border-border/50 hover:bg-muted/30 cursor-pointer"
                onClick={() => viewPortfolioDetail(p)}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <div className="font-medium">{p.name}</div>
                    <div className="text-[10px] text-muted-foreground">{p.owner}</div>
                  </div>
                  <div className="text-right">
                    <div className={`font-mono font-bold ${getReturnColor(p.total_return)}`}>
                      {formatPercent(p.total_return)}
                    </div>
                    <div className="text-[10px] text-muted-foreground">
                      胜率 {p.win_rate.toFixed(1)}%
                    </div>
                  </div>
                </div>
              </div>
            ))}
            {following.size === 0 && (
              <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                <Star className="w-12 h-12 opacity-30 mb-3" />
                <p className="text-sm">暂无关注的组合</p>
                <p className="text-xs mt-1">去排行榜关注喜欢的组合</p>
              </div>
            )}
          </div>
        </div>
      )}
      {view === 'detail' && renderDetailView()}
      {view === 'compare' && renderCompareView()}
    </div>
  );
}

// ==================== 辅助函数 ====================

function generateMockHoldings(portfolioId: string): PortfolioHolding[] {
  const holdings: Record<string, PortfolioHolding[]> = {
    'default': [
      { symbol: '600519.SH', name: '贵州茅台', price: 1850, change_pct: 2.35, cost: 1680, profit_ratio: 10.12, shares: 100, hold_days: 45, sector: '消费', lifecycle_stage: '持有' },
      { symbol: '000858.SZ', name: '五粮液', price: 178.5, change_pct: 3.21, cost: 145.8, profit_ratio: 22.43, shares: 800, hold_days: 32, sector: '消费', lifecycle_stage: '拉升' },
      { symbol: '601318.SH', name: '中国平安', price: 42.5, change_pct: -1.25, cost: 48.2, profit_ratio: -11.83, shares: 5000, hold_days: 60, sector: '金融', lifecycle_stage: '减仓' },
      { symbol: '300750.SZ', name: '宁德时代', price: 198.6, change_pct: 4.52, cost: 175.3, profit_ratio: 13.29, shares: 200, hold_days: 15, sector: '新能源', lifecycle_stage: '建仓' },
      { symbol: '688981.SH', name: '中芯国际', price: 52.3, change_pct: -2.15, cost: 48.9, profit_ratio: 6.95, shares: 2000, hold_days: 28, sector: '半导体', lifecycle_stage: '持有' },
    ],
  };
  return holdings[portfolioId] || holdings['default'];
}

function generateMockTransfers(portfolioId: string): TransferRecord[] {
  return [
    { symbol: '600519.SH', name: '贵州茅台', action: '买', position_before: 0, position_after: 100, price: 1680.5, time: '2026-03-20 10:30', amount: 168050, shares: 100 },
    { symbol: '000858.SZ', name: '五粮液', action: '买', position_before: 0, position_after: 800, price: 145.8, time: '2026-03-18 14:25', amount: 116640, shares: 800 },
    { symbol: '601318.SH', name: '中国平安', action: '卖', position_before: 5000, position_after: 0, price: 42.5, time: '2026-03-15 09:45', amount: 212500, shares: 5000 },
    { symbol: '300750.SZ', name: '宁德时代', action: '买', position_before: 0, position_after: 200, price: 175.3, time: '2026-03-12 11:20', amount: 35060, shares: 200 },
    { symbol: '002475.SZ', name: '立讯精密', action: '卖', position_before: 1500, position_after: 0, price: 32.15, time: '2026-03-10 13:40', amount: 48225, shares: 1500 },
  ];
}

function generateMockActivities(): ActivityItem[] {
  return [
    { type: 'buy', symbol: '600519', name: '贵州茅台', time: '2026-03-20 10:30', price: 1680.5, quantity: 100, amount: 168050 },
    { type: 'sell', symbol: '601318', name: '中国平安', time: '2026-03-15 09:45', price: 42.5, quantity: 5000, amount: 212500 },
    { type: 'buy', symbol: '300750', name: '宁德时代', time: '2026-03-12 11:20', price: 175.3, quantity: 200, amount: 35060 },
  ];
}
