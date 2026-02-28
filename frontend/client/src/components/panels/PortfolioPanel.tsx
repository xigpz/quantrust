/**
 * PortfolioPanel - 持仓分析/风险仪表盘
 */
import { useState, useEffect } from 'react';
import { PieChart, TrendingUp, TrendingDown, AlertTriangle, Shield, RefreshCw, DollarSign } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';

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

interface Account {
  cash: number;
  total_value: number;
  positions_value: number;
  positions_count: number;
  today_pnl: number;
  total_pnl: number;
}

interface RiskMetrics {
  totalValue: number;
  cash: number;
  positions: Position[];
  account: Account | null;
  loading: boolean;
}

function MetricCard({ title, value, subtitle, icon: Icon, color }: {
  title: string;
  value: string;
  subtitle?: string;
  icon: React.ElementType;
  color: string;
}) {
  return (
    <div className="bg-card rounded-lg p-3 border border-border">
      <div className="flex items-center gap-2 mb-1">
        <Icon className={`w-4 h-4 ${color}`} />
        <span className="text-xs text-muted-foreground">{title}</span>
      </div>
      <div className={`text-lg font-bold ${color}`}>{value}</div>
      {subtitle && <div className="text-xs text-muted-foreground">{subtitle}</div>}
    </div>
  );
}

