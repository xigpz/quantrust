use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 通知类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    /// 交易信号
    TradeSignal,
    /// 风险告警
    RiskAlert,
    /// 资金异动
    FlowAlert,
    /// 系统消息
    System,
    /// 定时报告
    Report,
}

/// 通知消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub content: String,
    pub notification_type: NotificationType,
    pub timestamp: i64,
}

/// 通知服务
pub struct NotificationService {
    feishu_webhook: Option<String>,
    email_smtp: Option<SmtpConfig>,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub to: Vec<String>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            feishu_webhook: None,
            email_smtp: None,
            enabled: false,
        }
    }

    /// 配置飞书 webhook
    pub fn with_feishu(webhook: &str) -> Self {
        Self {
            feishu_webhook: Some(webhook.to_string()),
            email_smtp: None,
            enabled: true,
        }
    }

    /// 配置邮件
    pub fn with_email(config: SmtpConfig) -> Self {
        Self {
            feishu_webhook: None,
            email_smtp: Some(config),
            enabled: true,
        }
    }

    /// 发送通知
    pub async fn send(&self, notification: &Notification) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        // 发送飞书通知
        if let Some(webhook) = &self.feishu_webhook {
            self.send_feishu(webhook, notification).await?;
        }

        // 发送邮件
        if let Some(smtp) = &self.email_smtp {
            self.send_email(smtp, notification).await?;
        }

        Ok(())
    }

    /// 发送飞书机器人消息
    async fn send_feishu(&self, webhook: &str, notification: &Notification) -> Result<(), String> {
        let client = reqwest::Client::new();
        
        let payload = serde_json::json!({
            "msg_type": "text",
            "content": {
                "text": format!("【{}】\n{}", notification.title, notification.content)
            }
        });

        client.post(webhook)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("飞书发送失败: {}", e))?;

        Ok(())
    }

    /// 发送邮件
    async fn send_email(&self, config: &SmtpConfig, notification: &Notification) -> Result<(), String> {
        // 简化实现，实际需要使用 lettre 或 async-smtp
        println!("[Email] To: {:?}, Subject: {}", config.to, notification.title);
        Ok(())
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

/// 告警管理器
pub struct AlertManager {
    notifications: Arc<RwLock<Vec<Notification>>>,
    notification_service: NotificationService,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(RwLock::new(Vec::new())),
            notification_service: NotificationService::new(),
        }
    }

    /// 发送交易信号
    pub async fn send_trade_signal(&self, symbol: &str, signal: &str, price: f64) {
        let notification = Notification {
            title: format!("交易信号 - {}", symbol),
            content: format!("信号: {}\n价格: {:.2}", signal, price),
            notification_type: NotificationType::TradeSignal,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.notifications.write().await.push(notification.clone());
        
        if let Err(e) = self.notification_service.send(&notification).await {
            eprintln!("发送通知失败: {}", e);
        }
    }

    /// 发送风险告警
    pub async fn send_risk_alert(&self, alert_type: &str, message: &str) {
        let notification = Notification {
            title: format!("风险告警 - {}", alert_type),
            content: message.to_string(),
            notification_type: NotificationType::RiskAlert,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.notifications.write().await.push(notification.clone());
        
        if let Err(e) = self.notification_service.send(&notification).await {
            eprintln!("发送通知失败: {}", e);
        }
    }

    /// 发送资金异动告警
    pub async fn send_flow_alert(&self, symbol: &str, inflow: f64) {
        let direction = if inflow > 0.0 { "流入" } else { "流出" };
        let notification = Notification {
            title: format!("资金异动 - {}", symbol),
            content: format!("主力{}: {:.1}万", direction, inflow),
            notification_type: NotificationType::FlowAlert,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.notifications.write().await.push(notification.clone());
        
        if let Err(e) = self.notification_service.send(&notification).await {
            eprintln!("发送通知失败: {}", e);
        }
    }

    /// 获取通知历史
    pub async fn get_history(&self, limit: usize) -> Vec<Notification> {
        let notifications = self.notifications.read().await;
        notifications.iter().rev().take(limit).cloned().collect()
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}
