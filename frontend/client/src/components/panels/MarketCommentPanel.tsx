/**
 * MarketCommentPanel - 市场评论分析面板 V2
 * 实时抓取个股评论分析市场风险 - 增强版
 */
import { useState, useEffect } from 'react';
import { RefreshCw, AlertTriangle, TrendingUp, MessageSquare, Clock, User, ThumbsUp, MessageCircle } from 'lucide-react';
import { PieChart, Pie, Cell, BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts';
import { useMarketComments } from '@/hooks/useMarketData';
import { useStockClick } from '@/pages/Dashboard';

export default function MarketCommentPanel() {
  const { data, loading, error, refresh } = useMarketComments();
  const [activeTab, setActiveTab] = useState<'overview' | 'hot' | 'high' | 'low' | 'comments'>('overview');
  const [selectedStock, setSelectedStock] = useState<string | null>(null);
  const { openStock } = useStockClick();

  const handleStockClick = (symbol: string, name: string) => {
    openStock(symbol, name);
  };

  const getRiskColor = (level: string) => {
    switch (level) {
      case 'high': return 'text-red-500';
      case 'medium': return 'text-yellow-500';
      default: return 'text-green-500';
    }
  };

  const getRiskBg = (level: string) => {
    switch (level) {
      case 'high': return 'bg-red-500/10 border-red-500/30';
      case 'medium': return 'bg-yellow-500/10 border-yellow-500/30';
      default: return 'bg-green-500/10 border-green-500/30';
    }
  };

  const getSentimentIcon = (sentiment: string) => {
    switch (sentiment) {
      case 'positive': return '📈';
      case 'negative': return '📉';
      default: return '➡️';
    }
  };

  // 格式化时间为相对时间
  const formatCommentTime = (pubTime: string) => {
    // pub_time 格式是 "HH:MM"
    const now = new Date();
    const [hours, minutes] = pubTime.split(':').map(Number);
    const commentDate = new Date();
    commentDate.setHours(hours, minutes, 0, 0);

    // 计算差异（分钟）
    const diffMs = now.getTime() - commentDate.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return '刚刚';
    if (diffMins < 60) return `${diffMins}分钟前`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}小时前`;
    return pubTime;
  };

  if (loading && !data) {
    return (
      <div className="h-full flex items-center justify-center">
        <RefreshCw className="w-8 h-8 animate-spin text-primary" />
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="h-full flex flex-col items-center justify-center gap-4">
        <AlertTriangle className="w-10 h-10 text-yellow-500" />
        <div className="text-sm text-muted-foreground">{error || '暂无数据'}</div>
        <button
          onClick={refresh}
          className="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm hover:bg-primary/90"
        >
          重试
        </button>
      </div>
    );
  }

  // 准备饼图数据
  const pieData = [
    { name: '正面', value: data.overall_sentiment.positive, color: '#22c55e' },
    { name: '负面', value: data.overall_sentiment.negative, color: '#ef4444' },
    { name: '中性', value: data.overall_sentiment.neutral, color: '#6b7280' },
  ].filter(d => d.value > 0);

  // 准备柱状图数据（按评论数排序）
  const barData = [...data.hot_stocks]
    .sort((a, b) => b.comment_count - a.comment_count)
    .slice(0, 10)
    .map(s => ({
      name: s.name.length > 4 ? s.name.slice(0, 4) : s.name,
      fullName: s.name,
      count: s.comment_count,
      riskScore: s.risk_score,
    }));

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* 顶部概览区 */}
      <div className="p-4 border-b bg-gradient-to-r from-primary/5 to-transparent">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-3">
            <MessageSquare className="w-5 h-5 text-primary" />
            <h2 className="text-lg font-semibold">市场评论分析</h2>
          </div>
          <button
            onClick={refresh}
            className="p-2 rounded-lg hover:bg-primary/10 transition-colors"
            title="刷新"
          >
            <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>

        {/* 关键指标卡片 */}
        <div className="grid grid-cols-4 gap-3">
          <div className={`rounded-lg border p-3 ${getRiskBg(data.market_risk_level)}`}>
            <div className="text-xs text-muted-foreground mb-1">市场风险</div>
            <div className={`text-xl font-bold ${getRiskColor(data.market_risk_level)}`}>
              {data.market_risk_level === 'high' ? '⚠️ 高' : data.market_risk_level === 'medium' ? '⚡ 中' : '✅ 低'}
            </div>
          </div>
          <div className="bg-card border rounded-lg p-3">
            <div className="text-xs text-muted-foreground mb-1">风险分数</div>
            <div className={`text-xl font-bold ${getRiskColor(data.market_risk_level)}`}>
              {data.market_risk_score.toFixed(1)}
            </div>
          </div>
          <div className="bg-card border rounded-lg p-3">
            <div className="text-xs text-muted-foreground mb-1">评论总数</div>
            <div className="text-xl font-bold">{data.total_comments.toLocaleString()}</div>
          </div>
          <div className="bg-card border rounded-lg p-3">
            <div className="text-xs text-muted-foreground mb-1">监控股票</div>
            <div className="text-xl font-bold">{data.hot_stocks.length}</div>
          </div>
        </div>
      </div>

      {/* Tab切换 */}
      <div className="px-4 pt-3 flex gap-1 bg-muted/20">
        {[
          { key: 'overview', label: '📊 概览' },
          { key: 'hot', label: '🔥 热门' },
          { key: 'high', label: '⚠️ 高风险' },
          { key: 'low', label: '✅ 低风险' },
          { key: 'comments', label: '💬 最新评论' },
        ].map(tab => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key as any)}
            className={`px-4 py-2 text-sm rounded-t-lg transition-colors ${
              activeTab === tab.key
                ? 'bg-card border-t border-x font-medium'
                : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* 内容区域 */}
      <div className="flex-1 overflow-y-auto p-4 bg-card">
        {/* 概览Tab */}
        {activeTab === 'overview' && (
          <div className="space-y-4">
            {/* 图表区域 */}
            <div className="grid grid-cols-2 gap-4">
              {/* 情感分布饼图 */}
              <div className="bg-muted/30 rounded-lg p-4">
                <h3 className="text-sm font-medium mb-3">情感分布</h3>
                <div className="h-48">
                  <ResponsiveContainer width="100%" height="100%">
                    <PieChart>
                      <Pie
                        data={pieData}
                        cx="50%"
                        cy="50%"
                        innerRadius={40}
                        outerRadius={70}
                        paddingAngle={2}
                        dataKey="value"
                      >
                        {pieData.map((entry, index) => (
                          <Cell key={`cell-${index}`} fill={entry.color} />
                        ))}
                      </Pie>
                      <Tooltip
                        contentStyle={{ background: 'var(--background)', border: '1px solid var(--border)', borderRadius: '8px', fontSize: '12px' }}
                        formatter={(value: any) => [`${value}条`, '']}
                      />
                    </PieChart>
                  </ResponsiveContainer>
                </div>
                <div className="flex justify-center gap-4 mt-2 text-xs">
                  {pieData.map(item => (
                    <div key={item.name} className="flex items-center gap-1">
                      <div className="w-2 h-2 rounded-full" style={{ backgroundColor: item.color }} />
                      <span>{item.name}: {item.value}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* 热门评论股票柱状图 */}
              <div className="bg-muted/30 rounded-lg p-4">
                <h3 className="text-sm font-medium mb-3">评论热度 TOP10</h3>
                <div className="h-48">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={barData} layout="vertical">
                      <XAxis type="number" tick={{ fontSize: 10 }} />
                      <YAxis dataKey="name" type="category" tick={{ fontSize: 10 }} width={50} />
                      <Tooltip
                        contentStyle={{ background: 'var(--background)', border: '1px solid var(--border)', borderRadius: '8px', fontSize: '12px' }}
                        formatter={(value: any, name: string) => [`${value}条评论`, name]}
                        labelFormatter={(label) => barData.find(d => d.name === label)?.fullName || label}
                      />
                      <Bar dataKey="count" fill="#3b82f6" radius={[0, 4, 4, 0]} />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              </div>
            </div>

            {/* 热门股票列表 */}
            <div className="bg-muted/30 rounded-lg p-4">
              <h3 className="text-sm font-medium mb-3">热门股票</h3>
              <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
                {data.hot_stocks.slice(0, 12).map(stock => (
                  <div
                    key={stock.symbol}
                    className="bg-card border rounded-lg p-3 hover:bg-muted/50 cursor-pointer transition-colors"
                    onClick={() => handleStockClick(stock.symbol, stock.name)}
                  >
                    <div className="flex items-center justify-between mb-1">
                      <span className="font-medium text-sm">{stock.name}</span>
                      <span className="text-xs">{getSentimentIcon(stock.sentiment)}</span>
                    </div>
                    <div className="flex items-center justify-between text-xs text-muted-foreground">
                      <span>{stock.comment_count}条评论</span>
                      <span className={getRiskColor(stock.risk_level)}>
                        风险{stock.risk_score.toFixed(0)}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {/* 热门讨论Tab */}
        {activeTab === 'hot' && (
          <div className="space-y-3">
            {data.hot_stocks.map(stock => (
              <div
                key={stock.symbol}
                className="bg-muted/30 rounded-lg p-4 hover:bg-muted/50 cursor-pointer transition-colors"
                onClick={() => handleStockClick(stock.symbol, stock.name)}
              >
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <span className="font-semibold">{stock.name}</span>
                    <span className="text-xs text-muted-foreground">{stock.symbol}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs bg-primary/10 px-2 py-1 rounded">
                      💬 {stock.comment_count}
                    </span>
                    <span className={`text-xs px-2 py-1 rounded ${getRiskBg(stock.risk_level)} ${getRiskColor(stock.risk_level)}`}>
                      {stock.risk_level === 'high' ? '⚠️高风险' : stock.risk_level === 'medium' ? '⚡中风险' : '✅低风险'}
                    </span>
                  </div>
                </div>
                <div className="flex items-center gap-4 text-xs text-muted-foreground">
                  <span>正面: {(stock.positive_ratio * 100).toFixed(0)}%</span>
                  <span>负面: {(stock.negative_ratio * 100).toFixed(0)}%</span>
                  <span>中性: {((1 - stock.positive_ratio - stock.negative_ratio) * 100).toFixed(0)}%</span>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* 高风险Tab */}
        {activeTab === 'high' && (
          <div className="space-y-3">
            {data.high_risk_stocks.length > 0 ? (
              data.high_risk_stocks.map(stock => (
                <div
                  key={stock.symbol}
                  className="bg-red-500/5 border border-red-500/20 rounded-lg p-4 hover:bg-red-500/10 cursor-pointer transition-colors"
                  onClick={() => handleStockClick(stock.symbol, stock.name)}
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                      <AlertTriangle className="w-4 h-4 text-red-500" />
                      <span className="font-semibold">{stock.name}</span>
                      <span className="text-xs text-muted-foreground">{stock.symbol}</span>
                    </div>
                    <span className="text-red-500 font-bold text-lg">{stock.risk_score.toFixed(1)}</span>
                  </div>
                  <div className="text-xs text-muted-foreground">
                    负面评论占比: {(stock.negative_ratio * 100).toFixed(1)}% | 评论数: {stock.comment_count}
                  </div>
                </div>
              ))
            ) : (
              <div className="text-center py-12">
                <div className="text-4xl mb-3">✅</div>
                <div className="text-muted-foreground">当前无高风险股票，市场情绪稳定</div>
              </div>
            )}
          </div>
        )}

        {/* 低风险Tab */}
        {activeTab === 'low' && (
          <div className="space-y-3">
            {data.low_risk_stocks.map(stock => (
              <div
                key={stock.symbol}
                className="bg-green-500/5 border border-green-500/20 rounded-lg p-4 hover:bg-green-500/10 cursor-pointer transition-colors"
                onClick={() => handleStockClick(stock.symbol, stock.name)}
              >
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <TrendingUp className="w-4 h-4 text-green-500" />
                    <span className="font-semibold">{stock.name}</span>
                    <span className="text-xs text-muted-foreground">{stock.symbol}</span>
                  </div>
                  <span className="text-green-500 font-bold text-lg">{stock.risk_score.toFixed(1)}</span>
                </div>
                <div className="text-xs text-muted-foreground">
                  正面评论占比: {(stock.positive_ratio * 100).toFixed(1)}% | 评论数: {stock.comment_count}
                </div>
              </div>
            ))}
          </div>
        )}

        {/* 最新评论Tab */}
        {activeTab === 'comments' && (
          <div className="space-y-3">
            {data.recent_comments.map((item, idx) => (
              <div
                key={`${item.symbol}-${idx}`}
                className="bg-muted/30 rounded-lg p-4 hover:bg-muted/50 cursor-pointer transition-colors"
                onClick={() => handleStockClick(item.symbol, item.name)}
              >
                {/* 股票头部 */}
                <div className="flex items-center gap-2 mb-3 pb-2 border-b border-border/50">
                  <div className="flex items-center gap-2 bg-primary/10 px-3 py-1.5 rounded-lg">
                    <span className="font-bold text-primary">{item.name}</span>
                    <span className="text-xs text-muted-foreground font-mono">{item.symbol}</span>
                  </div>
                  <span className={`text-xs px-2 py-0.5 rounded ${
                    item.comment.sentiment === 'positive' ? 'bg-green-500/20 text-green-400' :
                    item.comment.sentiment === 'negative' ? 'bg-red-500/20 text-red-400' :
                    'bg-gray-500/20 text-gray-400'
                  }`}>
                    {item.comment.sentiment === 'positive' ? '📈 利好' :
                     item.comment.sentiment === 'negative' ? '📉 利空' : '➡️ 中性'}
                  </span>
                  <div className="flex items-center gap-1 text-xs text-muted-foreground ml-auto">
                    <Clock className="w-3 h-3" />
                    <span>{formatCommentTime(item.comment.pub_time)}</span>
                  </div>
                </div>

                {/* 评论主题 */}
                <div className="text-xs text-muted-foreground mb-2 flex items-center gap-1">
                  <MessageSquare className="w-3 h-3" />
                  <span>用户评论：</span>
                </div>

                {/* 评论内容 */}
                <div className="text-sm mb-3 bg-card rounded-lg p-3 border-l-4 ${
                  item.comment.sentiment === 'positive' ? 'border-l-green-500' :
                  item.comment.sentiment === 'negative' ? 'border-l-red-500' :
                  'border-l-gray-500'
                }">
                  {item.comment.content}
                </div>

                {/* 评论者信息 */}
                <div className="flex items-center justify-between text-xs text-muted-foreground">
                  <div className="flex items-center gap-1">
                    <User className="w-3 h-3" />
                    <span>@{item.comment.nickname}</span>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className="flex items-center gap-1">
                      <ThumbsUp className="w-3 h-3" />
                      {item.comment.like_count}
                    </span>
                    <span className="flex items-center gap-1">
                      <MessageCircle className="w-3 h-3" />
                      {item.comment.reply_count}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
