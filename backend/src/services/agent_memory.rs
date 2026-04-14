//! 记忆系统 (Memory System)
//!
//! 实现四层记忆架构：短期、情景、语义、长期

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{Local, Utc};

/// 短期记忆 (Working Memory) - TTL: 5-30分钟
#[derive(Debug, Clone)]
pub struct WorkingMemory {
    /// 当前持仓
    pub positions: HashMap<String, PositionInfo>,
    /// 浮盈浮亏
    pub floating_pnl: f64,
    /// 关注列表
    pub watch_list: Vec<String>,
    /// 最后更新时间
    pub updated_at: String,
}

/// 持仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionInfo {
    pub symbol: String,
    pub name: String,
    pub quantity: f64,
    pub avg_cost: f64,
    pub current_price: f64,
    pub profit_loss: f64,
    pub profit_ratio: f64,
}

/// 情景记忆 (Episodic Memory) - TTL: 1-7天
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    /// 交易历史
    pub trades: VecDeque<EpisodicTrade>,
    /// 最大容量
    max_size: usize,
}

/// 单条情景记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicTrade {
    pub id: String,
    pub timestamp: String,
    pub symbol: String,
    pub name: String,
    pub action: String,        // buy | sell
    pub price: f64,
    pub quantity: f64,
    pub reason: String,        // 买入/卖出理由
    pub context: String,       // 当时的上下文
    pub result: Option<f64>,   // 交易结果 (卖出时记录盈亏)
}

/// 语义记忆 (Semantic Memory) - 永久
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    /// 股票知识图谱
    pub stock_knowledge: HashMap<String, StockKnowledge>,
    /// 板块关系
    pub sector_relations: HashMap<String, SectorInfo>,
}

/// 股票知识
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockKnowledge {
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub industry: String,
    pub concepts: Vec<String>,
    pub characteristics: String,  // 股票特性描述
}

/// 板块信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorInfo {
    pub name: String,
    pub stocks: Vec<String>,
    pub cycle: String,         // 周期类型
    pub lead_stock: Option<String>,
}

/// 长期记忆 (Long-term Memory) - TTL: 30+天
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermMemory {
    /// 策略表现统计
    pub strategy_stats: HashMap<String, StrategyStat>,
    /// 每日报告
    pub daily_reports: VecDeque<DailyReport>,
    /// 月度总结
    pub monthly_summaries: VecDeque<MonthlySummary>,
    pub max_reports: usize,
}

/// 策略统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStat {
    pub strategy_name: String,
    pub total_trades: i32,
    pub win_trades: i32,
    pub lose_trades: i32,
    pub win_rate: f64,
    pub avg_profit: f64,
    pub avg_loss: f64,
    pub max_drawdown: f64,
    pub profit_factor: f64,
    pub avg_holding_days: f64,
    pub updated_at: String,
}

/// 每日报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReport {
    pub date: String,
    pub total_pnl: f64,
    pub pnl_ratio: f64,
    pub positions_json: String,
    pub trades_json: String,
    pub summary: String,
    pub ai_observations: Vec<String>,
    pub next_outlook: String,
}

/// 月度总结
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySummary {
    pub month: String,
    pub total_pnl: f64,
    pub total_trades: i32,
    pub win_rate: f64,
    pub best_trade: String,
    pub worst_trade: String,
    pub lessons_learned: Vec<String>,
}

/// 记忆系统
pub struct MemorySystem {
    /// 短期记忆
    working: WorkingMemory,
    /// 情景记忆
    episodic: EpisodicMemory,
    /// 语义记忆
    semantic: SemanticMemory,
    /// 长期记忆
    longterm: LongTermMemory,
    /// 学习历史
    learning_history: Vec<LearningRecord>,
}

/// 学习记录
#[derive(Debug, Clone)]
pub struct LearningRecord {
    pub timestamp: String,
    pub event: String,
    pub lesson: String,
    pub adjustment: String,
}

impl MemorySystem {
    pub fn new() -> Self {
        Self {
            working: WorkingMemory {
                positions: HashMap::new(),
                floating_pnl: 0.0,
                watch_list: vec![],
                updated_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            },
            episodic: EpisodicMemory {
                trades: VecDeque::new(),
                max_size: 1000,  // 保留1000条交易记录
            },
            semantic: SemanticMemory {
                stock_knowledge: HashMap::new(),
                sector_relations: HashMap::new(),
            },
            longterm: LongTermMemory {
                strategy_stats: HashMap::new(),
                daily_reports: VecDeque::new(),
                monthly_summaries: VecDeque::new(),
                max_reports: 90,  // 保留90天报告
            },
            learning_history: vec![],
        }
    }

