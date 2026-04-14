use axum::{
    extract::State,
    Json,
};
use crate::api::routes::ApiResponse;
use crate::services::TimingOptimizer;

/// 获取交易时机信号 (核心API)
/// 返回当前交易指令、仓位建议、具体推荐等
pub async fn get_timing_signal(
) -> Json<ApiResponse<crate::models::TimingSignal>> {
    let signal = TimingOptimizer::get_timing_signal();
    Json(ApiResponse {
        success: true,
        data: signal,
        message: "ok".to_string(),
    })
}

/// 获取日内时段详情
pub async fn get_intraday_windows(
) -> Json<ApiResponse<crate::models::IntradayWindowDetail>> {
    let detail = TimingOptimizer::get_intraday_detail();
    Json(ApiResponse {
        success: true,
        data: detail,
        message: "ok".to_string(),
    })
}

/// 获取年度窗口详情
pub async fn get_annual_windows(
) -> Json<ApiResponse<crate::models::AnnualWindowDetail>> {
    let detail = TimingOptimizer::get_annual_detail();
    Json(ApiResponse {
        success: true,
        data: detail,
        message: "ok".to_string(),
    })
}
