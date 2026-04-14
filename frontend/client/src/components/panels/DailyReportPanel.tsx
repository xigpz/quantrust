/**
 * DailyReportPanel - 每日交易报告面板
 * 显示 AI 自主交易的每日报告、收益情况、交易记录等
 */
import { useState, useEffect } from 'react';
import { API_BASE } from '@/hooks/useMarketData';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  TrendingUp,
  TrendingDown,
  DollarSign,
  BarChart3,
  Calendar,
  Brain,
  RefreshCw,
  ChevronLeft,
  ChevronRight,
} from 'lucide-react';

interface Position {
  symbol: string;
  name: string;
  quantity: number;
  avg_cost: number;
  current_price: number;
  market_value: number;
  profit_loss: number;
  profit_ratio: number;
}

interface Trade {
  timestamp: string;
  symbol: string;
  name: string;
  action: string;
  price: number;
  quantity: number;
  amount: number;
  pnl: number | null;
  reason: string;
}

interface StrategyStats {
  total_trades: number;
  win_rate: number;
  profit_factor: number;
  max_drawdown: number;
  avg_holding_days: number;
}

interface LearningItem {
  timestamp: string;
  event: string;
  lesson: string;
  adjustment: string;
}

interface DailyReport {
  date: string;
  generated_at: string;
  initial_capital: number;
  final_capital: number;
  total_pnl: number;
  pnl_ratio: number;
  positions: Position[];
  positions_count: number;
  trades: Trade[];
  trades_count: number;
  buy_count: number;
  sell_count: number;
  win_count: number;
  lose_count: number;
  win_rate: number;
  strategy_stats: StrategyStats;
  hot_sectors: string[];
  market_summary: string;
  ai_observations: string[];
  tomorrow_outlook: string;
  recent_learning: LearningItem[];
}

interface ReportSummary {
  date: string;
  total_pnl: number;
  pnl_ratio: number;
  trades_count: number;
  win_rate: number;
}

