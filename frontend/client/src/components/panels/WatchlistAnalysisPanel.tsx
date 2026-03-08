import { useState, useEffect, useContext } from 'react';
import { Star, TrendingUp, TrendingDown, AlertTriangle, LineChart, Target, Shield, RefreshCw } from 'lucide-react';
import { StockClickContext } from '@/pages/Dashboard';

const API_BASE = '';

// 技术分析结果
interface TechnicalAnalysis {
  symbol: string;
  name: string;
  price: number;
  change: number;
  ma5: number;
  ma10: number;
  ma20: number;
  support: number;
  resistance: number;
  trend: '上涨' | '下跌' | '震荡';
  signal: '买入' | '卖出' | '观望';
  reason: string;
}

export default function WatchlistWithAnalysis() {
  const { openStock } = useContext(StockClickContext);
  const [watchlist, setWatchlist] = useState<string[]>(['000001.SZ', '399001.SZ', '300750.SZ', '600519.SH']);
  const [analysis, setAnalysis] = useState<TechnicalAnalysis[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedStock, setSelectedStock] = useState<TechnicalAnalysis | null>(null);

  // 计算技术指标
  const calculateTechnical = (candles: any[], quote: any): TechnicalAnalysis => {
    if (!candles || candles.length < 20) {
      return {
        symbol: quote.symbol,
        name: quote.name,
        price: quote.price,
        change: quote.change_pct,
        ma5: 0, ma10: 0, ma20: 0,
        support: 0, resistance: 0,
        trend: '震荡', signal: '观望', reason: '数据不足',
      };
    }

    const prices = candles.map(c => c.close);
    const ma5 = prices.slice(-5).reduce((a, b) => a + b, 0) / 5;
    const ma10 = prices.slice(-10).reduce((a, b) => a + b, 0) / 10;
    const ma20 = prices.slice(-20).reduce((a, b) => a + b, 0) / 20;
    
    const currentPrice = quote.price;
    
    // 支撑位和压力位
    const lows = prices.slice(-20).map(c => c.low);
    const highs = prices.slice(-20).map(c => c.high);
    const support = Math.min(...lows.slice(-5));
    const resistance = Math.max(...highs.slice(-5));
    
    // 判断趋势
    let trend: TechnicalAnalysis['trend'];
    if (ma5 > ma10 && ma10 > ma20) trend = '上涨';
    else if (ma5 < ma10 && ma10 < ma20) trend = '下跌';
    else trend = '震荡';
    
    // 买卖信号
    let signal: TechnicalAnalysis['signal'];
    let reason = '';
    
    if (currentPrice < ma5 && currentPrice > ma10 && currentPrice > ma20) {
      // 价格在均线附近，观望
      signal = '观望';
      reason = '价格位于均线之间';
    } else if (currentPrice < ma5 && currentPrice < ma10) {
      signal = '卖出';
      reason = '价格跌破多条均线';
    } else if (currentPrice > ma5 && currentPrice > ma10 && currentPrice > ma20) {
      // 强势上涨
      if (currentPrice - support < (resistance - support) * 0.3) {
        signal = '买入';
        reason = '上涨趋势，回调至支撑位';
      } else {
        signal = '观望';
        reason = '已涨幅较大';
      }
    } else if (currentPrice > ma5 && currentPrice > ma10) {
      signal = '买入';
      reason = '5日、10日均线向上';
    } else if (currentPrice < ma5 && currentPrice < ma10) {
      signal = '卖出';
      reason = '5日、10日均线向下';
    } else {
      signal = '观望';
      reason = '趋势不明';
    }
    
    return {
      symbol: quote.symbol,
      name: quote.name,
      price: currentPrice,
      change: quote.change_pct,
      ma5, ma10, ma20,
      support, resistance,
      trend, signal, reason,
    };
  };

  const fetchAnalysis = async () => {
    setLoading(true);
    try {
      const quotesRes = await fetch(`${API_BASE}/api/quotes`).then(r => r.json());
      if (!quotesRes.success) return;
      
      const quotes = quotesRes.data;
      const results: TechnicalAnalysis[] = [];
      
      for (const symbol of watchlist) {
        const quote = quotes.find((q: any) => q.symbol === symbol);
        if (quote) {
          // 获取日线数据
          try {
            const candlesRes = await fetch(`${API_BASE}/api/candles/${symbol}?period=1d&count=30`).then(r => r.json());
            const analysis = calculateTechnical(candlesRes.success ? candlesRes.data : [], quote);
            results.push(analysis);
          } catch (e) {
            results.push({
              symbol: quote.symbol,
              name: quote.name,
              price: quote.price,
              change: quote.change_pct,
              ma5: 0, ma10: 0, ma20: 0,
              support: 0, resistance: 0,
              trend: '震荡', signal: '观望', reason: '获取数据失败',
            });
          }
        }
      }
      
      setAnalysis(results);
    } catch (e) {
      console.error(e);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchAnalysis();
    const interval = setInterval(fetchAnalysis, 30000);
    return () => clearInterval(interval);
  }, []);

  const getSignalColor = (signal: string) => {
    if (signal === '买入') return 'bg-green-500/20 text-green-400 border-green-500/30';
    if (signal === '卖出') return 'bg-red-500/20 text-red-400 border-red-500/30';
    return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30';
  };

  const getTrendIcon = (trend: string) => {
    if (trend === '上涨') return <TrendingUp className="w-4 h-4 text-green-400" />;
    if (trend === '下跌') return <TrendingDown className="w-4 h-4 text-red-400" />;
    return <TrendingUp className="w-4 h-4 text-gray-400" />;
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold flex items-center gap-2">
          <Star className="w-5 h-5 text-yellow-400" />
          自选股分析
        </h2>
        <button 
          onClick={fetchAnalysis} 
          disabled={loading}
          className="flex items-center gap-1 text-xs px-2 py-1 bg-muted rounded hover:bg-muted/80"
        >
          <RefreshCw className={`w-3 h-3 ${loading ? 'animate-spin' : ''}`} />
          刷新
        </button>
      </div>

      {loading && analysis.length === 0 ? (
        <div className="text-center py-8 text-muted-foreground">加载中...</div>
      ) : (
        <div className="space-y-3">
          {analysis.map((item, i) => (
            <div 
              key={i}
              onClick={() => setSelectedStock(item)}
              className="bg-card rounded-lg p-4 border hover:border-primary/50 cursor-pointer transition-colors"
            >
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                  <span className="font-semibold">{item.symbol}</span>
                  <span className="text-sm text-muted-foreground">{item.name}</span>
                  {getTrendIcon(item.trend)}
                </div>
                <span className={`px-2 py-1 text-xs rounded border ${getSignalColor(item.signal)}`}>
                  {item.signal}
                </span>
              </div>
              
              <div className="grid grid-cols-4 gap-2 text-xs mb-3">
                <div>
                  <div className="text-muted-foreground">现价</div>
                  <div className="font-medium">{item.price.toFixed(2)}</div>
                </div>
                <div>
                  <div className="text-muted-foreground">5日线</div>
                  <div className={item.price > item.ma5 ? 'text-green-400' : 'text-red-400'}>{item.ma5.toFixed(2)}</div>
                </div>
                <div>
                  <div className="text-muted-foreground">10日线</div>
                  <div className={item.price > item.ma10 ? 'text-green-400' : 'text-red-400'}>{item.ma10.toFixed(2)}</div>
                </div>
                <div>
                  <div className="text-muted-foreground">20日线</div>
                  <div className={item.price > item.ma20 ? 'text-green-400' : 'text-red-400'}>{item.ma20.toFixed(2)}</div>
                </div>
              </div>
              
              <div className="flex items-center justify-between text-xs">
                <div className="flex gap-4">
                  <span className="text-muted-foreground">支撑: <span className="text-green-400">{item.support.toFixed(2)}</span></span>
                  <span className="text-muted-foreground">压力: <span className="text-red-400">{item.resistance.toFixed(2)}</span></span>
                </div>
                <span className="text-muted-foreground">{item.reason}</span>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* 详情弹窗 */}
      {selectedStock && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" onClick={() => setSelectedStock(null)}>
          <div className="bg-background rounded-lg p-6 w-full max-w-md" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <div>
                <h3 className="text-xl font-semibold">{selectedStock.symbol}</h3>
                <p className="text-sm text-muted-foreground">{selectedStock.name}</p>
              </div>
              <button onClick={() => openStock(selectedStock.symbol, selectedStock.name)} className="text-xs text-primary hover:underline">
                查看K线
              </button>
            </div>
            
            <div className="space-y-4">
              <div className="grid grid-cols-3 gap-4 text-center">
                <div className="bg-muted rounded p-3">
                  <div className="text-xs text-muted-foreground">现价</div>
                  <div className="text-lg font-bold">{selectedStock.price.toFixed(2)}</div>
                </div>
                <div className="bg-muted rounded p-3">
                  <div className="text-xs text-muted-foreground">涨跌</div>
                  <div className={`text-lg font-bold ${selectedStock.change >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                    {selectedStock.change >= 0 ? '+' : ''}{selectedStock.change.toFixed(2)}%
                  </div>
                </div>
                <div className="bg-muted rounded p-3">
                  <div className="text-xs text-muted-foreground">信号</div>
                  <div className={`text-lg font-bold ${selectedStock.signal === '买入' ? 'text-green-400' : selectedStock.signal === '卖出' ? 'text-red-400' : 'text-yellow-400'}`}>
                    {selectedStock.signal}
                  </div>
                </div>
              </div>
              
              <div className="border-t pt-4">
                <h4 className="text-sm font-medium mb-2 flex items-center gap-1">
                  <LineChart className="w-4 h-4" /> 均线分析
                </h4>
                <div className="grid grid-cols-3 gap-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">MA5</span>
                    <span>{selectedStock.ma5.toFixed(2)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">MA10</span>
                    <span>{selectedStock.ma10.toFixed(2)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">MA20</span>
                    <span>{selectedStock.ma20.toFixed(2)}</span>
                  </div>
                </div>
              </div>
              
              <div className="border-t pt-4">
                <h4 className="text-sm font-medium mb-2 flex items-center gap-1">
                  <Target className="w-4 h-4" /> 关键价位
                </h4>
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div className="flex justify-between text-green-400">
                    <span>支撑位</span>
                    <span>{selectedStock.support.toFixed(2)}</span>
                  </div>
                  <div className="flex justify-between text-red-400">
                    <span>压力位</span>
                    <span>{selectedStock.resistance.toFixed(2)}</span>
                  </div>
                </div>
              </div>
              
              <div className="border-t pt-4">
                <h4 className="text-sm font-medium mb-2 flex items-center gap-1">
                  <Shield className="w-4 h-4" /> 操作建议
                </h4>
                <p className="text-sm bg-muted rounded p-3">{selectedStock.reason}</p>
              </div>
            </div>
            
            <button onClick={() => setSelectedStock(null)} className="w-full mt-4 py-2 bg-muted rounded hover:bg-muted/80">
              关闭
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
