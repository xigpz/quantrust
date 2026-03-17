use crate::models::*;
use serde::{Deserialize, Serialize};
use std::env;

/// AI形态分析服务 - 使用MiniMax API分析股票形态
pub struct AIPatternService {
    api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternResult {
    pub symbol: String,
    pub name: String,
    pub pattern_type: PatternType,       // 形态类型
    pub consolidation_prob: f64,       // 横盘概率
    pub breakout_direction: BreakoutDirection, // 突破方向
    pub trend: TrendType,               // 趋势判断
    pub support_level: f64,             // 支撑位
    pub resistance_level: f64,           // 压力位
    pub analysis_text: String,          // 分析文字
    pub confidence: f64,                // 置信度
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PatternType {
    #[default]
    Consolidation,  // 横盘
    Breakout,       // 突破
    Uptrend,        // 上涨趋势
    Downtrend,      // 下跌趋势
    Volatile,       // 震荡
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum BreakoutDirection {
    #[default]
    Up,     // 向上突破
    Down,   // 向下突破
    Neutral, // 方向不明
    None,   // 无突破
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TrendType {
    #[default]
    Bullish,    // 看涨
    Bearish,    // 看跌
    Sideways,   // 震荡
    Strong,     // 强势
    Weak,       // 弱势
}

#[derive(Debug, Serialize, Deserialize)]
struct MiniMaxRequest {
    model: String,
    messages: Vec<MiniMaxMessage>,
    temperature: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MiniMaxMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct MiniMaxResponse {
    choices: Vec<MiniMaxChoice>,
}

#[derive(Debug, Deserialize)]
struct MiniMaxChoice {
    message: MiniMaxResponseMessage,
}

#[derive(Debug, Deserialize)]
struct MiniMaxResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct PatternAnalysisResponse {
    pattern_type: String,
    consolidation_prob: f64,
    breakout_direction: String,
    trend: String,
    support_level: f64,
    resistance_level: f64,
    analysis_text: String,
    confidence: f64,
}

impl AIPatternService {
    pub fn new() -> Self {
        let api_key = env::var("MINIMAX_API_KEY").unwrap_or_default();
        Self { api_key }
    }

    pub fn new_with_key(api_key: &str) -> Self {
        Self { api_key: api_key.to_string() }
    }

    /// 分析单只股票形态
    pub async fn analyze_pattern(&self, symbol: &str, name: &str, candles: &[Candle]) -> PatternResult {
        // 提取技术指标
        let indicators = self.calculate_indicators(candles);

        // 构建Prompt
        let prompt = self.build_analysis_prompt(symbol, name, candles, &indicators);

        // 调用MiniMax API
        if self.api_key.is_empty() {
            // 如果没有API Key，返回基于规则的分析结果
            return self.rule_based_analysis(symbol, name, candles, &indicators);
        }

        match self.call_minimax(&prompt).await {
            Ok(analysis) => PatternResult {
                symbol: symbol.to_string(),
                name: name.to_string(),
                pattern_type: self.parse_pattern_type(&analysis.pattern_type),
                consolidation_prob: analysis.consolidation_prob,
                breakout_direction: self.parse_breakout_direction(&analysis.breakout_direction),
                trend: self.parse_trend(&analysis.trend),
                support_level: analysis.support_level,
                resistance_level: analysis.resistance_level,
                analysis_text: analysis.analysis_text,
                confidence: analysis.confidence,
            },
            Err(_) => {
                // API调用失败时使用规则分析
                self.rule_based_analysis(symbol, name, candles, &indicators)
            }
        }
    }

    /// 计算技术指标
    fn calculate_indicators(&self, candles: &[Candle]) -> TechnicalIndicators {
        if candles.is_empty() {
            return TechnicalIndicators::default();
        }

        let prices: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let volumes: Vec<f64> = candles.iter().map(|c| c.volume).collect();

        // 计算120日振幅
        let (min_price, max_price) = prices.iter().fold((f64::MAX, f64::MIN), |(min, max), &p| {
            (min.min(p), max.max(p))
        });
        let amplitude_120d = if min_price > 0.0 {
            (max_price - min_price) / min_price * 100.0
        } else {
            0.0
        };

        // 计算价格变化
        let start_price = prices.first().copied().unwrap_or(0.0);
        let end_price = prices.last().copied().unwrap_or(0.0);
        let price_change = if start_price > 0.0 {
            (end_price - start_price) / start_price * 100.0
        } else {
            0.0
        };

        // 计算均线
        let ma5 = self.moving_average(&prices, 5);
        let ma10 = self.moving_average(&prices, 10);
        let ma20 = self.moving_average(&prices, 20);
        let ma60 = self.moving_average(&prices, 60);

        // 计算成交量变化
        let avg_volume = volumes.iter().sum::<f64>() / volumes.len() as f64;
        let recent_volume_ratio = if avg_volume > 0.0 && volumes.len() >= 5 {
            let recent_avg = volumes.iter().rev().take(5).sum::<f64>() / 5.0;
            recent_avg / avg_volume
        } else {
            1.0
        };

        // 计算波动率
        let volatility = self.calculate_volatility(&prices);

        TechnicalIndicators {
            amplitude_120d,
            price_change,
            ma5,
            ma10,
            ma20,
            ma60,
            avg_volume,
            recent_volume_ratio,
            volatility,
            current_price: end_price,
        }
    }

    fn moving_average(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() < period {
            return prices.iter().sum::<f64>() / prices.len() as f64;
        }
        prices.iter().rev().take(period).sum::<f64>() / period as f64
    }

    fn calculate_volatility(&self, prices: &[f64]) -> f64 {
        if prices.len() < 2 {
            return 0.0;
        }
        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;
        variance.sqrt() / mean * 100.0
    }

    fn build_analysis_prompt(&self, symbol: &str, name: &str, candles: &[Candle], indicators: &TechnicalIndicators) -> String {
        format!(r#"请分析股票 {} ({}) 的技术形态。

技术指标数据：
- 120日振幅：{:.2}%
- 近120日涨跌幅：{:.2}%
- 当前价格：{:.2}
- 5日均线：{:.2}
- 10日均线：{:.2}
- 20日均线：{:.2}
- 60日均线：{:.2}
- 平均成交量：{:.0}
- 近期成交量比：{:.2}
- 波动率：{:.2}%

请根据以上数据，分析该股票：
1. 形态类型（横盘/突破/上涨趋势/下跌趋势/震荡）
2. 横盘概率（0-100）
3. 突破方向（向上/向下/无/方向不明）
4. 趋势判断（看涨/看跌/震荡/强势/弱势）
5. 支撑位
6. 压力位
7. 置信度（0-1）

请以JSON格式返回分析结果，格式如下：
{{
    "pattern_type": "形态类型",
    "consolidation_prob": 横盘概率数值,
    "breakout_direction": "突破方向",
    "trend": "趋势判断",
    "support_level": 支撑位数值,
    "resistance_level": 压力位数值,
    "analysis_text": "简要分析说明",
    "confidence": 置信度数值
}}"#, symbol, name, indicators.amplitude_120d, indicators.price_change, indicators.current_price, indicators.ma5, indicators.ma10, indicators.ma20, indicators.ma60, indicators.avg_volume, indicators.recent_volume_ratio, indicators.volatility)
    }

    async fn call_minimax(&self, prompt: &str) -> Result<PatternAnalysisResponse, String> {
        let client = reqwest::Client::new();

        let request = MiniMaxRequest {
            model: "abab6.5s-chat".to_string(),
            messages: vec![
                MiniMaxMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
            temperature: 0.7,
        };

        let response = client
            .post("https://api.minimax.chat/v1/text/chatcompletion_v2")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let result: MiniMaxResponse = response.json().await.map_err(|e| e.to_string())?;

        let content = result.choices.first()
            .map(|c| c.message.content.clone())
            .ok_or("No response from API")?;

        // 解析JSON响应
        let json_str = content.trim();
        let parsed: PatternAnalysisResponse = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(parsed)
    }

    fn parse_pattern_type(&self, s: &str) -> PatternType {
        match s {
            "横盘" | "consolidation" => PatternType::Consolidation,
            "突破" | "breakout" => PatternType::Breakout,
            "上涨趋势" | "uptrend" | "上涨" => PatternType::Uptrend,
            "下跌趋势" | "downtrend" | "下跌" => PatternType::Downtrend,
            "震荡" | "volatile" => PatternType::Volatile,
            _ => PatternType::Unknown,
        }
    }

    fn parse_breakout_direction(&self, s: &str) -> BreakoutDirection {
        match s {
            "向上" | "向上突破" | "up" => BreakoutDirection::Up,
            "向下" | "向下突破" | "down" => BreakoutDirection::Down,
            "方向不明" | "neutral" => BreakoutDirection::Neutral,
            _ => BreakoutDirection::None,
        }
    }

    fn parse_trend(&self, s: &str) -> TrendType {
        match s {
            "看涨" | "bullish" | "上涨" => TrendType::Bullish,
            "看跌" | "bearish" | "下跌" => TrendType::Bearish,
            "震荡" | "sideways" => TrendType::Sideways,
            "强势" | "strong" => TrendType::Strong,
            "弱势" | "weak" => TrendType::Weak,
            _ => TrendType::Sideways,
        }
    }

    /// 基于规则的分析（当API不可用时）
    fn rule_based_analysis(&self, symbol: &str, name: &str, candles: &[Candle], indicators: &TechnicalIndicators) -> PatternResult {
        let pattern_type = if indicators.amplitude_120d < 25.0 {
            PatternType::Consolidation
        } else if indicators.price_change > 10.0 {
            PatternType::Uptrend
        } else if indicators.price_change < -10.0 {
            PatternType::Downtrend
        } else {
            PatternType::Volatile
        };

        let consolidation_prob = if indicators.amplitude_120d < 15.0 {
            80.0
        } else if indicators.amplitude_120d < 25.0 {
            60.0
        } else {
            30.0
        };

        let breakout_direction = if indicators.current_price > indicators.ma20 && indicators.current_price > indicators.ma5 {
            BreakoutDirection::Up
        } else if indicators.current_price < indicators.ma20 && indicators.current_price < indicators.ma5 {
            BreakoutDirection::Down
        } else {
            BreakoutDirection::Neutral
        };

        let trend = if indicators.price_change > 5.0 {
            TrendType::Bullish
        } else if indicators.price_change < -5.0 {
            TrendType::Bearish
        } else if indicators.amplitude_120d < 20.0 {
            TrendType::Sideways
        } else {
            TrendType::Weak
        };

        let support_level = indicators.current_price * 0.95;
        let resistance_level = indicators.current_price * 1.05;

        let analysis_text = format!(
            "120日振幅{:.1}%，近期涨跌幅{:.1}%，当前价格{:.2}",
            indicators.amplitude_120d, indicators.price_change, indicators.current_price
        );

        PatternResult {
            symbol: symbol.to_string(),
            name: name.to_string(),
            pattern_type,
            consolidation_prob,
            breakout_direction,
            trend,
            support_level,
            resistance_level,
            analysis_text,
            confidence: 0.7,
        }
    }

    /// 筛选横盘股票
    pub fn screen_consolidation(&self, candles: &[Candle], max_amplitude: f64, days: usize) -> bool {
        if candles.len() < days {
            return false;
        }

        let recent_candles = &candles[candles.len().saturating_sub(days)..];
        let prices: Vec<f64> = recent_candles.iter().map(|c| c.close).collect();

        if prices.is_empty() {
            return false;
        }

        let min_price = prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_price = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if min_price <= 0.0 {
            return false;
        }

        let amplitude = (max_price - min_price) / min_price * 100.0;
        amplitude <= max_amplitude
    }
}

#[derive(Debug, Default)]
struct TechnicalIndicators {
    amplitude_120d: f64,
    price_change: f64,
    current_price: f64,
    ma5: f64,
    ma10: f64,
    ma20: f64,
    ma60: f64,
    avg_volume: f64,
    recent_volume_ratio: f64,
    volatility: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenParams {
    pub pattern_type: Option<String>,   // 形态类型筛选
    pub max_amplitude: Option<f64>,      // 最大振幅
    pub days: Option<usize>,             // 筛选天数
    pub min_consolidation_prob: Option<f64>, // 最小横盘概率
    pub trend: Option<String>,           // 趋势筛选
    pub limit: Option<usize>,            // 返回数量限制
}

impl Default for ScreenParams {
    fn default() -> Self {
        Self {
            pattern_type: None,
            max_amplitude: Some(25.0),
            days: Some(120),
            min_consolidation_prob: None,
            trend: None,
            limit: Some(50),
        }
    }
}
