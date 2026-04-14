import { useState, useEffect, useContext } from 'react';
import { Sparkles, TrendingUp, TrendingDown, Minus, RefreshCw, ExternalLink, Clock } from 'lucide-react';
import { StockClickContext } from '@/pages/Dashboard';
import { useNewsImpact, type ImpactedStock, type NewsAnalysis } from '@/hooks/useClsNews';

// 格式化时间
function formatTime(timeStr: string): string {
  if (!timeStr) return '';
  try {
    const date = new Date(timeStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / (1000 * 60));
    if (minutes < 1) return '刚刚';
    if (minutes < 60) return `${minutes}分钟前`;
    const hours = Math.floor(diff / (1000 * 60 * 60));
    if (hours < 24) return `${hours}小时前`;
    return timeStr.substring(0, 16);
  } catch {
    return timeStr;
  }
}

// 获取影响类型颜色
function getImpactColor(type?: string) {
  if (type === '利好') return 'text-green-400';
  if (type === '利空') return 'text-red-400';
  return 'text-yellow-400';
}

function getImpactBg(type?: string) {
  if (type === '利好') return 'bg-green-500/20 border-green-500/30';
  if (type === '利空') return 'bg-red-500/20 border-red-500/30';
  return 'bg-yellow-500/20 border-yellow-500/30';
}

