mod api;
mod auth;
mod data;
mod db;
mod models;
mod services;
mod sim;
mod ws;

use std::sync::Arc;
use axum::{Router, Json, routing::get};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::routes::{AppState, create_router};
use auth::create_auth_router;
use sim::create_sim_router;
use data::DataProvider;
use db::init_db;
use services::scanner::MarketScanner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "quantrust_server=info,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("QuantRust Server starting...");

    // Initialize database
    let db = init_db()?;
    tracing::info!("Database initialized");

    // Initialize data provider
    let provider = Arc::new(DataProvider::new());

    // Initialize market scanner
    let scanner = Arc::new(MarketScanner::new(provider.clone()));

    // Run initial scan
    tracing::info!("Running initial market scan...");
    if let Err(e) = scanner.scan().await {
        tracing::warn!("Initial scan failed (normal outside trading hours): {}", e);
    }

    // Create app state
    let state = AppState {
        cache: scanner.cache.clone(),
        provider: provider.clone(),
        db: db.clone(),
        sim_state: Arc::new(SimState::default()),
    };

    // Start periodic scan task (every 30 seconds)
    let scanner_clone = scanner.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = scanner_clone.scan().await {
                tracing::warn!("Market scan failed: {}", e);
            }
        }
    });

    // Build router
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .merge(create_router(state.clone()))
        .merge(create_auth_router(state.clone()))
        .merge(create_sim_router(state.clone()))
        .route("/ws", axum::routing::get(ws::ws_handler).with_state(state))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = "0.0.0.0:8080";
    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("WebSocket available at ws://{}/ws", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "quantrust-server",
        "version": "0.1.0",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
pub use sim::SimState;
