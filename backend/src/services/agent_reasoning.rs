//! 推理引擎 (Reasoning Engine)
//!
//! 实现三层推理架构：快思考、慢思考、深思考

use serde::{Deserialize, Serialize};
use super::agent_perception::{
    MarketData, NewsData, RealtimeData, StockPrice,
};

/// 交易信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSignal {
    pub symbol: String,
    pub name: String,
    pub action: TradeAction,
    pub price: f64,
    pub quantity: f64,
    pub confidence: f64,       // 置信度 0-1
    pub reason: String,        // 决策理由
    pub source: SignalSource,  // 信号来源
    pub timestamp: String,
}

/// 交易动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeAction {
    Buy,
    Sell,
    Hold,
}

/// 信号来源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalSource {
    FastThink,   // 快思考
    SlowThink,   // 慢思考
    DeepThink,   // 深思考
}

/// 候选股票
#[derive(Debug, Clone)]
pub struct CandidateStock {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
    pub volume_ratio: f64,
    pub breakout_type: BreakoutType,
    pub sector_lead: bool,
    pub news_sentiment: f64,
    pub score: f64,
}

/// 突破类型
#[derive(Debug, Clone)]
pub enum BreakoutType {
    PriceBreakout,     // 价格突破
    VolumeBreakout,    // 量能突破
    SectorBreakout,    // 板块带动
    NewsBreakout,      // 消息驱动
}

/// 交易历史 (用于深思考分析)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeHistory {
    pub trades: Vec<TradeRecord>,
}

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub symbol: String,
    pub name: String,
    pub action: String,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: String,
    pub reason: String,
    pub pnl: Option<f64>,  // 盈亏 (如果是卖出)
}

/// 深思考分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepAnalysis {
    pub summary: String,
    pub strategy_performance: StrategyPerformance,
    pub observations: Vec<String>,
    pub recommendations: Vec<String>,
}

/// 策略表现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub total_trades: i32,
    pub win_rate: f64,
    pub avg_holding_days: f64,
    pub max_drawdown: f64,
    pub profit_factor: f64,
}

/// 次日展望
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomorrowOutlook {
    pub focus_sectors: Vec<String>,
    pub watch_list: Vec<String>,
    pub risk_factors: Vec<String>,
    pub ai_view: String,
}

/// 推理引擎
pub struct ReasoningEngine {
    // 可配置的交易策略参数
    min_breakout_ratio: f64,     // 最小突破幅度
    min_volume_ratio: f64,       // 最小量比
    min_confidence: f64,         // 最小置信度
    max_position_ratio: f64,     // 最大仓位比例
}

impl ReasoningEngine {
    pub fn new() -> Self {
        Self {
            min_breakout_ratio: 0.03,   // 3% 突破
            min_volume_ratio: 1.5,      // 1.5倍量能
            min_confidence: 0.6,        // 60% 置信度
            max_position_ratio: 0.2,     // 20% 仓位
        }
    }

    /// 综合分析 (感知 -> 推理)
    pub async fn analyze(&self, market: &MarketData, news: &[NewsData]) -> Vec<TradeSignal> {
        let mut signals = vec![];

        // 基于新闻情感分析
        for n in news {
            if n.is_hot {
                for stock in &n.related_stocks {
                    signals.push(TradeSignal {
                        symbol: stock.clone(),
                        name: stock.clone(),
                        action: TradeAction::Buy,
                        price: 0.0,
                        quantity: 0.0,
                        confidence: 0.7,
                        reason: format!("热点新闻驱动: {}", n.title),
                        source: SignalSource::FastThink,
                        timestamp: n.timestamp.clone(),
                    });
                }
            }
        }

        signals
    }

