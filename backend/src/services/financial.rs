use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::Result;

/// 财务数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialData {
    pub symbol: String,
    pub name: String,
    // 估值指标
    pub pe: f64,           // 市盈率
    pub pb: f64,           // 市净率
    pub ps: f64,           // 市销率
    // 盈利能力
    pub roe: f64,          // 净资产收益率
    pub roa: f64,          // 总资产收益率
    pub gross_margin: f64, // 毛利率
    pub net_margin: f64,   // 净利润率
    // 成长性
    pub revenue_growth: f64,   // 营收增长
    pub profit_growth: f64,    // 净利润增长
    // 财务健康
    pub debt_ratio: f64,   // 资产负债率
    pub current_ratio: f64,// 流动比率
    pub quick_ratio: f64,  // 速动比率
}

/// 财务筛选条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialFilter {
    pub min_pe: Option<f64>,
    pub max_pe: Option<f64>,
    pub min_roe: Option<f64>,
    pub min_gross_margin: Option<f64>,
    pub max_debt_ratio: Option<f64>,
    pub min_profit_growth: Option<f64>,
}

impl Default for FinancialFilter {
    fn default() -> Self {
        Self {
            min_pe: Some(0.0),
            max_pe: Some(50.0),
            min_roe: Some(10.0),
            min_gross_margin: Some(20.0),
            max_debt_ratio: Some(70.0),
            min_profit_growth: Some(0.0),
        }
    }
}

/// 财务数据服务
pub struct FinancialService {
    client: Client,
}

impl FinancialService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// 获取单只股票财务数据
    pub async fn get_financial_data(&self, symbol: &str) -> Result<FinancialData> {
        // 东方财富财务API
        let url = format!(
            "http://push2.eastmoney.com/api/qt/stock/get?secid=1.{}&fields=f57,f58,f84,f85,f127,f128,f162,f163,f164,f167,f168,f116,f117,f110,f111,f112,f113,f115,f114,f119,f120,f121,f122,f123,f124,f125,f138,f139,f146,f147,f148,f149,f150,f151,f152,f153,f154,f155,f156,f157,f158,f159,f160,f161,f162,f163,f164,f165,f166,f167,f168,f169,f170,f171,f187,f188,f189,f190,f191,f192,f193,f194,f195,f197,f198,f199,f200,f201,f202,f203,f204,f205,f206,f207,f208,f209,f210,f211,f212,f213,f214,f215,f216,f217,f218,f219,f220,f221,f222,f223,f224,f225,f226,f227,f228,f229,f230,f231,f232,f233,f234,f235,f236,f237,f238,f239,f240,f241,f242,f243,f244,f245",
            symbol
        );
        
        let resp = self.client.get(&url).send().await?.json::<serde_json::Value>().await?;
        
        let data = &resp["data"];
        
        Ok(FinancialData {
            symbol: symbol.to_string(),
            name: data["f58"].as_str().unwrap_or("").to_string(),
            pe: data["f162"].as_f64().unwrap_or(0.0),
            pb: data["f167"].as_f64().unwrap_or(0.0),
            ps: data["f164"].as_f64().unwrap_or(0.0),
            roe: data["f168"].as_f64().unwrap_or(0.0),
            roa: data["f116"].as_f64().unwrap_or(0.0),
            gross_margin: data["f167"].as_f64().unwrap_or(0.0),
            net_margin: data["f170"].as_f64().unwrap_or(0.0),
            revenue_growth: data["f184"].as_f64().unwrap_or(0.0),
            profit_growth: data["f185"].as_f64().unwrap_or(0.0),
            debt_ratio: data["f173"].as_f64().unwrap_or(0.0),
            current_ratio: data["f175"].as_f64().unwrap_or(0.0),
            quick_ratio: data["f176"].as_f64().unwrap_or(0.0),
        })
    }

    /// 筛选符合财务条件的股票
    pub fn filter_stocks(&self, stocks: &[FinancialData], filter: &FinancialFilter) -> Vec<FinancialData> {
        stocks.iter().filter(|s| {
            if let Some(min) = filter.min_pe {
                if s.pe < min || s.pe <= 0.0 { return false; }
            }
            if let Some(max) = filter.max_pe {
                if s.pe > max { return false; }
            }
            if let Some(min) = filter.min_roe {
                if s.roe < min { return false; }
            }
            if let Some(min) = filter.min_gross_margin {
                if s.gross_margin < min { return false; }
            }
            if let Some(max) = filter.max_debt_ratio {
                if s.debt_ratio > max { return false; }
            }
            if let Some(min) = filter.min_profit_growth {
                if s.profit_growth < min { return false; }
            }
            true
        }).cloned().collect()
    }

    /// 评分（综合财务质量）
    pub fn score(&self, data: &FinancialData) -> f64 {
        let mut score = 0.0;
        
        // ROE 评分 (0-25)
        score += (data.roe / 30.0 * 25.0).min(25.0);
        
        // 毛利率评分 (0-20)
        score += (data.gross_margin / 50.0 * 20.0).min(20.0);
        
        // 成长性评分 (0-20)
        score += ((data.profit_growth.max(0.0) / 50.0) * 20.0).min(20.0);
        
        // 估值评分 PE倒数 (0-20)
        if data.pe > 0.0 {
            let pe_score = (30.0 / data.pe * 20.0).min(20.0);
            score += pe_score;
        }
        
        // 资产负债 (0-15) 越低越好
        if data.debt_ratio < 50.0 {
            score += 15.0;
        } else if data.debt_ratio < 70.0 {
            score += 10.0;
        } else {
            score += 5.0;
        }
        
        score
    }
}

