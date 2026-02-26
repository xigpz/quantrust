/**
 * BacktestPanel - 策略回测面板
 */
import { useState } from 'react';
import { runBacktest, type BacktestResult, formatNumber } from '@/hooks/useMarketData';
import { FlaskConical, Play, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts';

export default function BacktestPanel() {
  const [symbol, setSymbol] = useState('600519.SH');
  const [shortMa, setShortMa] = useState(5);
  const [longMa, setLongMa] = useState(20);
  const [capital, setCapital] = useState(100000);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<BacktestResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleRun = async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await runBacktest({
        symbol,
        short_ma: shortMa,
        long_ma: longMa,
        initial_capital: capital,
        count: 500,
      });
      if (res.success && res.data) {
        setResult(res.data);
      } else {
        setError(res.message || '回测失败');
      }
    } catch (e) {
      setError('网络错误，请确保后端服务已启动');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-4 py-2.5 border-b border-border">
        <FlaskConical className="w-4 h-4 text-emerald-400" />
        <h2 className="text-sm font-semibold">策略回测</h2>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-4 space-y-4">
          {/* Parameters */}
          <div className="bg-card rounded-lg p-4 border border-border/50 space-y-3">
            <h3 className="text-xs font-semibold text-muted-foreground">双均线策略参数</h3>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="text-[10px] text-muted-foreground block mb-1">股票代码</label>
                <Input
                  value={symbol}
                  onChange={(e) => setSymbol(e.target.value)}
                  className="h-8 text-xs font-mono-data bg-background"
                  placeholder="600519.SH"
                />
              </div>
              <div>
                <label className="text-[10px] text-muted-foreground block mb-1">初始资金</label>
                <Input
                  type="number"
                  value={capital}
                  onChange={(e) => setCapital(Number(e.target.value))}
                  className="h-8 text-xs font-mono-data bg-background"
                />
              </div>
              <div>
                <label className="text-[10px] text-muted-foreground block mb-1">短期均线</label>
                <Input
                  type="number"
                  value={shortMa}
                  onChange={(e) => setShortMa(Number(e.target.value))}
                  className="h-8 text-xs font-mono-data bg-background"
                />
              </div>
              <div>
                <label className="text-[10px] text-muted-foreground block mb-1">长期均线</label>
                <Input
                  type="number"
                  value={longMa}
                  onChange={(e) => setLongMa(Number(e.target.value))}
                  className="h-8 text-xs font-mono-data bg-background"
                />
              </div>
            </div>
            <Button onClick={handleRun} disabled={loading} className="w-full h-8 text-xs">
              {loading ? (
                <><Loader2 className="w-3 h-3 mr-1 animate-spin" /> 回测中...</>
              ) : (
                <><Play className="w-3 h-3 mr-1" /> 运行回测</>
              )}
            </Button>
          </div>

          {error && (
            <div className="bg-destructive/10 border border-destructive/30 rounded-lg p-3 text-xs text-destructive">
              {error}
            </div>
          )}

          {/* Results */}
          {result && (
            <>
              {/* KPIs */}
              <div className="grid grid-cols-3 gap-2">
                {[
                  { label: '总收益率', value: `${result.kpis.total_return.toFixed(2)}%`, color: result.kpis.total_return > 0 ? 'text-up' : 'text-down' },
                  { label: '年化收益', value: `${result.kpis.annual_return.toFixed(2)}%`, color: result.kpis.annual_return > 0 ? 'text-up' : 'text-down' },
                  { label: '最大回撤', value: `${result.kpis.max_drawdown.toFixed(2)}%`, color: 'text-warning' },
                  { label: '夏普比率', value: result.kpis.sharpe_ratio.toFixed(2), color: 'text-info' },
                  { label: '胜率', value: `${result.kpis.win_rate.toFixed(1)}%`, color: 'text-foreground' },
                  { label: '盈亏比', value: result.kpis.profit_loss_ratio.toFixed(2), color: 'text-foreground' },
                  { label: '总交易次数', value: String(result.kpis.total_trades), color: 'text-foreground' },
                  { label: '盈利次数', value: String(result.kpis.winning_trades), color: 'text-up' },
                  { label: '亏损次数', value: String(result.kpis.losing_trades), color: 'text-down' },
                ].map((kpi) => (
                  <div key={kpi.label} className="bg-card rounded-lg p-2.5 border border-border/50 text-center">
                    <div className="text-[10px] text-muted-foreground mb-0.5">{kpi.label}</div>
                    <div className={`font-mono-data text-sm font-bold ${kpi.color}`}>{kpi.value}</div>
                  </div>
                ))}
              </div>

              {/* Equity Curve */}
              {result.equity_curve.length > 0 && (
                <div className="bg-card rounded-lg p-4 border border-border/50">
                  <h3 className="text-xs font-semibold text-muted-foreground mb-3">净值曲线</h3>
                  <ResponsiveContainer width="100%" height={250}>
                    <LineChart data={result.equity_curve.filter((_, i) => i % 3 === 0)}>
                      <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.05)" />
                      <XAxis
                        dataKey="timestamp"
                        tick={{ fontSize: 9, fill: '#6b7280' }}
                        tickFormatter={(v) => v.slice(5, 10)}
                      />
                      <YAxis tick={{ fontSize: 9, fill: '#6b7280' }} />
                      <Tooltip
                        contentStyle={{
                          background: '#1a1f2e',
                          border: '1px solid rgba(255,255,255,0.1)',
                          borderRadius: '6px',
                          fontSize: '11px',
                        }}
                      />
                      <Legend wrapperStyle={{ fontSize: '11px' }} />
                      <Line type="monotone" dataKey="equity" stroke="#3b82f6" dot={false} strokeWidth={1.5} name="策略净值" />
                      <Line type="monotone" dataKey="benchmark" stroke="#6b7280" dot={false} strokeWidth={1} strokeDasharray="4 4" name="基准" />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              )}

              {/* Trade History */}
              {result.trades.length > 0 && (
                <div className="bg-card rounded-lg border border-border/50">
                  <h3 className="text-xs font-semibold text-muted-foreground p-3 border-b border-border">
                    交易记录 ({result.trades.length})
                  </h3>
                  <table className="w-full text-xs">
                    <thead>
                      <tr className="text-muted-foreground border-b border-border">
                        <th className="text-left py-1.5 px-3 font-medium">时间</th>
                        <th className="text-center py-1.5 px-2 font-medium">方向</th>
                        <th className="text-right py-1.5 px-2 font-medium">价格</th>
                        <th className="text-right py-1.5 px-2 font-medium">数量</th>
                        <th className="text-right py-1.5 px-3 font-medium">盈亏</th>
                      </tr>
                    </thead>
                    <tbody>
                      {result.trades.map((t, i) => (
                        <tr key={i} className="border-b border-border/30">
                          <td className="py-1 px-3 font-mono-data text-muted-foreground">{t.timestamp}</td>
                          <td className={`text-center py-1 px-2 font-medium ${t.direction === 'BUY' ? 'text-up' : 'text-down'}`}>
                            {t.direction === 'BUY' ? '买入' : '卖出'}
                          </td>
                          <td className="text-right py-1 px-2 font-mono-data">{t.price.toFixed(2)}</td>
                          <td className="text-right py-1 px-2 font-mono-data">{t.quantity}</td>
                          <td className={`text-right py-1 px-3 font-mono-data font-medium ${t.pnl > 0 ? 'text-up' : t.pnl < 0 ? 'text-down' : ''}`}>
                            {t.pnl !== 0 ? formatNumber(t.pnl) : '—'}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </>
          )}

          {!result && !loading && (
            <div className="text-center py-12 text-muted-foreground text-sm">
              <FlaskConical className="w-10 h-10 mx-auto mb-3 opacity-20" />
              <p>配置参数后点击「运行回测」</p>
              <p className="text-xs mt-1">当前支持双均线交叉策略</p>
            </div>
          )}
        </div>
      </ScrollArea>
    </div>
  );
}
