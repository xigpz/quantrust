use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use crate::models::*;
use chrono::Utc;

/// 东方财富数据源
pub struct EastMoneyApi {
    client: Client,
}

impl EastMoneyApi {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .build()
                .unwrap(),
        }
    }

    /// 获取A股实时行情列表 (全市场)
    pub async fn get_realtime_quotes(&self, page: u32, page_size: u32) -> Result<Vec<StockQuote>> {
        let url = format!(
            "https://82.push2.eastmoney.com/api/qt/clist/get?\
            cb=&pn={}&pz={}&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281\
            &fltt=2&invt=2&wbp2u=|0|0|0|web&fid=f3&fs=m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23,m:0+t:81+s:2048\
            &fields=f1,f2,f3,f4,f5,f6,f7,f8,f9,f10,f12,f13,f14,f15,f16,f17,f18,f20,f21,f23,f24,f25,f22,f33,f11,f62,f128,f136,f115,f152,f31,f32,f34,f35,f38,f39,f40,f41",
            page, page_size
        );

        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let mut quotes = Vec::new();

        if let Some(data) = resp.get("data") {
            if let Some(diff) = data.get("diff").and_then(|d| d.as_array()) {
                for item in diff {
                    let symbol_code = item["f12"].as_str().unwrap_or("");
                    let market = item["f13"].as_i64().unwrap_or(0);
                    let market_str = if market == 1 { "SH" } else { "SZ" };
                    let symbol = format!("{}.{}", symbol_code, market_str);

                    let quote = StockQuote {
                        symbol,
                        name: item["f14"].as_str().unwrap_or("").to_string(),
                        price: item["f2"].as_f64().unwrap_or(0.0),
                        change: item["f4"].as_f64().unwrap_or(0.0),
                        change_pct: item["f3"].as_f64().unwrap_or(0.0),
                        open: item["f17"].as_f64().unwrap_or(0.0),
                        high: item["f15"].as_f64().unwrap_or(0.0),
                        low: item["f16"].as_f64().unwrap_or(0.0),
                        pre_close: item["f18"].as_f64().unwrap_or(0.0),
                        volume: item["f5"].as_f64().unwrap_or(0.0),
                        turnover: item["f6"].as_f64().unwrap_or(0.0),
                        turnover_rate: item["f8"].as_f64().unwrap_or(0.0),
                        amplitude: item["f7"].as_f64().unwrap_or(0.0),
                        pe_ratio: item["f9"].as_f64().unwrap_or(0.0),
                        total_market_cap: item["f20"].as_f64().unwrap_or(0.0),
                        circulating_market_cap: item["f21"].as_f64().unwrap_or(0.0),
                        timestamp: Utc::now(),
                        bid_prices: vec![],
                        bid_volumes: vec![],
                        ask_prices: vec![],
                        ask_volumes: vec![],
                    };
                    if quote.price > 0.0 {
                        quotes.push(quote);
                    }
                }
            }
        }

        Ok(quotes)
    }

    /// 获取单只股票详细行情(含五档盘口)
    pub async fn get_stock_detail(&self, symbol: &str) -> Result<StockQuote> {
        let (market, code) = parse_symbol(symbol);
        let secid = format!("{}.{}", market, code);
        
        let url = format!(
            "https://push2.eastmoney.com/api/qt/stock/get?\
            ut=fa5fd1943c7b386f172d6893dbbd1d0c&invt=2&fltt=2\
            &fields=f43,f44,f45,f46,f47,f48,f50,f51,f52,f55,f57,f58,f60,f116,f117,f162,f168,f169,f170,f171\
            ,f31,f32,f33,f34,f35,f36,f37,f38,f39,f40\
            &secid={}",
            secid
        );

        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let data = resp.get("data").ok_or_else(|| anyhow::anyhow!("No data"))?;

        let quote = StockQuote {
            symbol: symbol.to_string(),
            name: data["f58"].as_str().unwrap_or("").to_string(),
            price: data["f43"].as_f64().unwrap_or(0.0) / 100.0,
            change: data["f169"].as_f64().unwrap_or(0.0) / 100.0,
            change_pct: data["f170"].as_f64().unwrap_or(0.0) / 100.0,
            open: data["f46"].as_f64().unwrap_or(0.0) / 100.0,
            high: data["f44"].as_f64().unwrap_or(0.0) / 100.0,
            low: data["f45"].as_f64().unwrap_or(0.0) / 100.0,
            pre_close: data["f60"].as_f64().unwrap_or(0.0) / 100.0,
            volume: data["f47"].as_f64().unwrap_or(0.0),
            turnover: data["f48"].as_f64().unwrap_or(0.0),
            turnover_rate: data["f168"].as_f64().unwrap_or(0.0) / 100.0,
            amplitude: data["f171"].as_f64().unwrap_or(0.0) / 100.0,
            pe_ratio: data["f162"].as_f64().unwrap_or(0.0) / 100.0,
            total_market_cap: data["f116"].as_f64().unwrap_or(0.0),
            circulating_market_cap: data["f117"].as_f64().unwrap_or(0.0),
            timestamp: Utc::now(),
            bid_prices: vec![
                data["f31"].as_f64().unwrap_or(0.0) / 100.0,
                data["f33"].as_f64().unwrap_or(0.0) / 100.0,
                data["f35"].as_f64().unwrap_or(0.0) / 100.0,
                data["f37"].as_f64().unwrap_or(0.0) / 100.0,
                data["f39"].as_f64().unwrap_or(0.0) / 100.0,
            ],
            bid_volumes: vec![
                data["f32"].as_f64().unwrap_or(0.0),
                data["f34"].as_f64().unwrap_or(0.0),
                data["f36"].as_f64().unwrap_or(0.0),
                data["f38"].as_f64().unwrap_or(0.0),
                data["f40"].as_f64().unwrap_or(0.0),
            ],
            ask_prices: vec![],
            ask_volumes: vec![],
        };

        Ok(quote)
    }

    /// 获取K线数据
    pub async fn get_candles(&self, symbol: &str, period: &str, count: u32) -> Result<Vec<Candle>> {
        let (market, code) = parse_symbol(symbol);
        let secid = format!("{}.{}", market, code);
        
        // period mapping: 1d -> 101, 1w -> 102, 1M -> 103, 5m -> 5, 15m -> 15, 30m -> 30, 60m -> 60
        let klt = match period {
            "1d" | "daily" => "101",
            "1w" | "weekly" => "102",
            "1M" | "monthly" => "103",
            "5m" => "5",
            "15m" => "15",
            "30m" => "30",
            "60m" => "60",
            _ => "101",
        };

        let url = format!(
            "https://push2his.eastmoney.com/api/qt/stock/kline/get?\
            cb=&secid={}&ut=fa5fd1943c7b386f172d6893dbbd1d0c\
            &fields1=f1,f2,f3,f4,f5,f6\
            &fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61\
            &klt={}&fqt=1&end=20500101&lmt={}",
            secid, klt, count
        );

        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let mut candles = Vec::new();

        if let Some(data) = resp.get("data") {
            if let Some(klines) = data.get("klines").and_then(|k| k.as_array()) {
                for kline in klines {
                    if let Some(line) = kline.as_str() {
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() >= 7 {
                            candles.push(Candle {
                                symbol: symbol.to_string(),
                                timestamp: parts[0].to_string(),
                                open: parts[1].parse().unwrap_or(0.0),
                                close: parts[2].parse().unwrap_or(0.0),
                                high: parts[3].parse().unwrap_or(0.0),
                                low: parts[4].parse().unwrap_or(0.0),
                                volume: parts[5].parse().unwrap_or(0.0),
                                turnover: parts[6].parse().unwrap_or(0.0),
                            });
                        }
                    }
                }
            }
        }

        Ok(candles)
    }

    /// 获取板块行情
    pub async fn get_sectors(&self) -> Result<Vec<SectorInfo>> {
        let url = "https://82.push2.eastmoney.com/api/qt/clist/get?\
            cb=&pn=1&pz=50&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281\
            &fltt=2&invt=2&fid=f3&fs=m:90+t:2+f:!50\
            &fields=f1,f2,f3,f4,f8,f12,f13,f14,f15,f16,f17,f18,f20,f21,f24,f25,f104,f105,f128,f136,f140,f141";

        let resp: Value = self.client.get(url).send().await?.json().await?;
        let mut sectors = Vec::new();

        if let Some(data) = resp.get("data") {
            if let Some(diff) = data.get("diff").and_then(|d| d.as_array()) {
                for item in diff {
                    sectors.push(SectorInfo {
                        name: item["f14"].as_str().unwrap_or("").to_string(),
                        code: item["f12"].as_str().unwrap_or("").to_string(),
                        change_pct: item["f3"].as_f64().unwrap_or(0.0),
                        turnover: item["f20"].as_f64().unwrap_or(0.0),
                        leading_stock: item["f140"].as_str().unwrap_or("").to_string(),
                        leading_stock_pct: item["f136"].as_f64().unwrap_or(0.0),
                        stock_count: item["f104"].as_i64().unwrap_or(0) as i32 + item["f105"].as_i64().unwrap_or(0) as i32,
                        up_count: item["f104"].as_i64().unwrap_or(0) as i32,
                        down_count: item["f105"].as_i64().unwrap_or(0) as i32,
                    });
                }
            }
        }

        Ok(sectors)
    }

    /// 获取涨停板数据
    pub async fn get_limit_up_stocks(&self) -> Result<Vec<StockQuote>> {
        let url = "https://82.push2.eastmoney.com/api/qt/clist/get?\
            cb=&pn=1&pz=200&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281\
            &fltt=2&invt=2&fid=f3&fs=m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23,m:0+t:81+s:2048\
            &fields=f1,f2,f3,f4,f5,f6,f7,f8,f9,f12,f13,f14,f15,f16,f17,f18,f20,f21";

        let resp: Value = self.client.get(url).send().await?.json().await?;
        let mut stocks = Vec::new();

        if let Some(data) = resp.get("data") {
            if let Some(diff) = data.get("diff").and_then(|d| d.as_array()) {
                for item in diff {
                    let change_pct = item["f3"].as_f64().unwrap_or(0.0);
                    // 涨幅接近10%或20%（科创板/创业板）视为涨停
                    if change_pct >= 9.9 {
                        let symbol_code = item["f12"].as_str().unwrap_or("");
                        let market = item["f13"].as_i64().unwrap_or(0);
                        let market_str = if market == 1 { "SH" } else { "SZ" };
                        
                        stocks.push(StockQuote {
                            symbol: format!("{}.{}", symbol_code, market_str),
                            name: item["f14"].as_str().unwrap_or("").to_string(),
                            price: item["f2"].as_f64().unwrap_or(0.0),
                            change: item["f4"].as_f64().unwrap_or(0.0),
                            change_pct,
                            open: item["f17"].as_f64().unwrap_or(0.0),
                            high: item["f15"].as_f64().unwrap_or(0.0),
                            low: item["f16"].as_f64().unwrap_or(0.0),
                            pre_close: item["f18"].as_f64().unwrap_or(0.0),
                            volume: item["f5"].as_f64().unwrap_or(0.0),
                            turnover: item["f6"].as_f64().unwrap_or(0.0),
                            turnover_rate: item["f8"].as_f64().unwrap_or(0.0),
                            amplitude: item["f7"].as_f64().unwrap_or(0.0),
                            pe_ratio: item["f9"].as_f64().unwrap_or(0.0),
                            total_market_cap: item["f20"].as_f64().unwrap_or(0.0),
                            circulating_market_cap: item["f21"].as_f64().unwrap_or(0.0),
                            timestamp: Utc::now(),
                            bid_prices: vec![],
                            bid_volumes: vec![],
                            ask_prices: vec![],
                            ask_volumes: vec![],
                        });
                    }
                }
            }
        }

        Ok(stocks)
    }

    /// 获取资金流向
    pub async fn get_money_flow(&self, page_size: u32) -> Result<Vec<MoneyFlow>> {
        let url = format!(
            "https://push2.eastmoney.com/api/qt/clist/get?\
            cb=&pn=1&pz={}&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281\
            &fltt=2&invt=2&fid=f62&fs=m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23,m:0+t:81+s:2048\
            &fields=f12,f13,f14,f62,f184,f66,f69,f72,f75,f78,f81,f84,f87",
            page_size
        );

        let resp: Value = self.client.get(&url).send().await?.json().await?;
        let mut flows = Vec::new();

        if let Some(data) = resp.get("data") {
            if let Some(diff) = data.get("diff").and_then(|d| d.as_array()) {
                for item in diff {
                    let symbol_code = item["f12"].as_str().unwrap_or("");
                    let market = item["f13"].as_i64().unwrap_or(0);
                    let market_str = if market == 1 { "SH" } else { "SZ" };

                    flows.push(MoneyFlow {
                        symbol: format!("{}.{}", symbol_code, market_str),
                        name: item["f14"].as_str().unwrap_or("").to_string(),
                        main_net_inflow: item["f62"].as_f64().unwrap_or(0.0),
                        super_large_inflow: item["f66"].as_f64().unwrap_or(0.0),
                        large_inflow: item["f72"].as_f64().unwrap_or(0.0),
                        medium_inflow: item["f78"].as_f64().unwrap_or(0.0),
                        small_inflow: item["f84"].as_f64().unwrap_or(0.0),
                        timestamp: Utc::now(),
                    });
                }
            }
        }

        Ok(flows)
    }

    /// 获取大盘指数
    pub async fn get_market_overview(&self) -> Result<MarketOverview> {
        // 获取三大指数
        let indices = vec![
            ("1.000001", "上证指数"),
            ("0.399001", "深证成指"),
            ("0.399006", "创业板指"),
        ];

        let mut index_quotes = Vec::new();
        for (secid, name) in &indices {
            let url = format!(
                "https://push2.eastmoney.com/api/qt/stock/get?\
                ut=fa5fd1943c7b386f172d6893dbbd1d0c&invt=2&fltt=2\
                &fields=f43,f44,f45,f46,f47,f48,f57,f58,f169,f170\
                &secid={}",
                secid
            );
            let resp: Value = self.client.get(&url).send().await?.json().await?;
            if let Some(data) = resp.get("data") {
                index_quotes.push(IndexQuote {
                    name: name.to_string(),
                    code: data["f57"].as_str().unwrap_or("").to_string(),
                    price: data["f43"].as_f64().unwrap_or(0.0) / 100.0,
                    change: data["f169"].as_f64().unwrap_or(0.0) / 100.0,
                    change_pct: data["f170"].as_f64().unwrap_or(0.0) / 100.0,
                    volume: data["f47"].as_f64().unwrap_or(0.0),
                    turnover: data["f48"].as_f64().unwrap_or(0.0),
                });
            }
        }

        // 获取涨跌统计
        let stats_url = "https://82.push2.eastmoney.com/api/qt/clist/get?\
            cb=&pn=1&pz=1&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281\
            &fltt=2&invt=2&fid=f3&fs=m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23,m:0+t:81+s:2048\
            &fields=f1,f2,f3";
        let stats_resp: Value = self.client.get(stats_url).send().await?.json().await?;
        let total = stats_resp["data"]["total"].as_i64().unwrap_or(5000) as i32;

        let sh = index_quotes.get(0).cloned().unwrap_or(IndexQuote {
            name: "上证指数".into(), code: "000001".into(), price: 0.0, change: 0.0, change_pct: 0.0, volume: 0.0, turnover: 0.0,
        });
        let sz = index_quotes.get(1).cloned().unwrap_or(IndexQuote {
            name: "深证成指".into(), code: "399001".into(), price: 0.0, change: 0.0, change_pct: 0.0, volume: 0.0, turnover: 0.0,
        });
        let cyb = index_quotes.get(2).cloned().unwrap_or(IndexQuote {
            name: "创业板指".into(), code: "399006".into(), price: 0.0, change: 0.0, change_pct: 0.0, volume: 0.0, turnover: 0.0,
        });

        Ok(MarketOverview {
            sh_index: sh,
            sz_index: sz,
            cyb_index: cyb,
            total_turnover: 0.0,
            up_count: 0,
            down_count: 0,
            flat_count: 0,
            limit_up_count: 0,
            limit_down_count: 0,
            timestamp: Utc::now(),
        })
    }
}

/// 解析股票代码 -> (market_id, code)
fn parse_symbol(symbol: &str) -> (u8, &str) {
    if let Some(pos) = symbol.find('.') {
        let code = &symbol[..pos];
        let market = &symbol[pos + 1..];
        match market {
            "SH" => (1, code),
            "SZ" => (0, code),
            _ => (1, code),
        }
    } else {
        // 根据代码前缀判断市场
        if symbol.starts_with("6") || symbol.starts_with("9") {
            (1, symbol)
        } else {
            (0, symbol)
        }
    }
}
