import { useState, useEffect } from 'react';

interface Position {
  symbol: string;
  name: string;
  quantity: number;
  avg_cost: number;
  current_price: number;
  market_value: number;
  pnl: number;
  pnl_rate: number;
}

interface Order {
  id: string;
  symbol: string;
  name: string;
  direction: string;
  price: number;
  quantity: number;
  filled: number;
  status: string;
  created_at: string;
}

interface Trade {
  id: string;
  symbol: string;
  direction: string;
  price: number;
  quantity: number;
  amount: number;
  traded_at: string;
}

interface Account {
  cash: number;
  total_value: number;
  positions_value: number;
  positions_count: number;
  today_pnl: number;
  total_pnl: number;
}

export default function SimTrading() {
  const [account, setAccount] = useState<Account | null>(null);
  const [positions, setPositions] = useState<Position[]>([]);
  const [orders, setOrders] = useState<Order[]>([]);
  const [trades, setTrades] = useState<Trade[]>([]);
  const [activeTab, setActiveTab] = useState<'positions' | 'orders' | 'trades'>('positions');

  // 刷新数据
  const refresh = () => {
    fetch('/api/sim/account')
      .then(r => r.json())
      .then(d => { if (d.data) setAccount(d.data); });
    fetch('/api/sim/positions')
      .then(r => r.json())
      .then(d => { if (d.data) setPositions(d.data); });
    fetch('/api/sim/orders')
      .then(r => r.json())
      .then(d => { if (d.data) setOrders(d.data); });
    fetch('/api/sim/trades')
      .then(r => r.json())
      .then(d => { if (d.data) setTrades(d.data); });
  };

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, 5000); // 每5秒刷新
    return () => clearInterval(interval);
  }, []);

  const formatMoney = (n: number) => n.toLocaleString('zh-CN', { style: 'currency', currency: 'CNY' });
  const formatRate = (n: number) => `${n >= 0 ? '+' : ''}${n.toFixed(2)}%`;

  return (
    <div className="flex h-full gap-4 p-4">
      {/* 左侧：账户概览 */}
      <div className="w-72 bg-gray-800 rounded-lg p-4">
        <h3 className="text-lg font-bold mb-4">模拟账户</h3>
        
        <div className="space-y-3">
          <div className="flex justify-between">
            <span className="text-gray-400">现金</span>
            <span className="font-mono">{account ? formatMoney(account.cash) : '-'}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">持仓市值</span>
            <span className="font-mono">{account ? formatMoney(account.positions_value) : '-'}</span>
          </div>
          <div className="flex justify-between text-lg font-bold">
            <span>总资产</span>
            <span className="text-blue-400">{account ? formatMoney(account.total_value) : '-'}</span>
          </div>
          <hr className="border-gray-700" />
          <div className="flex justify-between">
            <span className="text-gray-400">持仓数</span>
            <span>{account?.positions_count || 0}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">今日盈亏</span>
            <span className={account?.today_pnl >= 0 ? 'text-green-400' : 'text-red-400'}>
              {account ? formatMoney(account.today_pnl) : '-'}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">累计盈亏</span>
            <span className={account?.total_pnl >= 0 ? 'text-green-400' : 'text-red-400'}>
              {account ? formatRate(account.total_pnl) : '-'}
            </span>
          </div>
        </div>

        <button
          onClick={() => fetch('/api/sim/reset', { method: 'POST' }).then(refresh)}
          className="w-full mt-4 bg-red-600 hover:bg-red-700 text-white py-2 rounded"
        >
          重置账户
        </button>
      </div>

      {/* 中间：持仓/订单/成交 */}
      <div className="flex-1 bg-gray-800 rounded-lg p-4 flex flex-col">
        {/* Tabs */}
        <div className="flex gap-2 mb-4">
          <button
            onClick={() => setActiveTab('positions')}
            className={`px-4 py-2 rounded ${activeTab === 'positions' ? 'bg-blue-600' : 'bg-gray-700'}`}
          >
            持仓 ({positions.length})
          </button>
          <button
            onClick={() => setActiveTab('orders')}
            className={`px-4 py-2 rounded ${activeTab === 'orders' ? 'bg-blue-600' : 'bg-gray-700'}`}
          >
            订单 ({orders.filter(o => o.status === 'pending').length})
          </button>
          <button
            onClick={() => setActiveTab('trades')}
            className={`px-4 py-2 rounded ${activeTab === 'trades' ? 'bg-blue-600' : 'bg-gray-700'}`}
          >
            成交 ({trades.length})
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-auto">
          {activeTab === 'positions' && (
            <table className="w-full text-sm">
              <thead className="text-gray-400 border-b border-gray-700">
                <tr>
                  <th className="text-left py-2">股票</th>
                  <th className="text-right py-2">持仓</th>
                  <th className="text-right py-2">成本</th>
                  <th className="text-right py-2">现价</th>
                  <th className="text-right py-2">市值</th>
                  <th className="text-right py-2">盈亏</th>
                </tr>
              </thead>
              <tbody>
                {positions.map(p => (
                  <tr key={p.symbol} className="border-b border-gray-700/50">
                    <td className="py-2">
                      <div className="font-medium">{p.symbol}</div>
                      <div className="text-xs text-gray-400">{p.name}</div>
                    </td>
                    <td className="text-right font-mono">{p.quantity}</td>
                    <td className="text-right font-mono">{p.avg_cost.toFixed(2)}</td>
                    <td className="text-right font-mono">{p.current_price.toFixed(2)}</td>
                    <td className="text-right font-mono">{formatMoney(p.market_value)}</td>
                    <td className={`text-right font-mono ${p.pnl >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                      {formatMoney(p.pnl)} ({formatRate(p.pnl_rate)})
                    </td>
                  </tr>
                ))}
                {positions.length === 0 && (
                  <tr>
                    <td colSpan={6} className="text-center text-gray-500 py-8">暂无持仓</td>
                  </tr>
                )}
              </tbody>
            </table>
          )}

          {activeTab === 'orders' && (
            <table className="w-full text-sm">
              <thead className="text-gray-400 border-b border-gray-700">
                <tr>
                  <th className="text-left py-2">时间</th>
                  <th className="text-left py-2">股票</th>
                  <th className="text-center py-2">方向</th>
                  <th className="text-right py-2">价格</th>
                  <th className="text-right py-2">数量</th>
                  <th className="text-center py-2">状态</th>
                </tr>
              </thead>
              <tbody>
                {orders.map(o => (
                  <tr key={o.id} className="border-b border-gray-700/50">
                    <td className="py-2 text-gray-400">{new Date(o.created_at).toLocaleTimeString()}</td>
                    <td className="py-2">{o.symbol}</td>
                    <td className={`text-center py-2 ${o.direction === 'buy' ? 'text-red-400' : 'text-green-400'}`}>
                      {o.direction === 'buy' ? '买入' : '卖出'}
                    </td>
                    <td className="text-right font-mono">{o.price.toFixed(2)}</td>
                    <td className="text-right font-mono">{o.quantity}</td>
                    <td className="text-center">
                      <span className={`px-2 py-1 rounded text-xs ${
                        o.status === 'filled' ? 'bg-green-600' :
                        o.status === 'pending' ? 'bg-yellow-600' : 'bg-gray-600'
                      }`}>
                        {o.status === 'filled' ? '已成交' : o.status === 'pending' ? '待成交' : '已取消'}
                      </span>
                    </td>
                  </tr>
                ))}
                {orders.length === 0 && (
                  <tr>
                    <td colSpan={6} className="text-center text-gray-500 py-8">暂无订单</td>
                  </tr>
                )}
              </tbody>
            </table>
          )}

          {activeTab === 'trades' && (
            <table className="w-full text-sm">
              <thead className="text-gray-400 border-b border-gray-700">
                <tr>
                  <th className="text-left py-2">时间</th>
                  <th className="text-left py-2">股票</th>
                  <th className="text-center py-2">方向</th>
                  <th className="text-right py-2">价格</th>
                  <th className="text-right py-2">数量</th>
                  <th className="text-right py-2">金额</th>
                </tr>
              </thead>
              <tbody>
                {trades.slice().reverse().map(t => (
                  <tr key={t.id} className="border-b border-gray-700/50">
                    <td className="py-2 text-gray-400">{new Date(t.traded_at).toLocaleTimeString()}</td>
                    <td className="py-2">{t.symbol}</td>
                    <td className={`text-center py-2 ${t.direction === 'buy' ? 'text-red-400' : 'text-green-400'}`}>
                      {t.direction === 'buy' ? '买入' : '卖出'}
                    </td>
                    <td className="text-right font-mono">{t.price.toFixed(2)}</td>
                    <td className="text-right font-mono">{t.quantity}</td>
                    <td className="text-right font-mono">{formatMoney(t.amount)}</td>
                  </tr>
                ))}
                {trades.length === 0 && (
                  <tr>
                    <td colSpan={6} className="text-center text-gray-500 py-8">暂无成交记录</td>
                  </tr>
                )}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </div>
  );
}
