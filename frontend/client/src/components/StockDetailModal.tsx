/**
 * StockDetailModal - 股票详情弹窗
 * Design: 暗夜终端 - 多标签详情面板，K线图 + 基本信息 + 资金流向
 */
import { useState, useEffect, useCallback, useMemo } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogClose } from '@/components/ui/dialog';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import DailyKChart from './DailyKChart';
import OfficialIntradayChart from './OfficialIntradayChart';
import { TrendingUp, TrendingDown, RefreshCw, X, ExternalLink, Star } from 'lucide-react';
import { formatPrice, formatPercent, formatNumber, getChangeColor, addToWatchlist, removeFromWatchlist } from '@/hooks/useMarketData';
import { toast } from 'sonner';

const API_BASE = import.meta.env.VITE_API_BASE || '';

interface StockDetail {
  symbol: string;
  name: string;
  price: number;
  change: number;
  change_pct: number;
  open: number;
  high: number;
  low: number;
  pre_close: number;
  volume: number;
  turnover: number;       // 成交额 (元)
  turnover_rate: number; // 换手率 (%)
  amplitude: number;     // 振幅 (%)
  pe_ratio: number;      // 市盈率
  total_market_cap: number;
  circulating_market_cap: number;
  timestamp?: string;
}

interface Candle {
  timestamp: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  turnover: number;
}

interface MoneyFlowDetail {
  symbol: string;
  name: string;
  main_net_inflow: number;
  super_large_inflow: number;
  large_inflow: number;
  medium_inflow: number;
  small_inflow: number;
}

