use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use serde_json::json;

use crate::api::routes::AppState;
use crate::models::portfolio::*;
use crate::services::PortfolioService;

/// 创建组合路由
pub fn create_portfolio_router() -> Router<AppState> {
    Router::new()
        // 组合管理
        .route("/api/portfolios", get(list_portfolios).post(create_portfolio))
        .route("/api/portfolios/{id}", get(get_portfolio).delete(delete_portfolio))
        // 调仓操作
        .route("/api/portfolios/{id}/buy", post(buy_stock))
        .route("/api/portfolios/{id}/sell", post(sell_stock))
        // 查询
        .route("/api/portfolios/{id}/positions", get(get_positions))
        .route("/api/portfolios/{id}/trades", get(get_trades))
        .route("/api/portfolios/{id}/returns", get(get_returns))
        .route("/api/portfolios/{id}/stats", get(get_stats))
        // 新增：月度/年度收益
        .route("/api/portfolios/{id}/returns/monthly", get(get_monthly_returns))
        .route("/api/portfolios/{id}/returns/annual", get(get_annual_returns))
}

/// API 响应结构
#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    data: T,
    message: String,
}

fn ok_response<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: true,
        data,
        message: "ok".to_string(),
    })
}

fn err_json(msg: &str) -> Json<serde_json::Value> {
    Json(json!({
        "success": false,
        "data": null,
        "message": msg
    }))
}

/// 获取组合列表
async fn list_portfolios(State(state): State<AppState>) -> impl IntoResponse {
    let service = PortfolioService::new(state.db.clone());
    
    match service.get_user_portfolios("default") {
        Ok(portfolios) => ok_response(portfolios).into_response(),
        Err(e) => err_json(&format!("获取组合列表失败: {}", e)).into_response(),
    }
}

/// 创建组合
async fn create_portfolio(
    State(state): State<AppState>,
    Json(req): Json<CreatePortfolioRequest>,
) -> impl IntoResponse {
    let service = PortfolioService::new(state.db.clone());
    
    match service.create_portfolio("default", &req) {
        Ok(portfolio) => ok_response(portfolio).into_response(),
        Err(e) => err_json(&format!("创建组合失败: {}", e)).into_response(),
    }
}

/// 获取组合详情
async fn get_portfolio(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = PortfolioService::new(state.db.clone());
    
    match service.get_portfolio(&id, "default") {
        Ok(Some(portfolio)) => ok_response(portfolio).into_response(),
        Ok(None) => err_json("组合不存在").into_response(),
        Err(e) => err_json(&format!("获取组合详情失败: {}", e)).into_response(),
    }
}

/// 删除组合
async fn delete_portfolio(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = PortfolioService::new(state.db.clone());
    
    match service.delete_portfolio(&id, "default") {
        Ok(true) => ok_response(json!({ "message": "组合已删除" })).into_response(),
        Ok(false) => err_json("组合不存在或无权限").into_response(),
        Err(e) => err_json(&format!("删除组合失败: {}", e)).into_response(),
    }
}

/// 买入股票
async fn buy_stock(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<BuyStockRequest>,
) -> impl IntoResponse {
    let service = PortfolioService::new(state.db.clone());
    
    // 先检查组合是否存在
    match service.get_portfolio(&id, "default") {
        Ok(None) => return err_json("组合不存在").into_response(),
        Err(e) => return err_json(&format!("验证组合失败: {}", e)).into_response(),
        _ => {}
    }
    
    match service.buy_stock(&id, &req) {
        Ok(trade) => ok_response(trade).into_response(),
        Err(e) => err_json(&format!("买入失败: {}", e)).into_response(),
    }
}

/// 卖出股票
async fn sell_stock(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<SellStockRequest>,
) -> impl IntoResponse {
    let service = PortfolioService::new(state.db.clone());
    
    // 先检查组合是否存在
    match service.get_portfolio(&id, "default") {
        Ok(None) => return err_json("组合不存在").into_response(),
        Err(e) => return err_json(&format!("验证组合失败: {}", e)).into_response(),
        _ => {}
    }
    
    match service.sell_stock(&id, &req) {
        Ok(trade) => ok_response(trade).into_response(),
        Err(e) => err_json(&format!("卖出失败: {}", e)).into_response(),
    }
}

/// 获取持仓
async fn get_positions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = PortfolioService::new(state.db.clone());
    
    // 更新持仓价格（简化处理：从行情缓存获取最新价格）
    let quotes = state.cache.all_quotes.read().await;
    let prices: Vec<(String, f64)> = {
        let positions = match service.get_positions(&id) {
            Ok(p) => p,
            Err(e) => return err_json(&format!("获取持仓失败: {}", e)).into_response(),
        };
        
        positions
            .iter()
            .filter_map(|pos| {
                quotes
                    .iter()
                    .find(|q| q.symbol == pos.symbol)
                    .map(|q| (pos.symbol.clone(), q.price))
            })
            .collect()
    };
    drop(quotes);
    
    // 更新价格
    if !prices.is_empty() {
        if let Err(e) = service.update_positions_price(&id, &prices) {
            tracing::warn!("更新持仓价格失败: {}", e);
        }
    }
    
    match service.get_positions(&id) {
        Ok(positions) => ok_response(positions).into_response(),
        Err(e) => err_json(&format!("获取持仓失败: {}", e)).into_response(),
    }
}

/// 获取调仓记录
async fn get_trades(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<TradeQueryParams>,
) -> impl IntoResponse {
    let service = PortfolioService::new(state.db.clone());
    
    match service.get_trades(&id, &params) {
        Ok(trades) => ok_response(trades).into_response(),
        Err(e) => err_json(&format!("获取调仓记录失败: {}", e)).into_response(),
    }
}

/// 获取收益走势
async fn get_returns(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<ReturnQueryParams>,
) -> impl IntoResponse {
    let service = PortfolioService::new(state.db.clone());
    
    match service.get_returns(&id, &params) {
        Ok(returns) => ok_response(returns).into_response(),
        Err(e) => err_json(&format!("获取收益走势失败: {}", e)).into_response(),
    }
}

/// 获取组合统计
async fn get_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = PortfolioService::new(state.db.clone());

    match service.get_portfolio_stats(&id) {
        Ok(stats) => ok_response(stats).into_response(),
        Err(e) => err_json(&format!("获取统计信息失败: {}", e)).into_response(),
    }
}

/// 获取月度收益
async fn get_monthly_returns(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = PortfolioService::new(state.db.clone());

    match service.get_monthly_returns(&id) {
        Ok(returns) => ok_response(returns).into_response(),
        Err(e) => err_json(&format!("获取月度收益失败: {}", e)).into_response(),
    }
}

/// 获取年度收益
async fn get_annual_returns(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = PortfolioService::new(state.db.clone());

    match service.get_annual_returns(&id) {
        Ok(returns) => ok_response(returns).into_response(),
        Err(e) => err_json(&format!("获取年度收益失败: {}", e)).into_response(),
    }
}
