/**
 * BigOrderMonitorPanel - 大单监控面板
 * 扫大单抢筹、大单异动提醒、板块大单排名
 */
import { useState, useEffect } from 'react';
import {
  Search,
  RefreshCw,
  AlertTriangle,
  TrendingUp,
  ArrowUpRight,
  ArrowDownRight,
  Clock,
  CheckCircle,
  X,
  Filter,
} from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { toast } from 'sonner';
import { API_BASE } from '@/hooks/useMarketData';

interface BigOrderStock {
  symbol: string;
  name: string;
  price: number;
  change_pct: number;
  huge_inflow: number;      // 超大单净流入(万)
  large_inflow: number;     // 大单净流入(万)
  total_main_inflow: number;
  inflow_ratio: number;
  reason: string;
}

interface BigOrderAlert {
  id: string;
  symbol: string;
  name: string;
  alert_type: string;
  threshold: number;
  actual: number;
  message: string;
  timestamp: string;
  acknowledged: boolean;
}

interface SectorBigOrder {
  sector_code: string;
  sector_name: string;
  stock_count: number;
  huge_inflow_total: number;
  large_inflow_total: number;
  main_inflow_total: number;
  avg_inflow_per_stock: number;
  hot_stocks: string[];
}

type TabType = 'scan' | 'surge' | 'sector';

