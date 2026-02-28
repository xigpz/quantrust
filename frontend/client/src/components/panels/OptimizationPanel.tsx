/**
 * OptimizationPanel - 参数优化面板
 */
import { useState } from 'react';
import { Settings, Play, TrendingUp, RefreshCw, BarChart2 } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { toast } from 'sonner';

interface OptimizationResult {
  params: { short_period: number; long_period: number };
  total_return: number;
  sharpe_ratio: number;
  max_drawdown: number;
  win_rate: number;
}

export default function OptimizationPanel() {
  const [symbol, setSymbol] = useState('600519');
  const [initialCapital, setInitialCapital] = useState(100000);
  const [results, setResults] = useState<OptimizationResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [hasSearched, setHasSearched] = useState(false);

  const runOptimization = async () => {
    if (!symbol.trim()) return;
    setLoading(true);
    setHasSearched(true);
    try {
      const res = await fetch('/api/backtest/optimize', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          symbol: symbol.toUpperCase(),
          count: 500,
          initial_capital: initialCapital,
        }),
      });
      const data = await res.json();
      if (data.success) {
        setResults(data.data);
        toast.success(`完成 ${data.data.length} 组参数测试`);
      } else {
        toast.error(data.message);
      }
    } catch (e) {
      toast.error('优化失败');
    }
    setLoading(false);
  };

  // 按收益排序 top 5
  const topResults = [...results]
    .sort((a, b) => b.total_return - a.total_return)
    .slice(0, 5);

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-2 border-b border-border">
        <div className="flex items-center gap-2">
          <Settings className="w-4 h-4 text-orange-400" />
          <h2 className="text-sm font-medium">参数优化</h2>
        </div>
        <button onClick={runOptimization} disabled={loading} className="text-muted-foreground hover:text-foreground transition-colors">
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {/* 设置 */}
      <div className="px-4 py-3 border-b border-border space-y-3">
        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="text-xs text-muted-foreground">股票代码</label>
            <input
              type="text"
              value={symbol}
              onChange={(e) => setSymbol(e.target.value.toUpperCase())}
              className="w-full mt-1 bg-background border border-border rounded px-2 py-1.5 text-sm"
              placeholder="600519"
            />
          </div>
          <div>
            <label className="text-xs text-muted-foreground">初始资金</label>
            <input
              type="number"
              value={initialCapital}
              onChange={(e) => setInitialCapital(parseInt(e.target.value))}
              className="w-full mt-1 bg-background border border-border rounded px-2 py-1.5 text-sm"
            />
          </div>
        </div>

        <button
          onClick={runOptimization}
          disabled={loading}
          className="w-full bg-orange-600 hover:bg-orange-700 text-white py-2 px-4 rounded flex items-center justify-center gap-2 disabled:opacity-50"
        >
          <Play className="w-4 h-4" />
          {loading ? '优化中...' : '开始优化'}
        </button>
      </div>

      <ScrollArea className="flex-1">
        {!hasSearched ? (
          <div className="p-4 text-center text-muted-foreground text-sm">
            设置参数后点击"开始优化"
          </div>
        ) : results.length === 0 ? (
          <div className="p-4 text-center text-muted-foreground text-sm">
            {loading ? '优化中...' : '暂无结果'}
          </div>
        ) : (
          <div className="p-4 space-y-4">
            {/* Top 5 结果 */}
            <div className="bg-card rounded-lg p-4 border border-border">
              <div className="flex items-center gap-2 mb-3">
                <TrendingUp className="w-4 h-4 text-green-400" />
                <span className="text-sm font-medium">Top 5 最佳参数</span>
              </div>
              
              <div className="space-y-2">
                {topResults.map((r, idx) => (
                  <div key={idx} className="flex items-center justify-between text-xs bg-gray-800 rounded p-2">
                    <div className="flex items-center gap-2">
                      <span className={`w-5 h-5 rounded-full flex items-center justify-center text-xs ${
                        idx === 0 ? 'bg-yellow-500 text-black' : 
                        idx === 1 ? 'bg-gray-400 text-black' :
                        idx === 2 ? 'bg-amber-600 text-white' : 'bg-gray-600'
                      }`}>
                        {idx + 1}
                      </span>
                      <span className="font-mono">
                        MA({r.params.short_period}, {r.params.long_period})
                      </span>
                    </div>
                    <div className="text-right">
                      <div className={`font-bold ${r.total_return >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                        {r.total_return >= 0 ? '+' : ''}{r.total_return.toFixed(2)}%
                      </div>
                      <div className="text-muted-foreground text-xs">
                        Sharpe: {r.sharpe_ratio.toFixed(2)} | 回撤: {r.max_drawdown.toFixed(2)}%
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* 全部结果表格 */}
            <div className="bg-card rounded-lg p-4 border border-border">
              <div className="flex items-center gap-2 mb-3">
                <BarChart2 className="w-4 h-4 text-blue-400" />
                <span className="text-sm font-medium">全部结果 ({results.length}组)</span>
              </div>
              
              <div className="overflow-x-auto">
                <table className="w-full text-xs">
                  <thead className="border-b border-border">
                    <tr className="text-muted-foreground">
                      <th className="text-left py-2 px-2">短期MA</th>
                      <th className="text-left py-2 px-2">长期MA</th>
                      <th className="text-right py-2 px-2">收益率</th>
                      <th className="text-right py-2 px-2">Sharpe</th>
                      <th className="text-right py-2 px-2">回撤</th>
                      <th className="text-right py-2 px-2">胜率</th>
                    </tr>
                  </thead>
                  <tbody>
                    {results.slice(0, 30).map((r, idx) => (
                      <tr key={idx} className="border-b border-border/50 hover:bg-accent/50">
                        <td className="py-2 px-2 font-mono">{r.params.short_period}</td>
                        <td className="py-2 px-2 font-mono">{r.params.long_period}</td>
                        <td className={`text-right py-2 px-2 font-mono ${r.total_return >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                          {r.total_return >= 0 ? '+' : ''}{r.total_return.toFixed(2)}%
                        </td>
                        <td className="text-right py-2 px-2 font-mono">{r.sharpe_ratio.toFixed(2)}</td>
                        <td className="text-right py-2 px-2 font-mono text-red-400">-{r.max_drawdown.toFixed(2)}%</td>
                        <td className="text-right py-2 px-2 font-mono">{(r.win_rate * 100).toFixed(1)}%</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              
              {results.length > 30 && (
                <div className="text-center text-xs text-muted-foreground mt-2">
                  显示前30条，共{results.length}条
                </div>
              )}
            </div>
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
