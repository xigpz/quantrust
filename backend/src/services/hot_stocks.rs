use crate::models::*;
use chrono::Utc;

/// 热点股票排名引擎
/// 综合成交额、换手率、涨跌幅、振幅等多维度指标计算热度评分
pub struct HotStockRanker;

impl HotStockRanker {
    pub fn new() -> Self {
        Self
    }

    /// 从行情数据中计算热点股票排名
    pub fn rank(&self, quotes: &[StockQuote], top_n: usize) -> Vec<HotStock> {
        let mut hot_stocks: Vec<HotStock> = quotes
            .iter()
            .filter(|q| q.price > 0.0 && q.turnover > 0.0)
            .map(|q| {
                let (score, reason) = self.calculate_hot_score(q);
                HotStock {
                    symbol: q.symbol.clone(),
                    name: q.name.clone(),
                    price: q.price,
                    change_pct: q.change_pct,
                    volume: q.volume,
                    turnover: q.turnover,
                    turnover_rate: q.turnover_rate,
                    hot_score: score,
                    hot_reason: reason,
                    timestamp: Utc::now(),
                }
            })
            .collect();

        // 按热度评分降序排列
        hot_stocks.sort_by(|a, b| b.hot_score.partial_cmp(&a.hot_score).unwrap_or(std::cmp::Ordering::Equal));
        hot_stocks.truncate(top_n);
        hot_stocks
    }

    /// 计算单只股票的热度评分
    /// 综合考虑: 成交额(40%) + 换手率(25%) + 涨跌幅绝对值(20%) + 振幅(15%)
    fn calculate_hot_score(&self, quote: &StockQuote) -> (f64, String) {
        let mut score = 0.0;
        let mut reasons = Vec::new();

        // 1. 成交额得分 (0-40分)
        // 成交额越大，热度越高
        let turnover_score = if quote.turnover > 0.0 {
            let turnover_billion = quote.turnover / 1e8; // 转换为亿
            (turnover_billion.ln().max(0.0) / 5.0 * 40.0).min(40.0)
        } else {
            0.0
        };
        score += turnover_score;
        if turnover_score > 25.0 {
            reasons.push(format!("成交额 {:.1}亿", quote.turnover / 1e8));
        }

        // 2. 换手率得分 (0-25分)
        let turnover_rate_score = (quote.turnover_rate / 20.0 * 25.0).min(25.0);
        score += turnover_rate_score;
        if quote.turnover_rate > 5.0 {
            reasons.push(format!("换手率 {:.1}%", quote.turnover_rate));
        }

        // 3. 涨跌幅绝对值得分 (0-20分)
        let change_score = (quote.change_pct.abs() / 10.0 * 20.0).min(20.0);
        score += change_score;
        if quote.change_pct.abs() > 5.0 {
            if quote.change_pct > 0.0 {
                reasons.push(format!("涨幅 +{:.2}%", quote.change_pct));
            } else {
                reasons.push(format!("跌幅 {:.2}%", quote.change_pct));
            }
        }

        // 4. 振幅得分 (0-15分)
        let amplitude_score = (quote.amplitude / 10.0 * 15.0).min(15.0);
        score += amplitude_score;
        if quote.amplitude > 6.0 {
            reasons.push(format!("振幅 {:.1}%", quote.amplitude));
        }

        // 涨停/跌停加分
        if quote.change_pct >= 9.8 {
            score += 20.0;
            reasons.insert(0, "涨停".to_string());
        } else if quote.change_pct <= -9.8 {
            score += 15.0;
            reasons.insert(0, "跌停".to_string());
        }

        let reason = if reasons.is_empty() {
            "活跃交易".to_string()
        } else {
            reasons.join(", ")
        };

        (score, reason)
    }
}