// 资金流向条形图
function MoneyFlowBar({ label, value, maxVal }: { label: string; value: number; maxVal: number }) {
  const isPositive = value >= 0;
  const pct = maxVal > 0 ? Math.abs(value) / maxVal * 100 : 0;
  return (
    <div className="flex items-center gap-3 py-1.5">
      <span className="text-xs text-muted-foreground w-12 shrink-0">{label}</span>
      <div className="flex-1 h-4 bg-muted rounded-sm overflow-hidden relative">
        <div
          className={`h-full rounded-sm transition-all duration-500 ${isPositive ? 'bg-up/70' : 'bg-down/70'}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className={`text-xs font-mono-data w-20 text-right shrink-0 ${isPositive ? 'text-up' : 'text-down'}`}>
        {isPositive ? '+' : ''}{formatNumber(value)}
      </span>
    </div>
  );
}

interface StockDetailModalProps {
  symbol: string | null;
  name?: string;
  onClose: () => void;
}

export default function StockDetailModal({ symbol, name, onClose }: StockDetailModalProps) {
  const [detail, setDetail] = useState<StockDetail | null>(null);
  const [candles, setCandles] = useState<Candle[]>([]);
  const [intradayData, setIntradayData] = useState<any>(null);
  const [moneyFlow, setMoneyFlow] = useState<MoneyFlowDetail | null>(null);
  const [news, setNews] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [period, setPeriod] = useState<'1m' | '5m' | '15m' | '1d'>('1d');

  // period 对应的 API 参数 (直接使用 period 字符串)
  const periodMap = {
    '1m': { count: 240 },    // 1分钟
    '5m': { count: 240 },    // 5分钟
    '15m': { count: 80 },    // 15分钟
    '1d': { count: 90 },    // 日线
  };
  const [favLoading, setFavLoading] = useState(false);
  const [isFav, setIsFav] = useState(false);

  const fetchDetail = useCallback(async () => {
    if (!symbol) return;
    setLoading(true);
    try {
      // 根据 period 获取不同的数据
      const isDaily = period === '1d';
      const range = isDaily ? '1d' : period;

      const [detailRes, candleRes, intradayRes, flowRes, watchlistRes, newsRes] = await Promise.allSettled([
        fetch(`${API_BASE}/api/quotes/${symbol}`).then(r => r.json()),
        // 日线用日K线数据，分时用对应周期的K线数据
        fetch(`${API_BASE}/api/candles/${symbol}?period=${period}&count=${isDaily ? 120 : 240}`).then(r => r.json()),
        // 分时数据
        fetch(`${API_BASE}/api/intraday/${symbol}?range=${range}`).then(r => r.json()),
        fetch(`${API_BASE}/api/money-flow`).then(r => r.json()),
        fetch(`${API_BASE}/api/watchlist`).then(r => r.json()),
        fetch(`${API_BASE}/api/news/${symbol}`).then(r => r.json()),
      ]);

      if (detailRes.status === 'fulfilled' && detailRes.value.success) {
        setDetail(detailRes.value.data);
      }
      if (candleRes.status === 'fulfilled' && candleRes.value.success) {
        setCandles(candleRes.value.data || []);
      }
      if (intradayRes.status === 'fulfilled' && intradayRes.value.success) {
        setIntradayData(intradayRes.value.data);
      }
      if (flowRes.status === 'fulfilled' && flowRes.value.success) {
        const flows: MoneyFlowDetail[] = flowRes.value.data || [];
        const found = flows.find((f: MoneyFlowDetail) => f.symbol === symbol);
        setMoneyFlow(found || null);
      }
      if (watchlistRes.status === 'fulfilled' && watchlistRes.value.success) {
        const list: any[] = watchlistRes.value.data || [];
        const exists = list.some((w) => w.symbol === symbol);
        setIsFav(exists);
      } else {
        setIsFav(false);
      }
      if (newsRes.status === 'fulfilled' && newsRes.value.success) {
        setNews(newsRes.value.data || []);
      } else {
        setNews([]);
      }
    } catch (e) {
      console.error('Failed to fetch stock detail', e);
    } finally {
      setLoading(false);
    }
  }, [symbol, period]);

  useEffect(() => {
    if (symbol) {
      setDetail(null);
      setCandles([]);
      setIntradayData(null);
      setMoneyFlow(null);
      fetchDetail();
    }
  }, [symbol, fetchDetail]);

  // 处理K线数据 - 所有周期数据返回都是降序，需要反转成升序
  const sortedCandles = useMemo(() => {
    return [...candles].reverse().map(c => ({
      timestamp: c.timestamp,
      open: c.open,
      high: c.high,
      low: c.low,
      close: c.close,
      volume: c.volume,
      turnover: c.turnover,
    }));
  }, [candles]);

  // 判断是否显示分时图（1m/5m/15m显示分时，1d显示日K线）
  const showIntraday = period !== '1d' && intradayData?.points?.length > 0;

  const isUp = (detail?.change_pct ?? 0) >= 0;
  const maxFlowVal = moneyFlow
    ? Math.max(
        Math.abs(moneyFlow.main_net_inflow),
        Math.abs(moneyFlow.super_large_inflow),
        Math.abs(moneyFlow.large_inflow),
        Math.abs(moneyFlow.medium_inflow),
        Math.abs(moneyFlow.small_inflow),
      )
    : 0;

  return (
    <Dialog open={!!symbol} onOpenChange={(open) => { if (!open) onClose(); }}>
      <DialogContent className="max-w-[90vw] w-full bg-card border-border text-foreground p-0 gap-0 overflow-hidden max-h-[95vh] flex flex-col" showCloseButton={false}>
        {/* Header */}
        <DialogHeader className="px-5 py-3.5 border-b border-border shrink-0">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div>
                <DialogTitle className="text-base font-bold flex items-center gap-2">
                  {detail?.name || name || symbol}
                  <span className="text-xs font-mono-data text-muted-foreground font-normal">{symbol}</span>
                </DialogTitle>
              </div>
              {detail && (
                <div className="flex items-center gap-2 ml-2">
                  <span className={`font-mono-data text-xl font-bold ${getChangeColor(detail.change_pct)}`}>
                    {formatPrice(detail.price)}
                  </span>
                  <div className="flex flex-col">
                    <span className={`font-mono-data text-xs ${getChangeColor(detail.change_pct)}`}>
                      {detail.change > 0 ? '+' : ''}{detail.change.toFixed(2)}
                    </span>
                    <span className={`font-mono-data text-xs font-semibold ${getChangeColor(detail.change_pct)}`}>
                      {formatPercent(detail.change_pct)}
                    </span>
                  </div>
                  {isUp ? <TrendingUp className="w-4 h-4 text-up" /> : <TrendingDown className="w-4 h-4 text-down" />}
                </div>
              )}
            </div>
            <div className="flex items-center gap-2">
              {detail && (
                <button
                  disabled={favLoading}
                  onClick={async () => {
                    if (!detail) return;
                    try {
                      setFavLoading(true);
                      if (isFav) {
                        const res = await removeFromWatchlist(detail.symbol);
                        if (res.success) {
                          setIsFav(false);
                          toast.success('已移除自选股', { description: `${detail.name} (${detail.symbol})` });
                        } else {
                          toast.error('取消自选失败', { description: res.message || '请稍后重试' });
                        }
                      } else {
                        const res = await addToWatchlist({ symbol: detail.symbol, name: detail.name });
                        if (res.success) {
                          setIsFav(true);
                          toast.success('已加入自选股', { description: `${detail.name} (${detail.symbol})` });
                        } else {
                          toast.error('加入自选股失败', { description: res.message || '请稍后重试' });
                        }
                      }
                    } catch (e) {
                      toast.error('操作自选股失败', { description: '网络异常，请检查后端服务' });
                    } finally {
                      setFavLoading(false);
                    }
                  }}
                  className={`transition-colors p-1.5 rounded hover:bg-muted disabled:opacity-60 ${
                    isFav ? 'text-yellow-400' : 'text-muted-foreground hover:text-yellow-300'
                  }`}
                  title={isFav ? '取消自选股' : '加入自选股'}
                >
                  <Star className={`w-4 h-4 ${isFav ? 'fill-yellow-400/90' : ''}`} />
                </button>
              )}
              <button
                onClick={fetchDetail}
                className="text-muted-foreground hover:text-foreground transition-colors p-1 rounded"
                title="刷新"
              >
                <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
              </button>
              <button
                onClick={onClose}
                className="text-muted-foreground hover:text-foreground transition-colors p-1.5 rounded hover:bg-muted"
                title="关闭"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
          </div>
        </DialogHeader>

        {/* Content */}
        <div className="flex-1 overflow-y-auto">
          <Tabs defaultValue="chart" className="flex flex-col h-full">
            <TabsList className="mx-5 mt-3 mb-0 bg-muted/50 w-fit shrink-0">
              <TabsTrigger value="chart" className="text-xs">K线图</TabsTrigger>
              <TabsTrigger value="info" className="text-xs">基本信息</TabsTrigger>
              <TabsTrigger value="flow" className="text-xs">资金流向</TabsTrigger>
              <TabsTrigger value="news" className="text-xs">新闻公告</TabsTrigger>
            </TabsList>

            {/* K线图 Tab */}
            <TabsContent value="chart" className="px-5 pb-5 mt-3">
              {/* Period Selector */}
              <div className="flex gap-1 mb-3">
                {(['1d', 'intraday'] as const).map(p => (
                  <button
                    key={p}
                    onClick={() => setPeriod(p === 'intraday' ? '5m' : '1d')}
                    className={`text-xs px-2.5 py-1 rounded transition-colors ${
                      (p === 'intraday' ? period !== '1d' : period === '1d')
                        ? 'bg-primary text-primary-foreground'
                        : 'bg-muted text-muted-foreground hover:text-foreground'
                    }`}
                  >
                    {p === '1d' ? '日线' : '分时'}
                  </button>
                ))}
              </div>

              {/* 东方财富官方风格K线图/分时图 */}
              {loading ? (
                <div className="h-[370px] flex items-center justify-center text-muted-foreground">
                  <div className="flex items-center gap-2">
                    <RefreshCw className="w-4 h-4 animate-spin" />
                    <span>加载K线数据...</span>
                  </div>
                </div>
              ) : showIntraday && intradayData ? (
                <OfficialIntradayChart series={intradayData} />
              ) : sortedCandles.length > 0 ? (
                <DailyKChart candles={sortedCandles} period="1d" height={370} />
              ) : (
                <div className="h-[370px] flex items-center justify-center text-muted-foreground">
                  <span>暂无K线数据</span>
                </div>
              )}

            </TabsContent>

            {/* 基本信息 Tab */}
            <TabsContent value="info" className="px-5 pb-5 mt-3">
              {detail ? (
                <div className="space-y-4">
                  {/* 今日行情 */}
                  <div>
                    <h4 className="text-xs font-semibold text-muted-foreground mb-2 uppercase tracking-wider">今日行情</h4>
                    <div className="grid grid-cols-3 gap-2">
                      {[
                        { label: '开盘价', value: formatPrice(detail.open) },
                        { label: '最高价', value: formatPrice(detail.high), color: 'text-up' },
                        { label: '最低价', value: formatPrice(detail.low), color: 'text-down' },
                        { label: '昨收价', value: formatPrice(detail.pre_close) },
                        { label: '振幅', value: `${detail.amplitude?.toFixed(2) ?? '—'}%` },
                        { label: '换手率', value: `${detail.turnover_rate?.toFixed(2) ?? '—'}%` },
                      ].map(item => (
                        <div key={item.label} className="bg-muted/40 rounded-lg p-2.5 border border-border/30">
                          <div className="text-[10px] text-muted-foreground mb-0.5">{item.label}</div>
                          <div className={`font-mono-data text-sm font-medium ${item.color || 'text-foreground'}`}>
                            {item.value}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>

                  {/* 成交数据 */}
                  <div>
                    <h4 className="text-xs font-semibold text-muted-foreground mb-2 uppercase tracking-wider">成交数据</h4>
                    <div className="grid grid-cols-2 gap-2">
                      {[
                        { label: '成交量', value: formatNumber(detail.volume, 0) + ' 手' },
                        { label: '成交额', value: formatNumber(detail.turnover) },
                        { label: '总市值', value: formatNumber(detail.total_market_cap) },
                        { label: '流通市值', value: formatNumber(detail.circulating_market_cap) },
                      ].map(item => (
                        <div key={item.label} className="bg-muted/40 rounded-lg p-2.5 border border-border/30">
                          <div className="text-[10px] text-muted-foreground mb-0.5">{item.label}</div>
                          <div className="font-mono-data text-sm font-medium text-foreground">{item.value}</div>
                        </div>
                      ))}
                    </div>
                  </div>

                  {/* 估值指标 */}
                  <div>
                    <h4 className="text-xs font-semibold text-muted-foreground mb-2 uppercase tracking-wider">估值指标</h4>
                    <div className="grid grid-cols-2 gap-2">
                      {[
                        { label: '市盈率 (TTM)', value: detail.pe_ratio > 0 ? detail.pe_ratio.toFixed(2) : '—' },
                        { label: '股票代码', value: detail.symbol },
                      ].map(item => (
                        <div key={item.label} className="bg-muted/40 rounded-lg p-2.5 border border-border/30">
                          <div className="text-[10px] text-muted-foreground mb-0.5">{item.label}</div>
                          <div className="font-mono-data text-sm font-medium text-foreground">{item.value}</div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              ) : (
                <div className="h-40 flex items-center justify-center text-muted-foreground text-sm">
                  {loading ? (
                    <div className="flex items-center gap-2">
                      <RefreshCw className="w-4 h-4 animate-spin" />
                      <span>加载中...</span>
                    </div>
                  ) : (
                    <span>暂无数据</span>
                  )}
                </div>
              )}
            </TabsContent>

            {/* 资金流向 Tab */}
            <TabsContent value="flow" className="px-5 pb-5 mt-3">
              {moneyFlow ? (
                <div>
                  <h4 className="text-xs font-semibold text-muted-foreground mb-3 uppercase tracking-wider">今日资金流向</h4>
                  <div className="space-y-1">
                    <MoneyFlowBar label="主力净" value={moneyFlow.main_net_inflow} maxVal={maxFlowVal} />
                    <MoneyFlowBar label="超大单" value={moneyFlow.super_large_inflow} maxVal={maxFlowVal} />
                    <MoneyFlowBar label="大单" value={moneyFlow.large_inflow} maxVal={maxFlowVal} />
                    <MoneyFlowBar label="中单" value={moneyFlow.medium_inflow} maxVal={maxFlowVal} />
                    <MoneyFlowBar label="小单" value={moneyFlow.small_inflow} maxVal={maxFlowVal} />
                  </div>

                  {/* 主力净流入汇总 */}
                  <div className={`mt-4 p-3 rounded-lg border ${
                    moneyFlow.main_net_inflow >= 0
                      ? 'bg-up/10 border-up/20'
                      : 'bg-down/10 border-down/20'
                  }`}>
                    <div className="text-xs text-muted-foreground mb-1">主力净流入</div>
                    <div className={`font-mono-data text-lg font-bold ${
                      moneyFlow.main_net_inflow >= 0 ? 'text-up' : 'text-down'
                    }`}>
                      {moneyFlow.main_net_inflow >= 0 ? '+' : ''}{formatNumber(moneyFlow.main_net_inflow)}
                    </div>
                  </div>
                </div>
              ) : (
                <div className="h-40 flex items-center justify-center text-muted-foreground text-sm">
                  {loading ? (
                    <div className="flex items-center gap-2">
                      <RefreshCw className="w-4 h-4 animate-spin" />
                      <span>加载资金流向...</span>
                    </div>
                  ) : (
                    <span>该股票暂无资金流向数据</span>
                  )}
                </div>
              )}
            </TabsContent>

            {/* 新闻公告 Tab */}
            <TabsContent value="news" className="mt-4">
              <div className="max-h-80 overflow-y-auto">
                {news && news.length > 0 ? (
                  <div className="space-y-2">
                    {news.map((item, idx) => (
                      <a
                        key={idx}
                        href={item.url || `https://guba.eastmoney.com/`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="block p-3 rounded-lg border border-border hover:bg-accent/50 transition-colors"
                      >
                        <div className="flex items-start justify-between gap-2">
                          <div className="flex-1 min-w-0">
                            <div className="text-sm font-medium line-clamp-2">{item.title}</div>
                            {item.pub_date && (
                              <div className="text-xs text-muted-foreground mt-1">{item.pub_date}</div>
                            )}
                          </div>
                          <span className="shrink-0 px-2 py-0.5 text-xs rounded bg-secondary">
                            {item.news_type}
                          </span>
                        </div>
                      </a>
                    ))}
                  </div>
                ) : (
                  <div className="h-40 flex items-center justify-center text-muted-foreground text-sm">
                    {loading ? (
                      <div className="flex items-center gap-2">
                        <RefreshCw className="w-4 h-4 animate-spin" />
                        <span>加载新闻...</span>
                      </div>
                    ) : (
                      <span>该股票暂无新闻数据</span>
                    )}
                  </div>
                )}
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </DialogContent>
    </Dialog>
  );
}
