use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::services::scanner::ScannerCache;
use crate::data::DataProvider;
use crate::services::backtest::{BacktestEngine, OptimizationResult as BtOptimizationResult};
use crate::services::momentum::MomentumStrategy;
use crate::services::risk::{RiskManager, RiskConfig, RiskReport};
use crate::services::financial::{DragonTigerService, DragonTigerData, FinancialService, FinancialFilter, FinancialData};
use crate::models::*;
use crate::db::DbPool;
use crate::sim::SimState;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<ScannerCache>,
    pub provider: Arc<DataProvider>,
    pub db: DbPool,
    pub sim_state: Arc<SimState>,
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
        // 策略管理
        .route("/api/strategies", get(get_strategies))
        .route("/api/strategies", post(create_strategy))
        .route("/api/strategies/{id}", get(get_strategy))
        .route("/api/strategies/{id}", axum::routing::delete(delete_strategy))
        // 回测
        .route("/api/backtest", post(run_backtest))
        .route("/api/momentum/{symbol}", get(get_momentum))
        .route("/api/backtest/history", get(get_backtest_history))
        // 风控
        .route("/api/risk/config", get(get_risk_config))
        .route("/api/risk/config", post(update_risk_config))
        .route("/api/risk/check", post(check_risk))
        // 龙虎榜
        .route("/api/dragon-tiger", get(get_dragon_tiger))
        // 选股器
        .route("/api/screener", post(screener_stocks))
        // 因子库
        .route("/api/factors/{symbol}", get(get_factors))
        // 参数优化
        .route("/api/backtest/optimize", post(optimize_params))
        // 策略版本
        .route("/api/strategies/{id}/versions", get(get_strategy_versions))
        .route("/api/strategies/{id}/versions", post(create_strategy_version))
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
    Query(params): Query<PaginationParams>,
) -> Json<ApiResponse<Vec<HotStock>>> {
    let hot = state.cache.hot_stocks.read().await.clone();
    let page = params.page.unwrap_or(1) as usize;
    let page_size = params.page_size.unwrap_or(50) as usize;
    let start = (page.saturating_sub(1)) * page_size;
    let end = (start + page_size).min(hot.len());

    if start < hot.len() {
        ok_response(hot[start..end].to_vec())
    } else {
        ok_response(vec![])
    }
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
/// 动量策略分析
async fn get_momentum(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Json<ApiResponse<crate::services::momentum::MomentumSignal>> {
    // 获取K线数据
    let candles = match state.provider.get_candles(&symbol, "101", 60).await {
        Ok(c) => c,
        Err(e) => return Json(ApiResponse {
            success: false,
            data: crate::services::momentum::MomentumSignal {
                score: 0,
                rsi: 0.0,
                macd_dif: 0.0,
                macd_dea: 0.0,
                macd_hist: 0.0,
                reasons: vec![format!("获取数据失败: {}", e)],
            },
            message: format!("获取数据失败: {}", e),
        }),
    };
    
    if candles.len() < 30 {
        return Json(ApiResponse {
            success: false,
            data: crate::services::momentum::MomentumSignal {
                score: 0,
                rsi: 0.0,
                macd_dif: 0.0,
                macd_dea: 0.0,
                macd_hist: 0.0,
                reasons: vec!["K线数据不足".to_string()],
            },
            message: "K线数据不足".to_string(),
        });
    }
    
    let strategy = MomentumStrategy::new();
    let signal = match strategy.buy_signal(&candles) {
        Ok(s) => s,
        Err(e) => return Json(ApiResponse {
            success: false,
            data: crate::services::momentum::MomentumSignal {
                score: 0,
                rsi: 0.0,
                macd_dif: 0.0,
                macd_dea: 0.0,
                macd_hist: 0.0,
                reasons: vec![format!("计算失败: {}", e)],
            },
            message: format!("计算失败: {}", e),
        }),
    };
    
    ok_response(signal)
}

// ============ 策略管理 API ============

#[derive(Debug, Serialize, Deserialize)]
#[derive(Default)]
pub struct Strategy {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub language: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct CreateStrategyReq {
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub language: Option<String>,
}

/// 获取策略列表
async fn get_strategies(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<Strategy>>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, name, description, code, language, created_at, updated_at FROM strategies ORDER BY updated_at DESC"
    ).unwrap();
    
    let items: Vec<Strategy> = stmt.query_map([], |row| {
        Ok(Strategy {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            code: row.get(3)?,
            language: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();

    ok_response(items)
}

/// 创建策略
async fn create_strategy(
    State(state): State<AppState>,
    Json(req): Json<CreateStrategyReq>,
) -> Json<ApiResponse<Strategy>> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let language = req.language.unwrap_or_else(|| "python".to_string());
    
    let db = state.db.lock().unwrap();
    match db.execute(
        "INSERT INTO strategies (id, name, description, code, language, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, req.name, req.description, req.code, language, now, now],
    ) {
        Ok(_) => {
            let strategy = Strategy {
                id,
                name: req.name,
                description: req.description,
                code: req.code,
                language,
                created_at: now.clone(),
                updated_at: now,
            };
            ok_response(strategy)
        }
        Err(e) => err_response(&format!("Failed to create strategy: {}", e)),
    }
}

/// 获取单个策略
async fn get_strategy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Strategy>> {
    let db = state.db.lock().unwrap();
    let result = db.query_row(
        "SELECT id, name, description, code, language, created_at, updated_at FROM strategies WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Strategy {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                code: row.get(3)?,
                language: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    );

    match result {
        Ok(s) => ok_response(s),
        Err(_) => err_response("Strategy not found"),
    }
}

/// 删除策略
async fn delete_strategy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<String>> {
    let db = state.db.lock().unwrap();
    match db.execute(
        "DELETE FROM strategies WHERE id = ?1",
        rusqlite::params![id],
    ) {
        Ok(_) => ok_response("ok".to_string()),
        Err(e) => err_response(&format!("Failed to delete: {}", e)),
    }
}

// ============ 风控 API ============

/// 获取风控配置
async fn get_risk_config() -> Json<ApiResponse<RiskConfig>> {
    ok_response(RiskConfig::default())
}

/// 更新风控配置
#[derive(Deserialize)]
pub struct UpdateRiskConfigReq {
    pub max_position_ratio: Option<f64>,
    pub max_single_position: Option<f64>,
    pub stop_loss_ratio: Option<f64>,
    pub take_profit_ratio: Option<f64>,
    pub max_drawdown_threshold: Option<f64>,
    pub enabled: Option<bool>,
}

async fn update_risk_config(
    Json(req): Json<UpdateRiskConfigReq>,
) -> Json<ApiResponse<RiskConfig>> {
    let mut config = RiskConfig::default();
    
    if let Some(v) = req.max_position_ratio { config.max_position_ratio = v; }
    if let Some(v) = req.max_single_position { config.max_single_position = v; }
    if let Some(v) = req.stop_loss_ratio { config.stop_loss_ratio = v; }
    if let Some(v) = req.take_profit_ratio { config.take_profit_ratio = v; }
    if let Some(v) = req.max_drawdown_threshold { config.max_drawdown_threshold = v; }
    if let Some(v) = req.enabled { config.enabled = v; }
    
    ok_response(config)
}

/// 风控检查请求
#[derive(Deserialize)]
pub struct RiskCheckReq {
    pub action: String,       // "buy" or "sell"
    pub symbol: String,
    pub entry_price: Option<f64>,
    pub current_price: f64,
    pub quantity: i32,
    pub current_position: f64,
    pub total_capital: f64,
}

/// 风控检查结果
#[derive(Serialize)]
pub struct RiskCheckResult {
    pub allowed: bool,
    pub reason: String,
    pub stop_loss_price: Option<f64>,
    pub take_profit_price: Option<f64>,
    pub max_quantity: i32,
}

/// 风控检查
async fn check_risk(
    Json(req): Json<RiskCheckReq>,
) -> Json<ApiResponse<RiskCheckResult>> {
    let manager = RiskManager::with_default();
    let config = manager.config.clone();
    
    let new_position = req.current_price * req.quantity as f64;
    
    let (allowed, reason) = if req.action == "buy" {
        manager.can_buy(req.current_position, new_position, req.total_capital)
    } else {
        (true, "卖出不受仓位限制".to_string())
    };
    
    // 计算止损止盈价格
    let (stop_loss_price, take_profit_price) = if let Some(entry) = req.entry_price {
        let stop = entry * (1.0 - config.stop_loss_ratio);
        let profit = entry * (1.0 + config.take_profit_ratio);
        (Some(stop), Some(profit))
    } else {
        (None, None)
    };
    
    // 计算最大可买数量
    let max_qty = ((config.max_single_position * req.total_capital) / req.current_price) as i32;
    
    ok_response(RiskCheckResult {
        allowed,
        reason,
        stop_loss_price,
        take_profit_price,
        max_quantity: max_qty,
    })
}

// ============ 龙虎榜 API ============

/// 获取龙虎榜数据
async fn get_dragon_tiger() -> Json<ApiResponse<Vec<DragonTigerData>>> {
    let service = DragonTigerService::new();
    match service.get_daily_list().await {
        Ok(data) => ok_response(data),
        Err(e) => Json(ApiResponse {
            success: false,
            data: vec![],
            message: format!("获取龙虎榜失败: {}", e),
        }),
    }
}

// ============ 选股器 API ============

#[derive(Deserialize)]
pub struct ScreenerReq {
    pub min_pe: Option<f64>,        // 最小市盈率
    pub max_pe: Option<f64>,        // 最大市盈率
    pub min_pb: Option<f64>,        // 最小市净率
    pub max_pb: Option<f64>,        // 最大市净率
    pub min_roe: Option<f64>,       // 最小ROE
    pub min_growth: Option<f64>,    // 最小增长率
    pub min_volume: Option<f64>,    // 最小成交量
    pub change_pct_min: Option<f64>, // 最小涨跌幅
    pub limit: Option<usize>,       // 返回数量
}

/// 选股器
async fn screener_stocks(
    State(state): State<AppState>,
    Json(req): Json<ScreenerReq>,
) -> Json<ApiResponse<Vec<StockQuote>>> {
    let limit = req.limit.unwrap_or(50);
    
    // 获取所有股票
    let quotes = state.cache.all_quotes.read().await;
    
    let mut results: Vec<StockQuote> = quotes
        .iter()
        .filter(|q| {
            // PE 筛选 - 只有设置了才筛选
            if let Some(min) = req.min_pe {
                if q.pe_ratio <= 0.0 || q.pe_ratio < min as f64 { return false; }
            }
            if let Some(max) = req.max_pe {
                if max > 0.0 && (q.pe_ratio <= 0.0 || q.pe_ratio > max as f64) { return false; }
            }
            
            // PB 筛选
            if let Some(max) = req.max_pb {
                if max > 0.0 {
                    let pb = if q.price > 0.0 { q.total_market_cap / q.price } else { 0.0 };
                    if pb <= 0.0 || pb > max as f64 { return false; }
                }
            }
            
            // 涨跌幅筛选
            if let Some(min) = req.change_pct_min {
                if q.change_pct < min as f64 { return false; }
            }
            
            // 成交量筛选
            if let Some(min) = req.min_volume {
                if q.volume < min as f64 { return false; }
            }
            
            true
        })
        .take(limit)
        .cloned()
        .collect();
    
    // 按涨跌幅排序
    results.sort_by(|a, b| b.change_pct.partial_cmp(&a.change_pct).unwrap_or(std::cmp::Ordering::Equal));
    
    ok_response(results)
}

// ============ 因子库 API ============

#[derive(Serialize)]
pub struct FactorData {
    // 估值因子
    pub pe: f64,
    pub pb: f64,
    pub ps: f64,
    // 盈利因子
    pub roe: f64,
    pub roa: f64,
    pub gross_margin: f64,
    pub net_margin: f64,
    // 成长因子
    pub revenue_growth: f64,
    pub profit_growth: f64,
    // 技术因子
    pub rsi_14: f64,
    pub macd: f64,
    pub volatility_20: f64,
    // 风险因子
    pub beta: f64,
    pub debt_ratio: f64,
}

/// 获取股票因子数据
async fn get_factors(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Json<ApiResponse<FactorData>> {
    // 自动添加市场后缀
    let symbol_with_suffix = if symbol.contains('.') {
        symbol.clone()
    } else if symbol.starts_with('6') {
        format!("{}.SH", symbol)
    } else if symbol.starts_with('0') || symbol.starts_with('3') {
        format!("{}.SZ", symbol)
    } else {
        symbol.clone()
    };
    
    // 获取行情数据
    let quotes = state.cache.all_quotes.read().await;
    let quote = quotes.iter().find(|q| q.symbol == symbol_with_suffix);
    
    let pe = quote.map(|q| q.pe_ratio).unwrap_or(0.0);
    let price = quote.map(|q| q.price).unwrap_or(0.0);
    let change_pct = quote.map(|q| q.change_pct).unwrap_or(0.0);
    
    // 获取K线计算技术因子
    let candles = state.provider.get_candles(&symbol_with_suffix, "101", 30).await.unwrap_or_default();
    
    // 计算RSI
    let rsi_14 = if candles.len() >= 15 {
        calculate_rsi(&candles, 14)
    } else {
        50.0
    };
    
    // 计算MACD
    let macd = if candles.len() >= 26 {
        calculate_macd(&candles)
    } else {
        0.0
    };
    
    // 计算波动率
    let volatility_20 = if candles.len() >= 20 {
        calculate_volatility(&candles, 20)
    } else {
        0.0
    };
    
    // 模拟其他因子（实际应该从财务数据获取）
    let factor_data = FactorData {
        pe,
        pb: if price > 0.0 { (quote.map(|q| q.total_market_cap).unwrap_or(0.0) / 1e8) / price } else { 0.0 },
        ps: 0.0,
        roe: 10.0 + (change_pct * 0.5).max(-5.0).min(5.0),  // 模拟
        roa: 5.0,
        gross_margin: 30.0,
        net_margin: 10.0,
        revenue_growth: change_pct * 0.8,
        profit_growth: change_pct * 0.6,
        rsi_14,
        macd,
        volatility_20,
        beta: 1.0 + (change_pct * 0.05).max(-0.3).min(0.3),
        debt_ratio: 50.0,
    };
    
    ok_response(factor_data)
}

fn calculate_rsi(candles: &[Candle], period: usize) -> f64 {
    if candles.len() < period + 1 { return 50.0; }
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let mut gains = Vec::new();
    let mut losses = Vec::new();
    for i in 1..closes.len() {
        let change = closes[i] - closes[i - 1];
        if change > 0.0 { gains.push(change); losses.push(0.0); }
        else { gains.push(0.0); losses.push(-change); }
    }
    let avg_gain: f64 = gains.iter().rev().take(period).sum::<f64>() / period as f64;
    let avg_loss: f64 = losses.iter().rev().take(period).sum::<f64>() / period as f64;
    if avg_loss == 0.0 { return 100.0; }
    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}

fn calculate_macd(candles: &[Candle]) -> f64 {
    if candles.len() < 26 { return 0.0; }
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    // 简化版 MACD
    let ema12 = closes.iter().rev().take(12).sum::<f64>() / 12.0;
    let ema26 = closes.iter().rev().take(26).sum::<f64>() / 26.0;
    ema12 - ema26
}

fn calculate_volatility(candles: &[Candle], period: usize) -> f64 {
    if candles.len() < period { return 0.0; }
    let closes: Vec<f64> = candles.iter().rev().take(period).map(|c| c.close).collect();
    let mean = closes.iter().sum::<f64>() / period as f64;
    let variance = closes.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / period as f64;
    (variance.sqrt() / mean) * 100.0
}

// ============ 参数优化 API ============

#[derive(Deserialize)]
pub struct OptimizeReq {
    pub symbol: String,
    pub period: Option<String>,
    pub count: Option<u32>,
    pub initial_capital: Option<f64>,
}

#[derive(Serialize)]
pub struct OptimizationResult {
    pub params: serde_json::Value,
    pub total_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
}

/// 参数优化
async fn optimize_params(
    State(state): State<AppState>,
    Json(req): Json<OptimizeReq>,
) -> Json<ApiResponse<Vec<OptimizationResult>>> {
    let period = req.period.unwrap_or_else(|| "1d".to_string());
    let count = req.count.unwrap_or(500);
    let initial_capital = req.initial_capital.unwrap_or(100000.0);
    
    // 获取K线数据
    let candles = match state.provider.get_candles(&req.symbol, &period, count).await {
        Ok(c) => c,
        Err(e) => return Json(ApiResponse {
            success: false,
            data: vec![],
            message: format!("获取数据失败: {}", e),
        }),
    };
    
    let params = BacktestParams {
        strategy_id: "ma_crossover".to_string(),
        symbol: req.symbol.clone(),
        start_date: candles.first().map(|c| c.timestamp.clone()).unwrap_or_default(),
        end_date: candles.last().map(|c| c.timestamp.clone()).unwrap_or_default(),
        initial_capital,
        commission_rate: 0.0003,
        slippage: 0.0,
    };
    
    let engine = BacktestEngine::new();
    let results = engine.optimize_ma_params(&candles, &params);
    
    // 只返回前50个结果
    let results: Vec<OptimizationResult> = results.into_iter().take(50).map(|r| OptimizationResult {
        params: r.params,
        total_return: r.total_return,
        sharpe_ratio: r.sharpe_ratio,
        max_drawdown: r.max_drawdown,
        win_rate: r.win_rate,
    }).collect();
    
    ok_response(results)
}

// ============ 策略版本管理 API ============

#[derive(Serialize, Deserialize)]
pub struct StrategyVersion {
    pub id: String,
    pub strategy_id: String,
    pub version: i32,
    pub code: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CreateVersionReq {
    pub code: String,
    pub description: Option<String>,
}

/// 获取策略版本历史
async fn get_strategy_versions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Vec<StrategyVersion>>> {
    let db = state.db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT id, strategy_id, version, code, description, created_at FROM strategy_versions WHERE strategy_id = ?1 ORDER BY version DESC"
    ) {
        Ok(s) => s,
        Err(_) => return Json(ApiResponse { success: true, data: vec![], message: "ok".to_string() }),
    };
    
    let versions: Vec<StrategyVersion> = stmt.query_map([id], |row| {
        Ok(StrategyVersion {
            id: row.get(0)?,
            strategy_id: row.get(1)?,
            version: row.get(2)?,
            code: row.get(3)?,
            description: row.get(4)?,
            created_at: row.get(5)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    
    ok_response(versions)
}

/// 创建新版本
async fn create_strategy_version(
    State(state): State<AppState>,
    Path(strategy_id): Path<String>,
    Json(req): Json<CreateVersionReq>,
) -> Json<ApiResponse<StrategyVersion>> {
    let db = state.db.lock().unwrap();
    
    // 获取最新版本号
    let latest_version: i32 = db.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM strategy_versions WHERE strategy_id = ?1",
        [&strategy_id],
        |row| row.get(0),
    ).unwrap_or(0);
    
    let new_version = latest_version + 1;
    let version_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    match db.execute(
        "INSERT INTO strategy_versions (id, strategy_id, version, code, description, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![version_id, strategy_id, new_version, req.code, req.description, now],
    ) {
        Ok(_) => ok_response(StrategyVersion {
            id: version_id,
            strategy_id,
            version: new_version,
            code: req.code,
            description: req.description,
            created_at: now,
        }),
        Err(e) => Json(ApiResponse { success: false, data: StrategyVersion { id: String::new(), strategy_id: String::new(), version: 0, code: String::new(), description: None, created_at: String::new() }, message: format!("创建版本失败: {}", e) }),
    }
}
