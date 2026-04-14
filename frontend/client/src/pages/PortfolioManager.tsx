import { useState } from 'react';
import { Plus, Trash2, TrendingUp, Wallet, ArrowRight } from 'lucide-react';
import { usePortfolios, type Portfolio } from '@/hooks/usePortfolio';
import TradeModal from '@/components/portfolio/TradeModal';

export default function PortfolioManager() {
  const { portfolios, loading, refresh, createPortfolio, deletePortfolio } =
    usePortfolios();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showTradeModal, setShowTradeModal] = useState(false);
  const [selectedPortfolio, setSelectedPortfolio] = useState<Portfolio | null>(
    null
  );
  const [createForm, setCreateForm] = useState({
    name: '',
    description: '',
    initial_capital: 1000000,
  });

  const handleCreate = async () => {
    try {
      await createPortfolio({
        name: createForm.name,
        description: createForm.description || undefined,
        initial_capital: createForm.initial_capital,
      });
      setShowCreateModal(false);
      setCreateForm({ name: '', description: '', initial_capital: 1000000 });
    } catch (e: any) {
      alert(e.message);
    }
  };

  const handleDelete = async (id: string, name: string) => {
    if (!confirm(`确定要删除组合 "${name}" 吗？\n所有持仓和交易记录将被清空！`)) {
      return;
    }
    try {
      await deletePortfolio(id);
    } catch (e: any) {
      alert(e.message);
    }
  };

  const formatPercent = (val: number) => {
    return `${val >= 0 ? '+' : ''}${val.toFixed(2)}%`;
  };

  const getReturnColor = (val: number) => {
    return val >= 0 ? 'text-red-500' : 'text-green-500';
  };

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b">
        <div className="flex items-center gap-2">
          <TrendingUp className="w-5 h-5 text-primary" />
          <h1 className="text-lg font-semibold">模拟组合</h1>
          <span className="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded">
            {portfolios.length} 个
          </span>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-1 px-3 py-1.5 bg-primary text-primary-foreground rounded hover:bg-primary/90 transition-colors text-sm"
        >
          <Plus className="w-4 h-4" />
          新建组合
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4">
        {loading ? (
          <div className="flex items-center justify-center py-12">
            <div className="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full" />
          </div>
        ) : portfolios.length === 0 ? (
          <div className="text-center py-12 text-muted-foreground">
            <Wallet className="w-12 h-12 mx-auto mb-3 opacity-30" />
            <p>还没有模拟组合</p>
            <p className="text-sm mt-1">点击上方按钮创建第一个组合</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {portfolios.map((portfolio) => (
              <div
                key={portfolio.id}
                className="bg-card border rounded-lg p-4 hover:border-primary/50 transition-colors"
              >
                {/* 组合名称 */}
                <div className="flex items-start justify-between mb-3">
                  <div>
                    <h3 className="font-semibold">{portfolio.name}</h3>
                    {portfolio.description && (
                      <p className="text-xs text-muted-foreground mt-0.5">
                        {portfolio.description}
                      </p>
                    )}
                  </div>
                  <button
                    onClick={() => handleDelete(portfolio.id, portfolio.name)}
                    className="p-1 text-muted-foreground hover:text-destructive transition-colors"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>

                {/* 收益 */}
                <div className="mb-3">
                  <div className="text-xs text-muted-foreground">总收益</div>
                  <div
                    className={`text-2xl font-bold ${getReturnColor(
                      portfolio.total_return_rate
                    )}`}
                  >
                    {formatPercent(portfolio.total_return_rate)}
                  </div>
                </div>

                {/* 资产信息 */}
                <div className="grid grid-cols-2 gap-3 mb-4 text-sm">
                  <div>
                    <div className="text-xs text-muted-foreground">总资产</div>
                    <div className="font-mono">
                      ¥{(portfolio.total_value / 10000).toFixed(2)}万
                    </div>
                  </div>
                  <div>
                    <div className="text-xs text-muted-foreground">持仓</div>
                    <div className="font-mono">{portfolio.positions_count} 只</div>
                  </div>
                </div>

                {/* 操作按钮 */}
                <div className="flex gap-2">
                  <button
                    onClick={() => {
                      setSelectedPortfolio(portfolio);
                      setShowTradeModal(true);
                    }}
                    className="flex-1 px-3 py-1.5 bg-green-600 text-white rounded text-sm hover:bg-green-700 transition-colors"
                  >
                    买入
                  </button>
                  <a
                    href={`#/portfolios/${portfolio.id}`}
                    className="flex-1 px-3 py-1.5 border rounded text-sm hover:bg-muted transition-colors text-center inline-flex items-center justify-center gap-1"
                  >
                    详情
                    <ArrowRight className="w-3 h-3" />
                  </a>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Create Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
          <div className="bg-card border rounded-lg w-full max-w-sm shadow-2xl">
            <div className="p-4 border-b">
              <h3 className="text-lg font-semibold">创建模拟组合</h3>
            </div>
            <div className="p-4 space-y-4">
              <div>
                <label className="block text-sm text-muted-foreground mb-1">
                  组合名称 *
                </label>
                <input
                  type="text"
                  value={createForm.name}
                  onChange={(e) =>
                    setCreateForm({ ...createForm, name: e.target.value })
                  }
                  placeholder="如: 稳健成长组合"
                  className="w-full px-3 py-2 bg-background border rounded focus:outline-none focus:ring-2 focus:ring-primary"
                />
              </div>
              <div>
                <label className="block text-sm text-muted-foreground mb-1">
                  描述 (可选)
                </label>
                <input
                  type="text"
                  value={createForm.description}
                  onChange={(e) =>
                    setCreateForm({ ...createForm, description: e.target.value })
                  }
                  placeholder="组合策略描述"
                  className="w-full px-3 py-2 bg-background border rounded focus:outline-none focus:ring-2 focus:ring-primary"
                />
              </div>
              <div>
                <label className="block text-sm text-muted-foreground mb-1">
                  初始资金
                </label>
                <input
                  type="number"
                  min="10000"
                  step="10000"
                  value={createForm.initial_capital}
                  onChange={(e) =>
                    setCreateForm({
                      ...createForm,
                      initial_capital: parseInt(e.target.value) || 1000000,
                    })
                  }
                  className="w-full px-3 py-2 bg-background border rounded focus:outline-none focus:ring-2 focus:ring-primary"
                />
              </div>
            </div>
            <div className="p-4 border-t flex gap-3">
              <button
                onClick={() => setShowCreateModal(false)}
                className="flex-1 px-4 py-2 border rounded hover:bg-muted transition-colors"
              >
                取消
              </button>
              <button
                onClick={handleCreate}
                disabled={!createForm.name}
                className="flex-1 px-4 py-2 bg-primary text-primary-foreground rounded hover:bg-primary/90 transition-colors disabled:opacity-50"
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Trade Modal */}
      <TradeModal
        isOpen={showTradeModal}
        onClose={() => {
          setShowTradeModal(false);
          setSelectedPortfolio(null);
        }}
        type="buy"
        portfolioId={selectedPortfolio?.id || ''}
        onSubmit={async (data) => {
          const { buyStock } = await import('@/hooks/usePortfolio');
          const { buyStock: buy } = buyStock();
          await buy(selectedPortfolio!.id, {
            ...data,
            trade_date: new Date().toISOString().split('T')[0],
          });
          refresh();
        }}
      />
    </div>
  );
}
