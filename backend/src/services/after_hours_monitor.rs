//! 盘后监控服务 (After Hours Monitor)
//!
//! 负责盘后市场监控、新闻追踪、美股开盘分析等

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{Local, NaiveTime, Duration};
use crate::services::agent_perception::PerceptionModule;

/// 监控任务类型
#[derive(Debug, Clone)]
pub enum MonitorTask {
    UsMarketAnalysis,      // 美股收盘分析
    AsiaMarketAnalysis,    // 亚太市场分析
    NewsTracking,          // 新闻追踪
    PositionAlert,        // 持仓预警
    ExternalMarketScan,    // 外盘扫描
}

/// 预警类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    PriceDrop,           // 价格下跌预警
    NewsRisk,           // 新闻风险预警
    UnusualVolume,      // 异常成交量
    LimitUp,            // 涨停预警
    LimitDown,          // 跌停预警
    SectorRotation,     // 板块轮动预警
    ExternalMarket,     // 外盘影响预警
}

/// 预警信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub symbol: Option<String>,
    pub title: String,
    pub content: String,
    pub severity: AlertSeverity,
    pub timestamp: String,
    pub action_suggested: String,
}

/// 预警级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 外盘分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalMarketAnalysis {
    pub us_indices: Vec<MarketIndexInfo>,
    pub asian_indices: Vec<MarketIndexInfo>,
    pub currency: Vec<CurrencyInfo>,
    pub commodities: Vec<CommodityInfo>,
    pub a_hun_effect: String,  // AH股联动分析
    pub us_market_impact: String, // 美股对A股影响
    pub recommendation: String,
}

/// 市场指数信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketIndexInfo {
    pub name: String,
    pub symbol: String,
    pub price: f64,
    pub change_pct: f64,
}

/// 汇率信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyInfo {
    pub pair: String,
    pub price: f64,
    pub change_pct: f64,
}

/// 商品信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommodityInfo {
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
}

/// 盘后监控服务
pub struct AfterHoursMonitor {
    perception: PerceptionModule,
    alerts: Vec<Alert>,
    last_us_close: Option<ExternalMarketAnalysis>,
}

impl AfterHoursMonitor {
    pub fn new() -> Self {
        Self {
            perception: PerceptionModule::new(),
            alerts: vec![],
            last_us_close: None,
        }
    }

    /// 执行所有监控任务
    pub async fn run_all_monitors(&mut self) -> Vec<Alert> {
        self.alerts.clear();

        // 并行执行多个监控任务
        let alerts = futures::join!(
            self.monitor_us_market(),
            self.monitor_asian_market(),
            self.monitor_news(),
            self.monitor_external_markets(),
        );

        // 合并所有预警
        let mut all_alerts = alerts.0;
        all_alerts.extend(alerts.1);
        all_alerts.extend(alerts.2);
        all_alerts.extend(alerts.3);

        self.alerts = all_alerts.clone();
        all_alerts
    }

    /// 美股市场监控 (21:00 - 23:30 北京时间)
    pub async fn monitor_us_market(&self) -> Vec<Alert> {
        let mut alerts = vec![];

        // 检测当前是否在美股交易时段
        let now = Local::now();
        let time = now.time();
        let us_start = NaiveTime::from_hms_opt(21, 30, 0).unwrap();
        let us_end = NaiveTime::from_hms_opt(23, 30, 0).unwrap();

        if time >= us_start && time <= us_end {
            let external = self.perception.scan_external().await;

            // 检测美股大涨大跌
            for index in &external.us_futures {
                if index.change_pct > 2.0 {
                    alerts.push(Alert {
                        id: uuid::Uuid::new_v4().to_string(),
                        alert_type: AlertType::ExternalMarket,
                        symbol: Some(index.symbol.clone()),
                        title: format!("美股期货大涨 {}%", index.change_pct),
                        content: format!("{} 期货上涨 {:.2}%，可能影响A股明日走势", index.name, index.change_pct),
                        severity: AlertSeverity::High,
                        timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                        action_suggested: "关注A股开盘情况，考虑调整仓位".to_string(),
                    });
                } else if index.change_pct < -2.0 {
                    alerts.push(Alert {
                        id: uuid::Uuid::new_v4().to_string(),
                        alert_type: AlertType::ExternalMarket,
                        symbol: Some(index.symbol.clone()),
                        title: format!("美股期货大跌 {}%", index.change_pct),
                        content: format!("{} 期货下跌 {:.2}%，注意防范风险", index.name, index.change_pct),
                        severity: AlertSeverity::Critical,
                        timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                        action_suggested: "建议减仓或空仓，等待市场稳定".to_string(),
                    });
                }
            }
        }

        alerts
    }

