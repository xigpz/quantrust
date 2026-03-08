use serde::{Deserialize, Serialize};

/// 新闻类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewsType {
    Announcement,
    News,
    Research,
    Industry,
}

/// 新闻情感分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentAnalysis {
    pub score: f64,
    pub label: String,
    pub keywords: Vec<String>,
    pub confidence: f64,
}

/// 异动预测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyPrediction {
    pub symbol: String,
    pub name: String,
    pub prediction_type: String,
    pub sentiment: SentimentAnalysis,
    pub urgency: String,
    pub timestamp: String,
    pub source: String,
    pub related_sectors: Vec<String>,
}

/// 新闻项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockNews {
    pub symbol: String,
    pub name: String,
    pub title: String,
    pub content: String,
    pub pub_time: String,
    pub source: String,
    pub news_type: String,
}

/// 新闻服务
#[derive(Debug, Clone)]
pub struct NewsService {
    // 可以添加HTTP客户端等
}

impl NewsService {
    pub fn new() -> Self {
        Self {}
    }

    /// 分析新闻情感
    pub fn analyze_sentiment(&self, title: &str, content: &str) -> SentimentAnalysis {
        let text = format!("{} {}", title, content);
        let text_lower = text.to_lowercase();
        
        let positive_words = vec![
            "利好", "上涨", "增长", "盈利", "突破", "创新", "高增长", "业绩预增", 
            "订单", "签约", "中标", "合作", "扩张", "回购", "增持", "评级", 
            "买入", "推荐", "目标价", "上涨空间", "景气", "复苏", "爆发",
            "特大利好", "重大突破", "业绩暴增", "订单爆满", "供不应求"
        ];
        
        let negative_words = vec![
            "利空", "下跌", "亏损", "风险", "警示", "调查", "处罚", "立案",
            "减持", "业绩预亏", "商誉减值", "债务", "违约", "诉讼",
            "跌停", "暴跌", "大跌", "恐慌", "抛售", "业绩下滑",
            "特大利空", "重大利空", "业绩暴亏", "资金紧张", "资不抵债"
        ];
        
        let mut positive_count = 0;
        let mut negative_count = 0;
        let mut keywords = Vec::new();
        
        for word in &positive_words {
            if text_lower.contains(&word.to_lowercase()) {
                positive_count += 1;
                keywords.push(word.clone());
            }
        }
        
        for word in &negative_words {
            if text_lower.contains(&word.to_lowercase()) {
                negative_count += 1;
                keywords.push(word.clone());
            }
        }
        
        let total = positive_count + negative_count;
        let score = if total > 0 {
            (positive_count as f64 - negative_count as f64) / (total as f64)
        } else {
            0.0
        };
        
        let (label, confidence) = if total == 0 {
            ("中性".to_string(), 0.3)
        } else if score > 0.3 {
            ("利好".to_string(), 0.5 + (positive_count as f64 / 10.0).min(0.4))
        } else if score < -0.3 {
            ("利空".to_string(), 0.5 + (negative_count as f64 / 10.0).min(0.4))
        } else {
            ("中性".to_string(), 0.4)
        };
        
        SentimentAnalysis { score, label, keywords, confidence }
    }
    
    /// 预测异动
    pub fn predict_anomaly(&self, title: &str, content: &str) -> Option<(String, Vec<String>)> {
        let text = format!("{} {}", title, content).to_lowercase();
        
        let patterns = vec![
            (vec!["业绩", "预增", "暴增", "扭亏"], "业绩预增", vec!["业绩预增", "年报"]),
            (vec!["订单", "中标", "签约", "合作"], "订单利好", vec!["订单", "新能源", "基建"]),
            (vec!["回购", "增持", "激励"], "回购增持", vec!["回购", "股权激励"]),
            (vec!["政策", "支持", "补贴"], "政策利好", vec!["政策", "新能源", "半导体"]),
            (vec!["AI", "人工智能", "大模型"], "AI利好", vec!["AI", "人工智能"]),
            (vec!["新能源", "锂电", "光伏"], "新能源利好", vec!["新能源", "锂电池"]),
            (vec!["减持", "利空", "亏损"], "风险警示", vec!["减持", "利空"]),
            (vec!["重组", "并购", "借壳"], "并购重组", vec!["并购", "重组"]),
        ];
        
        for (keywords, pred_type, sectors) in patterns {
            for kw in &keywords {
                if text.contains(&kw.to_lowercase()) {
                    return Some((pred_type.to_string(), sectors.iter().map(|s| s.to_string()).collect()));
                }
            }
        }
        None
    }
    
    /// 获取异动预测
    pub fn get_anomaly_predictions(&self, news_list: &[StockNews]) -> Vec<AnomalyPrediction> {
        let mut predictions = Vec::new();
        
        for news in news_list {
            let sentiment = self.analyze_sentiment(&news.title, &news.content);
            
            if sentiment.confidence > 0.5 {
                if let Some((pred_type, sectors)) = self.predict_anomaly(&news.title, &news.content) {
                    let urgency = if sentiment.label == "利好" && sentiment.score > 0.5 {
                        "高"
                    } else if sentiment.label == "利空" && sentiment.score < -0.5 {
                        "高"
                    } else {
                        "中"
                    };
                    
                    predictions.push(AnomalyPrediction {
                        symbol: news.symbol.clone(),
                        name: news.name.clone(),
                        prediction_type: pred_type,
                        sentiment,
                        urgency: urgency.to_string(),
                        timestamp: news.pub_time.clone(),
                        source: news.source.clone(),
                        related_sectors: sectors,
                    });
                }
            }
        }
        
        predictions.sort_by(|a, b| {
            let order = |u: &str| match u { "高" => 0, "中" => 1, _ => 2 };
            order(&b.urgency).cmp(&order(&a.urgency))
        });
        
        predictions
    }
}

impl Default for NewsService {
    fn default() -> Self { Self::new() }
}
