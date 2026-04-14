import { ArrowUpCircle, ArrowDownCircle } from 'lucide-react';
import type { PortfolioTrade } from '@/hooks/usePortfolio';

interface TradeHistoryProps {
  trades: PortfolioTrade[];
  loading?: boolean;
}

export default function TradeHistory({ trades, loading }: TradeHistoryProps) {
  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="animate-spin w-6 h-6 border-2 border-primary border-t-transparent rounded-full" />
      </div>
    );
  }

  if (trades.length === 0) {
    return (
      <div className="text-center py-8 text-muted-foreground">
        暂无调仓记录
      </div>
    );
  }

  const formatDate = (date: string, time: string) => {
    return `${date} ${time}`;
  };

  return (
    <div className="space-y-2">
      {trades.map((trade) => (
        <div
          key={trade.id}
          className="bg-card border rounded-lg p-3 hover:bg-muted/30 transition-colors"
        >
          <div className="flex items-start justify-between">
            {/* 左侧：股票信息 */}
            <div className="flex items-center gap-3">
              {trade.direction === 'buy' ? (
                <ArrowUpCircle className="w-5 h-5 text-green-500" />
              ) : (
                <ArrowDownCircle className="w-5 h-5 text-red-500" />
              )}
              <div>
                <div className="font-medium">{trade.name}</div>
                <div className="text-xs text-muted-foreground">
                  {trade.symbol}
                </div>
              </div>
            </div>

            {/* 右侧：交易信息 */}
            <div className="text-right">
              <div
                className={`font-medium ${
                  trade.direction === 'buy' ? 'text-green-500' : 'text-red-500'
                }`}
              >
                {trade.direction === 'buy' ? '买入' : '卖出'}
              </div>
              <div className="text-xs text-muted-foreground">
                {formatDate(trade.trade_date, trade.trade_time)}
              </div>
            </div>
          </div>

          {/* 交易详情 */}
          <div className="mt-3 grid grid-cols-3 gap-4 text-sm">
            <div>
              <div className="text-xs text-muted-foreground">成交价格</div>
              <div className="font-mono">¥{trade.price.toFixed(2)}</div>
            </div>
            <div>
              <div className="text-xs text-muted-foreground">成交数量</div>
              <div className="font-mono">{trade.quantity.toFixed(0)}</div>
            </div>
            <div>
              <div className="text-xs text-muted-foreground">成交金额</div>
              <div className="font-mono">¥{trade.amount.toFixed(2)}</div>
            </div>
          </div>

          {/* 仓位变化 */}
          {(trade.position_before !== undefined &&
            trade.position_after !== undefined) && (
            <div className="mt-2 text-xs text-muted-foreground">
              仓位: {trade.position_before.toFixed(0)} →{' '}
              {trade.position_after.toFixed(0)}
              {trade.weight_before !== undefined &&
                trade.weight_after !== undefined && (
                  <span className="ml-2">
                    ({trade.weight_before.toFixed(1)}% →{' '}
                    {trade.weight_after.toFixed(1)}%)
                  </span>
                )}
            </div>
          )}

          {/* 交易理由 */}
          {trade.reason && (
            <div className="mt-2 text-xs text-muted-foreground bg-muted/50 p-2 rounded">
              理由: {trade.reason}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
