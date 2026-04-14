/**
 * QuantHunterPanel - 量子猎手
 * 清晰炫酷的实时监控界面
 */
import { useState, useEffect, useCallback, useRef } from 'react';
import { useStockClick } from '@/pages/Dashboard';
import { API_BASE, formatPrice, formatPercent, getChangeColor, ApiResponse } from '@/hooks/useMarketData';
import { RefreshCw, Zap, Activity, Target, Filter, Play, Settings, Cpu, Layers, Sparkles, Flame } from 'lucide-react';

interface HunterSignal {
  symbol: string;
  name: string;
  price: number;
  change_pct: number;
  signal_type: string;
  signal_name: string;
  score: number;
  reasons: string[];
  strength: string;
  timestamp: string;
}

interface HunterConfig {
  enable_momentum: boolean;
  enable_breakout: boolean;
  enable_volume: boolean;
  enable_reversal: boolean;
  min_score: number;
}

// 信号配置 - 更清晰的配色
const signalConfig: Record<string, { icon: string; color: string; bg: string; border: string }> = {
  limit_up: { icon: '🔥', color: '#ef4444', bg: 'bg-red-500/10', border: 'border-red-500/30' },
  momentum: { icon: '⚡', color: '#f59e0b', bg: 'bg-amber-500/10', border: 'border-amber-500/30' },
  volume: { icon: '📈', color: '#22c55e', bg: 'bg-green-500/10', border: 'border-green-500/30' },
  breakout: { icon: '🎯', color: '#3b82f6', bg: 'bg-blue-500/10', border: 'border-blue-500/30' },
  reversal: { icon: '🔄', color: '#a855f7', bg: 'bg-purple-500/10', border: 'border-purple-500/30' },
  watch: { icon: '👁', color: '#6b7280', bg: 'bg-gray-500/10', border: 'border-gray-500/30' },
};

// 股票卡片
function StockCard({ signal, index, onClick, isNew }: {
  signal: HunterSignal;
  index: number;
  onClick: () => void;
  isNew?: boolean;
}) {
  const config = signalConfig[signal.signal_type] || signalConfig.watch;
  const [hovered, setHovered] = useState(false);

  return (
    <div
      className={`
        relative p-3 rounded-xl border cursor-pointer transition-all duration-200
        ${config.bg} ${config.border}
        ${hovered ? 'scale-105 border-white/50 shadow-lg' : 'hover:border-white/30'}
        ${isNew ? 'animate-cardEnter' : ''}
      `}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={onClick}
      style={{
        boxShadow: hovered ? `0 0 20px ${config.color}30` : 'none',
      }}
    >
      {/* 新入场标记 */}
      {isNew && (
        <div className="absolute -top-1 -right-1 w-3 h-3 bg-green-500 rounded-full animate-ping" />
      )}

      {/* 强度指示 */}
      {signal.strength === 'strong' && (
        <div className="absolute top-2 right-2">
          <Flame className="w-3 h-3 text-orange-500" />
        </div>
      )}

      {/* 头部 */}
      <div className="flex items-start justify-between mb-2">
        <div className="flex items-center gap-2 min-w-0">
          <span className="text-lg">{config.icon}</span>
          <div className="min-w-0">
            <div className="font-semibold text-sm text-foreground truncate">{signal.symbol}</div>
            <div className="text-[10px] text-muted-foreground truncate">{signal.name}</div>
          </div>
        </div>
        <div className="text-right shrink-0 ml-2">
          <div className="font-mono font-semibold">{formatPrice(signal.price)}</div>
          <div className={`font-mono text-xs font-medium ${getChangeColor(signal.change_pct)}`}>
            {formatPercent(signal.change_pct)}
          </div>
        </div>
      </div>

      {/* 评分条 */}
      <div className="mb-2">
        <div className="flex items-center justify-between text-[10px] mb-1">
          <span className="text-muted-foreground">评分</span>
          <span className="font-mono font-semibold" style={{ color: config.color }}>
            {signal.score.toFixed(0)}
          </span>
        </div>
        <div className="h-1.5 bg-black/20 rounded-full overflow-hidden">
          <div
            className="h-full rounded-full transition-all"
            style={{
              width: `${signal.score}%`,
              backgroundColor: config.color,
            }}
          />
        </div>
      </div>

      {/* 底部 */}
      <div className="flex items-center justify-between">
        <span
          className="text-[10px] px-1.5 py-0.5 rounded"
          style={{ backgroundColor: `${config.color}20`, color: config.color }}
        >
          {signal.signal_name}
        </span>
        <span className="text-[10px] text-muted-foreground">#{index + 1}</span>
      </div>
    </div>
  );
}

