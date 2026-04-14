/**
 * StrategyMarketPanel - 策略模板市场
 * 浏览、使用、分享量化策略
 */
import { useState, useEffect } from 'react';
import { useLocation } from 'wouter';
import { 
  Search, 
  Download, 
  Copy, 
  Star, 
  TrendingUp, 
  Code, 
  Filter,
  RefreshCw,
  ChevronRight,
  Play,
  Users,
  BookOpen
} from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { toast } from 'sonner';

interface StrategyTemplate {
  id: string;
  name: string;
  description: string;
  author: string;
  category: string;
  tags: string[];
  stars: number;
  uses: number;
  code: string;
  language: string;
  created_at: string;
  performance?: {
    return_rate: number;
    win_rate: number;
    max_drawdown: number;
  };
}

interface TemplateBacktestKpis {
  total_return: number;
  win_rate: number;
  max_drawdown: number;
  total_trades: number;
}

interface TemplateBacktestResult {
  kpis: TemplateBacktestKpis;
}

// 内置策略模板
const BUILT_IN_TEMPLATES: StrategyTemplate[] = [
  {
    id: 'ma-cross',
    name: '双均线交叉策略',
    description: '经典的MA5/MA20金叉买入，死叉卖出策略',
    author: 'QuantRust',
    category: '趋势跟踪',
    tags: ['均线', '趋势', '入门'],
    stars: 128,
    uses: 2341,
    language: 'python',
    created_at: '2026-01-15',
    performance: { return_rate: 12.5, win_rate: 55.2, max_drawdown: -8.3 },
    code: `def init(context):
    context.fast_ma = 5
    context.slow_ma = 20
    context.symbols = ['000001']

def handle_bar(context, data):
    for symbol in context.symbols:
        prices = data.history(symbol, 'close', 30, '1d')
        if len(prices) < 30: continue
        
        ma_fast = prices.iloc[-context.fast_ma:].mean()
        ma_slow = prices.iloc[-context.slow_ma:].mean()
        ma_prev = prices.iloc[-context.fast_ma-1:-1].mean()
        
        # 金叉买入
        if ma_prev < ma_slow and ma_fast > ma_slow:
            order_target_percent(symbol, 0.8)
        # 死叉卖出
        elif ma_prev > ma_slow and ma_fast < ma_slow:
            order_target_percent(symbol, 0)
`
  },
  {
    id: 'rsi-reversal',
    name: 'RSI均值回归策略',
    description: 'RSI超卖买入，超买卖出，适合震荡市场',
    author: 'QuantRust',
    category: '均值回归',
    tags: ['RSI', '震荡', '短线'],
    stars: 89,
    uses: 1567,
    language: 'python',
    created_at: '2026-01-20',
    performance: { return_rate: 8.3, win_rate: 62.1, max_drawdown: -6.5 },
    code: `def init(context):
    context.rsi_period = 14
    context.oversold = 30
    context.overbought = 70
    context.symbols = ['000001']

def calculate_rsi(prices, period):
    delta = prices.diff()
    gain = delta.where(delta > 0, 0).rolling(period).mean()
    loss = (-delta.where(delta < 0, 0)).rolling(period).mean()
    rs = gain / loss
    return 100 - (100 / (1 + rs))

def handle_bar(context, data):
    for symbol in context.symbols:
        prices = data.history(symbol, 'close', 30, '1d')
        if len(prices) < 30: continue
        
        rsi = calculate_rsi(prices, context.rsi_period)
        
        if rsi < context.oversold:
            order_target_percent(symbol, 0.6)
        elif rsi > context.overbought:
            order_target_percent(symbol, 0)
`
  },
  {
    id: 'macd-trend',
    name: 'MACD趋势策略',
    description: 'MACD金叉买入，死叉卖出，配合成交量过滤',
    author: 'QuantRust',
    category: '趋势跟踪',
    tags: ['MACD', '趋势', '中线'],
    stars: 156,
    uses: 2890,
    language: 'python',
    created_at: '2026-01-10',
    performance: { return_rate: 15.8, win_rate: 52.3, max_drawdown: -10.2 },
    code: `def init(context):
    context.fast = 12
    context.slow = 26
    context.signal = 9
    context.symbols = ['000001']

def calculate_ema(prices, period):
    return prices.ewm(span=period, adjust=False).mean()

def handle_bar(context, data):
    for symbol in context.symbols:
        prices = data.history(symbol, ['close', 'volume'], 60, '1d')
        if len(prices) < 60: continue
        
        ema_fast = calculate_ema(prices['close'], context.fast)
        ema_slow = calculate_ema(prices['close'], context.slow)
        macd = ema_fast - ema_slow
        signal = calculate_ema(macd, context.signal)
        
        # 成交量过滤
        vol_ma = prices['volume'].rolling(20).mean()
        
        if macd > signal and macd.iloc[-1] < signal.iloc[-2] and prices['volume'].iloc[-1] > vol_ma.iloc[-1]:
            order_target_percent(symbol, 0.8)
        elif macd < signal and macd.iloc[-1] > signal.iloc[-2]:
            order_target_percent(symbol, 0)
`
  },
  {
    id: 'breakout-momentum',
    name: '突破动量策略',
    description: '20日新高突破买入，配合动量过滤',
    author: 'QuantRust',
    category: '动量策略',
    tags: ['突破', '动量', '中线'],
    stars: 203,
    uses: 3456,
    language: 'python',
    created_at: '2026-01-05',
    performance: { return_rate: 22.1, win_rate: 48.5, max_drawdown: -12.8 },
    code: `def init(context):
    context.lookback = 20
    context.symbols = get_index_stocks('000300')

def handle_bar(context, data):
    for symbol in context.symbols[:10]:  # 限制数量
        prices = data.history(symbol, 'close', 30, '1d')
        if len(prices) < 30: continue
        
        # 20日新高
        high_20 = prices.iloc[-context.lookback:].max()
        
        # 动量指标
        momentum = (prices.iloc[-1] - prices.iloc[-10]) / prices.iloc[-10]
        
        if prices.iloc[-1] >= high_20 and momentum > 0.05:
            order_target_percent(symbol, 0.1)
`
  },
  {
    id: 'value-screening',
    name: '价值股筛选策略',
    description: '基于PE、PB筛选低估价值股',
    author: 'QuantRust',
    category: '价值投资',
    tags: ['价值', 'PE', 'PB', '长线'],
    stars: 67,
    uses: 890,
    language: 'python',
    created_at: '2026-01-25',
    performance: { return_rate: 9.2, win_rate: 58.0, max_drawdown: -7.1 },
    code: `def init(context):
    context.max_pe = 20
    context.max_pb = 2.0
    context.roe_threshold = 10

def handle_bar(context, data):
    # 获取所有A股
    all_stocks = get_all_securities()['code'].tolist()
    
    candidates = []
    for symbol in all_stocks[:500]:  # 限制数量
        fundamentals = get_fundamentals(symbol, 'pe_ratio', 'pb_ratio', 'roe')
        if fundamentals.empty: continue
        
        pe = fundamentals['pe_ratio'].iloc[-1]
        pb = fundamentals['pb_ratio'].iloc[-1]
        roe = fundamentals['roe'].iloc[-1]
        
        if pe < context.max_pe and pb < context.max_pb and roe > context.roe_threshold:
            candidates.append(symbol)
    
    # 等权配置
    for symbol in candidates[:20]:
        order_target_percent(symbol, 1.0 / len(candidates))
`
  },
  {
    id: 'grid-trading',
    name: '网格交易策略',
    description: '在震荡行情中网格买卖，适合横盘整理',
    author: 'QuantRust',
    category: '震荡策略',
    tags: ['网格', '震荡', '自动化'],
    stars: 112,
    uses: 1876,
    language: 'python',
    created_at: '2026-01-18',
    performance: { return_rate: 6.8, win_rate: 72.3, max_drawdown: -4.2 },
    code: `def init(context):
    context.grid_count = 10
    context.grid_pct = 0.02  # 2%网格
    context.base_price = None
    context.symbol = '000001'

def handle_bar(context, data):
    price = data.current(context.symbol, 'close')
    
    if context.base_price is None:
        context.base_price = price
    
    grid_level = round((price - context.base_price) / (context.base_price * context.grid_pct))
    
    # 在每个网格价位下单
    for i in range(-context.grid_count, context.grid_count + 1):
        target_price = context.base_price * (1 + i * context.grid_pct)
        target_level = round((target_price - context.base_price) / (context.base_price * context.grid_pct))
        
        # 下跌买入
        if grid_level <= target_level - 1:
            order(context.symbol, 100)
        # 上涨卖出
        elif grid_level >= target_level + 1:
            order_target_percent(context.symbol, 0)
`
  },
];

