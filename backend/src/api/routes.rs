use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::services::scanner::ScannerCache;
use crate::data::DataProvider;
use crate::services::backtest::BacktestEngine;
use crate::models::*;
use crate::db::DbPool;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<ScannerCache>,
    pub provider: Arc<DataProvider>,
    pub db: DbPool,
}

/// 创建路由
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // 市场概览
        .route("/api/market/overview", get(get_market_overview))
        // 行情数据
        .route("/api/quotes", get(get_quotes))
        .route("/api/quotes/{symbol}", get(get_stock_detail))
        .route("/api/candles/{symbol}", get(get_candles))
        // 热点股票
        .route("/api/hot-stocks", get(get_hot_stocks))
        // 异动检测
        .route("/api/anomalies", get(get_anomalies))
        // 板块行情
        .route("/api/sectors", get(get_sectors))
        // 资金流向
        .route("/api/money-flow", get(get_money_flow))
        // 涨停板
        .route("/api/limit-up", get(get_limit_up))
        // 自选股
        .route("/api/watchlist", get(get_watchlist))
        .route("/api/watchlist", post(add_watchlist))
        .route("/api/watchlist/{symbol}", axum::routing::delete(remove_watchlist))
        // 回测
        .route("/api/backtest", post(run_backtest))
        .route("/api/backtest/history", get(get_backtest_history))
        // 搜索
        .route("/api/search", get(search_stocks))
        .with_state(state)
}

// ============ 查询参数 ============

#[derive(Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Deserialize)]
pub struct CandleParams {
    pub period: Option<String>,
    pub count: Option<u32>,
}

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
}

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

fn ok_response<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: true,
        data,
        message: "ok".to_string(),
    })
}

fn err_response<T: Serialize + Default>(msg: &str) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: false,
        data: T::default(),
        message: msg.to_string(),
    })
}

// ============ 路由处理器 ============

/// 获取市场概览
async fn get_market_overview(
    State(state): State<AppState>,
) -> Json<ApiResponse<Option<MarketOverview>>> {
    let overview = state.cache.market_overview.read().await.clone();
    ok_response(overview)
}

