use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::Result;

/// 券商类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Broker {
    Futu,      // 富途
    Tiger,     // 老虎
    Custom,    // 自定义
}

/// 订单类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderType {
    Market,    // 市价单
    Limit,     // 限价单
}

/// 交易方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Direction {
    Buy,
    Sell,
}

/// 订单状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,   // 待成交
    Partial,   // 部分成交
    Filled,    // 全部成交
    Cancelled, // 已取消
    Rejected,  // 已拒绝
}

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub symbol: String,
    pub name: String,
    pub direction: Direction,
    pub order_type: OrderType,
    pub price: f64,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub status: OrderStatus,
    pub create_time: String,
    pub update_time: String,
}

/// 持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub name: String,
    pub quantity: f64,
    pub avg_cost: f64,
    pub current_price: f64,
    pub market_value: f64,
    pub profit_loss: f64,
    pub profit_loss_pct: f64,
}

/// 账户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub broker: Broker,
    pub cash: f64,
    pub market_value: f64,
    pub total_assets: f64,
    pub positions: Vec<Position>,
}

/// 富途配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FutuConfig {
    pub host: String,
    pub app_id: String,
    pub license: String,
}

/// 富途服务
pub struct FutuService {
    config: FutuConfig,
    client: Client,
}

impl FutuService {
    pub fn new(config: FutuConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// 买入
    pub async fn buy(&self, symbol: &str, price: f64, quantity: f64) -> Result<Order> {
        // TODO: 实现真实API调用
        Ok(Order {
            order_id: format!("ORDER_{}", chrono::Utc::now().timestamp_millis()),
            symbol: symbol.to_string(),
            name: "".to_string(),
            direction: Direction::Buy,
            order_type: OrderType::Limit,
            price,
            quantity,
            filled_quantity: 0.0,
            status: OrderStatus::Pending,
            create_time: chrono::Utc::now().to_rfc3339(),
            update_time: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// 卖出
    pub async fn sell(&self, symbol: &str, price: f64, quantity: f64) -> Result<Order> {
        Ok(Order {
            order_id: format!("ORDER_{}", chrono::Utc::now().timestamp_millis()),
            symbol: symbol.to_string(),
            name: "".to_string(),
            direction: Direction::Sell,
            order_type: OrderType::Limit,
            price,
            quantity,
            filled_quantity: 0.0,
            status: OrderStatus::Pending,
            create_time: chrono::Utc::now().to_rfc3339(),
            update_time: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// 撤单
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        // TODO: 实现真实API
        Ok(())
    }

    /// 查询订单
    pub async fn query_order(&self, order_id: &str) -> Result<Order> {
        Ok(Order {
            order_id: order_id.to_string(),
            symbol: "".to_string(),
            name: "".to_string(),
            direction: Direction::Buy,
            order_type: OrderType::Limit,
            price: 0.0,
            quantity: 0.0,
            filled_quantity: 0.0,
            status: OrderStatus::Filled,
            create_time: "".to_string(),
            update_time: "".to_string(),
        })
    }

    /// 查询持仓
    pub async fn query_positions(&self) -> Result<Vec<Position>> {
        Ok(vec![])
    }

    /// 查询账户
    pub async fn query_account(&self) -> Result<Account> {
        Ok(Account {
            broker: Broker::Futu,
            cash: 100000.0,
            market_value: 0.0,
            total_assets: 100000.0,
            positions: vec![],
        })
    }
}

/// 交易引擎
pub struct TradingEngine {
    broker: Option<FutuService>,
    stop_loss_ratio: f64,
    take_profit_ratio: f64,
    max_position_ratio: f64,
}

impl TradingEngine {
    pub fn new() -> Self {
        Self {
            broker: None,
            stop_loss_ratio: 0.05,
            take_profit_ratio: 0.15,
            max_position_ratio: 0.8,
        }
    }

    /// 配置券商
    pub fn with_broker(mut self, config: FutuConfig) -> Self {
        self.broker = Some(FutuService::new(config));
        self
    }

    /// 配置风控参数
    pub fn with_risk(mut self, stop_loss: f64, take_profit: f64, max_position: f64) -> Self {
        self.stop_loss_ratio = stop_loss;
        self.take_profit_ratio = take_profit;
        self.max_position_ratio = max_position;
        self
    }

    /// 执行买入
    pub async fn execute_buy(&self, symbol: &str, price: f64, quantity: f64) -> Result<Order> {
        if let Some(broker) = &self.broker {
            broker.buy(symbol, price, quantity).await
        } else {
            anyhow::bail!("券商未配置")
        }
    }

    /// 执行卖出
    pub async fn execute_sell(&self, symbol: &str, price: f64, quantity: f64) -> Result<Order> {
        if let Some(broker) = &self.broker {
            broker.sell(symbol, price, quantity).await
        } else {
            anyhow::bail!("券商未配置")
        }
    }

    /// 查询账户
    pub async fn get_account(&self) -> Result<Account> {
        if let Some(broker) = &self.broker {
            broker.query_account().await
        } else {
            anyhow::bail!("券商未配置")
        }
    }

    /// 检查止损
    pub async fn check_stop_loss(&self, symbol: &str) -> Result<Option<Order>> {
        if let Some(broker) = &self.broker {
            let positions = broker.query_positions().await?;
            
            for pos in positions {
                if pos.symbol == symbol {
                    let loss_pct = -pos.profit_loss_pct / 100.0;
                    
                    if loss_pct >= self.stop_loss_ratio {
                        // 触发止损，卖出全部
                        return Ok(Some(broker.sell(symbol, pos.current_price, pos.quantity).await?));
                    }
                }
            }
        }
        
        Ok(None)
    }
}

impl Default for TradingEngine {
    fn default() -> Self {
        Self::new()
    }
}
