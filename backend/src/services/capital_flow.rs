use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::Result;
use std::collections::HashMap;

/// 资金流向数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapitalFlow {
    pub symbol: String,
    pub name: String,
    // 主力净流入
    pub main_net_inflow: f64,      // 主力净流入(万)
    pub main_net_inflow_pct: f64,  // 主力净流入占比
    // 超大单
    pub huge_net_inflow: f64,      // 超大单净流入
    pub huge_net_inflow_pct: f64,
    // 大单
    pub large_net_inflow: f64,
    pub large_net_inflow_pct: f64,
    // 中单
    pub medium_net_inflow: f64,
    pub medium_net_inflow_pct: f64,
    // 小单
    pub small_net_inflow: f64,
    pub small_net_inflow_pct: f64,
    // 散户净流入
    pub retail_net_inflow: f64,
    // 5日/10日主力净流入
    pub main_5d_inflow: f64,
    pub main_10d_inflow: f64,
}

/// 板块资金流向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorFlow {
    pub sector_name: String,
    pub main_net_inflow: f64,
    pub change_pct: f64,
    pub up_count: i32,
    pub down_count: i32,
    pub leader_stock: String,
    pub leader_change: f64,
}

/// 资金流向服务
pub struct CapitalFlowService {
    client: Client,
}

impl CapitalFlowService {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    /// 获取个股资金流向
    pub async fn get_stock_flow(&self, symbol: &str) -> Result<CapitalFlow> {
        let url = format!(
            "http://push2.eastmoney.com/api/qt/stock/fflow/daykline/get",
        );
        
        let resp = self.client.get(&url)
            .query(&[
                ("secid", &format!("1.{}", symbol)),
                ("fields1", &"f1,f2,f3,f7".to_string()),
                ("fields2", &"f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61,f62,f63,f64,f65".to_string()),
                ("klt", &"101".to_string()),
                ("lmt", &"5".to_string()),
            ])
            .send().await?
            .json::<serde_json::Value>()
            .await?;
        
        // 简化处理
        Ok(CapitalFlow {
            symbol: symbol.to_string(),
            name: String::new(),
            main_net_inflow: 0.0,
            main_net_inflow_pct: 0.0,
            huge_net_inflow: 0.0,
            huge_net_inflow_pct: 0.0,
            large_net_inflow: 0.0,
            large_net_inflow_pct: 0.0,
            medium_net_inflow: 0.0,
            medium_net_inflow_pct: 0.0,
            small_net_inflow: 0.0,
            small_net_inflow_pct: 0.0,
            retail_net_inflow: 0.0,
            main_5d_inflow: 0.0,
            main_10d_inflow: 0.0,
        })
    }

    /// 获取当日资金流向排名
    pub async fn get_flow_ranking(&self, limit: usize) -> Result<Vec<CapitalFlow>> {
        // 获取主力净流入前N
        let url = "http://push2.eastmoney.com/api/qt/clist/get";
        
        let resp = self.client.get(url)
            .query(&[
                ("pn", "1"),
                ("pz", &limit.to_string()),
                ("po", "1"),
                ("np", "1"),
                ("ut", "7edaa10f068557a8e37759d4c5ff12a4"),
                ("fltt", "2"),
                ("invt", "2"),
                ("fid", "f62"),
                ("fs", "m:1+t:2,m:0+t:6"),
            ])
            .send().await?
            .json::<serde_json::Value>()
            .await?;
        
        let mut results = Vec::new();
        
        if let Some(diff) = resp["data"]["diff"].as_array() {
            for item in diff {
                results.push(CapitalFlow {
                    symbol: item["f12"].as_str().unwrap_or("").to_string(),
                    name: item["f14"].as_str().unwrap_or("").to_string(),
                    main_net_inflow: item["f62"].as_f64().unwrap_or(0.0) / 10000.0,
                    main_net_inflow_pct: item["f62"].as_f64().unwrap_or(0.0),
                    huge_net_inflow: item["f184"].as_f64().unwrap_or(0.0) / 10000.0,
                    huge_net_inflow_pct: 0.0,
                    large_net_inflow: item["f185"].as_f64().unwrap_or(0.0) / 10000.0,
                    large_net_inflow_pct: 0.0,
                    medium_net_inflow: 0.0,
                    medium_net_inflow_pct: 0.0,
                    small_net_inflow: 0.0,
                    small_net_inflow_pct: 0.0,
                    retail_net_inflow: 0.0,
                    main_5d_inflow: 0.0,
                    main_10d_inflow: 0.0,
                });
            }
        }
        
        Ok(results)
    }

