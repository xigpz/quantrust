/**
 * TimingOptimizerPanel - 交易时机优化面板
 * 指导性优先：现在该买/卖？买什么？买多少？
 */
import { useState, useEffect } from 'react';
import { Clock, TrendingUp, AlertTriangle, Target, CheckCircle } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { API_BASE } from '@/hooks/useMarketData';

const API = import.meta.env.VITE_API_BASE || '';

interface TimingSignal {
  timestamp: string;
  action: 'Buy' | 'Sell' | 'Hold' | 'Watch';
  action_strength: number;
  intraday_window: string;
  intraday_score: number;
  intraday_remaining_minutes: number;
  intraday_next: string;
  annual_window: string;
  annual_nickname: string;
  annual_score: number;
  annual_win_rate: number;
  position_advice: {
    current_position: string;
    max_position: string;
    stop_loss_pct: number;
    take_profit_pct: number;
  };
  sector_recommendation: {
    hot_sectors: string[];
    defensive_sectors: string[];
    sector_reason: string;
    risk_level: string;
    position_advice: string;
  };
  stock_picks: Array<{
    symbol: string;
    name: string;
    reason: string;
    entry_min: number;
    entry_max: number;
    stop_loss_pct: number;
    target_pct: number;
  }>;
  risk_warnings: Array<{
    level: string;
    message: string;
    suggestion: string;
  }>;
  actionable_steps: string[];
}