export default function DailyReportPanel() {
  const [report, setReport] = useState<DailyReport | null>(null);
  const [summary, setSummary] = useState<ReportSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedDate, setSelectedDate] = useState(() => {
    const now = new Date();
    return now.toISOString().split('T')[0];
  });

  // 加载报告摘要
  const loadSummary = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/reports/summary?days=7`);
      const data = await res.json();
      if (data.success) {
        setSummary(data.data);
      }
    } catch (e) {
      console.error('加载报告摘要失败', e);
    }
  };

  // 加载指定日期的报告
  const loadReport = async (date: string) => {
    setLoading(true);
    try {
      const res = await fetch(`${API_BASE}/api/reports/daily/${date}`);
      const data = await res.json();
      if (data.success) {
        setReport(data.data);
      }
    } catch (e) {
      console.error('加载报告失败', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadSummary();
    loadReport(selectedDate);
  }, [selectedDate]);

  // 切换日期
  const changeDate = (delta: number) => {
    const date = new Date(selectedDate);
    date.setDate(date.getDate() + delta);
    setSelectedDate(date.toISOString().split('T')[0]);
  };

  // 格式化日期显示
  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return `${date.getMonth() + 1}月${date.getDate()}日`;
  };

  // 获取当前报告的摘要
  const currentSummary = summary.find((s) => s.date === selectedDate);

  return (
    <div className="flex flex-col h-full bg-background">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-border bg-card/50">
        <div className="flex items-center gap-3">
          <BarChart3 className="w-5 h-5 text-primary" />
          <h2 className="text-base font-semibold">每日交易报告</h2>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={() => loadReport(selectedDate)}>
            <RefreshCw className="w-4 h-4 mr-1" />
            刷新
          </Button>
        </div>
      </div>

      {/* 日期选择器 */}
      <div className="flex items-center justify-center gap-4 py-3 border-b border-border bg-muted/30">
        <Button variant="ghost" size="sm" onClick={() => changeDate(-1)}>
          <ChevronLeft className="w-4 h-4" />
        </Button>
        <div className="flex items-center gap-2">
          <Calendar className="w-4 h-4 text-muted-foreground" />
          <span className="font-medium">{selectedDate}</span>
        </div>
        <Button variant="ghost" size="sm" onClick={() => changeDate(1)}>
          <ChevronRight className="w-4 h-4" />
        </Button>
      </div>

      {/* 近期报告概览 */}
      <div className="flex gap-2 px-4 py-2 overflow-x-auto border-b border-border bg-muted/20">
        {summary.slice(0, 7).map((s) => (
          <button
            key={s.date}
            onClick={() => setSelectedDate(s.date)}
            className={`flex-shrink-0 px-3 py-1.5 rounded text-xs transition-colors ${
              s.date === selectedDate
                ? 'bg-primary text-primary-foreground'
                : 'bg-card border border-border hover:bg-muted'
            }`}
          >
            <div className="font-medium">{formatDate(s.date)}</div>
            <div className={s.total_pnl >= 0 ? 'text-green-500' : 'text-red-500'}>
              {s.total_pnl >= 0 ? '+' : ''}{s.total_pnl.toFixed(0)}
            </div>
          </button>
        ))}
      </div>

      {/* 主内容 */}
      <div className="flex-1 overflow-auto p-4">
        {loading ? (
          <div className="flex items-center justify-center h-64">
            <RefreshCw className="w-8 h-8 animate-spin text-muted-foreground" />
          </div>
        ) : report ? (
          <Tabs defaultValue="overview" className="space-y-4">
            <TabsList>
              <TabsTrigger value="overview">收益概览</TabsTrigger>
              <TabsTrigger value="trades">交易记录</TabsTrigger>
              <TabsTrigger value="positions">持仓明细</TabsTrigger>
              <TabsTrigger value="analysis">AI分析</TabsTrigger>
            </TabsList>

            {/* 收益概览 */}
            <TabsContent value="overview" className="space-y-4">
              {/* 核心指标卡片 */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <Card>
                  <CardContent className="pt-4">
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <DollarSign className="w-4 h-4" />
                      总盈亏
                    </div>
                    <div
                      className={`text-2xl font-bold mt-1 ${
                        report.total_pnl >= 0 ? 'text-green-500' : 'text-red-500'
                      }`}
                    >
                      {report.total_pnl >= 0 ? '+' : ''}
                      {report.total_pnl.toFixed(2)}
                    </div>
                    <div
                      className={`text-sm ${
                        report.pnl_ratio >= 0 ? 'text-green-500' : 'text-red-500'
                      }`}
                    >
                      {report.pnl_ratio >= 0 ? '+' : ''}
                      {(report.pnl_ratio * 100).toFixed(2)}%
                    </div>
                  </CardContent>
                </Card>

                <Card>
                  <CardContent className="pt-4">
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <BarChart3 className="w-4 h-4" />
                      胜率
                    </div>
                    <div className="text-2xl font-bold mt-1">
                      {(report.win_rate * 100).toFixed(1)}%
                    </div>
                    <div className="text-sm text-muted-foreground">
                      {report.win_count}胜/{report.lose_count}负
                    </div>
                  </CardContent>
                </Card>

                <Card>
                  <CardContent className="pt-4">
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <TrendingUp className="w-4 h-4" />
                      交易次数
                    </div>
                    <div className="text-2xl font-bold mt-1">{report.trades_count}</div>
                    <div className="text-sm text-muted-foreground">
                      买{report.buy_count}/卖{report.sell_count}
                    </div>
                  </CardContent>
                </Card>

                <Card>
                  <CardContent className="pt-4">
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <Brain className="w-4 h-4" />
                      策略胜率
                    </div>
                    <div className="text-2xl font-bold mt-1">
                      {(report.strategy_stats.win_rate * 100).toFixed(1)}%
                    </div>
                    <div className="text-sm text-muted-foreground">
                      盈亏比 {report.strategy_stats.profit_factor.toFixed(2)}
                    </div>
                  </CardContent>
                </Card>
              </div>

              {/* 资金曲线 */}
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">资金概览</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-3 gap-4 text-center">
                    <div>
                      <div className="text-sm text-muted-foreground">初始资金</div>
                      <div className="text-lg font-semibold">
                        {report.initial_capital.toLocaleString()}
                      </div>
                    </div>
                    <div>
                      <div className="text-sm text-muted-foreground">当前资金</div>
                      <div className="text-lg font-semibold">
                        {report.final_capital.toLocaleString()}
                      </div>
                    </div>
                    <div>
                      <div className="text-sm text-muted-foreground">持仓数</div>
                      <div className="text-lg font-semibold">{report.positions_count}</div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </TabsContent>

            {/* 交易记录 */}
            <TabsContent value="trades">
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">交易记录</CardTitle>
                </CardHeader>
                <CardContent>
                  {report.trades.length > 0 ? (
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead>时间</TableHead>
                          <TableHead>股票</TableHead>
                          <TableHead>操作</TableHead>
                          <TableHead className="text-right">价格</TableHead>
                          <TableHead className="text-right">数量</TableHead>
                          <TableHead className="text-right">盈亏</TableHead>
                          <TableHead>理由</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {report.trades.map((trade, idx) => (
                          <TableRow key={idx}>
                            <TableCell className="text-xs">
                              {trade.timestamp.slice(11, 16)}
                            </TableCell>
                            <TableCell>
                              <div className="font-medium">{trade.symbol}</div>
                              <div className="text-xs text-muted-foreground">
                                {trade.name}
                              </div>
                            </TableCell>
                            <TableCell>
                              <Badge
                                className={
                                  trade.action === 'buy'
                                    ? 'bg-green-500'
                                    : 'bg-red-500'
                                }
                              >
                                {trade.action === 'buy' ? '买入' : '卖出'}
                              </Badge>
                            </TableCell>
                            <TableCell className="text-right">{trade.price}</TableCell>
                            <TableCell className="text-right">{trade.quantity}</TableCell>
                            <TableCell
                              className={`text-right ${
                                trade.pnl && trade.pnl >= 0
                                  ? 'text-green-500'
                                  : 'text-red-500'
                              }`}
                            >
                              {trade.pnl !== null
                                ? `${trade.pnl >= 0 ? '+' : ''}${trade.pnl.toFixed(2)}`
                                : '-'}
                            </TableCell>
                            <TableCell className="text-xs text-muted-foreground max-w-[150px] truncate">
                              {trade.reason}
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  ) : (
                    <div className="text-center py-8 text-muted-foreground">
                      暂无交易记录
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>

            {/* 持仓明细 */}
            <TabsContent value="positions">
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">当前持仓</CardTitle>
                </CardHeader>
                <CardContent>
                  {report.positions.length > 0 ? (
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead>股票</TableHead>
                          <TableHead className="text-right">持仓数量</TableHead>
                          <TableHead className="text-right">成本价</TableHead>
                          <TableHead className="text-right">当前价</TableHead>
                          <TableHead className="text-right">市值</TableHead>
                          <TableHead className="text-right">盈亏</TableHead>
                          <TableHead className="text-right">盈亏比例</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {report.positions.map((pos) => (
                          <TableRow key={pos.symbol}>
                            <TableCell>
                              <div className="font-medium">{pos.symbol}</div>
                              <div className="text-xs text-muted-foreground">
                                {pos.name}
                              </div>
                            </TableCell>
                            <TableCell className="text-right">{pos.quantity}</TableCell>
                            <TableCell className="text-right">{pos.avg_cost}</TableCell>
                            <TableCell className="text-right">{pos.current_price}</TableCell>
                            <TableCell className="text-right">
                              {pos.market_value.toFixed(2)}
                            </TableCell>
                            <TableCell
                              className={`text-right ${
                                pos.profit_loss >= 0
                                  ? 'text-green-500'
                                  : 'text-red-500'
                              }`}
                            >
                              {pos.profit_loss >= 0 ? '+' : ''}
                              {pos.profit_loss.toFixed(2)}
                            </TableCell>
                            <TableCell
                              className={`text-right ${
                                pos.profit_ratio >= 0
                                  ? 'text-green-500'
                                  : 'text-red-500'
                              }`}
                            >
                              {pos.profit_ratio >= 0 ? '+' : ''}
                              {(pos.profit_ratio * 100).toFixed(2)}%
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  ) : (
                    <div className="text-center py-8 text-muted-foreground">
                      暂无持仓
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>

            {/* AI 分析 */}
            <TabsContent value="analysis" className="space-y-4">
              {/* AI 观察 */}
              {report.ai_observations.length > 0 && (
                <Card>
                  <CardHeader>
                    <CardTitle className="text-lg flex items-center gap-2">
                      <Brain className="w-5 h-5 text-purple-500" />
                      AI 观察
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <ul className="space-y-2">
                      {report.ai_observations.map((obs, idx) => (
                        <li key={idx} className="flex items-start gap-2">
                          <span className="text-purple-500">•</span>
                          <span className="text-sm">{obs}</span>
                        </li>
                      ))}
                    </ul>
                  </CardContent>
                </Card>
              )}

              {/* 次日展望 */}
              {report.tomorrow_outlook && (
                <Card>
                  <CardHeader>
                    <CardTitle className="text-lg flex items-center gap-2">
                      <TrendingUp className="w-5 h-5 text-blue-500" />
                      次日展望
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <p className="text-sm">{report.tomorrow_outlook}</p>
                  </CardContent>
                </Card>
              )}

              {/* 学习记录 */}
              {report.recent_learning.length > 0 && (
                <Card>
                  <CardHeader>
                    <CardTitle className="text-lg">最近学习</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-3">
                      {report.recent_learning.map((item, idx) => (
                        <div
                          key={idx}
                          className="border-l-2 border-primary pl-3 py-1"
                        >
                          <div className="text-xs text-muted-foreground">
                            {item.timestamp}
                          </div>
                          <div className="font-medium text-sm">{item.event}</div>
                          <div className="text-sm text-muted-foreground">
                            {item.lesson}
                          </div>
                          {item.adjustment && (
                            <div className="text-xs text-primary mt-1">
                              调整: {item.adjustment}
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              )}

              {/* 策略表现详情 */}
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">策略表现</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                    <div>
                      <div className="text-sm text-muted-foreground">总交易数</div>
                      <div className="text-lg font-semibold">
                        {report.strategy_stats.total_trades}
                      </div>
                    </div>
                    <div>
                      <div className="text-sm text-muted-foreground">胜率</div>
                      <div className="text-lg font-semibold">
                        {(report.strategy_stats.win_rate * 100).toFixed(1)}%
                      </div>
                    </div>
                    <div>
                      <div className="text-sm text-muted-foreground">盈亏比</div>
                      <div className="text-lg font-semibold">
                        {report.strategy_stats.profit_factor.toFixed(2)}
                      </div>
                    </div>
                    <div>
                      <div className="text-sm text-muted-foreground">最大回撤</div>
                      <div className="text-lg font-semibold text-red-500">
                        {(report.strategy_stats.max_drawdown * 100).toFixed(1)}%
                      </div>
                    </div>
                    <div>
                      <div className="text-sm text-muted-foreground">平均持仓</div>
                      <div className="text-lg font-semibold">
                        {report.strategy_stats.avg_holding_days.toFixed(1)}天
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>
        ) : (
          <div className="flex items-center justify-center h-64 text-muted-foreground">
            暂无报告数据
          </div>
        )}
      </div>
    </div>
  );
}
