/**
 * MomentumPanel - 动量策略分析面板
 */
import { useState } from 'react';
import { useMomentum } from '@/hooks/useMarketData';
import { TrendingUp, TrendingDown, RefreshCw, Search, Zap } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { toast } from 'sonner';

export default function MomentumPanel() {
  const [symbol, setSymbol] = useState('600519');
  const [searchInput, setSearchInput] = useState('600519');
  const { data: momentum, loading, error, refetch } = useMomentum(symbol);

  const handleSearch = () => {
    if (searchInput.trim()) {
      setSymbol(searchInput.trim());
    }
  };

  const getScoreColor = (score: number) => {
    if (score >= 4) return 'text-green-400';
    if (score >= 2) return 'text-yellow-400';
    return 'text-gray-400';
  };

  const getScoreLabel = (score: number) => {
    if (score >= 4) return '强烈买入';
    if (score >= 2) return '温和买入';
    if (score >= 1) return '观望';
    return '观望';
  };

  const getRSIColor = (rsi: number) => {
    if (rsi < 30) return 'text-green-400';
    if (rsi > 70) return 'text-red-400';
    return 'text-yellow-400';
  };

  const getMACDColor = (hist: number) => {
    if (hist > 0) return 'text-green-400';
    if (hist < 0) return 'text-red-400';
    return 'text-gray-400';
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border">
        <div className="flex items-center gap-2">
          <Zap className="w-4 h-4 text-yellow-400" />
          <h2 className="text-sm font-semibold">动量分析</h2>
        </div>
        <button onClick={() => refetch()} className="text-muted-foreground hover:text-foreground transition-colors">
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {/* 搜索框 */}
      <div className="px-4 py-3 border-b border-border">
        <div className="flex gap-2">
          <input
            type="text"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value.toUpperCase())}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="输入股票代码"
            className="flex-1 bg-background border border-border rounded px-3 py-1.5 text-sm focus:outline-none focus:ring-1 focus:ring-primary"
          />
          <button
            onClick={handleSearch}
            className="bg-primary text-primary-foreground px-3 py-1.5 rounded text-sm hover:bg-primary/90"
          >
            <Search className="w-4 h-4" />
          </button>
        </div>
      </div>

      <ScrollArea className="flex-1">
        {error ? (
          <div className="p-4 text-center text-muted-foreground text-sm">
            加载失败: {error}
          </div>
        ) : !momentum ? (
          <div className="p-4 text-center text-muted-foreground text-sm">
            输入股票代码查看动量分析
          </div>
        ) : (
          <div className="p-4 space-y-4">
            {/* 综合评分 */}
            <div className="bg-card rounded-lg p-4 border border-border">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-muted-foreground">综合评分</span>
                <span className={`text-2xl font-bold ${getScoreColor(momentum.score)}`}>
                  {momentum.score}
                </span>
              </div>
              <div className={`text-sm font-medium ${getScoreColor(momentum.score)}`}>
                {getScoreLabel(momentum.score)}
              </div>
            </div>

            {/* RSI 指标 */}
            <div className="bg-card rounded-lg p-4 border border-border">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-muted-foreground">RSI (14)</span>
                <span className={`font-mono text-lg ${getRSIColor(momentum.rsi)}`}>
                  {momentum.rsi.toFixed(1)}
                </span>
              </div>
              <div className="h-2 bg-gray-700 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-gradient-to-r from-green-400 via-yellow-400 to-red-400"
                  style={{ width: `${Math.min(momentum.rsi, 100)}%` }}
                />
              </div>
              <div className="flex justify-between text-xs text-muted-foreground mt-1">
                <span>0 (超卖)</span>
                <span>100 (超买)</span>
              </div>
            </div>

            {/* MACD 指标 */}
            <div className="bg-card rounded-lg p-4 border border-border">
              <div className="text-sm text-muted-foreground mb-3">MACD</div>
              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-xs text-muted-foreground">DIF</span>
                  <span className={`font-mono ${momentum.macd_dif >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                    {momentum.macd_dif.toFixed(4)}
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-xs text-muted-foreground">DEA</span>
                  <span className={`font-mono ${momentum.macd_dea >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                    {momentum.macd_dea.toFixed(4)}
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-xs text-muted-foreground">MACD柱</span>
                  <span className={`font-mono flex items-center gap-1 ${getMACDColor(momentum.macd_hist)}`}>
                    {momentum.macd_hist >= 0 ? <TrendingUp className="w-3 h-3" /> : <TrendingDown className="w-3 h-3" />}
                    {momentum.macd_hist.toFixed(4)}
                  </span>
                </div>
              </div>
            </div>

            {/* 信号原因 */}
            {momentum.reasons && momentum.reasons.length > 0 && (
              <div className="bg-card rounded-lg p-4 border border-border">
                <div className="text-sm text-muted-foreground mb-2">信号原因</div>
                <div className="space-y-1">
                  {momentum.reasons.map((reason, idx) => (
                    <div key={idx} className="text-sm flex items-center gap-2">
                      <span className="w-1.5 h-1.5 rounded-full bg-yellow-400" />
                      {reason}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* 提示 */}
            <div className="text-xs text-muted-foreground text-center">
              动量分析仅供参考，不构成投资建议
            </div>
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