export default function TimingOptimizerPanel() {
  const [signal, setSignal] = useState<TimingSignal | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchSignal = async () => {
    try {
      const res = await fetch(`${API}/api/timing/signal`);
      const data = await res.json();
      if (data.success) {
        setSignal(data.data);
      }
    } catch (e) {
      console.error('Failed to fetch timing signal', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSignal();
    const interval = setInterval(fetchSignal, 60000); // 每分钟刷新
    return () => clearInterval(interval);
  }, []);

  const getActionColor = (action: string) => {
    switch (action) {
      case 'Buy': return 'text-red-500 bg-red-500/10 border-red-500/30';
      case 'Sell': return 'text-orange-500 bg-orange-500/10 border-orange-500/30';
      case 'Hold': return 'text-yellow-500 bg-yellow-500/10 border-yellow-500/30';
      default: return 'text-gray-500 bg-gray-500/10 border-gray-500/30';
    }
  };

  const getActionText = (action: string) => {
    switch (action) {
      case 'Buy': return '买入 ⭐';
      case 'Sell': return '卖出 ⚠️';
      case 'Hold': return '持有 🔄';
      default: return '观望 👀';
    }
  };

  const getRiskColor = (level: string) => {
    switch (level) {
      case 'Red': return 'text-red-400 bg-red-500/20 border border-red-500/30';
      case 'Orange': return 'text-orange-400 bg-orange-500/20 border border-orange-500/30';
      default: return 'text-yellow-400 bg-yellow-500/20 border border-yellow-500/30';
    }
  };

  const getPositionBar = (position: string) => {
    const pct = position.includes('满仓') ? 100 :
                position.includes('重仓') ? 75 :
                position.includes('半仓') ? 50 :
                position.includes('轻仓') ? 25 : 0;
    return pct;
  };

  if (loading || !signal) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-muted-foreground">加载中...</div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* 核心指令区 */}
      <div className="p-4 border-b border-border">
        <div className={`p-4 rounded-lg border-2 ${getActionColor(signal.action)}`}>
          <div className="flex items-center justify-between">
            <div>
              <div className="text-2xl font-bold">{getActionText(signal.action)}</div>
              <div className="text-sm mt-1 opacity-70">
                信号强度: {signal.action_strength}分
              </div>
            </div>
            <div className="text-right">
              <div className="text-4xl font-bold">{signal.action_strength}</div>
              <div className="text-sm text-muted-foreground">置信度</div>
            </div>
          </div>
        </div>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-4 space-y-4">
          {/* 时段信息 */}
          <div className="grid grid-cols-2 gap-3">
            <div className="bg-card border border-border rounded-lg p-3">
              <div className="flex items-center gap-2 text-sm text-muted-foreground mb-1">
                <Clock className="w-4 h-4" />
                日内时段
              </div>
              <div className="font-semibold text-foreground">{signal.intraday_window === 'MorningPeak' ? '早盘高峰' : signal.intraday_window === 'AfternoonActive' ? '尾盘活跃' : signal.intraday_window}</div>
              <div className="text-xs text-muted-foreground mt-1">
                剩余 {signal.intraday_remaining_minutes} 分钟
              </div>
            </div>
            <div className="bg-card border border-border rounded-lg p-3">
              <div className="flex items-center gap-2 text-sm text-muted-foreground mb-1">
                <TrendingUp className="w-4 h-4" />
                年度窗口
              </div>
              <div className="font-semibold text-foreground">
                {signal.annual_window === 'SpringRally' ? '春季躁动' :
                 signal.annual_window === 'MidYear' ? '中报行情' :
                 signal.annual_window === 'YearEnd' ? '跨年行情' : signal.annual_window}
                {signal.annual_nickname && ` (${signal.annual_nickname})`}
              </div>
              <div className="text-xs text-muted-foreground mt-1">
                胜率 {signal.annual_win_rate}%
              </div>
            </div>
          </div>

          {/* 仓位指导 */}
          <div className="bg-card border border-border rounded-lg p-4">
            <div className="flex items-center gap-2 mb-3">
              <Target className="w-4 h-4 text-primary" />
              <span className="font-semibold">仓位指导</span>
            </div>
            <div className="space-y-2">
              <div className="flex items-center gap-3">
                <span className="text-sm text-muted-foreground w-20">建议仓位</span>
                <div className="flex-1 h-3 bg-muted rounded-full overflow-hidden">
                  <div
                    className="h-full bg-primary rounded-full transition-all"
                    style={{ width: `${getPositionBar(signal.position_advice.current_position)}%` }}
                  />
                </div>
                <span className="text-sm font-medium w-16 text-right">{signal.position_advice.current_position}</span>
              </div>
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>止损: -{signal.position_advice.stop_loss_pct}%</span>
                <span>止盈: +{signal.position_advice.take_profit_pct}%</span>
              </div>
            </div>
          </div>

          {/* 具体推荐 */}
          {signal.stock_picks.length > 0 && (
            <div className="bg-card border border-border rounded-lg p-4">
              <div className="flex items-center gap-2 mb-3">
                <CheckCircle className="w-4 h-4 text-green-500" />
                <span className="font-semibold">具体推荐</span>
              </div>
              <div className="space-y-3">
                {signal.stock_picks.map((pick, idx) => (
                  <div key={idx} className="bg-muted/50 rounded-lg p-3">
                    <div className="flex items-start justify-between">
                      <div>
                        <div className="font-medium">{pick.name}</div>
                        <div className="text-xs text-muted-foreground">{pick.symbol}</div>
                      </div>
                      <div className="text-right">
                        <div className="text-sm font-medium text-primary">
                          {pick.entry_min}-{pick.entry_max}元
                        </div>
                        <div className="text-xs text-muted-foreground">
                          止损 -{pick.stop_loss_pct}% | 目标 +{pick.target_pct}%
                        </div>
                      </div>
                    </div>
                    <div className="mt-2 text-xs text-muted-foreground">{pick.reason}</div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* 板块推荐 */}
          <div className="bg-card border border-border rounded-lg p-4">
            <div className="flex items-center gap-2 mb-3">
              <TrendingUp className="w-4 h-4 text-primary" />
              <span className="font-semibold">板块推荐</span>
            </div>
            {signal.sector_recommendation.hot_sectors.length > 0 && (
              <div className="mb-3">
                <div className="text-xs text-muted-foreground mb-2">热门板块</div>
                <div className="flex flex-wrap gap-2">
                  {signal.sector_recommendation.hot_sectors.map((sector, idx) => (
                    <span key={idx} className="px-2 py-1 bg-red-500/20 text-red-400 rounded text-xs">
                      {sector} ⭐
                    </span>
                  ))}
                </div>
              </div>
            )}
            {signal.sector_recommendation.defensive_sectors.length > 0 && (
              <div>
                <div className="text-xs text-muted-foreground mb-2">防御板块</div>
                <div className="flex flex-wrap gap-2">
                  {signal.sector_recommendation.defensive_sectors.map((sector, idx) => (
                    <span key={idx} className="px-2 py-1 bg-blue-500/20 text-blue-400 rounded text-xs">
                      {sector}
                    </span>
                  ))}
                </div>
              </div>
            )}
            <div className="mt-3 text-xs text-muted-foreground bg-muted/50 rounded p-2">
              {signal.sector_recommendation.sector_reason}
            </div>
          </div>

          {/* 风险提示 */}
          {signal.risk_warnings.length > 0 && (
            <div className="bg-card border border-border rounded-lg p-4">
              <div className="flex items-center gap-2 mb-3">
                <AlertTriangle className="w-4 h-4 text-orange-500" />
                <span className="font-semibold">风险提示</span>
              </div>
              <div className="space-y-2">
                {signal.risk_warnings.map((warning, idx) => (
                  <div key={idx} className={`p-2 rounded ${getRiskColor(warning.level)}`}>
                    <div className="font-medium text-sm">{warning.message}</div>
                    <div className="text-xs opacity-70 mt-1">{warning.suggestion}</div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* 操作步骤 */}
          <div className="bg-card border border-border rounded-lg p-4">
            <div className="flex items-center gap-2 mb-3">
              <CheckCircle className="w-4 h-4 text-primary" />
              <span className="font-semibold">操作步骤</span>
            </div>
            <div className="space-y-2">
              {signal.actionable_steps.map((step, idx) => (
                <div key={idx} className="flex gap-3 text-sm">
                  <span className="text-primary font-medium shrink-0">{idx + 1}.</span>
                  <span className="text-muted-foreground">{step}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
