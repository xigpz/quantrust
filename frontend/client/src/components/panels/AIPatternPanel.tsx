/**
 * AIPatternPanel - AI形态选股面板
 * Design: 暗夜终端风格 - 使用MiniMax AI分析股票形态
 */
import { useState } from 'react';
import { Brain, RefreshCw, Filter, TrendingUp, TrendingDown, Minus, ArrowUp, ArrowDown, Loader2 } from 'lucide-react';
import { useStockClick } from '@/pages/Dashboard';
import { formatPrice, formatPercent } from '@/hooks/useMarketData';

interface PatternResult {
  symbol: string;
  name: string;
  pattern_type: string;
  consolidation_prob: number;
  breakout_direction: string;
  trend: string;
  support_level: number;
  resistance_level: number;
  analysis_text: string;
  confidence: number;
}

interface ScreenParams {
  max_amplitude?: number;
  days?: number;
  min_consolidation_prob?: number;
  trend?: string;
  limit?: number;
}

export default function AIPatternPanel() {
  const [loading, setLoading] = useState(false);
  const [results, setResults] = useState<PatternResult[]>([]);
  const [params, setParams] = useState<ScreenParams>({
    max_amplitude: 25,
    days: 120,
    min_consolidation_prob: 50,
    trend: undefined,
    limit: 50,
  });
  const [analyzedSymbol, setAnalyzedSymbol] = useState<string | null>(null);
  const [analyzeResult, setAnalyzeResult] = useState<PatternResult | null>(null);
  const { openStock } = useStockClick();

  const screenPatterns = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/ai/screen-patterns', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(params),
      });
      const data = await response.json();
      if (data.success) {
        setResults(data.data);
      } else {
        console.error('Screening failed:', data.message);
      }
    } catch (error) {
      console.error('Error screening patterns:', error);
    } finally {
      setLoading(false);
    }
  };

  const analyzeSingleStock = async (symbol: string) => {
    setAnalyzedSymbol(symbol);
    try {
      const response = await fetch('/api/ai/analyze-pattern', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ symbol, days: 120 }),
      });
      const data = await response.json();
      if (data.success) {
        setAnalyzeResult(data.data);
      }
    } catch (error) {
      console.error('Error analyzing stock:', error);
    } finally {
      setAnalyzedSymbol(null);
    }
  };

  const getTrendIcon = (trend: string) => {
    switch (trend) {
      case 'Bullish':
      case 'Strong':
        return <TrendingUp className="w-4 h-4 text-green-600 dark:text-green-400" />;
      case 'Bearish':
      case 'Weak':
        return <TrendingDown className="w-4 h-4 text-red-600 dark:text-red-400" />;
      default:
        return <Minus className="w-4 h-4 text-muted-foreground" />;
    }
  };

  const getTrendColor = (trend: string) => {
    switch (trend) {
      case 'Bullish':
      case 'Strong':
        return 'text-green-600 dark:text-green-400';
      case 'Bearish':
      case 'Weak':
        return 'text-red-600 dark:text-red-400';
      default:
        return 'text-muted-foreground';
    }
  };

  const getBreakoutIcon = (direction: string) => {
    switch (direction) {
      case 'Up':
        return <ArrowUp className="w-3 h-3 text-green-600 dark:text-green-400" />;
      case 'Down':
        return <ArrowDown className="w-3 h-3 text-red-600 dark:text-red-400" />;
      default:
        return <Minus className="w-3 h-3 text-muted-foreground" />;
    }
  };

  const getPatternBadge = (pattern: string) => {
    const colors: Record<string, string> = {
      Consolidation: 'bg-blue-100 dark:bg-blue-500/20 text-blue-700 dark:text-blue-400',
      Breakout: 'bg-purple-100 dark:bg-purple-500/20 text-purple-700 dark:text-purple-400',
      Uptrend: 'bg-green-100 dark:bg-green-500/20 text-green-700 dark:text-green-400',
      Downtrend: 'bg-red-100 dark:bg-red-500/20 text-red-700 dark:text-red-400',
      Volatile: 'bg-yellow-100 dark:bg-yellow-500/20 text-yellow-700 dark:text-yellow-400',
    };
    const labels: Record<string, string> = {
      Consolidation: '横盘',
      Breakout: '突破',
      Uptrend: '上涨',
      Downtrend: '下跌',
      Volatile: '震荡',
      Unknown: '未知',
    };
    const color = colors[pattern] || 'bg-gray-500/20 text-gray-400';
    const label = labels[pattern] || pattern;
    return <span className={`px-2 py-0.5 rounded text-xs ${color}`}>{label}</span>;
  };

  return (
    <div className="flex flex-col h-full">
      {/* 头部筛选条件 */}
      <div className="p-4 border-b border-border">
        <div className="flex items-center gap-2 mb-4">
          <Brain className="w-5 h-5 text-purple-500" />
          <h2 className="text-lg font-medium">AI形态选股</h2>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-5 gap-3">
          <div>
            <label className="block text-xs text-muted-foreground mb-1">振幅阈值(%)</label>
            <input
              type="number"
              value={params.max_amplitude}
              onChange={(e) => setParams({ ...params, max_amplitude: Number(e.target.value) })}
              className="w-full bg-background border border-input rounded px-2 py-1.5 text-sm"
            />
          </div>
          <div>
            <label className="block text-xs text-muted-foreground mb-1">筛选天数</label>
            <input
              type="number"
              value={params.days}
              onChange={(e) => setParams({ ...params, days: Number(e.target.value) })}
              className="w-full bg-background border border-input rounded px-2 py-1.5 text-sm"
            />
          </div>
          <div>
            <label className="block text-xs text-muted-foreground mb-1">最小横盘概率</label>
            <input
              type="number"
              value={params.min_consolidation_prob}
              onChange={(e) => setParams({ ...params, min_consolidation_prob: Number(e.target.value) })}
              className="w-full bg-background border border-input rounded px-2 py-1.5 text-sm"
            />
          </div>
          <div>
            <label className="block text-xs text-muted-foreground mb-1">趋势筛选</label>
            <select
              value={params.trend || ''}
              onChange={(e) => setParams({ ...params, trend: e.target.value || undefined })}
              className="w-full bg-background border border-input rounded px-2 py-1.5 text-sm"
            >
              <option value="">全部</option>
              <option value="bullish">看涨</option>
              <option value="bearish">看跌</option>
              <option value="sideways">震荡</option>
            </select>
          </div>
          <div>
            <label className="block text-xs text-muted-foreground mb-1">返回数量</label>
            <input
              type="number"
              value={params.limit}
              onChange={(e) => setParams({ ...params, limit: Number(e.target.value) })}
              className="w-full bg-background border border-input rounded px-2 py-1.5 text-sm"
            />
          </div>
        </div>

        <button
          onClick={screenPatterns}
          disabled={loading}
          className="mt-4 flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-purple-800 rounded text-white text-sm transition-colors"
        >
          {loading ? <Loader2 className="w-4 h-4 animate-spin" /> : <Filter className="w-4 h-4" />}
          {loading ? '分析中...' : '开始筛选'}
        </button>
      </div>

      {/* 结果列表 */}
      <div className="flex-1 overflow-auto p-4">
        {results.length > 0 ? (
          <div className="space-y-2">
            {results.map((stock) => (
              <div
                key={stock.symbol}
                onClick={() => openStock(stock.symbol, stock.name)}
                className="p-3 bg-card hover:bg-accent border border-border rounded-lg cursor-pointer transition-colors"
              >
                <div className="flex items-start justify-between mb-2">
                  <div>
                    <span className="font-medium">{stock.symbol}</span>
                    <span className="ml-2 text-muted-foreground text-sm">{stock.name}</span>
                  </div>
                  {getPatternBadge(stock.pattern_type)}
                </div>

                <div className="grid grid-cols-4 gap-2 text-xs">
                  <div>
                    <span className="text-muted-foreground">横盘概率</span>
                    <div>{stock.consolidation_prob.toFixed(1)}%</div>
                  </div>
                  <div>
                    <span className="text-muted-foreground">突破方向</span>
                    <div className="flex items-center gap-1">
                      {getBreakoutIcon(stock.breakout_direction)}
                      {stock.breakout_direction === 'Up' ? '向上' : stock.breakout_direction === 'Down' ? '向下' : '不明'}
                    </div>
                  </div>
                  <div>
                    <span className="text-muted-foreground">趋势判断</span>
                    <div className={`flex items-center gap-1 ${getTrendColor(stock.trend)}`}>
                      {getTrendIcon(stock.trend)}
                      {stock.trend === 'Bullish' ? '看涨' : stock.trend === 'Bearish' ? '看跌' : stock.trend === 'Strong' ? '强势' : stock.trend === 'Weak' ? '弱势' : '震荡'}
                    </div>
                  </div>
                  <div>
                    <span className="text-muted-foreground">置信度</span>
                    <div>{(stock.confidence * 100).toFixed(0)}%</div>
                  </div>
                </div>

                <div className="mt-2 pt-2 border-t border-border flex justify-between text-xs">
                  <div>
                    <span className="text-muted-foreground">支撑位: </span>
                    <span className="text-green-600 dark:text-green-400">{formatPrice(stock.support_level)}</span>
                  </div>
                  <div>
                    <span className="text-muted-foreground">压力位: </span>
                    <span className="text-red-600 dark:text-red-400">{formatPrice(stock.resistance_level)}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground">
            <Brain className="w-12 h-12 mb-3 opacity-50" />
            <p>设置筛选条件后点击"开始筛选"</p>
            <p className="text-xs mt-1">使用MiniMax AI分析股票技术形态</p>
          </div>
        )}
      </div>

      {/* 单只股票分析详情 */}
      {analyzeResult && (
        <div className="p-4 border-t border-border bg-accent">
          <h3 className="text-sm font-medium mb-2">
            {analyzeResult.symbol} - {analyzeResult.name} 分析详情
          </h3>
          <p className="text-xs text-muted-foreground">{analyzeResult.analysis_text}</p>
        </div>
      )}
    </div>
  );
}
