import { useState, useEffect } from 'react';
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts';

const API_BASE = '';

interface VirtualAccount {
  total_assets: number;
  cash: number;
  market_value: number;
  total_profit: number;
  profit_ratio: number;
  total_trades: number;
  update_time: string;
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
  update_time: string;
}

interface VirtualTrade {
  id: string;
  symbol: string;
  name: string;
  direction: string;
  price: number;
  quantity: number;
  amount: number;
  commission: number;
  trade_time: string;
  reason: string;
}

export default function VirtualTradingPanel() {
  const [account, setAccount] = useState<VirtualAccount | null>(null);
  const [positions, setPositions] = useState<VirtualPosition[]>([]);
  const [trades, setTrades] = useState<VirtualTrade[]>([]);
  const [activeTab, setActiveTab] = useState<'account' | 'positions' | 'trades'>('account');
  const [tradeModal, setTradeModal] = useState<{type: 'buy' | 'sell'; symbol?: string; name?: string; price?: number} | null>(null);
  const [tradeForm, setTradeForm] = useState({ symbol: '', name: '', price: 0, quantity: 0, reason: '' });
  const [loading, setLoading] = useState(false);

  const fetchData = async () => {
    try {
      const [accRes, posRes, tradesRes, quotesRes] = await Promise.all([
        fetch(`${API_BASE}/api/virtual/account`).then(r => r.json()),
        fetch(`${API_BASE}/api/virtual/positions`).then(r => r.json()),
        fetch(`${API_BASE}/api/virtual/trades?limit=20`).then(r => r.json()),
        fetch(`${API_BASE}/api/quotes`).then(r => r.json()).catch(() => ({ success: false, data: [] })),
      ]);
      if (accRes.success) setAccount(accRes.data);
      if (posRes.success) {
        // 用实时价格更新持仓盈亏
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
      if (tradesRes.success) setTrades(tradesRes.data);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 30000);
    return () => clearInterval(interval);
  }, []);

  const handleTrade = async () => {
    if (!tradeForm.symbol || !tradeForm.price || !tradeForm.quantity) return;
    setLoading(true);
    try {
      const endpoint = tradeModal?.type === 'buy' ? '/api/virtual/buy' : '/api/virtual/sell';
      const res = await fetch(`${API_BASE}${endpoint}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          symbol: tradeForm.symbol,
          name: tradeForm.name || tradeForm.symbol,
          price: tradeForm.price,
          quantity: tradeForm.quantity,
          reason: tradeForm.reason,
        }),
      }).then(r => r.json());
      
      if (res.success) {
        setAccount(res.data);
        fetchData();
        setTradeModal(null);
        setTradeForm({ symbol: '', name: '', price: 0, quantity: 0, reason: '' });
      } else {
        alert(res.message || '交易失败');
      }
    } catch (e) {
      alert('交易失败');
    }
    setLoading(false);
  };

  const openTradeModal = (type: 'buy' | 'sell', pos?: VirtualPosition) => {
    setTradeModal({ 
      type, 
      symbol: pos?.symbol, 
      name: pos?.name, 
      price: pos?.current_price 
    });
    setTradeForm({
      symbol: pos?.symbol || '',
      name: pos?.name || '',
      price: pos?.current_price || 0,
      quantity: 0,
      reason: '',
    });
  };

  const resetAccount = async () => {
    if (!confirm('确定要重置账户吗？所有持仓和交易记录将被清空！')) return;
    await fetch(`${API_BASE}/api/virtual/reset?capital=100000`, { method: 'POST' });
    fetchData();
  };

  return (
    <div className="p-4 space-y-4">
      {/* 账户概览 */}
      <div className="bg-card rounded-lg p-4 border">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg font-semibold">虚拟账户</h2>
          <button 
            onClick={resetAccount}
            className="text-xs px-2 py-1 bg-destructive text-destructive-foreground rounded hover:bg-destructive/80"
          >
            重置账户
          </button>
        </div>
        
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div>
            <div className="text-xs text-muted-foreground">总资产</div>
            <div className="text-xl font-bold">{account?.total_assets.toFixed(2)}</div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">现金</div>
            <div className="text-xl font-bold">{account?.cash.toFixed(2)}</div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">持仓市值</div>
            <div className="text-xl font-bold">{account?.market_value.toFixed(2)}</div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">总盈亏</div>
            <div className={`text-xl font-bold ${(account?.total_profit || 0) >= 0 ? 'text-green-500' : 'text-red-500'}`}>
              {account?.total_profit?.toFixed(2)} ({account?.profit_ratio?.toFixed(2)}%)
            </div>
          </div>
        </div>
      </div>

      {/* 快捷操作 */}
      <div className="flex gap-2">
        <button
          onClick={() => openTradeModal('buy')}
          className="flex-1 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 font-medium"
        >
          买入股票
        </button>
        <button
          onClick={() => openTradeModal('sell')}
          className="flex-1 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 font-medium"
        >
          卖出股票
        </button>
      </div>

      {/* Tab切换 */}
      <div className="flex border-b">
        {(['account', 'positions', 'trades'] as const).map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 text-sm font-medium ${
              activeTab === tab 
                ? 'border-b-2 border-primary text-primary' 
                : 'text-muted-foreground'
            }`}
          >
            {tab === 'account' ? '账户' : tab === 'positions' ? '持仓' : '交易记录'}
            {tab === 'positions' && positions.length > 0 && ` (${positions.length})`}
            {tab === 'trades' && trades.length > 0 && ` (${trades.length})`}
          </button>
        ))}
      </div>

      {/* 持仓列表 */}
      {activeTab === 'positions' && (
        <div className="space-y-2">
          {positions.length === 0 ? (
            <div className="text-center text-muted-foreground py-8">暂无持仓</div>
          ) : (
            positions.map(pos => (
              <div key={pos.symbol} className="bg-card rounded-lg p-3 border flex justify-between items-center">
                <div>
                  <div className="font-medium">{pos.symbol}</div>
                  <div className="text-xs text-muted-foreground">
                    数量: {pos.quantity} | 成本: {pos.avg_cost.toFixed(2)}
                  </div>
                </div>
                <div className="text-right">
                  <div className="font-medium">{pos.market_value.toFixed(2)}</div>
                  <div className={`text-xs ${pos.profit_loss >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                    {pos.profit_loss.toFixed(2)} ({pos.profit_ratio.toFixed(2)}%)
                  </div>
                </div>
                <button
                  onClick={() => openTradeModal('sell', pos)}
                  className="ml-3 px-3 py-1 text-xs bg-red-100 text-red-700 rounded"
                >
                  卖出
                </button>
              </div>
            ))
          )}
        </div>
      )}

      {/* 交易记录 */}
      {activeTab === 'trades' && (
        <div className="space-y-2">
          {trades.length === 0 ? (
            <div className="text-center text-muted-foreground py-8">暂无交易记录</div>
          ) : (
            trades.map(trade => (
              <div key={trade.id} className="bg-card rounded-lg p-3 border">
                <div className="flex justify-between items-center">
                  <div>
                    <span className={`font-medium ${trade.direction === '买入' ? 'text-green-500' : 'text-red-500'}`}>
                      {trade.direction}
                    </span>
                    <span className="ml-2">{trade.symbol}</span>
                  </div>
                  <div className="text-sm text-muted-foreground">{trade.trade_time}</div>
                </div>
                <div className="text-xs text-muted-foreground mt-1">
                  {trade.quantity}股 @{trade.price} = {trade.amount.toFixed(2)} (手续费: {trade.commission.toFixed(2)})
                </div>
                {trade.reason && (
                  <div className="text-xs text-muted-foreground mt-1">原因: {trade.reason}</div>
                )}
              </div>
            ))
          )}
        </div>
      )}

      {/* 交易弹窗 */}
      {tradeModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-background rounded-lg p-6 w-96 max-w-[90vw]">
            <h3 className="text-lg font-semibold mb-4">
              {tradeModal.type === 'buy' ? '买入' : '卖出'}股票
            </h3>
            
            <div className="space-y-3">
              <div>
                <label className="text-xs text-muted-foreground">股票代码</label>
                <input
                  type="text"
                  value={tradeForm.symbol}
                  onChange={e => setTradeForm({...tradeForm, symbol: e.target.value})}
                  className="w-full px-3 py-2 border rounded bg-background"
                  placeholder="如: 301218.SZ"
                />
              </div>
              <div>
                <label className="text-xs text-muted-foreground">股票名称</label>
                <input
                  type="text"
                  value={tradeForm.name}
                  onChange={e => setTradeForm({...tradeForm, name: e.target.value})}
                  className="w-full px-3 py-2 border rounded bg-background"
                  placeholder="可选"
                />
              </div>
              <div>
                <label className="text-xs text-muted-foreground">价格</label>
                <input
                  type="number"
                  step="0.01"
                  value={tradeForm.price}
                  onChange={e => setTradeForm({...tradeForm, price: parseFloat(e.target.value) || 0})}
                  className="w-full px-3 py-2 border rounded bg-background"
                />
              </div>
              <div>
                <label className="text-xs text-muted-foreground">数量(股)</label>
                <input
                  type="number"
                  value={tradeForm.quantity}
                  onChange={e => setTradeForm({...tradeForm, quantity: parseInt(e.target.value) || 0})}
                  className="w-full px-3 py-2 border rounded bg-background"
                />
              </div>
              <div>
                <label className="text-xs text-muted-foreground">交易原因(可选)</label>
                <input
                  type="text"
                  value={tradeForm.reason}
                  onChange={e => setTradeForm({...tradeForm, reason: e.target.value})}
                  className="w-full px-3 py-2 border rounded bg-background"
                  placeholder="如: 突破买入"
                />
              </div>
              
              {tradeForm.price > 0 && tradeForm.quantity > 0 && (
                <div className="bg-muted p-3 rounded text-sm">
                  <div>成交金额: {(tradeForm.price * tradeForm.quantity).toFixed(2)}</div>
                  <div>手续费(万三): {(tradeForm.price * tradeForm.quantity * 0.0003).toFixed(2)}</div>
                </div>
              )}
            </div>

            <div className="flex gap-2 mt-6">
              <button
                onClick={() => setTradeModal(null)}
                className="flex-1 py-2 border rounded hover:bg-muted"
              >
                取消
              </button>
              <button
                onClick={handleTrade}
                disabled={loading || !tradeForm.symbol || !tradeForm.price || !tradeForm.quantity}
                className={`flex-1 py-2 text-white rounded ${
                  tradeModal.type === 'buy' ? 'bg-green-600 hover:bg-green-700' : 'bg-red-600 hover:bg-red-700'
                } disabled:opacity-50`}
              >
                {loading ? '处理中...' : tradeModal.type === 'buy' ? '确认买入' : '确认卖出'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
