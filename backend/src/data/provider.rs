use anyhow::Result;
use crate::models::*;
use super::eastmoney::EastMoneyApi;
use super::notices;
use super::news as news_data;
use crate::models::intraday::IntradaySeries;

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

    pub async fn get_concept_sectors(&self) -> Result<Vec<SectorInfo>> {
        self.api.get_concept_sectors().await
    }

    pub async fn get_sector_stocks(&self, bk_code: &str) -> Result<Vec<StockQuote>> {
        self.api.get_sector_stocks(bk_code).await
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

    pub async fn get_stock_notices(&self, symbol: &str, page_index: u32, page_size: u32) -> Result<StockNoticesResponse> {
        let stock_code = symbol.split('.').next().unwrap_or(symbol);
        notices::get_stock_notices(stock_code, page_index, page_size).await
    }

    pub async fn get_notice_detail(&self, art_code: &str) -> Result<StockNoticeDetail> {
        notices::get_notice_detail(art_code).await
    }

    pub async fn get_stock_news(&self, symbol: &str, page_index: u32, page_size: u32) -> Result<StockNewsResponse> {
        let stock_code = symbol.split('.').next().unwrap_or(symbol);
        news_data::get_stock_news(stock_code, page_index, page_size).await
    }

    pub async fn get_news_detail(&self, news_id: &str) -> Result<StockNews> {
        news_data::get_news_detail(news_id).await
    }

    pub async fn get_intraday(&self, symbol: &str, range: &str) -> Result<IntradaySeries> {
        self.api.get_intraday(symbol, range).await
    }
}