const CATEGORIES = ['全部', '趋势跟踪', '均值回归', '动量策略', '价值投资', '震荡策略', '高频策略'];

function formatPercent(value: number): string {
  return (value >= 0 ? '+' : '') + value.toFixed(2) + '%';
}

export default function StrategyMarketPanel() {
  const [, navigate] = useLocation();
  const [templates, setTemplates] = useState<StrategyTemplate[]>(BUILT_IN_TEMPLATES);
  const [selectedCategory, setSelectedCategory] = useState('全部');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTemplate, setSelectedTemplate] = useState<StrategyTemplate | null>(null);
  const [loading, setLoading] = useState(false);
  const [runningBacktest, setRunningBacktest] = useState(false);
  const [backtestResult, setBacktestResult] = useState<TemplateBacktestResult | null>(null);
  const [backtestError, setBacktestError] = useState<string | null>(null);

  const filteredTemplates = templates.filter(t => {
    const matchCategory = selectedCategory === '全部' || t.category === selectedCategory;
    const matchSearch = t.name.includes(searchQuery) || t.description.includes(searchQuery) || t.tags.some(tag => tag.includes(searchQuery));
    return matchCategory && matchSearch;
  });

  const handleUseTemplate = (template: StrategyTemplate) => {
    localStorage.setItem(
      'quantrust_strategy_ide_draft',
      JSON.stringify({
        name: template.name,
        description: template.description,
        code: template.code,
        language: template.language || 'python',
        source: 'strategy-market',
        templateId: template.id,
        injectedAt: Date.now(),
      }),
    );
    toast.success(`已加载策略: ${template.name}`, {
      description: '正在跳转策略IDE...',
    });
    navigate('/strategy');
  };

  const handleCopyCode = (code: string) => {
    navigator.clipboard.writeText(code);
    toast.success('代码已复制到剪贴板');
  };

  const getBacktestParamsByTemplate = (template: StrategyTemplate) => {
    switch (template.id) {
      case 'ma-cross':
        return { symbol: '600519.SH', short_ma: 5, long_ma: 20 };
      case 'rsi-reversal':
        return { symbol: '000001.SZ', short_ma: 6, long_ma: 24 };
      case 'macd-trend':
        return { symbol: '300750.SZ', short_ma: 8, long_ma: 26 };
      case 'breakout-momentum':
        return { symbol: '000300.SH', short_ma: 10, long_ma: 30 };
      case 'value-screening':
        return { symbol: '601318.SH', short_ma: 12, long_ma: 40 };
      case 'grid-trading':
        return { symbol: '000001.SZ', short_ma: 4, long_ma: 16 };
      default:
        return { symbol: '600519.SH', short_ma: 5, long_ma: 20 };
    }
  };

  const runTemplateBacktest = async (template: StrategyTemplate) => {
    setRunningBacktest(true);
    setBacktestError(null);
    setBacktestResult(null);
    try {
      const p = getBacktestParamsByTemplate(template);
      const res = await fetch('/api/backtest/code', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          code: template.code,
          symbol: p.symbol,
          period: '1d',
          count: 500,
          initial_capital: 100000,
          commission_rate: 0.0003,
        }),
      });
      const json = await res.json();
      if (json.success && json.data) {
        setBacktestResult(json.data as TemplateBacktestResult);
        toast.success('模板回测完成');
      } else {
        setBacktestError(json.message || '回测失败');
      }
    } catch (e) {
      setBacktestError(e instanceof Error ? e.message : '网络错误');
    } finally {
      setRunningBacktest(false);
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border">
        <div className="flex items-center gap-2">
          <BookOpen className="w-4 h-4 text-primary" />
          <h2 className="text-sm font-semibold">策略模板市场</h2>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setLoading(true)}
            className="text-muted-foreground hover:text-foreground transition-colors"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {/* 搜索和分类 */}
      <div className="px-4 py-3 border-b border-border space-y-3">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="搜索策略名称、描述、标签..."
            className="w-full pl-9 pr-4 py-2 text-sm bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-primary"
          />
        </div>
        <div className="flex gap-1.5 flex-wrap">
          {CATEGORIES.map(cat => (
            <button
              key={cat}
              onClick={() => setSelectedCategory(cat)}
              className={`px-3 py-1 text-xs rounded-full transition-colors ${
                selectedCategory === cat
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted hover:bg-muted/80 text-muted-foreground'
              }`}
            >
              {cat}
            </button>
          ))}
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* 策略列表 */}
        <div className="flex-1 overflow-y-auto">
          {filteredTemplates.length > 0 ? (
            <div className="grid gap-3 p-4">
              {filteredTemplates.map(template => (
                <div
                  key={template.id}
                  onClick={() => setSelectedTemplate(template)}
                  className={`p-4 bg-card border rounded-lg cursor-pointer transition-all hover:shadow-md ${
                    selectedTemplate?.id === template.id ? 'border-primary ring-1 ring-primary' : 'border-border'
                  }`}
                >
                  <div className="flex items-start justify-between mb-2">
                    <div>
                      <h3 className="font-medium text-sm">{template.name}</h3>
                      <p className="text-xs text-muted-foreground mt-1">{template.description}</p>
                    </div>
                    <span className="px-2 py-0.5 text-[10px] bg-muted rounded">{template.category}</span>
                  </div>
                  
                  <div className="flex items-center gap-3 mt-3 text-xs text-muted-foreground">
                    <span className="flex items-center gap-1">
                      <Star className="w-3 h-3 text-yellow-500" />
                      {template.stars}
                    </span>
                    <span className="flex items-center gap-1">
                      <Users className="w-3 h-3" />
                      {template.uses}
                    </span>
                    {template.performance && (
                      <>
                        <span className={template.performance.return_rate >= 0 ? 'text-green-500' : 'text-red-500'}>
                          {formatPercent(template.performance.return_rate)}
                        </span>
                        <span>胜率: {template.performance.win_rate.toFixed(1)}%</span>
                      </>
                    )}
                  </div>
                  
                  <div className="flex gap-1.5 mt-3">
                    {template.tags.map(tag => (
                      <span key={tag} className="px-2 py-0.5 text-[10px] bg-muted/50 rounded">
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              <div className="text-center">
                <Search className="w-8 h-8 mx-auto mb-2 opacity-30" />
                <p className="text-sm">没有找到匹配的策略</p>
              </div>
            </div>
          )}
        </div>

        {/* 策略详情侧边栏 */}
        {selectedTemplate && (
          <div className="w-96 border-l border-border bg-card/50 flex flex-col">
            <div className="p-4 border-b border-border">
              <h3 className="font-medium">{selectedTemplate.name}</h3>
              <p className="text-xs text-muted-foreground mt-1">{selectedTemplate.description}</p>
              
              {selectedTemplate.performance && (
                <div className="grid grid-cols-3 gap-2 mt-4">
                  <div className="bg-background/50 rounded p-2 text-center">
                    <div className="text-[10px] text-muted-foreground">收益率</div>
                    <div className={`text-sm font-semibold ${selectedTemplate.performance.return_rate >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                      {formatPercent(selectedTemplate.performance.return_rate)}
                    </div>
                  </div>
                  <div className="bg-background/50 rounded p-2 text-center">
                    <div className="text-[10px] text-muted-foreground">胜率</div>
                    <div className="text-sm font-semibold">{selectedTemplate.performance.win_rate.toFixed(1)}%</div>
                  </div>
                  <div className="bg-background/50 rounded p-2 text-center">
                    <div className="text-[10px] text-muted-foreground">最大回撤</div>
                    <div className="text-sm font-semibold text-red-500">{selectedTemplate.performance.max_drawdown.toFixed(1)}%</div>
                  </div>
                </div>
              )}
              
              <div className="flex gap-2 mt-4">
                <button
                  onClick={() => handleUseTemplate(selectedTemplate)}
                  className="flex-1 flex items-center justify-center gap-1.5 py-2 text-xs bg-primary text-primary-foreground rounded hover:bg-primary/90"
                >
                  <Play className="w-3 h-3" />
                  使用模板
                </button>
                <button
                  onClick={() => runTemplateBacktest(selectedTemplate)}
                  disabled={runningBacktest}
                  className="flex-1 flex items-center justify-center gap-1.5 py-2 text-xs bg-emerald-600 text-white rounded hover:bg-emerald-500 disabled:opacity-60"
                >
                  <TrendingUp className="w-3 h-3" />
                  {runningBacktest ? '回测中...' : '运行预览'}
                </button>
                <button
                  onClick={() => handleCopyCode(selectedTemplate.code)}
                  className="flex-1 flex items-center justify-center gap-1.5 py-2 text-xs bg-secondary text-secondary-foreground rounded hover:bg-secondary/80"
                >
                  <Copy className="w-3 h-3" />
                  复制代码
                </button>
              </div>
              <p className="mt-2 text-[10px] text-muted-foreground">
                注：当前为 Python 代码真实回测预览（调用 /api/backtest/code）。
              </p>
            </div>
            
            <ScrollArea className="flex-1">
              <div className="p-4">
                <div className="mb-3 rounded border border-border bg-background/40 p-3">
                  <div className="text-[11px] font-medium mb-2">模板回测预览</div>
                  {backtestError ? (
                    <div className="text-[10px] text-red-400">{backtestError}</div>
                  ) : backtestResult ? (
                    <div className="grid grid-cols-2 gap-2 text-[10px]">
                      <div className="rounded bg-card/60 p-2">
                        <div className="text-muted-foreground">总收益率</div>
                        <div className={backtestResult.kpis.total_return >= 0 ? 'text-green-400 font-semibold' : 'text-red-400 font-semibold'}>
                          {formatPercent(backtestResult.kpis.total_return)}
                        </div>
                      </div>
                      <div className="rounded bg-card/60 p-2">
                        <div className="text-muted-foreground">胜率</div>
                        <div className="font-semibold">{backtestResult.kpis.win_rate.toFixed(1)}%</div>
                      </div>
                      <div className="rounded bg-card/60 p-2">
                        <div className="text-muted-foreground">最大回撤</div>
                        <div className="text-red-400 font-semibold">{backtestResult.kpis.max_drawdown.toFixed(2)}%</div>
                      </div>
                      <div className="rounded bg-card/60 p-2">
                        <div className="text-muted-foreground">交易次数</div>
                        <div className="font-semibold">{backtestResult.kpis.total_trades}</div>
                      </div>
                    </div>
                  ) : (
                    <div className="text-[10px] text-muted-foreground">点击“运行预览”查看模板效果。</div>
                  )}
                </div>

                <div className="flex items-center gap-1.5 mb-2">
                  <Code className="w-4 h-4 text-muted-foreground" />
                  <span className="text-xs font-medium">策略代码</span>
                </div>
                <pre className="p-3 bg-background rounded text-[10px] font-mono overflow-x-auto text-muted-foreground">
                  {selectedTemplate.code}
                </pre>
              </div>
            </ScrollArea>
          </div>
        )}
      </div>
    </div>
  );
}
