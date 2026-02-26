use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub code: String,
    pub language: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 回测参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestParams {
    pub strategy_id: String,
    pub symbol: String,
    pub start_date: String,
    pub end_date: String,
    pub initial_capital: f64,
    pub commission_rate: f64,
    pub slippage: f64,
}

/// 回测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub id: String,
    pub strategy_id: String,
    pub params: BacktestParams,
    pub kpis: BacktestKpis,
    pub trades: Vec<BacktestTrade>,
    pub equity_curve: Vec<EquityPoint>,
    pub created_at: DateTime<Utc>,
}

/// 回测绩效指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestKpis {
    pub total_return: f64,
    pub annual_return: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub win_rate: f64,
    pub profit_loss_ratio: f64,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
}

/// 回测交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestTrade {
    pub timestamp: String,
    pub symbol: String,
    pub direction: String,
    pub price: f64,
    pub quantity: f64,
    pub commission: f64,
    pub pnl: f64,
}

/// 净值曲线点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub timestamp: String,
    pub equity: f64,
    pub benchmark: f64,
}
