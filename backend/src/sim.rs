use crate::api::routes::AppState;
use crate::sim::SimState;
use axum::{
    extract::{State, Path},
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// 创建模拟交易路由
pub fn create_sim_router(state: AppState) -> Router {
    Router::new()
        // 账户
        .route("/api/sim/account", get(get_account))
        .route("/api/sim/reset", post(reset_account))
        // 持仓
        .route("/api/sim/positions", get(get_positions))
        // 订单
        .route("/api/sim/orders", get(get_orders))
        .route("/api/sim/orders", post(create_order))
        .route("/api/sim/orders/{id}/cancel", post(cancel_order))
        // 成交记录
        .route("/api/sim/trades", get(get_trades))
        .with_state(state)
}

// ============ 数据模型 ============

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimAccount {
    pub cash: f64,
    pub total_value: f64,
    pub positions_value: f64,
    pub positions_count: i32,
    pub today_pnl: f64,
    pub total_pnl: f64,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position {
    pub symbol: String,
    pub name: String,
    pub quantity: f64,
    pub avg_cost: f64,
    pub current_price: f64,
    pub market_value: f64,
    pub pnl: f64,
    pub pnl_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub direction: String,  // "buy" or "sell"
    pub price: f64,
    pub quantity: f64,
    pub filled: f64,
    pub status: String,     // "pending", "filled", "cancelled"
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub id: String,
    pub order_id: String,
    pub symbol: String,
    pub direction: String,
    pub price: f64,
    pub quantity: f64,
    pub amount: f64,
    pub traded_at: String,
}

#[derive(Deserialize)]
pub struct CreateOrderReq {
    pub symbol: String,
    pub name: String,
    pub direction: String,
    pub price: f64,
    pub quantity: f64,
}

// 内存中的模拟交易状态
pub struct SimState {
    pub account: Mutex<SimAccount>,
    pub positions: Mutex<Vec<Position>>,
    pub orders: Mutex<Vec<Order>>,
    pub trades: Mutex<Vec<Trade>>,
    pub next_order_id: Mutex<u64>,
    pub next_trade_id: Mutex<u64>,
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            account: Mutex::new(SimAccount {
                cash: 1_000_000.0,  // 初始资金 100 万
                total_value: 1_000_000.0,
                positions_value: 0.0,
                positions_count: 0,
                today_pnl: 0.0,
                total_pnl: 0.0,
                updated_at: Utc::now().to_rfc3339(),
            }),
            positions: Mutex::new(Vec::new()),
            orders: Mutex::new(Vec::new()),
            trades: Mutex::new(Vec::new()),
            next_order_id: Mutex::new(1),
            next_trade_id: Mutex::new(1),
        }
    }
}

// ============ 路由处理器 ============

/// 获取账户信息
async fn get_account(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    // 获取实时行情来计算持仓市值
    let quotes = state.cache.all_quotes.read().await;
    
    let mut account = state.sim_state.account.lock().unwrap();
    let positions = state.sim_state.positions.lock().unwrap();
    
    let mut positions_value = 0.0;
    let mut positions_count = 0;
    
    for pos in positions.iter() {
        if pos.quantity > 0.0 {
            positions_count += 1;
            // 查找当前价格
            let current_price = quotes.iter()
                .find(|q| q.symbol == pos.symbol)
                .map(|q| q.price)
                .unwrap_or(pos.avg_cost);
            
            positions_value += pos.quantity * current_price;
        }
    }
    
    account.positions_value = positions_value;
    account.total_value = account.cash + positions_value;
    account.positions_count = positions_count;
    account.updated_at = Utc::now().to_rfc3339();
    
    Json(serde_json::json!({
        "success": true,
        "data": account.clone(),
        "message": "ok"
    }))
}

/// 重置账户
async fn reset_account(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    *state.sim_state.account.lock().unwrap() = SimAccount {
        cash: 1_000_000.0,
        total_value: 1_000_000.0,
        positions_value: 0.0,
        positions_count: 0,
        today_pnl: 0.0,
        total_pnl: 0.0,
        updated_at: Utc::now().to_rfc3339(),
    };
    *state.sim_state.positions.lock().unwrap() = Vec::new();
    *state.sim_state.orders.lock().unwrap() = Vec::new();
    *state.sim_state.trades.lock().unwrap() = Vec::new();
    *state.sim_state.next_order_id.lock().unwrap() = 1;
    *state.sim_state.next_trade_id.lock().unwrap() = 1;
    
    Json(serde_json::json!({
        "success": true,
        "message": "账户已重置"
    }))
}

/// 获取持仓
async fn get_positions(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let quotes = state.cache.all_quotes.read().await;
    let mut positions = state.sim_state.positions.lock().unwrap();
    
    // 更新当前价格
    for pos in positions.iter_mut() {
        if let Some(quote) = quotes.iter().find(|q| q.symbol == pos.symbol) {
            pos.current_price = quote.price;
            pos.market_value = pos.quantity * pos.current_price;
            pos.pnl = (pos.current_price - pos.avg_cost) * pos.quantity;
            pos.pnl_rate = if pos.avg_cost > 0.0 {
                (pos.current_price - pos.avg_cost) / pos.avg_cost * 100.0
            } else {
                0.0
            };
        }
    }
    
    Json(serde_json::json!({
        "success": true,
        "data": positions.clone(),
        "message": "ok"
    }))
}

