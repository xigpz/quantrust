/**
 * MarketPredictionPanel - 行情预测面板
 * 左右布局：左侧显示指标摘要，右侧显示完整AI分析报告
 */
import { useState, useEffect, useRef } from 'react';
import {
  TrendingUp, TrendingDown, Minus, AlertTriangle, Lightbulb, Target,
  RefreshCw, Loader2, BarChart3, Clock, Activity, Sparkles, ArrowUp, ArrowDown, ArrowRight,
  Database, ChevronDown, ChevronRight, Globe, Newspaper
} from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import {
  useMarketPrediction, usePredictionHistory, generateMarketPrediction,
  fetchPredictionProgress, PredictionProgress
} from '@/hooks/useMarketData';

// 评分环形进度条组件
function ScoreRing({ score, label, color }: { score: number; label: string; color: string }) {
  const radius = 28;
  const circumference = 2 * Math.PI * radius;
  const strokeDashoffset = circumference - (score / 100) * circumference;

  return (
    <div className="flex flex-col items-center">
      <div className="relative w-14 h-14">
        <svg className="w-full h-full -rotate-90" viewBox="0 0 64 64">
          <circle cx="32" cy="32" r={radius} fill="none" stroke="currentColor" strokeWidth="4" className="text-muted/30" />
          <circle cx="32" cy="32" r={radius} fill="none" stroke={color} strokeWidth="4" strokeLinecap="round" strokeDasharray={circumference} strokeDashoffset={strokeDashoffset} className="transition-all duration-700 ease-out" />
        </svg>
        <div className="absolute inset-0 flex items-center justify-center">
          <span className="text-xs font-bold">{score}</span>
        </div>
      </div>
      <span className="text-[10px] text-muted-foreground mt-1 text-center">{label}</span>
    </div>
  );
}

// 概率条组件
function ProbabilityBar({ probability, type }: { probability: number; type: 'bull' | 'bear' | 'base' }) {
  const colors = { bull: 'from-green-500/80 to-green-500', bear: 'from-red-500/80 to-red-500', base: 'from-yellow-500/80 to-yellow-500' };
  const labels = { bull: '多头', bear: '空头', base: '基准' };

  return (
    <div className="space-y-1">
      <div className="flex justify-between text-xs">
        <span className={type === 'bull' ? 'text-green-500' : type === 'bear' ? 'text-red-500' : 'text-yellow-500'}>{labels[type]}</span>
        <span className="font-medium">{probability}%</span>
      </div>
      <div className="h-1.5 bg-muted/50 rounded-full overflow-hidden">
        <div className={`h-full bg-gradient-to-r ${colors[type]} transition-all duration-500 rounded-full`} style={{ width: `${probability}%` }} />
      </div>
    </div>
  );
}

// 仓位指示器组件
function PositionIndicator({ value, label, color }: { value: number; label: string; color: string }) {
  return (
    <div className="flex flex-col items-center p-2 rounded-lg bg-muted/30 border border-border/50">
      <div className="relative w-12 h-12">
        <svg className="w-full h-full -rotate-90" viewBox="0 0 48 48">
          <circle cx="24" cy="24" r="20" fill="none" stroke="currentColor" strokeWidth="3" className="text-muted/30" />
          <circle cx="24" cy="24" r="20" fill="none" stroke={color} strokeWidth="3" strokeLinecap="round" strokeDasharray={2 * Math.PI * 20} strokeDashoffset={2 * Math.PI * 20 * (1 - value / 100)} className="transition-all duration-700" />
        </svg>
        <div className="absolute inset-0 flex items-center justify-center">
          <span className="text-xs font-bold">{value}%</span>
        </div>
      </div>
      <span className="text-[10px] text-muted-foreground mt-1">{label}</span>
    </div>
  );
}

