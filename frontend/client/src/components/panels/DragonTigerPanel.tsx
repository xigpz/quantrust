/**
 * DragonTigerPanel - 龙虎榜面板
 */
import { useState } from 'react';
import { TrendingUp, TrendingDown, RefreshCw, Award, Users } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { useStockClick } from '@/pages/Dashboard';

interface DragonTigerData {
  symbol: string;
  name: string;
  trade_date: string;
  close_price: number;
  change_pct: number;
  turnover: number;
  buy_amount: number;
  sell_amount: number;
  net_amount: number;
  reason: string;
  buy_seats: { name: string; amount: number }[];
  sell_seats: { name: string; amount: number }[];
}

function formatMoney(num: number): string {
  if (Math.abs(num) >= 10000) {
    return (num / 10000).toFixed(1) + '亿';
  }
  return num.toFixed(0) + '万';
}

export default function DragonTigerPanel() {
  const [data, setData] = useState<DragonTigerData[]>([]);
  const { openStock } = useStockClick();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch('/api/dragon-tiger');
      const json = await res.json();
      if (json.success) {
        setData(json.data);
      } else {
        setError(json.message);
      }
    } catch (e) {
      setError('加载失败');
    }
    setLoading(false);
  };

  // 初始加载
  useState(() => {
    loadData();
  });

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border">
        <div className="flex items-center gap-2">
          <Award className="w-4 h-4 text-yellow-400" />
          <h2 className="text-sm font-semibold">龙虎榜</h2>
        </div>
        <button onClick={loadData} className="text-muted-foreground hover:text-foreground transition-colors">
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      <ScrollArea className="flex-1">
        {error ? (
          <div className="p-4 text-center text-muted-foreground text-sm">
            {error}
          </div>
        ) : data.length === 0 ? (
          <div className="p-4 text-center text-muted-foreground text-sm">
            {loading ? '加载中...' : '点击刷新按钮加载数据'}
          </div>
        ) : (
          <div className="p-2">
            <table className="w-full text-xs">
              <thead className="sticky top-0 bg-card z-10">
                <tr className="text-muted-foreground border-b border-border">
                  <th className="text-left py-2 px-2 font-medium">股票</th>
                  <th className="text-right py-2 px-2 font-medium">现价</th>
                  <th className="text-right py-2 px-2 font-medium">涨跌幅</th>
                  <th className="text-right py-2 px-2 font-medium">净买入</th>
                </tr>
              </thead>
              <tbody>
                {data.slice(0, 30).map((item, idx) => (
                  <tr key={idx} onClick={() => openStock(item.symbol, item.name)} className="border-b border-border/50 hover:bg-accent/50 transition-colors cursor-pointer">
                    <td className="py-2 px-2">
                      <div className="font-medium">{item.name}</div>
                      <div className="text-muted-foreground text-xs">{item.symbol}</div>
                    </td>
                    <td className="text-right py-2 px-2 font-mono">
                      {item.close_price > 0 ? item.close_price.toFixed(2) : '—'}
                    </td>
                    <td className={`text-right py-2 px-2 font-mono ${item.change_pct >= 0 ? 'text-up' : 'text-down'}`}>
                      {item.change_pct !== 0 ? `${item.change_pct > 0 ? '+' : ''}${item.change_pct.toFixed(2)}%` : '—'}
                    </td>
                    <td className={`text-right py-2 px-2 font-mono ${item.net_amount >= 0 ? 'text-up' : 'text-down'}`}>
                      {item.net_amount !== 0 ? `${item.net_amount > 0 ? '+' : ''}${formatMoney(item.net_amount)}` : '—'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            
            {data.length > 30 && (
              <div className="text-center text-xs text-muted-foreground py-2">
                显示前30条，共{data.length}条
              </div>
            )}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
