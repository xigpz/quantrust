use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
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
use crate::services::news_analyzer::{NewsAnalyzer, AnomalyPrediction, Sentiment};
use crate::services::screener::{ScreenerExecutionResult, ScreenerService};
use crate::services::ai_pattern::{AIPatternService, PatternResult, ScreenParams};
use crate::models::*;
use crate::db::DbPool;
use crate::sim::SimState;

/// 应用运行时配置
#[derive(Clone, Default)]
pub struct AppConfig {
    pub minimax_api_key: String,
}

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<ScannerCache>,
    pub provider: Arc<DataProvider>,
    pub db: DbPool,
    pub sim_state: Arc<SimState>,
    pub config: Arc<std::sync::RwLock<AppConfig>>,
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
        .route("/api/intraday/{symbol}", get(get_intraday))
        // 热点股票
        .route("/api/hot-stocks", get(get_hot_stocks))
        // 异动检测
        .route("/api/anomalies", get(get_anomalies))
        // 板块行情
        .route("/api/sectors", get(get_sectors))
        .route("/api/sectors/{code}/stocks", get(get_sector_stocks))
        // 资金流向
        .route("/api/money-flow", get(get_money_flow))
        // 涨停板
        .route("/api/limit-up", get(get_limit_up))
        // 新闻公告
        .route("/api/notices/{symbol}", get(get_stock_notices))
        .route("/api/notice/{art_code}", get(get_notice_detail))
        // 财经新闻
        .route("/api/news/{symbol}", get(get_stock_news))
        .route("/api/news/detail/{news_id}", get(get_news_detail))
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
        .route("/api/screener/catalog", get(get_screener_catalog))
        .route("/api/screener/run", post(run_screener_definition))
        .route("/api/screener/import-eastmoney", post(import_eastmoney_screener))
        .route("/api/screener/templates", get(list_screener_templates))
        .route("/api/screener/templates", post(create_screener_template))
        .route("/api/screener/templates/{id}", put(update_screener_template))
        .route("/api/screener/templates/{id}", delete(delete_screener_template))
        .route("/api/screener", post(screener_stocks))
        // 因子库
        .route("/api/factors/{symbol}", get(get_factors))
        // 参数优化
        .route("/api/backtest/optimize", post(optimize_params))
        // AutoML 因子选择
        .route("/api/factors/automl", post(automl_factor_selection))
        // 策略版本
        .route("/api/strategies/{id}/versions", get(get_strategy_versions))
        .route("/api/strategies/{id}/versions", post(create_strategy_version))
        // 搜索
        .route("/api/search", get(search_stocks))
        // 异动预测
        .route("/api/anomaly/predictions", get(get_anomaly_predictions))
        // AI形态分析
        .route("/api/ai/analyze-pattern", post(analyze_pattern))
        .route("/api/ai/screen-patterns", post(screen_patterns))
        // 每日推荐
        .route("/api/recommend", get(get_recommend))
        // 配置管理
        .route("/api/config", get(get_config))
        .route("/api/config", post(update_config))
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

