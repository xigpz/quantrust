use crate::data::DataProvider;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 大单监控配置
#[derive(Debug, Clone)]
pub struct BigOrderConfig {
    pub huge_threshold: f64,   // 超大单阈值(万元), 默认500万
    pub large_threshold: f64,  // 大单阈值(万元), 默认200万
    pub min_change_pct: f64,  // 最小涨跌幅(%), 默认0%
}

impl Default for BigOrderConfig {
    fn default() -> Self {
        Self {
            huge_threshold: 500.0,
            large_threshold: 200.0,
            min_change_pct: 0.0,
        }
    }
}

/// 大单抢筹股票信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigOrderStock {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
    pub huge_inflow: f64,      // 超大单净流入(万)
    pub large_inflow: f64,     // 大单净流入(万)
    pub total_main_inflow: f64, // 主力净流入(万)
    pub inflow_ratio: f64,     // 流入占成交额比
    pub reason: String,        // 抢筹原因
}

/// 大单预警
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigOrderAlert {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub alert_type: String,    // "huge_inflow", "large_inflow_surge", "sector_focus"
    pub threshold: f64,
    pub actual: f64,
    pub message: String,
    pub timestamp: String,
    pub acknowledged: bool,
}

/// 板块大单统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorBigOrder {
    pub sector_code: String,
    pub sector_name: String,
    pub stock_count: i32,
    pub huge_inflow_total: f64,   // 超大单合计(万)
    pub large_inflow_total: f64,   // 大单合计(万)
    pub main_inflow_total: f64,    // 主力合计(万)
    pub avg_inflow_per_stock: f64, // 股均流入(万)
    pub hot_stocks: Vec<String>,   // 热门股代码
}

/// 大单监控服务
pub struct BigOrderMonitor {
    provider: Arc<DataProvider>,
    config: BigOrderConfig,
    alerts: RwLock<Vec<BigOrderAlert>>,
    history: RwLock<HashMap<String, f64>>, // 记录上次大单流入用于检测突增
}

impl BigOrderMonitor {
    pub fn new(provider: Arc<DataProvider>) -> Arc<Self> {
        Arc::new(Self {
            provider,
            config: BigOrderConfig::default(),
            alerts: RwLock::new(Vec::new()),
            history: RwLock::new(HashMap::new()),
        })
    }

    pub fn with_config(provider: Arc<DataProvider>, config: BigOrderConfig) -> Arc<Self> {
        Arc::new(Self {
            provider,
            config,
            alerts: RwLock::new(Vec::new()),
            history: RwLock::new(HashMap::new()),
        })
    }

