/**
 * MarketCommandCenterPanel - 全球市场指挥中心
 * 统一展示：全球指数、商品、外汇、数字货币、新闻快讯、市场情绪
 * 设计：沉浸式驾驶舱，感知市场全局
 */
import { useState, useEffect, useRef } from 'react';
import {
  Globe,
  TrendingUp,
  TrendingDown,
  Droplet,
  Coins,
  Newspaper,
  Brain,
  RefreshCw,
  Activity,
  ChevronRight,
  ChevronDown,
  X,
  Clock,
  AlertCircle,
  ArrowUpRight,
  ArrowDownRight,
} from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Sheet, SheetContent, SheetHeader, SheetTitle } from '@/components/ui/sheet';
import { API_BASE, useWebSocket } from '@/hooks/useMarketData';
import { ExternalLink, List } from 'lucide-react';

interface MarketIndex {
  name: string;
  symbol: string;
  price: number;
  change: number;
  change_pct: number;
  region: string;
}

interface Commodity {
  name: string;
  price: number;
  change: number;
  change_pct: number;
  unit: string;
}

interface ForexPair {
  pair: string;
  price: number;
  change_pct: number;
}

interface Crypto {
  name: string;
  symbol: string;
  price: number;
  change_24h: number;
  market_cap: string;
}

interface NewsItem {
  id: string;
  title: string;
  summary: string;
  source: string;
  timestamp: string;
  url?: string;
  labels?: string[];
  is_hot?: boolean;
}

interface SentimentGauge {
  name: string;
  value: number; // 0-100
  level: 'fear' | 'neutral' | 'greed';
  description: string;
}

interface VolumeRatioPoint {
  time: string;
  today_volume: number;
  yesterday_volume: number;
  volume_ratio: number;
  cumulative_volume: number;
  cumulative_yesterday: number;
  cumulative_ratio: number;
}

interface VolumeRatioData {
  symbol: string;
  name: string;
  today_date: string;
  yesterday_date: string;
  points: VolumeRatioPoint[];
  avg_volume_ratio: number;
  max_volume_ratio: number;
  max_volume_time: string;
}

type SectionKey = 'indices' | 'commodities' | 'forex' | 'crypto' | 'news' | 'sentiment' | 'volume' | 'breadth';

