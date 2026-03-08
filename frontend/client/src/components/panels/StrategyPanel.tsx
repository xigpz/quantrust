import { useState, useEffect, useContext } from 'react';
import { StockClickContext } from '@/pages/Dashboard';

const API_BASE = '';

interface StrategyStock {
  symbol: string;
  name: string;
  price: number;
  change: number;
  change_pct: number;
  volume: number;
  turnover: number;
  reason: string;
  score: number;
}

interface VirtualPosition {
  symbol: string;
  name: string;
  quantity: number;
  avg_cost: number;
  current_price: number;
  market_value: number;
  profit_loss: number;
  profit_ratio: number;
}

interface VirtualAccount {
  total_assets: number;
  cash: number;
  market_value: number;
  total_profit: number;
  profit_ratio: number;
}

type StrategyType = 'breakout' | 'momentum' | 'volume' | 'turnover' | 'lowpe' | 'allback';

export default function StrategyPanel() {
  const { openStock } = useContext(StockClickContext);
  const [strategy, setStrategy] = useState<StrategyType>('breakout');
  const [stocks, setStocks] = useState<StrategyStock[]>([]);
  const [positions, setPositions] = useState<VirtualPosition[]>([]);
  const [account, setAccount] = useState<VirtualAccount | null>(null);
  const [loading, setLoading] = useState(false);
  const [buying, setBuying] = useState<string | null>(null);

  const strategies = [
    { id: 'breakout', name: '突破新高', desc: '连续涨停突破前期高点' },
    { id: 'momentum', name: '动量强劲', desc: '近期涨幅靠前且成交活跃' },
    { id: 'volume', name: '量价齐升', desc: '成交量放大且股价上涨' },
    { id: 'turnover', name: '换手率高', desc: '换手率异常放大' },
    { id: 'lowpe', name: '低估值', desc: 'PE低于行业平均' },
    { id: 'allback', name: '超跌反弹', desc: '近期跌幅较大但有企稳迹象' },
  ];

  const fetchPositions = async () => {
    const posRes = await fetch(`${API_BASE}/api/virtual/positions`).then(r => r.json());
    const accRes = await fetch(`${API_BASE}/api/virtual/account`).then(r => r.json());
    const quotesRes = await fetch(`${API_BASE}/api/quotes`).then(r => r.json()).catch(() => ({ success: false, data: [] }));
    
    if (posRes.success) {
      const quotes = quotesRes.success ? quotesRes.data : [];
      const updatedPositions = posRes.data.map((p: VirtualPosition) => {
        const quote = quotes.find((q: any) => q.symbol === p.symbol);
        const currentPrice = quote?.price || p.current_price;
        const profitLoss = (currentPrice - p.avg_cost) * p.quantity;
        const profitRatio = p.avg_cost > 0 ? ((currentPrice - p.avg_cost) / p.avg_cost * 100) : 0;
        return { ...p, current_price: currentPrice, profit_loss: profitLoss, profit_ratio: profitRatio, market_value: currentPrice * p.quantity };
      });
      setPositions(updatedPositions);
    }
    if (accRes.success) setAccount(accRes.data);
  };

  const runStrategy = async () => {
    setLoading(true);
    setStocks([]);
    
    try {
      let results: StrategyStock[] = [];
      const quotesRes = await fetch(`${API_BASE}/api/quotes`).then(r => r.json());
      if (!quotesRes.success) {
        alert('获取行情数据失败');
        setLoading(false);
        return;
      }
      
      const quotes = quotesRes.data;
      
      switch (strategy) {
        case 'breakout':
          results = quotes
            .filter((q: any) => q.change_pct >= 9.5 && q.volume > 50000000)
            .slice(0, 10)
            .map((q: any) => ({
              symbol: q.symbol,
              name: q.name || q.symbol,
              price: q.price,
              change: q.change,
              change_pct: q.change_pct,
              volume: q.volume,
              turnover: q.turnover,
              reason: '涨停突破，有望继续上涨',
              score: q.change_pct * 10 + (q.volume / 1000000),
            }));
          break;
          
        case 'momentum':
          results = quotes
            .filter((q: any) => q.change_pct > 0)
            .sort((a: any, b: any) => b.change_pct - a.change_pct)
            .slice(0, 15)
            .map((q: any) => ({
              symbol: q.symbol,
              name: q.name || q.symbol,
              price: q.price,
              change: q.change,
              change_pct: q.change_pct,
              volume: q.volume,
              turnover: q.turnover,
              reason: '动量强劲，持续上涨',
              score: q.change_pct + (q.volume / 100000000),
            }));
          break;
          
        case 'volume':
          results = quotes
            .filter((q: any) => q.change_pct > 3 && q.volume > 30000000)
            .sort((a: any, b: any) => b.volume - a.volume)
            .slice(0, 10)
            .map((q: any) => ({
              symbol: q.symbol,
              name: q.name || q.symbol,
              price: q.price,
              change: q.change,
              change_pct: q.change_pct,
              volume: q.volume,
              turnover: q.turnover,
              reason: '成交量放大，量价齐升',
              score: q.volume / 10000000 + q.change_pct * 5,
            }));
          break;
          
        case 'turnover':
          results = quotes
            .filter((q: any) => q.turnover > 15 && q.change_pct > 0)
            .sort((a: any, b: any) => b.turnover - a.turnover)
            .slice(0, 10)
            .map((q: any) => ({
              symbol: q.symbol,
              name: q.name || q.symbol,
              price: q.price,
              change: q.change,
              change_pct: q.change_pct,
              volume: q.volume,
              turnover: q.turnover,
              reason: '换手率极高，交易活跃',
              score: q.turnover * 2 + q.change_pct * 3,
            }));
          break;
          
        case 'lowpe':
          results = quotes
            .filter((q: any) => q.change_pct > -3 && q.change_pct < 5)
            .sort((a: any, b: any) => a.change_pct - b.change_pct)
            .slice(0, 10)
            .map((q: any) => ({
              symbol: q.symbol,
              name: q.name || q.symbol,
              price: q.price,
              change: q.change,
              change_pct: q.change_pct,
              volume: q.volume,
              turnover: q.turnover,
              reason: '估值较低，安全边际高',
              score: 50 - q.change_pct,
            }));
          break;
          
        case 'allback':
          results = quotes
            .filter((q: any) => q.change_pct < -5 && q.volume > 20000000)
            .sort((a: any, b: any) => a.change_pct - b.change_pct)
            .slice(0, 10)
            .map((q: any) => ({
              symbol: q.symbol,
              name: q.name || q.symbol,
              price: q.price,
              change: q.change,
              change_pct: q.change_pct,
              volume: q.volume,
              turnover: q.turnover,
              reason: '超跌有望反弹',
              score: Math.abs(q.change_pct) * 2 + q.volume / 10000000,
            }));
          break;
      }
      
      results.sort((a, b) => b.score - a.score);
      setStocks(results);
    } catch (e) {
      console.error(e);
      alert('策略执行失败');
    }
    
    setLoading(false);
  };

  const buyStock = async (stock: StrategyStock) => {
    if (!account || account.cash < stock.price * 100) {
      alert('现金不足，至少需要买入100股');
      return;
    }
    
    setBuying(stock.symbol);
    try {
      const res = await fetch(`${API_BASE}/api/virtual/buy`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          symbol: stock.symbol,
          name: stock.name,
          price: stock.price,
          quantity: 100,
          reason: `${strategies.find(s => s.id === strategy)?.name}策略选股`,
        }),
      }).then(r => r.json());
      
      if (res.success) {
        setAccount(res.data);
        fetchPositions();
        alert(`成功买入 ${stock.symbol} 100股`);
      } else {
        alert(res.message || '买入失败');
      }
    } catch (e) {
      alert('买入失败');
    }
    setBuying(null);
  };

  useEffect(() => {
    fetchPositions();
  }, []);

  return (
    <div className="p-4 space-y-4">
      {/* 策略选择 */}
      <div className="bg-card rounded-lg p-4 border">
        <h2 className="text-lg font-semibold mb-3">选股策略</h2>
        <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
          {strategies.map(s => (
            <button
              key={s.id}
              onClick={() => setStrategy(s.id as StrategyType)}
              className={`p-3 rounded-lg text-left transition-colors ${
                strategy === s.id
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted hover:bg-muted/80'
              }`}
            >
              <div className="font-medium">{s.name}</div>
              <div className={`text-xs ${strategy === s.id ? 'text-primary-foreground/70' : 'text-muted-foreground'}`}>
                {s.desc}
              </div>
            </button>
          ))}
        </div>
        
        <button
          onClick={runStrategy}
          disabled={loading}
          className="w-full mt-4 py-3 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 font-medium"
        >
          {loading ? '正在选股...' : '执行选股'}
        </button>
      </div>

      {/* 账户信息 */}
      {account && (
        <div className="bg-card rounded-lg p-3 border flex justify-between items-center">
          <div>
            <span className="text-muted-foreground text-sm">可用现金: </span>
            <span className="font-bold text-green-500">{account.cash.toFixed(2)}</span>
          </div>
          <div>
            <span className="text-muted-foreground text-sm">持仓: </span>
            <span className="font-bold">{positions.length}只</span>
          </div>
        </div>
      )}

      {/* 选股结果 */}
      <div className="bg-card rounded-lg border">
        <div className="p-3 border-b">
          <h3 className="font-semibold">策略选股结果 {stocks.length > 0 && `(${stocks.length}只)`}</h3>
        </div>
        
        {stocks.length === 0 ? (
          <div className="p-8 text-center text-muted-foreground">
            点击上方"执行选股"获取股票列表
          </div>
        ) : (
          <div className="divide-y">
            {stocks.map(stock => {
              const hasPosition = positions.some(p => p.symbol === stock.symbol);
              return (
                <div key={stock.symbol} className="p-3 flex items-center justify-between hover:bg-muted/50 cursor-pointer" onClick={() => openStock(stock.symbol, stock.name)}>
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{stock.symbol}</span>
                      <span className="text-sm text-muted-foreground">{stock.name}</span>
                    </div>
                    <div className="text-xs text-muted-foreground mt-1">{stock.reason}</div>
                  </div>
                  
                  <div className="text-right mx-4">
                    <div className="font-medium">{stock.price.toFixed(2)}</div>
                    <div className={`text-xs ${stock.change_pct >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                      {stock.change >= 0 ? '+' : ''}{stock.change_pct.toFixed(2)}%
                    </div>
                  </div>
                  
                  {hasPosition ? (
                    <span className="px-3 py-1 bg-muted text-muted-foreground text-xs rounded">
                      已持仓
                    </span>
                  ) : (
                    <button
                      onClick={() => buyStock(stock)}
                      disabled={buying === stock.symbol || (account && account.cash < stock.price * 100)}
                      className="px-3 py-1 bg-green-600 text-white text-xs rounded hover:bg-green-700 disabled:opacity-50"
                    >
                      {buying === stock.symbol ? '买入中' : '买入100股'}
                    </button>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* 当前持仓 */}
      {positions.length > 0 && (
        <div className="bg-card rounded-lg border">
          <div className="p-3 border-b">
            <h3 className="font-semibold">当前持仓</h3>
          </div>
          <div className="divide-y">
            {positions.map(pos => (
              <div key={pos.symbol} className="p-3 flex justify-between items-center">
                <div>
                  <div className="font-medium">{pos.symbol}</div>
                  <div className="text-xs text-muted-foreground">
                    {pos.quantity}股 @ {pos.avg_cost.toFixed(2)}
                  </div>
                </div>
                <div className="text-right">
                  <div className="font-medium">{pos.market_value.toFixed(2)}</div>
                  <div className={`text-xs ${pos.profit_loss >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                    {pos.profit_loss >= 0 ? '+' : ''}{pos.profit_loss.toFixed(2)} ({pos.profit_ratio.toFixed(2)}%)
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
