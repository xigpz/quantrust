//! 自主交易引擎 (Autonomous Trader)
//!
//! 实现 AI 自动选股、决策、交易的完整流程

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{Local, NaiveTime, Datelike};

use crate::services::agent_core::{AgentConfig, AgentState};
use crate::services::agent_perception::{PerceptionModule, MarketData, RealtimeData, NewsData};
use crate::services::agent_reasoning::{ReasoningEngine, TradeSignal, TradeAction, CandidateStock, BreakoutType};
use crate::services::agent_memory::MemorySystem;
use crate::services::agent_execution::ExecutionModule;

/// 选股策略
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum StrategyType {
    Breakout,        // 突破策略
    Momentum,        // 动量策略
    SectorRotation,  // 板块轮动
    NewsDriven,      // 消息驱动
    Sentiment,       // 情绪驱动
}

/// 策略配置
#[derive(Debug, Clone)]
pub struct StrategyConfig {
    pub strategy_type: StrategyType,
    pub enabled: bool,
    pub weight: f64,           // 策略权重
    pub min_confidence: f64,   // 最小置信度
    pub max_positions: usize,  // 最大持仓数
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            strategy_type: StrategyType::Breakout,
            enabled: true,
            weight: 1.0,
            min_confidence: 0.6,
            max_positions: 5,
        }
    }
}

/// 每日交易计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPlan {
    pub date: String,
    pub focus_sectors: Vec<String>,     // 重点关注板块
    pub watch_list: Vec<String>,         // 观察名单
    pub avoid_list: Vec<String>,         // 规避名单
    pub target_positions: Vec<TargetPosition>,
    pub risk_level: RiskLevel,
}

/// 目标持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetPosition {
    pub symbol: String,
    pub name: String,
    pub priority: i32,       // 优先级 1-5
    pub entry_reason: String,
    pub target_price: f64,
    pub stop_loss: f64,
}

/// 风险等级
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// 交易决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingDecision {
    pub id: String,
    pub timestamp: String,
    pub symbol: String,
    pub name: String,
    pub action: TradeAction,
    pub quantity: f64,
    pub price: f64,
    pub confidence: f64,
    pub reason: String,
    pub strategies: Vec<String>,  // 触发策略
    pub market_context: String,
    pub executed: bool,
}

/// 市场时段
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarketSession {
    PreMarket,    // 盘前 (9:00-9:15)
    Morning,      // 早盘 (9:30-11:30)
    Lunch,        // 午间 (11:30-13:00)
    Afternoon,    // 午盘 (13:00-15:00)
    AfterHours,   // 盘后 (15:00-)
    Closed,       // 休市
}

impl MarketSession {
    pub fn current() -> Self {
        let now = Local::now();
        let time = now.time();
        let weekday = now.naive_local().weekday();

        // 周六周日休市
        if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
            return MarketSession::Closed;
        }

        let pre_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let pre_end = NaiveTime::from_hms_opt(9, 15, 0).unwrap();
        let morning_start = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
        let morning_end = NaiveTime::from_hms_opt(11, 30, 0).unwrap();
        let lunch_start = NaiveTime::from_hms_opt(11, 30, 0).unwrap();
        let lunch_end = NaiveTime::from_hms_opt(13, 0, 0).unwrap();
        let afternoon_start = NaiveTime::from_hms_opt(13, 0, 0).unwrap();
        let afternoon_end = NaiveTime::from_hms_opt(15, 0, 0).unwrap();

        if time >= pre_start && time < pre_end {
            MarketSession::PreMarket
        } else if time >= morning_start && time < morning_end {
            MarketSession::Morning
        } else if time >= lunch_start && time < lunch_end {
            MarketSession::Lunch
        } else if time >= afternoon_start && time < afternoon_end {
            MarketSession::Afternoon
        } else if time >= afternoon_end {
            MarketSession::AfterHours
        } else {
            MarketSession::Closed
        }
    }

    pub fn is_trading(&self) -> bool {
        matches!(self, MarketSession::Morning | MarketSession::Afternoon)
    }

    pub fn allow_new_positions(&self) -> bool {
        matches!(self, MarketSession::Morning | MarketSession::PreMarket)
    }
}

/// 自主交易引擎
pub struct AutonomousTrader {
    config: AgentConfig,
    strategies: HashMap<StrategyType, StrategyConfig>,
    daily_plan: Option<DailyPlan>,
    decisions: Vec<TradingDecision>,
}