// 统计卡片
function StatCard({ label, value, color, subValue }: {
  label: string;
  value: string | number;
  color: string;
  subValue?: string;
}) {
  return (
    <div className="px-3 py-2 rounded-lg bg-card/50 border border-border/50">
      <div className="text-[10px] text-muted-foreground mb-0.5">{label}</div>
      <div className="text-xl font-bold font-mono" style={{ color }}>{value}</div>
      {subValue && <div className="text-[9px] text-muted-foreground">{subValue}</div>}
    </div>
  );
}

export default function QuantHunterPanel() {
  const { openStock } = useStockClick();
  const [signals, setSignals] = useState<HunterSignal[]>([]);
  const [prevSignals, setPrevSignals] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);
  const [config, setConfig] = useState<HunterConfig>({
    enable_momentum: true,
    enable_breakout: true,
    enable_volume: true,
    enable_reversal: true,
    min_score: 60,
  });
  const [showSettings, setShowSettings] = useState(false);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [scanCount, setScanCount] = useState(0);

  const fetchSignals = useCallback(async () => {
    setLoading(true);
    try {
      const res = await fetch(`${API_BASE}/api/quant-hunter`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
      const json: ApiResponse<HunterSignal[]> = await res.json();
      if (json.success) {
        setPrevSignals(new Set(signals.map(s => s.symbol)));
        setSignals(json.data);
        setLastUpdate(new Date());
        setScanCount(c => c + 1);
      }
    } catch (e) {
      console.error('Failed to fetch:', e);
    } finally {
      setLoading(false);
    }
  }, [config, signals]);

  useEffect(() => {
    fetchSignals();
    if (autoRefresh) {
      const interval = setInterval(fetchSignals, 10000);
      return () => clearInterval(interval);
    }
  }, [fetchSignals, autoRefresh]);

  // 新入场股票
  const newSignalSymbols = new Set(
    signals.filter(s => !prevSignals.has(s.symbol)).map(s => s.symbol)
  );

  const stats = {
    total: signals.length,
    strong: signals.filter(s => s.strength === 'strong').length,
    limitUp: signals.filter(s => s.signal_type === 'limit_up').length,
    avgScore: signals.length > 0
      ? signals.reduce((sum, s) => sum + s.score, 0) / signals.length
      : 0,
  };

  return (
    <div className="h-full flex flex-col bg-background">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b bg-card/50">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center">
            <Zap className="w-5 h-5 text-white" />
          </div>
          <div>
            <h2 className="text-lg font-bold">量化猎手</h2>
            <p className="text-[10px] text-muted-foreground">
              实时监控 · 扫描 #{scanCount}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {/* 统计 */}
          <div className="hidden md:flex items-center gap-2 mr-2">
            <StatCard label="股票" value={stats.total} color="#06b6d4" subValue="只" />
            <StatCard label="强势" value={stats.strong} color="#f59e0b" subValue="只" />
            <StatCard label="均分" value={stats.avgScore.toFixed(0)} color="#22c55e" />
          </div>

          <button
            onClick={() => setShowSettings(!showSettings)}
            className={`p-2 rounded-lg transition-colors ${showSettings ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'}`}
          >
            <Settings className="w-4 h-4" />
          </button>

          <button
            onClick={() => setAutoRefresh(!autoRefresh)}
            className={`p-2 rounded-lg transition-colors ${autoRefresh ? 'bg-green-500 text-white' : 'bg-muted hover:bg-muted/80'}`}
            title={autoRefresh ? '自动刷新中' : '已暂停'}
          >
            <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
          </button>

          <button
            onClick={fetchSignals}
            disabled={loading}
            className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-lg font-medium text-sm hover:bg-primary/90 transition-colors"
          >
            <Play className="w-4 h-4" />
            扫描
          </button>
        </div>
      </div>

      {/* Settings */}
      {showSettings && (
        <div className="px-4 py-3 border-b bg-muted/30">
          <div className="flex items-center flex-wrap gap-4 text-sm">
            <span className="text-muted-foreground">策略:</span>
            {[
              { key: 'enable_momentum', label: '⚡ 动量' },
              { key: 'enable_breakout', label: '🎯 突破' },
              { key: 'enable_volume', label: '📈 量价' },
              { key: 'enable_reversal', label: '🔄 反弹' },
            ].map(({ key, label }) => (
              <label key={key} className="flex items-center gap-1.5 cursor-pointer">
                <input
                  type="checkbox"
                  checked={config[key as keyof HunterConfig] as boolean}
                  onChange={(e) => setConfig({ ...config, [key]: e.target.checked })}
                  className="rounded"
                />
                <span>{label}</span>
              </label>
            ))}
            <div className="flex items-center gap-2 ml-auto">
              <Filter className="w-4 h-4 text-muted-foreground" />
              <span className="text-muted-foreground">最低评分:</span>
              <input
                type="range"
                min="20"
                max="90"
                value={config.min_score}
                onChange={(e) => setConfig({ ...config, min_score: Number(e.target.value) })}
                className="w-24"
              />
              <span className="w-8 text-center font-mono">{config.min_score}</span>
            </div>
          </div>
        </div>
      )}

      {/* 信号类型 */}
      {signals.length > 0 && (
        <div className="px-4 py-2 border-b bg-muted/20">
          <div className="flex items-center gap-2 flex-wrap text-xs">
            <span className="text-muted-foreground">信号:</span>
            {Object.entries(
              signals.reduce((acc, s) => {
                acc[s.signal_type] = (acc[s.signal_type] || 0) + 1;
                return acc;
              }, {} as Record<string, number>)
            ).map(([type, count]) => {
              const cfg = signalConfig[type];
              return (
                <span
                  key={type}
                  className="px-2 py-0.5 rounded text-xs"
                  style={{ backgroundColor: `${cfg?.color}20`, color: cfg?.color }}
                >
                  {cfg?.icon} {count}
                </span>
              );
            })}
          </div>
        </div>
      )}

      {/* 股票网格 */}
      <div className="flex-1 overflow-y-auto p-4">
        {signals.length === 0 && !loading ? (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground">
            <Zap className="w-16 h-16 mb-4 opacity-30" />
            <p>点击"扫描"开始抓股</p>
          </div>
        ) : (
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-8 gap-2">
            {signals.map((signal, idx) => (
              <StockCard
                key={signal.symbol}
                signal={signal}
                index={idx}
                onClick={() => openStock(signal.symbol, signal.name)}
                isNew={newSignalSymbols.has(signal.symbol)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between px-4 py-2 border-t bg-card/50 text-xs text-muted-foreground">
        <div className="flex items-center gap-4">
          <span className="flex items-center gap-1">
            <Layers className="w-3 h-3" />
            深度5级
          </span>
          <span className="flex items-center gap-1">
            <Cpu className="w-3 h-3" />
            AI引擎
          </span>
        </div>
        {lastUpdate && (
          <span>更新: {lastUpdate.toLocaleTimeString()}</span>
        )}
      </div>

      <style>{`
        @keyframes cardEnter {
          0% { opacity: 0; transform: scale(0.8); }
          50% { transform: scale(1.05); }
          100% { opacity: 1; transform: scale(1); }
        }
        .animate-cardEnter {
          animation: cardEnter 0.4s ease-out;
        }
      `}</style>
    </div>
  );
}
