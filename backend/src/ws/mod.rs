use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::api::routes::AppState;

/// WebSocket 推送的消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "market_overview")]
    MarketOverview { data: serde_json::Value },
    #[serde(rename = "hot_stocks")]
    HotStocks { data: serde_json::Value },
    #[serde(rename = "anomalies")]
    Anomalies { data: serde_json::Value },
    #[serde(rename = "quotes")]
    Quotes { data: serde_json::Value },
    #[serde(rename = "sectors")]
    Sectors { data: serde_json::Value },
    #[serde(rename = "heartbeat")]
    Heartbeat { timestamp: i64 },
}

/// WebSocket 广播通道
pub type WsBroadcast = broadcast::Sender<String>;

/// 创建广播通道
pub fn create_broadcast() -> WsBroadcast {
    let (tx, _) = broadcast::channel(256);
    tx
}

/// WebSocket 处理器
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// 处理单个 WebSocket 连接
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // 订阅广播
    let mut rx = state.cache.hot_stocks.read().await;
    drop(rx);

    // 发送初始数据
    let initial_data = build_initial_data(&state).await;
    if let Ok(json) = serde_json::to_string(&initial_data) {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    // 定期推送更新
    let cache = state.cache.clone();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let update = build_update_data(&cache).await;
                if let Ok(json) = serde_json::to_string(&update) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // 处理客户端消息 (如订阅特定股票)
                        tracing::debug!("Received WS message: {}", text);
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    tracing::info!("WebSocket connection closed");
}

/// 构建初始数据包
async fn build_initial_data(state: &AppState) -> serde_json::Value {
    let overview = state.cache.market_overview.read().await.clone();
    let hot = state.cache.hot_stocks.read().await.clone();
    let anomalies = state.cache.anomaly_stocks.read().await.clone();
    let sectors = state.cache.sectors.read().await.clone();

    serde_json::json!({
        "type": "initial",
        "data": {
            "market_overview": overview,
            "hot_stocks": hot,
            "anomalies": anomalies,
            "sectors": sectors,
        }
    })
}

/// 构建更新数据包
async fn build_update_data(cache: &Arc<crate::services::scanner::ScannerCache>) -> serde_json::Value {
    let overview = cache.market_overview.read().await.clone();
    let hot = cache.hot_stocks.read().await.clone();
    let anomalies = cache.anomaly_stocks.read().await.clone();

    serde_json::json!({
        "type": "update",
        "data": {
            "market_overview": overview,
            "hot_stocks": hot,
            "anomalies": anomalies,
        },
        "timestamp": chrono::Utc::now().timestamp()
    })
}
