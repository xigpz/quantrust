use crate::models::*;
use chrono::Utc;

/// 异动检测引擎
/// 通过多维度指标实时检测股票异动
pub struct AnomalyDetector {
    /// 成交量突增阈值 (相对于平均值的倍数)
    volume_spike_threshold: f64,
    /// 急速拉升阈值 (涨幅百分比)
    price_surge_threshold: f64,
    /// 急速下跌阈值 (跌幅百分比)
    price_drop_threshold: f64,
    /// 换手率突增阈值
    turnover_spike_threshold: f64,
    /// 涨停阈值
    limit_up_threshold: f64,
    /// 跌停阈值
    limit_down_threshold: f64,
    /// 跳空高开阈值
    gap_up_threshold: f64,
    /// 跳空低开阈值
    gap_down_threshold: f64,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            volume_spike_threshold: 3.0,
            price_surge_threshold: 5.0,
            price_drop_threshold: -5.0,
            turnover_spike_threshold: 10.0,
            limit_up_threshold: 9.8,
            limit_down_threshold: -9.8,
            gap_up_threshold: 3.0,
            gap_down_threshold: -3.0,
        }
    }

    /// 对一批股票行情进行异动检测
    pub fn detect(&self, quotes: &[StockQuote], history: &[StockQuote]) -> Vec<AnomalyStock> {
        let mut anomalies = Vec::new();

        for quote in quotes {
            // 跳过无效数据
            if quote.price <= 0.0 || quote.pre_close <= 0.0 {
                continue;
            }

            let detected = self.detect_single(quote, history);
            anomalies.extend(detected);
        }

        // 按异动评分排序
        anomalies.sort_by(|a, b| b.anomaly_score.partial_cmp(&a.anomaly_score).unwrap_or(std::cmp::Ordering::Equal));
        anomalies
    }

    /// 检测单只股票的异动
    fn detect_single(&self, quote: &StockQuote, _history: &[StockQuote]) -> Vec<AnomalyStock> {
        let mut anomalies = Vec::new();

        // 1. 涨停检测
        if quote.change_pct >= self.limit_up_threshold {
            anomalies.push(AnomalyStock {
                symbol: quote.symbol.clone(),
                name: quote.name.clone(),
                price: quote.price,
                change_pct: quote.change_pct,
                anomaly_type: AnomalyType::LimitUp,
                anomaly_score: 100.0,
                description: format!("涨停! 涨幅 {:.2}%", quote.change_pct),
                volume: quote.volume,
                turnover_rate: quote.turnover_rate,
                timestamp: Utc::now(),
            });
        }

        // 2. 跌停检测
        if quote.change_pct <= self.limit_down_threshold {
            anomalies.push(AnomalyStock {
                symbol: quote.symbol.clone(),
                name: quote.name.clone(),
                price: quote.price,
                change_pct: quote.change_pct,
                anomaly_type: AnomalyType::LimitDown,
                anomaly_score: 100.0,
                description: format!("跌停! 跌幅 {:.2}%", quote.change_pct),
                volume: quote.volume,
                turnover_rate: quote.turnover_rate,
                timestamp: Utc::now(),
            });
        }

        // 3. 急速拉升检测
        if quote.change_pct >= self.price_surge_threshold && quote.change_pct < self.limit_up_threshold {
            let score = (quote.change_pct / self.price_surge_threshold) * 50.0;
            anomalies.push(AnomalyStock {
                symbol: quote.symbol.clone(),
                name: quote.name.clone(),
                price: quote.price,
                change_pct: quote.change_pct,
                anomaly_type: AnomalyType::PriceSurge,
                anomaly_score: score.min(95.0),
                description: format!("急速拉升! 涨幅 {:.2}%", quote.change_pct),
                volume: quote.volume,
                turnover_rate: quote.turnover_rate,
                timestamp: Utc::now(),
            });
        }

        // 4. 急速下跌检测
        if quote.change_pct <= self.price_drop_threshold && quote.change_pct > self.limit_down_threshold {
            let score = (quote.change_pct.abs() / self.price_drop_threshold.abs()) * 50.0;
            anomalies.push(AnomalyStock {
                symbol: quote.symbol.clone(),
                name: quote.name.clone(),
                price: quote.price,
                change_pct: quote.change_pct,
                anomaly_type: AnomalyType::PriceDrop,
                anomaly_score: score.min(95.0),
                description: format!("急速下跌! 跌幅 {:.2}%", quote.change_pct),
                volume: quote.volume,
                turnover_rate: quote.turnover_rate,
                timestamp: Utc::now(),
            });
        }

        // 5. 换手率异常检测
        if quote.turnover_rate >= self.turnover_spike_threshold {
            let score = (quote.turnover_rate / self.turnover_spike_threshold) * 40.0;
            anomalies.push(AnomalyStock {
                symbol: quote.symbol.clone(),
                name: quote.name.clone(),
                price: quote.price,
                change_pct: quote.change_pct,
                anomaly_type: AnomalyType::TurnoverSpike,
                anomaly_score: score.min(80.0),
                description: format!("换手率异常! 换手率 {:.2}%", quote.turnover_rate),
                volume: quote.volume,
                turnover_rate: quote.turnover_rate,
                timestamp: Utc::now(),
            });
        }

        // 6. 振幅异常检测 (高振幅可能意味着大单异动)
        if quote.amplitude >= 8.0 {
            let score = (quote.amplitude / 8.0) * 35.0;
            anomalies.push(AnomalyStock {
                symbol: quote.symbol.clone(),
                name: quote.name.clone(),
                price: quote.price,
                change_pct: quote.change_pct,
                anomaly_type: AnomalyType::LargeOrder,
                anomaly_score: score.min(70.0),
                description: format!("振幅异常! 振幅 {:.2}%，可能存在大单异动", quote.amplitude),
                volume: quote.volume,
                turnover_rate: quote.turnover_rate,
                timestamp: Utc::now(),
            });
        }

        // 7. 跳空高开检测
        if quote.pre_close > 0.0 {
            let gap_pct = (quote.open - quote.pre_close) / quote.pre_close * 100.0;
            if gap_pct >= self.gap_up_threshold {
                let score = (gap_pct / self.gap_up_threshold) * 40.0;
                anomalies.push(AnomalyStock {
                    symbol: quote.symbol.clone(),
                    name: quote.name.clone(),
                    price: quote.price,
                    change_pct: quote.change_pct,
                    anomaly_type: AnomalyType::GapUp,
                    anomaly_score: score.min(75.0),
                    description: format!("跳空高开 {:.2}%", gap_pct),
                    volume: quote.volume,
                    turnover_rate: quote.turnover_rate,
                    timestamp: Utc::now(),
                });
            }

            // 8. 跳空低开检测
            if gap_pct <= self.gap_down_threshold {
                let score = (gap_pct.abs() / self.gap_down_threshold.abs()) * 40.0;
                anomalies.push(AnomalyStock {
                    symbol: quote.symbol.clone(),
                    name: quote.name.clone(),
                    price: quote.price,
                    change_pct: quote.change_pct,
                    anomaly_type: AnomalyType::GapDown,
                    anomaly_score: score.min(75.0),
                    description: format!("跳空低开 {:.2}%", gap_pct),
                    volume: quote.volume,
                    turnover_rate: quote.turnover_rate,
                    timestamp: Utc::now(),
                });
            }
        }

        anomalies
    }
}
