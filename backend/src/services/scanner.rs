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

        // 5. 获取板块数据（需要在计算热点股票之前获取）
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

        // 获取概念板块（用于更精确的个股板块归属）
        let concept_sectors = match self.provider.get_concept_sectors().await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Failed to fetch concept sectors: {}", e);
                Vec::new()
            }
        };

        // 构建股票到板块的映射（获取板块的个股）
        let mut stock_sector_map: HashMap<String, (String, f64)> = HashMap::new();

        // 获取所有行业板块的个股（提高覆盖率）
        for sector in &sectors {
            if let Ok(stocks) = self.provider.get_sector_stocks(&sector.code).await {
                for stock in stocks.iter().take(30) {
                    if !stock.name.is_empty() {
                        stock_sector_map.entry(stock.name.clone()).or_insert((
                            sector.name.clone(),
                            sector.change_pct,
                        ));
                    }
                }
            }
        }

        // 获取概念板块的个股（概念板块覆盖行业板块）
        for sector in &concept_sectors {
            if let Ok(stocks) = self.provider.get_sector_stocks(&sector.code).await {
                for stock in stocks.iter().take(20) {
                    if !stock.name.is_empty() {
                        // 概念板块会覆盖行业板块的归属
                        stock_sector_map.insert(
                            stock.name.clone(),
                            (sector.name.clone(), sector.change_pct),
                        );
                    }
                }
            }
        }

        tracing::info!("Built stock-sector map with {} entries", stock_sector_map.len());

        if !quotes.is_empty() {
            // 2. 计算热点股票（传入板块信息）
            let hot_stocks = self.hot_ranker.rank(&quotes, &stock_sector_map, 200);
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
