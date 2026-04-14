import { useState, useEffect, useContext, useRef } from 'react';
import { Newspaper, TrendingUp, AlertTriangle, Building2, RefreshCw, ChevronDown, Eye, Activity, X } from 'lucide-react';
import { StockClickContext } from '@/pages/Dashboard';

const API_BASE = '';

// 市场情绪判断
interface MarketSentiment {
  score: number;
  level: '极弱' | '偏弱' | '中性' | '偏强' | '极强';
  recommendation: '空仓观望' | '轻仓参与' | '适度参与' | '积极做多';
  reasons: string[];
  newsCount: number;
}

// 公告类型
interface StockNotice {
  art_code: string;
  title: string;
  notice_date: string;
  display_time: string;
  column_name: string;
  source_type: string;
}

// 公告详情
interface StockNoticeDetail {
  title: string;
  content: string;
  notice_date: string;
  display_time: string;
  source: string;
  column_name: string;
}

// 分析市场情绪
function analyzeMarketSentiment(news: any[]): MarketSentiment {
  let positive = 0;
  let negative = 0;
  const reasons: string[] = [];

  const keywords = {
    positive: ['利好','上涨','增长','突破','创新','爆发','涨停','大涨','反弹','回升','景气','强劲','订单','合作','业绩预增','政策支持'],
    negative: ['利空','下跌','亏损','风险','大跌','跌停','暴跌','警示','调查','处罚','减持','业绩预亏','疲软','低迷','震荡'],
  };

  for (const n of news) {
    const text = (n.title + ' ' + (n.content || '')).toLowerCase();
    for (const kw of keywords.positive) {
      if (text.includes(kw.toLowerCase())) { positive++; reasons.push(n.title.substring(0, 20)); break; }
    }
    for (const kw of keywords.negative) {
      if (text.includes(kw.toLowerCase())) { negative++; reasons.push(n.title.substring(0, 20)); break; }
    }
  }

  const total = positive + negative;
  const score = total === 0 ? 0 : Math.round(((positive - negative) / total) * 100);

  let level: MarketSentiment['level'];
  let recommendation: MarketSentiment['recommendation'];

  if (score >= 60) { level = '极强'; recommendation = '积极做多'; }
  else if (score >= 30) { level = '偏强'; recommendation = '适度参与'; }
  else if (score >= -30) { level = '中性'; recommendation = '轻仓参与'; }
  else if (score >= -60) { level = '偏弱'; recommendation = '空仓观望'; }
  else { level = '极弱'; recommendation = '空仓观望'; }

  return { score, level, recommendation, reasons: reasons.slice(0, 5), newsCount: news.length };
}

