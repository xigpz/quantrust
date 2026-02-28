use crate::api::routes::AppState;
use axum::{
    extract::{State, Path},
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

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
            account: Mutex::new(SimAccount { cash: 1_000_000.0, total_value: 1_000_000.0, positions_value: 0.0, positions_count: 0, today_pnl: 0.0, total_pnl: 0.0, updated_at: Utc::now().to_rfc3339() }),
            positions: Mutex::new(Vec::new()),
            orders: Mutex::new(Vec::new()),
            trades: Mutex::new(Vec::new()),
            next_order_id: Mutex::new(1),
            next_trade_id: Mutex::new(1),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimAccount { pub cash: f64, pub total_value: f64, pub positions_value: f64, pub positions_count: i32, pub today_pnl: f64, pub total_pnl: f64, pub updated_at: String }
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position { pub symbol: String, pub name: String, pub quantity: f64, pub avg_cost: f64, pub current_price: f64, pub market_value: f64, pub pnl: f64, pub pnl_rate: f64 }
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order { pub id: String, pub symbol: String, pub name: String, pub direction: String, pub price: f64, pub quantity: f64, pub filled: f64, pub status: String, pub created_at: String, pub updated_at: String }
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade { pub id: String, pub order_id: String, pub symbol: String, pub direction: String, pub price: f64, pub quantity: f64, pub amount: f64, pub traded_at: String }

#[derive(Deserialize)]
pub struct CreateOrderReq { pub symbol: String, pub name: String, pub direction: String, pub price: f64, pub quantity: f64 }

pub fn create_sim_router(state: AppState) -> Router {
    Router::new()
        .route("/api/sim/account", get(get_account))
        .route("/api/sim/reset", post(reset_account))
        .route("/api/sim/positions", get(get_positions))
        .route("/api/sim/orders", get(get_orders))
        .route("/api/sim/orders", post(create_order))
        .route("/api/sim/orders/{id}/cancel", post(cancel_order))
        .route("/api/sim/trades", get(get_trades))
        .with_state(state)
}

async fn get_account(State(state): State<AppState>) -> Json<serde_json::Value> {
    let quotes = state.cache.all_quotes.read().await;
    let mut account = state.sim_state.account.lock().unwrap();
    let positions = state.sim_state.positions.lock().unwrap();
    let mut positions_value = 0.0;
    let mut positions_count = 0;
    for pos in positions.iter() { if pos.quantity > 0.0 { positions_count += 1; let cp = quotes.iter().find(|q|q.symbol==pos.symbol).map(|q|q.price).unwrap_or(pos.avg_cost); positions_value += pos.quantity * cp; } }
    account.positions_value = positions_value;
    account.total_value = account.cash + positions_value;
    account.positions_count = positions_count;
    account.updated_at = Utc::now().to_rfc3339();
    Json(serde_json::json!({ "success": true, "data": account.clone() }))
}

async fn reset_account(State(state): State<AppState>) -> Json<serde_json::Value> {
    *state.sim_state.account.lock().unwrap() = SimAccount { cash: 1_000_000.0, total_value: 1_000_000.0, positions_value: 0.0, positions_count: 0, today_pnl: 0.0, total_pnl: 0.0, updated_at: Utc::now().to_rfc3339() };
    *state.sim_state.positions.lock().unwrap() = Vec::new();
    *state.sim_state.orders.lock().unwrap() = Vec::new();
    *state.sim_state.trades.lock().unwrap() = Vec::new();
    *state.sim_state.next_order_id.lock().unwrap() = 1;
    *state.sim_state.next_trade_id.lock().unwrap() = 1;
    Json(serde_json::json!({ "success": true, "message": "账户已重置" }))
}

async fn get_positions(State(state): State<AppState>) -> Json<serde_json::Value> {
    let quotes = state.cache.all_quotes.read().await;
    let mut positions = state.sim_state.positions.lock().unwrap();
    for pos in positions.iter_mut() { if let Some(q) = quotes.iter().find(|q|q.symbol==pos.symbol) { pos.current_price = q.price; pos.market_value = pos.quantity * q.price; pos.pnl = (q.price - pos.avg_cost) * pos.quantity; pos.pnl_rate = if pos.avg_cost > 0.0 { (q.price - pos.avg_cost) / pos.avg_cost * 100.0 } else { 0.0 }; } }
    Json(serde_json::json!({ "success": true, "data": positions.clone() }))
}

async fn get_orders(State(state): State<AppState>) -> Json<serde_json::Value> { let o = state.sim_state.orders.lock().unwrap(); Json(serde_json::json!({ "success": true, "data": o.clone() })) }
async fn get_trades(State(state): State<AppState>) -> Json<serde_json::Value> { let t = state.sim_state.trades.lock().unwrap(); Json(serde_json::json!({ "success": true, "data": t.clone() })) }

async fn create_order(State(state): State<AppState>, Json(req): Json<CreateOrderReq>) -> Json<serde_json::Value> {
    let oid = format!("ORD{:06}", { let mut c = state.sim_state.next_order_id.lock().unwrap(); let r=*c; *c+=1; r });
    let now = Utc::now().to_rfc3339();
    let acc = state.sim_state.account.lock().unwrap();
    if req.direction=="buy" && req.price*req.quantity>acc.cash { 
        return Json(serde_json::json!({"success":false,"message":"资金不足"})); 
    }
    if req.direction=="sell" { 
        let ps = state.sim_state.positions.lock().unwrap(); 
        let h:f64 = ps.iter().filter(|p|p.symbol==req.symbol).map(|p|p.quantity).sum(); 
        if h<req.quantity { 
            return Json(serde_json::json!({"success":false,"message":"持仓不足"})); 
        } 
    }
    drop(acc);
    let order = Order { 
        id: oid.clone(), 
        symbol: req.symbol.clone(), 
        name: req.name.clone(), 
        direction: req.direction.clone(), 
        price: req.price, 
        quantity: req.quantity, 
        filled: req.quantity, 
        status: "filled".to_string(), 
        created_at: now.clone(), 
        updated_at: now.clone() 
    };
    state.sim_state.orders.lock().unwrap().push(order.clone());
    let tid = format!("TRD{:06}", { let mut c = state.sim_state.next_trade_id.lock().unwrap(); let r=*c; *c+=1; r });
    state.sim_state.trades.lock().unwrap().push(Trade { 
        id: tid, 
        order_id: oid, 
        symbol: req.symbol.clone(), 
        direction: req.direction.clone(), 
        price: req.price, 
        quantity: req.quantity, 
        amount: req.price*req.quantity, 
        traded_at: now.clone() 
    });
    { 
        let mut a = state.sim_state.account.lock().unwrap(); 
        if req.direction=="buy" { a.cash -= req.price*req.quantity; } 
        else { a.cash += req.price*req.quantity; } 
    }
    { 
        let mut ps = state.sim_state.positions.lock().unwrap(); 
        if req.direction=="buy" { 
            if let Some(p)=ps.iter_mut().find(|p|p.symbol==req.symbol) { 
                let tc = p.avg_cost*p.quantity + req.price*req.quantity; 
                p.quantity+=req.quantity; 
                p.avg_cost=tc/p.quantity; 
            } else { 
                ps.push(Position { 
                    symbol: req.symbol.clone(), 
                    name: req.name.clone(), 
                    quantity: req.quantity, 
                    avg_cost: req.price, 
                    current_price: req.price, 
                    market_value: req.price*req.quantity, 
                    pnl: 0.0, 
                    pnl_rate: 0.0 
                }); 
            } 
        } else { 
            if let Some(p)=ps.iter_mut().find(|p|p.symbol==req.symbol) { 
                p.quantity-=req.quantity; 
                if p.quantity<=0.0 { 
                    ps.retain(|p|p.symbol!=req.symbol); 
                } 
            } 
        } 
    }
    Json(serde_json::json!({ "success": true, "data": order, "message": "成交" }))
}

async fn cancel_order(State(state): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let mut os = state.sim_state.orders.lock().unwrap();
    if let Some(o) = os.iter_mut().find(|o|o.id==id) { if o.status=="pending" { o.status="cancelled".to_string(); o.updated_at=Utc::now().to_rfc3339(); return Json(serde_json::json!({"success":true,"message":"已取消"})); } }
    Json(serde_json::json!({"success":false,"message":"无法取消"}))
}
