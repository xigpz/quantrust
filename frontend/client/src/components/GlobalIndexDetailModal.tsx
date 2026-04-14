/**
 * GlobalIndexDetailModal - 全球指数详情弹窗
 */
import { useState, useEffect } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { TrendingUp, TrendingDown, RefreshCw } from 'lucide-react';
import { formatPrice, formatPercent, getChangeColor } from '@/hooks/useMarketData';

const API_BASE = import.meta.env.VITE_API_BASE || '';

interface Candle {
  timestamp: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

interface GlobalIndexDetailModalProps {
  symbol: string | null;
  name: string | null;
  onClose: () => void;
}

export default function GlobalIndexDetailModal({ symbol, name, onClose }: GlobalIndexDetailModalProps) {
  const [candles, setCandles] = useState<Candle[]>([]);
  const [loading, setLoading] = useState(false);
  const [period, setPeriod] = useState<'1d' | '1w' | '1M'>('1d');
  const [quote, setQuote] = useState<any>(null);

  useEffect(() => {
    if (symbol) {
      fetchData();
    }
  }, [symbol, period]);

  const fetchData = async () => {
    if (!symbol) return;
    setLoading(true);

    try {
      // 获取K线数据
      const candleRes = await fetch(`${API_BASE}/api/global/candles/${symbol}?period=${period}&count=120`);
      const candleData = await candleRes.json();
      if (candleData.success && candleData.data) {
        setCandles(candleData.data);
      }

      // 获取实时行情
      const quoteRes = await fetch(`${API_BASE}/api/global/us/${symbol}`);
      const quoteData = await quoteRes.json();
      if (quoteData.success) {
        setQuote(quoteData.data);
      }
    } catch (e) {
      console.error('Failed to fetch global index data:', e);
    } finally {
      setLoading(false);
    }
  };

  const drawChart = () => {
    const container = document.getElementById(`chart-${symbol}`);
    if (!container || candles.length === 0) return;

    // 使用 canvas 绘制简单K线图
    const width = container.clientWidth;
    const height = 300;
    const padding = 40;

    container.innerHTML = '';

    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    container.appendChild(canvas);

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const closes = candles.map(c => c.close);
    const minPrice = Math.min(...closes) * 0.99;
    const maxPrice = Math.max(...closes) * 1.01;
    const priceRange = maxPrice - minPrice;

    const chartWidth = width - padding * 2;
    const chartHeight = height - padding * 2;
    const barWidth = Math.max(2, chartWidth / candles.length - 2);

    // 绘制背景
    ctx.fillStyle = '#1a1a2e';
    ctx.fillRect(0, 0, width, height);

    // 绘制网格
    ctx.strokeStyle = '#2a2a3e';
    ctx.lineWidth = 0.5;
    for (let i = 0; i <= 4; i++) {
      const y = padding + (chartHeight / 4) * i;
      ctx.beginPath();
      ctx.moveTo(padding, y);
      ctx.lineTo(width - padding, y);
      ctx.stroke();
    }

    // 绘制K线
    candles.forEach((candle, i) => {
      const x = padding + (chartWidth / candles.length) * i + barWidth / 2;
      const isUp = candle.close >= candle.open;
      const color = isUp ? '#ef4444' : '#22c55e';

      const openY = padding + chartHeight - ((candle.open - minPrice) / priceRange) * chartHeight;
      const closeY = padding + chartHeight - ((candle.close - minPrice) / priceRange) * chartHeight;
      const highY = padding + chartHeight - ((candle.high - minPrice) / priceRange) * chartHeight;
      const lowY = padding + chartHeight - ((candle.low - minPrice) / priceRange) * chartHeight;

      // 影线
      ctx.strokeStyle = color;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(x, highY);
      ctx.lineTo(x, lowY);
      ctx.stroke();

      // 实体
      const bodyTop = Math.min(openY, closeY);
      const bodyHeight = Math.abs(closeY - openY) || 1;
      ctx.fillStyle = color;
      ctx.fillRect(x - barWidth / 2, bodyTop, barWidth, bodyHeight);
    });

    // 绘制价格标签
    ctx.fillStyle = '#888';
    ctx.font = '10px monospace';
    ctx.textAlign = 'right';
    for (let i = 0; i <= 4; i++) {
      const price = maxPrice - (priceRange / 4) * i;
      const y = padding + (chartHeight / 4) * i + 3;
      ctx.fillText(price.toFixed(2), width - 5, y);
    }

    // 绘制日期标签
    ctx.textAlign = 'center';
    const dateStep = Math.floor(candles.length / 5);
    candles.forEach((candle, i) => {
      if (i % dateStep === 0) {
        const x = padding + (chartWidth / candles.length) * i;
        const dateStr = candle.timestamp.substring(5); // MM-DD
        ctx.fillText(dateStr, x, height - 10);
      }
    });
  };

  useEffect(() => {
    if (candles.length > 0) {
      // 延迟绘制，等待容器尺寸确定
      const timer = setTimeout(drawChart, 100);
      return () => clearTimeout(timer);
    }
  }, [candles, symbol]);

  // 窗口大小变化时重绘
  useEffect(() => {
    if (candles.length > 0) {
      const handleResize = () => drawChart();
      window.addEventListener('resize', handleResize);
      return () => window.removeEventListener('resize', handleResize);
    }
  }, [candles, symbol]);

  if (!symbol) return null;

  const displayName = name || symbol;
  const currentPrice = quote?.price || 0;
  const changePct = quote?.change_pct || 0;
  const change = quote?.change || 0;

  return (
    <Dialog open={!!symbol} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <span>{displayName}</span>
            <span className="text-sm text-muted-foreground font-mono">{symbol}</span>
          </DialogTitle>
        </DialogHeader>

        {/* 价格信息 */}
        <div className="flex items-center gap-4 px-1">
          <span className="text-2xl font-semibold font-mono-data">
            {formatPrice(currentPrice)}
          </span>
          <div className={`flex items-center gap-1 ${getChangeColor(changePct)}`}>
            {changePct > 0 ? <TrendingUp className="w-4 h-4" /> : <TrendingDown className="w-4 h-4" />}
            <span className="font-mono-data text-sm">
              {change > 0 ? '+' : ''}{change.toFixed(2)} ({formatPercent(changePct)})
            </span>
          </div>
        </div>

        {/* 周期选择 */}
        <div className="flex gap-1 mt-2">
          {(['1d', '1w', '1M'] as const).map(p => (
            <button
              key={p}
              onClick={() => setPeriod(p)}
              className={`text-xs px-3 py-1.5 rounded transition-colors ${
                period === p
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted text-muted-foreground hover:text-foreground'
              }`}
            >
              {p === '1d' ? '日线' : p === '1w' ? '周线' : '月线'}
            </button>
          ))}
        </div>

        {/* 图表 */}
        <div className="relative mt-2">
          {loading && (
            <div className="absolute inset-0 flex items-center justify-center bg-background/50 z-10">
              <RefreshCw className="w-6 h-6 animate-spin" />
            </div>
          )}
          <div id={`chart-${symbol}`} className="w-full h-[300px]" />
          {!loading && candles.length === 0 && (
            <div className="absolute inset-0 flex items-center justify-center text-muted-foreground">
              暂无数据
            </div>
          )}
        </div>

        {/* 统计信息 */}
        {candles.length > 0 && (
          <div className="grid grid-cols-4 gap-2 text-xs text-muted-foreground mt-2">
            <div className="bg-muted/30 rounded p-2">
              <div>最高</div>
              <div className="text-foreground font-mono-data">
                {formatPrice(Math.max(...candles.map(c => c.high)))}
              </div>
            </div>
            <div className="bg-muted/30 rounded p-2">
              <div>最低</div>
              <div className="text-foreground font-mono-data">
                {formatPrice(Math.min(...candles.map(c => c.low)))}
              </div>
            </div>
            <div className="bg-muted/30 rounded p-2">
              <div>成交量</div>
              <div className="text-foreground font-mono-data">
                {(candles.reduce((sum, c) => sum + c.volume, 0) / 1e6).toFixed(2)}M
              </div>
            </div>
            <div className="bg-muted/30 rounded p-2">
              <div>数据点数</div>
              <div className="text-foreground font-mono-data">{candles.length}</div>
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