// 格式化时间
function formatTime(timeStr: string): string {
  if (!timeStr) return '';
  // timeStr 格式: 2026-03-16 10:30:00
  try {
    const date = new Date(timeStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const minutes = Math.floor(diff / (1000 * 60));

    if (minutes < 1) return '刚刚';
    if (minutes < 60) return `${minutes}分钟前`;
    if (hours < 24) return `${hours}小时前`;
    // 超过24小时显示具体日期
    return timeStr.substring(0, 16);
  } catch {
    return timeStr;
  }
}

function parseTimeMs(timeStr: string): number {
  if (!timeStr) return 0;
  const normalized = timeStr.replace('T', ' ').replace(/\.\d+Z$/, '').replace('Z', '');
  const ms = new Date(normalized).getTime();
  return Number.isFinite(ms) ? ms : 0;
}

// 解析后端返回的新闻数据
function parseNewsFromApi(data: any[]) {
  return data.map((item: any) => ({
    art_code: item.art_code,
    title: item.title,
    pub_time: item.display_time || item.notice_date,
    notice_date: item.notice_date,
    display_time: item.display_time,
    source: item.column_name || item.source_type || '东方财富',
    type: getNewsType(item.column_name || item.title),
    content: '',
    column_name: item.column_name,
  }));
}

// 解析财联社快讯
function parseClsNewsFromApi(data: any[]) {
  return data.map((item: any) => ({
    art_code: item.id || item.art_code || `${item.title}-${item.pub_time}`,
    title: item.title,
    pub_time: item.pub_time,
    notice_date: item.pub_time,
    display_time: item.pub_time,
    source: item.source || '财联社',
    type: getNewsType(item.category || '', item.title),
    content: item.content || '',
    column_name: item.category || '',
  }));
}

/** 过滤无效新闻，避免出现空标题/空正文占位卡片 */
function sanitizeNewsItems(items: any[]) {
  return items
    .map((item) => {
      const title = (item.title || '').trim();
      const content = (item.content || '').trim();
      const hasCore = title.length > 0 || content.length > 0;
      if (!hasCore) return null;
      return {
        ...item,
        title: title || content.slice(0, 48) || '（无标题）',
        content,
      };
    })
    .filter((x): x is any => x !== null);
}

function getNewsType(columnName: string, title: string = ''): string {
  if (!columnName && !title) return '行业';
  const text = (columnName + ' ' + title).toLowerCase();
  if (text.includes('政策') || text.includes('央行') || text.includes('证监会')) return '政策';
  if (text.includes('市场') || text.includes('大盘') || text.includes('指数')) return '市场';
  return '行业';
}

export default function NewsPanel() {
  const { openStock } = useContext(StockClickContext);
  const [news, setNews] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [category, setCategory] = useState<'all' | 'industry' | 'policy' | 'market'>('all');
  const [lastUpdate, setLastUpdate] = useState<string>('');
  const [expandedNews, setExpandedNews] = useState<number | null>(null);
  const [sentiment, setSentiment] = useState<MarketSentiment | null>(null);
  const [monitoredItems, setMonitoredItems] = useState<any[]>([]);
  const [notificationsEnabled, setNotificationsEnabled] = useState(false);
  const [selectedNotice, setSelectedNotice] = useState<StockNotice | null>(null);
  const [noticeDetail, setNoticeDetail] = useState<StockNoticeDetail | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const monitorIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // 默认股票列表用于获取新闻
  const defaultSymbols = ['000001', '600519', '601318', '600036', '300750'];

  const KEYWORD_MAP: Record<string, any> = {
    'AI': { sectors: ['人工智能', '科技', '半导体'], stocks: [{ symbol: '688256', name: '寒武纪' }] },
    '新能源': { sectors: ['新能源', '锂电池', '光伏'], stocks: [{ symbol: '300750', name: '宁德时代' }] },
    '医药': { sectors: ['医药', '医疗器械'], stocks: [{ symbol: '600276', name: '恒瑞医药' }] },
    '芯片': { sectors: ['半导体'], stocks: [{ symbol: '688981', name: '中芯国际' }] },
    '5G': { sectors: ['通信', '5G'], stocks: [{ symbol: '000063', name: '中兴通讯' }] },
    '军工': { sectors: ['国防军工'], stocks: [{ symbol: '600760', name: '中航沈飞' }] },
  };

  const analyzeNewsRelation = (title: string, content: string) => {
    const text = (title + ' ' + (content || '')).toLowerCase();
    const sectors = new Set<string>();
    const stocks = new Set<string>();
    for (const [keyword, mapping] of Object.entries(KEYWORD_MAP)) {
      if (text.includes(keyword.toLowerCase())) {
        mapping.sectors.forEach((s: string) => sectors.add(s));
        mapping.stocks.forEach((s: any) => stocks.add(JSON.stringify(s)));
      }
    }
    return { sectors: Array.from(sectors), stocks: Array.from(stocks).map((s: string) => JSON.parse(s)) };
  };

  const enableNotifications = async () => {
    if ('Notification' in window) {
      const perm = await Notification.requestPermission();
      setNotificationsEnabled(perm === 'granted');
    }
  };

  const sendNotification = (title: string, body: string) => {
    if ('Notification' in window && Notification.permission === 'granted') {
      new Notification(title, { body, icon: '/favicon.ico' });
    }
  };

  const addToMonitor = (name: string, type: 'sector' | 'stock') => {
    if (monitoredItems.find(i => i.name === name && i.type === type)) return;
    setMonitoredItems(prev => [...prev, { name, type, addTime: Date.now(), currentChange: 0, alerted: false }]);
    startMonitoring();
  };

  const startMonitoring = async () => {
    if (monitorIntervalRef.current) return;
    monitorIntervalRef.current = setInterval(async () => {
      try {
        const res = await fetch(`${API_BASE}/api/quotes`).then(r => r.json());
        if (!res.success) return;
        const quotes = res.data;
        setMonitoredItems(prev => prev.map(item => {
          if (Date.now() - item.addTime > 10 * 60 * 1000) return null;
          let change = 0;
          if (item.type === 'stock') {
            const quote = quotes.find((q: any) => q.name?.includes(item.name));
            change = quote?.change_pct || 0;
          }
          if (change >= 2.0 && !item.alerted) {
            if (notificationsEnabled) sendNotification(`🚀 ${item.name} 异动提醒`, `涨幅${change.toFixed(1)}%！`);
            return { ...item, currentChange: change, alerted: true };
          }
          return { ...item, currentChange: change };
        }).filter(Boolean));
      } catch (e) {}
    }, 5000);
  };

  useEffect(() => {
    return () => { if (monitorIntervalRef.current) clearInterval(monitorIntervalRef.current); };
  }, []);

  const fetchNews = async () => {
    setLoading(true);
    try {
      // 优先使用财联社快讯（更新更及时）
      const clsRes = await fetch(`${API_BASE}/api/cls/news?page=1&page_size=80`);
      const clsJson = await clsRes.json();

      let merged: any[] = [];
      if (clsJson.success && clsJson.data?.list?.length) {
        merged = sanitizeNewsItems(parseClsNewsFromApi(clsJson.data.list));
      } else {
        // 回退到多只股票公告
        const allNotices: any[] = [];
        for (const symbol of defaultSymbols) {
          try {
            const res = await fetch(`${API_BASE}/api/notices/${symbol}?page=1&page_size=10`);
            const json = await res.json();
            if (json.success && json.data?.list) {
              allNotices.push(...parseNewsFromApi(json.data.list));
            }
          } catch (e) {
            console.error(`Failed to fetch notices for ${symbol}:`, e);
          }
        }
        merged = sanitizeNewsItems(allNotices);
      }

      // 按真实时间倒序（避免字符串比较导致旧消息排前）
      merged.sort((a, b) => parseTimeMs(b.pub_time) - parseTimeMs(a.pub_time));

      // 去重（标题+时间）
      const uniqueNotices = merged.filter((item, index, self) =>
        index === self.findIndex((t) => t.title === item.title && t.pub_time === item.pub_time)
      );

      const analyzed = uniqueNotices.map(n => ({ ...n, ...analyzeNewsRelation(n.title, n.content || '') }));
      setNews(analyzed);
      setSentiment(analyzeMarketSentiment(analyzed));
    } catch (e) {
      console.error('Failed to fetch news:', e);
      // 使用fallback数据
      const fallback = getFallbackNews();
      setNews(fallback);
      setSentiment(analyzeMarketSentiment(fallback));
    }
    setLastUpdate(new Date().toLocaleTimeString());
    setLoading(false);
  };

  // 获取公告详情
  const fetchNoticeDetail = async (notice: StockNotice) => {
    setSelectedNotice(notice);
    setNoticeDetail(null);
    setDetailLoading(true);

    try {
      // 尝试从财经新闻API获取详情
      const newsRes = await fetch(`${API_BASE}/api/news/detail/${notice.art_code}`);
      const newsJson = await newsRes.json();
      if (newsJson.success && newsJson.data && newsJson.data.content && !newsJson.data.content.includes('无法获取')) {
        setNoticeDetail({
          title: newsJson.data.title,
          content: newsJson.data.content,
          notice_date: newsJson.data.pub_time,
          display_time: newsJson.data.pub_time,
          source: newsJson.data.source,
          column_name: newsJson.data.category,
        });
        setDetailLoading(false);
        return;
      }
    } catch (e) {
      console.log('News API failed, trying notice API:', e);
    }

    // 回退到公告API
    try {
      const res = await fetch(`${API_BASE}/api/notice/${notice.art_code}`);
      const json = await res.json();
      if (json.success && json.data) {
        setNoticeDetail(json.data);
      }
    } catch (e) {
      console.error('Failed to fetch notice detail:', e);
    }

    setDetailLoading(false);
  };

  useEffect(() => {
    fetchNews();
    enableNotifications();
    const timer = setInterval(fetchNews, 60_000);
    return () => clearInterval(timer);
  }, []);

  const isMonitored = (name: string, type: 'sector' | 'stock') => monitoredItems.some(i => i.name === name && i.type === type);

  const categories = [
    { id: 'all', name: '全部', icon: <Newspaper class="w-4 h-4" /> },
    { id: 'industry', name: '行业', icon: <Building2 class="w-4 h-4" /> },
    { id: 'policy', name: '政策', icon: <TrendingUp class="w-4 h-4" /> },
    { id: 'market', name: '市场', icon: <AlertTriangle class="w-4 h-4" /> },
  ];

  const getSentimentColor = (level: string) => {
    if (level.includes('极强') || level.includes('积极')) return 'text-green-400';
    if (level.includes('偏强') || level.includes('适度')) return 'text-green-300';
    if (level.includes('中性') || level.includes('轻仓')) return 'text-yellow-400';
    if (level.includes('偏弱') || level.includes('空仓')) return 'text-red-300';
    return 'text-red-400';
  };

  const getSentimentBg = (level: string) => {
    if (level.includes('极强') || level.includes('积极')) return 'bg-green-500/20 border-green-500/30';
    if (level.includes('偏强') || level.includes('适度')) return 'bg-green-500/10 border-green-500/20';
    if (level.includes('中性') || level.includes('轻仓')) return 'bg-yellow-500/10 border-yellow-500/20';
    return 'bg-red-500/10 border-red-500/20';
  };

  const filtered = category === 'all' ? news : news.filter(n => n.type === category);

  return (
    <div class="p-4 space-y-4">
      {/* 市场情绪面板 */}
      {sentiment && (
        <div class={`rounded-lg p-4 border ${getSentimentBg(sentiment.level)}`}>
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <Activity class="w-5 h-5" />
              <span class="font-semibold">市场情绪分析</span>
            </div>
            <button onClick={fetchNews} disabled={loading} class="p-1 hover:bg-muted rounded">
              <RefreshCw class={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
            </button>
          </div>

          <div class="flex items-center gap-6 mb-3">
            <div>
              <div class="text-xs text-muted-foreground">情绪指数</div>
              <div class={`text-2xl font-bold ${getSentimentColor(sentiment.level)}`}>{sentiment.score}</div>
            </div>
            <div>
              <div class="text-xs text-muted-foreground">市场强度</div>
              <div class={`text-lg font-semibold ${getSentimentColor(sentiment.level)}`}>{sentiment.level}</div>
            </div>
            <div>
              <div class="text-xs text-muted-foreground">操作建议</div>
              <div class={`text-lg font-bold ${getSentimentColor(sentiment.recommendation)}`}>{sentiment.recommendation}</div>
            </div>
          </div>

          {sentiment.reasons.length > 0 && (
            <div class="text-xs text-muted-foreground">
              依据: {sentiment.reasons.join('、')}
            </div>
          )}
        </div>
      )}

      {/* 监控面板 */}
      {monitoredItems.length > 0 && (
        <div class="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-3">
          <div class="flex items-center justify-between mb-2">
            <div class="text-xs font-medium text-yellow-400 flex items-center gap-1">
              <Eye class="w-3 h-3" /> 实时监控中
            </div>
            <span class="text-xs text-muted-foreground">{monitoredItems.length}个</span>
          </div>
          <div class="flex flex-wrap gap-2">
            {monitoredItems.map((item, i) => (
              <div key={i} class={`flex items-center gap-1 px-2 py-1 rounded text-xs ${
                item.alerted ? 'bg-red-500/20 text-red-400' : 'bg-yellow-500/20 text-yellow-400'
              }`}>
                <span>{item.type === 'sector' ? '📌' : '📈'}{item.name}</span>
                <span class="font-bold">{item.currentChange > 0 ? '+' : ''}{item.currentChange.toFixed(1)}%</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 分类 */}
      <div class="flex gap-2 overflow-x-auto pb-2">
        {categories.map(cat => (
          <button key={cat.id} onClick={() => setCategory(cat.id as any)}
            class={`flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs whitespace-nowrap ${
              category === cat.id ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
            }`}>
            {cat.icon} {cat.name}
          </button>
        ))}
      </div>

      {/* 新闻列表 */}
      {loading && news.length === 0 ? (
        <div class="text-center py-8 text-muted-foreground">加载中...</div>
      ) : (
        <div class="space-y-3 max-h-[50vh] overflow-y-auto">
          {filtered.map((item, i) => (
            <div key={i} class="bg-card rounded-lg border overflow-hidden">
              <div class="p-4 cursor-pointer hover:bg-muted/30" onClick={() => setExpandedNews(expandedNews === i ? null : i)}>
                <div class="flex items-start justify-between">
                  <div>
                    <div class="flex items-center gap-2 mb-1">
                      <span class={`px-2 py-0.5 text-xs rounded border ${
                        item.type === '政策' ? 'bg-blue-500/20 text-blue-400 border-blue-500/30' : 'bg-green-500/20 text-green-400 border-green-500/30'
                      }`}>{item.type}</span>
                      <span class="text-xs text-muted-foreground">{formatTime(item.pub_time)}</span>
                    </div>
                    <h3 class="font-medium text-sm">{item.title}</h3>
                  </div>
                  <ChevronDown class={`w-4 h-4 text-muted-foreground ${expandedNews === i ? 'rotate-180' : ''}`} />
                </div>
              </div>

              {expandedNews === i && (
                <div class="px-4 pb-4 border-t bg-muted/20">
                  <button
                    onClick={(e) => { e.stopPropagation(); fetchNoticeDetail(item); }}
                    class="mt-3 px-3 py-1.5 bg-primary text-primary-foreground text-xs rounded hover:bg-primary/80"
                  >
                    查看详情
                  </button>

                  {item.sectors?.length > 0 && (
                    <div class="mt-3">
                      <div class="text-xs font-medium mb-1">📌 相关板块</div>
                      <div class="flex gap-1 flex-wrap">
                        {item.sectors.map((sector: string, j: number) => (
                          <button key={j} onClick={() => addToMonitor(sector, 'sector')}
                            class={`px-2 py-0.5 text-xs rounded ${
                              isMonitored(sector, 'sector') ? 'bg-yellow-500/20 text-yellow-400' : 'bg-blue-500/20 text-blue-400 hover:bg-blue-500/30'
                            }`}>
                            {isMonitored(sector, 'sector') ? '⏱ 监控中' : '+ 监控'}
                          </button>
                        ))}
                      </div>
                    </div>
                  )}

                  {item.stocks?.length > 0 && (
                    <div class="mt-3">
                      <div class="text-xs font-medium mb-1">📈 相关股票</div>
                      <div class="flex gap-1 flex-wrap">
                        {item.stocks.map((stock: any, j: number) => (
                          <div key={j} class="flex items-center gap-1">
                            <button onClick={() => openStock(stock.symbol, stock.name)} class="px-2 py-0.5 bg-green-500/20 text-green-400 text-xs rounded hover:bg-green-500/30">
                              {stock.name}
                            </button>
                            <button onClick={() => addToMonitor(stock.name, 'stock')} disabled={isMonitored(stock.name, 'stock')}
                              class={`px-1 py-0.5 text-xs rounded ${
                                isMonitored(stock.name, 'stock') ? 'bg-yellow-500/20 text-yellow-400' : 'bg-gray-500/20 text-gray-400 hover:bg-gray-500/30'
                              }`}>
                              {isMonitored(stock.name, 'stock') ? '✓' : '+'}
                            </button>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                  <div class="mt-3 text-xs text-muted-foreground">来源: {item.source}</div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* 公告详情弹窗 */}
      {selectedNotice && (
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setSelectedNotice(null)}>
          <div class="bg-card rounded-lg border max-w-2xl w-full max-h-[80vh] overflow-hidden" onClick={e => e.stopPropagation()}>
            <div class="flex items-center justify-between p-4 border-b">
              <h3 class="font-semibold">公告详情</h3>
              <button onClick={() => setSelectedNotice(null)} class="p-1 hover:bg-muted rounded">
                <X class="w-5 h-5" />
              </button>
            </div>
            <div class="p-4 overflow-y-auto max-h-[calc(80vh-60px)]">
              {detailLoading ? (
                <div class="text-center py-8 text-muted-foreground">加载中...</div>
              ) : noticeDetail ? (
                <div class="space-y-4">
                  <h4 class="text-lg font-medium">{noticeDetail.title}</h4>
                  <div class="text-xs text-muted-foreground">
                    <span>公告日期: {noticeDetail.notice_date}</span>
                    {noticeDetail.display_time && <span class="ml-4">发布时间: {noticeDetail.display_time}</span>}
                  </div>
                  {noticeDetail.column_name && (
                    <div class="text-xs text-muted-foreground">栏目: {noticeDetail.column_name}</div>
                  )}
                  {noticeDetail.source && (
                    <div class="text-xs text-muted-foreground">来源: {noticeDetail.source}</div>
                  )}
                  <div class="border-t pt-4">
                    <div class="prose prose-sm max-w-none whitespace-pre-wrap text-sm">
                      {noticeDetail.content || '暂无详细内容'}
                    </div>
                  </div>
                </div>
              ) : (
                <div class="text-center py-8 text-muted-foreground">加载失败</div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function getFallbackNews() {
  return [
    { title: 'AI芯片需求持续爆发 半导体板块再掀涨停潮', pub_time: '2026-03-16 14:30', source: '证券时报', type: '行业', content: 'AI芯片需求持续爆发。' },
    { title: '新能源车销量同比增45%', pub_time: '2026-03-16 13:00', source: '中汽协', type: '行业', content: '新能源车销量增长。' },
    { title: '医药板块迎来反弹', pub_time: '2026-03-16 09:30', source: '证券日报', type: '行业', content: '医药板块反弹。' },
    { title: '央行：保持流动性合理充裕', pub_time: '2026-03-16 15:30', source: '央行', type: '政策', content: '货币政策支持。' },
    { title: 'A股成交超1.2万亿', pub_time: '2026-03-16 15:00', source: '东方财富', type: '市场', content: '市场交投活跃。' },
  ];
}
