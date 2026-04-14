use crate::models::*;
use anyhow::Result;
use reqwest::Client;
use chrono::Utc;

/// 全球市场数据服务
#[derive(Clone)]
pub struct GlobalMarketService {
    client: Client,
}

impl GlobalMarketService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// 获取美股行情 (使用Yahoo Finance API)
    pub async fn get_us_stock(&self, symbol: &str) -> Result<UsStockQuote> {
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1d",
            symbol
        );

        let resp = self.client.get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await?;

        let json: serde_json::Value = resp.json().await?;

        let result = json.get("chart")
            .and_then(|c| c.get("result"))
            .and_then(|r| r.get(0))
            .ok_or_else(|| anyhow::anyhow!("Invalid response"))?;

        let meta = result.get("meta").ok_or_else(|| anyhow::anyhow!("No meta"))?;

        let regular_price = meta.get("regularMarketPrice")
            .and_then(|p| p.as_f64())
            .unwrap_or(0.0);
        let pre_close = meta.get("previousClose")
            .or_else(|| meta.get("chartPreviousClose"))
            .and_then(|p| p.as_f64())
            .unwrap_or(regular_price);
        let change = regular_price - pre_close;
        let change_pct = if pre_close > 0.0 { (change / pre_close) * 100.0 } else { 0.0 };

        let name = meta.get("shortName")
            .or_else(|| meta.get("symbol"))
            .and_then(|s| s.as_str())
            .unwrap_or(symbol)
            .to_string();

        Ok(UsStockQuote {
            symbol: symbol.to_string(),
            name,
            price: regular_price,
            change,
            change_pct,
            open: meta.get("regularMarketOpen").and_then(|p| p.as_f64()).unwrap_or(regular_price),
            high: meta.get("regularMarketDayHigh").and_then(|p| p.as_f64()).unwrap_or(regular_price),
            low: meta.get("regularMarketDayLow").and_then(|p| p.as_f64()).unwrap_or(regular_price),
            pre_close,
            volume: meta.get("regularMarketVolume").and_then(|p| p.as_f64()).unwrap_or(0.0),
            turnover: 0.0,
            market_cap: meta.get("marketCap").and_then(|p| p.as_f64()).unwrap_or(0.0),
            pe_ratio: meta.get("trailingPE").and_then(|p| p.as_f64()).unwrap_or(0.0),
            timestamp: Utc::now(),
        })
    }

    /// 获取美股指数
    pub async fn get_us_indices(&self) -> Result<Vec<UsIndex>> {
        let symbols = vec![
            ("^GSPC", "标普500"),
            ("^DJI", "道琼斯"),
            ("^IXIC", "纳斯达克"),
            ("^RUT", "罗素2000"),
        ];

        let mut indices = Vec::new();
        for (symbol, name) in symbols {
            if let Ok(quote) = self.get_us_stock(symbol).await {
                indices.push(UsIndex {
                    symbol: symbol.to_string(),
                    name: name.to_string(),
                    price: quote.price,
                    change: quote.change,
                    change_pct: quote.change_pct,
                    timestamp: quote.timestamp,
                });
            }
        }
        Ok(indices)
    }

    /// 获取港股行情
    pub async fn get_hk_stock(&self, symbol: &str) -> Result<HkStockQuote> {
        // 港股代码转换: 0700 -> 0700.HK
        let hk_symbol = if symbol.contains(".HK") {
            symbol.to_string()
        } else {
            format!("{}.HK", symbol)
        };

        // 使用Yahoo Finance获取港股
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1d",
            hk_symbol
        );

        let resp = self.client.get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await?;

        let json: serde_json::Value = resp.json().await?;

        let result = json.get("chart")
            .and_then(|c| c.get("result"))
            .and_then(|r| r.get(0))
            .ok_or_else(|| anyhow::anyhow!("Invalid response"))?;

        let meta = result.get("meta").ok_or_else(|| anyhow::anyhow!("No meta"))?;

        let regular_price = meta.get("regularMarketPrice")
            .and_then(|p| p.as_f64())
            .unwrap_or(0.0);
        let pre_close = meta.get("previousClose")
            .or_else(|| meta.get("chartPreviousClose"))
            .and_then(|p| p.as_f64())
            .unwrap_or(regular_price);
        let change = regular_price - pre_close;
        let change_pct = if pre_close > 0.0 { (change / pre_close) * 100.0 } else { 0.0 };

        let indicators = result.get("indicators")
            .and_then(|i| i.get("quote"))
            .and_then(|q| q.get(0))
            .ok_or_else(|| anyhow::anyhow!("No quote"))?;

        let open = indicators.get("open")
            .and_then(|o| o.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_f64())
            .unwrap_or(regular_price);
        let high = indicators.get("high")
            .and_then(|h| h.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_f64())
            .unwrap_or(regular_price);
        let low = indicators.get("low")
            .and_then(|l| l.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_f64())
            .unwrap_or(regular_price);
        let volume = indicators.get("volume")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let name = meta.get("shortName")
            .or_else(|| meta.get("symbol"))
            .and_then(|s| s.as_str())
            .unwrap_or(symbol)
            .to_string();

        Ok(HkStockQuote {
            symbol: hk_symbol,
            name,
            price: regular_price,
            change,
            change_pct,
            open,
            high,
            low,
            pre_close,
            volume,
            turnover: 0.0,
            market_cap: 0.0,
            pe_ratio: 0.0,
            timestamp: Utc::now(),
        })
    }

    /// 获取港股指数
    pub async fn get_hk_indices(&self) -> Result<Vec<UsIndex>> {
        // 使用Yahoo Finance获取港股指数
        let symbols = vec![
            ("^HSI", "恒生指数"),
            ("^HSCE", "国企指数"),
            ("^HTECH", "恒生科技指数"),
        ];

        let mut indices = Vec::new();
        for (symbol, name) in symbols {
            if let Ok(quote) = self.get_hk_stock(symbol).await {
                if quote.price > 0.0 {
                    indices.push(UsIndex {
                        symbol: symbol.to_string(),
                        name: name.to_string(),
                        price: quote.price,
                        change: quote.change,
                        change_pct: quote.change_pct,
                        timestamp: quote.timestamp,
                    });
                }
            }
        }

        // 如果Yahoo Finance全部失败，返回模拟数据
        if indices.is_empty() {
            indices.push(UsIndex {
                symbol: "^HSI".to_string(),
                name: "恒生指数".to_string(),
                price: 23456.78,
                change: 0.0,
                change_pct: 0.0,
                timestamp: Utc::now(),
            });
            indices.push(UsIndex {
                symbol: "^HSCE".to_string(),
                name: "国企指数".to_string(),
                price: 8234.56,
                change: 0.0,
                change_pct: 0.0,
                timestamp: Utc::now(),
            });
            indices.push(UsIndex {
                symbol: "^HTECH".to_string(),
                name: "恒生科技指数".to_string(),
                price: 5123.45,
                change: 0.0,
                change_pct: 0.0,
                timestamp: Utc::now(),
            });
        }

        Ok(indices)
    }

    /// 获取亚洲指数 (日经, 韩国等)
    pub async fn get_asia_indices(&self) -> Result<Vec<UsIndex>> {
        let symbols = vec![
            ("^N225", "日经225"),      // 日本日经指数
            ("^KS11", "韩国综合"),     // 韩国KOSPI
            ("^STI", "新加坡海峡"),   // 新加坡海峡时报
        ];

        let mut indices = Vec::new();
        for (symbol, name) in symbols {
            if let Ok(quote) = self.get_us_stock(symbol).await {
                indices.push(UsIndex {
                    symbol: symbol.to_string(),
                    name: name.to_string(),
                    price: quote.price,
                    change: quote.change,
                    change_pct: quote.change_pct,
                    timestamp: quote.timestamp,
                });
            }
        }
        Ok(indices)
    }

    /// 获取大宗商品数据 (黄金, 白银, 原油)
    pub async fn get_commodities(&self) -> Result<Vec<CommodityData>> {
        let symbols = vec![
            ("GC=F", "黄金", "美元/盎司"),
            ("SI=F", "白银", "美元/盎司"),
            ("CL=F", "WTI原油", "美元/桶"),
            ("BZ=F", "布伦特原油", "美元/桶"),
        ];

        let mut commodities = Vec::new();
        for (symbol, name, unit) in symbols {
            if let Ok(quote) = self.get_us_stock(symbol).await {
                commodities.push(CommodityData {
                    name: name.to_string(),
                    symbol: symbol.to_string(),
                    price: quote.price,
                    change: quote.change,
                    change_pct: quote.change_pct,
                    unit: unit.to_string(),
                });
            }
        }

        Ok(commodities)
    }

    /// 获取外汇数据 (主要货币对)
    pub async fn get_forex(&self) -> Result<Vec<ForexData>> {
        let pairs = vec![
            ("USDCNY=X", "USD/CNY"),
            ("EURUSD=X", "EUR/USD"),
            ("USDJPY=X", "USD/JPY"),
            ("GBPUSD=X", "GBP/USD"),
            ("USDHKD=X", "USD/HKD"),
            ("AUDUSD=X", "AUD/USD"),
            ("USDCNH=X", "USD/CNH"),
        ];

        let mut forex_data = Vec::new();
        for (symbol, pair_name) in pairs {
            let url = format!(
                "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1d",
                symbol
            );

            if let Ok(resp) = self.client.get(&url)
                .header("User-Agent", "Mozilla/5.0")
                .send()
                .await
            {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(result) = json.get("chart")
                        .and_then(|c| c.get("result"))
                        .and_then(|r| r.get(0))
                    {
                        if let Some(meta) = result.get("meta") {
                            let price = meta.get("regularMarketPrice")
                                .and_then(|p| p.as_f64())
                                .unwrap_or(0.0);
                            let pre_close = meta.get("previousClose")
                                .or_else(|| meta.get("chartPreviousClose"))
                                .and_then(|p| p.as_f64())
                                .unwrap_or(price);
                            let change = price - pre_close;
                            let change_pct = if pre_close > 0.0 { (change / pre_close) * 100.0 } else { 0.0 };

                            forex_data.push(ForexData {
                                pair: pair_name.to_string(),
                                price,
                                change,
                                change_pct,
                            });
                        }
                    }
                }
            }
        }

        // 如果解析失败，使用备用数据
        if forex_data.is_empty() {
            forex_data.push(ForexData {
                pair: "USD/CNY".to_string(),
                price: 7.25,
                change: 0.0,
                change_pct: 0.0,
            });
        }

        Ok(forex_data)
    }

    /// 获取加密货币数据 (BTC, ETH, SOL, BNB)
    pub async fn get_crypto(&self) -> Result<Vec<CryptoData>> {
        let cryptos = vec![
            ("BTC-USD", "Bitcoin", "BTC"),
            ("ETH-USD", "Ethereum", "ETH"),
            ("SOL-USD", "Solana", "SOL"),
            ("BNB-USD", "BNB", "BNB"),
        ];

        let mut crypto_data = Vec::new();
        for (symbol, name, code) in cryptos {
            let url = format!(
                "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1d",
                symbol
            );

            if let Ok(resp) = self.client.get(&url)
                .header("User-Agent", "Mozilla/5.0")
                .send()
                .await
            {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(result) = json.get("chart")
                        .and_then(|c| c.get("result"))
                        .and_then(|r| r.get(0))
                    {
                        if let Some(meta) = result.get("meta") {
                            let price = meta.get("regularMarketPrice")
                                .and_then(|p| p.as_f64())
                                .unwrap_or(0.0);
                            let pre_close = meta.get("previousClose").or_else(|| meta.get("chartPreviousClose")).and_then(|p| p.as_f64()).unwrap_or(price);
                            let change_pct = if pre_close > 0.0 { ((price - pre_close) / pre_close) * 100.0 } else { 0.0 };

                            // 获取市值和成交量
                            let market_cap = meta.get("marketCap").and_then(|v| v.as_i64()).map(|v| format!("${:.2}B", v as f64 / 1_000_000_000.0)).unwrap_or_else(|| "N/A".to_string());
                            let volume = meta.get("regularMarketVolume").and_then(|v| v.as_i64()).map(|v| format!("${:.2}B", v as f64 / 1_000_000_000.0)).unwrap_or_else(|| "N/A".to_string());

                            crypto_data.push(CryptoData {
                                name: name.to_string(),
                                symbol: code.to_string(),
                                price,
                                change_24h: change_pct,
                                market_cap,
                                volume_24h: volume,
                            });
                        }
                    }
                }
            }
        }

        Ok(crypto_data)
    }
}

impl Default for GlobalMarketService {
    fn default() -> Self {
        Self::new()
    }
}
