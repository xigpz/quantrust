use crate::models::*;
use crate::data::DataProvider;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 量化猎手 - 实时抓股引擎
pub struct QuantHunter {
    provider: DataProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HunterSignal {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
    pub signal_type: String,        // 信号类型
    pub signal_name: String,        // 信号名称
    pub score: f64,                 // 综合评分 0-100
    pub reasons: Vec<String>,       // 入场原因
    pub strength: String,            // 信号强度: strong/medium/weak
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HunterConfig {
    pub enable_momentum: bool,      // 动量策略
    pub enable_breakout: bool,       // 突破策略
    pub enable_volume: bool,         // 量价齐升
    pub enable_reversal: bool,      // 超跌反弹
    pub enable_ma_cross: bool,      // 均线金叉
    pub enable_high_turnover: bool, // 高换手率
    pub enable_limit_watch: bool,    // 涨停关注
    pub min_score: f64,             // 最低评分
}

impl Default for HunterConfig {
    fn default() -> Self {
        Self {
            enable_momentum: true,
            enable_breakout: true,
            enable_volume: true,
            enable_reversal: true,
            enable_ma_cross: false,
            enable_high_turnover: true,
            enable_limit_watch: true,
            min_score: 60.0,
        }
    }
}

impl QuantHunter {
    pub fn new(provider: DataProvider) -> Self {
        Self { provider }
    }

    /// 扫描全市场寻找符合条件的股票
    pub async fn hunt(&self, config: HunterConfig) -> Result<Vec<HunterSignal>, String> {
        // 获取实时行情
        let quotes = match self.provider.get_realtime_quotes(1, 200).await {
            Ok(q) => q,
            Err(e) => return Err(format!("获取行情失败: {}", e)),
        };

        let mut signals = Vec::new();

        for quote in quotes.iter().filter(|q| q.price > 0.0 && q.turnover > 0.0) {
            let score = self.calculate_score(quote, &config);

            if score >= config.min_score {
                let (signal_type, signal_name, reasons, strength) = self.analyze_signal(quote, &config);

                signals.push(HunterSignal {
                    symbol: quote.symbol.clone(),
                    name: quote.name.clone(),
                    price: quote.price,
                    change_pct: quote.change_pct,
                    signal_type,
                    signal_name,
                    score,
                    reasons,
                    strength,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });
            }
        }

        // 按评分排序
        signals.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        signals.truncate(50);

        Ok(signals)
    }

    /// 计算综合评分
    fn calculate_score(&self, quote: &StockQuote, config: &HunterConfig) -> f64 {
        let mut score = 0.0;
        let mut factors = 0.0;

        // 动量因子 (40%)
        if config.enable_momentum {
            let momentum_score = self.calc_momentum_score(quote);
            score += momentum_score * 0.4;
            factors += 0.4;
        }

        // 突破因子 (30%)
        if config.enable_breakout {
            let breakout_score = self.calc_breakout_score(quote);
            score += breakout_score * 0.3;
            factors += 0.3;
        }

        // 量价因子 (20%)
        if config.enable_volume {
            let volume_score = self.calc_volume_score(quote);
            score += volume_score * 0.2;
            factors += 0.2;
        }

        // 超跌反弹因子 (10%)
        if config.enable_reversal {
            let reversal_score = self.calc_reversal_score(quote);
            score += reversal_score * 0.1;
            factors += 0.1;
        }

        if factors > 0.0 {
            score / factors * 100.0
        } else {
            0.0
        }
    }

    /// 动量评分
    fn calc_momentum_score(&self, quote: &StockQuote) -> f64 {
        let mut score: f64 = 0.0;

        // 涨幅评分
        if quote.change_pct > 5.0 {
            score += 40.0;
        } else if quote.change_pct > 3.0 {
            score += 30.0;
        } else if quote.change_pct > 0.0 {
            score += 20.0;
        }

        // 换手率评分
        if quote.turnover_rate > 20.0 {
            score += 30.0;
        } else if quote.turnover_rate > 10.0 {
            score += 20.0;
        } else if quote.turnover_rate > 5.0 {
            score += 10.0;
        }

        // 振幅评分
        if quote.amplitude > 8.0 {
            score += 30.0;
        } else if quote.amplitude > 5.0 {
            score += 20.0;
        } else if quote.amplitude > 3.0 {
            score += 10.0;
        }

        score.min(100.0)
    }

    /// 突破评分
    fn calc_breakout_score(&self, quote: &StockQuote) -> f64 {
        let mut score: f64 = 0.0;

        // 接近涨停
        if quote.change_pct >= 9.5 {
            score += 50.0;
        } else if quote.change_pct >= 7.0 {
            score += 35.0;
        } else if quote.change_pct >= 5.0 {
            score += 25.0;
        }

        // 高成交额
        if quote.turnover > 1e9 {  // 10亿
            score += 30.0;
        } else if quote.turnover > 5e8 {  // 5亿
            score += 20.0;
        } else if quote.turnover > 1e8 {  // 1亿
            score += 10.0;
        }

        // 股价强度（相对市场中位数）
        if quote.change_pct > 5.0 && quote.turnover_rate > 10.0 {
            score += 20.0;
        }

        score.min(100.0)
    }

    /// 量价评分
    fn calc_volume_score(&self, quote: &StockQuote) -> f64 {
        let mut score: f64 = 0.0;

        // 成交额
        if quote.turnover > 5e8 {
            score += 40.0;
        } else if quote.turnover > 1e8 {
            score += 25.0;
        } else if quote.turnover > 5e7 {
            score += 15.0;
        }

        // 换手率
        if quote.turnover_rate > 30.0 {
            score += 35.0;
        } else if quote.turnover_rate > 15.0 {
            score += 25.0;
        } else if quote.turnover_rate > 8.0 {
            score += 15.0;
        }

        // 量价配合 (上涨且放量)
        if quote.change_pct > 0.0 && quote.turnover_rate > 10.0 {
            score += 25.0;
        }

        score.min(100.0)
    }

    /// 超跌反弹评分
    fn calc_reversal_score(&self, quote: &StockQuote) -> f64 {
        let mut score: f64 = 0.0;

        // 跌幅较大但开始反弹
        if quote.change_pct > -3.0 && quote.change_pct < 0.0 {
            score += 40.0;
        } else if quote.change_pct >= 0.0 && quote.change_pct < 3.0 {
            score += 30.0;
        }

        // 高换手率反弹
        if quote.turnover_rate > 15.0 {
            score += 35.0;
        } else if quote.turnover_rate > 8.0 {
            score += 20.0;
        }

        // 振幅大
        if quote.amplitude > 6.0 {
            score += 25.0;
        }

        score.min(100.0)
    }

    /// 分析信号类型
    fn analyze_signal(&self, quote: &StockQuote, config: &HunterConfig) -> (String, String, Vec<String>, String) {
        let mut reasons = Vec::new();

        // 涨停板信号
        if quote.change_pct >= 9.5 {
            return (
                "limit_up".to_string(),
                "涨停冲击".to_string(),
                vec!["涨停板".to_string(), format!("涨幅 {:.2}%", quote.change_pct)],
                "strong".to_string(),
            );
        }

        // 动量信号
        if config.enable_momentum && quote.change_pct > 5.0 && quote.turnover_rate > 10.0 {
            reasons.push(format!("强势上涨 {:.2}%", quote.change_pct));
            if quote.turnover_rate > 20.0 {
                reasons.push("高换手率".to_string());
            }
            if quote.amplitude > 8.0 {
                reasons.push("振幅较大".to_string());
            }
            return ("momentum".to_string(), "动量突破".to_string(), reasons, "strong".to_string());
        }

        // 量价齐升
        if config.enable_volume && quote.change_pct > 3.0 && quote.turnover_rate > 15.0 {
            reasons.push(format!("涨幅 {:.2}%", quote.change_pct));
            reasons.push(format!("换手率 {:.1}%", quote.turnover_rate));
            if quote.turnover > 1e8 {
                reasons.push("成交额放大".to_string());
            }
            return ("volume".to_string(), "量价齐升".to_string(), reasons, "medium".to_string());
        }

        // 超跌反弹
        if config.enable_reversal && quote.change_pct > -2.0 && quote.change_pct < 2.0 && quote.turnover_rate > 10.0 {
            reasons.push("低位反弹".to_string());
            reasons.push(format!("换手率 {:.1}%", quote.turnover_rate));
            return ("reversal".to_string(), "超跌反弹".to_string(), reasons, "medium".to_string());
        }

        // 突破信号
        if config.enable_breakout && quote.change_pct > 3.0 && quote.turnover > 5e7 {
            reasons.push(format!("涨幅 {:.2}%", quote.change_pct));
            reasons.push("资金活跃".to_string());
            return ("breakout".to_string(), "技术突破".to_string(), reasons, "weak".to_string());
        }

        // 高换手率信号
        if config.enable_high_turnover && quote.turnover_rate > 25.0 {
            reasons.push(format!("换手率 {:.1}%", quote.turnover_rate));
            if quote.change_pct > 0.0 {
                reasons.push("放量上涨".to_string());
            }
            return ("high_turnover".to_string(), "高换手率".to_string(), reasons, "medium".to_string());
        }

        // 涨停关注（未涨停但接近涨停）
        if config.enable_limit_watch && quote.change_pct >= 7.0 && quote.change_pct < 9.5 {
            reasons.push(format!("涨幅 {:.2}%", quote.change_pct));
            reasons.push("接近涨停".to_string());
            return ("limit_watch".to_string(), "涨停关注".to_string(), reasons, "medium".to_string());
        }

        // 默认信号
        reasons.push(format!("涨幅 {:.2}%", quote.change_pct));
        ("watch".to_string(), "观察".to_string(), reasons, "weak".to_string())
    }
}