impl AutonomousTrader {
    pub fn new(config: AgentConfig) -> Self {
        let mut strategies = HashMap::new();
        strategies.insert(StrategyType::Breakout, StrategyConfig {
            strategy_type: StrategyType::Breakout,
            enabled: true,
            weight: 1.0,
            min_confidence: 0.65,
            max_positions: 3,
        });
        strategies.insert(StrategyType::Momentum, StrategyConfig {
            strategy_type: StrategyType::Momentum,
            enabled: true,
            weight: 0.8,
            min_confidence: 0.6,
            max_positions: 2,
        });
        strategies.insert(StrategyType::SectorRotation, StrategyConfig {
            strategy_type: StrategyType::SectorRotation,
            enabled: true,
            weight: 0.7,
            min_confidence: 0.6,
            max_positions: 2,
        });
        strategies.insert(StrategyType::NewsDriven, StrategyConfig {
            strategy_type: StrategyType::NewsDriven,
            enabled: true,
            weight: 0.9,
            min_confidence: 0.7,
            max_positions: 2,
        });

        Self {
            config,
            strategies,
            daily_plan: None,
            decisions: vec![],
        }
    }

    /// 获取当前市场时段
    pub fn get_market_session(&self) -> MarketSession {
        MarketSession::current()
    }

    /// 生成每日交易计划 (盘前)
    pub async fn generate_daily_plan(
        &mut self,
        perception: &PerceptionModule,
        reasoning: &ReasoningEngine,
    ) -> DailyPlan {
        let today = Local::now().format("%Y-%m-%d").to_string();

        // 感知市场数据
        let market = perception.scan_market().await;

        // 获取新闻
        let news = perception.scan_news().await;

        // 分析热点
        let hot_sectors: Vec<String> = market.hot_sectors
            .iter()
            .filter(|s| s.hot_score > 70.0)
            .map(|s| s.name.clone())
            .collect();

        // 生成观察名单
        let watch_list: Vec<String> = market.hot_sectors
            .iter()
            .filter(|s| s.hot_score > 60.0 && s.hot_score <= 70.0)
            .take(10)
            .map(|s| s.lead_stock.clone())
            .collect();

        // 生成目标持仓
        let mut target_positions = vec![];
        for sector in market.hot_sectors.iter().take(3) {
            if sector.hot_score > 75.0 {
                target_positions.push(TargetPosition {
                    symbol: sector.lead_stock.clone(),
                    name: sector.lead_stock.clone(),
                    priority: 1,
                    entry_reason: format!("热点板块 {} 龙头", sector.name),
                    target_price: 0.0, // 由执行时确定
                    stop_loss: 0.0,
                });
            }
        }

        // 判断风险等级
        let risk_level = if market.limit_up_count > 50 && market.limit_down_count < 10 {
            RiskLevel::Low
        } else if market.limit_up_count < 20 || market.limit_down_count > 30 {
            RiskLevel::High
        } else {
            RiskLevel::Medium
        };

        let plan = DailyPlan {
            date: today,
            focus_sectors: hot_sectors,
            watch_list,
            avoid_list: vec![],
            target_positions,
            risk_level,
        };

        self.daily_plan = Some(plan.clone());
        plan
    }

    /// 执行盘中扫描
    pub async fn intraday_scan(
        &self,
        perception: &PerceptionModule,
        reasoning: &ReasoningEngine,
    ) -> Vec<TradeSignal> {
        let session = self.get_market_session();
        if !session.is_trading() {
            return vec![];
        }

        // 扫描实时数据
        let realtime = perception.scan_realtime().await;

        // 快思考筛选
        let candidates = reasoning.fast_think(&realtime).await;

        // 慢思考验证
        let signals = reasoning.slow_think(&candidates).await;

        // 应用策略过滤
        let filtered = self.apply_strategy_filter(&signals);

        filtered
    }

    /// 应用策略过滤
    fn apply_strategy_filter(&self, signals: &[TradeSignal]) -> Vec<TradeSignal> {
        let mut result = vec![];

        for signal in signals {
            // 检查是否在规避名单
            if let Some(ref plan) = self.daily_plan {
                if plan.avoid_list.contains(&signal.symbol) {
                    continue;
                }
            }

            // 检查策略置信度
            let config = self.get_strategy_config_for_signal(signal);
            if let Some(config) = config {
                if signal.confidence >= config.min_confidence {
                    result.push(signal.clone());
                }
            } else {
                // 无策略匹配，默认通过
                result.push(signal.clone());
            }
        }

        result
    }

