/**
 * ThemeAnalysisPanel - 主题驱动选股面板
 * 基于主题/事件分析，挖掘利好逻辑链和受益板块个股
 */
import { useState } from 'react';
import { Sparkles, TrendingUp, Building2, RefreshCw, ChevronRight, Lightbulb, ArrowRight } from 'lucide-react';
import { useStockClick } from '@/pages/Dashboard';

interface ThemeLogic {
  cause: string;
  effect: string;
  mechanism: string;
}

interface ThemeSector {
  code: string;
  name: string;
  relevance: number;
  change_pct: number;
  money_flow: number;
}

interface ThemeStock {
  symbol: string;
  name: string;
  change_pct: number;
  main_net_inflow: number;
  reason: string;
}

interface ThemeAnalysis {
  theme: string;
  logic: ThemeLogic[];
  sectors: ThemeSector[];
  stocks: ThemeStock[];
  ai_insight: string | null;
}

const PRESET_THEMES = [
  { name: '石油战争', icon: '🛢️', desc: '原油供应紧张、能源替代' },
  { name: '原材料短缺', icon: '⚡', desc: '供应链中断、价格上涨' },
  { name: '新能源', icon: '☀️', desc: '政策支持、碳中和' },
  { name: '军工', icon: '🚀', desc: '地缘风险、装备升级' },
  { name: 'AI', icon: '🤖', desc: '技术突破、算力需求' },
  { name: '芯片战争', icon: '💻', desc: '国产替代、成熟制程' },
  { name: '粮食危机', icon: '🌾', desc: '气候异常、粮价上涨' },
  { name: '医药', icon: '🏥', desc: '老龄化、创新药' },
];

function formatMoney(num: number): string {
  if (Math.abs(num) >= 1e8) {
    return (num / 1e8).toFixed(2) + '亿';
  } else if (Math.abs(num) >= 1e4) {
    return (num / 1e4).toFixed(2) + '万';
  }
  return num.toFixed(0);
}

function getChangeColor(pct: number): string {
  if (pct > 0) return 'text-red-400';
  if (pct < 0) return 'text-green-400';
  return 'text-gray-400';
}

