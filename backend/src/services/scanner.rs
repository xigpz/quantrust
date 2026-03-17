use anyhow::Result;
use std::collections::HashMap;
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

        // 1. 获取全市场行情
        //
        // A股数量已超过 5000，单页会不全；这里按页拉取并合并（最多 10000 条）。
        let page_size = 5000;
        let mut quotes = Vec::new();
        for page in 1..=2 {
            match self.provider.get_realtime_quotes(page, page_size).await {
                Ok(mut q) => {
                    if q.is_empty() {
                        break;
                    }
                    quotes.append(&mut q);
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch quotes page {}: {}", page, e);
                    break;
                }
            }
        }

        // 去重（极少数情况下分页可能出现重复）
        if quotes.len() > 1 {
            quotes.sort_by(|a, b| a.symbol.cmp(&b.symbol));
            quotes.dedup_by(|a, b| a.symbol == b.symbol);
        }

        // 5. 获取板块数据
        let sectors = match self.provider.get_sectors().await {
            Ok(s) => {
                *self.cache.sectors.write().await = s.clone();
                s
            }
            Err(e) => {
                tracing::warn!("Failed to fetch sectors: {}", e);
                Vec::new()
            }
        };

        if !quotes.is_empty() {
            // 2. 计算热点股票（暂时不传板块信息，后续单独获取）
            let mut hot_stocks = self.hot_ranker.rank(&quotes, 200);

            // 3. 为热点股票获取所属板块（直接查询每只股票的概念）
            for stock in hot_stocks.iter_mut() {
                if let Ok(concepts) = self.provider.get_stock_concepts(&stock.symbol).await {
                    if !concepts.is_empty() {
                        stock.sector_name = concepts[0].clone();
                        // 尝试获取概念板块的涨跌幅
                        if concepts.len() > 1 {
                            stock.sector_name = concepts.join(", ");
                        }
                    }
                }
            }

            *self.cache.hot_stocks.write().await = hot_stocks;

            // 3. 检测异动
            let anomalies = self.anomaly_detector.detect(&quotes, &[]);
            *self.cache.anomaly_stocks.write().await = anomalies;

            // 4. 缓存全部行情
            *self.cache.all_quotes.write().await = quotes;
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
