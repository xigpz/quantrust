use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::data::DataProvider;
use crate::models::*;
use super::anomaly::AnomalyDetector;
use super::hot_stocks::HotStockRanker;

/// 市场扫描器 - 定期扫描全市场，检测热点和异动
pub struct MarketScanner {
    provider: Arc<DataProvider>,
    anomaly_detector: AnomalyDetector,
    hot_ranker: HotStockRanker,
    // 缓存最新的扫描结果
    pub cache: Arc<ScannerCache>,
}

/// 扫描结果缓存
pub struct ScannerCache {
    pub hot_stocks: RwLock<Vec<HotStock>>,
    pub anomaly_stocks: RwLock<Vec<AnomalyStock>>,
    pub market_overview: RwLock<Option<MarketOverview>>,
    pub sectors: RwLock<Vec<SectorInfo>>,
    pub all_quotes: RwLock<Vec<StockQuote>>,
    pub money_flow: RwLock<Vec<MoneyFlow>>,
    pub limit_up_stocks: RwLock<Vec<StockQuote>>,
}

impl ScannerCache {
    pub fn new() -> Self {
        Self {
            hot_stocks: RwLock::new(Vec::new()),
            anomaly_stocks: RwLock::new(Vec::new()),
            market_overview: RwLock::new(None),
            sectors: RwLock::new(Vec::new()),
            all_quotes: RwLock::new(Vec::new()),
            money_flow: RwLock::new(Vec::new()),
            limit_up_stocks: RwLock::new(Vec::new()),
        }
    }
}

impl MarketScanner {
    pub fn new(provider: Arc<DataProvider>) -> Self {
        Self {
            provider,
            anomaly_detector: AnomalyDetector::new(),
            hot_ranker: HotStockRanker::new(),
            cache: Arc::new(ScannerCache::new()),
        }
    }

    /// 执行一次全市场扫描
    pub async fn scan(&self) -> Result<()> {
        tracing::info!("Starting market scan...");

        // 1. 获取全市场行情 (前500只活跃股票)
        let quotes = self.provider.get_realtime_quotes(1, 500).await
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to fetch quotes: {}", e);
                Vec::new()
            });

        if !quotes.is_empty() {
            // 2. 计算热点股票
            let hot_stocks = self.hot_ranker.rank(&quotes, 50);
            *self.cache.hot_stocks.write().await = hot_stocks;

            // 3. 检测异动
            let anomalies = self.anomaly_detector.detect(&quotes, &[]);
            *self.cache.anomaly_stocks.write().await = anomalies;

            // 4. 缓存全部行情
            *self.cache.all_quotes.write().await = quotes;
        }

        // 5. 获取板块数据
        if let Ok(sectors) = self.provider.get_sectors().await {
            *self.cache.sectors.write().await = sectors;
        }

        // 6. 获取大盘概览
        if let Ok(overview) = self.provider.get_market_overview().await {
            *self.cache.market_overview.write().await = Some(overview);
        }

        // 7. 获取资金流向
        if let Ok(flows) = self.provider.get_money_flow(50).await {
            *self.cache.money_flow.write().await = flows;
        }

        // 8. 获取涨停股
        if let Ok(limit_ups) = self.provider.get_limit_up_stocks().await {
            *self.cache.limit_up_stocks.write().await = limit_ups;
        }

        tracing::info!("Market scan completed");
        Ok(())
    }
}