export default function ThemeAnalysisPanel() {
  const [theme, setTheme] = useState('');
  const [inputTheme, setInputTheme] = useState('');
  const [analysis, setAnalysis] = useState<ThemeAnalysis | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [useAI, setUseAI] = useState(false);
  const { openStock } = useStockClick();

  const handleAnalyze = async (themeName: string) => {
    setLoading(true);
    setError('');
    setTheme(themeName);

    try {
      const res = await fetch('/api/theme-analysis', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ theme: themeName, use_ai: useAI }),
      });
      const json = await res.json();

      if (json.success) {
        setAnalysis(json.data);
      } else {
        setError(json.message || '分析失败');
        setAnalysis(null);
      }
    } catch (e) {
      setError('网络错误，请稍后重试');
      setAnalysis(null);
    } finally {
      setLoading(false);
    }
  };

  const handleCustomAnalyze = () => {
    if (inputTheme.trim()) {
      handleAnalyze(inputTheme.trim());
    }
  };

  return (
    <div className="flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border">
        <div className="flex items-center gap-2">
          <Sparkles className="w-4 h-4 text-yellow-400" />
          <h2 className="text-sm font-semibold">主题猎手</h2>
        </div>
        <label className="flex items-center gap-1.5 cursor-pointer">
          <span className="text-[10px] text-muted-foreground">AI分析</span>
          <div
            className={`relative w-8 h-4 rounded-full transition-colors ${useAI ? 'bg-yellow-500' : 'bg-muted'}`}
            onClick={() => setUseAI(!useAI)}
          >
            <div
              className={`absolute top-0.5 w-3 h-3 rounded-full bg-white transition-transform ${useAI ? 'translate-x-4' : 'translate-x-0.5'}`}
            />
          </div>
        </label>
      </div>

      {/* Preset Themes */}
      <div className="px-4 py-3 border-b border-border/50">
        <div className="text-[10px] text-muted-foreground mb-2">快捷主题</div>
        <div className="flex flex-wrap gap-1.5">
          {PRESET_THEMES.map((t) => (
            <button
              key={t.name}
              onClick={() => handleAnalyze(t.name)}
              disabled={loading}
              className={`px-2 py-1 text-xs rounded transition-colors ${
                theme === t.name
                  ? 'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30'
                  : 'bg-muted hover:bg-muted/80 text-foreground'
              }`}
            >
              {t.icon} {t.name}
            </button>
          ))}
        </div>
      </div>

      {/* Custom Input */}
      <div className="px-4 py-3 border-b border-border/50">
        <div className="flex gap-2">
          <input
            type="text"
            value={inputTheme}
            onChange={(e) => setInputTheme(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleCustomAnalyze()}
            placeholder="输入自定义主题..."
            className="flex-1 bg-muted border border-border rounded px-2 py-1 text-xs focus:outline-none focus:border-yellow-500/50"
          />
          <button
            onClick={handleCustomAnalyze}
            disabled={loading || !inputTheme.trim()}
            className="px-3 py-1 bg-yellow-500/20 text-yellow-400 rounded text-xs hover:bg-yellow-500/30 disabled:opacity-50"
          >
            {loading ? <RefreshCw className="w-3 h-3 animate-spin" /> : '分析'}
          </button>
        </div>
      </div>

      {/* Results */}
      {loading && (
        <div className="flex items-center justify-center py-12">
          <RefreshCw className="w-5 h-5 animate-spin text-yellow-400" />
          <span className="ml-2 text-sm text-muted-foreground">分析中...</span>
        </div>
      )}

      {error && (
        <div className="px-4 py-4">
          <div className="bg-red-500/10 border border-red-500/20 rounded p-3 text-xs text-red-400">
            {error}
          </div>
        </div>
      )}

      {analysis && !loading && (
        <div className="overflow-y-auto max-h-[calc(100vh-300px)]">
          {/* Theme Title */}
          <div className="px-4 py-3 bg-yellow-500/5 border-b border-border/50">
            <div className="flex items-center gap-2">
              <Sparkles className="w-4 h-4 text-yellow-400" />
              <span className="text-sm font-semibold text-yellow-400">{analysis.theme}</span>
            </div>
          </div>

          {/* Logic Chain */}
          {analysis.logic.length > 0 && (
            <div className="px-4 py-3 border-b border-border/50">
              <div className="flex items-center gap-1 mb-2">
                <Lightbulb className="w-3.5 h-3.5 text-orange-400" />
                <span className="text-[10px] font-medium text-orange-400">利好逻辑链</span>
              </div>
              <div className="space-y-2">
                {analysis.logic.map((logic, idx) => (
                  <div key={idx} className="bg-muted/50 rounded p-2 text-[10px]">
                    <div className="flex items-center gap-1 text-foreground mb-1">
                      <span className="text-orange-400">因</span>
                      <ArrowRight className="w-2.5 h-2.5 text-muted-foreground" />
                      <span className="text-orange-400">果</span>
                    </div>
                    <div className="text-muted-foreground">
                      <span className="text-red-400">{logic.cause}</span>
                      <span className="mx-1">→</span>
                      <span className="text-green-400">{logic.effect}</span>
                    </div>
                    <div className="text-muted-foreground mt-1 text-[9px]">
                      {logic.mechanism}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Sectors */}
          {analysis.sectors.length > 0 && (
            <div className="px-4 py-3 border-b border-border/50">
              <div className="flex items-center gap-1 mb-2">
                <Building2 className="w-3.5 h-3.5 text-cyan-400" />
                <span className="text-[10px] font-medium text-cyan-400">受益板块</span>
              </div>
              <div className="grid grid-cols-2 gap-1.5">
                {analysis.sectors.map((sector) => (
                  <div
                    key={sector.code}
                    className="bg-muted/50 rounded p-2 flex items-center justify-between"
                  >
                    <div>
                      <div className="text-xs font-medium">{sector.name}</div>
                      <div className="text-[9px] text-muted-foreground">相关度 {sector.relevance * 100}%</div>
                    </div>
                    <div className={`text-xs font-mono ${getChangeColor(sector.change_pct)}`}>
                      {sector.change_pct > 0 ? '+' : ''}{sector.change_pct.toFixed(2)}%
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Stocks */}
          {analysis.stocks.length > 0 && (
            <div className="px-4 py-3">
              <div className="flex items-center gap-1 mb-2">
                <TrendingUp className="w-3.5 h-3.5 text-red-400" />
                <span className="text-[10px] font-medium text-red-400">受益个股（成交活跃且涨幅靠前）</span>
              </div>
              <div className="space-y-1">
                {analysis.stocks.slice(0, 15).map((stock) => (
                  <div
                    key={stock.symbol}
                    onClick={() => openStock(stock.symbol, stock.name)}
                    className="flex items-center justify-between bg-muted/50 hover:bg-muted rounded px-2 py-1.5 cursor-pointer transition-colors"
                  >
                    <div className="flex items-center gap-2">
                      <div>
                        <div className="text-xs font-medium">{stock.name}</div>
                        <div className="text-[9px] text-muted-foreground font-mono">{stock.symbol}</div>
                      </div>
                    </div>
                    <div className="flex items-center gap-3">
                      <div className="text-[10px] text-muted-foreground text-right">
                        <div>净流入</div>
                        <div>{formatMoney(stock.main_net_inflow)}</div>
                      </div>
                      <div className={`text-xs font-mono font-medium ${getChangeColor(stock.change_pct)}`}>
                        {stock.change_pct > 0 ? '+' : ''}{stock.change_pct.toFixed(2)}%
                      </div>
                      <ChevronRight className="w-3 h-3 text-muted-foreground" />
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* AI Insight */}
          {analysis?.ai_insight && (
            <div className="px-4 py-3 border-t border-border/50">
              <div className="flex items-center gap-1 mb-2">
                <Sparkles className="w-3.5 h-3.5 text-yellow-400" />
                <span className="text-[10px] font-medium text-yellow-400">AI 深度分析</span>
              </div>
              <div className="bg-muted/50 rounded p-3 text-xs text-muted-foreground whitespace-pre-wrap leading-relaxed">
                {analysis.ai_insight}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Empty State */}
      {!analysis && !loading && !error && (
        <div className="flex flex-col items-center justify-center py-12 text-center px-4">
          <Sparkles className="w-8 h-8 text-yellow-400/50 mb-2" />
          <div className="text-sm text-muted-foreground">
            选择或输入主题<br />分析利好逻辑和受益板块
          </div>
        </div>
      )}
    </div>
  );
}
