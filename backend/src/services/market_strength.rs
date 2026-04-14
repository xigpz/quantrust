use anyhow::Result;
use crate::data::DataProvider;
use crate::models::{Candle, MarketOverview};
use crate::services::indicators::{self, macd, rsi, MacdResult};

/// 市场强弱状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MarketStrength {
    #[serde(rename = "strong")]
    Strong,      // 强势
    #[serde(rename = "weak")]
    Weak,        // 弱势
    #[serde(rename = "neutral")]
    Neutral,     // 震荡
}

impl std::fmt::Display for MarketStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketStrength::Strong => write!(f, "强势"),
            MarketStrength::Weak => write!(f, "弱势"),
            MarketStrength::Neutral => write!(f, "震荡"),
        }
    }
}

/// 市场强弱指标详情
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketStrengthResult {
    pub strength: MarketStrength,
    pub strength_score: f64,       // 0-100 评分
    pub up_count: i32,
    pub down_count: i32,
    pub flat_count: i32,
    pub up_down_ratio: f64,       // 上涨/下跌比率
    pub limit_up_count: i32,
    pub limit_down_count: i32,
    pub sh_price: f64,
    pub sz_price: f64,
    pub cyb_price: f64,
    pub sh_change_pct: f64,
    pub sz_change_pct: f64,
    pub cyb_change_pct: f64,
    // 5分钟指标
    pub ma5_above_ma20: bool,     // MA多头排列
    pub macd_bullish: bool,        // MACD看多
    pub rsi_value: f64,           // RSI值
    pub volume_ratio: f64,         // 量比（当前/均量）
    // 详细说明
    pub signals: Vec<String>,       // 触发信号列表
    pub summary: String,            // 简要总结
}

impl Default for MarketStrengthResult {
    fn default() -> Self {
        Self {
            strength: MarketStrength::Neutral,
            strength_score: 50.0,
            up_count: 0,
            down_count: 0,
            flat_count: 0,
            up_down_ratio: 1.0,
            limit_up_count: 0,
            limit_down_count: 0,
            sh_price: 0.0,
            sz_price: 0.0,
            cyb_price: 0.0,
            sh_change_pct: 0.0,
            sz_change_pct: 0.0,
            cyb_change_pct: 0.0,
            ma5_above_ma20: false,
            macd_bullish: false,
            rsi_value: 50.0,
            volume_ratio: 1.0,
            signals: vec![],
            summary: "数据加载中...".to_string(),
        }
    }
}

/// 市场强弱分析服务
pub struct MarketStrengthService {
    provider: DataProvider,
}

impl MarketStrengthService {
    pub fn new(provider: DataProvider) -> Self {
        Self { provider }
    }