/// 获取行情列表
async fn get_quotes(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Json<ApiResponse<Vec<StockQuote>>> {
    let quotes = state.cache.all_quotes.read().await.clone();
    let page = params.page.unwrap_or(1) as usize;
    let page_size = params.page_size.unwrap_or(50) as usize;
    let start = (page - 1) * page_size;
    let end = (start + page_size).min(quotes.len());
    
    if start < quotes.len() {
        ok_response(quotes[start..end].to_vec())
    } else {
        ok_response(vec![])
    }
}

/// 获取单只股票详情
async fn get_stock_detail(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Json<ApiResponse<Option<StockQuote>>> {
    match state.provider.get_stock_detail(&symbol).await {
        Ok(quote) => ok_response(Some(quote)),
        Err(e) => {
            tracing::warn!("Failed to get stock detail for {}: {}", symbol, e);
            ok_response(None)
        }
    }
}

/// 获取K线数据
async fn get_candles(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<CandleParams>,
) -> Json<ApiResponse<Vec<Candle>>> {
    let period = params.period.unwrap_or_else(|| "1d".to_string());
    let count = params.count.unwrap_or(120);
    
    match state.provider.get_candles(&symbol, &period, count).await {
        Ok(candles) => ok_response(candles),
        Err(e) => {
            tracing::warn!("Failed to get candles for {}: {}", symbol, e);
            ok_response(vec![])
        }
    }
}

/// 获取热点股票
async fn get_hot_stocks(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<HotStock>>> {
    let hot = state.cache.hot_stocks.read().await.clone();
    ok_response(hot)
}

/// 获取异动股票
async fn get_anomalies(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<AnomalyStock>>> {
    let anomalies = state.cache.anomaly_stocks.read().await.clone();
    ok_response(anomalies)
}

/// 获取板块行情
async fn get_sectors(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<SectorInfo>>> {
    let sectors = state.cache.sectors.read().await.clone();
    ok_response(sectors)
}

/// 获取资金流向
async fn get_money_flow(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<MoneyFlow>>> {
    let flows = state.cache.money_flow.read().await.clone();
    ok_response(flows)
}

/// 获取涨停板
async fn get_limit_up(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<StockQuote>>> {
    let stocks = state.cache.limit_up_stocks.read().await.clone();
    ok_response(stocks)
}

/// 获取自选股
async fn get_watchlist(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<WatchlistItem>>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, symbol, name, group_name, added_at FROM watchlist ORDER BY added_at DESC"
    ).unwrap();
    
    let items: Vec<WatchlistItem> = stmt.query_map([], |row| {
        Ok(WatchlistItem {
            id: row.get(0)?,
            symbol: row.get(1)?,
            name: row.get(2)?,
            group_name: row.get(3)?,
            added_at: row.get(4)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();

    ok_response(items)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WatchlistItem {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub group_name: String,
    pub added_at: String,
}

#[derive(Deserialize)]
pub struct AddWatchlistReq {
    pub symbol: String,
    pub name: String,
    pub group_name: Option<String>,
}

/// 添加自选股
async fn add_watchlist(
    State(state): State<AppState>,
    Json(req): Json<AddWatchlistReq>,
) -> Json<ApiResponse<String>> {
    let id = uuid::Uuid::new_v4().to_string();
    let group = req.group_name.unwrap_or_else(|| "默认".to_string());
    
    let db = state.db.lock().unwrap();
    match db.execute(
        "INSERT OR IGNORE INTO watchlist (id, user_id, symbol, name, group_name) VALUES (?1, 'default', ?2, ?3, ?4)",
        rusqlite::params![id, req.symbol, req.name, group],
    ) {
        Ok(_) => ok_response("ok".to_string()),
        Err(e) => err_response(&format!("Failed to add: {}", e)),
    }
}

/// 删除自选股
async fn remove_watchlist(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Json<ApiResponse<String>> {
    let db = state.db.lock().unwrap();
    match db.execute(
        "DELETE FROM watchlist WHERE symbol = ?1",
        rusqlite::params![symbol],
    ) {
        Ok(_) => ok_response("ok".to_string()),
        Err(e) => err_response(&format!("Failed to delete: {}", e)),
    }
}

/// 搜索股票
async fn search_stocks(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Json<ApiResponse<Vec<StockQuote>>> {
    let query = params.q.unwrap_or_default().to_uppercase();
    if query.is_empty() {
        return ok_response(vec![]);
    }

    let quotes = state.cache.all_quotes.read().await;
    let results: Vec<StockQuote> = quotes
        .iter()
        .filter(|q| {
            q.symbol.contains(&query) || q.name.contains(&query)
        })
        .take(20)
        .cloned()
        .collect();

    ok_response(results)
}

/// 回测请求
#[derive(Deserialize)]
pub struct BacktestRequest {
    pub symbol: String,
    pub period: Option<String>,
    pub count: Option<u32>,
    pub short_ma: Option<usize>,
    pub long_ma: Option<usize>,
    pub initial_capital: Option<f64>,
    pub commission_rate: Option<f64>,
}

/// 执行回测
async fn run_backtest(
    State(state): State<AppState>,
    Json(req): Json<BacktestRequest>,
) -> Json<ApiResponse<Option<BacktestResult>>> {
    let period = req.period.unwrap_or_else(|| "1d".to_string());
    let count = req.count.unwrap_or(500);
    let short_ma = req.short_ma.unwrap_or(5);
    let long_ma = req.long_ma.unwrap_or(20);
    let initial_capital = req.initial_capital.unwrap_or(100000.0);
    let commission_rate = req.commission_rate.unwrap_or(0.0003);

    // 获取K线数据
    let candles = match state.provider.get_candles(&req.symbol, &period, count).await {
        Ok(c) => c,
        Err(e) => {
            return Json(ApiResponse {
                success: false,
                data: None,
                message: format!("获取K线数据失败: {}", e),
            });
        }
    };

    let params = BacktestParams {
        strategy_id: "ma_crossover".to_string(),
        symbol: req.symbol.clone(),
        start_date: candles.first().map(|c| c.timestamp.clone()).unwrap_or_default(),
        end_date: candles.last().map(|c| c.timestamp.clone()).unwrap_or_default(),
        initial_capital,
        commission_rate,
        slippage: 0.0,
    };

    let engine = BacktestEngine::new();
    match engine.run_ma_crossover(&candles, &params, short_ma, long_ma) {
        Ok(result) => {
            // 保存到数据库
            if let Ok(db) = state.db.lock() {
                let _ = db.execute(
                    "INSERT INTO backtest_results (id, strategy_id, params, kpis, trades, equity_curve) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        result.id,
                        result.strategy_id,
                        serde_json::to_string(&result.params).unwrap_or_default(),
                        serde_json::to_string(&result.kpis).unwrap_or_default(),
                        serde_json::to_string(&result.trades).unwrap_or_default(),
                        serde_json::to_string(&result.equity_curve).unwrap_or_default(),
                    ],
                );
            }
            ok_response(Some(result))
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            message: format!("回测失败: {}", e),
        }),
    }
}

/// 获取回测历史
async fn get_backtest_history(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<BacktestResult>>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, strategy_id, params, kpis, trades, equity_curve, created_at FROM backtest_results ORDER BY created_at DESC LIMIT 20"
    ).unwrap();

    let results: Vec<BacktestResult> = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let strategy_id: String = row.get(1)?;
        let params_str: String = row.get(2)?;
        let kpis_str: String = row.get(3)?;
        let trades_str: String = row.get(4)?;
        let equity_str: String = row.get(5)?;
        let created_at: String = row.get(6)?;

        Ok((id, strategy_id, params_str, kpis_str, trades_str, equity_str, created_at))
    }).unwrap().filter_map(|r| {
        if let Ok((id, strategy_id, params_str, kpis_str, trades_str, equity_str, _created_at)) = r {
            Some(BacktestResult {
                id,
                strategy_id,
                params: serde_json::from_str(&params_str).unwrap_or(BacktestParams {
                    strategy_id: String::new(), symbol: String::new(),
                    start_date: String::new(), end_date: String::new(),
                    initial_capital: 0.0, commission_rate: 0.0, slippage: 0.0,
                }),
                kpis: serde_json::from_str(&kpis_str).unwrap_or(BacktestKpis {
                    total_return: 0.0, annual_return: 0.0, max_drawdown: 0.0,
                    sharpe_ratio: 0.0, sortino_ratio: 0.0, win_rate: 0.0,
                    profit_loss_ratio: 0.0, total_trades: 0, winning_trades: 0, losing_trades: 0,
                }),
                trades: serde_json::from_str(&trades_str).unwrap_or_default(),
                equity_curve: serde_json::from_str(&equity_str).unwrap_or_default(),
                created_at: chrono::Utc::now(),
            })
        } else {
            None
        }
    }).collect();

    ok_response(results)
}