impl Default for FinancialService {
    fn default() -> Self {
        Self::new()
    }
}

/// 龙虎榜数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragonTigerData {
    pub symbol: String,
    pub name: String,
    pub trade_date: String,
    pub close_price: f64,
    pub change_pct: f64,
    pub turnover: f64,           // 成交额(万)
    pub buy_amount: f64,         // 买入金额(万)
    pub sell_amount: f64,       // 卖出金额(万)
    pub net_amount: f64,         // 净买入(万)
    pub reason: String,         // 上榜原因
    pub buy_seats: Vec<Seat>,   // 买入营业部
    pub sell_seats: Vec<Seat>,  // 卖出营业部
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seat {
    pub name: String,
    pub amount: f64,
}

/// 龙虎榜服务
pub struct DragonTigerService {
    client: Client,
}

impl DragonTigerService {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    /// 获取当日龙虎榜
    pub async fn get_daily_list(&self) -> Result<Vec<DragonTigerData>> {
        let url = "http://push2.eastmoney.com/api/qt/clist/getpn";
        
        let resp = self.client.get(url)
            .query(&[
                ("pn", &"1".to_string()),
                ("pz", &"100".to_string()),
                ("po", &"1".to_string()),
                ("np", &"1".to_string()),
                ("ut", &"bd1d9ddb04089700cf9c27f6f7426281".to_string()),
                ("fltt", &"2".to_string()),
                ("invt", &"2".to_string()),
                ("fid", &"f3".to_string()),
                ("fs", &"m:1+t:23".to_string()),
                ("fields", &"f2,f3,f4,f12,f13,f14,f100,f101,f102,f103,f104,f105,f106,f107,f108,f109,f110,f111,f112".to_string()),
            ])
            .send().await?
            .json::<serde_json::Value>()
            .await?;
        
        let mut results = Vec::new();
        
        if let Some(diff) = resp["data"]["diff"].as_array() {
            for item in diff {
                results.push(DragonTigerData {
                    symbol: item["f12"].as_str().unwrap_or("").to_string(),
                    name: item["f14"].as_str().unwrap_or("").to_string(),
                    trade_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                    close_price: item["f2"].as_f64().unwrap_or(0.0),
                    change_pct: item["f3"].as_f64().unwrap_or(0.0),
                    turnover: 0.0,
                    buy_amount: 0.0,
                    sell_amount: 0.0,
                    net_amount: 0.0,
                    reason: "".to_string(),
                    buy_seats: vec![],
                    sell_seats: vec![],
                });
            }
        }
        
        Ok(results)
    }

    /// 获取个股龙虎榜历史
    pub async fn get_stock_history(&self, symbol: &str) -> Result<Vec<DragonTigerData>> {
        let url = format!(
            "http://push2his.eastmoney.com/api/qt/stock/lhb/getpn",
        );
        
        let resp = self.client.get(&url)
            .query(&[
                ("secid", &format!("1.{}", symbol)),
                ("pn", &"1".to_string()),
                ("pz", &"20".to_string()),
            ])
            .send().await?
            .json::<serde_json::Value>()
            .await?;
        
        let mut results = Vec::new();
        
        if let Some(data) = resp["data"]["data"].as_array() {
            for item in data {
                results.push(DragonTigerData {
                    symbol: symbol.to_string(),
                    name: item["f14"].as_str().unwrap_or("").to_string(),
                    trade_date: item["f30"].as_str().unwrap_or("").to_string(),
                    close_price: item["f2"].as_f64().unwrap_or(0.0),
                    change_pct: item["f3"].as_f64().unwrap_or(0.0),
                    turnover: item["f8"].as_f64().unwrap_or(0.0) / 10000.0,
                    buy_amount: item["f17"].as_f64().unwrap_or(0.0) / 10000.0,
                    sell_amount: item["f18"].as_f64().unwrap_or(0.0) / 10000.0,
                    net_amount: item["f21"].as_f64().unwrap_or(0.0) / 10000.0,
                    reason: item["f19"].as_str().unwrap_or("").to_string(),
                    buy_seats: vec![],
                    sell_seats: vec![],
                });
            }
        }
        
        Ok(results)
    }
}

impl Default for DragonTigerService {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FinancialData {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            name: String::new(),
            pe: 0.0,
            pb: 0.0,
            ps: 0.0,
            roe: 0.0,
            roa: 0.0,
            gross_margin: 0.0,
            net_margin: 0.0,
            revenue_growth: 0.0,
            profit_growth: 0.0,
            debt_ratio: 0.0,
            current_ratio: 0.0,
            quick_ratio: 0.0,
        }
    }
}