    /// 分析市场强弱
    pub async fn analyze(&self) -> Result<MarketStrengthResult> {
        // 1. 获取市场概览（广度数据）
        let overview = self.provider.get_market_overview().await?;

        // 2. 获取上证5分钟K线
        let candles = self.provider.get_candles("1.000001", "5m", 100).await?;

        // 3. 计算技术指标（捕获panic）
        let (ma5_above_ma20, macd_bullish, rsi_value, volume_ratio) =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.calculate_indicators(&candles)
            })).unwrap_or((false, false, 50.0, 1.0));

        // 4. 综合判断
        let result = self.judge_strength(
            &overview,
            ma5_above_ma20,
            macd_bullish,
            rsi_value,
            volume_ratio,
        );

        Ok(result)
    }

    /// 计算5分钟级别技术指标
    fn calculate_indicators(&self, candles: &[Candle]) -> (bool, bool, f64, f64) {
        if candles.len() < 30 {
            return (false, false, 50.0, 1.0);
        }

        // MA5 和 MA20
        let ma5 = match indicators::sma(candles, 5, "close") {
            Ok(v) => v,
            Err(_) => vec![],
        };
        let ma20 = match indicators::sma(candles, 20, "close") {
            Ok(v) => v,
            Err(_) => vec![],
        };

        // MA多头排列：当前价格 > MA5 > MA20
        let ma5_above_ma20 = if ma5.len() >= 2 && ma20.len() >= 2 {
            let latest = candles.len() - 1;
            candles[latest].close > ma5[latest]
            && !ma5[latest].is_nan()
            && ma5[latest] > ma20[latest]
            && !ma20[latest].is_nan()
        } else {
            false
        };

        // MACD (12, 26, 9)
        let macd_result: MacdResult = if candles.len() >= 34 {
            macd(candles, 12, 26, 9, "close").unwrap_or(MacdResult {
                dif: vec![],
                dea: vec![],
                hist: vec![],
            })
        } else {
            MacdResult { dif: vec![], dea: vec![], hist: vec![] }
        };

        // MACD看多：DIF在DEA上方 且 hist > 0
        let macd_bullish = if !macd_result.hist.is_empty() {
            let latest = macd_result.hist.len() - 1;
            macd_result.hist[latest] > 0.0
            && !macd_result.hist[latest].is_nan()
        } else {
            false
        };

        // RSI (14周期)
        let rsi_value = if candles.len() >= 15 {
            match indicators::rsi(candles, 14, "close") {
                Ok(v) => v.last().copied().unwrap_or(50.0),
                Err(_) => 50.0,
            }
        } else {
            50.0
        };

        // 量比：最近5根K线平均成交量 / 前20根K线平均成交量
        let volume_ratio = self.calculate_volume_ratio(candles);

        (ma5_above_ma20, macd_bullish, rsi_value, volume_ratio)
    }

    /// 计算量比
    fn calculate_volume_ratio(&self, candles: &[Candle]) -> f64 {
        if candles.len() < 25 {
            return 1.0;
        }

        let recent_avg: f64 = candles[candles.len() - 5..]
            .iter()
            .map(|c| c.volume)
            .sum::<f64>()
            / 5.0;

        let history_avg: f64 = candles[candles.len() - 20..candles.len() - 5]
            .iter()
            .map(|c| c.volume)
            .sum::<f64>()
            / 15.0;

        if history_avg > 0.0 {
            recent_avg / history_avg
        } else {
            1.0
        }
    }

    /// 综合判断市场强弱
    fn judge_strength(
        &self,
        overview: &MarketOverview,
        ma5_above_ma20: bool,
        macd_bullish: bool,
        rsi_value: f64,
        volume_ratio: f64,
    ) -> MarketStrengthResult {
        let mut signals = Vec::new();
        let mut score: f64 = 50.0; // 基础分

        // 1. 上涨/下跌家数比率 (权重: 30分)
        let up_down_ratio = if overview.down_count > 0 {
            overview.up_count as f64 / overview.down_count as f64
        } else {
            overview.up_count as f64
        };

        if up_down_ratio >= 2.0 {
            score += 30.0;
            signals.push(format!("上涨/下跌 = {:.1}，市场强势", up_down_ratio));
        } else if up_down_ratio >= 1.5 {
            score += 20.0;
            signals.push(format!("上涨/下跌 = {:.1}，市场偏强", up_down_ratio));
        } else if up_down_ratio >= 1.0 {
            score += 10.0;
            signals.push(format!("上涨/下跌 = {:.1}，市场偏暖", up_down_ratio));
        } else if up_down_ratio < 0.67 {
            score -= 20.0;
            signals.push(format!("上涨/下跌 = {:.1}，市场偏弱", up_down_ratio));
        } else if up_down_ratio < 0.5 {
            score -= 30.0;
            signals.push(format!("上涨/下跌 = {:.1}，市场弱势", up_down_ratio));
        }

        // 2. 涨停家数 (权重: 20分)
        if overview.limit_up_count >= 50 {
            score += 20.0;
            signals.push(format!("涨停 {} 家，赚钱效应强", overview.limit_up_count));
        } else if overview.limit_up_count >= 30 {
            score += 15.0;
            signals.push(format!("涨停 {} 家，赚钱效应较好", overview.limit_up_count));
        } else if overview.limit_up_count >= 15 {
            score += 5.0;
            signals.push(format!("涨停 {} 家，赚钱效应一般", overview.limit_up_count));
        } else if overview.limit_up_count < 5 {
            score -= 15.0;
            signals.push(format!("涨停仅 {} 家，市场情绪低迷", overview.limit_up_count));
        }

        // 3. MA多头排列 (权重: 20分)
        if ma5_above_ma20 {
            score += 20.0;
            signals.push("MA5 > MA20，均线多头排列".to_string());
        } else {
            score -= 10.0;
            signals.push("MA5 < MA20，均线未多头排列".to_string());
        }

        // 4. MACD看多 (权重: 15分)
        if macd_bullish {
            score += 15.0;
            signals.push("MACD histogram > 0，红柱动能".to_string());
        } else {
            score -= 10.0;
            signals.push("MACD histogram < 0，绿柱动能".to_string());
        }

        // 5. RSI (权重: 10分)
        if rsi_value >= 60.0 {
            score += 10.0;
            signals.push(format!("RSI = {:.1}，多方强势", rsi_value));
        } else if rsi_value >= 50.0 {
            score += 5.0;
            signals.push(format!("RSI = {:.1}，多方略占优", rsi_value));
        } else if rsi_value < 40.0 {
            score -= 10.0;
            signals.push(format!("RSI = {:.1}，空方主导", rsi_value));
        }

        // 6. 量比 (权重: 5分)
        if volume_ratio >= 1.5 {
            score += 5.0;
            signals.push(format!("量比 = {:.1}，放量配合", volume_ratio));
        } else if volume_ratio < 0.8 {
            score -= 5.0;
            signals.push(format!("量比 = {:.1}，缩量不足", volume_ratio));
        }

        // 限制分数范围
        score = score.max(0.0).min(100.0);

        // 判断强弱
        let strength = if score >= 70.0 {
            MarketStrength::Strong
        } else if score <= 30.0 {
            MarketStrength::Weak
        } else {
            MarketStrength::Neutral
        };

        // 总结
        let summary = match strength {
            MarketStrength::Strong => format!(
                "市场强势信号：上涨家数远大于下跌家数（{}:{}），涨停{}家，技术面多头共振",
                overview.up_count, overview.down_count, overview.limit_up_count
            ),
            MarketStrength::Weak => format!(
                "市场弱势信号：下跌家数大于上涨家数（{}:{}），涨停仅{}家，技术面偏空",
                overview.up_count, overview.down_count, overview.limit_up_count
            ),
            MarketStrength::Neutral => format!(
                "市场震荡整理：涨跌家数接近（{}:{}），涨停{}家，等待方向明朗",
                overview.up_count, overview.down_count, overview.limit_up_count
            ),
        };

        MarketStrengthResult {
            strength,
            strength_score: score,
            up_count: overview.up_count,
            down_count: overview.down_count,
            flat_count: overview.flat_count,
            up_down_ratio,
            limit_up_count: overview.limit_up_count,
            limit_down_count: overview.limit_down_count,
            sh_price: overview.sh_index.price,
            sz_price: overview.sz_index.price,
            cyb_price: overview.cyb_index.price,
            sh_change_pct: overview.sh_index.change_pct,
            sz_change_pct: overview.sz_index.change_pct,
            cyb_change_pct: overview.cyb_index.change_pct,
            ma5_above_ma20,
            macd_bullish,
            rsi_value,
            volume_ratio,
            signals,
            summary,
        }
    }
}