export default function NewsImpactPanel() {
  const { openStock } = useContext(StockClickContext);
  const { impact, loading, error, refetch } = useNewsImpact(30);
  const [activeTab, setActiveTab] = useState<'stocks' | 'news'>('stocks');

  return (
    <div className="p-4 space-y-4">
      {/* 标题栏 */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Sparkles className="w-5 h-5 text-yellow-400" />
          <span className="font-semibold">AI新闻选股</span>
        </div>
        <button
          onClick={refetch}
          disabled={loading}
          className="p-1 hover:bg-muted rounded"
        >
          <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {/* 市场情绪 */}
      {impact && (
        <div className={`rounded-lg p-3 border ${getImpactBg(impact.market_sentiment)}`}>
          <div className="flex items-center gap-2 mb-2">
            {impact.market_sentiment === '偏多' && <TrendingUp className="w-4 h-4 text-green-400" />}
            {impact.market_sentiment === '偏空' && <TrendingDown className="w-4 h-4 text-red-400" />}
            {impact.market_sentiment === '中性' && <Minus className="w-4 h-4 text-yellow-400" />}
            <span className="text-sm font-medium">市场情绪: {impact.market_sentiment}</span>
          </div>
          <div className="text-xs text-muted-foreground">
            分析时间: {formatTime(impact.timestamp)}
          </div>
        </div>
      )}

      {/* 标签页 */}
      <div className="flex gap-2">
        <button
          onClick={() => setActiveTab('stocks')}
          className={`px-3 py-1.5 text-xs rounded ${
            activeTab === 'stocks' ? 'bg-primary text-primary-foreground' : 'bg-muted'
          }`}
        >
          AI推荐股票
        </button>
        <button
          onClick={() => setActiveTab('news')}
          className={`px-3 py-1.5 text-xs rounded ${
            activeTab === 'news' ? 'bg-primary text-primary-foreground' : 'bg-muted'
          }`}
        >
          重要新闻
        </button>
      </div>

      {/* 加载状态 */}
      {loading && !impact && (
        <div className="text-center py-8 text-muted-foreground">AI分析中...</div>
      )}

      {/* 错误状态 */}
      {error && (
        <div className="text-center py-4 text-red-400 text-sm">{error}</div>
      )}

      {/* 股票列表 */}
      {activeTab === 'stocks' && impact && (
        <div className="space-y-2">
          {(impact.top_stocks || []).length === 0 ? (
            <div className="text-center py-4 text-muted-foreground text-sm">暂无推荐</div>
          ) : (
            (impact.top_stocks || []).slice(0, 10).map((stock, i) => (
              <div
                key={`${stock.symbol}-${i}`}
                className="flex items-center justify-between p-3 bg-card rounded-lg border hover:bg-muted/50 cursor-pointer"
                onClick={() => openStock(stock.symbol, stock.name)}
              >
                <div className="flex items-center gap-3">
                  <div className="text-xs text-muted-foreground w-6">{i + 1}</div>
                  <div>
                    <div className="font-medium text-sm">{stock.name}</div>
                    <div className="text-xs text-muted-foreground">{stock.symbol}</div>
                  </div>
                </div>
                <div className="text-right">
                  <div className={`text-sm font-medium ${getImpactColor(stock.impact_type?.type)}`}>
                    {stock.impact_type?.type || '中性'}
                  </div>
                  <div className="text-xs text-muted-foreground">
                    置信度 {(stock.confidence * 100).toFixed(0)}%
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {/* 新闻列表 */}
      {activeTab === 'news' && impact && (
        <div className="space-y-3 max-h-[50vh] overflow-y-auto">
          {(impact.news_list || []).length === 0 ? (
            <div className="text-center py-4 text-muted-foreground text-sm">暂无重要新闻</div>
          ) : (
            (impact.news_list || []).map((item, i) => (
              <NewsItem key={i} analysis={item} />
            ))
          )}
        </div>
      )}
    </div>
  );
}

// 单条新闻组件
function NewsItem({ analysis }: { analysis: NewsAnalysis }) {
  const { openStock } = useContext(StockClickContext);
  const [expanded, setExpanded] = useState(false);

  const news = analysis.news || {};
  const sectors = analysis.sectors || [];
  const impactStocks = analysis.impact_stocks || [];

  return (
    <div className="bg-card rounded-lg border overflow-hidden">
      <div
        className="p-3 cursor-pointer hover:bg-muted/30"
        onClick={() => setExpanded(!expanded)}
      >
        <div className="flex items-start justify-between gap-2">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className={`px-2 py-0.5 text-xs rounded border ${getImpactBg(analysis.impact_type?.type)}`}>
                {analysis.impact_type?.type || '中性'}
              </span>
              <span className="text-xs text-muted-foreground flex items-center gap-1">
                <Clock className="w-3 h-3" />
                {formatTime(news.pub_time || '')}
              </span>
            </div>
            <h3 className="font-medium text-sm line-clamp-2">{news.title || '无标题'}</h3>
          </div>
          {news.url && news.url.length > 0 && (
            <a
              href={news.url}
              target="_blank"
              rel="noopener noreferrer"
              className="p-1 hover:bg-muted rounded"
              onClick={(e) => e.stopPropagation()}
            >
              <ExternalLink className="w-4 h-4 text-muted-foreground" />
            </a>
          )}
        </div>
      </div>

      {expanded && (
        <div className="px-3 pb-3 border-t bg-muted/20">
          {analysis.reason && analysis.reason.length > 0 && (
            <div className="mt-2 text-xs text-muted-foreground">
              <span className="font-medium">分析:</span> {analysis.reason}
            </div>
          )}

          {sectors.length > 0 && (
            <div className="mt-2">
              <div className="text-xs font-medium mb-1">📌 相关板块</div>
              <div className="flex gap-1 flex-wrap">
                {sectors.map((sector, i) => (
                  <span key={i} className="px-2 py-0.5 text-xs bg-blue-500/20 text-blue-400 rounded">
                    {sector}
                  </span>
                ))}
              </div>
            </div>
          )}

          {impactStocks.length > 0 && (
            <div className="mt-2">
              <div className="text-xs font-medium mb-1">📈 相关股票</div>
              <div className="flex gap-1 flex-wrap">
                {impactStocks.map((stock, i) => (
                  <button
                    key={i}
                    onClick={() => openStock(stock.symbol, stock.name)}
                    className={`px-2 py-0.5 text-xs rounded ${getImpactBg(stock.impact_type?.type)}`}
                  >
                    {stock.name}
                  </button>
                ))}
              </div>
            </div>
          )}

          <div className="mt-2 text-xs text-muted-foreground">
            来源: {news.source || '未知'}
          </div>
        </div>
      )}
    </div>
  );
}