    /// 根据信号获取策略配置
    fn get_strategy_config_for_signal(&self, signal: &TradeSignal) -> Option<&StrategyConfig> {
        let strategy_type = match signal.source {
            crate::services::agent_reasoning::SignalSource::FastThink => StrategyType::Breakout,
            crate::services::agent_reasoning::SignalSource::SlowThink => StrategyType::Momentum,
            crate::services::agent_reasoning::SignalSource::DeepThink => StrategyType::Sentiment,
        };

        self.strategies.get(&strategy_type)
    }

    /// 生成交易决策
    pub async fn make_decision(
        &mut self,
        signal: &TradeSignal,
        execution: &ExecutionModule,
    ) -> Option<TradingDecision> {
        let session = self.get_market_session();

        // 检查是否允许开新仓
        if signal.action == TradeAction::Buy && !session.allow_new_positions() {
            return None;
        }

        // 风控检查
        let approved = execution.risk_check(&[signal.clone()]).await;
        if approved.is_empty() {
            return None;
        }

        let signal = &approved[0];

        // 检查最大持仓数
        if signal.action == TradeAction::Buy {
            let positions = execution.get_positions();
            if positions.len() >= 5 {
                return None;
            }
        }

        let decision = TradingDecision {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            symbol: signal.symbol.clone(),
            name: signal.name.clone(),
            action: signal.action.clone(),
            quantity: signal.quantity,
            price: signal.price,
            confidence: signal.confidence,
            reason: signal.reason.clone(),
            strategies: vec![],
            market_context: format!("{:?}", session),
            executed: false,
        };

        self.decisions.push(decision.clone());
        Some(decision)
    }

    /// 执行交易决策
    pub async fn execute_decision(
        &mut self,
        decision: &TradingDecision,
        execution: &mut ExecutionModule,
    ) -> Result<(), String> {
        if decision.executed {
            return Err("决策已执行".to_string());
        }

        let signal = TradeSignal {
            symbol: decision.symbol.clone(),
            name: decision.name.clone(),
            action: decision.action.clone(),
            price: decision.price,
            quantity: decision.quantity,
            confidence: decision.confidence,
            reason: decision.reason.clone(),
            source: crate::services::agent_reasoning::SignalSource::FastThink,
            timestamp: decision.timestamp.clone(),
        };

        execution.execute(&signal).await;

        // 标记为已执行
        if let Some(d) = self.decisions.iter_mut().find(|d| d.id == decision.id) {
            d.executed = true;
        }

        Ok(())
    }

    /// 盘后生成决策日志
    pub async fn generate_decision_log(&self) -> String {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let decisions: Vec<_> = self.decisions.iter()
            .filter(|d| d.timestamp.starts_with(&today))
            .collect();

        let mut log = format!("# {} 决策日志\n\n", today);
        log += &format!("总决策数: {}\n", decisions.len());
        log += &format!("执行数: {}\n\n", decisions.iter().filter(|d| d.executed).count());

        log += "## 买入决策\n";
        for d in decisions.iter().filter(|d| d.action == TradeAction::Buy) {
            log += &format!(
                "- [{}] 买入 {} @ {:.2} (置信度: {:.0}%, 理由: {})\n",
                d.timestamp, d.symbol, d.price, d.confidence * 100.0, d.reason
            );
        }

        log += "\n## 卖出决策\n";
        for d in decisions.iter().filter(|d| d.action == TradeAction::Sell) {
            log += &format!(
                "- [{}] 卖出 {} @ {:.2} (置信度: {:.0}%, 理由: {})\n",
                d.timestamp, d.symbol, d.price, d.confidence * 100.0, d.reason
            );
        }

        log
    }

    /// 获取策略配置
    pub fn get_strategy_configs(&self) -> HashMap<String, StrategyConfig> {
        self.strategies.iter()
            .map(|(k, v)| (format!("{:?}", k), v.clone()))
            .collect()
    }

    /// 更新策略配置
    pub fn update_strategy_config(&mut self, strategy_type: StrategyType, config: StrategyConfig) {
        self.strategies.insert(strategy_type, config);
    }

    /// 获取当日所有决策
    pub fn get_today_decisions(&self) -> Vec<&TradingDecision> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        self.decisions.iter()
            .filter(|d| d.timestamp.starts_with(&today))
            .collect()
    }
}
