import { useEffect, useState } from 'react';

import { useStockClick } from '@/pages/Dashboard';

interface StockRecommend {
  symbol: string;
  name: string;
  price: number;
  change_pct: number;
  score: number;
  level: string;
  reasons: string[];
  risk_level: string;
  target_price: number | null;
  stop_loss: number | null;
}

export default function RecommendPanel() {
  const [recommends, setRecommends] = useState<StockRecommend[]>([]);
  const [loading, setLoading] = useState(true);
  const [lastUpdate, setLastUpdate] = useState('');
  const [error, setError] = useState('');
  const { openStock } = useStockClick();

  const fetchRecommends = async () => {
    try {
      setError('');

      const response = await fetch('/api/recommend');
      const payload = await response.json();

      if (!response.ok || !payload.success) {
        setRecommends([]);
        setError(payload.message || '每日荐股接口返回异常。');
        return;
      }

      const nextData = Array.isArray(payload.data) ? payload.data : [];
      setRecommends(nextData);
      setLastUpdate(new Date().toLocaleTimeString('zh-CN'));

      if (nextData.length === 0) {
        setError('暂无荐股数据，请检查后端行情同步是否正常。');
      }
    } catch (fetchError) {
      console.error('Failed to fetch recommends:', fetchError);
      setRecommends([]);
      setError('每日荐股服务暂时不可用，请稍后重试。');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchRecommends();
    const interval = window.setInterval(fetchRecommends, 60_000);
    return () => window.clearInterval(interval);
  }, []);

  const getLevelColor = (level: string) => {
    switch (level) {
      case '强烈推荐':
        return 'text-red-400 bg-red-500/10 border-red-500/30';
      case '推荐':
        return 'text-yellow-400 bg-yellow-500/10 border-yellow-500/30';
      default:
        return 'text-blue-400 bg-blue-500/10 border-blue-500/30';
    }
  };

  const getRiskColor = (risk: string) => {
    switch (risk) {
      case '低':
        return 'text-green-400';
      case '中':
        return 'text-yellow-400';
      default:
        return 'text-red-400';
    }
  };

  const getScoreColor = (score: number) => {
    if (score >= 70) {
      return 'text-red-400';
    }
    if (score >= 50) {
      return 'text-yellow-400';
    }
    return 'text-blue-400';
  };

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center p-6">
        <div className="h-12 w-12 animate-spin rounded-full border-b-2 border-primary" />
      </div>
    );
  }

  return (
    <div className="space-y-4 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">每日荐股</h2>
          <p className="text-sm text-muted-foreground">
            综合动量、成交活跃度和换手表现生成当日推荐。
          </p>
        </div>

        <div className="text-right">
          <div className="text-sm text-muted-foreground">更新时间: {lastUpdate || '--:--:--'}</div>
          <button
            onClick={fetchRecommends}
            className="mt-1 rounded bg-secondary px-3 py-1 text-xs hover:bg-secondary/80"
            type="button"
          >
            刷新
          </button>
        </div>
      </div>

      {recommends.length === 0 ? (
        <div className="py-12 text-center text-muted-foreground">{error || '暂无推荐股票'}</div>
      ) : (
        <div className="space-y-3">
          {recommends.map((stock, index) => (
            <div
              key={stock.symbol}
              className="cursor-pointer rounded-lg border bg-card p-4 transition-colors hover:border-primary/50"
              onClick={() => openStock(stock.symbol, stock.name)}
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-4">
                  <div
                    className={`flex h-8 w-8 items-center justify-center rounded-full text-lg font-bold ${
                      index === 0
                        ? 'bg-yellow-500/20 text-yellow-400'
                        : index === 1
                          ? 'bg-gray-400/20 text-gray-300'
                          : index === 2
                            ? 'bg-amber-600/20 text-amber-500'
                            : 'bg-secondary text-muted-foreground'
                    }`}
                  >
                    {index + 1}
                  </div>

                  <div>
                    <div className="flex items-center gap-2">
                      <span className="text-lg font-bold">{stock.name}</span>
                      <span className="font-mono text-muted-foreground">{stock.symbol}</span>
                      <span className={`rounded border px-2 py-0.5 text-xs ${getLevelColor(stock.level)}`}>
                        {stock.level}
                      </span>
                    </div>

                    <div className="mt-1 flex items-center gap-4 text-sm">
                      <span className="font-mono">¥{stock.price.toFixed(2)}</span>
                      <span className={stock.change_pct >= 0 ? 'text-red-400' : 'text-green-400'}>
                        {stock.change_pct >= 0 ? '+' : ''}
                        {stock.change_pct.toFixed(2)}%
                      </span>
                      <span className={`font-bold ${getScoreColor(stock.score)}`}>评分: {stock.score.toFixed(1)}</span>
                      <span className={`text-xs ${getRiskColor(stock.risk_level)}`}>风险: {stock.risk_level}</span>
                    </div>
                  </div>
                </div>

                <div className="text-right text-sm">
                  {stock.target_price !== null && (
                    <div>
                      <span className="text-muted-foreground">目标价 </span>
                      <span className="font-mono text-green-400">¥{stock.target_price.toFixed(2)}</span>
                    </div>
                  )}
                  {stock.stop_loss !== null && (
                    <div>
                      <span className="text-muted-foreground">止损价 </span>
                      <span className="font-mono text-red-400">¥{stock.stop_loss.toFixed(2)}</span>
                    </div>
                  )}
                </div>
              </div>

              {stock.reasons.length > 0 && (
                <div className="mt-3 flex flex-wrap gap-2">
                  {stock.reasons.map((reason) => (
                    <span
                      key={`${stock.symbol}-${reason}`}
                      className="rounded bg-secondary/50 px-2 py-1 text-xs text-muted-foreground"
                    >
                      {reason}
                    </span>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      <div className="mt-6 rounded-lg bg-muted/30 p-4">
        <h3 className="mb-2 font-semibold">评分说明</h3>
        <div className="space-y-1 text-sm text-muted-foreground">
          <p>
            <span className="text-red-400">强烈推荐(&gt;=70分)</span>
            ：多项指标同时走强，可重点关注。
          </p>
          <p>
            <span className="text-yellow-400">推荐(50-70分)</span>
            ：存在 1-2 项利好信号，可持续跟踪。
          </p>
          <p>
            <span className="text-blue-400">关注(&lt;50分)</span>
            ：暂未形成强共振，建议结合自有策略判断。
          </p>
          <p className="mt-2 text-xs">风险提示：每日荐股仅供参考，不构成投资建议。</p>
        </div>
      </div>
    </div>
  );
}
