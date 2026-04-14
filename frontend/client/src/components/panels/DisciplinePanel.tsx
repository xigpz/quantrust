import { useState } from 'react';
import { useDisciplineRules, checkBuyDiscipline, checkSellDiscipline, useDisciplineChecklist, type DisciplineTradeSignal } from '@/hooks/useMarketData';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Loader2, CheckCircle, XCircle, AlertTriangle } from 'lucide-react';

export default function DisciplinePanel() {
  const { data: rules, loading: rulesLoading } = useDisciplineRules();
  const { data: checklist } = useDisciplineChecklist('buy');
  const [symbol, setSymbol] = useState('');
  const [price, setPrice] = useState('');
  const [changePct, setChangePct] = useState('');
  const [entryPrice, setEntryPrice] = useState('');
  const [result, setResult] = useState<DisciplineTradeSignal | null>(null);
  const [loading, setLoading] = useState(false);
  const [mode, setMode] = useState<'buy' | 'sell'>('buy');

  const handleCheck = async () => {
    if (!symbol || !price || !changePct) return;
    setLoading(true);
    try {
      const params = {
        symbol,
        price: parseFloat(price),
        change_pct: parseFloat(changePct),
        ...(mode === 'sell' ? { entry_price: parseFloat(entryPrice || price) } : {})
      };
      const res = mode === 'sell'
        ? await checkSellDiscipline(params)
        : await checkBuyDiscipline(params);
      if (res.success) {
        setResult(res.data);
      }
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const getActionBadge = (action: string) => {
    switch (action) {
      case 'Buy': return <Badge className="bg-green-500">买入</Badge>;
      case 'Sell': return <Badge className="bg-red-500">卖出</Badge>;
      case 'Hold': return <Badge className="bg-yellow-500">持有</Badge>;
      default: return <Badge className="bg-gray-500">等待</Badge>;
    }
  };

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">交易纪律检查</h2>
      </div>

      {/* 模式切换 */}
      <div className="flex gap-2">
        <Button
          variant={mode === 'buy' ? 'default' : 'outline'}
          onClick={() => { setMode('buy'); setResult(null); }}
        >
          检查买入
        </Button>
        <Button
          variant={mode === 'sell' ? 'default' : 'outline'}
          onClick={() => { setMode('sell'); setResult(null); }}
        >
          检查卖出
        </Button>
      </div>

      {/* 输入表单 */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">输入股票信息</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="text-sm text-muted-foreground">股票代码</label>
              <Input
                placeholder="如: 000001"
                value={symbol}
                onChange={(e) => setSymbol(e.target.value)}
              />
            </div>
            <div>
              <label className="text-sm text-muted-foreground">当前价格</label>
              <Input
                type="number"
                placeholder="如: 10.5"
                value={price}
                onChange={(e) => setPrice(e.target.value)}
              />
            </div>
            <div>
              <label className="text-sm text-muted-foreground">涨跌幅(%)</label>
              <Input
                type="number"
                placeholder="如: 3.0"
                value={changePct}
                onChange={(e) => setChangePct(e.target.value)}
              />
            </div>
            {mode === 'sell' && (
              <div>
                <label className="text-sm text-muted-foreground">买入价格</label>
                <Input
                  type="number"
                  placeholder="如: 9.5"
                  value={entryPrice}
                  onChange={(e) => setEntryPrice(e.target.value)}
                />
              </div>
            )}
          </div>
          <Button onClick={handleCheck} disabled={loading || !symbol || !price} className="w-full">
            {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {mode === 'buy' ? '检查买入合规性' : '检查卖出合规性'}
          </Button>
        </CardContent>
      </Card>

      {/* 纪律规则列表 */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">当前纪律规则</CardTitle>
        </CardHeader>
        <CardContent>
          {rulesLoading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <div className="space-y-2">
              {rules?.map((rule: any, i: number) => (
                <div key={i} className="flex items-center gap-2 text-sm">
                  {rule.NoChaseHigh && (
                    <>
                      <AlertTriangle className="h-4 w-4 text-yellow-500" />
                      <span>不追高: 涨幅不超过 {rule.NoChaseHigh.max_change_pct}%</span>
                    </>
                  )}
                  {rule.TrendFollowing && (
                    <>
                      <AlertTriangle className="h-4 w-4 text-blue-500" />
                      <span>趋势交易: MA5 &gt; MA10 &gt; MA20</span>
                    </>
                  )}
                  {rule.StopLoss && (
                    <>
                      <AlertTriangle className="h-4 w-4 text-red-500" />
                      <span>止损: {rule.StopLoss.max_loss_pct}%</span>
                    </>
                  )}
                  {rule.TakeProfit && (
                    <>
                      <AlertTriangle className="h-4 w-4 text-green-500" />
                      <span>止盈: +{rule.TakeProfit.min_profit_pct}%</span>
                    </>
                  )}
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* 检查结果 */}
      {result && (
        <Card className={result.action === 'Buy' || result.action === 'Sell' ? 'border-green-500' : 'border-yellow-500'}>
          <CardHeader>
            <CardTitle className="text-lg flex items-center justify-between">
              检查结果
              {getActionBadge(result.action)}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm mb-3">{result.reason}</p>
            <div className="space-y-2">
              {result.rules_checked.map((rule, i) => (
                <div key={i} className="flex items-start gap-2 text-sm border p-2 rounded">
                  {rule.passed ? (
                    <CheckCircle className="h-4 w-4 text-green-500 mt-0.5" />
                  ) : (
                    <XCircle className="h-4 w-4 text-red-500 mt-0.5" />
                  )}
                  <div>
                    <div className="font-medium">{rule.rule_name}</div>
                    <div className="text-muted-foreground text-xs">{rule.message}</div>
                    {rule.details.length > 0 && (
                      <div className="text-muted-foreground text-xs mt-1">
                        {rule.details.map((d, j) => <div key={j}>{d}</div>)}
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* 检查清单 */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">买入检查清单</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-1">
            {checklist?.map((item, i) => (
              <div key={i} className="text-sm">{item}</div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
