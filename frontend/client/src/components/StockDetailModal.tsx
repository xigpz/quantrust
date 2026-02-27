/**
 * StockDetailModal - 股票详情弹窗
 * Design: 暗夜终端 - 多标签详情面板，K线图 + 基本信息 + 资金流向
 */
import { useState, useEffect, useCallback } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import {
  ComposedChart, Bar, Line, XAxis, YAxis, CartesianGrid, Tooltip,
  ResponsiveContainer, Cell
} from 'recharts';
import { TrendingUp, TrendingDown, RefreshCw, X, ExternalLink } from 'lucide-react';
import { formatPrice, formatPercent, formatNumber, getChangeColor } from '@/hooks/useMarketData';

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
  turnover: number;
  turnover_rate: number;
  amplitude: number;
  pe_ratio: number;
  total_market_cap: number;
  circulating_market_cap: number;
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

// 自定义K线蜡烛图 Tooltip
function CandleTooltip({ active, payload }: any) {
  if (!active || !payload?.length) return null;
  const d = payload[0]?.payload;
  if (!d) return null;
  const isUp = d.close >= d.open;
  return (
    <div className="bg-card border border-border rounded-lg p-2.5 text-xs shadow-xl">
      <div className="text-muted-foreground mb-1.5">{d.date}</div>
      <div className="grid grid-cols-2 gap-x-4 gap-y-0.5">
        <span className="text-muted-foreground">开盘</span>
        <span className="font-mono-data text-right">{d.open.toFixed(2)}</span>
        <span className="text-muted-foreground">收盘</span>
        <span className={`font-mono-data text-right ${isUp ? 'text-up' : 'text-down'}`}>{d.close.toFixed(2)}</span>
        <span className="text-muted-foreground">最高</span>
        <span className="font-mono-data text-right text-up">{d.high.toFixed(2)}</span>
        <span className="text-muted-foreground">最低</span>
        <span className="font-mono-data text-right text-down">{d.low.toFixed(2)}</span>
        <span className="text-muted-foreground">成交量</span>
        <span className="font-mono-data text-right">{formatNumber(d.volume, 0)}</span>
      </div>
    </div>
  );
}

