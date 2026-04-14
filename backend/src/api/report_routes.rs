//! 报告 API 路由
//!
//! 提供每日、每周、每月交易报告的 API

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::DbPool;
use crate::services::report_generator::{
    self, ReportGenerator, DailyTradingReport, WeeklyReport, MonthlyReport, format_report_markdown,
};

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub report_generator: Arc<RwLock<ReportGenerator>>,
}

/// 通用 API 响应
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.to_string()),
        }
    }
}

/// 日期范围查询
#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// 报告摘要
#[derive(Debug, Serialize)]
pub struct ReportSummary {
    pub date: String,
    pub total_pnl: f64,
    pub pnl_ratio: f64,
    pub trades_count: i32,
    pub win_rate: f64,
}

/// 历史报告列表项
#[derive(Debug, Serialize)]
pub struct HistoryReportItem {
    pub date: String,
    pub total_pnl: f64,
    pub pnl_ratio: f64,
    pub positions_count: i32,
    pub trades_count: i32,
    pub win_trades: i32,
    pub lose_trades: i32,
}

/// 创建报告路由
pub fn create_report_router(db_pool: DbPool) -> Router {
    let report_generator = Arc::new(RwLock::new(ReportGenerator::new(db_pool.clone())));
    let state = AppState { db_pool, report_generator };

    Router::new()
        .route("/api/reports/daily/{date}", get(get_daily_report))
        .route("/api/reports/weekly", get(get_weekly_report))
        .route("/api/reports/monthly", get(get_monthly_report))
        .route("/api/reports/summary", get(get_report_summary))
        .route("/api/reports/history", get(get_history_reports))
        .route("/api/reports/generate", post(generate_report))
        .route("/api/reports/markdown/{date}", get(get_report_markdown))
        .with_state(state)
}

/// 获取日报
async fn get_daily_report(
    State(state): State<AppState>,
    Path(date): Path<String>,
) -> Result<Json<ApiResponse<DailyTradingReport>>, StatusCode> {
    let generator = state.report_generator.read().await;
    match generator.generate_daily_report(&date).await {
        Ok(report) => Ok(Json(ApiResponse::success(report))),
        Err(e) => Ok(Json(ApiResponse::error(&e))),
    }
}

/// 获取周报
async fn get_weekly_report(
    State(state): State<AppState>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<ApiResponse<WeeklyReport>>, StatusCode> {
    let start = query.start_date.unwrap_or_else(|| {
        chrono::Local::now().date_naive() - chrono::Duration::days(6)
    }.format("%Y-%m-%d").to_string());
    let end = query.end_date.unwrap_or_else(|| {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    });

    let generator = state.report_generator.read().await;
    match generator.generate_weekly_report(&start, &end).await {
        Ok(report) => Ok(Json(ApiResponse::success(report))),
        Err(e) => Ok(Json(ApiResponse::error(&e))),
    }
}

/// 获取月报
async fn get_monthly_report(
    State(state): State<AppState>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<ApiResponse<MonthlyReport>>, StatusCode> {
    let now = chrono::Local::now();
    let year = query.start_date.as_ref()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(now.format("%Y").to_string().parse().unwrap_or(2024));
    let month = query.end_date.as_ref()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(now.format("%m").to_string().parse().unwrap_or(1));

    let generator = state.report_generator.read().await;
    match generator.generate_monthly_report(year, month).await {
        Ok(report) => Ok(Json(ApiResponse::success(report))),
        Err(e) => Ok(Json(ApiResponse::error(&e))),
    }
}

/// 获取报告摘要
async fn get_report_summary(
    State(state): State<AppState>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<ApiResponse<Vec<ReportSummary>>>, StatusCode> {
    let days = query.start_date.as_ref()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(7);

    let generator = state.report_generator.read().await;
    let mut summaries = vec![];

    let today = chrono::Local::now().date_naive();
    for i in 0..days as i64 {
        let date = today - chrono::Duration::days(i);
        match generator.generate_daily_report(&date.format("%Y-%m-%d").to_string()).await {
            Ok(report) => {
                summaries.push(ReportSummary {
                    date: report.date,
                    total_pnl: report.total_pnl,
                    pnl_ratio: report.pnl_ratio,
                    trades_count: report.trades_count,
                    win_rate: report.win_rate,
                });
            }
            Err(_) => continue,
        }
    }

    Ok(Json(ApiResponse::success(summaries)))
}

/// 获取历史报告列表
async fn get_history_reports(
    State(state): State<AppState>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<ApiResponse<Vec<HistoryReportItem>>>, StatusCode> {
    use crate::db::autonomous_trading::get_history_reports as db_get_history;

    let limit = query.start_date.as_ref()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(30);

    match db_get_history(&state.db_pool, limit) {
        Ok(reports) => {
            let items: Vec<HistoryReportItem> = reports.into_iter().map(|r| {
                // 解析 positions_json 和 trades_json 来获取数量
                let positions_count = serde_json::from_str::<Vec<serde_json::Value>>(&r.positions_json)
                    .map(|v| v.len() as i32)
                    .unwrap_or(0);
                let trades_count = r.trades_count;
                let win_trades = r.win_trades;
                let lose_trades = r.lose_trades;

                HistoryReportItem {
                    date: r.date,
                    total_pnl: r.total_pnl,
                    pnl_ratio: r.pnl_ratio,
                    positions_count,
                    trades_count,
                    win_trades,
                    lose_trades,
                }
            }).collect();
            Ok(Json(ApiResponse::success(items)))
        }
        Err(e) => Ok(Json(ApiResponse::error(&format!("获取历史报告失败: {}", e)))),
    }
}

/// 手动触发报告生成
async fn generate_report(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<DailyTradingReport>>, StatusCode> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let date = payload.get("date")
        .and_then(|v| v.as_str())
        .unwrap_or(&today)
        .to_string();

    let generator = state.report_generator.read().await;
    match generator.generate_daily_report(&date).await {
        Ok(report) => Ok(Json(ApiResponse::success(report))),
        Err(e) => Ok(Json(ApiResponse::error(&e))),
    }
}

/// 获取 Markdown 格式的报告
async fn get_report_markdown(
    State(state): State<AppState>,
    Path(date): Path<String>,
) -> Result<String, StatusCode> {
    let generator = state.report_generator.read().await;
    match generator.generate_daily_report(&date).await {
        Ok(report) => Ok(format_report_markdown(&report)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}