export default function PortfolioPanel() {
  const [data, setData] = useState<RiskMetrics>({
    totalValue: 0,
    cash: 0,
    positions: [],
    account: null,
    loading: true,
  });

  const loadData = async () => {
    setData(prev => ({ ...prev, loading: true }));
    try {
      // 获取账户数据
      const [accountRes, positionsRes] = await Promise.all([
        fetch('/api/sim/account'),
        fetch('/api/sim/positions'),
      ]);
      
      const accountData = await accountRes.json();
      const positionsData = await positionsRes.json();
      
      const account = accountData.success ? accountData.data : null;
      const positions = positionsData.success ? positionsData.data : [];
      
      const positionsValue = positions.reduce((sum: number, p: Position) => sum + p.market_value, 0);
      
      setData({
        totalValue: account?.total_value || 0,
        cash: account?.cash || 0,
        positions,
        account,
        loading: false,
      });
    } catch (e) {
      console.error('Failed to load portfolio:', e);
      setData(prev => ({ ...prev, loading: false }));
    }
  };

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 30000); // 每30秒刷新
    return () => clearInterval(interval);
  }, []);

  const { totalValue, cash, positions, account, loading } = data;
  const positionsValue = totalValue - cash;
  
  // 计算持仓占比
  const positionRatios = positions.map(p => ({
    ...p,
    ratio: totalValue > 0 ? (p.market_value / totalValue) * 100 : 0,
  })).sort((a, b) => b.market_value - a.market_value);

  // 盈亏统计
  const winningPositions = positions.filter(p => p.pnl > 0);
  const losingPositions = positions.filter(p => p.pnl < 0);
  const totalPnl = positions.reduce((sum, p) => sum + p.pnl, 0);

  // 资金占比
  const cashRatio = totalValue > 0 ? (cash / totalValue) * 100 : 0;
  const positionsRatio = totalValue > 0 ? (positionsValue / totalValue) * 100 : 0;

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-2 border-b border-border">
        <div className="flex items-center gap-2">
          <PieChart className="w-4 h-4 text-indigo-400" />
          <h2 className="text-sm font-medium">持仓分析</h2>
        </div>
        <button onClick={loadData} className="text-muted-foreground hover:text-foreground transition-colors">
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-4 space-y-4">
          {/* 总资产卡片 */}
          <div className="bg-gradient-to-r from-indigo-900 to-purple-900 rounded-lg p-4 border border-border">
            <div className="text-xs text-indigo-300 mb-1">总资产</div>
            <div className="text-2xl font-bold text-white">
              ¥{totalValue.toLocaleString('zh-CN', { maximumFractionDigits: 2 })}
            </div>
            <div className={`text-sm mt-1 ${totalPnl >= 0 ? 'text-green-400' : 'text-red-400'}`}>
              {totalPnl >= 0 ? '+' : ''}¥{totalPnl.toLocaleString('zh-CN', { maximumFractionDigits: 2 })} 
              ({totalPnl >= 0 ? '+' : ''}{((totalPnl / (totalValue - totalPnl)) * 100).toFixed(2)}%)
            </div>
          </div>

          {/* 关键指标 */}
          <div className="grid grid-cols-2 gap-3">
            <MetricCard 
              title="现金" 
              value={`¥${cash.toLocaleString('zh-CN', { maximumFractionDigits: 0 })}`}
              subtitle={`${cashRatio.toFixed(1)}%`}
              icon={DollarSign}
              color="text-blue-400"
            />
            <MetricCard 
              title="持仓市值" 
              value={`¥${positionsValue.toLocaleString('zh-CN', { maximumFractionDigits: 0 })}`}
              subtitle={`${positionsRatio.toFixed(1)}%`}
              icon={PieChart}
              color="text-purple-400"
            />
            <MetricCard 
              title="盈利持仓" 
              value={`${winningPositions.length}只`}
              subtitle={`¥${winningPositions.reduce((s, p) => s + p.pnl, 0).toLocaleString()}`}
              icon={TrendingUp}
              color="text-green-400"
            />
            <MetricCard 
              title="亏损持仓" 
              value={`${losingPositions.length}只`}
              subtitle={`¥${losingPositions.reduce((s, p) => s + p.pnl, 0).toLocaleString()}`}
              icon={TrendingDown}
              color="text-red-400"
            />
          </div>

          {/* 仓位分布 */}
          {positions.length > 0 && (
            <div className="bg-card rounded-lg p-4 border border-border">
              <div className="flex items-center gap-2 mb-3">
                <Shield className="w-4 h-4 text-yellow-400" />
                <span className="text-sm font-medium">持仓明细</span>
              </div>
              
              {/* 仓位条 */}
              <div className="h-4 bg-gray-700 rounded-full overflow-hidden flex mb-4">
                <div 
                  className="h-full bg-blue-500"
                  style={{ width: `${cashRatio}%` }}
                  title={`现金 ${cashRatio.toFixed(1)}%`}
                />
                {positionRatios.slice(0, 5).map((p, i) => (
                  <div
                    key={p.symbol}
                    className={`h-full ${p.pnl >= 0 ? 'bg-green-500' : 'bg-red-500'}`}
                    style={{ width: `${p.ratio}%` }}
                    title={`${p.name} ${p.ratio.toFixed(1)}%`}
                  />
                ))}
              </div>

              {/* 持仓列表 */}
              <div className="space-y-2">
                {positionRatios.map((p) => (
                  <div key={p.symbol} className="flex items-center justify-between text-xs">
                    <div className="flex items-center gap-2">
                      <div className={`w-2 h-2 rounded-full ${p.pnl >= 0 ? 'bg-green-500' : 'bg-red-500'}`} />
                      <span className="font-medium">{p.name}</span>
                      <span className="text-muted-foreground">{p.symbol}</span>
                    </div>
                    <div className="text-right">
                      <div className={p.pnl >= 0 ? 'text-green-400' : 'text-red-400'}>
                        {p.pnl >= 0 ? '+' : ''}¥{p.pnl.toFixed(0)}
                      </div>
                      <div className="text-muted-foreground">
                        {p.ratio.toFixed(1)}%
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* 风控提示 */}
          <div className="bg-card rounded-lg p-4 border border-border">
            <div className="flex items-center gap-2 mb-2">
              <AlertTriangle className="w-4 h-4 text-yellow-400" />
              <span className="text-sm font-medium">风控检查</span>
            </div>
            <div className="space-y-1 text-xs">
              <div className="flex justify-between">
                <span className="text-muted-foreground">仓位是否过满</span>
                <span className={positionsRatio > 80 ? 'text-red-400' : 'text-green-400'}>
                  {positionsRatio > 80 ? '⚠️ 过满' : '✅ 正常'}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">是否分散</span>
                <span className={positions.length < 3 ? 'text-yellow-400' : 'text-green-400'}>
                  {positions.length < 3 ? '⚠️ 集中' : '✅ 分散'}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">现金储备</span>
                <span className={cashRatio < 10 ? 'text-red-400' : 'text-green-400'}>
                  {cashRatio < 10 ? '⚠️ 不足' : '✅ 充足'}
                </span>
              </div>
            </div>
          </div>

          {positions.length === 0 && !loading && (
            <div className="text-center text-muted-foreground py-8">
              暂无持仓，快去模拟交易试试吧
            </div>
          )}
        </div>
      </ScrollArea>
    </div>
  );
}