// 自定义蜡烛图形状
function CandleBar(props: any) {
  const { x, y, width, height, payload } = props;
  if (!payload) return null;
  const { open, close, high, low } = payload;
  const isUp = close >= open;
  const color = isUp ? '#ef4444' : '#22c55e';
  const bodyTop = Math.min(open, close);
  const bodyBottom = Math.max(open, close);
  const scale = props.yAxisScale;
  if (!scale) return null;

  const yTop = scale(high);
  const yBodyTop = scale(bodyTop);
  const yBodyBottom = scale(bodyBottom);
  const yBottom = scale(low);
  const centerX = x + width / 2;
  const bodyHeight = Math.max(1, yBodyBottom - yBodyTop);

  return (
    <g>
      {/* 上影线 */}
      <line x1={centerX} y1={yTop} x2={centerX} y2={yBodyTop} stroke={color} strokeWidth={1} />
      {/* 实体 */}
      <rect x={x + 1} y={yBodyTop} width={Math.max(1, width - 2)} height={bodyHeight} fill={color} />
      {/* 下影线 */}
      <line x1={centerX} y1={yBodyBottom} x2={centerX} y2={yBottom} stroke={color} strokeWidth={1} />
    </g>
  );
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
  const [moneyFlow, setMoneyFlow] = useState<MoneyFlowDetail | null>(null);
  const [loading, setLoading] = useState(false);
  const [period, setPeriod] = useState<'1d' | '1w' | '1M'>('1d');

  const fetchDetail = useCallback(async () => {
    if (!symbol) return;
    setLoading(true);
    try {
      const [detailRes, candleRes, flowRes] = await Promise.allSettled([
        fetch(`${API_BASE}/api/stocks/${symbol}`).then(r => r.json()),
        fetch(`${API_BASE}/api/candles/${symbol}?period=${period}&count=90`).then(r => r.json()),
        fetch(`${API_BASE}/api/money-flow`).then(r => r.json()),
      ]);

      if (detailRes.status === 'fulfilled' && detailRes.value.success) {
        setDetail(detailRes.value.data);
      }
      if (candleRes.status === 'fulfilled' && candleRes.value.success) {
        setCandles(candleRes.value.data || []);
      }
      if (flowRes.status === 'fulfilled' && flowRes.value.success) {
        const flows: MoneyFlowDetail[] = flowRes.value.data || [];
        const found = flows.find((f: MoneyFlowDetail) => f.symbol === symbol);
        setMoneyFlow(found || null);
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
      setMoneyFlow(null);
      fetchDetail();
    }
  }, [symbol, fetchDetail]);

  // 处理K线数据
  const chartData = candles.map(c => ({
    date: c.timestamp.slice(0, 10),
    open: c.open,
    high: c.high,
    low: c.low,
    close: c.close,
    volume: c.volume,
    isUp: c.close >= c.open,
  }));

  // 计算K线Y轴范围
  const prices = chartData.flatMap(d => [d.high, d.low]).filter(Boolean);
  const minPrice = prices.length ? Math.min(...prices) * 0.995 : 0;
  const maxPrice = prices.length ? Math.max(...prices) * 1.005 : 100;

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
      <DialogContent className="max-w-3xl w-full bg-card border-border text-foreground p-0 gap-0 overflow-hidden max-h-[90vh] flex flex-col">
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
              <button
                onClick={fetchDetail}
                className="text-muted-foreground hover:text-foreground transition-colors p-1 rounded"
              >
                <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
              </button>
              <button
                onClick={onClose}
                className="text-muted-foreground hover:text-foreground transition-colors p-1 rounded"
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
            </TabsList>

            {/* K线图 Tab */}
            <TabsContent value="chart" className="px-5 pb-5 mt-3">
              {/* Period Selector */}
              <div className="flex gap-1 mb-3">
                {(['1d', '1w', '1M'] as const).map(p => (
                  <button
                    key={p}
                    onClick={() => setPeriod(p)}
                    className={`text-xs px-2.5 py-1 rounded transition-colors ${
                      period === p
                        ? 'bg-primary text-primary-foreground'
                        : 'bg-muted text-muted-foreground hover:text-foreground'
                    }`}
                  >
                    {p === '1d' ? '日线' : p === '1w' ? '周线' : '月线'}
                  </button>
                ))}
              </div>

              {chartData.length > 0 ? (
                <div>
                  {/* Price Chart */}
                  <ResponsiveContainer width="100%" height={240}>
                    <ComposedChart data={chartData} margin={{ top: 5, right: 5, bottom: 5, left: 45 }}>
                      <CartesianGrid strokeDasharray="3 3" stroke="oklch(1 0 0 / 5%)" />
                      <XAxis
                        dataKey="date"
                        tick={{ fontSize: 9, fill: 'oklch(0.6 0.01 256)' }}
                        tickLine={false}
                        interval={Math.floor(chartData.length / 6)}
                      />
                      <YAxis
                        domain={[minPrice, maxPrice]}
                        tick={{ fontSize: 9, fill: 'oklch(0.6 0.01 256)' }}
                        tickLine={false}
                        tickFormatter={(v) => v.toFixed(2)}
                        width={44}
                      />
                      <Tooltip content={<CandleTooltip />} />
                      {/* 用 Bar 模拟蜡烛图（Recharts 原生不支持蜡烛图，用 Bar + 自定义形状） */}
                      <Bar dataKey="close" shape={<CandleBar />}>
                        {chartData.map((entry, index) => (
                          <Cell
                            key={`cell-${index}`}
                            fill={entry.isUp ? '#ef4444' : '#22c55e'}
                          />
                        ))}
                      </Bar>
                    </ComposedChart>
                  </ResponsiveContainer>

                  {/* Volume Chart */}
                  <ResponsiveContainer width="100%" height={70}>
                    <ComposedChart data={chartData} margin={{ top: 2, right: 5, bottom: 0, left: 45 }}>
                      <XAxis dataKey="date" hide />
                      <YAxis
                        tick={{ fontSize: 8, fill: 'oklch(0.6 0.01 256)' }}
                        tickLine={false}
                        tickFormatter={(v) => formatNumber(v, 0)}
                        width={44}
                      />
                      <Bar dataKey="volume" maxBarSize={8}>
                        {chartData.map((entry, index) => (
                          <Cell
                            key={`vol-${index}`}
                            fill={entry.isUp ? 'rgba(239,68,68,0.6)' : 'rgba(34,197,94,0.6)'}
                          />
                        ))}
                      </Bar>
                    </ComposedChart>
                  </ResponsiveContainer>
                </div>
              ) : (
                <div className="h-64 flex items-center justify-center text-muted-foreground text-sm">
                  {loading ? (
                    <div className="flex items-center gap-2">
                      <RefreshCw className="w-4 h-4 animate-spin" />
                      <span>加载K线数据...</span>
                    </div>
                  ) : (
                    <span>暂无K线数据</span>
                  )}
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
          </Tabs>
        </div>
      </DialogContent>
    </Dialog>
  );
}
