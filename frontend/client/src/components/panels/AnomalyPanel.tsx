import { useState, useEffect, useContext } from 'react';
import { StockClickContext } from '@/pages/Dashboard';
import { TrendingUp, TrendingDown, AlertTriangle, Activity } from 'lucide-react';

const API_BASE = '';

interface AnomalyPrediction {
  symbol: string;
  name: string;
  change_pct?: number;
  pred_type: string;
  sentiment: {
    score: number;
    label: string;
    keywords: string[];
  };
  urgency: string;
  timestamp: string;
  reason: string;
}

export default function AnomalyPanel() {
  const { openStock } = useContext(StockClickContext);
  const [predictions, setPredictions] = useState<AnomalyPrediction[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<'all' | 'high'>('all');

  const fetchPredictions = async () => {
    setLoading(true);
    try {
      const res = await fetch(`${API_BASE}/api/anomaly/predictions`).then(r => r.json());
      if (res.success) {
        setPredictions(res.data);
      }
    } catch (e) {
      console.error(e);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchPredictions();
    const interval = setInterval(fetchPredictions, 30000);
    return () => clearInterval(interval);
  }, []);

  const filtered = filter === 'high' ? predictions.filter(p => p.urgency === '高') : predictions;

  const getUrgencyColor = (u: string) => {
    if (u === '高') return 'bg-red-500/20 text-red-400 border-red-500/30';
    if (u === '中') return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30';
    return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
  };

  const getTypeIcon = (type_: string) => {
    if (type_.includes('涨停')) return <TrendingUp className="w-4 h-4 text-red-400" />;
    if (type_.includes('风险')) return <AlertTriangle className="w-4 h-4 text-red-400" />;
    return <Activity className="w-4 h-4 text-yellow-400" />;
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">异动预测</h2>
        <div className="flex gap-2">
          <button
            onClick={() => setFilter('all')}
            className={`px-3 py-1 text-xs rounded ${filter === 'all' ? 'bg-primary text-primary-foreground' : 'bg-muted'}`}
          >
            全部
          </button>
          <button
            onClick={() => setFilter('high')}
            className={`px-3 py-1 text-xs rounded ${filter === 'high' ? 'bg-red-600 text-white' : 'bg-muted'}`}
          >
            仅高紧急
          </button>
        </div>
      </div>

      {loading ? (
        <div className="text-center py-8 text-muted-foreground">加载中...</div>
      ) : filtered.length === 0 ? (
        <div className="text-center py-8 text-muted-foreground">暂无异动预测</div>
      ) : (
        <div className="space-y-2">
          {filtered.map((pred, i) => (
            <div
              key={`${pred.symbol}-${i}`}
              onClick={() => openStock(pred.symbol, pred.name)}
              className="bg-card rounded-lg p-4 border hover:border-primary/50 cursor-pointer transition-colors"
            >
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-3">
                  {getTypeIcon(pred.pred_type)}
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-semibold">{pred.symbol}</span>
                      <span className="text-sm text-muted-foreground">{pred.name}</span>
                      {pred.change_pct !== undefined && (
                        <span className={`text-sm font-medium ${pred.change_pct >= 0 ? 'text-red-400' : 'text-green-400'}`}>
                          {pred.change_pct >= 0 ? '+' : ''}{pred.change_pct.toFixed(2)}%
                        </span>
                      )}
                    </div>
                    <div className="text-xs text-muted-foreground mt-1">{pred.reason}</div>
                  </div>
                </div>
                <div className="text-right">
                  <span className={`px-2 py-1 text-xs rounded border ${getUrgencyColor(pred.urgency)}`}>
                    {pred.urgency}
                  </span>
                  <div className="text-xs text-muted-foreground mt-1">{pred.pred_type}</div>
                </div>
              </div>
              {pred.sentiment.keywords.length > 0 && (
                <div className="flex gap-1 mt-2 flex-wrap">
                  {pred.sentiment.keywords.map((kw, j) => (
                    <span key={j} className="px-2 py-0.5 bg-muted text-xs rounded">
                      {kw}
                    </span>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