    /// 扫描大单抢筹股票 - 获取超大单净流入排名靠前的股票
    pub async fn scan_big_order_stocks(&self) -> Result<Vec<BigOrderStock>> {
        // 获取资金流向排名
        let flows = self.provider.get_money_flow(100).await?;

        let mut big_order_stocks = Vec::new();

        for flow in flows {
            let total_inflow = flow.super_large_inflow + flow.large_inflow;

            // 过滤：大单净流入超过阈值
            if total_inflow < self.config.large_threshold {
                continue;
            }

            // 获取股票详情
            let stock_result = self.provider.get_stock_detail(&flow.symbol).await;
            if let Ok(stock) = stock_result {
                // 过滤：涨跌幅要求
                if stock.change_pct.abs() < self.config.min_change_pct {
                    continue;
                }

                // 计算流入占成交额比
                let inflow_ratio = if stock.turnover > 0.0 {
                    (total_inflow * 10000.0) / (stock.turnover * 10000.0) * 100.0
                } else {
                    0.0
                };

                // 判断抢筹原因
                let reason = self.analyze_inflow_reason(
                    flow.super_large_inflow,
                    flow.large_inflow,
                    stock.change_pct,
                );

                big_order_stocks.push(BigOrderStock {
                    symbol: flow.symbol.clone(),
                    name: stock.name,
                    price: stock.price,
                    change_pct: stock.change_pct,
                    huge_inflow: flow.super_large_inflow,
                    large_inflow: flow.large_inflow,
                    total_main_inflow: flow.main_net_inflow,
                    inflow_ratio,
                    reason,
                });
            }
        }

        // 按超大单流入排序
        big_order_stocks.sort_by(|a, b| {
            b.huge_inflow.partial_cmp(&a.huge_inflow).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(big_order_stocks)
    }

    /// 分析流入原因
    fn analyze_inflow_reason(&self, huge: f64, large: f64, change_pct: f64) -> String {
        if huge > self.config.huge_threshold && large > self.config.large_threshold {
            if change_pct > 5.0 {
                "超大单+大单联合拉升".to_string()
            } else if change_pct > 0.0 {
                "超大单主导吸筹".to_string()
            } else {
                "超大单逆势买入".to_string()
            }
        } else if huge > self.config.huge_threshold {
            "超大单净流入突出".to_string()
        } else if large > self.config.large_threshold {
            "大单持续买入".to_string()
        } else {
            "资金温和流入".to_string()
        }
    }

    /// 检测大单异动 - 检测流入突然放大的股票
    pub async fn detect_big_order_surge(&self) -> Result<Vec<BigOrderAlert>> {
        let flows = self.provider.get_money_flow(100).await?;
        let mut new_alerts = Vec::new();
        let mut history = self.history.write().await;

        for flow in flows {
            let symbol = flow.symbol.clone();

            // 检测超大单异动
            if flow.super_large_inflow > self.config.huge_threshold {
                let prev = history.get(&format!("huge_{}", symbol)).copied().unwrap_or(0.0);
                let surge = if prev > 0.0 {
                    (flow.super_large_inflow - prev) / prev * 100.0
                } else {
                    100.0
                };

                if surge > 50.0 || prev == 0.0 {
                    let name = self.provider.get_stock_detail(&symbol)
                        .await
                        .map(|s| s.name)
                        .unwrap_or_else(|_| symbol.clone());

                    new_alerts.push(BigOrderAlert {
                        id: uuid::Uuid::new_v4().to_string(),
                        symbol: symbol.clone(),
                        name,
                        alert_type: "huge_inflow_surge".to_string(),
                        threshold: self.config.huge_threshold,
                        actual: flow.super_large_inflow,
                        message: format!("超大单流入突增{:.0}%, 当前{:.0}万", surge, flow.super_large_inflow),
                        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        acknowledged: false,
                    });
                }
            }

            // 记录历史
            history.insert(format!("huge_{}", symbol), flow.super_large_inflow);
            history.insert(format!("large_{}", symbol), flow.large_inflow);
        }

        // 保存警报
        if !new_alerts.is_empty() {
            let mut alerts = self.alerts.write().await;
            alerts.extend(new_alerts.clone());
            // 保留最近100条
            if alerts.len() > 100 {
                let drain_count = alerts.len() - 100;
                alerts.drain(0..drain_count);
            }
        }

        Ok(new_alerts)
    }

    /// 获取板块大单排名
    /// 注意：由于个股资金流向数据不包含所属板块信息，
    /// 目前基于板块资金流向数据返回排名，大单股票匹配仅供参考
    pub async fn get_sector_big_order_ranking(&self) -> Result<Vec<SectorBigOrder>> {
        let sector_flows = self.provider.get_sector_money_flow().await?;
        let flows = self.provider.get_money_flow(100).await?;

        // 按板块分组计算大单流入
        let mut sector_stats: HashMap<String, SectorBigOrder> = HashMap::new();

        // 初始化所有板块
        // 注意：EastMoney API返回的 main_net_inflow 单位是万元，不需要转换
        for sf in &sector_flows {
            sector_stats.insert(sf.name.clone(), SectorBigOrder {
                sector_code: sf.code.clone(),
                sector_name: sf.name.clone(),
                stock_count: 0,
                huge_inflow_total: 0.0,
                large_inflow_total: 0.0,
                main_inflow_total: sf.main_net_inflow, // 万元
                avg_inflow_per_stock: sf.main_net_inflow,
                hot_stocks: Vec::new(),
            });
        }

        // 获取有大单/超大单流入的股票
        for flow in flows.iter() {
            let total = flow.super_large_inflow + flow.large_inflow;
            if total < self.config.large_threshold {
                continue;
            }

            // 尝试通过股票名称关键词匹配板块
            let stock_name = &flow.name;
            let mut best_match: Option<&str> = None;
            let mut best_sector: Option<&crate::models::SectorMoneyFlow> = None;

            // 关键词映射：股票名称 -> 板块名称
            let keywords: [(&str, [&str; 4]); 12] = [
                ("新能源", ["宁德", "比亚迪", "亿纬", "赣锋"]),
                ("汽车整车", ["比亚迪", "长城", "长安", "上汽"]),
                ("医药生物", ["恒瑞", "药明", "迈瑞", "片仔癀"]),
                ("白酒", ["茅台", "五粮液", "泸州", "洋河"]),
                ("半导体", ["中芯", "华虹", "韦尔", "兆易"]),
                ("军工", ["中航", "航发", "兵器", "船舶"]),
                ("光伏设备", ["隆基", "通威", "天合", "晶澳"]),
                ("电池", ["宁德", "亿纬", "欣旺达", "国轩"]),
                ("有色金属", ["紫金", "洛阳钼业", "赣锋", "华友"]),
                ("银行", ["工商", "建设", "农业", "中国"]),
                ("证券", ["中信", "华泰", "国泰", "海通"]),
                ("房地产", ["万科", "保利", "招商", "金地"]),
            ];

            for (sector_name, stock_keywords) in keywords {
                for kw in stock_keywords {
                    if stock_name.contains(kw) {
                        // 找到匹配的板块
                        if let Some(sf) = sector_flows.iter().find(|s| s.name.contains(sector_name)) {
                            best_match = Some(sf.name.as_str());
                            best_sector = Some(sf);
                            break;
                        }
                    }
                }
                if best_match.is_some() {
                    break;
                }
            }

            // 如果匹配到板块，加入该板块的统计
            if let Some(sf) = best_sector {
                if let Some(entry) = sector_stats.get_mut(sf.name.as_str()) {
                    entry.stock_count += 1;
                    entry.huge_inflow_total += flow.super_large_inflow;
                    entry.large_inflow_total += flow.large_inflow;
                    entry.main_inflow_total += flow.main_net_inflow;
                    if entry.hot_stocks.len() < 5 {
                        // 使用股票名称，更直观
                        let display_name = if flow.name.is_empty() {
                            flow.symbol.clone()
                        } else {
                            flow.name.clone()
                        };
                        entry.hot_stocks.push(display_name);
                    }
                }
            }
        }

        // 重新计算股均流入
        for entry in sector_stats.values_mut() {
            if entry.stock_count > 0 {
                entry.avg_inflow_per_stock = entry.main_inflow_total / entry.stock_count as f64;
            }
        }

        let mut result: Vec<SectorBigOrder> = sector_stats.into_values().collect();
        result.sort_by(|a, b| {
            b.main_inflow_total.partial_cmp(&a.main_inflow_total)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(result)
    }

    /// 获取所有未确认的警报
    pub async fn get_pending_alerts(&self) -> Vec<BigOrderAlert> {
        let alerts = self.alerts.read().await;
        alerts.iter()
            .filter(|a| !a.acknowledged)
            .cloned()
            .collect()
    }

    /// 确认警报
    pub async fn acknowledge_alert(&self, alert_id: &str) -> bool {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            return true;
        }
        false
    }

    /// 清空已确认的警报
    pub async fn clear_acknowledged(&self) {
        let mut alerts = self.alerts.write().await;
        alerts.retain(|a| !a.acknowledged);
    }

    /// 获取大单监控配置
    pub fn get_config(&self) -> BigOrderConfig {
        self.config.clone()
    }

    /// 更新配置
    pub fn update_config(&mut self, config: BigOrderConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = BigOrderConfig::default();
        assert_eq!(config.huge_threshold, 500.0);
        assert_eq!(config.large_threshold, 200.0);
    }
}
