use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use chrono::Utc;

/// 虚拟持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualPosition {
    pub symbol: String,
    pub name: String,
    pub quantity: f64,
    pub avg_cost: f64,
    pub current_price: f64,
    pub market_value: f64,
    pub profit_loss: f64,
    pub profit_ratio: f64,
    pub update_time: String,
}

/// 虚拟交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualTrade {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub direction: String,
    pub price: f64,
    pub quantity: f64,
    pub amount: f64,
    pub commission: f64,
    pub trade_time: String,
    pub reason: String,
}

/// 虚拟账户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualAccount {
    pub total_assets: f64,
    pub cash: f64,
    pub market_value: f64,
    pub total_profit: f64,
    pub profit_ratio: f64,
    pub total_trades: i32,
    pub update_time: String,
}

/// 虚拟交易服务
pub struct VirtualTrading {
    initial_capital: f64,
    cash: f64,
    positions: HashMap<String, VirtualPosition>,
    trades: Vec<VirtualTrade>,
}

impl VirtualTrading {
    pub fn new(initial_capital: f64) -> Self {
        Self {
            initial_capital,
            cash: initial_capital,
            positions: HashMap::new(),
            trades: Vec::new(),
        }
    }
}

static VIRTUAL_TRADING: Lazy<Mutex<VirtualTrading>> = Lazy::new(|| {
    Mutex::new(VirtualTrading::new(100000.0))
});

fn get_account_summary(vt: &VirtualTrading) -> VirtualAccount {
    let mut market_value = 0.0;
    for pos in vt.positions.values() {
        market_value += pos.current_price * pos.quantity;
    }
    
    let total_assets = vt.cash + market_value;
    let total_profit = total_assets - vt.initial_capital;
    let profit_ratio = if vt.initial_capital > 0.0 {
        total_profit / vt.initial_capital * 100.0
    } else {
        0.0
    };
    
    VirtualAccount {
        total_assets,
        cash: vt.cash,
        market_value,
        total_profit,
        profit_ratio,
        total_trades: vt.trades.len() as i32,
        update_time: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

/// 买入股票
pub fn buy_stock(symbol: &str, name: &str, price: f64, quantity: f64, reason: &str) -> Result<VirtualAccount, String> {
    let mut vt = VIRTUAL_TRADING.lock().map_err(|e| e.to_string())?;
    
    let amount = price * quantity;
    let commission = amount * 0.0003;
    
    if amount + commission > vt.cash {
        return Err("现金不足".to_string());
    }
    
    let trade = VirtualTrade {
        id: format!("{}", Utc::now().timestamp_millis()),
        symbol: symbol.to_string(),
        name: name.to_string(),
        direction: "买入".to_string(),
        price,
        quantity,
        amount,
        commission,
        trade_time: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        reason: reason.to_string(),
    };
    
    vt.cash -= amount + commission;
    
    if let Some(pos) = vt.positions.get_mut(symbol) {
        let total_cost = pos.avg_cost * pos.quantity + amount;
        pos.quantity += quantity;
        pos.avg_cost = total_cost / pos.quantity;
    } else {
        vt.positions.insert(symbol.to_string(), VirtualPosition {
            symbol: symbol.to_string(),
            name: name.to_string(),
            quantity,
            avg_cost: price,
            current_price: price,
            market_value: amount,
            profit_loss: 0.0,
            profit_ratio: 0.0,
            update_time: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });
    }
    
    vt.trades.push(trade);
    
    Ok(get_account_summary(&vt))
}

/// 卖出股票
pub fn sell_stock(symbol: &str, price: f64, quantity: f64, reason: &str) -> Result<VirtualAccount, String> {
    let mut vt = VIRTUAL_TRADING.lock().map_err(|e| e.to_string())?;
    
    // 先检查持仓
    let position = vt.positions.get(symbol).ok_or("没有持仓")?;
    if position.quantity < quantity {
        return Err("持仓不足".to_string());
    }
    
    let amount = price * quantity;
    let commission = amount * 0.0003;
    
    let trade = VirtualTrade {
        id: format!("{}", Utc::now().timestamp_millis()),
        symbol: symbol.to_string(),
        name: position.name.clone(),
        direction: "卖出".to_string(),
        price,
        quantity,
        amount,
        commission,
        trade_time: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        reason: reason.to_string(),
    };
    
    // 更新持仓
    if let Some(pos) = vt.positions.get_mut(symbol) {
        pos.quantity -= quantity;
        if pos.quantity <= 0.0 {
            vt.positions.remove(symbol);
        }
    }
    
    vt.cash += amount - commission;
    vt.trades.push(trade);
    
    Ok(get_account_summary(&vt))
}

/// 获取账户信息
pub fn get_account() -> Result<VirtualAccount, String> {
    let vt = VIRTUAL_TRADING.lock().map_err(|e| e.to_string())?;
    Ok(get_account_summary(&vt))
}

/// 获取所有持仓（自动更新价格）
pub fn get_positions() -> Result<Vec<VirtualPosition>, String> {
    let symbols = {
        let vt = VIRTUAL_TRADING.lock().map_err(|e| e.to_string())?;
        vt.positions.keys().cloned().collect::<Vec<String>>()
    };
    
    // 获取最新价格
    if !symbols.is_empty() {
        let prices = fetch_latest_prices(&symbols);
        let _ = update_positions_prices(prices);
    }
    
    let vt = VIRTUAL_TRADING.lock().map_err(|e| e.to_string())?;
    Ok(vt.positions.values().cloned().collect())
}

fn fetch_latest_prices(symbols: &[String]) -> HashMap<String, f64> {
    let mut prices = HashMap::new();
    // 简化：返回空价格，实际由外部更新
    for symbol in symbols {
        prices.insert(symbol.clone(), 0.0);
    }
    prices
}

/// 获取交易记录
pub fn get_trades(limit: i32) -> Result<Vec<VirtualTrade>, String> {
    let vt = VIRTUAL_TRADING.lock().map_err(|e| e.to_string())?;
    let trades: Vec<VirtualTrade> = vt.trades.iter().rev().take(limit as usize).cloned().collect();
    Ok(trades)
}

/// 更新持仓价格
pub fn update_positions_prices(prices: HashMap<String, f64>) -> Result<(), String> {
    let mut vt = VIRTUAL_TRADING.lock().map_err(|e| e.to_string())?;
    
    for (symbol, price) in prices {
        if let Some(pos) = vt.positions.get_mut(&symbol) {
            pos.current_price = price;
            pos.market_value = price * pos.quantity;
            pos.profit_loss = (price - pos.avg_cost) * pos.quantity;
            pos.profit_ratio = if pos.avg_cost > 0.0 {
                (price - pos.avg_cost) / pos.avg_cost * 100.0
            } else {
                0.0
            };
            pos.update_time = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        }
    }
    
    Ok(())
}

/// 重置账户
pub fn reset_account(initial_capital: f64) -> Result<VirtualAccount, String> {
    let mut vt = VIRTUAL_TRADING.lock().map_err(|e| e.to_string())?;
    *vt = VirtualTrading::new(initial_capital);
    Ok(get_account_summary(&vt))
}