export default function MarketCommandCenterPanel() {
  const [indices, setIndices] = useState<MarketIndex[]>([]);
  const [commodities, setCommodities] = useState<Commodity[]>([]);
  const [forexPairs, setForexPairs] = useState<ForexPair[]>([]);
  const [cryptos, setCryptos] = useState<Crypto[]>([]);
  const [news, setNews] = useState<NewsItem[]>([]);
  const [selectedNews, setSelectedNews] = useState<NewsItem | null>(null);
  const [newsDetail, setNewsDetail] = useState<{ content: string; url?: string } | null>(null);
  const [expandedNewsId, setExpandedNewsId] = useState<string | null>(null);
  const [newsSheetOpen, setNewsSheetOpen] = useState(false);
  const [sentiments, setSentiments] = useState<SentimentGauge[]>([]);
  const [loading, setLoading] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<string>('');
  const [activeSections, setActiveSections] = useState<Set<SectionKey>>(
    new Set<SectionKey>(['indices', 'commodities', 'news', 'sentiment'])
  );
  // 量比分析状态
  const [selectedSymbol, setSelectedSymbol] = useState<string>('000001');
  const [volumeRatioData, setVolumeRatioData] = useState<VolumeRatioData | null>(null);
  // 多股量比对比
  const [compareSymbols, setCompareSymbols] = useState<string[]>([]);
  const [compareData, setCompareData] = useState<Record<string, VolumeRatioData>>({});
  const [compareColors, setCompareColors] = useState<string[]>(['#ef4444', '#f97316', '#eab308', '#22c55e', '#06b6d4', '#8b5cf6']);
  const [compareInput, setCompareInput] = useState<string>('');
  // 市场广度状态
  const [marketBreadth, setMarketBreadth] = useState<{ up: number; down: number; flat: number }>({ up: 0, down: 0, flat: 0 });
  const [marketStatus, setMarketStatus] = useState<{
    us: 'open' | 'closed' | 'pre' | 'after';
    hk: 'open' | 'closed' | 'pre' | 'after';
    cn: 'open' | 'closed' | 'pre' | 'after';
    eu: 'open' | 'closed';
    jp: 'open' | 'closed';
  }>({ us: 'closed', hk: 'closed', cn: 'closed', eu: 'closed', jp: 'closed' });

  const newsScrollRef = useRef<HTMLDivElement>(null);

  // 加载全球指数
  const loadGlobalIndices = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/global/indices`);
      const data = await res.json();
      if (data.success) {
        const allIndices: MarketIndex[] = [
          ...(data.data.us || []).map((i: any) => ({ ...i, region: '美国' })),
          ...(data.data.hk || []).map((i: any) => ({ ...i, region: '港股' })),
          ...(data.data.asia || []).map((i: any) => ({ ...i, region: '亚太' })),
          ...(data.data.eu || []).map((i: any) => ({ ...i, region: '欧洲' })),
        ];
        setIndices(allIndices);
      }
    } catch (e) {
      console.error('加载全球指数失败', e);
    }
  };

  // 加载商品数据
  const loadCommodities = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/global/commodities`);
      const data = await res.json();
      if (data.success) {
        setCommodities(data.data || []);
      }
    } catch (e) {
      console.error('加载商品数据失败', e);
    }
  };

  // 加载外汇数据
  const loadForex = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/global/forex`);
      const data = await res.json();
      if (data.success) {
        setForexPairs(data.data || []);
      }
    } catch (e) {
      console.error('加载外汇数据失败', e);
    }
  };

  // 加载加密货币
  const loadCrypto = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/global/crypto`);
      const data = await res.json();
      if (data.success) {
        setCryptos(data.data || []);
      }
    } catch (e) {
      console.error('加载加密货币失败', e);
    }
  };

  // 加载新闻快讯
  const loadNews = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/news/realtime?limit=20`);
      const data = await res.json();
      if (data.success && data.data?.list) {
        const newsList = data.data.list.map((item: any) => ({
          id: item.id,
          title: item.title,
          summary: item.content || item.title,
          source: item.source,
          timestamp: item.pub_time?.slice(11, 16) || '',
          url: item.url,
          is_hot: item.is_hot,
          labels: item.category ? [item.category] : [],
        }));
        setNews(newsList);
      }
    } catch (e) {
      console.error('加载新闻失败', e);
    }
  };

  // 加载快讯详情
  const loadNewsDetail = async (newsItem: NewsItem) => {
    try {
      const res = await fetch(`${API_BASE}/api/news/detail/${newsItem.id}`);
      const data = await res.json();
      if (data.success) {
        return {
          content: data.data?.content || data.data?.digest || newsItem.summary || '',
          url: data.data?.url || newsItem.url,
        };
      }
    } catch (e) {
      console.error('加载快讯详情失败', e);
    }
    return { content: newsItem.summary || '', url: newsItem.url };
  };

  // 计算市场情绪
  const loadSentiment = async () => {
    // 基于市场数据计算情绪
    const fearGreedIndices: SentimentGauge[] = [
      {
        name: 'A股情绪',
        value: calculateSentiment(indices.filter(i => i.region === '港股' || i.region === '美国')),
        level: 'neutral',
        description: '中性偏谨慎',
      },
      {
        name: '恐慌贪婪',
        value: calculateSentiment(indices),
        level: 'neutral',
        description: '市场情绪稳定',
      },
      {
        name: '资金流向',
        value: 45,
        level: 'fear',
        description: '资金净流出',
      },
      {
        name: '波动率',
        value: 30,
        level: 'fear',
        description: '低波动环境',
      },
      {
        name: '期权市场',
        value: 55,
        level: 'greed',
        description: '隐含波动率偏低',
      },
      {
        name: '情绪趋势',
        value: 60,
        level: 'greed',
        description: '近5日情绪上升',
      },
    ];
    setSentiments(fearGreedIndices);
  };

  // 计算情绪值 (基于涨跌)
  const calculateSentiment = (marketIndices: MarketIndex[]): number => {
    if (marketIndices.length === 0) return 50;
    const avgChange = marketIndices.reduce((sum, i) => sum + i.change_pct, 0) / marketIndices.length;
    // 将涨跌幅转换为 0-100 的情绪值
    // 涨 3% -> 100, 涨 0% -> 50, 跌 3% -> 0
    return Math.min(100, Math.max(0, 50 + avgChange * 16.67));
  };

  // 加载量比数据
  const loadVolumeRatio = async (symbol: string) => {
    try {
      const res = await fetch(`${API_BASE}/api/volume-ratio/${symbol}`);
      const data = await res.json();
      if (data.success && data.data.points && data.data.points.length > 0) {
        setVolumeRatioData(data.data);
      } else {
        setVolumeRatioData(null);
      }
    } catch (e) {
      console.error('加载量比数据失败', e);
      setVolumeRatioData(null);
    }
  };

  // 添加对比股票
  const addCompareSymbol = () => {
    const symbols = compareInput.split(',').map(s => s.trim()).filter(s => s && !compareSymbols.includes(s));
    if (symbols.length > 0) {
      setCompareSymbols([...compareSymbols, ...symbols].slice(0, 6));
      setCompareInput('');
      // 加载对比数据
      symbols.forEach((sym, idx) => {
        loadCompareVolumeRatio(sym, (compareSymbols.length + idx) % compareColors.length);
      });
    }
  };

  // 移除对比股票
  const removeCompareSymbol = (symbol: string) => {
    setCompareSymbols(compareSymbols.filter(s => s !== symbol));
    setCompareData(prev => {
      const next = { ...prev };
      delete next[symbol];
      return next;
    });
  };

  // 加载对比股票的量比数据
  const loadCompareVolumeRatio = async (symbol: string, colorIndex: number) => {
    try {
      const res = await fetch(`${API_BASE}/api/volume-ratio/${symbol}`);
      const data = await res.json();
      if (data.success && data.data.points && data.data.points.length > 0) {
        setCompareData(prev => ({
          ...prev,
          [symbol]: { ...data.data, _colorIndex: colorIndex }
        }));
      }
    } catch (e) {
      console.error('加载对比量比数据失败', e);
    }
  };

  // 批量加载所有对比股票的量比数据
  const loadAllCompareData = () => {
    compareSymbols.forEach((sym, idx) => {
      loadCompareVolumeRatio(sym, idx % compareColors.length);
    });
  };

  // 从市场概览获取市场广度
  const loadMarketBreadth = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/market/overview`);
      const data = await res.json();
      if (data.success && data.data) {
        setMarketBreadth({
          up: data.data.up_count || 0,
          down: data.data.down_count || 0,
          flat: data.data.flat_count || 0,
        });
      }
    } catch (e) {
      // 使用默认值
    }
  };

  // 判断市场状态
  const checkMarketStatus = () => {
    const now = new Date();
    const utcHour = now.getUTCHours();
    const cnHour = utcHour + 8;

    // 美国时间 (UTC-5/-4)
    const usHour = (utcHour - 5 + 24) % 24;
    const usOpen = usHour >= 9.5 && usHour < 16;
    setMarketStatus(prev => ({ ...prev, us: usOpen ? 'open' : 'closed' }));

    // 中国时间
    const cnOpen = cnHour >= 9.5 && cnHour < 15;
    setMarketStatus(prev => ({ ...prev, cn: cnOpen ? 'open' : 'closed' }));

    // 港股时间
    const hkOpen = cnHour >= 9.5 && cnHour < 16;
    setMarketStatus(prev => ({ ...prev, hk: hkOpen ? 'open' : 'closed' }));

    // 日本时间
    const jpHour = (utcHour + 9) % 24;
    const jpOpen = jpHour >= 9 && jpHour < 15;
    setMarketStatus(prev => ({ ...prev, jp: jpOpen ? 'open' : 'closed' }));

    // 欧洲时间
    const euHour = (utcHour + 1) % 24;
    const euOpen = euHour >= 9 && euHour < 17.5;
    setMarketStatus(prev => ({ ...prev, eu: euOpen ? 'open' : 'closed' }));
  };

  // 刷新所有数据
  const refreshAll = async () => {
    setLoading(true);
    try {
      await Promise.all([
        loadGlobalIndices(),
        loadCommodities(),
        loadForex(),
        loadCrypto(),
        loadNews(),
        loadSentiment(),
        loadMarketBreadth(),
      ]);
      loadAllCompareData();
      if (selectedSymbol) {
        loadVolumeRatio(selectedSymbol);
      }
      setLastUpdate(new Date().toLocaleTimeString());
    } finally {
      setLoading(false);
    }
  };

  // 初始化和定时刷新
  useEffect(() => {
    refreshAll();
    checkMarketStatus();

    const interval = setInterval(() => {
      refreshAll();
      checkMarketStatus();
    }, 30000); // 30秒刷新

    return () => clearInterval(interval);
  }, []);

  // 自动滚动新闻（弹窗打开时暂停）
  useEffect(() => {
    if (news.length === 0 || !newsScrollRef.current || selectedNews) return;
    const scrollEl = newsScrollRef.current;
    let scrollPos = 0;
    const scrollSpeed = 0.5;
    let animationId: number;

    const autoScroll = () => {
      scrollPos += scrollSpeed;
      if (scrollPos >= scrollEl.scrollHeight - scrollEl.clientHeight) {
        scrollPos = 0;
      }
      scrollEl.scrollTop = scrollPos;
      animationId = requestAnimationFrame(autoScroll);
    };

    const timer = setTimeout(() => {
      animationId = requestAnimationFrame(autoScroll);
    }, 3000);

    return () => {
      clearTimeout(timer);
      cancelAnimationFrame(animationId);
    };
  }, [news, selectedNews]);

  // 切换区块显示
  const toggleSection = (section: SectionKey) => {
    setActiveSections(prev => {
      const next = new Set(prev);
      if (next.has(section)) {
        next.delete(section);
      } else {
        next.add(section);
      }
      return next;
    });
  };

  // 格式化涨跌
  const formatChange = (change: number, pct: number) => {
    const sign = change >= 0 ? '+' : '';
    return `${sign}${change.toFixed(2)} (${sign}${pct.toFixed(2)}%)`;
  };

  // 渲染涨跌图标
  const renderTrendIcon = (change: number) => {
    if (change >= 0) {
      return <TrendingUp className="w-4 h-4 text-red-500" />;
    }
    return <TrendingDown className="w-4 h-4 text-green-500" />;
  };

  // 渲染情绪仪表
  const renderSentimentGauge = (sentiment: SentimentGauge) => {
    const color = sentiment.level === 'fear' ? 'text-blue-500' : sentiment.level === 'greed' ? 'text-red-500' : 'text-yellow-500';
    const bgColor = sentiment.level === 'fear' ? 'bg-blue-500' : sentiment.level === 'greed' ? 'bg-red-500' : 'bg-yellow-500';

    return (
      <div key={sentiment.name} className="flex flex-col items-center p-3 bg-card rounded-lg border border-border">
        <div className="text-xs text-muted-foreground mb-2">{sentiment.name}</div>
        <div className={`text-2xl font-bold ${color}`}>{sentiment.value}</div>
        <div className="w-full h-2 bg-muted rounded-full mt-2 overflow-hidden">
          <div
            className={`h-full ${bgColor} transition-all duration-500`}
            style={{ width: `${sentiment.value}%` }}
          />
        </div>
        <div className="text-[10px] text-muted-foreground mt-1">{sentiment.description}</div>
      </div>
    );
  };

  return (
    <div className="flex flex-col h-full bg-background">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-border bg-card/50">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            <Globe className="w-5 h-5 text-primary" />
            <h2 className="text-base font-semibold">全球市场指挥中心</h2>
          </div>
          <div className="flex items-center gap-2 text-xs">
            {marketStatus.us === 'open' && (
              <span className="px-2 py-0.5 bg-green-500/20 text-green-500 rounded">美股开市</span>
            )}
            {marketStatus.cn === 'open' && (
              <span className="px-2 py-0.5 bg-red-500/20 text-red-500 rounded">A股开市</span>
            )}
            {marketStatus.hk === 'open' && (
              <span className="px-2 py-0.5 bg-orange-500/20 text-orange-500 rounded">港股开市</span>
            )}
            {marketStatus.jp === 'open' && (
              <span className="px-2 py-0.5 bg-purple-500/20 text-purple-500 rounded">日股开市</span>
            )}
            {marketStatus.eu === 'open' && (
              <span className="px-2 py-0.5 bg-blue-500/20 text-blue-500 rounded">欧股开市</span>
            )}
          </div>
        </div>
        <div className="flex items-center gap-3">
          {lastUpdate && (
            <span className="text-[10px] text-muted-foreground">
              <Clock className="w-3 h-3 inline mr-1" />
              {lastUpdate}
            </span>
          )}
          <button
            onClick={refreshAll}
            disabled={loading}
            className="p-1.5 hover:bg-muted rounded transition-colors"
          >
            <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {/* Section Toggles */}
      <div className="flex items-center gap-2 px-4 py-2 border-b border-border bg-muted/30 overflow-x-auto">
        {[
          { key: 'indices', label: '全球指数', icon: Globe },
          { key: 'commodities', label: '商品', icon: Droplet },
          { key: 'forex', label: '外汇', icon: Coins },
          { key: 'crypto', label: '加密货币', icon: Coins },
          { key: 'news', label: '快讯', icon: Newspaper },
          { key: 'sentiment', label: '情绪', icon: Brain },
          { key: 'volume', label: '量比', icon: Activity },
          { key: 'breadth', label: '广度', icon: Activity },
        ].map(({ key, label, icon: Icon }) => (
          <button
            key={key}
            onClick={() => toggleSection(key as SectionKey)}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-medium transition-all ${
              activeSections.has(key as SectionKey)
                ? 'bg-primary text-primary-foreground'
                : 'bg-card border border-border hover:bg-muted'
            }`}
          >
            <Icon className="w-3 h-3" />
            {label}
          </button>
        ))}
      </div>

      {/* Main Content */}
      <ScrollArea className="flex-1">
        <div className="p-4 space-y-4">
          {/* 全球指数 */}
          {activeSections.has('indices') && (
            <section className="space-y-2">
              <h3 className="text-sm font-medium flex items-center gap-2">
                <Globe className="w-4 h-4 text-primary" />
                全球指数
              </h3>
              <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
                {indices.slice(0, 12).map((index) => (
                  <div
                    key={index.symbol}
                    className="p-3 bg-card border border-border rounded-lg hover:border-primary/50 transition-colors"
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex-1 min-w-0">
                        <div className="text-xs text-muted-foreground truncate">{index.region}</div>
                        <div className="font-medium text-sm truncate">{index.name}</div>
                      </div>
                      {renderTrendIcon(index.change)}
                    </div>
                    <div className="mt-2">
                      <div className="text-sm font-semibold">{index.price.toLocaleString()}</div>
                      <div className={`text-xs ${index.change >= 0 ? 'text-red-500' : 'text-green-500'}`}>
                        {formatChange(index.change, index.change_pct)}
                      </div>
                    </div>
                  </div>
                ))}
                {indices.length === 0 && (
                  <div className="col-span-full text-center py-6 text-muted-foreground text-sm">
                    暂无指数数据
                  </div>
                )}
              </div>
            </section>
          )}

          {/* 商品 */}
          {activeSections.has('commodities') && (
            <section className="space-y-2">
              <h3 className="text-sm font-medium flex items-center gap-2">
                <Droplet className="w-4 h-4 text-orange-500" />
                大宗商品
              </h3>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
                {commodities.map((commodity) => (
                  <div
                    key={commodity.name}
                    className="p-3 bg-card border border-border rounded-lg"
                  >
                    <div className="text-xs text-muted-foreground">{commodity.name}</div>
                    <div className="text-sm font-semibold mt-1">${commodity.price.toLocaleString()}</div>
                    <div className={`text-xs ${commodity.change >= 0 ? 'text-red-500' : 'text-green-500'}`}>
                      {commodity.change >= 0 ? '+' : ''}{commodity.change_pct.toFixed(2)}%
                    </div>
                  </div>
                ))}
                {commodities.length === 0 && (
                  <div className="col-span-full text-center py-6 text-muted-foreground text-sm">
                    暂无商品数据
                  </div>
                )}
              </div>
            </section>
          )}

          {/* 外汇 */}
          {activeSections.has('forex') && (
            <section className="space-y-2">
              <h3 className="text-sm font-medium flex items-center gap-2">
                <Coins className="w-4 h-4 text-blue-500" />
                外汇
              </h3>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
                {forexPairs.length > 0 ? forexPairs.map((pair) => (
                  <div key={pair.pair} className="p-3 bg-card border border-border rounded-lg">
                    <div className="text-sm font-medium">{pair.pair}</div>
                    <div className="text-sm font-semibold mt-1">{pair.price.toFixed(4)}</div>
                    <div className={`text-xs ${pair.change_pct >= 0 ? 'text-red-500' : 'text-green-500'}`}>
                      {pair.change_pct >= 0 ? '+' : ''}{pair.change_pct.toFixed(2)}%
                    </div>
                  </div>
                )) : (<div className="col-span-full text-center py-6 text-muted-foreground text-sm">暂无加密货币数据</div>)}
              </div>
            </section>
          )}

          {/* 加密货币 */}
          {activeSections.has('crypto') && (
            <section className="space-y-2">
              <h3 className="text-sm font-medium flex items-center gap-2">
                <Coins className="w-4 h-4 text-yellow-500" />
                加密货币
              </h3>
              <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
                {cryptos.length > 0 ? cryptos.map((crypto) => (
                  <div key={crypto.symbol} className="p-3 bg-card border border-border rounded-lg">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium">{crypto.name}</span>
                      <span className="text-xs text-muted-foreground">{crypto.symbol}</span>
                    </div>
                    <div className="text-sm font-semibold mt-1">${crypto.price.toLocaleString()}</div>
                    <div className={`text-xs ${crypto.change_24h >= 0 ? 'text-red-500' : 'text-green-500'}`}>
                      24h {crypto.change_24h >= 0 ? '+' : ''}{crypto.change_24h.toFixed(2)}%
                    </div>
                  </div>
                )) : (
                  <div className="col-span-full text-center py-6 text-muted-foreground text-sm">
                    暂无加密货币数据
                  </div>
                )}
              </div>
            </section>
          )}

          {/* 新闻快讯 */}
          {activeSections.has('news') && (
            <section className="space-y-2">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-medium flex items-center gap-2">
                  <Newspaper className="w-4 h-4 text-cyan-500" />
                  最新快讯
                </h3>
                <button
                  onClick={() => setNewsSheetOpen(true)}
                  className="flex items-center gap-1 px-2 py-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
                >
                  <List className="w-3 h-3" />
                  全部
                </button>
              </div>
              <div
                ref={newsScrollRef}
                className="h-48 overflow-y-auto space-y-2 pr-2"
              >
                {news.length > 0 ? news.map((item) => (
                  <div
                    key={item.id}
                    className={`bg-card border rounded-lg transition-colors ${
                      item.is_hot ? 'border-orange-500/50 bg-orange-500/5' : 'border-border'
                    } ${expandedNewsId === item.id ? 'ring-1 ring-primary/30' : 'hover:border-primary/50'}`}
                  >
                    {/* 卡片主体 - 点击打开弹窗 */}
                    <div
                      className="p-3 cursor-pointer"
                      onClick={async () => {
                        setSelectedNews(item);
                        const detail = await loadNewsDetail(item);
                        setNewsDetail(detail);
                      }}
                    >
                      <div className="flex items-start justify-between gap-2">
                        <div className="flex-1">
                          <div className="flex items-center gap-2">
                            <span className="text-xs text-muted-foreground">{item.source}</span>
                            <span className="text-[10px] text-muted-foreground">{item.timestamp}</span>
                            {item.is_hot && (
                              <span className="px-1.5 py-0.5 text-[10px] bg-orange-500/20 text-orange-500 rounded">
                                热点
                              </span>
                            )}
                          </div>
                          <div className="text-sm mt-1 leading-snug">{item.title}</div>
                        </div>
                        <button
                          className="p-1 hover:bg-muted rounded transition-colors shrink-0"
                          onClick={(e) => {
                            e.stopPropagation();
                            setExpandedNewsId(expandedNewsId === item.id ? null : item.id);
                          }}
                        >
                          {expandedNewsId === item.id ? (
                            <ChevronDown className="w-4 h-4 text-muted-foreground" />
                          ) : (
                            <ChevronRight className="w-4 h-4 text-muted-foreground" />
                          )}
                        </button>
                      </div>
                    </div>
                    {/* 展开的内容 */}
                    {expandedNewsId === item.id && (
                      <div className="px-3 pb-3 border-t border-border/50">
                        <p className="text-sm leading-relaxed text-muted-foreground mt-2">
                          {item.summary || '暂无详细内容'}
                        </p>
                        {item.url && (
                          <a
                            href={item.url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="inline-flex items-center gap-1 mt-2 text-xs text-primary hover:underline"
                            onClick={(e) => e.stopPropagation()}
                          >
                            查看原文
                            <ExternalLink className="w-3 h-3" />
                          </a>
                        )}
                      </div>
                    )}
                  </div>
                )) : (
                  <div className="text-center py-8 text-muted-foreground text-sm">
                    暂无新闻数据
                  </div>
                )}
              </div>
            </section>
          )}

          {/* 快讯详情弹窗 */}
          {selectedNews && (
            <Dialog open={!!selectedNews} onOpenChange={() => setSelectedNews(null)}>
              <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
                <DialogHeader>
                  <DialogTitle className="text-left pr-8">{selectedNews.title}</DialogTitle>
                  <div className="flex items-center gap-3 text-sm text-muted-foreground">
                    <span>{selectedNews.source}</span>
                    <span>{selectedNews.timestamp}</span>
                    {selectedNews.is_hot && (
                      <span className="px-1.5 py-0.5 text-[10px] bg-orange-500/20 text-orange-500 rounded">
                        热点
                      </span>
                    )}
                  </div>
                </DialogHeader>
                <div className="space-y-4">
                  {newsDetail?.content ? (
                    <p className="text-sm leading-relaxed whitespace-pre-wrap">{newsDetail.content}</p>
                  ) : (
                    <p className="text-sm text-muted-foreground">暂无详细内容</p>
                  )}
                  {newsDetail?.url && (
                    <a
                      href={newsDetail.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="inline-flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground text-sm rounded hover:bg-primary/80"
                    >
                      查看原文
                      <ExternalLink className="w-4 h-4" />
                    </a>
                  )}
                </div>
              </DialogContent>
            </Dialog>
          )}

          {/* 快讯全部列表抽屉 */}
          <Sheet open={newsSheetOpen} onOpenChange={setNewsSheetOpen}>
            <SheetContent side="right" className="w-full sm:max-w-xl flex flex-col">
              <SheetHeader>
                <SheetTitle className="flex items-center gap-2">
                  <Newspaper className="w-4 h-4 text-cyan-500" />
                  全部快讯 ({news.length})
                </SheetTitle>
              </SheetHeader>
              <ScrollArea className="flex-1 mt-4">
                <div className="space-y-3 pr-4">
                  {news.map((item) => (
                    <div
                      key={item.id}
                      className={`p-4 bg-card border rounded-lg transition-colors hover:border-primary/50 ${
                        item.is_hot ? 'border-orange-500/50 bg-orange-500/5' : 'border-border'
                      }`}
                    >
                      <div className="flex items-start justify-between gap-2">
                        <div className="flex-1">
                          <div className="flex items-center gap-2 flex-wrap">
                            <span className="text-xs text-muted-foreground">{item.source}</span>
                            <span className="text-[10px] text-muted-foreground">{item.timestamp}</span>
                            {item.is_hot && (
                              <span className="px-1.5 py-0.5 text-[10px] bg-orange-500/20 text-orange-500 rounded">
                                热点
                              </span>
                            )}
                          </div>
                          <div className="text-sm mt-1 leading-snug font-medium">{item.title}</div>
                        </div>
                      </div>
                      <div className="mt-2 text-xs text-muted-foreground line-clamp-2">
                        {item.summary || '暂无详细内容'}
                      </div>
                      <div className="mt-2 flex items-center gap-2">
                        <button
                          onClick={async () => {
                            setSelectedNews(item);
                            const detail = await loadNewsDetail(item);
                            setNewsDetail(detail);
                          }}
                          className="px-3 py-1 text-xs bg-primary text-primary-foreground rounded hover:bg-primary/90"
                        >
                          查看详情
                        </button>
                        {item.url && (
                          <a
                            href={item.url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="px-3 py-1 text-xs border border-border rounded hover:bg-muted transition-colors"
                          >
                            原文链接
                          </a>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </ScrollArea>
            </SheetContent>
          </Sheet>

          {/* 市场情绪 */}
          {activeSections.has('sentiment') && (
            <section className="space-y-2">
              <h3 className="text-sm font-medium flex items-center gap-2">
                <Brain className="w-4 h-4 text-purple-500" />
                市场情绪
              </h3>
              <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-2">
                {sentiments.map((s) => renderSentimentGauge(s))}
              </div>

              {/* 全局情绪仪表 */}
              <div className="mt-4 p-4 bg-gradient-to-r from-blue-900/30 via-purple-900/30 to-orange-900/30 border border-border rounded-lg">
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center gap-2">
                    <Activity className="w-5 h-5 text-purple-500" />
                    <span className="font-medium">全球市场综合情绪</span>
                  </div>
                  <div className="flex items-center gap-2">
                    {indices.filter(i => i.change_pct > 0).length > indices.length / 2 ? (
                      <span className="flex items-center gap-1 text-green-500 text-sm">
                        <ArrowUpRight className="w-4 h-4" />
                        多头占优
                      </span>
                    ) : indices.filter(i => i.change_pct < 0).length > indices.length / 2 ? (
                      <span className="flex items-center gap-1 text-red-500 text-sm">
                        <ArrowDownRight className="w-4 h-4" />
                        空头占优
                      </span>
                    ) : (
                      <span className="flex items-center gap-1 text-yellow-500 text-sm">
                        <AlertCircle className="w-4 h-4" />
                        平衡
                      </span>
                    )}
                  </div>
                </div>

                {/* 大盘涨跌家数 */}
                <div className="grid grid-cols-2 gap-4">
                  <div className="flex items-center gap-3">
                    <div className="flex flex-col items-center">
                      <span className="text-2xl font-bold text-red-500">
                        {indices.filter(i => i.change_pct > 0).length}
                      </span>
                      <span className="text-xs text-muted-foreground">上涨</span>
                    </div>
                    <div className="flex-1 h-2 bg-muted rounded-full overflow-hidden">
                      <div
                        className="h-full bg-red-500"
                        style={{
                          width: indices.length > 0
                            ? `${(indices.filter(i => i.change_pct > 0).length / indices.length) * 100}%`
                            : '50%'
                        }}
                      />
                    </div>
                    <div className="flex flex-col items-center">
                      <span className="text-2xl font-bold text-green-500">
                        {indices.filter(i => i.change_pct < 0).length}
                      </span>
                      <span className="text-xs text-muted-foreground">下跌</span>
                    </div>
                  </div>
                  <div className="text-center">
                    <div className={`text-3xl font-bold ${
                      calculateSentiment(indices) > 55 ? 'text-red-500' :
                      calculateSentiment(indices) < 45 ? 'text-green-500' : 'text-yellow-500'
                    }`}>
                      {calculateSentiment(indices) > 55 ? '偏热' :
                       calculateSentiment(indices) < 45 ? '偏冷' : '中性'}
                    </div>
                    <div className="text-xs text-muted-foreground">
                      建议：{calculateSentiment(indices) > 70 ? '注意风险' :
                            calculateSentiment(indices) < 30 ? '关注机会' : '观望为主'}
                    </div>
                  </div>
                </div>
              </div>
            </section>
          )}

          {/* 量比分析 */}
          {activeSections.has('volume') && (
            <section className="space-y-2">
              <h3 className="text-sm font-medium flex items-center gap-2">
                <Activity className="w-4 h-4 text-cyan-500" />
                今日量比分析
              </h3>
              <div className="flex items-center gap-2 mb-2">
                <input
                  type="text"
                  value={selectedSymbol}
                  onChange={(e) => setSelectedSymbol(e.target.value)}
                  placeholder="输入股票代码"
                  className="px-3 py-1.5 bg-card border border-border rounded text-sm w-28"
                />
                <button
                  onClick={() => loadVolumeRatio(selectedSymbol)}
                  className="px-3 py-1.5 bg-primary text-primary-foreground rounded text-sm hover:bg-primary/90"
                >
                  查询
                </button>
                <div className="h-4 w-px bg-border mx-1" />
                <input
                  type="text"
                  value={compareInput}
                  onChange={(e) => setCompareInput(e.target.value)}
                  placeholder="对比: 000001,000002"
                  className="px-3 py-1.5 bg-card border border-border rounded text-sm w-36"
                  onKeyDown={(e) => e.key === 'Enter' && addCompareSymbol()}
                />
                <button
                  onClick={addCompareSymbol}
                  className="px-3 py-1.5 bg-orange-500 text-white rounded text-sm hover:bg-orange-600"
                >
                  +对比
                </button>
              </div>
              {/* 对比股票标签 */}
              {compareSymbols.length > 0 && (
                <div className="flex flex-wrap gap-1 mb-2">
                  {compareSymbols.map((sym, idx) => (
                    <span
                      key={sym}
                      className="flex items-center gap-1 px-2 py-0.5 rounded text-xs text-white"
                      style={{ backgroundColor: compareColors[idx % compareColors.length] }}
                    >
                      {sym}
                      <button
                        onClick={() => removeCompareSymbol(sym)}
                        className="ml-1 hover:opacity-70"
                      >
                        ×
                      </button>
                    </span>
                  ))}
                </div>
              )}
              {volumeRatioData ? (
                <div className="space-y-2">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">
                      {volumeRatioData.name} ({volumeRatioData.symbol})
                    </span>
                    <span className="text-muted-foreground">
                      今日: {volumeRatioData.today_date} | 昨日: {volumeRatioData.yesterday_date}
                    </span>
                  </div>
                  <div className="grid grid-cols-3 gap-2">
                    <div className="p-3 bg-card border border-border rounded-lg text-center">
                      <div className="text-xs text-muted-foreground">平均量比</div>
                      <div className={`text-lg font-bold ${volumeRatioData.avg_volume_ratio > 1 ? 'text-red-500' : 'text-green-500'}`}>
                        {volumeRatioData.avg_volume_ratio.toFixed(2)}
                      </div>
                    </div>
                    <div className="p-3 bg-card border border-border rounded-lg text-center">
                      <div className="text-xs text-muted-foreground">最大量比</div>
                      <div className="text-lg font-bold text-red-500">
                        {volumeRatioData.max_volume_ratio.toFixed(2)}
                      </div>
                    </div>
                    <div className="p-3 bg-card border border-border rounded-lg text-center">
                      <div className="text-xs text-muted-foreground">最大量比时间</div>
                      <div className="text-lg font-bold text-orange-500">
                        {volumeRatioData.max_volume_time}
                      </div>
                    </div>
                  </div>
                  {/* 多股对比柱状图 */}
                  <div className="p-3 bg-card border border-border rounded-lg">
                    <div className="text-xs text-muted-foreground mb-2">分钟级量比对比</div>
                    <div className="flex items-end gap-0.5 h-28">
                      {volumeRatioData.points.slice(0, 60).map((point, i) => {
                        // 找出所有股票在这个时间点的量比
                        const maxRatio = Math.max(
                          point.volume_ratio,
                          ...compareSymbols.map(sym => compareData[sym]?.points[i]?.volume_ratio || 0)
                        );
                        return (
                          <div key={i} className="flex-1 flex items-end gap-px">
                            {/* 主股票 */}
                            <div
                              className="flex-1 rounded-t bg-cyan-500"
                              style={{ height: `${Math.min(100, point.volume_ratio * 50)}%` }}
                              title={`${volumeRatioData.symbol}: ${point.volume_ratio.toFixed(2)}`}
                            />
                            {/* 对比股票 */}
                            {compareSymbols.map((sym, cidx) => {
                              const cmpPoint = compareData[sym]?.points[i];
                              return cmpPoint ? (
                                <div
                                  key={sym}
                                  className="flex-1 rounded-t"
                                  style={{
                                    height: `${Math.min(100, (cmpPoint.volume_ratio / maxRatio) * 100)}%`,
                                    backgroundColor: compareColors[cidx % compareColors.length],
                                    opacity: 0.7
                                  }}
                                  title={`${sym}: ${cmpPoint.volume_ratio.toFixed(2)}`}
                                />
                              ) : <div key={sym} className="flex-1" />;
                            })}
                          </div>
                        );
                      })}
                    </div>
                    <div className="flex justify-between text-[10px] text-muted-foreground mt-1">
                      <span>09:30</span>
                      <span>10:30</span>
                      <span>11:30</span>
                    </div>
                    {/* 图例 */}
                    <div className="flex items-center gap-3 mt-2 text-[10px]">
                      <span className="flex items-center gap-1">
                        <span className="w-2 h-2 rounded bg-cyan-500" /> {volumeRatioData.symbol}
                      </span>
                      {compareSymbols.map((sym, idx) => (
                        <span key={sym} className="flex items-center gap-1">
                          <span
                            className="w-2 h-2 rounded"
                            style={{ backgroundColor: compareColors[idx % compareColors.length] }}
                          /> {sym}
                        </span>
                      ))}
                    </div>
                  </div>
                  {/* 对比股票统计 */}
                  {compareSymbols.length > 0 && Object.keys(compareData).length > 0 && (
                    <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
                      {compareSymbols.map((sym, idx) => {
                        const data = compareData[sym];
                        if (!data) return null;
                        return (
                          <div
                            key={sym}
                            className="p-3 bg-card border rounded-lg"
                            style={{ borderColor: compareColors[idx % compareColors.length] + '60' }}
                          >
                            <div className="text-sm font-medium">{data.name || sym}</div>
                            <div className="flex gap-3 mt-1">
                              <div>
                                <div className="text-[10px] text-muted-foreground">均量比</div>
                                <div className="text-sm font-bold" style={{ color: compareColors[idx % compareColors.length] }}>
                                  {data.avg_volume_ratio.toFixed(2)}
                                </div>
                              </div>
                              <div>
                                <div className="text-[10px] text-muted-foreground">最大量比</div>
                                <div className="text-sm font-bold text-red-500">
                                  {data.max_volume_ratio.toFixed(2)}
                                </div>
                              </div>
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              ) : (
                <div className="text-center py-6 text-muted-foreground text-sm">
                  暂无量比数据
                </div>
              )}
            </section>
          )}

          {/* 市场广度 */}
          {activeSections.has('breadth') && (
            <section className="space-y-2">
              <h3 className="text-sm font-medium flex items-center gap-2">
                <Activity className="w-4 h-4 text-teal-500" />
                A股市场广度
              </h3>
              <div className="grid grid-cols-3 gap-2">
                <div className="p-4 bg-card border border-border rounded-lg text-center">
                  <div className="text-3xl font-bold text-red-500">{marketBreadth.up}</div>
                  <div className="text-xs text-muted-foreground mt-1">上涨家数</div>
                </div>
                <div className="p-4 bg-card border border-border rounded-lg text-center">
                  <div className="text-3xl font-bold text-green-500">{marketBreadth.down}</div>
                  <div className="text-xs text-muted-foreground mt-1">下跌家数</div>
                </div>
                <div className="p-4 bg-card border border-border rounded-lg text-center">
                  <div className="text-3xl font-bold text-gray-500">{marketBreadth.flat}</div>
                  <div className="text-xs text-muted-foreground mt-1">平盘家数</div>
                </div>
              </div>
              {/* 广度柱状图 */}
              <div className="p-3 bg-card border border-border rounded-lg">
                <div className="text-xs text-muted-foreground mb-2">上涨/下跌比例</div>
                <div className="flex h-6 rounded-full overflow-hidden">
                  <div
                    className="bg-red-500 flex items-center justify-center text-xs text-white"
                    style={{ width: marketBreadth.up + marketBreadth.down > 0 ? `${(marketBreadth.up / (marketBreadth.up + marketBreadth.down)) * 100}%` : '50%' }}
                  >
                    {marketBreadth.up > 0 && `${(marketBreadth.up / (marketBreadth.up + marketBreadth.down) * 100).toFixed(0)}%`}
                  </div>
                  <div
                    className="bg-green-500 flex items-center justify-center text-xs text-white"
                    style={{ width: marketBreadth.up + marketBreadth.down > 0 ? `${(marketBreadth.down / (marketBreadth.up + marketBreadth.down)) * 100}%` : '50%' }}
                  >
                    {marketBreadth.down > 0 && `${(marketBreadth.down / (marketBreadth.up + marketBreadth.down) * 100).toFixed(0)}%`}
                  </div>
                </div>
              </div>
              {/* 市场广度趋势 */}
              <div className="p-4 bg-gradient-to-r from-red-900/30 via-yellow-900/30 to-green-900/30 border border-border rounded-lg">
                <div className="text-center">
                  <div className={`text-2xl font-bold ${
                    marketBreadth.up > marketBreadth.down ? 'text-red-500' :
                    marketBreadth.down > marketBreadth.up ? 'text-green-500' : 'text-yellow-500'
                  }`}>
                    {marketBreadth.up > marketBreadth.down ? '多头市场' :
                     marketBreadth.down > marketBreadth.up ? '空头市场' : '平衡市场'}
                  </div>
                  <div className="text-xs text-muted-foreground mt-1">
                    上涨/下跌比: {(marketBreadth.up / Math.max(1, marketBreadth.down)).toFixed(2)}
                  </div>
                </div>
              </div>
            </section>
          )}
        </div>
      </ScrollArea>
    </div>
  );
}
