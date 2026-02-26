use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 告警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub symbol: Option<String>,
    pub rule_type: AlertRuleType,
    pub threshold: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

/// 告警规则类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertRuleType {
    PriceAbove,
    PriceBelow,
    ChangeAbove,
    ChangeBelow,
    VolumeAbove,
    TurnoverRateAbove,
}

/// 告警记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRecord {
    pub id: String,
    pub rule_id: String,
    pub symbol: String,
    pub name: String,
    pub message: String,
    pub triggered_at: DateTime<Utc>,
    pub read: bool,
}
