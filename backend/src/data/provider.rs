use anyhow::Result;
use crate::models::*;
use super::eastmoney::EastMoneyApi;

/// 统一数据提供者
pub struct DataProvider {
    api: EastMoneyApi,
}

impl DataProvider {
    pub fn new() -> Self {
        Self {
            api: EastMoneyApi::new(),
        }
    }

    pub async fn get_realtime_quotes(&self, page: u32, page_size: u32) -> Result<Vec<StockQuote>> {
        self.api.get_realtime_quotes(page, page_size).await
    }

    pub async fn get_stock_detail(&self, symbol: &str) -> Result<StockQuote> {
        self.api.get_stock_detail(symbol).await
    }

    pub async fn get_candles(&self, symbol: &str, period: &str, count: u32) -> Result<Vec<Candle>> {
        self.api.get_candles(symbol, period, count).await
    }

    pub async fn get_sectors(&self) -> Result<Vec<SectorInfo>> {
        self.api.get_sectors().await
    }

    pub async fn get_limit_up_stocks(&self) -> Result<Vec<StockQuote>> {
        self.api.get_limit_up_stocks().await
    }

    pub async fn get_money_flow(&self, page_size: u32) -> Result<Vec<MoneyFlow>> {
        self.api.get_money_flow(page_size).await
    }

    pub async fn get_market_overview(&self) -> Result<MarketOverview> {
        self.api.get_market_overview().await
    }
}
