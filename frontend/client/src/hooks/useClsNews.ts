import { useState, useEffect, useCallback } from 'react';

const API_BASE = '';

// 新闻影响类型
export interface ImpactType {
  type: '利好' | '利空' | '中性';
}

export interface ImpactedStock {
  symbol: string;
  name: string;
  impact_type: ImpactType;
  impact_strength: number;
  confidence: number;
}

export interface NewsAnalysis {
  news: {
    id: string;
    title: string;
    content: string;
    pub_time: string;
    source: string;
    url: string;
    category: string;
  };
  impact_type: ImpactType;
  impact_stocks: ImpactedStock[];
  sectors: string[];
  strength: number;
  reason: string;
}

export interface NewsImpactResult {
  news_list: NewsAnalysis[];
  top_stocks: ImpactedStock[];
  sector_impacts: Record<string, number>;
  market_sentiment: string;
  timestamp: string;
}

// 获取财联社新闻
export function useClsNews(count = 30) {
  const [news, setNews] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchNews = useCallback(async () => {
    try {
      const res = await fetch(`${API_BASE}/api/cls/news?page=1&page_size=${count}`);
      const json = await res.json();
      if (json.success) {
        setNews(json.data.list || []);
      } else {
        setError(json.message || '获取新闻失败');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '网络错误');
    } finally {
      setLoading(false);
    }
  }, [count]);

  useEffect(() => {
    fetchNews();
    const interval = setInterval(fetchNews, 60000); // 每分钟刷新
    return () => clearInterval(interval);
  }, [fetchNews]);

  return { news, loading, error, refetch: fetchNews };
}

// 获取新闻影响分析（带AI分析）
export function useNewsImpact(count = 30) {
  const [impact, setImpact] = useState<NewsImpactResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchImpact = useCallback(async () => {
    setLoading(true);
    try {
      const res = await fetch(`${API_BASE}/api/news/impact?count=${count}`);
      const json = await res.json();
      if (json.success) {
        setImpact(json.data);
      } else {
        setError(json.message || '获取分析失败');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '网络错误');
    } finally {
      setLoading(false);
    }
  }, [count]);

  useEffect(() => {
    fetchImpact();
    const interval = setInterval(fetchImpact, 120000); // 每2分钟刷新
    return () => clearInterval(interval);
  }, [fetchImpact]);

  return { impact, loading, error, refetch: fetchImpact };
}
