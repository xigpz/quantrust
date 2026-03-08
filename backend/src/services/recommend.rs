use crate::models::*;
use crate::data::DataProvider;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 每日股票推荐服务
/// 综合多维度进行评分：动量、资金流向、技术形态
pub struct RecommendService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockRecommend {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
    pub score: f64,              // 综合评分 0-100
    pub level: String,           // 评级: 强烈推荐/推荐/关注
    pub reasons: Vec<String>,    // 推荐原因
    pub risk_level: String,      // 风险等级: 低/中/高
    pub target_price: Option<f64>,
    pub stop_loss: Option<f64>,
}

impl RecommendService {
    pub fn new() -> Self {
        Self
    }

    /// 获取每日推荐股票 - 优化版本，使用缓存数据
    pub async fn get_daily_recommends(&self, _provider: &DataProvider, quotes: &[StockQuote]) -> Vec<StockRecommend> {
        let mut recommends = Vec::new();
        
        // 过滤ST股票和退市股票，排除科创板和北交所
        let valid_quotes: Vec<&StockQuote> = quotes
            .iter()
            .filter(|q| {
                !q.name.contains("ST") && 
                !q.name.contains("退") && 
                q.price > 0.0 &&
                !q.symbol.contains("BJ") &&  // 排除北交所
                !q.symbol.contains("688")    // 排除科创板（风险较大）
            })
            .collect();
        
        // 对每只股票进行评分
        for quote in valid_quotes {
            let rec = self.evaluate_stock(quote, quotes);
            recommends.push(rec);
        }
        
        // 按评分排序，取前10
        recommends.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        recommends.truncate(10);
        
        recommends
    }

    /// 评估单只股票 - 简化版，不获取K线
    fn evaluate_stock(&self, quote: &StockQuote, all_quotes: &[StockQuote]) -> StockRecommend {
        let mut score = 0.0;
        let mut reasons = Vec::new();
        
        // 1. 涨幅评分 (最高35分)
        let change = quote.change_pct;
        if change >= 3.0 && change <= 7.0 {
            reasons.push("涨幅健康，突破有效".to_string());
            score += 35.0;
        } else if change > 0.0 && change < 3.0 {
            reasons.push("稳步上涨，风险较低".to_string());
            score += 25.0;
        } else if change >= 7.0 && change < 10.0 {
            reasons.push("涨幅较大，注意追高风险".to_string());
            score += 15.0;
        } else if change >= 10.0 {
            reasons.push("涨停或接近涨停".to_string());
            score += 20.0;
        }
        
        // 2. 成交量评分 (最高25分)
        let avg_volume: f64 = all_quotes.iter().map(|q| q.volume).sum::<f64>() / all_quotes.len() as f64;
        let volume_ratio = if avg_volume > 0.0 { quote.volume / avg_volume } else { 1.0 };
        
        if volume_ratio > 3.0 {
            reasons.push("成交量大幅放大".to_string());
            score += 25.0;
        } else if volume_ratio > 2.0 {
            reasons.push("成交量明显放大".to_string());
            score += 20.0;
        } else if volume_ratio > 1.5 {
            reasons.push("成交量适度放大".to_string());
            score += 15.0;
        }
        
        // 3. 换手率评分 (最高20分)
        let turnover_rate = quote.turnover_rate;
        if turnover_rate >= 15.0 && turnover_rate <= 40.0 {
            reasons.push("换手率活跃，资金进出频繁".to_string());
            score += 20.0;
        } else if turnover_rate >= 8.0 && turnover_rate < 15.0 {
            reasons.push("换手率适中".to_string());
            score += 15.0;
        } else if turnover_rate > 0.0 && turnover_rate < 8.0 {
            score += 8.0;
        }
        
        // 4. 成交额评分 (最高15分)
        let avg_turnover: f64 = all_quotes.iter().map(|q| q.turnover).sum::<f64>() / all_quotes.len() as f64;
        let turnover_ratio = if avg_turnover > 0.0 { quote.turnover / avg_turnover } else { 1.0 };
        
        if turnover_ratio > 3.0 {
            reasons.push("成交额巨大，市场关注度高".to_string());
            score += 15.0;
        } else if turnover_ratio > 2.0 {
            score += 10.0;
        }
        
        // 5. 价格位置评分 (最高5分)
        if quote.price > 5.0 && quote.price < 50.0 {
            reasons.push("价格适中，适合参与".to_string());
            score += 5.0;
        }
        
        // 确定评级
        let level = if score >= 70.0 {
            "强烈推荐".to_string()
        } else if score >= 50.0 {
            "推荐".to_string()
        } else {
            "关注".to_string()
        };
        
        // 风险等级
        let risk_level = if score >= 60.0 && quote.change_pct.abs() < 5.0 {
            "低".to_string()
        } else if quote.change_pct.abs() < 8.0 {
            "中".to_string()
        } else {
            "高".to_string()
        };
        
        // 计算目标价和止损价
        let (target_price, stop_loss) = self.calc_prices(quote, score);
        
        StockRecommend {
            symbol: quote.symbol.clone(),
            name: quote.name.clone(),
            price: quote.price,
            change_pct: quote.change_pct,
            score: (score * 10.0).round() / 10.0,
            level,
            reasons,
            risk_level,
            target_price,
            stop_loss,
        }
    }

    /// 计算目标价和止损价
    fn calc_prices(&self, quote: &StockQuote, score: f64) -> (Option<f64>, Option<f64>) {
        let multiplier = if score >= 70.0 { 1.15 }    // 15% 上涨空间
                        else if score >= 50.0 { 1.10 }
                        else { 1.05 };
        
        let target = quote.price * multiplier;
        let stop = quote.price * 0.95;  // 5% 止损
        
        (Some((target * 100.0).round() / 100.0), Some((stop * 100.0).round() / 100.0))
    }
}