    /// 亚太市场监控 (次日 8:00)
    pub async fn monitor_asian_market(&self) -> Vec<Alert> {
        let mut alerts = vec![];

        let now = Local::now();
        let time = now.time();
        let asia_start = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let asia_end = NaiveTime::from_hms_opt(9, 0, 0).unwrap();

        if time >= asia_start && time <= asia_end {
            let external = self.perception.scan_external().await;

            for index in &external.asian_indices {
                if index.change_pct > 1.5 {
                    alerts.push(Alert {
                        id: uuid::Uuid::new_v4().to_string(),
                        alert_type: AlertType::ExternalMarket,
                        symbol: Some(index.symbol.clone()),
                        title: format!("亚太市场上涨 {}%", index.change_pct),
                        content: format!("{} 上涨 {:.2}%，亚太市场情绪偏暖", index.name, index.change_pct),
                        severity: AlertSeverity::Medium,
                        timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                        action_suggested: "可适当加仓，关注A股开盘".to_string(),
                    });
                }
            }
        }

        alerts
    }

    /// 新闻追踪监控
    pub async fn monitor_news(&self) -> Vec<Alert> {
        let mut alerts = vec![];
        let news = self.perception.scan_news().await;
        let now = Local::now();

        for n in news.iter().take(10) {
            // 检测重大利空新闻
            let text = format!("{} {}", n.title, n.content).to_lowercase();
            let risk_keywords = ["调查", "处罚", "减持", "业绩预亏", "暴雷", "造假", "退市"];

            for keyword in &risk_keywords {
                if text.contains(keyword) {
                    alerts.push(Alert {
                        id: uuid::Uuid::new_v4().to_string(),
                        alert_type: AlertType::NewsRisk,
                        symbol: n.related_stocks.first().cloned(),
                        title: format!("风险预警: {}", n.title),
                        content: format!("检测到关键词「{}」，可能对相关股票造成影响", keyword),
                        severity: AlertSeverity::High,
                        timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                        action_suggested: "建议检查相关持仓，考虑规避风险".to_string(),
                    });
                    break;
                }
            }

            // 检测重大利好
            let positive_keywords = ["业绩预增", "订单大增", "合作", "回购", "增持", "特大利好"];
            for keyword in &positive_keywords {
                if text.contains(keyword) {
                    alerts.push(Alert {
                        id: uuid::Uuid::new_v4().to_string(),
                        alert_type: AlertType::NewsRisk,
                        symbol: n.related_stocks.first().cloned(),
                        title: format!("利好关注: {}", n.title),
                        content: format!("检测到关键词「{}」，可能存在投资机会", keyword),
                        severity: AlertSeverity::Low,
                        timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                        action_suggested: "可关注相关标的，等待买入时机".to_string(),
                    });
                    break;
                }
            }
        }

        alerts
    }

