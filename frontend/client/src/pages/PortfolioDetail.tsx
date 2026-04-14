import { useState, useEffect } from 'react';
import { useParams, useLocation } from 'wouter';
import {
  ArrowLeft,
  TrendingUp,
  TrendingDown,
  Wallet,
  Package,
  History,
  BarChart3,
  Plus,
  Minus,
  Activity,
  Target,
  Award,
  AlertTriangle,
} from 'lucide-react';
import {
  usePortfolioDetail,
  usePortfolioPositions,
  usePortfolioTrades,
  usePortfolioStats,
  useBuyStock,
  useSellStock,
  useMonthlyReturns,
  useAnnualReturns,
} from '@/hooks/usePortfolio';
import TradeModal from '@/components/portfolio/TradeModal';
import TradeHistory from '@/components/portfolio/TradeHistory';

export default function PortfolioDetail() {
  const params = useParams();
  const [, navigate] = useLocation();
  const portfolioId = params.id || '';

  const { portfolio, refresh: refreshPortfolio } =
    usePortfolioDetail(portfolioId);
  const { positions, refresh: refreshPositions } =
    usePortfolioPositions(portfolioId);
  const { trades, refresh: refreshTrades } = usePortfolioTrades(portfolioId);
  const { stats, refresh: refreshStats } = usePortfolioStats(portfolioId);
  const { buyStock } = useBuyStock();
  const { sellStock } = useSellStock();
  const { monthlyReturns } = useMonthlyReturns(portfolioId);
  const { annualReturns } = useAnnualReturns(portfolioId);

  const [activeTab, setActiveTab] = useState<'positions' | 'trades' | 'stats'>(
    'positions'
  );
  const [tradeModal, setTradeModal] = useState<{
    open: boolean;
    type: 'buy' | 'sell';
    position?: (typeof positions)[0];
  }>({ open: false, type: 'buy' });

  // 自动刷新数据
  useEffect(() => {
    const interval = setInterval(() => {
      refreshPortfolio();
      refreshPositions();
      refreshStats();
    }, 30000);
    return () => clearInterval(interval);
  }, [refreshPortfolio, refreshPositions, refreshStats]);

  const handleBuy = async (data: {
    symbol: string;
    name: string;
    price: number;
    quantity: number;
    reason?: string;
  }) => {
    await buyStock(portfolioId, {
      ...data,
      trade_date: new Date().toISOString().split('T')[0],
    });
    refreshAll();
  };

  const handleSell = async (data: {
    symbol: string;
    name: string;
    price: number;
    quantity: number;
    reason?: string;
  }) => {
    await sellStock(portfolioId, {
      symbol: data.symbol,
      price: data.price,
      quantity: data.quantity,
      reason: data.reason,
      trade_date: new Date().toISOString().split('T')[0],
    });
    refreshAll();
  };

  const refreshAll = () => {
    refreshPortfolio();
    refreshPositions();
    refreshTrades();
    refreshStats();
  };

  const formatPercent = (val: number) => {
    return `${val >= 0 ? '+' : ''}${val.toFixed(2)}%`;
  };

  const getReturnColor = (val: number) => {
    return val >= 0 ? 'text-red-500' : 'text-green-500';
  };

  if (!portfolio) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full" />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center gap-3 px-4 py-3 border-b">
        <button
          onClick={() => navigate('/portfolios')}
          className="p-1.5 hover:bg-muted rounded transition-colors"
        >
          <ArrowLeft className="w-5 h-5" />
        </button>
        <div className="flex-1">
          <h1 className="text-lg font-semibold">{portfolio.name}</h1>
          <p className="text-xs text-muted-foreground">
            创建于 {new Date(portfolio.created_at).toLocaleDateString()}
          </p>
        </div>
        <div className="text-right">
          <div className={`text-xl font-bold ${getReturnColor(portfolio.total_return_rate)}`}>
            {formatPercent(portfolio.total_return_rate)}
          </div>
          <div className="text-xs text-muted-foreground">总收益</div>
        </div>
      </div>

      {/* 资产概览 */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 p-4 border-b bg-muted/20">
        <div>
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground mb-1">
            <Wallet className="w-3.5 h-3.5" />
            总资产
          </div>
          <div className="text-lg font-mono font-medium">
            ¥{portfolio.total_value.toLocaleString()}
          </div>
        </div>
        <div>
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground mb-1">
            <Package className="w-3.5 h-3.5" />
            持仓市值
          </div>
          <div className="text-lg font-mono font-medium">
            ¥{portfolio.positions_value.toLocaleString()}
          </div>
        </div>
        <div>
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground mb-1">
            <TrendingUp className="w-3.5 h-3.5" />
            可用现金
          </div>
          <div className="text-lg font-mono font-medium">
            ¥{portfolio.current_capital.toLocaleString()}
          </div>
        </div>
        <div>
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground mb-1">
            <BarChart3 className="w-3.5 h-3.5" />
            持仓数量
          </div>
          <div className="text-lg font-mono font-medium">
            {portfolio.positions_count} 只
          </div>
        </div>
      </div>

      {/* 快捷操作 */}
      <div className="flex gap-2 p-4 border-b">
        <button
          onClick={() => setTradeModal({ open: true, type: 'buy' })}
          className="flex-1 flex items-center justify-center gap-2 py-2 bg-green-600 text-white rounded hover:bg-green-700 transition-colors"
        >
          <Plus className="w-4 h-4" />
          买入股票
        </button>
        <button
          onClick={() => setTradeModal({ open: true, type: 'sell' })}
          className="flex-1 flex items-center justify-center gap-2 py-2 bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
        >
          <Minus className="w-4 h-4" />
          卖出股票
        </button>
      </div>

      {/* Tab 切换 */}
      <div className="flex border-b">
        {[
          { key: 'positions', label: '持仓', icon: Package },
          { key: 'trades', label: '调仓记录', icon: History },
          { key: 'stats', label: '统计', icon: BarChart3 },
        ].map(({ key, label, icon: Icon }) => (
          <button
            key={key}
            onClick={() => setActiveTab(key as any)}
            className={`flex-1 flex items-center justify-center gap-2 py-3 text-sm font-medium transition-colors ${
              activeTab === key
                ? 'border-b-2 border-primary text-primary'
                : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            <Icon className="w-4 h-4" />
            {label}
          </button>
        ))}
      </div>

      {/* 内容区域 */}
      <div className="flex-1 overflow-y-auto p-4">
        {activeTab === 'positions' && (
          <div className="space-y-2">
            {positions.length === 0 ? (
              <div className="text-center py-12 text-muted-foreground">
                <Package className="w-12 h-12 mx-auto mb-3 opacity-30" />
                <p>暂无持仓</p>
                <p className="text-sm mt-1">点击上方买入按钮添加股票</p>
              </div>
            ) : (
              positions.map((pos) => (
                <div
                  key={pos.symbol}
                  className="bg-card border rounded-lg p-4 hover:bg-muted/30 transition-colors"
                >
                  <div className="flex items-start justify-between mb-3">
                    <div>
                      <div className="font-semibold">{pos.name}</div>
                      <div className="text-xs text-muted-foreground">
                        {pos.symbol}
                      </div>
                    </div>
                    <div className="text-right">
                      <div
                        className={`font-medium ${getReturnColor(
                          pos.profit_rate
                        )}`}
                      >
                        {formatPercent(pos.profit_rate)}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        ¥{pos.total_profit.toFixed(2)}
                      </div>
                    </div>
                  </div>

                  <div className="grid grid-cols-4 gap-3 text-sm">
                    <div>
                      <div className="text-xs text-muted-foreground">持仓</div>
                      <div className="font-mono">{pos.quantity.toFixed(0)}</div>
                    </div>
                    <div>
                      <div className="text-xs text-muted-foreground">成本</div>
                      <div className="font-mono">¥{pos.avg_cost.toFixed(2)}</div>
                    </div>
                    <div>
                      <div className="text-xs text-muted-foreground">现价</div>
                      <div className="font-mono">
                        ¥{pos.current_price.toFixed(2)}
                      </div>
                    </div>
                    <div>
                      <div className="text-xs text-muted-foreground">市值</div>
                      <div className="font-mono">
                        ¥{pos.market_value.toFixed(0)}
                      </div>
                    </div>
                  </div>

                  <div className="mt-3 flex items-center justify-between">
                    <div className="text-xs text-muted-foreground">
                      权重: {pos.weight.toFixed(2)}%
                    </div>
                    <button
                      onClick={() =>
                        setTradeModal({ open: true, type: 'sell', position: pos })
                      }
                      className="px-3 py-1 text-xs bg-red-100 text-red-700 rounded hover:bg-red-200 transition-colors"
                    >
                      卖出
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === 'trades' && (
          <TradeHistory trades={trades} />
        )}

        {activeTab === 'stats' && stats && (
          <div className="space-y-4">
            {/* 收益概览 */}
            <div className="bg-card border rounded-lg p-4">
              <div className="flex items-center gap-2 mb-3">
                <TrendingUp className="w-4 h-4 text-primary" />
                <h3 className="font-medium">收益概览</h3>
              </div>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                <div className="bg-muted/50 p-3 rounded">
                  <div className="text-xs text-muted-foreground">总收益</div>
                  <div className={`text-lg font-bold ${getReturnColor(stats.total_return_rate)}`}>
                    {formatPercent(stats.total_return_rate)}
                  </div>
                  <div className="text-xs text-muted-foreground">¥{stats.total_return.toFixed(2)}</div>
                </div>
                <div className="bg-muted/50 p-3 rounded">
                  <div className="text-xs text-muted-foreground">年化收益</div>
                  <div className={`text-lg font-bold ${getReturnColor(stats.annualized_return)}`}>
                    {formatPercent(stats.annualized_return)}
                  </div>
                </div>
                <div className="bg-muted/50 p-3 rounded">
                  <div className="text-xs text-muted-foreground">日收益</div>
                  <div className={`text-lg font-bold ${getReturnColor(stats.daily_return_rate)}`}>
                    {formatPercent(stats.daily_return_rate)}
                  </div>
                </div>
                <div className="bg-muted/50 p-3 rounded">
                  <div className="text-xs text-muted-foreground">基准收益</div>
                  <div className="text-lg font-bold text-blue-500">
                    {formatPercent(stats.benchmark_return)}
                  </div>
                </div>
              </div>
            </div>

            {/* 风险指标 */}
            <div className="bg-card border rounded-lg p-4">
              <div className="flex items-center gap-2 mb-3">
                <AlertTriangle className="w-4 h-4 text-yellow-500" />
                <h3 className="font-medium">风险指标</h3>
              </div>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                <div className="bg-muted/50 p-3 rounded">
                  <div className="text-xs text-muted-foreground">最大回撤</div>
                  <div className="text-lg font-bold text-red-500">
                    {stats.max_drawdown.toFixed(2)}%
                  </div>
                </div>
                <div className="bg-muted/50 p-3 rounded">
                  <div className="text-xs text-muted-foreground">波动率</div>
                  <div className="text-lg font-bold">
                    {stats.volatility.toFixed(2)}%
                  </div>
                </div>
                <div className="bg-muted/50 p-3 rounded">
                  <div className="text-xs text-muted-foreground">夏普比率</div>
                  <div className={`text-lg font-bold ${stats.sharpe_ratio >= 1 ? 'text-green-500' : 'text-yellow-500'}`}>
                    {stats.sharpe_ratio.toFixed(2)}
                  </div>
                </div>
                <div className="bg-muted/50 p-3 rounded">
                  <div className="text-xs text-muted-foreground">Beta</div>
                  <div className="text-lg font-bold">
                    {stats.beta.toFixed(2)}
                  </div>
                </div>
              </div>
              <div className="mt-3 pt-3 border-t">
                <div className="flex justify-between items-center">
                  <span className="text-xs text-muted-foreground">Alpha (超额收益)</span>
                  <span className={`text-sm font-medium ${getReturnColor(stats.alpha)}`}>
                    {formatPercent(stats.alpha)}
                  </span>
                </div>
                <div className="flex justify-between items-center mt-1">
                  <span className="text-xs text-muted-foreground">持仓集中度</span>
                  <span className={`text-sm font-medium ${stats.position_concentration > 30 ? 'text-yellow-500' : 'text-green-500'}`}>
                    {stats.position_concentration.toFixed(1)}%
                  </span>
                </div>
              </div>
            </div>

            {/* 交易统计 */}
            <div className="bg-card border rounded-lg p-4">
              <div className="flex items-center gap-2 mb-3">
                <Activity className="w-4 h-4 text-green-500" />
                <h3 className="font-medium">交易统计</h3>
              </div>
              <div className="grid grid-cols-3 gap-3">
                <div className="text-center bg-muted/50 p-3 rounded">
                  <div className="text-2xl font-bold">{stats.total_trades}</div>
                  <div className="text-xs text-muted-foreground">总交易</div>
                </div>
                <div className="text-center bg-muted/50 p-3 rounded">
                  <div className="text-2xl font-bold text-green-500">{stats.win_trades}</div>
                  <div className="text-xs text-muted-foreground">盈利</div>
                </div>
                <div className="text-center bg-muted/50 p-3 rounded">
                  <div className="text-2xl font-bold text-red-500">{stats.loss_trades}</div>
                  <div className="text-xs text-muted-foreground">亏损</div>
                </div>
              </div>
              <div className="mt-3 grid grid-cols-3 gap-2 text-center">
                <div>
                  <div className="text-lg font-bold text-green-500">{stats.win_rate.toFixed(1)}%</div>
                  <div className="text-xs text-muted-foreground">胜率</div>
                </div>
                <div>
                  <div className="text-lg font-bold text-blue-500">{stats.win_streak}</div>
                  <div className="text-xs text-muted-foreground">连胜</div>
                </div>
                <div>
                  <div className="text-lg font-bold text-red-500">{stats.lose_streak}</div>
                  <div className="text-xs text-muted-foreground">连亏</div>
                </div>
              </div>
            </div>

            {/* 月度收益 */}
            {monthlyReturns.length > 0 && (
              <div className="bg-card border rounded-lg p-4">
                <div className="flex items-center gap-2 mb-3">
                  <BarChart3 className="w-4 h-4 text-purple-500" />
                  <h3 className="font-medium">月度收益</h3>
                </div>
                <div className="overflow-x-auto">
                  <table className="w-full text-xs">
                    <thead>
                      <tr className="text-muted-foreground border-b">
                        <th className="text-left py-1">月份</th>
                        <th className="text-right py-1">期初</th>
                        <th className="text-right py-1">期末</th>
                        <th className="text-right py-1">收益</th>
                        <th className="text-right py-1">基准</th>
                      </tr>
                    </thead>
                    <tbody>
                      {monthlyReturns.slice(-6).reverse().map((m) => (
                        <tr key={`${m.year}-${m.month}`} className="border-b border-muted/30">
                          <td className="py-1.5">{m.year}-{String(m.month).padStart(2, '0')}</td>
                          <td className="text-right font-mono">¥{m.start_value.toFixed(0)}</td>
                          <td className="text-right font-mono">¥{m.end_value.toFixed(0)}</td>
                          <td className={`text-right font-mono font-medium ${getReturnColor(m.monthly_return_rate)}`}>
                            {formatPercent(m.monthly_return_rate)}
                          </td>
                          <td className="text-right font-mono text-blue-400">
                            {formatPercent(m.benchmark_return)}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            )}

            {/* 年度收益 */}
            {annualReturns.length > 0 && (
              <div className="bg-card border rounded-lg p-4">
                <div className="flex items-center gap-2 mb-3">
                  <Award className="w-4 h-4 text-yellow-500" />
                  <h3 className="font-medium">年度收益</h3>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                  {annualReturns.map((a) => (
                    <div key={a.year} className="flex items-center justify-between bg-muted/50 p-3 rounded">
                      <div className="flex items-center gap-3">
                        <span className="font-bold">{a.year}</span>
                        <span className="text-xs text-muted-foreground">{a.trades_count}笔交易</span>
                      </div>
                      <div className="text-right">
                        <div className={`text-lg font-bold ${getReturnColor(a.annual_return_rate)}`}>
                          {formatPercent(a.annual_return_rate)}
                        </div>
                        <div className="text-xs text-muted-foreground">
                          基准 {formatPercent(a.benchmark_return)}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Trade Modal */}
      <TradeModal
        isOpen={tradeModal.open}
        onClose={() => setTradeModal({ open: false, type: 'buy' })}
        type={tradeModal.type}
        portfolioId={portfolioId}
        defaultSymbol={tradeModal.position?.symbol}
        defaultName={tradeModal.position?.name}
        maxQuantity={
          tradeModal.type === 'sell' ? tradeModal.position?.quantity : undefined
        }
        onSubmit={tradeModal.type === 'buy' ? handleBuy : handleSell}
      />
    </div>
  );
}
