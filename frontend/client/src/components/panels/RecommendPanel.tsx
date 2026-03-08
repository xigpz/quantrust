/**
 * 每日推荐股票面板
 * 综合多维度评分：动量、资金流向、技术形态
 */
import { useState, useEffect } from 'react';
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
  const [lastUpdate, setLastUpdate] = useState<string>('');
  const { openStock } = useStockClick();

  const fetchRecommends = async () => {
    try {
      const res = await fetch('/api/recommend');
      const data = await res.json();
      if (data.success) {
        setRecommends(data.data);
        setLastUpdate(new Date().toLocaleTimeString('zh-CN'));
      }
    } catch (e) {
      console.error('Failed to fetch recommends:', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchRecommends();
    const interval = setInterval(fetchRecommends, 60000); // 每分钟刷新
    return () => clearInterval(interval);
  }, []);

  const getLevelColor = (level: string) => {
    switch (level) {
      case '强烈推荐': return 'text-red-400 bg-red-500/10 border-red-500/30';
      case '推荐': return 'text-yellow-400 bg-yellow-500/10 border-yellow-500/30';
      default: return 'text-blue-400 bg-blue-500/10 border-blue-500/30';
    }
  };

  const getRiskColor = (risk: string) => {
    switch (risk) {
      case '低': return 'text-green-400';
      case '中': return 'text-yellow-400';
      default: return 'text-red-400';
    }
  };

  const getScoreColor = (score: number) => {
    if (score >= 70) return 'text-red-400';
    if (score >= 50) return 'text-yellow-400';
    return 'text-blue-400';
  };

  if (loading) {
    return (
      <div className="p-6 flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">每日股票推荐</h2>
          <p className="text-sm text-muted-foreground">
            综合动量、资金流向、技术形态多维度评分 · 每日更新
          </p>
        </div>
        <div className="text-right">
          <div className="text-sm text-muted-foreground">
            更新时间: {lastUpdate}
          </div>
          <button
            onClick={fetchRecommends}
            className="mt-1 px-3 py-1 text-xs bg-secondary hover:bg-secondary/80 rounded"
          >
            刷新
          </button>
        </div>
      </div>

      {recommends.length === 0 ? (
        <div className="text-center py-12 text-muted-foreground">
          暂无推荐股票
        </div>
      ) : (
        <div className="space-y-3">
          {recommends.map((stock, idx) => (
            <div
              key={stock.symbol}
              className="bg-card border rounded-lg p-4 hover:border-primary/50 transition-colors cursor-pointer"
              onClick={() => openStock(stock.symbol, stock.name)}
            >
              <div className="flex items-start justify-between">
                {/* Left: Rank + Stock Info */}
                <div className="flex items-start gap-4">
                  <div className={`
                    w-8 h-8 rounded-full flex items-center justify-center font-bold text-lg
                    ${idx === 0 ? 'bg-yellow-500/20 text-yellow-400' : 
                      idx === 1 ? 'bg-gray-400/20 text-gray-300' : 
                      idx === 2 ? 'bg-amber-600/20 text-amber-500' : 
                      'bg-secondary text-muted-foreground'}
                  `}>
                    {idx + 1}
                  </div>
                  
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-bold text-lg">{stock.name}</span>
                      <span className="text-muted-foreground font-mono">{stock.symbol}</span>
                      <span className={`
                        px-2 py-0.5 text-xs rounded border
                        ${getLevelColor(stock.level)}
                      `}>
                        {stock.level}
                      </span>
                    </div>
                    
                    <div className="flex items-center gap-4 mt-1 text-sm">
                      <span className="font-mono">¥{stock.price.toFixed(2)}</span>
                      <span className={stock.change_pct >= 0 ? 'text-red-400' : 'text-green-400'}>
                        {stock.change_pct >= 0 ? '+' : ''}{stock.change_pct.toFixed(2)}%
                      </span>
                      <span className={`font-bold ${getScoreColor(stock.score)}`}>
                        评分: {stock.score.toFixed(1)}
                      </span>
                      <span className={`text-xs ${getRiskColor(stock.risk_level)}`}>
                        风险: {stock.risk_level}
                      </span>
                    </div>
                  </div>
                </div>

                {/* Right: Target & Stop Loss */}
                <div className="text-right">
                  <div className="text-sm">
                    {stock.target_price && (
                      <div>
                        <span className="text-muted-foreground">目标价: </span>
                        <span className="text-green-400 font-mono">¥{stock.target_price.toFixed(2)}</span>
                      </div>
                    )}
                    {stock.stop_loss && (
                      <div>
                        <span className="text-muted-foreground">止损价: </span>
                        <span className="text-red-400 font-mono">¥{stock.stop_loss.toFixed(2)}</span>
                      </div>
                    )}
                  </div>
                </div>
              </div>

              {/* Reasons */}
              <div className="mt-3 flex flex-wrap gap-2">
                {stock.reasons.map((reason, i) => (
                  <span
                    key={i}
                    className="text-xs px-2 py-1 bg-secondary/50 rounded text-muted-foreground"
                  >
                    {reason}
                  </span>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Tips */}
      <div className="mt-6 p-4 bg-muted/30 rounded-lg">
        <h3 className="font-semibold mb-2">评分说明</h3>
        <div className="text-sm text-muted-foreground space-y-1">
          <p>• <span className="text-red-400">强烈推荐(≥70分)</span>：多项指标同时看涨，可重点关注</p>
          <p>• <span className="text-yellow-400">推荐(50-70分)</span>：有1-2项利好，可适当关注</p>
          <p>• <span className="text-blue-400">关注(50分以下)</span>：需要进一步观察</p>
          <p className="mt-2 text-xs">风险提示：推荐仅供参考，不构成投资建议</p>
        </div>
      </div>
    </div>
  );
}
