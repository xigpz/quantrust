//! AI Agent 核心调度器
//!
//! 负责协调感知、推理、行动、记忆四个模块，形成智能体闭环

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::services::agent_perception::PerceptionModule;
use crate::services::agent_reasoning::ReasoningEngine;
use crate::services::agent_memory::MemorySystem;
use crate::services::agent_execution::ExecutionModule;

/// 智能体配置
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// 初始虚拟资金
    pub initial_capital: f64,
    /// 单股最大仓位比例
    pub max_position_ratio: f64,
    /// 日亏损止损比例
    pub daily_stop_loss_ratio: f64,
    /// 单笔交易最大比例
    pub max_trade_ratio: f64,
    /// 是否自动交易
    pub auto_trade: bool,
    /// 交易执行模式: paper | semi | full
    pub execution_mode: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            initial_capital: 100000.0,
            max_position_ratio: 0.2,       // 单股不超过20%
            daily_stop_loss_ratio: 0.05,   // 日亏损5%止损
            max_trade_ratio: 0.1,          // 单笔不超过10%
            auto_trade: false,              // 默认不自动交易
            execution_mode: "paper".to_string(),
        }
    }
}

/// 智能体状态
#[derive(Debug, Clone)]
pub enum AgentState {
    Idle,
    Scanning,
    Reasoning,
    Trading,
    Monitoring,
    Reviewing,
}

impl Default for AgentState {
    fn default() -> Self {
        Self::Idle
    }
}

/// AI Agent 核心
pub struct AgentCore {
    pub config: AgentConfig,
    pub state: AgentState,
    pub perception: Arc<RwLock<PerceptionModule>>,
    pub reasoning: Arc<RwLock<ReasoningEngine>>,
    pub memory: Arc<RwLock<MemorySystem>>,
    pub execution: Arc<RwLock<ExecutionModule>>,
    scheduler: Option<JobScheduler>,
}

impl AgentCore {
    /// 创建新的 Agent Core
    pub async fn new(config: AgentConfig) -> Self {
        let perception = Arc::new(RwLock::new(PerceptionModule::new()));
        let reasoning = Arc::new(RwLock::new(ReasoningEngine::new()));
        let memory = Arc::new(RwLock::new(MemorySystem::new()));
        let execution = Arc::new(RwLock::new(ExecutionModule::new(config.clone())));

        Self {
            config,
            state: AgentState::Idle,
            perception,
            reasoning,
            memory,
            execution,
            scheduler: None,
        }
    }

    /// 启动调度器
    pub async fn start_scheduler(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 调度器实现待完善
        // let scheduler = JobScheduler::new().await?;
        // ... job scheduling code ...
        // scheduler.start().await?;
        // self.scheduler = Some(scheduler);
        tracing::info!("Agent scheduler started (placeholder)");
        Ok(())
    }

    async fn add_job<F, Fut>(
        &self,
        _scheduler: JobScheduler,
        _cron: &str,
        _handler: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        Ok(())
    }

    /// 执行盘前扫描流程
    pub async fn pre_market_scan(&mut self) {
        self.state = AgentState::Scanning;
        println!("[Agent] 开始盘前扫描...");

        // 1. 感知市场数据
        let market_data = self.perception.read().await.scan_market().await;

        // 2. 感知新闻
        let news = self.perception.read().await.scan_news().await;

        // 3. 推理分析
        let signals = self.reasoning.read().await.analyze(&market_data, &news).await;

        // 4. 更新记忆
        self.memory.write().await.update_market_state(&market_data).await;

        self.state = AgentState::Idle;
        println!("[Agent] 盘前扫描完成，发现 {} 个信号", signals.len());
    }

    /// 执行盘中交易流程
    pub async fn intraday_trade(&mut self) {
        self.state = AgentState::Trading;
        println!("[Agent] 开始盘中交易...");

        // 1. 感知实时数据
        let realtime = self.perception.read().await.scan_realtime().await;

        // 2. 快思考筛选
        let candidates = self.reasoning.read().await.fast_think(&realtime).await;

        // 3. 慢思考验证
        let validated = self.reasoning.read().await.slow_think(&candidates).await;

        // 4. 风险检查
        let approved = self.execution.read().await.risk_check(&validated).await;

        // 5. 执行交易
        for signal in &approved {
            self.execution.write().await.execute(signal).await;
        }

        self.state = AgentState::Idle;
    }

    /// 执行盘后复盘
    pub async fn post_market_review(&mut self) {
        self.state = AgentState::Reviewing;
        println!("[Agent] 开始盘后复盘...");

        // 1. 获取当日交易记录
        let trades = self.execution.read().await.get_today_trades().await;

        // 2. 深思考分析
        let trade_history = crate::services::agent_reasoning::TradeHistory { trades: trades.clone() };
        let analysis = self.reasoning.read().await.deep_think(&trade_history).await;

        // 3. 更新记忆
        self.memory.write().await.record_trades(&trades).await;
        self.memory.write().await.update_strategy_performance(&analysis).await;

        // 4. 生成次日展望
        let outlook = self.reasoning.read().await.generate_outlook().await;

        self.memory.write().await.set_tomorrow_outlook(&outlook).await;

        self.state = AgentState::Idle;
        println!("[Agent] 盘后复盘完成");
    }
}

/// 创建并启动 Agent
pub async fn create_agent(config: AgentConfig) -> AgentCore {
    let mut agent = AgentCore::new(config).await;
    agent.start_scheduler().await.ok();
    agent
}
