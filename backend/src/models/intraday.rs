use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntradayPoint {
    pub timestamp: String,
    pub price: f64,
    pub avg_price: f64,
    pub volume: f64,
    pub turnover: f64,
    pub change_pct: Option<f64>,
    pub change: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntradaySeries {
    pub symbol: String,
    pub name: String,
    pub range: String,
    pub pre_close: f64,
    pub points: Vec<IntradayPoint>,
}
