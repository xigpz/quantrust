//! 执行模块 (Execution Module)
//!
//! 负责交易执行、风控检查、订单管理

use serde::{Deserialize, Serialize};
use super::agent_core::AgentConfig;
use super::agent_reasoning::TradeSignal;

/// 订单信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub direction: OrderDirection,
    pub order_type: OrderType,
    pub price: f64,
    pub quantity: f64,
    pub filled_price: Option<f64>,
    pub status: OrderStatus,
    pub created_at: String,
    pub filled_at: Option<String>,
}

/// 订单方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderDirection {
    Buy,
    Sell,
}

/// 订单类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,      // 市价单
    Limit,      // 限价单
    Condition,   // 条件单
}

/// 订单状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Submitted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// 持仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub name: String,
    pub quantity: f64,
    pub avg_cost: f64,
    pub current_price: f64,
    pub market_value: f64,
    pub profit_loss: f64,
    pub profit_ratio: f64,
}

/// 账户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub total_assets: f64,
    pub cash: f64,
    pub market_value: f64,
    pub total_pnl: f64,
    pub pnl_ratio: f64,
    pub today_pnl: f64,
    pub frozen: f64,
}

/// 风控结果
#[derive(Debug, Clone)]
pub struct RiskCheckResult {
    pub approved: bool,
    pub reason: String,
    pub adjusted_quantity: Option<f64>,
    pub risk_level: RiskLevel,
}

/// 风险等级
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// 执行模块
pub struct ExecutionModule {
    config: AgentConfig,
    orders: Vec<Order>,
    positions: HashMap<String, Position>,
    account: Account,
}

use std::collections::HashMap;

