use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// 股票基本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInfo {
    pub symbol: String,
    pub name: String,
    pub market: String,       // SH, SZ
    pub industry: String,
    pub total_market_cap: f64,
    pub circulating_market_cap: f64,
}

/// 实时行情快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub pre_close: f64,
    pub volume: f64,          // 成交量(手)
    pub turnover: f64,        // 成交额(元)
    pub turnover_rate: f64,   // 换手率(%)
    pub amplitude: f64,       // 振幅(%)
    pub pe_ratio: f64,        // 市盈率
    pub total_market_cap: f64,
    pub circulating_market_cap: f64,
    pub timestamp: DateTime<Utc>,
    // 五档盘口
    pub bid_prices: Vec<f64>,
    pub bid_volumes: Vec<f64>,
    pub ask_prices: Vec<f64>,
    pub ask_volumes: Vec<f64>,
}

/// K线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub symbol: String,
    pub timestamp: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub turnover: f64,
}

/// 板块信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorInfo {
    pub name: String,
    pub code: String,
    pub change_pct: f64,
    pub turnover: f64,
    pub leading_stock: String,
    pub leading_stock_pct: f64,
    pub stock_count: i32,
    pub up_count: i32,
    pub down_count: i32,
    /// 主力净流入（亿元），来自东方财富 f62
    #[serde(default)]
    pub main_net_inflow: f64,
}

/// 板块分时资金流曲线上的一个点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorIntradayPoint {
    pub t: String,
    pub v: f64,
}

/// 单板块当日累计采样序列（随行情扫描追加）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorIntradaySeries {
    pub code: String,
    pub name: String,
    pub points: Vec<SectorIntradayPoint>,
    pub last: f64,
}

/// 多板块分时主力净流入走势（服务端内存聚合）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorIntradayResponse {
    pub trade_date: String,
    pub updated_at: String,
    pub series: Vec<SectorIntradaySeries>,
}

/// 热点股票
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotStock {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
    pub volume: f64,
    pub turnover: f64,
    pub turnover_rate: f64,
    pub hot_score: f64,       // 热度评分
    pub hot_reason: String,   // 热度原因
    pub sector_name: String,  // 所属板块
    pub sector_change_pct: f64, // 板块涨跌幅
    pub timestamp: DateTime<Utc>,
}

/// 异动股票
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyStock {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
    pub anomaly_type: AnomalyType,
    pub anomaly_score: f64,
    pub description: String,
    pub volume: f64,
    pub turnover_rate: f64,
    pub timestamp: DateTime<Utc>,
}

/// 异动类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnomalyType {
    VolumeSpike,       // 成交量突增
    PriceSurge,        // 急速拉升
    PriceDrop,         // 急速下跌
    LimitUp,           // 涨停
    LimitDown,         // 跌停
    LimitUpOpen,       // 涨停打开
    LimitDownOpen,     // 跌停打开
    LargeOrder,        // 大单异动
    TurnoverSpike,     // 换手率突增
    GapUp,             // 跳空高开
    GapDown,           // 跳空低开
    BreakResistance,   // 突破压力位
    BreakSupport,      // 跌破支撑位
    BoardRush,         // 板块异动
}

impl std::fmt::Display for AnomalyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnomalyType::VolumeSpike => write!(f, "成交量突增"),
            AnomalyType::PriceSurge => write!(f, "急速拉升"),
            AnomalyType::PriceDrop => write!(f, "急速下跌"),
            AnomalyType::LimitUp => write!(f, "涨停"),
            AnomalyType::LimitDown => write!(f, "跌停"),
            AnomalyType::LimitUpOpen => write!(f, "涨停打开"),
            AnomalyType::LimitDownOpen => write!(f, "跌停打开"),
            AnomalyType::LargeOrder => write!(f, "大单异动"),
            AnomalyType::TurnoverSpike => write!(f, "换手率突增"),
            AnomalyType::GapUp => write!(f, "跳空高开"),
            AnomalyType::GapDown => write!(f, "跳空低开"),
            AnomalyType::BreakResistance => write!(f, "突破压力位"),
            AnomalyType::BreakSupport => write!(f, "跌破支撑位"),
            AnomalyType::BoardRush => write!(f, "板块异动"),
        }
    }
}

/// 市场概览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOverview {
    pub sh_index: IndexQuote,
    pub sz_index: IndexQuote,
    pub cyb_index: IndexQuote,    // 创业板
    pub total_turnover: f64,       // 两市总成交额
    pub up_count: i32,             // 上涨家数
    pub down_count: i32,           // 下跌家数
    pub flat_count: i32,           // 平盘家数
    pub limit_up_count: i32,       // 涨停家数
    pub limit_down_count: i32,     // 跌停家数
    pub timestamp: DateTime<Utc>,
}

/// 指数行情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexQuote {
    pub name: String,
    pub code: String,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub volume: f64,
    pub turnover: f64,
}

/// 美股行情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsStockQuote {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub pre_close: f64,
    pub volume: f64,
    pub turnover: f64,
    pub market_cap: f64,
    pub pe_ratio: f64,
    pub timestamp: DateTime<Utc>,
}

/// 美股指数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsIndex {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub timestamp: DateTime<Utc>,
}

/// 港股行情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HkStockQuote {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub pre_close: f64,
    pub volume: f64,
    pub turnover: f64,
    pub market_cap: f64,
    pub pe_ratio: f64,
    pub timestamp: DateTime<Utc>,
}

/// 大宗商品数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommodityData {
    pub name: String,
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub unit: String,
}

/// 外汇数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexData {
    pub pair: String,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
}

/// 加密货币数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoData {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_24h: f64,
    pub market_cap: String,
    pub volume_24h: String,
}

/// 资金流向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneyFlow {
    pub symbol: String,
    pub name: String,
    pub main_net_inflow: f64,      // 主力净流入
    pub super_large_inflow: f64,   // 超大单净流入
    pub large_inflow: f64,         // 大单净流入
    pub medium_inflow: f64,        // 中单净流入
    pub small_inflow: f64,         // 小单净流入
    pub timestamp: DateTime<Utc>,
}

/// 公告列表项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockNotice {
    pub art_code: String,
    pub title: String,
    pub notice_date: String,
    pub display_time: String,
    pub column_name: String,
    pub source_type: String,
}

/// 公告列表响应
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StockNoticesResponse {
    pub list: Vec<StockNotice>,
    pub total_hits: i32,
    pub page_index: i32,
    pub page_size: i32,
}

/// 公告详情
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StockNoticeDetail {
    pub title: String,
    pub content: String,
    pub notice_date: String,
    pub display_time: String,
    pub source: String,
    pub column_name: String,
}

/// 财经新闻项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StockNews {
    pub id: String,
    pub title: String,
    pub content: String,
    pub pub_time: String,
    pub source: String,
    pub url: String,
    pub category: String,
}

/// 新闻列表响应
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StockNewsResponse {
    pub list: Vec<StockNews>,
    pub total: i32,
}