    /// 更新市场状态 (短期记忆)
    pub async fn update_market_state(&mut self, market_data: &super::agent_perception::MarketData) {
        self.working.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // 更新热点板块
        for sector in &market_data.hot_sectors {
            if sector.hot_score > 70.0 {
                if !self.working.watch_list.contains(&sector.name) {
                    self.working.watch_list.push(sector.name.clone());
                }
            }
        }

        // 保持关注列表在合理大小
        if self.working.watch_list.len() > 50 {
            self.working.watch_list.truncate(50);
        }
    }

    /// 记录交易 (情景记忆)
    pub async fn record_trades(&mut self, trades: &[super::agent_reasoning::TradeRecord]) {
        for trade in trades {
            let episodic = EpisodicTrade {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: trade.timestamp.clone(),
                symbol: trade.symbol.clone(),
                name: trade.name.clone(),
                action: trade.action.clone(),
                price: trade.price,
                quantity: trade.quantity,
                reason: trade.reason.clone(),
                context: format!("AI Agent 自动交易 - {}", Local::now().format("%Y-%m-%d %H:%M")),
                result: trade.pnl,
            };

            self.episodic.trades.push_back(episodic);

            // 超过容量时移除最旧的
            if self.episodic.trades.len() > self.episodic.max_size {
                self.episodic.trades.pop_front();
            }
        }

        // 触发学习
        self.learn_from_trades(trades).await;
    }

    /// 更新策略表现 (长期记忆)
    pub async fn update_strategy_performance(&mut self, analysis: &super::agent_reasoning::DeepAnalysis) {
        let perf = &analysis.strategy_performance;

        // 更新统计
        let stat = self.longterm.strategy_stats
            .entry("ai_strategy".to_string())
            .or_insert_with(|| StrategyStat {
                strategy_name: "ai_strategy".to_string(),
                total_trades: 0,
                win_trades: 0,
                lose_trades: 0,
                win_rate: 0.0,
                avg_profit: 0.0,
                avg_loss: 0.0,
                max_drawdown: 0.0,
                profit_factor: 0.0,
                avg_holding_days: 0.0,
                updated_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            });

        stat.total_trades += perf.total_trades;
        stat.win_rate = perf.win_rate;
        stat.max_drawdown = perf.max_drawdown;
        stat.profit_factor = perf.profit_factor;
        stat.avg_holding_days = perf.avg_holding_days;
        stat.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }

    /// 设置次日展望
    pub async fn set_tomorrow_outlook(&mut self, outlook: &super::agent_reasoning::TomorrowOutlook) {
        // 将展望添加到学习历史
        self.learning_history.push(LearningRecord {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            event: "明日展望生成".to_string(),
            lesson: outlook.ai_view.clone(),
            adjustment: format!("关注板块: {:?}", outlook.focus_sectors),
        });
    }

    /// 从交易中学习
    async fn learn_from_trades(&mut self, trades: &[super::agent_reasoning::TradeRecord]) {
        // 分析最近交易
        let recent_trades: Vec<_> = trades.iter().take(10).collect();

        let mut wins = 0;
        let mut losses = 0;

        for t in &recent_trades {
            if let Some(pnl) = t.pnl {
                if pnl > 0.0 {
                    wins += 1;
                } else if pnl < 0.0 {
                    losses += 1;
                }
            }
        }

        let total = wins + losses;
        if total > 0 {
            let win_rate = wins as f64 / total as f64;

            // 根据胜率调整策略
            if win_rate < 0.4 {
                self.learning_history.push(LearningRecord {
                    timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    event: "胜率过低警告".to_string(),
                    lesson: format!("最近{}笔交易胜率仅{:.1}%", total, win_rate * 100.0),
                    adjustment: "建议降低仓位或暂停交易".to_string(),
                });
            }
        }
    }

    /// 获取短期记忆
    pub fn get_working_memory(&self) -> &WorkingMemory {
        &self.working
    }

    /// 获取情景记忆
    pub fn get_episodic_memory(&self) -> &EpisodicMemory {
        &self.episodic
    }

    /// 获取语义记忆
    pub fn get_semantic_memory(&self) -> &SemanticMemory {
        &self.semantic
    }

    /// 获取长期记忆
    pub fn get_longterm_memory(&self) -> &LongTermMemory {
        &self.longterm
    }

    /// 获取最近的学习记录
    pub fn get_recent_learning(&self, count: usize) -> Vec<&LearningRecord> {
        self.learning_history.iter().rev().take(count).collect()
    }

    /// 添加股票知识
    pub async fn add_stock_knowledge(&mut self, stock: StockKnowledge) {
        self.semantic.stock_knowledge.insert(stock.symbol.clone(), stock);
    }

    /// 添加板块信息
    pub async fn add_sector_info(&mut self, sector: SectorInfo) {
        self.semantic.sector_relations.insert(sector.name.clone(), sector);
    }
}

impl Default for MemorySystem {
    fn default() -> Self {
        Self::new()
    }
}