impl ExecutionModule {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config: config.clone(),
            orders: vec![],
            positions: HashMap::new(),
            account: Account {
                total_assets: config.initial_capital,
                cash: config.initial_capital,
                market_value: 0.0,
                total_pnl: 0.0,
                pnl_ratio: 0.0,
                today_pnl: 0.0,
                frozen: 0.0,
            },
        }
    }

    /// 风控检查
    pub async fn risk_check(&self, signals: &[TradeSignal]) -> Vec<TradeSignal> {
        let mut approved = vec![];

        for signal in signals {
            // 1. 置信度检查
            if signal.confidence < 0.6 {
                continue;
            }

            // 2. 仓位检查
            let current_position = self.positions.get(&signal.symbol);
            let position_ratio = if let Some(pos) = current_position {
                pos.market_value / self.account.total_assets
            } else {
                0.0
            };

            if position_ratio >= self.config.max_position_ratio {
                continue;
            }

            // 3. 单笔金额检查
            let trade_amount = signal.price * signal.quantity;
            let trade_ratio = trade_amount / self.account.total_assets;
            if trade_ratio > self.config.max_trade_ratio {
                continue;
            }

            // 4. 日亏损检查
            if self.account.today_pnl < -self.config.daily_stop_loss_ratio * self.account.total_assets {
                // 已达日止损线，不允许开新仓
                continue;
            }

            approved.push(signal.clone());
        }

        approved
    }

    /// 执行交易信号
    pub async fn execute(&mut self, signal: &TradeSignal) -> Option<Order> {
        match signal.action {
            super::agent_reasoning::TradeAction::Buy => {
                self.execute_buy(signal).await
            }
            super::agent_reasoning::TradeAction::Sell => {
                self.execute_sell(signal).await
            }
            super::agent_reasoning::TradeAction::Hold => None,
        }
    }

    /// 执行买入
    async fn execute_buy(&mut self, signal: &TradeSignal) -> Option<Order> {
        // 计算买入数量
        let trade_ratio = self.config.max_trade_ratio.min(1.0 - (signal.confidence - 0.6) / 0.4);
        let available_cash = self.account.cash * 0.95; // 留5%手续费
        let max_amount = available_cash * trade_ratio;
        let quantity = (max_amount / signal.price / 100.0).floor() * 100.0; // 整百股

        if quantity < 100.0 {
            return None;
        }

        let order = Order {
            id: uuid::Uuid::new_v4().to_string(),
            symbol: signal.symbol.clone(),
            name: signal.name.clone(),
            direction: OrderDirection::Buy,
            order_type: OrderType::Market,
            price: signal.price,
            quantity,
            filled_price: Some(signal.price),
            status: OrderStatus::Filled,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            filled_at: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        };

        // 更新持仓
        let cost = order.filled_price.unwrap_or(order.price) * order.quantity;
        let commission = cost * 0.0003; // 佣金万三

        if let Some(pos) = self.positions.get_mut(&signal.symbol) {
            let total_quantity = pos.quantity + order.quantity;
            pos.avg_cost = (pos.avg_cost * pos.quantity + cost) / total_quantity;
            pos.quantity = total_quantity;
            pos.market_value = pos.quantity * signal.price;
            pos.profit_loss = pos.market_value - pos.avg_cost * pos.quantity;
            pos.profit_ratio = pos.profit_loss / (pos.avg_cost * pos.quantity);
        } else {
            self.positions.insert(signal.symbol.clone(), Position {
                symbol: signal.symbol.clone(),
                name: signal.name.clone(),
                quantity: order.quantity,
                avg_cost: signal.price,
                current_price: signal.price,
                market_value: signal.price * order.quantity,
                profit_loss: 0.0,
                profit_ratio: 0.0,
            });
        }

        // 更新账户
        self.account.cash -= (cost + commission);
        self.account.frozen += commission;

        self.orders.push(order.clone());
        Some(order)
    }

    /// 执行卖出
    async fn execute_sell(&mut self, signal: &TradeSignal) -> Option<Order> {
        let position = self.positions.get(&signal.symbol)?;

        let order = Order {
            id: uuid::Uuid::new_v4().to_string(),
            symbol: signal.symbol.clone(),
            name: signal.name.clone(),
            direction: OrderDirection::Sell,
            order_type: OrderType::Market,
            price: signal.price,
            quantity: position.quantity,
            filled_price: Some(signal.price),
            status: OrderStatus::Filled,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            filled_at: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        };

        // 计算收益
        let revenue = order.filled_price.unwrap_or(order.price) * order.quantity;
        let commission = revenue * 0.0003;
        let stamp_tax = revenue * 0.0005; // 印花税万五
        let net_revenue = revenue - commission - stamp_tax;

        let cost = position.avg_cost * position.quantity;
        let pnl = net_revenue - cost;

        // 更新账户
        self.account.cash += net_revenue;
        self.account.total_pnl += pnl;
        self.account.today_pnl += pnl;
        self.account.pnl_ratio = self.account.total_pnl / self.config.initial_capital;

        // 移除持仓
        self.positions.remove(&signal.symbol);

        self.orders.push(order.clone());
        Some(order)
    }

    /// 获取当日交易记录
    pub async fn get_today_trades(&self) -> Vec<super::agent_reasoning::TradeRecord> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        self.orders.iter()
            .filter(|o| o.created_at.starts_with(&today))
            .map(|o| {
                let pnl = if o.direction == OrderDirection::Sell {
                    let revenue = o.filled_price.unwrap_or(o.price) * o.quantity;
                    let cost = self.account.total_assets - self.account.cash; // 简化计算
                    Some(revenue - cost)
                } else {
                    None
                };

                super::agent_reasoning::TradeRecord {
                    symbol: o.symbol.clone(),
                    name: o.name.clone(),
                    action: match o.direction {
                        OrderDirection::Buy => "buy".to_string(),
                        OrderDirection::Sell => "sell".to_string(),
                    },
                    price: o.price,
                    quantity: o.quantity,
                    timestamp: o.created_at.clone(),
                    reason: format!("{:?}", o.order_type),
                    pnl,
                }
            })
            .collect()
    }

    /// 获取当前持仓
    pub fn get_positions(&self) -> &HashMap<String, Position> {
        &self.positions
    }

    /// 获取账户信息
    pub fn get_account(&self) -> &Account {
        &self.account
    }

    /// 检查是否需要止损
    pub async fn check_stop_loss(&self, symbol: &str, current_price: f64) -> Option<TradeSignal> {
        let position = self.positions.get(symbol)?;

        let loss_ratio = (current_price - position.avg_cost) / position.avg_cost;

        // 止损线 -5%
        if loss_ratio <= -0.05 {
            Some(TradeSignal {
                symbol: symbol.to_string(),
                name: position.name.clone(),
                action: super::agent_reasoning::TradeAction::Sell,
                price: current_price,
                quantity: position.quantity,
                confidence: 0.9,
                reason: format!("触发止损线，当前亏损{:.1}%", loss_ratio * 100.0),
                source: super::agent_reasoning::SignalSource::FastThink,
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        } else {
            None
        }
    }

    /// 更新持仓价格
    pub async fn update_prices(&mut self, prices: &HashMap<String, f64>) {
        for (symbol, price) in prices {
            if let Some(pos) = self.positions.get_mut(symbol) {
                pos.current_price = *price;
                pos.market_value = pos.quantity * price;
                pos.profit_loss = pos.market_value - pos.avg_cost * pos.quantity;
                pos.profit_ratio = if pos.avg_cost > 0.0 {
                    pos.profit_loss / (pos.avg_cost * pos.quantity)
                } else {
                    0.0
                };
            }
        }

        // 更新账户市值
        self.account.market_value = self.positions.values()
            .map(|p| p.market_value)
            .sum();
        self.account.total_assets = self.account.cash + self.account.market_value;
    }
}
