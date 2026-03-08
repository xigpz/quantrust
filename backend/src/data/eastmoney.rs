use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use crate::models::*;
use chrono::Utc;

/// 东方财富数据源
/// 
/// API域名说明：
/// - push2.eastmoney.com: 实时数据（交易时段）
/// - push2delay.eastmoney.com: 延迟数据（非交易时段自动回退）
/// - 82.push2delay.eastmoney.com: 列表类数据的延迟版本
/// - push2his.eastmoney.com: 历史K线数据
///
/// 非交易时段 push2 会 302 重定向到 push2delay，
/// 因此我们直接使用 delay 版本确保全时段可用。
pub struct EastMoneyApi {
    client: Client,
}

impl EastMoneyApi {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .redirect(reqwest::redirect::Policy::limited(5))
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap(),
        }
    }

    /// 安全地发送GET请求并解析JSON
    /// 处理重定向（302 -> push2delay）和空响应
    async fn fetch_json(&self, url: &str) -> Result<Value> {
        let resp = self.client.get(url)
            .header("Referer", "https://quote.eastmoney.com/")
            .send()
            .await?;
        
        let status = resp.status();
        let text = resp.text().await?;
        
        if text.is_empty() {
            return Err(anyhow::anyhow!("Empty response from {}, status: {}", url, status));
        }
        
        // 有些API返回JSONP格式 callback(json)，需要去掉
        let json_str = if text.starts_with("jQuery") || text.starts_with("callback") {
            if let Some(start) = text.find('(') {
                if let Some(end) = text.rfind(')') {
                    &text[start + 1..end]
                } else {
                    &text
                }
            } else {
                &text
            }
        } else {
            &text
        };
        
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("JSON parse error: {} (first 200 chars: {})", e, &text[..text.len().min(200)]))?;
        
        Ok(value)
    }

    /// 获取A股实时行情列表 (全市场)
    /// 使用 82.push2delay 域名确保全时段可用
    pub async fn get_realtime_quotes(&self, page: u32, page_size: u32) -> Result<Vec<StockQuote>> {
        let url = format!(
            "https://82.push2delay.eastmoney.com/api/qt/clist/get?\
            pn={}&pz={}&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281\
            &fltt=2&invt=2&fid=f3&fs=m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23,m:0+t:81+s:2048\
            &fields=f2,f3,f4,f5,f6,f7,f8,f9,f10,f12,f13,f14,f15,f16,f17,f18,f20,f21",
            page, page_size
        );

        let resp = self.fetch_json(&url).await?;
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

        tracing::info!("Fetched {} quotes from EastMoney", quotes.len());
        Ok(quotes)
    }

    /// 获取单只股票详细行情(含五档盘口)
    /// 使用 push2delay 域名
    pub async fn get_stock_detail(&self, symbol: &str) -> Result<StockQuote> {
        let (market, code) = parse_symbol(symbol);
        let secid = format!("{}.{}", market, code);
        
        let url = format!(
            "https://push2delay.eastmoney.com/api/qt/stock/get?\
            ut=fa5fd1943c7b386f172d6893dbbd1d0c&invt=2&fltt=2\
            &fields=f43,f44,f45,f46,f47,f48,f50,f51,f52,f55,f57,f58,f60,f116,f117,f162,f168,f169,f170,f171\
            ,f31,f32,f33,f34,f35,f36,f37,f38,f39,f40\
            &secid={}",
            secid
        );

        let resp = self.fetch_json(&url).await?;
        let data = resp.get("data").ok_or_else(|| anyhow::anyhow!("No data field in response"))?;

        // 注意：个股详情API返回的价格单位是原始值（不需要除以100）
        let quote = StockQuote {
            symbol: symbol.to_string(),
            name: data["f58"].as_str().unwrap_or("").to_string(),
            price: data["f43"].as_f64().unwrap_or(0.0),
            change: data["f169"].as_f64().unwrap_or(0.0),
            change_pct: data["f170"].as_f64().unwrap_or(0.0),
            open: data["f46"].as_f64().unwrap_or(0.0),
            high: data["f44"].as_f64().unwrap_or(0.0),
            low: data["f45"].as_f64().unwrap_or(0.0),
            pre_close: data["f60"].as_f64().unwrap_or(0.0),
            volume: data["f47"].as_f64().unwrap_or(0.0),
            turnover: data["f48"].as_f64().unwrap_or(0.0),
            turnover_rate: data["f168"].as_f64().unwrap_or(0.0),
            amplitude: data["f171"].as_f64().unwrap_or(0.0),
            pe_ratio: data["f162"].as_f64().unwrap_or(0.0),
            total_market_cap: data["f116"].as_f64().unwrap_or(0.0),
            circulating_market_cap: data["f117"].as_f64().unwrap_or(0.0),
            timestamp: Utc::now(),
            bid_prices: vec![
                data["f31"].as_f64().unwrap_or(0.0),
                data["f33"].as_f64().unwrap_or(0.0),
                data["f35"].as_f64().unwrap_or(0.0),
                data["f37"].as_f64().unwrap_or(0.0),
                data["f39"].as_f64().unwrap_or(0.0),
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
    /// 使用 push2his 域名，需要 Referer 头
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

        // 分钟级别数据直接使用新浪财经
        if period == "1m" || period == "5m" || period == "15m" || period == "60m" {
            return self.get_sina_candles(symbol, &market, &code, period, count).await;
        }

        // 日周月线优先尝试东方财富，失败则回退到新浪财经
        let urls = vec![
            format!(
                "https://push2his.eastmoney.com/api/qt/stock/kline/get?\
                secid={}&ut=fa5fd1943c7b386f172d6893dbbd1d0c\
                &fields1=f1,f2,f3,f4,f5,f6\
                &fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61\
                &klt={}&fqt=1&end=20500101&lmt={}",
                secid, klt, count
            ),
            format!(
                "https://push2delay.eastmoney.com/api/qt/stock/kline/get?\
                secid={}&ut=fa5fd1943c7b386f172d6893dbbd1d0c\
                &fields1=f1,f2,f3,f4,f5,f6\
                &fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61\
                &klt={}&fqt=1&end=20500101&lmt={}",
                secid, klt, count
            ),
        ];

        // 先尝试东方财富
        for url in &urls {
            match self.fetch_json(url).await {
                Ok(resp) => {
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
                    if !candles.is_empty() {
                        tracing::info!("Fetched {} candles for {} from EastMoney", candles.len(), symbol);
                        return Ok(candles);
                    }
                }
                Err(e) => {
                    tracing::warn!("Kline fetch failed from {}: {}", url, e);
                    continue;
                }
            }
        }

        // 回退到新浪财经
        tracing::info!("Trying Sina finance as fallback for {}", symbol);
        let sina_scale = match period {
            "1d" | "daily" => "240",
            "1w" | "weekly" => "240",
            "1M" | "monthly" => "240",
            "5m" => "5",
            "15m" => "15",
            "30m" => "30",
            "60m" => "60",
            _ => "240",
        };
        
        let sina_symbol = if market == 1 { 
            format!("sh{}", code) 
        } else { 
            format!("sz{}", code) 
        };
        
        let sina_url = format!(
            "https://money.finance.sina.com.cn/quotes_service/api/json_v2.php/CN_MarketData.getKLineData?symbol={}&scale={}&ma=5&datalen={}",
            sina_symbol, sina_scale, count
        );
        
        if let Ok(resp) = self.client.get(&sina_url).send().await {
            if let Ok(text) = resp.text().await {
                if text.starts_with('[') {
                    if let Ok(data) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                        let mut candles: Vec<Candle> = data.iter().rev().map(|k| {
                            Candle {
                                symbol: symbol.to_string(),
                                timestamp: k.get("day").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                open: k.get("open").and_then(|v| v.as_str()).unwrap_or("0").parse().unwrap_or(0.0),
                                close: k.get("close").and_then(|v| v.as_str()).unwrap_or("0").parse().unwrap_or(0.0),
                                high: k.get("high").and_then(|v| v.as_str()).unwrap_or("0").parse().unwrap_or(0.0),
                                low: k.get("low").and_then(|v| v.as_str()).unwrap_or("0").parse().unwrap_or(0.0),
                                volume: k.get("volume").and_then(|v| v.as_str()).unwrap_or("0").parse().unwrap_or(0.0),
                                turnover: 0.0,
                            }
                        }).collect();
                        
                        if !candles.is_empty() {
                            tracing::info!("Fetched {} candles for {} from Sina", candles.len(), symbol);
                            return Ok(candles);
                        }
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Failed to fetch candles for {} from all sources", symbol))
    }

    // 新浪财经获取K线数据（用于分钟级别）
    async fn get_sina_candles(&self, symbol: &str, market: &u8, code: &str, period: &str, count: u32) -> Result<Vec<Candle>> {
        tracing::info!("Fetching {} candles for {} from Sina", period, symbol);
        
        let sina_scale = match period {
            "5m" | "1" => "5",
            "15m" | "5" => "15",
            "30m" | "30" => "30",
            "60m" | "15" => "60",
            _ => "240",
        };
        
        let sina_symbol = if *market == 1 { 
            format!("sh{}", code) 
        } else { 
            format!("sz{}", code) 
        };
        
        let sina_url = format!(
            "https://money.finance.sina.com.cn/quotes_service/api/json_v2.php/CN_MarketData.getKLineData?symbol={}&scale={}&ma=5&datalen={}",
            sina_symbol, sina_scale, count
        );
        
        if let Ok(resp) = self.client.get(&sina_url).send().await {
            if let Ok(text) = resp.text().await {
                if text.starts_with('[') {
                    if let Ok(data) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                        // 获取今天的日期（YYYY-MM-DD格式）
                        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                        
                        let candles: Vec<Candle> = data.iter()
                            .rev()  // 反转，最新的在前
                            .filter(|k| {
                                // 过滤：只保留今天的数据
                                if let Some(day) = k.get("day").and_then(|v| v.as_str()) {
                                    day.starts_with(&today)
                                } else {
                                    false
                                }
                            })
                            .take(count as usize)
                            .map(|k| {
                                Candle {
                                    symbol: symbol.to_string(),
                                    timestamp: k.get("day").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                    open: k.get("open").and_then(|v| v.as_str()).unwrap_or("0").parse().unwrap_or(0.0),
                                    close: k.get("close").and_then(|v| v.as_str()).unwrap_or("0").parse().unwrap_or(0.0),
                                    high: k.get("high").and_then(|v| v.as_str()).unwrap_or("0").parse().unwrap_or(0.0),
                                    low: k.get("low").and_then(|v| v.as_str()).unwrap_or("0").parse().unwrap_or(0.0),
                                    volume: k.get("volume").and_then(|v| v.as_str()).unwrap_or("0").parse().unwrap_or(0.0),
                                    turnover: 0.0,
                                }
                            }).collect();
                        
                        if !candles.is_empty() {
                            tracing::info!("Fetched {} {} candles for {} from Sina", candles.len(), today, symbol);
                            return Ok(candles);
                        }
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Failed to fetch candles for {} from Sina", symbol))
    }

    /// 获取板块行情
    /// 使用 82.push2delay 域名
    pub async fn get_sectors(&self) -> Result<Vec<SectorInfo>> {
        let url = "https://82.push2delay.eastmoney.com/api/qt/clist/get?\
            pn=1&pz=50&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281\
            &fltt=2&invt=2&fid=f3&fs=m:90+t:2+f:!50\
            &fields=f2,f3,f4,f8,f12,f14,f104,f105,f128,f136,f140";

        let resp = self.fetch_json(url).await?;
        let mut sectors = Vec::new();

        if let Some(data) = resp.get("data") {
            if let Some(diff) = data.get("diff").and_then(|d| d.as_array()) {
                for item in diff {
                    let leading_stock = item["f140"].as_str()
                        .or_else(|| item["f128"].as_str())
                        .unwrap_or("")
                        .to_string();
                    
                    sectors.push(SectorInfo {
                        name: item["f14"].as_str().unwrap_or("").to_string(),
                        code: item["f12"].as_str().unwrap_or("").to_string(),
                        change_pct: item["f3"].as_f64().unwrap_or(0.0),
                        turnover: item["f8"].as_f64().unwrap_or(0.0),
                        leading_stock,
                        leading_stock_pct: item["f136"].as_f64().unwrap_or(0.0),
                        stock_count: item["f104"].as_i64().unwrap_or(0) as i32 + item["f105"].as_i64().unwrap_or(0) as i32,
                        up_count: item["f104"].as_i64().unwrap_or(0) as i32,
                        down_count: item["f105"].as_i64().unwrap_or(0) as i32,
                    });
                }
            }
        }

        tracing::info!("Fetched {} sectors", sectors.len());
        Ok(sectors)
    }

    /// 获取板块成分股
    /// bk_code: 板块代码，如 "BK0891"（新能源板块）
    pub async fn get_sector_stocks(&self, bk_code: &str) -> Result<Vec<StockQuote>> {
        let url = format!(
            "https://82.push2delay.eastmoney.com/api/qt/clist/get?\
            pn=1&pz=100&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281\
            &fltt=2&invt=2&fid=f3&fs=b:{bk_code}\
            &fields=f2,f3,f4,f5,f6,f7,f8,f9,f12,f13,f14,f15,f16,f17,f18,f20,f21",
            bk_code = bk_code
        );

        let resp = self.fetch_json(&url).await?;
        let mut stocks = Vec::new();

        if let Some(data) = resp.get("data") {
            if let Some(diff) = data.get("diff").and_then(|d| d.as_array()) {
                for item in diff {
                    stocks.push(StockQuote {
                        symbol: item["f12"].as_str().unwrap_or("").to_string(),
                        name: item["f14"].as_str().unwrap_or("").to_string(),
                        price: item["f2"].as_f64().unwrap_or(0.0),
                        change_pct: item["f3"].as_f64().unwrap_or(0.0),
                        change: item["f4"].as_f64().unwrap_or(0.0),
                        open: item["f17"].as_f64().unwrap_or(0.0),
                        high: item["f15"].as_f64().unwrap_or(0.0),
                        low: item["f16"].as_f64().unwrap_or(0.0),
                        pre_close: 0.0,
                        volume: item["f5"].as_f64().unwrap_or(0.0),
                        turnover: item["f8"].as_f64().unwrap_or(0.0),
                        turnover_rate: 0.0,
                        amplitude: 0.0,
                        pe_ratio: 0.0,
                        total_market_cap: 0.0,
                        circulating_market_cap: 0.0,
                        timestamp: Utc::now(),
                        bid_prices: vec![],
                        bid_volumes: vec![],
                        ask_prices: vec![],
                        ask_volumes: vec![],
                    });
                }
            }
        }

        tracing::info!("Fetched {} stocks in sector {}", stocks.len(), bk_code);
        Ok(stocks)
    }

    /// 获取涨停板数据
    pub async fn get_limit_up_stocks(&self) -> Result<Vec<StockQuote>> {
        let url = "https://82.push2delay.eastmoney.com/api/qt/clist/get?\
            pn=1&pz=200&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281\
            &fltt=2&invt=2&fid=f3&fs=m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23,m:0+t:81+s:2048\
            &fields=f2,f3,f4,f5,f6,f7,f8,f9,f12,f13,f14,f15,f16,f17,f18,f20,f21";

        let resp = self.fetch_json(url).await?;
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

        tracing::info!("Found {} limit-up stocks", stocks.len());
        Ok(stocks)
    }

    /// 获取资金流向
    /// 使用 push2delay 域名（不带82前缀）
    pub async fn get_money_flow(&self, page_size: u32) -> Result<Vec<MoneyFlow>> {
        let url = format!(
            "https://push2delay.eastmoney.com/api/qt/clist/get?\
            pn=1&pz={}&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281\
            &fltt=2&invt=2&fid=f62&fs=m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23,m:0+t:81+s:2048\
            &fields=f12,f13,f14,f62,f66,f72,f78,f84",
            page_size
        );

        let resp = self.fetch_json(&url).await?;
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

        tracing::info!("Fetched {} money flow records", flows.len());
        Ok(flows)
    }

    /// 获取大盘指数
    /// 使用 push2delay 域名
    pub async fn get_market_overview(&self) -> Result<MarketOverview> {
        // 1. 先获取三大指数的详细行情
        let indices = vec![
            ("1.000001", "上证指数"),
            ("0.399001", "深证成指"),
            ("0.399006", "创业板指"),
        ];

        let mut index_quotes = Vec::new();
        for (secid, name) in &indices {
            let url = format!(
                "https://push2delay.eastmoney.com/api/qt/stock/get?\
                ut=fa5fd1943c7b386f172d6893dbbd1d0c&invt=2&fltt=2\
                &fields=f43,f44,f45,f46,f47,f48,f57,f58,f169,f170\
                &secid={}",
                secid
            );
            match self.fetch_json(&url).await {
                Ok(resp) => {
                    if let Some(data) = resp.get("data") {
                        index_quotes.push(IndexQuote {
                            name: name.to_string(),
                            code: data["f57"].as_str().unwrap_or("").to_string(),
                            price: data["f43"].as_f64().unwrap_or(0.0),
                            change: data["f169"].as_f64().unwrap_or(0.0),
                            change_pct: data["f170"].as_f64().unwrap_or(0.0),
                            volume: data["f47"].as_f64().unwrap_or(0.0),
                            turnover: data["f48"].as_f64().unwrap_or(0.0),
                        });
                        tracing::debug!("Index {} price: {}", name, data["f43"].as_f64().unwrap_or(0.0));
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch index {}: {}", name, e);
                    index_quotes.push(IndexQuote {
                        name: name.to_string(),
                        code: String::new(),
                        price: 0.0, change: 0.0, change_pct: 0.0,
                        volume: 0.0, turnover: 0.0,
                    });
                }
            }
        }

        // 2. 统计涨跌平家数 + 涨跌停家数
        //
        // 东方财富 API 每次只返回 100 条，需要分页获取全部 5800+ 股票
        let mut up_count = 0i32;
        let mut down_count = 0i32;
        let mut flat_count = 0i32;
        let mut limit_up = 0i32;
        let mut limit_down = 0i32;
        
        // 分页获取所有股票数据，每页 100 条
        for page in 1..=60 {
            let stats_url = format!(
                "https://82.push2delay.eastmoney.com/api/qt/clist/get?\
                pn={}&pz=100&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281\
                &fltt=2&invt=2&fid=f3&fs=m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23,m:0+t:81+s:2048\
                &fields=f3",
                page
            );
            
            if let Ok(stats_resp) = self.fetch_json(&stats_url).await {
                if let Some(data) = stats_resp.get("data") {
                    if let Some(diff) = data.get("diff").and_then(|d| d.as_array()) {
                        if diff.is_empty() {
                            break; // 没有更多数据
                        }
                        
                        for item in diff {
                            let pct = item["f3"].as_f64().unwrap_or(0.0);

                            if pct > 0.0 {
                                up_count += 1;
                            } else if pct < 0.0 {
                                down_count += 1;
                            } else {
                                flat_count += 1;
                            }
                            if pct >= 9.9 {
                                limit_up += 1;
                            }
                            if pct <= -9.9 {
                                limit_down += 1;
                            }
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let sh = index_quotes.get(0).cloned().unwrap_or(IndexQuote {
            name: "上证指数".into(), code: "000001".into(), price: 0.0, change: 0.0, change_pct: 0.0, volume: 0.0, turnover: 0.0,
        });
        let sz = index_quotes.get(1).cloned().unwrap_or(IndexQuote {
            name: "深证成指".into(), code: "399001".into(), price: 0.0, change: 0.0, change_pct: 0.0, volume: 0.0, turnover: 0.0,
        });
        let cyb = index_quotes.get(2).cloned().unwrap_or(IndexQuote {
            name: "创业板指".into(), code: "399006".into(), price: 0.0, change: 0.0, change_pct: 0.0, volume: 0.0, turnover: 0.0,
        });

        let total_turnover = sh.turnover + sz.turnover;

        tracing::info!("Market overview: SH={:.2}, SZ={:.2}, CYB={:.2}, Up={}, Down={}, Flat={}, LimitUp={}, LimitDown={}", 
            sh.price, sz.price, cyb.price, up_count, down_count, flat_count, limit_up, limit_down);

        Ok(MarketOverview {
            sh_index: sh,
            sz_index: sz,
            cyb_index: cyb,
            total_turnover,
            up_count,
            down_count,
            flat_count,
            limit_up_count: limit_up,
            limit_down_count: limit_down,
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
        if symbol.starts_with("6") || symbol.starts_with("9") {
            (1, symbol)
        } else {
            (0, symbol)
        }
    }
}
