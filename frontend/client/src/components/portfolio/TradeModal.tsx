import { useState, useEffect } from 'react';
import { X, ArrowUpCircle, ArrowDownCircle } from 'lucide-react';

interface TradeModalProps {
  isOpen: boolean;
  onClose: () => void;
  type: 'buy' | 'sell';
  portfolioId: string;
  defaultSymbol?: string;
  defaultName?: string;
  maxQuantity?: number; // 卖出时的最大持仓
  onSubmit: (data: TradeFormData) => Promise<void>;
}

export interface TradeFormData {
  symbol: string;
  name: string;
  price: number;
  quantity: number;
  reason?: string;
}

export default function TradeModal({
  isOpen,
  onClose,
  type,
  portfolioId,
  defaultSymbol = '',
  defaultName = '',
  maxQuantity,
  onSubmit,
}: TradeModalProps) {
  const [form, setForm] = useState<TradeFormData>({
    symbol: defaultSymbol,
    name: defaultName,
    price: 0,
    quantity: 0,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) {
      setForm({
        symbol: defaultSymbol,
        name: defaultName,
        price: 0,
        quantity: 0,
      });
      setError(null);
    }
  }, [isOpen, defaultSymbol, defaultName]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.symbol || !form.price || !form.quantity) {
      setError('请填写完整信息');
      return;
    }

    if (type === 'sell' && maxQuantity && form.quantity > maxQuantity) {
      setError(`持仓不足，最多可卖出 ${maxQuantity} 股`);
      return;
    }

    setLoading(true);
    try {
      await onSubmit(form);
      onClose();
    } catch (e: any) {
      setError(e.message || '交易失败');
    } finally {
      setLoading(false);
    }
  };

  const amount = form.price * form.quantity;
  const commission = amount * 0.0003;
  const total = type === 'buy' ? amount + commission : amount - commission;

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
      <div className="bg-card border rounded-lg w-full max-w-md shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b">
          <div className="flex items-center gap-2">
            {type === 'buy' ? (
              <ArrowUpCircle className="w-5 h-5 text-green-500" />
            ) : (
              <ArrowDownCircle className="w-5 h-5 text-red-500" />
            )}
            <h3 className="text-lg font-semibold">
              {type === 'buy' ? '买入股票' : '卖出股票'}
            </h3>
          </div>
          <button
            onClick={onClose}
            className="p-1 hover:bg-muted rounded transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-4 space-y-4">
          {error && (
            <div className="bg-destructive/10 text-destructive px-3 py-2 rounded text-sm">
              {error}
            </div>
          )}

          {/* 股票代码 */}
          <div>
            <label className="block text-sm text-muted-foreground mb-1">
              股票代码
            </label>
            <input
              type="text"
              value={form.symbol}
              onChange={(e) =>
                setForm({ ...form, symbol: e.target.value.toUpperCase() })
              }
              placeholder="如: 600519.SH"
              className="w-full px-3 py-2 bg-background border rounded focus:outline-none focus:ring-2 focus:ring-primary"
              disabled={!!defaultSymbol}
            />
          </div>

          {/* 股票名称 */}
          <div>
            <label className="block text-sm text-muted-foreground mb-1">
              股票名称
            </label>
            <input
              type="text"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
              placeholder="如: 贵州茅台"
              className="w-full px-3 py-2 bg-background border rounded focus:outline-none focus:ring-2 focus:ring-primary"
              disabled={!!defaultName}
            />
          </div>

          {/* 价格 */}
          <div>
            <label className="block text-sm text-muted-foreground mb-1">
              价格 (元)
            </label>
            <input
              type="number"
              step="0.01"
              min="0"
              value={form.price || ''}
              onChange={(e) =>
                setForm({ ...form, price: parseFloat(e.target.value) || 0 })
              }
              className="w-full px-3 py-2 bg-background border rounded focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>

          {/* 数量 */}
          <div>
            <label className="block text-sm text-muted-foreground mb-1">
              数量 (股)
              {type === 'sell' && maxQuantity !== undefined && (
                <span className="text-muted-foreground ml-2">
                  (可卖: {maxQuantity})
                </span>
              )}
            </label>
            <input
              type="number"
              min="1"
              step="1"
              value={form.quantity || ''}
              onChange={(e) =>
                setForm({ ...form, quantity: parseInt(e.target.value) || 0 })
              }
              className="w-full px-3 py-2 bg-background border rounded focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>

          {/* 交易理由 */}
          <div>
            <label className="block text-sm text-muted-foreground mb-1">
              交易理由 (可选)
            </label>
            <input
              type="text"
              value={form.reason || ''}
              onChange={(e) => setForm({ ...form, reason: e.target.value })}
              placeholder="如: 突破买入 / 止盈减仓"
              className="w-full px-3 py-2 bg-background border rounded focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>

          {/* 交易汇总 */}
          {amount > 0 && (
            <div className="bg-muted/50 p-3 rounded space-y-1 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">成交金额</span>
                <span className="font-mono">¥{amount.toFixed(2)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">手续费 (万3)</span>
                <span className="font-mono">¥{commission.toFixed(2)}</span>
              </div>
              <div className="flex justify-between pt-1 border-t">
                <span className="text-muted-foreground">
                  {type === 'buy' ? '总支出' : '净收入'}
                </span>
                <span
                  className={`font-mono font-medium ${
                    type === 'buy' ? 'text-red-500' : 'text-green-500'
                  }`}
                >
                  ¥{Math.abs(total).toFixed(2)}
                </span>
              </div>
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-3 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 border rounded hover:bg-muted transition-colors"
            >
              取消
            </button>
            <button
              type="submit"
              disabled={loading}
              className={`flex-1 px-4 py-2 rounded text-white transition-colors ${
                type === 'buy'
                  ? 'bg-green-600 hover:bg-green-700'
                  : 'bg-red-600 hover:bg-red-700'
              } disabled:opacity-50`}
            >
              {loading ? '处理中...' : type === 'buy' ? '确认买入' : '确认卖出'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
