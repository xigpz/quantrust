use serde::{Deserialize, Serialize};
use crate::models::StockQuote;

/// 情感分析结果
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
    pub reason: String,
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

/// 市场异动预测服务
pub struct AnomalyPredictor;

impl AnomalyPredictor {
    /// 基于市场数据预测异动
    pub fn predict_from_market(quotes: &[super::StockQuote]) -> Vec<AnomalyPrediction> {
        let mut predictions = Vec::new();
        
        for quote in quotes {
            // 检测涨停前兆
            if quote.change_pct >= 8.0 && quote.change_pct < 9.5 {
                predictions.push(AnomalyPrediction {
                    symbol: quote.symbol.clone(),
                    name: quote.name.clone(),
                    prediction_type: "即将涨停".to_string(),
                    sentiment: SentimentAnalysis {
                        score: 0.9,
                        label: "利好".to_string(),
                        keywords: vec!["接近涨停".to_string()],
                        confidence: 0.8,
                    },
                    urgency: "高".to_string(),
                    timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    reason: format!("涨幅{}%，有望冲击涨停", quote.change_pct),
                    related_sectors: vec![],
                });
            }
            // 检测大跌风险
            else if quote.change_pct <= -7.0 {
                predictions.push(AnomalyPrediction {
                    symbol: quote.symbol.clone(),
                    name: quote.name.clone(),
                    prediction_type: "风险警示".to_string(),
                    sentiment: SentimentAnalysis {
                        score: -0.9,
                        label: "利空".to_string(),
                        keywords: vec!["大幅下跌".to_string()],
                        confidence: 0.8,
                    },
                    urgency: "高".to_string(),
                    timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    reason: format!("跌幅{}%，注意风险", quote.change_pct),
                    related_sectors: vec![],
                });
            }
            // 检测放量异动
            else if quote.turnover > 20.0 && quote.change_pct.abs() > 3.0 {
                let direction = if quote.change_pct > 0 { "上涨" } else { "下跌" };
                predictions.push(AnomalyPrediction {
                    symbol: quote.symbol.clone(),
                    name: quote.name.clone(),
                    prediction_type: format!("放量{}", direction),
                    sentiment: SentimentAnalysis {
                        score: quote.change_pct / 100.0,
                        label: if quote.change_pct > 0 { "利好" } else { "利空" }.to_string(),
                        keywords: vec!["放量".to_string(), "换手率高".to_string()],
                        confidence: 0.7,
                    },
                    urgency: "中".to_string(),
                    timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    reason: format!("换手率{}%，成交量异常放大", quote.turnover),
                    related_sectors: vec![],
                });
            }
        }
        
        // 按紧急程度排序
        predictions.sort_by(|a, b| {
            let order = |u: &str| match u { "高" => 0, "中" => 1, _ => 2 };
            order(&b.urgency).cmp(&order(&a.urgency))
        });
        
        predictions.truncate(20);
        predictions
    }
    
    /// 基于关键词分析文本情感
    pub fn analyze_text_sentiment(title: &str, content: &str) -> SentimentAnalysis {
        let text = format!("{} {}", title, content).to_lowercase();
        
        let positive = vec!["利好","上涨","增长","盈利","突破","创新","业绩预增","订单","中标","合作","回购","增持","评级","买入","推荐","特大利好","业绩暴增"];
        let negative = vec!["利空","下跌","亏损","风险","警示","调查","处罚","减持","业绩预亏","跌停","暴跌","特大利空","业绩暴亏"];
        
        let mut pos_cnt = 0;
        let mut neg_cnt = 0;
        let mut keywords = vec![];
        
        for w in &positive {
            if text.contains(&w.to_lowercase()) { pos_cnt += 1; keywords.push(w.clone()); }
        }
        for w in &negative {
            if text.contains(&w.to_lowercase()) { neg_cnt += 1; keywords.push(w.clone()); }
        }
        
        let total = pos_cnt + neg_cnt;
        let score = if total > 0 { (pos_cnt as f64 - neg_cnt as f64) / (total as f64) } else { 0.0 };
        
        let (label, confidence) = if total == 0 {
            ("中性".to_string(), 0.3)
        } else if score > 0.3 {
            ("利好".to_string(), 0.6_f64.min(0.4 + pos_cnt as f64 / 10.0))
        } else if score < -0.3 {
            ("利空".to_string(), 0.6_f64.min(0.4 + neg_cnt as f64 / 10.0))
        } else {
            ("中性".to_string(), 0.4)
        };
        
        SentimentAnalysis { score, label, keywords, confidence }
    }
}