fn err_response_msg(msg: &str) -> Json<ApiResponse<AutoMLResult>> {
    Json(ApiResponse {
        success: false,
        data: AutoMLResult {
            selected_factors: vec![],
            all_factors: vec![],
            method: "".to_string(),
            sample_size: 0,
        },
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

/// 获取分时数据
#[derive(Deserialize)]
pub struct IntradayParams {
    pub range: Option<String>,
}

async fn get_intraday(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<IntradayParams>,
) -> Json<ApiResponse<crate::models::intraday::IntradaySeries>> {
    let range = params.range.unwrap_or_else(|| "1d".to_string());

    match state.provider.get_intraday(&symbol, &range).await {
        Ok(data) => ok_response(data),
        Err(e) => {
            tracing::warn!("Failed to get intraday for {}: {}", symbol, e);
            err_response(&format!("获取分时数据失败: {}", e))
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

/// 获取板块成分股
async fn get_sector_stocks(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Json<ApiResponse<Vec<StockQuote>>> {
    match state.provider.get_sector_stocks(&code).await {
        Ok(stocks) => ok_response(stocks),
        Err(e) => err_response(&format!("获取板块成分股失败: {}", e)),
    }
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

/// 获取个股新闻公告列表
async fn get_stock_notices(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<StockNoticesResponse>> {
    let page_index: u32 = params.get("page")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let page_size: u32 = params.get("page_size")
        .and_then(|v| v.parse().ok())
        .unwrap_or(50);

    match state.provider.get_stock_notices(&symbol, page_index, page_size).await {
        Ok(notices) => ok_response(notices),
        Err(e) => err_response(&e.to_string()),
    }
}

/// 获取公告详情
async fn get_notice_detail(
    State(state): State<AppState>,
    Path(art_code): Path<String>,
) -> Json<ApiResponse<StockNoticeDetail>> {
    match state.provider.get_notice_detail(&art_code).await {
        Ok(detail) => ok_response(detail),
        Err(e) => err_response(&e.to_string()),
    }
}

/// 获取财经新闻列表
async fn get_stock_news(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<StockNewsResponse>> {
    let page_index: u32 = params.get("page")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let page_size: u32 = params.get("page_size")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);

    match state.provider.get_stock_news(&symbol, page_index, page_size).await {
        Ok(news) => ok_response(news),
        Err(e) => err_response(&e.to_string()),
    }
}

/// 获取新闻详情
async fn get_news_detail(
    State(state): State<AppState>,
    Path(news_id): Path<String>,
) -> Json<ApiResponse<StockNews>> {
    match state.provider.get_news_detail(&news_id).await {
        Ok(detail) => ok_response(detail),
        Err(e) => err_response(&e.to_string()),
    }
}

/// 获取自选股
async fn get_watchlist(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<WatchlistItem>>> {
    // 使用 tokio::task::spawn_blocking 在阻塞线程中执行数据库查询
    let symbols: Vec<(String, String, String, String, String)> = tokio::task::spawn_blocking(move || {
        let db = state.db.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT id, symbol, name, group_name, added_at FROM watchlist ORDER BY added_at DESC"
        ).unwrap();
        stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        }).unwrap().filter_map(|r| r.ok()).collect()
    }).await.unwrap();

    // 获取实时行情和所属板块
    let provider = state.provider.clone();
    let mut items = Vec::with_capacity(symbols.len());

    for (id, symbol, name, group_name, added_at) in symbols {
        let quote = provider.get_stock_detail(&symbol).await;
        let (price, change, change_pct, volume, turnover, turnover_rate) = match quote {
            Ok(q) => (Some(q.price), Some(q.change), Some(q.change_pct), Some(q.volume), Some(q.turnover), Some(q.turnover_rate)),
            Err(_) => (None, None, None, None, None, None),
        };

        // 获取所属板块
        let sector_name = match provider.get_stock_concepts(&symbol).await {
            Ok(concepts) if !concepts.is_empty() => Some(concepts[0].clone()),
            _ => None,
        };

        items.push(WatchlistItem { id, symbol, name, group_name, added_at, price, change, change_pct, volume, turnover, turnover_rate, sector_name });
    }

    ok_response(items)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WatchlistItem {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub group_name: String,
    pub added_at: String,
    // 实时行情数据
    pub price: Option<f64>,
    pub change: Option<f64>,
    pub change_pct: Option<f64>,
    pub volume: Option<f64>,
    pub turnover: Option<f64>,
    pub turnover_rate: Option<f64>,
    // 所属板块
    pub sector_name: Option<String>,
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

/// 每日推荐股票
#[derive(Serialize)]
pub struct StockRecommend {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
    pub score: f64,
    pub level: String,
    pub reasons: Vec<String>,
    pub risk_level: String,
    pub target_price: Option<f64>,
    pub stop_loss: Option<f64>,
}

async fn get_recommend(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<StockRecommend>>> {
    let quotes = state.cache.all_quotes.read().await;

    // 从热点股票中筛选推荐
    let mut recommends: Vec<StockRecommend> = quotes
        .iter()
        .filter(|q| q.price > 0.0 && q.turnover > 0.0)
        .filter(|q| q.change_pct > 0.0)  // 只推荐上涨的
        .take(20)
        .map(|q| {
            let (score, level, reasons, risk_level) = calculate_recommend(q);

            StockRecommend {
                symbol: q.symbol.clone(),
                name: q.name.clone(),
                price: q.price,
                change_pct: q.change_pct,
                score,
                level,
                reasons,
                risk_level,
                target_price: Some(q.price * 1.15),  // 目标价+15%
                stop_loss: Some(q.price * 0.95),     // 止损-5%
            }
        })
        .collect();

    // 按评分排序
    recommends.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    ok_response(recommends)
}

fn calculate_recommend(quote: &StockQuote) -> (f64, String, Vec<String>, String) {
    let mut score = 0.0;
    let mut reasons = Vec::new();
    let mut risk_level = "中".to_string();

    // 涨幅得分
    if quote.change_pct >= 5.0 && quote.change_pct <= 9.0 {
        score += 30.0;
        reasons.push("强势涨停".to_string());
    } else if quote.change_pct >= 3.0 {
        score += 20.0;
        reasons.push("涨幅较好".to_string());
    } else if quote.change_pct > 0.0 {
        score += 10.0;
    }

    // 成交额得分
    if quote.turnover > 1e9 {
        score += 25.0;
        reasons.push("成交活跃".to_string());
    } else if quote.turnover > 5e8 {
        score += 15.0;
    }

    // 换手率得分
    if quote.turnover_rate > 10.0 {
        score += 20.0;
        reasons.push("换手率高".to_string());
    } else if quote.turnover_rate > 5.0 {
        score += 10.0;
    }

    // 振幅得分
    if quote.amplitude > 8.0 {
        score += 15.0;
        reasons.push("波动大".to_string());
    }

    // 风险评估
    if quote.change_pct >= 9.0 {
        risk_level = "高".to_string();
        reasons.push("注意风险".to_string());
    } else if quote.turnover_rate > 20.0 {
        risk_level = "中高".to_string();
    }

    let level = if score >= 80.0 {
        "强烈推荐".to_string()
    } else if score >= 60.0 {
        "推荐".to_string()
    } else if score >= 40.0 {
        "观望".to_string()
    } else {
        "谨慎".to_string()
    };

    (score, level, reasons, risk_level)
}

/// 获取异动预测
async fn get_anomaly_predictions(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<AnomalyPrediction>>> {
    let quotes = state.cache.all_quotes.read().await;
    let mut predictions = Vec::new();
    
    // 基于市场数据预测
    // 过滤掉无效数据（非交易时段 turnover=0）
    for quote in quotes.iter() {
        // 跳过无效行情数据
        if quote.price <= 0.0 || quote.turnover <= 0.0 {
            continue;
        }
        let pt: String;
        let urgency: String;
        let reason: String;
        let label: String;
        
        if quote.change_pct >= 8.0 && quote.change_pct < 9.5 {
            pt = "即将涨停".to_string();
            urgency = "高".to_string();
            reason = format!("涨幅{}%，有望冲击涨停", quote.change_pct);
            label = "利好".to_string();
        } else if quote.change_pct <= -7.0 {
            pt = "风险警示".to_string();
            urgency = "高".to_string();
            reason = format!("跌幅{}%，注意风险", quote.change_pct);
            label = "利空".to_string();
        } else if quote.turnover_rate > 15.0 && quote.change_pct.abs() > 3.0 {
            pt = "放量异动".to_string();
            urgency = "中".to_string();
            reason = format!("换手率{:.1}%，成交量异常放大", quote.turnover_rate);
            label = if quote.change_pct > 0.0 { "利好".to_string() } else { "利空".to_string() };
        } else {
            continue;
        }
        
        predictions.push(AnomalyPrediction {
            symbol: quote.symbol.clone(),
            name: quote.name.clone(),
            pred_type: pt,
            sentiment: Sentiment {
                score: quote.change_pct / 100.0,
                label,
                keywords: vec![],
            },
            urgency,
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            reason,
        });
    }
    
    predictions.truncate(20);
    ok_response(predictions)
}

/// 分析单只股票形态
#[derive(Deserialize)]
pub struct AnalyzePatternRequest {
    pub symbol: String,
    pub name: Option<String>,
    pub days: Option<usize>,
}

async fn analyze_pattern(
    State(state): State<AppState>,
    Json(req): Json<AnalyzePatternRequest>,
) -> Json<ApiResponse<PatternResult>> {
    let days = req.days.unwrap_or(120);

    // 获取运行时配置的 API Key
    let api_key = {
        let config = state.config.read().unwrap();
        config.minimax_api_key.clone()
    };

    // 获取K线数据
    match state.provider.get_candles(&req.symbol, "daily", days as u32).await {
        Ok(candles) => {
            if candles.is_empty() {
                return err_response("无法获取K线数据");
            }

            let name = req.name.unwrap_or(req.symbol.clone());
            let ai_service = AIPatternService::new_with_key(&api_key);
            let result = ai_service.analyze_pattern(&req.symbol, &name, &candles).await;

            ok_response(result)
        }
        Err(e) => err_response(&format!("获取数据失败: {}", e)),
    }
}

/// 获取配置
#[derive(Serialize)]
pub struct ConfigResponse {
    pub minimax_api_key_set: bool,
}

async fn get_config(
    State(state): State<AppState>,
) -> Json<ApiResponse<ConfigResponse>> {
    let config = state.config.read().unwrap();
    ok_response(ConfigResponse {
        minimax_api_key_set: !config.minimax_api_key.is_empty(),
    })
}

/// 更新配置
#[derive(Deserialize)]
pub struct UpdateConfigRequest {
    pub minimax_api_key: Option<String>,
}

async fn update_config(
    State(state): State<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> Json<ApiResponse<ConfigResponse>> {
    let mut config = state.config.write().unwrap();

    if let Some(key) = req.minimax_api_key {
        config.minimax_api_key = key;
    }

    ok_response(ConfigResponse {
        minimax_api_key_set: !config.minimax_api_key.is_empty(),
    })
}

/// 筛选形态股票
#[derive(Deserialize)]
pub struct ScreenPatternsRequest {
    pub max_amplitude: Option<f64>,
    pub days: Option<usize>,
    pub min_consolidation_prob: Option<f64>,
    pub trend: Option<String>,
    pub limit: Option<usize>,
}

async fn screen_patterns(
    State(state): State<AppState>,
    Json(req): Json<ScreenPatternsRequest>,
) -> Json<ApiResponse<Vec<PatternResult>>> {
    let max_amplitude = req.max_amplitude.unwrap_or(25.0);
    let days = req.days.unwrap_or(120);
    let limit = req.limit.unwrap_or(50);

    // 获取运行时配置的 API Key
    let api_key = {
        let config = state.config.read().unwrap();
        config.minimax_api_key.clone()
    };

    // 获取所有股票行情
    let quotes = state.cache.all_quotes.read().await;
    let mut candidates: Vec<(String, String)> = Vec::new();

    // 过滤掉无效数据
    for quote in quotes.iter() {
        if quote.price > 0.0 && quote.turnover > 0.0 {
            candidates.push((quote.symbol.clone(), quote.name.clone()));
        }
    }
    drop(quotes);

    let ai_service = AIPatternService::new_with_key(&api_key);
    let mut results: Vec<PatternResult> = Vec::new();

    // 对候选股票进行分析
    for (symbol, name) in candidates.iter().take(200) {  // 限制分析数量
        match state.provider.get_candles(symbol, "daily", days as u32).await {
            Ok(candles) => {
                if candles.len() < 20 {
                    continue;
                }

                // 先用规则筛选横盘股票
                if !ai_service.screen_consolidation(&candles, max_amplitude, days) {
                    continue;
                }

                // 获取AI分析结果
                let result = ai_service.analyze_pattern(symbol, &name, &candles).await;

                // 应用额外筛选条件
                if let Some(min_prob) = req.min_consolidation_prob {
                    if result.consolidation_prob < min_prob {
                        continue;
                    }
                }

                if let Some(ref trend_filter) = req.trend {
                    let trend_str = match result.trend {
                        crate::services::ai_pattern::TrendType::Bullish | crate::services::ai_pattern::TrendType::Strong => "bullish",
                        crate::services::ai_pattern::TrendType::Bearish | crate::services::ai_pattern::TrendType::Weak => "bearish",
                        crate::services::ai_pattern::TrendType::Sideways => "sideways",
                    };
                    if trend_filter != trend_str {
                        continue;
                    }
                }

                results.push(result);

                if results.len() >= limit {
                    break;
                }
            }
            Err(_) => continue,
        }
    }

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
pub struct ScreenerRunRequest {
    pub definition: ScreenerDefinition,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct ScreenerImportRequest {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScreenerTemplateRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub definition: ScreenerDefinition,
    pub source_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct ScreenerTemplateUpsertRequest {
    pub name: String,
    pub description: Option<String>,
    pub definition: ScreenerDefinition,
    pub source_type: Option<String>,
}

fn empty_screener_definition() -> ScreenerDefinition {
    ScreenerDefinition {
        name: None,
        description: None,
        logic: ScreenerGroup {
            id: "root".to_string(),
            operator: ScreenerLogic::And,
            children: vec![],
        },
        sorts: vec![],
        columns: vec![],
        source: None,
        import_meta: None,
    }
}

fn empty_screener_result() -> ScreenerExecutionResult {
    ScreenerExecutionResult {
        total_count: 0,
        rows: vec![],
    }
}

async fn get_screener_catalog() -> Json<ApiResponse<Vec<ScreenerCatalogField>>> {
    ok_response(ScreenerService::new().catalog().to_vec())
}

async fn run_screener_definition(
    State(state): State<AppState>,
    Json(req): Json<ScreenerRunRequest>,
) -> Json<ApiResponse<ScreenerExecutionResult>> {
    let quotes = state.cache.all_quotes.read().await.clone();
    match ScreenerService::new().execute(&req.definition, &quotes, req.limit) {
        Ok(result) => ok_response(result),
        Err(errors) => Json(ApiResponse {
            success: false,
            data: empty_screener_result(),
            message: serde_json::to_string(&errors).unwrap_or_else(|_| "validation failed".to_string()),
        }),
    }
}

async fn import_eastmoney_screener(
    Json(req): Json<ScreenerImportRequest>,
) -> Json<ApiResponse<ScreenerDefinition>> {
    match ScreenerService::new().import_eastmoney_url(&req.url) {
        Ok(definition) => ok_response(definition),
        Err(error) => Json(ApiResponse {
            success: false,
            data: empty_screener_definition(),
            message: error.message,
        }),
    }
}

async fn list_screener_templates(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<ScreenerTemplateRecord>>> {
    let db = state.db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT id, name, description, definition_json, source_type, created_at, updated_at FROM screener_templates ORDER BY updated_at DESC"
    ) {
        Ok(stmt) => stmt,
        Err(error) => return err_response(&format!("Failed to load screener templates: {}", error)),
    };

    let templates = stmt
        .query_map([], |row| {
            let definition_json: String = row.get(3)?;
            let definition = serde_json::from_str(&definition_json).unwrap_or_else(|_| empty_screener_definition());
            Ok(ScreenerTemplateRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                definition,
                source_type: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .unwrap()
        .filter_map(|item| item.ok())
        .collect::<Vec<_>>();

    ok_response(templates)
}

async fn create_screener_template(
    State(state): State<AppState>,
    Json(req): Json<ScreenerTemplateUpsertRequest>,
) -> Json<ApiResponse<ScreenerTemplateRecord>> {
    if let Err(errors) = ScreenerService::new().validate_definition(&req.definition) {
        return Json(ApiResponse {
            success: false,
            data: ScreenerTemplateRecord {
                id: String::new(),
                name: String::new(),
                description: None,
                definition: empty_screener_definition(),
                source_type: String::new(),
                created_at: String::new(),
                updated_at: String::new(),
            },
            message: serde_json::to_string(&errors).unwrap_or_else(|_| "validation failed".to_string()),
        });
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let source_type = req.source_type.unwrap_or_else(|| "manual".to_string());
    let definition_json = serde_json::to_string(&req.definition).unwrap_or_else(|_| "{}".to_string());

    let db = state.db.lock().unwrap();
    match db.execute(
        "INSERT INTO screener_templates (id, user_id, name, description, definition_json, source_type, created_at, updated_at) VALUES (?1, 'default', ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, req.name, req.description, definition_json, source_type, now, now],
    ) {
        Ok(_) => ok_response(ScreenerTemplateRecord {
            id,
            name: req.name,
            description: req.description,
            definition: req.definition,
            source_type,
            created_at: now.clone(),
            updated_at: now,
        }),
        Err(error) => Json(ApiResponse {
            success: false,
            data: ScreenerTemplateRecord {
                id: String::new(),
                name: String::new(),
                description: None,
                definition: empty_screener_definition(),
                source_type: String::new(),
                created_at: String::new(),
                updated_at: String::new(),
            },
            message: format!("Failed to create screener template: {}", error),
        }),
    }
}

async fn update_screener_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ScreenerTemplateUpsertRequest>,
) -> Json<ApiResponse<ScreenerTemplateRecord>> {
    if let Err(errors) = ScreenerService::new().validate_definition(&req.definition) {
        return Json(ApiResponse {
            success: false,
            data: ScreenerTemplateRecord {
                id: String::new(),
                name: String::new(),
                description: None,
                definition: empty_screener_definition(),
                source_type: String::new(),
                created_at: String::new(),
                updated_at: String::new(),
            },
            message: serde_json::to_string(&errors).unwrap_or_else(|_| "validation failed".to_string()),
        });
    }

    let now = chrono::Utc::now().to_rfc3339();
    let source_type = req.source_type.unwrap_or_else(|| "manual".to_string());
    let definition_json = serde_json::to_string(&req.definition).unwrap_or_else(|_| "{}".to_string());

    let db = state.db.lock().unwrap();
    match db.execute(
        "UPDATE screener_templates SET name = ?2, description = ?3, definition_json = ?4, source_type = ?5, updated_at = ?6 WHERE id = ?1",
        rusqlite::params![id, req.name, req.description, definition_json, source_type, now],
    ) {
        Ok(_) => ok_response(ScreenerTemplateRecord {
            id,
            name: req.name,
            description: req.description,
            definition: req.definition,
            source_type,
            created_at: String::new(),
            updated_at: now,
        }),
        Err(error) => Json(ApiResponse {
            success: false,
            data: ScreenerTemplateRecord {
                id: String::new(),
                name: String::new(),
                description: None,
                definition: empty_screener_definition(),
                source_type: String::new(),
                created_at: String::new(),
                updated_at: String::new(),
            },
            message: format!("Failed to update screener template: {}", error),
        }),
    }
}

async fn delete_screener_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<String>> {
    let db = state.db.lock().unwrap();
    match db.execute("DELETE FROM screener_templates WHERE id = ?1", rusqlite::params![id]) {
        Ok(_) => ok_response("ok".to_string()),
        Err(error) => err_response(&format!("Failed to delete screener template: {}", error)),
    }
}
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
    pub macd_signal: f64,
    pub macd_hist: f64,
    pub volatility_20: f64,
    // 新增技术因子
    pub kdj_k: f64,
    pub kdj_d: f64,
    pub kdj_j: f64,
    pub boll_upper: f64,
    pub boll_middle: f64,
    pub boll_lower: f64,
    pub atr: f64,
    pub williams_r: f64,
    pub cci: f64,
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
    let macd_result = if candles.len() >= 26 {
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
    
    // 计算KDJ
    let (kdj_k, kdj_d, kdj_j) = if candles.len() >= 9 {
        calculate_kdj(&candles)
    } else {
        (50.0, 50.0, 50.0)
    };
    
    // 计算布林带
    let (boll_upper, boll_middle, boll_lower) = if candles.len() >= 20 {
        calculate_bollinger_bands(&candles, 20)
    } else {
        (0.0, 0.0, 0.0)
    };
    
    // 计算ATR
    let atr = if candles.len() >= 14 {
        calculate_atr(&candles, 14)
    } else {
        0.0
    };
    
    // 计算威廉指标
    let williams_r = if candles.len() >= 14 {
        calculate_williams_r(&candles, 14)
    } else {
        -50.0
    };
    
    // 计算CCI
    let cci = if candles.len() >= 14 {
        calculate_cci(&candles, 14)
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
        macd: macd_result,
        macd_signal: macd_result * 0.8,  // 简化
        macd_hist: macd_result * 0.2,      // 简化
        volatility_20,
        kdj_k,
        kdj_d,
        kdj_j,
        boll_upper,
        boll_middle,
        boll_lower,
        atr,
        williams_r,
        cci,
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

/// 计算KDJ指标
fn calculate_kdj(candles: &[Candle]) -> (f64, f64, f64) {
    if candles.len() < 9 { return (50.0, 50.0, 50.0); }
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let mut k_values = Vec::new();
    
    for i in 8..closes.len() {
        let window = &closes[i-8..=i];
        let low = window.iter().cloned().fold(f64::INFINITY, f64::min);
        let high = window.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let rsv = if high != low { (closes[i] - low) / (high - low) * 100.0 } else { 50.0 };
        k_values.push(rsv);
    }
    
    let k = k_values.iter().rev().take(3).sum::<f64>() / 3.0;
    let d = k_values.iter().rev().skip(1).take(3).sum::<f64>() / 3.0;
    let j = 3.0 * k - 2.0 * d;
    (k, d, j)
}

/// 计算布林带
fn calculate_bollinger_bands(candles: &[Candle], period: usize) -> (f64, f64, f64) {
    if candles.len() < period { return (0.0, 0.0, 0.0); }
    let closes: Vec<f64> = candles.iter().rev().take(period).map(|c| c.close).collect();
    let mean = closes.iter().sum::<f64>() / period as f64;
    let variance = closes.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / period as f64;
    let std = variance.sqrt();
    let upper = mean + 2.0 * std;
    let lower = mean - 2.0 * std;
    (upper, mean, lower)
}

/// 计算ATR (平均真实波幅)
fn calculate_atr(candles: &[Candle], period: usize) -> f64 {
    if candles.len() < period + 1 { return 0.0; }
    let mut tr_values = Vec::new();
    for i in 1..candles.len() {
        let high = candles[i].high;
        let low = candles[i].low;
        let prev_close = candles[i-1].close;
        let tr = (high - low).max((high - prev_close).abs()).max((low - prev_close).abs());
        tr_values.push(tr);
    }
    tr_values.iter().rev().take(period).sum::<f64>() / period as f64
}

/// 计算威廉指标
fn calculate_williams_r(candles: &[Candle], period: usize) -> f64 {
    if candles.len() < period { return -50.0; }
    let window: Vec<f64> = candles.iter().rev().take(period).map(|c| c.close).collect();
    let high = window.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let low = window.iter().cloned().fold(f64::INFINITY, f64::min);
    let current = window[0];
    if high == low { return -50.0; }
    -((high - current) / (high - low) * 100.0)
}

/// 计算CCI (商品通道指数)
fn calculate_cci(candles: &[Candle], period: usize) -> f64 {
    if candles.len() < period { return 0.0; }
    let mut typical_prices: Vec<f64> = candles.iter().map(|c| (c.high + c.low + c.close) / 3.0).collect();
    typical_prices.reverse();
    let window = &typical_prices[0..period];
    let sma = window.iter().sum::<f64>() / period as f64;
    let mean_deviation = window.iter().map(|x| (x - sma).abs()).sum::<f64>() / period as f64;
    if mean_deviation == 0.0 { return 0.0; }
    (typical_prices[0] - sma) / (0.015 * mean_deviation)
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

// ============ AutoML 因子选择 API ============

#[derive(Deserialize)]
pub struct AutoMLReq {
    pub symbols: Vec<String>,
    pub target: String,  // 预测目标: "return_5d", "return_20d", "updown"
    pub factor_pool: Option<Vec<String>>,
    pub method: Option<String>,  // "correlation", "importance", "greedy"
}

/// AutoML 因子选择结果
#[derive(Serialize, Clone)]
pub struct FactorImportance {
    pub factor: String,
    pub importance: f64,
    pub correlation: f64,
    pub selected: bool,
}

#[derive(Serialize)]
pub struct AutoMLResult {
    pub selected_factors: Vec<FactorImportance>,
    pub all_factors: Vec<FactorImportance>,
    pub method: String,
    pub sample_size: usize,
}

/// AutoML 自动因子选择
async fn automl_factor_selection(
    State(state): State<AppState>,
    Json(req): Json<AutoMLReq>,
) -> Json<ApiResponse<AutoMLResult>> {
    let method = req.method.unwrap_or_else(|| "importance".to_string());
    let target = req.target.clone();
    
    // 默认因子池
    let default_factors: Vec<String> = vec![
        "pe".to_string(), "pb".to_string(), "roe".to_string(), "roa".to_string(), 
        "gross_margin".to_string(), "net_margin".to_string(),
        "revenue_growth".to_string(), "profit_growth".to_string(), "rsi_14".to_string(), 
        "macd".to_string(), "volatility_20".to_string(),
        "kdj_k".to_string(), "kdj_d".to_string(), "boll_upper".to_string(), 
        "atr".to_string(), "williams_r".to_string(), "cci".to_string()
    ];
    let factor_pool = req.factor_pool.unwrap_or(default_factors);
    
    // 收集所有股票的因子数据
    let mut all_data: Vec<std::collections::HashMap<String, f64>> = Vec::new();
    
    for symbol in &req.symbols {
        let symbol_with_suffix = if symbol.contains('.') {
            symbol.clone()
        } else if symbol.starts_with('6') {
            format!("{}.SH", symbol)
        } else {
            format!("{}.SZ", symbol)
        };
        
        // 获取因子数据
        let quotes = state.cache.all_quotes.read().await;
        let quote = quotes.iter().find(|q| q.symbol == symbol_with_suffix);
        
        if let Some(q) = quote {
            let mut data = std::collections::HashMap::new();
            data.insert("pe".to_string(), q.pe_ratio);
            data.insert("pb".to_string(), if q.price > 0.0 { q.total_market_cap / q.price / 1e8 } else { 0.0 });
            // 模拟其他因子
            data.insert("roe".to_string(), 10.0 + (q.change_pct * 0.5).max(-5.0).min(5.0));
            data.insert("roa".to_string(), 5.0);
            data.insert("gross_margin".to_string(), 30.0);
            data.insert("net_margin".to_string(), 10.0);
            data.insert("revenue_growth".to_string(), q.change_pct * 0.8);
            data.insert("profit_growth".to_string(), q.change_pct * 0.6);
            data.insert("rsi_14".to_string(), 50.0);
            data.insert("macd".to_string(), 0.0);
            data.insert("volatility_20".to_string(), 2.0);
            data.insert("kdj_k".to_string(), 50.0);
            data.insert("kdj_d".to_string(), 50.0);
            data.insert("boll_upper".to_string(), q.price * 1.02);
            data.insert("atr".to_string(), q.price * 0.02);
            data.insert("williams_r".to_string(), -50.0);
            data.insert("cci".to_string(), 0.0);
            
            // 模拟目标变量
            let target_value = if target == "updown" {
                if q.change_pct > 0.0 { 1.0 } else { 0.0 }
            } else {
                q.change_pct
            };
            data.insert("__target__".to_string(), target_value);
            
            all_data.push(data);
        }
    }
    
    if all_data.is_empty() {
        return err_response_msg("没有获取到足够的数据");
    }
    
    // 计算各因子与目标的相关性/重要性
    let mut factor_scores: Vec<FactorImportance> = Vec::new();
    
    for factor in &factor_pool {
        let mut values: Vec<f64> = Vec::new();
        let mut targets: Vec<f64> = Vec::new();
        
        for data in &all_data {
            if let (Some(&v), Some(&t)) = (data.get(factor), data.get("__target__")) {
                if v.is_finite() && t.is_finite() {
                    values.push(v);
                    targets.push(t);
                }
            }
        }
        
        if values.len() >= 5 {
            let correlation = calculate_correlation(&values, &targets);
            let importance = correlation.abs();
            
            factor_scores.push(FactorImportance {
                factor: factor.clone(),
                importance,
                correlation,
                selected: false,
            });
        }
    }
    
    // 根据方法选择因子
    factor_scores.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
    
    let selected_count = (factor_scores.len() as f64 * 0.4).max(3.0) as usize;
    let mut selected_factors: Vec<FactorImportance> = Vec::new();
    
    match method.as_str() {
        "correlation" => {
            // 选择相关性最高的因子
            for (i, f) in factor_scores.iter().enumerate() {
                let mut f = f.clone();
                f.selected = i < selected_count;
                if f.selected {
                    selected_factors.push(f.clone());
                }
            }
        },
        "greedy" => {
            // 贪心选择：逐步添加因子，选择提升最大的
            let mut current_factors: Vec<String> = Vec::new();
            let mut best_score = 0.0;
            
            for _ in 0..selected_count {
                let mut best_factor = String::new();
                for f in &factor_scores {
                    if !current_factors.contains(&f.factor) {
                        let test_factors = {
                            let mut tf = current_factors.clone();
                            tf.push(f.factor.clone());
                            tf
                        };
                        let score = evaluate_factor_combination(&all_data, &test_factors, &target);
                        if score > best_score {
                            best_score = score;
                            best_factor = f.factor.clone();
                        }
                    }
                }
                if !best_factor.is_empty() {
                    current_factors.push(best_factor.clone());
                    if let Some(fi) = factor_scores.iter().find(|f| f.factor == best_factor) {
                        let mut f = fi.clone();
                        f.selected = true;
                        selected_factors.push(f);
                    }
                }
            }
        },
        _ => {
            // 默认: importance - 选择重要性最高的
            for (i, f) in factor_scores.iter().enumerate() {
                let mut f = f.clone();
                f.selected = i < selected_count;
                if f.selected {
                    selected_factors.push(f.clone());
                }
            }
        }
    }
    
    let result = AutoMLResult {
        selected_factors: selected_factors.clone(),
        all_factors: factor_scores,
        method: method.clone(),
        sample_size: all_data.len(),
    };
    
    ok_response(result)
}

fn calculate_correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.len() < 2 { return 0.0; }
    let n = x.len() as f64;
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    let sum_x2: f64 = x.iter().map(|a| a * a).sum();
    let sum_y2: f64 = y.iter().map(|b| b * b).sum();
    
    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();
    
    if denominator == 0.0 { 0.0 } else { numerator / denominator }
}

fn evaluate_factor_combination(
    data: &[std::collections::HashMap<String, f64>],
    factors: &[String],
    target: &str,
) -> f64 {
    // 简化评估：计算因子组合与目标的相关性均值
    let mut correlations: Vec<f64> = Vec::new();
    
    for factor in factors {
        let mut values: Vec<f64> = Vec::new();
        let mut targets: Vec<f64> = Vec::new();
        
        for d in data {
            if let (Some(&v), Some(&t)) = (d.get(factor), d.get("__target__")) {
                if v.is_finite() && t.is_finite() {
                    values.push(v);
                    targets.push(t);
                }
            }
        }
        
        if values.len() >= 5 {
            correlations.push(calculate_correlation(&values, &targets).abs());
        }
    }
    
    if correlations.is_empty() { 0.0 } else { correlations.iter().sum::<f64>() / correlations.len() as f64 }
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

#[cfg(test)]
mod screener_route_tests {
    use super::*;
    use axum::{body::{to_bytes, Body}, http::{Request, StatusCode}};
    use crate::db::init_db_at;
    use crate::services::scanner::ScannerCache;
    use chrono::Utc;
    use tower::ServiceExt;

    fn temp_db_path(name: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("quantrust-routes-{}-{}.db", name, uuid::Uuid::new_v4()));
        path
    }

    fn sample_quote() -> StockQuote {
        StockQuote {
            symbol: "000001.SZ".to_string(),
            name: "Alpha".to_string(),
            price: 12.0,
            change: 0.5,
            change_pct: 4.0,
            open: 11.4,
            high: 12.2,
            low: 11.1,
            pre_close: 11.5,
            volume: 1_000_000.0,
            turnover: 12_000_000.0,
            turnover_rate: 2.0,
            amplitude: 5.0,
            pe_ratio: 18.0,
            total_market_cap: 50_000_000.0,
            circulating_market_cap: 40_000_000.0,
            timestamp: Utc::now(),
            bid_prices: vec![],
            bid_volumes: vec![],
            ask_prices: vec![],
            ask_volumes: vec![],
        }
    }

    async fn response_json(response: axum::response::Response) -> serde_json::Value {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    async fn app_with_quote() -> (Router, std::path::PathBuf) {
        let cache = Arc::new(ScannerCache::new());
        *cache.all_quotes.write().await = vec![sample_quote()];

        let db_path = temp_db_path("screener");
        let state = AppState {
            cache,
            provider: Arc::new(DataProvider::new()),
            db: init_db_at(&db_path).unwrap(),
            sim_state: Arc::new(SimState::default()),
        };

        (create_router(state), db_path)
    }

    #[tokio::test]
    async fn screener_routes_expose_catalog_run_and_import() {
        let (app, db_path) = app_with_quote().await;

        let catalog_response = app.clone().oneshot(
            Request::builder().uri("/api/screener/catalog").body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(catalog_response.status(), StatusCode::OK);
        let catalog_json = response_json(catalog_response).await;
        assert_eq!(catalog_json["success"], true);
        assert!(catalog_json["data"].as_array().unwrap().iter().any(|field| field["field"] == "latest_price"));

        let run_payload = serde_json::json!({
            "definition": {
                "name": "Route run",
                "logic": {
                    "id": "root",
                    "operator": "AND",
                    "children": [
                        {
                            "id": "price-band",
                            "field": "latest_price",
                            "operator": "between",
                            "value": [10.0, 20.0]
                        }
                    ]
                },
                "sorts": [{ "field": "change_pct", "direction": "desc" }],
                "columns": ["symbol", "latest_price"]
            },
            "limit": 10
        });
        let run_response = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/screener/run")
                .header("content-type", "application/json")
                .body(Body::from(run_payload.to_string()))
                .unwrap()
        ).await.unwrap();
        assert_eq!(run_response.status(), StatusCode::OK);
        let run_json = response_json(run_response).await;
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["data"]["total_count"], 1);

        let import_payload = serde_json::json!({
            "url": "https://xuangu.eastmoney.com/result?filters=latest_price:between:10..20;change_pct:>=:3"
        });
        let import_response = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/screener/import-eastmoney")
                .header("content-type", "application/json")
                .body(Body::from(import_payload.to_string()))
                .unwrap()
        ).await.unwrap();
        assert_eq!(import_response.status(), StatusCode::OK);
        let import_json = response_json(import_response).await;
        assert_eq!(import_json["success"], true);
        assert_eq!(import_json["data"]["importMeta"]["importedConditions"], 2);

        std::fs::remove_file(db_path).ok();
    }

    #[tokio::test]
    async fn screener_routes_support_template_crud() {
        let (app, db_path) = app_with_quote().await;

        let create_payload = serde_json::json!({
            "name": "Momentum template",
            "description": "Saved from test",
            "definition": {
                "name": "Saved",
                "logic": {
                    "id": "root",
                    "operator": "AND",
                    "children": [
                        {
                            "id": "pct-up",
                            "field": "change_pct",
                            "operator": ">=",
                            "value": 3.0
                        }
                    ]
                },
                "sorts": [],
                "columns": ["symbol", "change_pct"]
            },
            "source_type": "manual"
        });
        let create_response = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/screener/templates")
                .header("content-type", "application/json")
                .body(Body::from(create_payload.to_string()))
                .unwrap()
        ).await.unwrap();
        assert_eq!(create_response.status(), StatusCode::OK);
        let create_json = response_json(create_response).await;
        let template_id = create_json["data"]["id"].as_str().unwrap().to_string();

        let list_response = app.clone().oneshot(
            Request::builder().uri("/api/screener/templates").body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);
        let list_json = response_json(list_response).await;
        assert_eq!(list_json["success"], true);
        assert_eq!(list_json["data"].as_array().unwrap().len(), 1);

        let update_payload = serde_json::json!({
            "name": "Momentum template v2",
            "description": "Updated",
            "definition": {
                "name": "Saved",
                "logic": {
                    "id": "root",
                    "operator": "AND",
                    "children": []
                },
                "sorts": [],
                "columns": ["symbol"]
            },
            "source_type": "manual"
        });
        let update_response = app.clone().oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/screener/templates/{template_id}"))
                .header("content-type", "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap()
        ).await.unwrap();
        assert_eq!(update_response.status(), StatusCode::OK);

        let delete_response = app.oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/screener/templates/{template_id}"))
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();
        assert_eq!(delete_response.status(), StatusCode::OK);

        std::fs::remove_file(db_path).ok();
    }
}