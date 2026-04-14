use axum::{extract::State, http::StatusCode, Json, Router};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::data::global_market::GlobalMarketService;
use crate::models::stock::{
    CommodityData, CryptoData, ForexData, HkStockQuote, UsIndex, UsStockQuote,
};

#[derive(Clone)]
pub struct GlobalState {
    pub market_service: Arc<RwLock<GlobalMarketService>>,
}

pub async fn get_us_indices(
    State(state): State<GlobalState>,
) -> Result<Json<Vec<UsIndex>>, StatusCode> {
    let service = state.market_service.read().await;
    service.get_us_indices().await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_hk_indices(
    State(state): State<GlobalState>,
) -> Result<Json<Vec<UsIndex>>, StatusCode> {
    let service = state.market_service.read().await;
    service.get_hk_indices().await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_commodities(
    State(state): State<GlobalState>,
) -> Result<Json<Vec<CommodityData>>, StatusCode> {
    let service = state.market_service.read().await;
    service.get_commodities().await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_forex(
    State(state): State<GlobalState>,
) -> Result<Json<Vec<ForexData>>, StatusCode> {
    let service = state.market_service.read().await;
    service.get_forex().await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_crypto(
    State(state): State<GlobalState>,
) -> Result<Json<Vec<CryptoData>>, StatusCode> {
    let service = state.market_service.read().await;
    service.get_crypto().await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_us_stocks(
    State(state): State<GlobalState>,
    axum::extract::Path(symbol): axum::extract::Path<String>,
) -> Result<Json<UsStockQuote>, StatusCode> {
    let service = state.market_service.read().await;
    service.get_us_stock(&symbol).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_hk_stock(
    State(state): State<GlobalState>,
    axum::extract::Path(symbol): axum::extract::Path<String>,
) -> Result<Json<HkStockQuote>, StatusCode> {
    let service = state.market_service.read().await;
    service.get_hk_stock(&symbol).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn create_global_router(state: GlobalState) -> Router {
    Router::new()
        .route("/api/global/indices", axum::routing::get(get_us_indices))
        .route("/api/global/hk/indices", axum::routing::get(get_hk_indices))
        .route("/api/global/commodities", axum::routing::get(get_commodities))
        .route("/api/global/forex", axum::routing::get(get_forex))
        .route("/api/global/crypto", axum::routing::get(get_crypto))
        .route("/api/global/us/{symbol}", axum::routing::get(get_us_stocks))
        .route("/api/global/hk/{symbol}", axum::routing::get(get_hk_stock))
        .with_state(state)
}