    /// 快思考 - 规则引擎筛选
    pub async fn fast_think(&self, realtime: &RealtimeData) -> Vec<CandidateStock> {
        let mut candidates = vec![];

        // 检测异常成交量
        for uv in &realtime.unusual_volumes {
            if uv.volume_ratio >= self.min_volume_ratio {
                candidates.push(CandidateStock {
                    symbol: uv.symbol.clone(),
                    name: uv.name.clone(),
                    price: 0.0,
                    change_pct: 0.0,
                    volume_ratio: uv.volume_ratio,
                    breakout_type: BreakoutType::VolumeBreakout,
                    sector_lead: false,
                    news_sentiment: 0.0,
                    score: uv.volume_ratio * 10.0,
                });
            }
        }

        // 检测价格突破
        for (symbol, price) in &realtime.prices {
            if price.change_pct >= self.min_breakout_ratio * 100.0 && price.volume_ratio >= self.min_volume_ratio {
                candidates.push(CandidateStock {
                    symbol: symbol.clone(),
                    name: price.name.clone(),
                    price: price.price,
                    change_pct: price.change_pct,
                    volume_ratio: price.volume_ratio,
                    breakout_type: BreakoutType::PriceBreakout,
                    sector_lead: false,
                    news_sentiment: 0.0,
                    score: price.change_pct + price.volume_ratio * 5.0,
                });
            }
        }

        // 按评分排序
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        candidates.truncate(20); // 最多保留20个候选

        candidates
    }

    /// 慢思考 - 多因子验证
    pub async fn slow_think(&self, candidates: &[CandidateStock]) -> Vec<TradeSignal> {
        let mut signals = vec![];

        for c in candidates {
            // 多因子评分
            let mut total_score = c.score;

            // 板块龙头加成
            if c.sector_lead {
                total_score *= 1.2;
            }

            // 新闻情绪加成
            if c.news_sentiment > 0.3 {
                total_score *= 1.1;
            } else if c.news_sentiment < -0.3 {
                total_score *= 0.8;
            }

            let confidence = (total_score / 100.0).min(0.95);

            if confidence >= self.min_confidence {
                signals.push(TradeSignal {
                    symbol: c.symbol.clone(),
                    name: c.name.clone(),
                    action: TradeAction::Buy,
                    price: c.price,
                    quantity: 0.0, // 由执行器计算
                    confidence,
                    reason: format!(
                        "{:?}突破, 量比{:.1}倍, 评分{:.1}",
                        c.breakout_type, c.volume_ratio, total_score
                    ),
                    source: SignalSource::SlowThink,
                    timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                });
            }
        }

        signals
    }

    /// 深思考 - LLM 推理 (需要人工调用 LLM API)
    pub async fn deep_think(&self, _trades: &TradeHistory) -> DeepAnalysis {
        // TODO: 集成 LLM API 进行深度分析
        // 这里先返回基础分析
        DeepAnalysis {
            summary: "市场震荡，AI 策略表现稳健".to_string(),
            strategy_performance: StrategyPerformance {
                total_trades: 0,
                win_rate: 0.0,
                avg_holding_days: 0.0,
                max_drawdown: 0.0,
                profit_factor: 0.0,
            },
            observations: vec![
                "热点板块轮动加快".to_string(),
                "量能有所萎缩".to_string(),
            ],
            recommendations: vec![
                "控制仓位，谨慎追涨".to_string(),
                "关注业绩预增个股".to_string(),
            ],
        }
    }

    /// 生成次日展望
    pub async fn generate_outlook(&self) -> TomorrowOutlook {
        // TODO: 基于当日分析生成次日展望
        TomorrowOutlook {
            focus_sectors: vec!["新能源".to_string(), "半导体".to_string()],
            watch_list: vec![],
            risk_factors: vec!["外盘波动".to_string(), "政策变化".to_string()],
            ai_view: "市场情绪有所回暖，建议关注政策受益板块".to_string(),
        }
    }

    /// 设置置信度阈值
    pub fn set_min_confidence(&mut self, confidence: f64) {
        self.min_confidence = confidence;
    }
}

impl Default for ReasoningEngine {
    fn default() -> Self {
        Self::new()
    }
}