/// 获取订单列表
async fn get_orders(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let orders = state.sim_state.orders.lock().unwrap();
    
    Json(serde_json::json!({
        "success": true,
        "data": orders.clone(),
        "message": "ok"
    }))
}

/// 创建订单
async fn create_order(
    State(state): State<AppState>,
    Json(req): Json<CreateOrderReq>,
) -> Json<serde_json::Value> {
    let order_id = {
        let mut counter = state.sim_state.next_order_id.lock().unwrap();
        let id = *counter;
        *counter += 1;
        format!("ORD{:06}", id)
    };
    
    let now = Utc::now().to_rfc3339();
    let order = Order {
        id: order_id.clone(),
        symbol: req.symbol.clone(),
        name: req.name.clone(),
        direction: req.direction.clone(),
        price: req.price,
        quantity: req.quantity,
        filled: 0.0,
        status: "pending".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    
    // 检查资金/持仓是否足够
    let account = state.sim_state.account.lock().unwrap();
    let required_cash = req.price * req.quantity;
    
    if req.direction == "buy" && required_cash > account.cash {
        return Json(serde_json::json!({
            "success": false,
            "message": "资金不足"
        }));
    }
    
    if req.direction == "sell" {
        let positions = state.sim_state.positions.lock().unwrap();
        let holdings: f64 = positions.iter()
            .filter(|p| p.symbol == req.symbol)
            .map(|p| p.quantity)
            .sum();
        
        if holdings < req.quantity {
            return Json(serde_json::json!({
                "success": false,
                "message": "持仓不足"
            }));
        }
    }
    drop(account);
    
    // 添加订单
    state.sim_state.orders.lock().unwrap().push(order.clone());
    
    // 立即撮合（简化版：市价成交）
    let quotes = state.cache.all_quotes.read().await;
    let market_price = quotes.iter()
        .find(|q| q.symbol == req.symbol)
        .map(|q| q.price)
        .unwrap_or(req.price);
    
    let fill_price = market_price; // 简化为市价
    let fill_qty = req.quantity;
    let fill_amount = fill_price * fill_qty;
    
    // 更新订单状态
    {
        let mut orders = state.sim_state.orders.lock().unwrap();
        if let Some(o) = orders.iter_mut().find(|o| o.id == order_id) {
            o.status = "filled".to_string();
            o.filled = fill_qty;
            o.updated_at = Utc::now().to_rfc3339();
        }
    }
    
    // 创建成交记录
    let trade_id = {
        let mut counter = state.sim_state.next_trade_id.lock().unwrap();
        let id = *counter;
        *counter += 1;
        format!("TRD{:06}", id)
    };
    
    let trade = Trade {
        id: trade_id,
        order_id: order_id.clone(),
        symbol: req.symbol.clone(),
        direction: req.direction.clone(),
        price: fill_price,
        quantity: fill_qty,
        amount: fill_amount,
        traded_at: Utc::now().to_rfc3339(),
    };
    
    state.sim_state.trades.lock().unwrap().push(trade);
    
    // 更新账户和持仓
    {
        let mut account = state.sim_state.account.lock().unwrap();
        if req.direction == "buy" {
            account.cash -= fill_amount;
        } else {
            account.cash += fill_amount;
        }
    }
    
    {
        let mut positions = state.sim_state.positions.lock().unwrap();
        
        if req.direction == "buy" {
            // 新增或增持持仓
            if let Some(pos) = positions.iter_mut().find(|p| p.symbol == req.symbol) {
                let total_cost = pos.avg_cost * pos.quantity + fill_amount;
                pos.quantity += fill_qty;
                pos.avg_cost = total_cost / pos.quantity;
            } else {
                positions.push(Position {
                    symbol: req.symbol.clone(),
                    name: req.name.clone(),
                    quantity: fill_qty,
                    avg_cost: fill_price,
                    current_price: fill_price,
                    market_value: fill_qty * fill_price,
                    pnl: 0.0,
                    pnl_rate: 0.0,
                });
            }
        } else {
            // 减持持仓
            if let Some(pos) = positions.iter_mut().find(|p| p.symbol == req.symbol) {
                pos.quantity -= fill_qty;
                if pos.quantity <= 0.0 {
                    positions.retain(|p| p.symbol != req.symbol);
                }
            }
        }
    }
    
    Json(serde_json::json!({
        "success": true,
        "data": order,
        "message": "订单已成交"
    }))
}

/// 取消订单
async fn cancel_order(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let mut orders = state.sim_state.orders.lock().unwrap();
    
    if let Some(order) = orders.iter_mut().find(|o| o.id == id) {
        if order.status == "pending" {
            order.status = "cancelled".to_string();
            order.updated_at = Utc::now().to_rfc3339();
            
            return Json(serde_json::json!({
                "success": true,
                "message": "订单已取消"
            }));
        }
    }
    
    Json(serde_json::json!({
        "success": false,
        "message": "无法取消订单"
    }))
}

/// 获取成交记录
async fn get_trades(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let trades = state.sim_state.trades.lock().unwrap();
    
    Json(serde_json::json!({
        "success": true,
        "data": trades.clone(),
        "message": "ok"
    }))
}