    /// 外盘综合扫描
    pub async fn monitor_external_markets(&self) -> Vec<Alert> {
        let mut alerts = vec![];
        let external = self.perception.scan_external().await;
        let now = Local::now();

        // 黄金大涨大跌预警
        for commodity in &external.commodities {
            let name = commodity.name.to_lowercase();
            if name.contains("黄金") || name.contains("原油") {
                if commodity.change_pct > 2.0 {
                    alerts.push(Alert {
                        id: uuid::Uuid::new_v4().to_string(),
                        alert_type: AlertType::ExternalMarket,
                        symbol: None,
                        title: format!("{}大涨 {:.2}%", commodity.name, commodity.change_pct),
                        content: format!("大宗商品价格异动，可能影响相关板块",),
                        severity: AlertSeverity::Medium,
                        timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                        action_suggested: "关注黄金/能源板块走势".to_string(),
                    });
                }
            }
        }

        // 汇率预警
        for currency in &external.currency {
            if currency.change_pct.abs() > 0.5 {
                alerts.push(Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    alert_type: AlertType::ExternalMarket,
                    symbol: None,
                    title: format!("汇率波动: {} {}%", currency.pair, currency.change_pct),
                    content: format!("{} 波动较大，可能影响外资流向", currency.pair),
                    severity: AlertSeverity::Medium,
                    timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                    action_suggested: "关注外资重仓股".to_string(),
                });
            }
        }

        alerts
    }

    /// 生成外盘分析报告
    pub async fn generate_external_analysis(&self) -> ExternalMarketAnalysis {
        let external = self.perception.scan_external().await;

        // AH股联动分析
        let a_hun_effect = if external.asian_indices.iter().any(|i| i.change_pct > 1.0) {
            "亚太市场整体偏暖，AH股有望联动上涨".to_string()
        } else if external.asian_indices.iter().any(|i| i.change_pct < -1.0) {
            "亚太市场整体偏弱，注意防范跟跌风险".to_string()
        } else {
            "亚太市场整体平稳，AH股联动效应较弱".to_string()
        };

        // 美股对A股影响
        let us_avg: f64 = external.us_futures.iter().map(|i| i.change_pct).sum::<f64>()
            / external.us_futures.len().max(1) as f64;
        let us_market_impact = if us_avg > 1.0 {
            "美股期货大涨，预计明日A股高开，关注金融、消费板块".to_string()
        } else if us_avg < -1.0 {
            "美股期货大跌，预计明日A股低开，建议控制仓位".to_string()
        } else {
            "美股期货平稳，A股预计维持震荡格局".to_string()
        };

        // 投资建议
        let recommendation = format!(
            "当前外盘环境:{}，建议{}。{}",
            if us_avg > 0.0 { "偏暖" } else if us_avg < 0.0 { "偏冷" } else { "中性" },
            if us_avg > 1.0 { "适度加仓" } else if us_avg < -1.0 { "减仓观望" } else { "维持仓位" },
            a_hun_effect
        );

        ExternalMarketAnalysis {
            us_indices: external.us_futures.into_iter().map(|i| MarketIndexInfo {
                name: i.name,
                symbol: i.symbol,
                price: i.price,
                change_pct: i.change_pct,
            }).collect(),
            asian_indices: external.asian_indices.into_iter().map(|i| MarketIndexInfo {
                name: i.name,
                symbol: i.symbol,
                price: i.price,
                change_pct: i.change_pct,
            }).collect(),
            currency: external.currency.into_iter().map(|c| CurrencyInfo {
                pair: c.pair,
                price: c.price,
                change_pct: c.change_pct,
            }).collect(),
            commodities: external.commodities.into_iter().map(|c| CommodityInfo {
                name: c.name,
                price: c.price,
                change_pct: c.change_pct,
            }).collect(),
            a_hun_effect,
            us_market_impact,
            recommendation,
        }
    }

    /// 获取最近预警
    pub fn get_recent_alerts(&self, count: usize) -> Vec<&Alert> {
        self.alerts.iter().rev().take(count).collect()
    }

    /// 获取所有预警
    pub fn get_all_alerts(&self) -> &Vec<Alert> {
        &self.alerts
    }

    /// 清除已处理的预警
    pub fn clear_processed_alerts(&mut self) {
        self.alerts.retain(|a| a.severity == AlertSeverity::Critical);
    }
}

impl Default for AfterHoursMonitor {
    fn default() -> Self {
        Self::new()
    }
}