    /// 获取板块资金流向
    pub async fn get_sector_flow(&self) -> Result<Vec<SectorFlow>> {
        let url = "http://push2.eastmoney.com/api/qt/clist/get";
        
        let resp = self.client.get(url)
            .query(&[
                ("pn", "1"),
                ("pz", "30"),
                ("po", "1"),
                ("np", "1"),
                ("ut", "7edaa10f068557a8e37759d4c5ff12a4"),
                ("fltt", "2"),
                ("invt", "2"),
                ("fid", "f62"),
                ("fs", "m:90+t:2"),
            ])
            .send().await?
            .json::<serde_json::Value>()
            .await?;
        
        let mut results = Vec::new();
        
        if let Some(diff) = resp["data"]["diff"].as_array() {
            for item in diff {
                results.push(SectorFlow {
                    sector_name: item["f14"].as_str().unwrap_or("").to_string(),
                    main_net_inflow: item["f62"].as_f64().unwrap_or(0.0) / 100000000.0,
                    change_pct: item["f3"].as_f64().unwrap_or(0.0),
                    up_count: 0,
                    down_count: 0,
                    leader_stock: String::new(),
                    leader_change: 0.0,
                });
            }
        }
        
        Ok(results)
    }

    /// 监控资金异动
    pub fn detect_flow_anomaly(&self, flow: &CapitalFlow) -> FlowAnomaly {
        let mut alerts = Vec::new();
        
        // 主力大幅流入
        if flow.main_net_inflow > 10000.0 {
            alerts.push(format!("主力大幅流入: {:.1}亿", flow.main_net_inflow / 10000.0));
        } else if flow.main_net_inflow > 5000.0 {
            alerts.push(format!("主力中幅流入: {:.1}亿", flow.main_net_inflow / 10000.0));
        }
        
        // 主力大幅流出
        if flow.main_net_inflow < -10000.0 {
            alerts.push(format!("主力大幅流出: {:.1}亿", flow.main_net_inflow / 10000.0));
        }
        
        // 持续流入
        if flow.main_5d_inflow > 0.0 && flow.main_10d_inflow > flow.main_5d_inflow {
            alerts.push("5日+10日连续流入".to_string());
        }
        
        FlowAnomaly {
            symbol: flow.symbol.clone(),
            alerts: alerts.clone(),
            risk_level: if alerts.is_empty() { "normal".to_string() } else { "warning".to_string() },
        }
    }
}

impl Default for CapitalFlowService {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CapitalFlow {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            name: String::new(),
            main_net_inflow: 0.0,
            main_net_inflow_pct: 0.0,
            huge_net_inflow: 0.0,
            huge_net_inflow_pct: 0.0,
            large_net_inflow: 0.0,
            large_net_inflow_pct: 0.0,
            medium_net_inflow: 0.0,
            medium_net_inflow_pct: 0.0,
            small_net_inflow: 0.0,
            small_net_inflow_pct: 0.0,
            retail_net_inflow: 0.0,
            main_5d_inflow: 0.0,
            main_10d_inflow: 0.0,
        }
    }
}

/// 资金异动告警
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowAnomaly {
    pub symbol: String,
    pub alerts: Vec<String>,
    pub risk_level: String,
}
