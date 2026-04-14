use serde::{Deserialize, Serialize};

/// 模拟组合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub initial_capital: f64,
    pub current_capital: f64,
    pub total_value: f64,
    pub total_return_rate: f64,
    pub positions_value: f64,
    pub positions_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// 组合持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPosition {
    pub id: String,
    pub portfolio_id: String,
    pub symbol: String,
    pub name: String,
    pub quantity: f64,
    pub avg_cost: f64,
    pub current_price: f64,
    pub market_value: f64,
    pub total_profit: f64,
    pub profit_rate: f64,
    pub weight: f64,
    pub first_buy_date: Option<String>,
    pub last_trade_date: Option<String>,
    pub updated_at: String,
}

/// 调仓记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioTrade {
    pub id: String,
    pub portfolio_id: String,
    pub symbol: String,
    pub name: String,
    pub direction: TradeDirection,
    pub price: f64,
    pub quantity: f64,
    pub amount: f64,
    pub commission: f64,
    pub position_before: Option<f64>,
    pub position_after: Option<f64>,
    pub weight_before: Option<f64>,
    pub weight_after: Option<f64>,
    pub reason: Option<String>,
    pub trade_date: String,
    pub trade_time: String,
    pub created_at: String,
}

/// 交易方向
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeDirection {
    Buy,
    Sell,
}

impl TradeDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            TradeDirection::Buy => "buy",
            TradeDirection::Sell => "sell",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "buy" => Some(TradeDirection::Buy),
            "sell" => Some(TradeDirection::Sell),
            _ => None,
        }
    }
}

/// 操盘日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioLog {
    pub id: String,
    pub portfolio_id: String,
    pub log_type: LogType,
    pub content: String,
    pub created_at: String,
}

/// 日志类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogType {
    Trade,
    Memo,
    Adjustment,
}

impl LogType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogType::Trade => "trade",
            LogType::Memo => "memo",
            LogType::Adjustment => "adjustment",
        }
    }
}

/// 收益历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioReturn {
    pub id: i64,
    pub portfolio_id: String,
    pub date: String,
    pub total_value: f64,
    pub cash: f64,
    pub positions_value: f64,
    pub daily_return: Option<f64>,
    pub total_return_rate: f64,
    pub benchmark_return: f64,
}

/// 创建组合请求
#[derive(Debug, Deserialize)]
pub struct CreatePortfolioRequest {
    pub name: String,
    pub description: Option<String>,
    pub initial_capital: Option<f64>,
}

/// 买入股票请求
#[derive(Debug, Deserialize)]
pub struct BuyStockRequest {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub quantity: f64,
    pub reason: Option<String>,
    #[serde(default = "default_trade_date")]
    pub trade_date: String,
}

/// 卖出股票请求
#[derive(Debug, Deserialize)]
pub struct SellStockRequest {
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub reason: Option<String>,
    #[serde(default = "default_trade_date")]
    pub trade_date: String,
}

fn default_trade_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// 组合统计信息
#[derive(Debug, Serialize)]
pub struct PortfolioStats {
    pub total_return: f64,
    pub total_return_rate: f64,
    pub daily_return: f64,
    pub daily_return_rate: f64,
    pub total_trades: i64,
    pub win_trades: i64,
    pub loss_trades: i64,
    pub win_rate: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    // 新增增强指标
    pub annualized_return: f64,
    pub volatility: f64,
    pub benchmark_return: f64,
    pub alpha: f64,
    pub beta: f64,
    pub position_concentration: f64, // 持仓集中度 (最大仓位占比)
    pub win_streak: i32,            // 连胜次数
    pub lose_streak: i32,           // 连亏次数
}

/// 调仓记录查询参数
#[derive(Debug, Deserialize)]
pub struct TradeQueryParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub symbol: Option<String>,
}

/// 收益走势查询参数
#[derive(Debug, Deserialize)]
pub struct ReturnQueryParams {
    pub period: Option<String>, // 1d, 5d, 20d, 60d, 250d, all
}

/// 月度收益
#[derive(Debug, Clone, Serialize)]
pub struct MonthlyReturn {
    pub year: i32,
    pub month: i32,
    pub start_value: f64,
    pub end_value: f64,
    pub monthly_return: f64,
    pub monthly_return_rate: f64,
    pub benchmark_return: f64,
}

/// 年度收益
#[derive(Debug, Clone, Serialize)]
pub struct AnnualReturn {
    pub year: i32,
    pub start_value: f64,
    pub end_value: f64,
    pub annual_return: f64,
    pub annual_return_rate: f64,
    pub benchmark_return: f64,
    pub trades_count: i32,
    pub win_count: i32,
}