export default function BigOrderMonitorPanel() {
  const [activeTab, setActiveTab] = useState<TabType>('scan');
  const [scanStocks, setScanStocks] = useState<BigOrderStock[]>([]);
  const [alerts, setAlerts] = useState<BigOrderAlert[]>([]);
  const [sectors, setSectors] = useState<SectorBigOrder[]>([]);
  const [loading, setLoading] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<string>('');

  // 加载大单抢筹数据
  const loadBigOrderStocks = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/big-order/scan`);
      const data = await res.json();
      if (data.success) {
        setScanStocks(data.data || []);
        setLastUpdate(new Date().toLocaleTimeString());
      }
    } catch (e) {
      console.error('加载大单抢筹失败', e);
    }
  };

  // 加载大单异动
  const loadSurgeAlerts = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/big-order/surge`);
      const data = await res.json();
      if (data.success) {
        setAlerts(data.data || []);
      }
    } catch (e) {
      console.error('加载大单异动失败', e);
    }
  };

  // 加载板块排名
  const loadSectorRanking = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/big-order/sector-ranking`);
      const data = await res.json();
      if (data.success) {
        setSectors(data.data || []);
      }
    } catch (e) {
      console.error('加载板块排名失败', e);
    }
  };

  // 确认预警
  const acknowledgeAlert = async (id: string) => {
    try {
      const res = await fetch(`${API_BASE}/api/big-order/alerts/${id}`, {
        method: 'PUT',
      });
      const data = await res.json();
      if (data.success) {
        setAlerts(prev => prev.map(a => a.id === id ? { ...a, acknowledged: true } : a));
        toast.success('已确认预警');
      }
    } catch (e) {
      toast.error('确认失败');
    }
  };

  // 刷新数据
  const refreshData = async () => {
    setLoading(true);
    try {
      await Promise.all([
        loadBigOrderStocks(),
        loadSurgeAlerts(),
        loadSectorRanking(),
      ]);
    } finally {
      setLoading(false);
    }
  };

  // 初始加载和定时刷新
  useEffect(() => {
    refreshData();
    const interval = setInterval(refreshData, 30000); // 30秒刷新
    return () => clearInterval(interval);
  }, []);

  // 格式化金额
  const formatMoney = (value: number) => {
    if (Math.abs(value) >= 10000) {
      return `${(value / 10000).toFixed(1)}亿`;
    }
    return `${value.toFixed(0)}万`;
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border">
        <div className="flex items-center gap-2">
          <TrendingUp className="w-4 h-4 text-primary" />
          <h2 className="text-sm font-semibold">大单监控</h2>
          {lastUpdate && (
            <span className="text-[10px] text-muted-foreground">
              {lastUpdate}
            </span>
          )}
        </div>
        <button
          onClick={refreshData}
          disabled={loading}
          className="text-muted-foreground hover:text-foreground transition-colors"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {/* Tab 切换 */}
      <div className="flex border-b border-border">
        <button
          onClick={() => setActiveTab('scan')}
          className={`flex-1 px-4 py-2 text-xs font-medium transition-colors ${
            activeTab === 'scan'
              ? 'text-primary border-b-2 border-primary'
              : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          <div className="flex items-center justify-center gap-1.5">
            <ArrowUpRight className="w-3.5 h-3.5" />
            大单抢筹
          </div>
        </button>
        <button
          onClick={() => setActiveTab('surge')}
          className={`flex-1 px-4 py-2 text-xs font-medium transition-colors ${
            activeTab === 'surge'
              ? 'text-primary border-b-2 border-primary'
              : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          <div className="flex items-center justify-center gap-1.5">
            <AlertTriangle className="w-3.5 h-3.5" />
            大单异动
            {alerts.filter(a => !a.acknowledged).length > 0 && (
              <span className="px-1.5 py-0.5 text-[10px] bg-red-500 text-white rounded-full">
                {alerts.filter(a => !a.acknowledged).length}
              </span>
            )}
          </div>
        </button>
        <button
          onClick={() => setActiveTab('sector')}
          className={`flex-1 px-4 py-2 text-xs font-medium transition-colors ${
            activeTab === 'sector'
              ? 'text-primary border-b-2 border-primary'
              : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          <div className="flex items-center justify-center gap-1.5">
            <Filter className="w-3.5 h-3.5" />
            板块大单
          </div>
        </button>
      </div>

      {/* Content */}
      <ScrollArea className="flex-1">
        {activeTab === 'scan' && (
          <div className="p-3 space-y-2">
            {scanStocks.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                <TrendingUp className="w-8 h-8 mx-auto mb-2 opacity-30" />
                <p className="text-sm">暂无大单抢筹数据</p>
              </div>
            ) : (
              scanStocks.slice(0, 30).map((stock, index) => (
                <div
                  key={`${stock.symbol}-${index}`}
                  className="p-3 bg-card border border-border rounded-lg hover:border-primary/50 transition-colors"
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium text-sm">{stock.name}</span>
                        <span className="text-xs text-muted-foreground">{stock.symbol}</span>
                      </div>
                      <div className="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
                        <span>现价: {stock.price.toFixed(2)}</span>
                        <span className={stock.change_pct >= 0 ? 'text-red-500' : 'text-green-500'}>
                          {stock.change_pct >= 0 ? '+' : ''}{stock.change_pct.toFixed(2)}%
                        </span>
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="text-xs text-muted-foreground">超大单</div>
                      <div className="text-sm font-medium text-red-500">
                        +{formatMoney(stock.huge_inflow)}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-4 mt-2 pt-2 border-t border-border/50 text-xs">
                    <div>
                      <span className="text-muted-foreground">大单: </span>
                      <span className="text-orange-500">+{formatMoney(stock.large_inflow)}</span>
                    </div>
                    <div>
                      <span className="text-muted-foreground">主力: </span>
                      <span className={stock.total_main_inflow >= 0 ? 'text-red-500' : 'text-green-500'}>
                        {stock.total_main_inflow >= 0 ? '+' : ''}{formatMoney(stock.total_main_inflow)}
                      </span>
                    </div>
                    <div>
                      <span className="text-muted-foreground">占比: </span>
                      <span>{stock.inflow_ratio.toFixed(1)}%</span>
                    </div>
                  </div>
                  <div className="mt-2">
                    <span className={`inline-block px-2 py-0.5 text-[10px] rounded ${
                      stock.reason.includes('联合拉升') ? 'bg-red-500/20 text-red-500' :
                      stock.reason.includes('吸筹') ? 'bg-orange-500/20 text-orange-500' :
                      stock.reason.includes('逆势') ? 'bg-blue-500/20 text-blue-500' :
                      'bg-yellow-500/20 text-yellow-600'
                    }`}>
                      {stock.reason}
                    </span>
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === 'surge' && (
          <div className="p-3 space-y-2">
            {alerts.filter(a => !a.acknowledged).length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                <CheckCircle className="w-8 h-8 mx-auto mb-2 opacity-30" />
                <p className="text-sm">暂无大单异动预警</p>
              </div>
            ) : (
              alerts.filter(a => !a.acknowledged).map((alert) => (
                <div
                  key={alert.id}
                  className="p-3 bg-card border border-red-500/30 rounded-lg"
                >
                  <div className="flex items-start justify-between">
                    <div className="flex items-center gap-2">
                      <AlertTriangle className="w-4 h-4 text-red-500" />
                      <span className="font-medium text-sm">{alert.name}</span>
                      <span className="text-xs text-muted-foreground">{alert.symbol}</span>
                    </div>
                    <button
                      onClick={() => acknowledgeAlert(alert.id)}
                      className="p-1 hover:bg-muted rounded"
                    >
                      <CheckCircle className="w-4 h-4 text-green-500" />
                    </button>
                  </div>
                  <p className="mt-2 text-xs text-muted-foreground">{alert.message}</p>
                  <div className="flex items-center gap-2 mt-2 text-xs text-muted-foreground">
                    <Clock className="w-3 h-3" />
                    {alert.timestamp}
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === 'sector' && (
          <div className="p-3 space-y-2">
            {sectors.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                <Filter className="w-8 h-8 mx-auto mb-2 opacity-30" />
                <p className="text-sm">暂无板块数据</p>
              </div>
            ) : (
              sectors.slice(0, 20).map((sector) => (
                <div
                  key={sector.sector_code}
                  className="p-3 bg-card border border-border rounded-lg"
                >
                  <div className="flex items-start justify-between">
                    <div>
                      <div className="font-medium text-sm">{sector.sector_name}</div>
                      <div className="text-xs text-muted-foreground mt-1">
                        {sector.stock_count}只股票 | 股均 {formatMoney(sector.avg_inflow_per_stock)}
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="text-xs text-muted-foreground">主力净流入</div>
                      <div className={`text-sm font-medium ${
                        sector.main_inflow_total >= 0 ? 'text-red-500' : 'text-green-500'
                      }`}>
                        {sector.main_inflow_total >= 0 ? '+' : ''}{formatMoney(sector.main_inflow_total)}
                      </div>
                    </div>
                  </div>
                  {sector.hot_stocks.length > 0 && (
                    <div className="flex flex-wrap gap-1 mt-2">
                      {sector.hot_stocks.map(code => (
                        <span
                          key={code}
                          className="px-2 py-0.5 text-[10px] bg-muted rounded"
                        >
                          {code}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