export default function MarketPredictionPanel() {
  const { data: prediction, loading, refetch } = useMarketPrediction();
  const [generating, setGenerating] = useState(false);
  const [progress, setProgress] = useState<PredictionProgress | null>(null);
  const [showRawData, setShowRawData] = useState(false);
  const progressPollingRef = useRef<number | null>(null);

  const handleGenerate = async () => {
    setGenerating(true);
    setProgress(null);
    try {
      progressPollingRef.current = window.setInterval(async () => {
        try {
          const res = await fetchPredictionProgress();
          if (res.success && res.data) {
            setProgress(res.data);
            if (res.data.is_completed || res.data.is_error) {
              if (progressPollingRef.current) {
                clearInterval(progressPollingRef.current);
                progressPollingRef.current = null;
              }
            }
          }
        } catch (e) {
          console.error('Failed to fetch progress:', e);
        }
      }, 1000);

      await generateMarketPrediction();
      await refetch();
    } catch (e) {
      console.error('Failed to generate prediction:', e);
    } finally {
      setGenerating(false);
      if (progressPollingRef.current) {
        clearInterval(progressPollingRef.current);
        progressPollingRef.current = null;
      }
      setProgress(null);
    }
  };

  useEffect(() => {
    return () => {
      if (progressPollingRef.current) clearInterval(progressPollingRef.current);
    };
  }, []);

  if (loading && !prediction) {
    return (
      <div className="flex items-center justify-center h-full">
        <RefreshCw className="w-6 h-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!prediction) {
    return (
      <div className="flex flex-col h-full p-6">
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-2">
            <Sparkles className="w-5 h-5 text-primary" />
            <h2 className="text-sm font-semibold">行情预测</h2>
          </div>
          <button onClick={handleGenerate} disabled={generating} className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 transition-colors shadow-sm">
            {generating ? <><Loader2 className="w-3.5 h-3.5 animate-spin" />生成中...</> : <><Sparkles className="w-3.5 h-3.5" />生成预测</>}
          </button>
        </div>
        {generating && progress ? (
          <div className="flex-1 flex flex-col items-center justify-center gap-8">
            {/* 动态球体动画 */}
            <div className="relative w-32 h-32">
              {/* 外圈脉冲 */}
              <div className="absolute inset-0 rounded-full border-2 border-primary/20 animate-ping" style={{ animationDuration: '2s' }} />
              <div className="absolute inset-2 rounded-full border-2 border-primary/30 animate-ping" style={{ animationDuration: '1.5s', animationDelay: '0.5s' }} />
              {/* 内圈脉冲 */}
              <div className="absolute inset-4 rounded-full bg-primary/10 animate-pulse" />
              {/* 进度环 */}
              <svg className="w-full h-full -rotate-90" viewBox="0 0 128 128">
                <circle cx="64" cy="64" r="56" fill="none" stroke="currentColor" strokeWidth="8" className="text-muted/20" />
                <circle cx="64" cy="64" r="56" fill="none" stroke="url(#progressGradient)" strokeWidth="8" strokeLinecap="round"
                  strokeDasharray={2 * Math.PI * 56} strokeDashoffset={2 * Math.PI * 56 * (1 - progress.progress / 100)}
                  className="transition-all duration-500 ease-out" />
                <defs>
                  <linearGradient id="progressGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stopColor="hsl(var(--primary))" />
                    <stop offset="100%" stopColor="hsl(var(--primary))" stopOpacity="0.5" />
                  </linearGradient>
                </defs>
              </svg>
              {/* 中心图标 */}
              <div className="absolute inset-0 flex items-center justify-center">
                {progress.is_completed ? (
                  <Sparkles className="w-8 h-8 text-primary animate-spin" />
                ) : (
                  <Loader2 className="w-8 h-8 text-primary animate-spin" />
                )}
              </div>
            </div>

            {/* 百分比显示 */}
            <div className="text-center">
              <div className="text-4xl font-bold text-primary">{Math.round(progress.progress)}%</div>
              <div className="text-sm text-muted-foreground mt-1">分析进度</div>
            </div>

            {/* 多阶段进度指示器 */}
            <div className="flex items-center gap-2">
              {['data', 'analysis', 'format', 'save'].map((stage, idx) => {
                const stageProgress = {
                  'data': 25,
                  'analysis': 60,
                  'format': 85,
                  'save': 100
                };
                const stageLabels = {
                  'data': '数据收集',
                  'analysis': 'AI分析',
                  'format': '整理结果',
                  'save': '保存完成'
                };
                const isActive = progress.progress >= stageProgress[stage as keyof typeof stageProgress] * 0.95;
                const isCurrent = progress.progress >= stageProgress[stage as keyof typeof stageProgress] * 0.5 && progress.progress < stageProgress[stage as keyof typeof stageProgress];
                return (
                  <div key={stage} className="flex items-center">
                    <div className={`w-16 h-16 rounded-full flex items-center justify-center transition-all duration-300 ${
                      isActive ? 'bg-primary text-primary-foreground scale-105' :
                      isCurrent ? 'bg-primary/50 text-primary-foreground animate-pulse' :
                      'bg-muted/50 text-muted-foreground'
                    }`}>
                      {isActive && !isCurrent ? (
                        <Sparkles className="w-4 h-4" />
                      ) : isCurrent ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : (
                        <span className="text-xs">{idx + 1}</span>
                      )}
                    </div>
                    <span className={`ml-1 text-[10px] ${isActive ? 'text-foreground' : 'text-muted-foreground'}`}>
                      {stageLabels[stage as keyof typeof stageLabels]}
                    </span>
                    {idx < 3 && (
                      <div className={`w-8 h-0.5 mx-1 ${progress.progress >= stageProgress[stage as keyof typeof stageProgress] ? 'bg-primary' : 'bg-muted/30'}`} />
                    )}
                  </div>
                );
              })}
            </div>

            {/* 当前阶段消息 */}
            <div className="bg-card/80 rounded-lg border border-border/50 px-4 py-2 shadow-sm">
              <div className="text-sm font-medium flex items-center gap-2">
                {progress.is_error ? (
                  <AlertTriangle className="w-4 h-4 text-red-500" />
                ) : progress.is_completed ? (
                  <Sparkles className="w-4 h-4 text-green-500" />
                ) : (
                  <Activity className="w-4 h-4 text-primary animate-pulse" />
                )}
                {progress.message}
              </div>
              {progress.is_error && (
                <div className="text-xs text-red-500 mt-1">{progress.error_message}</div>
              )}
            </div>

            {/* 动画提示文字 */}
            <div className="text-xs text-muted-foreground animate-pulse">
              {progress.is_error ? '处理失败，请重试' : progress.is_completed ? '报告生成完成！' : '正在分析市场数据，请稍候...'}
            </div>
          </div>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center gap-4 text-center">
            <div className="w-16 h-16 rounded-full bg-muted/50 flex items-center justify-center">
              <TrendingUp className="w-8 h-8 text-muted-foreground/50" />
            </div>
            <div className="text-muted-foreground text-sm">
              暂无预测数据<br />
              <span className="text-xs">点击"生成预测"创建AI分析报告</span>
            </div>
          </div>
        )}
      </div>
    );
  }

  const getJudgmentColor = (text: string) => {
    if (text.includes('强')) return 'text-green-500';
    if (text.includes('弱')) return 'text-red-500';
    if (text.includes('震荡')) return 'text-yellow-500';
    return 'text-muted-foreground';
  };

  const getJudgmentBg = (text: string) => {
    if (text.includes('强')) return 'from-green-500/10 to-green-500/5 border-green-500/30';
    if (text.includes('弱')) return 'from-red-500/10 to-red-500/5 border-red-500/30';
    if (text.includes('震荡')) return 'from-yellow-500/10 to-yellow-500/5 border-yellow-500/30';
    return 'from-muted/10 to-muted/5 border-muted/30';
  };

  return (
    <div className="flex flex-col h-full overflow-hidden bg-gradient-to-b from-card to-background">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-border/50 bg-card/80 backdrop-blur-sm shrink-0">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
            <Sparkles className="w-4 h-4 text-primary" />
          </div>
          <div>
            <h2 className="text-sm font-semibold">行情预测</h2>
            <span className="text-[10px] text-muted-foreground">预测日期: {prediction.prediction_date}</span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button onClick={handleGenerate} disabled={generating} className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 transition-all shadow-sm hover:shadow-md">
            {generating ? <><Loader2 className="w-3 h-3 animate-spin" />生成中...</> : <><Sparkles className="w-3 h-3" />重新生成</>}
          </button>
        </div>
      </div>

      {/* 左右布局主体 */}
      <div className="flex-1 flex overflow-hidden">
        {/* 左侧指标区域 */}
        <div className="w-80 shrink-0 overflow-y-auto border-r border-border/30 p-4 space-y-4">
          {/* 核心判断 */}
          <div className={`relative overflow-hidden rounded-xl border bg-gradient-to-br ${getJudgmentBg(prediction.core_judgment)} p-4`}>
            <div className="absolute top-0 right-0 w-24 h-24 bg-gradient-to-bl from-primary/5 to-transparent rounded-bl-full" />
            <div className="flex items-center gap-3">
              <div className={`w-12 h-12 rounded-xl bg-background/80 flex items-center justify-center ${getJudgmentColor(prediction.core_judgment)}`}>
                {prediction.core_judgment.includes('强') ? <TrendingUp className="w-6 h-6" /> : prediction.core_judgment.includes('弱') ? <TrendingDown className="w-6 h-6" /> : <Minus className="w-6 h-6" />}
              </div>
              <div className="flex-1">
                <div className={`text-xl font-bold ${getJudgmentColor(prediction.core_judgment)}`}>{prediction.core_judgment.split('（')[0]}</div>
                <div className="flex items-center gap-2 mt-1">
                  <span className="text-[10px] text-muted-foreground">置信度</span>
                  <div className="flex items-center gap-1">
                    <div className="w-16 h-1 bg-muted/50 rounded-full overflow-hidden">
                      <div className={`h-full ${getJudgmentColor(prediction.core_judgment).replace('text-', 'bg-')} transition-all duration-500`} style={{ width: `${prediction.confidence}%` }} />
                    </div>
                    <span className="text-[10px] font-medium">{prediction.confidence}%</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* 原始数据（发送给DeepSeek的数据） */}
          {prediction.input_data && (
            <div className="bg-card rounded-xl border border-border/50 p-3 shadow-sm">
              <button
                onClick={() => setShowRawData(!showRawData)}
                className="flex items-center gap-2 w-full text-left"
              >
                {showRawData ? <ChevronDown className="w-4 h-4 text-primary" /> : <ChevronRight className="w-4 h-4 text-primary" />}
                <Database className="w-4 h-4 text-primary" />
                <h3 className="text-xs font-medium">原始数据</h3>
                <span className="text-[10px] text-muted-foreground ml-auto">发送给AI的数据</span>
              </button>
              {showRawData && prediction.input_data && (
                <div className="mt-3 space-y-3 text-[10px]">
                  {/* 量能数据 */}
                  <div className="p-2 rounded bg-muted/30">
                    <div className="flex items-center gap-1 mb-2 text-muted-foreground">
                      <Activity className="w-3 h-3" />
                      <span className="font-medium">量能数据</span>
                    </div>
                    <div className="grid grid-cols-2 gap-1">
                      <div>沪市成交: <span className="text-foreground">{prediction.input_data.shanghai_volume}</span></div>
                      <div>深市成交: <span className="text-foreground">{prediction.input_data.shenzhen_volume}</span></div>
                      <div>总成交: <span className="text-foreground">{prediction.input_data.total_volume}</span></div>
                      <div>昨日成交: <span className="text-foreground">{prediction.input_data.yesterday_volume}</span></div>
                      <div>量能变化: <span className="text-foreground">{prediction.input_data.volume_change_pct}</span></div>
                      <div>涨跌家数: <span className="text-foreground">{prediction.input_data.up_count} / {prediction.input_data.down_count}</span></div>
                      <div>涨停: <span className="text-green-500">{prediction.input_data.limit_up_count}</span></div>
                      <div>跌停: <span className="text-red-500">{prediction.input_data.limit_down_count}</span></div>
                    </div>
                  </div>

                  {/* 大盘走势 */}
                  <div className="p-2 rounded bg-muted/30">
                    <div className="flex items-center gap-1 mb-2 text-muted-foreground">
                      <TrendingUp className="w-3 h-3" />
                      <span className="font-medium">大盘走势</span>
                    </div>
                    <div className="grid grid-cols-2 gap-1">
                      <div>上证指数: <span className="text-foreground">{prediction.input_data.sh_index}</span> <span className={prediction.input_data.sh_change_pct.startsWith('-') ? 'text-red-500' : 'text-green-500'}>{prediction.input_data.sh_change_pct}</span></div>
                      <div>深证指数: <span className="text-foreground">{prediction.input_data.sz_index}</span> <span className={prediction.input_data.sz_change_pct.startsWith('-') ? 'text-red-500' : 'text-green-500'}>{prediction.input_data.sz_change_pct}</span></div>
                      <div>创业板: <span className="text-foreground">{prediction.input_data.cyb_index}</span> <span className={prediction.input_data.cyb_change_pct.startsWith('-') ? 'text-red-500' : 'text-green-500'}>{prediction.input_data.cyb_change_pct}</span></div>
                      <div>K线形态: <span className="text-foreground">{prediction.input_data.kline_pattern}</span></div>
                      <div>支撑位: <span className="text-green-500">{prediction.input_data.support_level}</span></div>
                      <div>压力位: <span className="text-red-500">{prediction.input_data.resistance_level}</span></div>
                      <div>MACD: <span className="text-foreground">{prediction.input_data.macd_status}</span></div>
                    </div>
                  </div>

                  {/* 资金流向 */}
                  <div className="p-2 rounded bg-muted/30">
                    <div className="flex items-center gap-1 mb-2 text-muted-foreground">
                      <TrendingUp className="w-3 h-3" />
                      <span className="font-medium">资金流向</span>
                    </div>
                    <div className="space-y-1">
                      <div className="text-green-500">净流入TOP: {prediction.input_data.sector_flow_top3}</div>
                      <div className="text-red-500">净流出TOP: {prediction.input_data.sector_flow_bottom3}</div>
                      <div className="text-muted-foreground">个股净流入: {prediction.input_data.top_inflow_stocks}</div>
                    </div>
                  </div>

                  {/* 全球市场 */}
                  <div className="p-2 rounded bg-muted/30">
                    <div className="flex items-center gap-1 mb-2 text-muted-foreground">
                      <Globe className="w-3 h-3" />
                      <span className="font-medium">全球市场</span>
                    </div>
                    <div className="grid grid-cols-3 gap-1">
                      <div>道指: <span className={prediction.input_data.dow_change.startsWith('-') ? 'text-red-500' : 'text-green-500'}>{prediction.input_data.dow_change}</span></div>
                      <div>纳斯达克: <span className={prediction.input_data.nasdaq_change.startsWith('-') ? 'text-red-500' : 'text-green-500'}>{prediction.input_data.nasdaq_change}</span></div>
                      <div>标普500: <span className={prediction.input_data.sp500_change.startsWith('-') ? 'text-red-500' : 'text-green-500'}>{prediction.input_data.sp500_change}</span></div>
                      <div>日经: <span className={prediction.input_data.nikkei_change.startsWith('-') ? 'text-red-500' : 'text-green-500'}>{prediction.input_data.nikkei_change}</span></div>
                      <div>恒生: <span className={prediction.input_data.hsi_change.startsWith('-') ? 'text-red-500' : 'text-green-500'}>{prediction.input_data.hsi_change}</span></div>
                      <div>韩综: <span className={prediction.input_data.kospi_change.startsWith('-') ? 'text-red-500' : 'text-green-500'}>{prediction.input_data.kospi_change}</span></div>
                    </div>
                    <div className="grid grid-cols-2 gap-1 mt-1">
                      <div>美元指数: <span className="text-foreground">{prediction.input_data.dxy}</span></div>
                      <div>人民币: <span className="text-foreground">{prediction.input_data.cny_rate}</span></div>
                      <div>原油: <span className="text-foreground">{prediction.input_data.oil_price}</span></div>
                      <div>美债收益率: <span className="text-foreground">{prediction.input_data.us_bond_yield}</span></div>
                      <div>VIX: <span className="text-foreground">{prediction.input_data.vix}</span> ({prediction.input_data.vix_status})</div>
                    </div>
                  </div>

                  {/* 新闻与情绪 */}
                  <div className="p-2 rounded bg-muted/30">
                    <div className="flex items-center gap-1 mb-2 text-muted-foreground">
                      <Newspaper className="w-3 h-3" />
                      <span className="font-medium">新闻与情绪</span>
                    </div>
                    <div className="space-y-1">
                      <div>情绪指数: <span className="text-foreground">{prediction.input_data.sentiment_score}</span></div>
                      <div>板块情绪: <span className="text-foreground">{prediction.input_data.hot_sector_sentiment}</span></div>
                      <div>龙虎榜: <span className="text-foreground">{prediction.input_data.lhb_summary}</span></div>
                      {prediction.input_data.news_list.slice(0, 3).map((news, idx) => (
                        <div key={idx} className="text-muted-foreground truncate">
                          <span className={news.category === '利好' ? 'text-green-500' : news.category === '利空' ? 'text-red-500' : 'text-yellow-500'}>[{news.category}]</span> {news.title}
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* 综合评分 */}
          <div className="bg-card rounded-xl border border-border/50 p-3 shadow-sm">
            <div className="flex items-center gap-2 mb-3">
              <BarChart3 className="w-4 h-4 text-blue-500" />
              <h3 className="text-xs font-medium">综合评分</h3>
              <span className="ml-auto text-[10px] text-muted-foreground">总分: <span className="font-bold text-primary">{prediction.total_score}</span></span>
            </div>
            <div className="flex justify-around">
              <ScoreRing score={prediction.score_volume} label="量能" color="#22c55e" />
              <ScoreRing score={prediction.score_tech} label="技术" color="#3b82f6" />
              <ScoreRing score={prediction.score_capital} label="资金" color="#f59e0b" />
              <ScoreRing score={prediction.score_external} label="外部" color="#8b5cf6" />
              <ScoreRing score={prediction.score_news} label="新闻" color="#ec4899" />
              <ScoreRing score={prediction.score_sentiment} label="情绪" color="#06b6d4" />
            </div>
          </div>

          {/* 多空情景 */}
          <div className="bg-card rounded-xl border border-border/50 p-3 shadow-sm">
            <div className="flex items-center gap-2 mb-3">
              <Activity className="w-4 h-4 text-orange-500" />
              <h3 className="text-xs font-medium">多空情景</h3>
            </div>
            <div className="space-y-2">
              <ProbabilityBar probability={prediction.bull_probability} type="bull" />
              <ProbabilityBar probability={prediction.base_probability} type="base" />
              <ProbabilityBar probability={prediction.bear_probability} type="bear" />
            </div>
            <div className="mt-3 p-2 rounded-lg bg-muted/30 text-[10px] space-y-1">
              <div className="flex justify-between">
                <span className="text-muted-foreground">支撑</span>
                <span className="font-medium text-green-500">{prediction.base_support.split('：')[1] || prediction.base_support}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">压力</span>
                <span className="font-medium text-red-500">{prediction.base_resistance.split('：')[1] || prediction.base_resistance}</span>
              </div>
            </div>
          </div>

          {/* 仓位建议 */}
          <div className="bg-card rounded-xl border border-border/50 p-3 shadow-sm">
            <div className="flex items-center gap-2 mb-3">
              <Target className="w-4 h-4 text-cyan-500" />
              <h3 className="text-xs font-medium">仓位建议</h3>
            </div>
            <div className="grid grid-cols-3 gap-2">
              <PositionIndicator value={prediction.position_aggressive} label="激进" color="#ef4444" />
              <PositionIndicator value={prediction.position_steady} label="稳健" color="#f59e0b" />
              <PositionIndicator value={prediction.position_conservative} label="保守" color="#22c55e" />
            </div>
          </div>

          {/* 风险提示 */}
          <div className="bg-card rounded-xl border border-border/50 p-3 shadow-sm">
            <div className="flex items-center gap-2 mb-2">
              <AlertTriangle className="w-4 h-4 text-red-500" />
              <h3 className="text-xs font-medium">风险提示</h3>
            </div>
            <div className="space-y-1.5">
              <div className="flex items-start gap-2 p-1.5 rounded bg-red-500/5 border border-red-500/20">
                <span className="text-[10px]">🔴</span>
                <span className="text-[10px] text-muted-foreground leading-tight">{prediction.risk_high}</span>
              </div>
              <div className="flex items-start gap-2 p-1.5 rounded bg-yellow-500/5 border border-yellow-500/20">
                <span className="text-[10px]">🟡</span>
                <span className="text-[10px] text-muted-foreground leading-tight">{prediction.risk_medium}</span>
              </div>
              <div className="flex items-start gap-2 p-1.5 rounded bg-green-500/5 border border-green-500/20">
                <span className="text-[10px]">🟢</span>
                <span className="text-[10px] text-muted-foreground leading-tight">{prediction.risk_low}</span>
              </div>
            </div>
          </div>

          {/* 操作建议摘要 */}
          <div className="bg-card rounded-xl border border-border/50 p-3 shadow-sm">
            <div className="flex items-center gap-2 mb-2">
              <Lightbulb className="w-4 h-4 text-yellow-500" />
              <h3 className="text-xs font-medium">操作建议</h3>
            </div>
            <div className="space-y-1.5 text-[10px]">
              <div className="flex items-center gap-1.5">
                <ArrowUp className="w-3 h-3 text-green-500 shrink-0" />
                <span className="text-muted-foreground truncate">{prediction.short_term_trader}</span>
              </div>
              <div className="flex items-center gap-1.5">
                <ArrowRight className="w-3 h-3 text-yellow-500 shrink-0" />
                <span className="text-muted-foreground truncate">{prediction.swing_trader}</span>
              </div>
              <div className="flex items-center gap-1.5">
                <Activity className="w-3 h-3 text-blue-500 shrink-0" />
                <span className="text-muted-foreground truncate">{prediction.position_holder}</span>
              </div>
            </div>
          </div>

          {/* 板块机会 */}
          {prediction.sector_opportunities?.length > 0 && (
            <div className="bg-card rounded-xl border border-border/50 p-3 shadow-sm">
              <div className="flex items-center gap-2 mb-2">
                <TrendingUp className="w-4 h-4 text-green-500" />
                <h3 className="text-xs font-medium">关注板块</h3>
              </div>
              <div className="space-y-1.5">
                {prediction.sector_opportunities.slice(0, 3).map((s, idx) => (
                  <div key={idx} className="p-1.5 rounded bg-green-500/5 border border-green-500/10">
                    <div className="text-[10px] font-medium text-green-500">{s.sector}</div>
                    <div className="text-[9px] text-muted-foreground mt-0.5 line-clamp-2">{s.reason}</div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* 板块风险 */}
          {prediction.sector_risks?.length > 0 && (
            <div className="bg-card rounded-xl border border-border/50 p-3 shadow-sm">
              <div className="flex items-center gap-2 mb-2">
                <TrendingDown className="w-4 h-4 text-red-500" />
                <h3 className="text-xs font-medium">回避板块</h3>
              </div>
              <div className="space-y-1.5">
                {prediction.sector_risks.slice(0, 2).map((s, idx) => (
                  <div key={idx} className="p-1.5 rounded bg-red-500/5 border border-red-500/10">
                    <div className="text-[10px] font-medium text-red-500">{s.sector}</div>
                    <div className="text-[9px] text-muted-foreground mt-0.5 line-clamp-2">{s.reason}</div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* 明日观察 */}
          {prediction.observation_indicators?.length > 0 && (
            <div className="bg-card rounded-xl border border-border/50 p-3 shadow-sm">
              <div className="flex items-center gap-2 mb-2">
                <Clock className="w-4 h-4 text-indigo-500" />
                <h3 className="text-xs font-medium">明日重点观察</h3>
              </div>
              <div className="space-y-1">
                {prediction.observation_indicators.slice(0, 4).map((indicator, idx) => (
                  <div key={idx} className="flex items-start gap-1.5 text-[10px]">
                    <span className="w-4 h-4 rounded-full bg-indigo-500/10 text-indigo-500 flex items-center justify-center text-[9px] font-medium shrink-0">{idx + 1}</span>
                    <span className="text-muted-foreground leading-tight">{indicator.replace(/^[\-\*]+/, '').replace(/\*\*/g, '').trim()}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* 底部信息 */}
          <div className="text-center py-2 text-[9px] text-muted-foreground/60">
            生成: {prediction.created_at}
          </div>
        </div>

        {/* 右侧AI报告区域 */}
        <div className="flex-1 overflow-hidden flex flex-col">
          <div className="px-4 py-3 border-b border-border/30 bg-card/50 shrink-0">
            <div className="flex items-center gap-2">
              <Sparkles className="w-4 h-4 text-primary" />
              <h3 className="text-sm font-medium">AI分析报告</h3>
            </div>
          </div>
          <div className="flex-1 overflow-y-auto p-4">
            {prediction.ai_insight && (
              <div className="markdown-content text-sm leading-relaxed">
                <ReactMarkdown remarkPlugins={[remarkGfm]}>
                  {prediction.ai_insight}
                </ReactMarkdown>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
