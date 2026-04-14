use crate::db::DbPool;
use crate::models::*;
use crate::data::DataProvider;
use crate::data::global_market::GlobalMarketService;
use anyhow::Result;
use chrono::{Local, Datelike, Weekday, NaiveDate};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

/// 预测进度状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionProgress {
    pub stage: String,           // 当前阶段
    pub message: String,         // 进度消息
    pub progress: f64,           // 进度百分比 0-100
    pub is_completed: bool,      // 是否完成
    pub is_error: bool,          // 是否有错误
    pub error_message: String,    // 错误信息
}

impl Default for PredictionProgress {
    fn default() -> Self {
        Self {
            stage: "idle".to_string(),
            message: "等待开始".to_string(),
            progress: 0.0,
            is_completed: false,
            is_error: false,
            error_message: String::new(),
        }
    }
}

/// 预测进度广播
pub struct PredictionProgressBroadcaster {
    sender: broadcast::Sender<PredictionProgress>,
}

impl PredictionProgressBroadcaster {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PredictionProgress> {
        self.sender.subscribe()
    }

    pub fn broadcast(&self, progress: PredictionProgress) {
        let _ = self.sender.send(progress);
    }

    pub fn send_progress(&self, stage: &str, message: &str, progress: f64) {
        self.broadcast(PredictionProgress {
            stage: stage.to_string(),
            message: message.to_string(),
            progress,
            is_completed: false,
            is_error: false,
            error_message: String::new(),
        });
    }

    pub fn send_error(&self, message: &str) {
        self.broadcast(PredictionProgress {
            stage: "error".to_string(),
            message: message.to_string(),
            progress: 0.0,
            is_completed: true,
            is_error: true,
            error_message: message.to_string(),
        });
    }

    pub fn send_completed(&self, message: &str) {
        self.broadcast(PredictionProgress {
            stage: "completed".to_string(),
            message: message.to_string(),
            progress: 100.0,
            is_completed: true,
            is_error: false,
            error_message: String::new(),
        });
    }
}

impl Default for PredictionProgressBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

/// 详细预测数据收集
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionData {
    pub date: String,
    pub prediction_date: String,
    /// true=针对今日盘中预测，false=针对明日预测
    pub is_today_prediction: bool,

    // 量能数据
    pub shanghai_volume: f64,
    pub shenzhen_volume: f64,
    pub total_volume: f64,
    pub yesterday_volume: f64,
    pub volume_change_pct: f64,
    pub north_flow: f64,
    pub margin_change: f64,
    pub limit_up_count: i32,
    pub limit_down_count: i32,
    pub up_count: i32,
    pub down_count: i32,
    pub volume_ratio: f64,

    // 大盘走势
    pub sh_index: f64,
    pub sh_change_pct: f64,
    pub sh_high: f64,  // 估算
    pub sh_low: f64,   // 估算
    pub sz_index: f64,
    pub sz_change_pct: f64,
    pub cyb_index: f64,
    pub cyb_change_pct: f64,
    pub kline_pattern: String,
    pub support_level: f64,
    pub resistance_level: f64,
    pub macd_status: String,
    pub sector_flow_top3: Vec<String>,
    pub sector_flow_bottom3: Vec<String>,

    // 全球市场
    pub dow_change: f64,
    pub nasdaq_change: f64,
    pub sp500_change: f64,
    pub nikkei_change: f64,
    pub hsi_change: f64,
    pub kospi_change: f64,
    pub dxy: f64,
    pub dxy_change: f64,
    pub cny_rate: f64,
    pub cny_trend: String,
    pub oil_price: f64,
    pub oil_change: f64,
    pub us_bond_yield: f64,
    pub vix: f64,
    pub fed_stance: String,

    // 新闻与情绪
    pub news_list: Vec<NewsItem>,
    pub hot_sector_sentiment: String,
    pub lhb_summary: String,
    pub sentiment_score: f64,
    pub top_inflow_stocks: Vec<String>,
    pub top_outflow_stocks: Vec<String>,
    pub tomorrow_unlock_amount: f64,
    pub tomorrow_unlock_stocks: Vec<String>,
    pub tomorrow_events: Vec<String>,

    // 资金流向
    pub main_net_inflow: f64,
    pub super_large_net_inflow: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub category: String,  // 利好/利空/中性
    pub title: String,
    pub impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorFlow {
    pub name: String,
    pub net_inflow: f64,
}

/// 市场预测详细结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPredictionDetail {
    pub id: i64,
    pub date: String,
    pub prediction_date: String,

    // 综合评分
    pub score_volume: i32,
    pub score_tech: i32,
    pub score_capital: i32,
    pub score_external: i32,
    pub score_news: i32,
    pub score_sentiment: i32,
    pub total_score: i32,
    pub overall_tendency: String,

    // 明日研判
    pub core_judgment: String,
    pub confidence: i32,
    pub bull_probability: i32,
    pub bull_target: String,
    pub bull_focus: String,
    pub bear_probability: i32,
    pub bear_target: String,
    pub bear_risk: String,
    pub base_probability: i32,
    pub base_description: String,
    pub base_support: String,
    pub base_resistance: String,

    // 板块机会
    pub sector_opportunities: Vec<SectorOpportunity>,
    pub sector_risks: Vec<SectorRisk>,

    // 操作建议
    pub position_aggressive: i32,
    pub position_steady: i32,
    pub position_conservative: i32,
    pub action_aggressive: String,
    pub action_steady: String,
    pub action_conservative: String,
    pub short_term_trader: String,
    pub swing_trader: String,
    pub position_holder: String,

    // 风险提示
    pub risk_high: String,
    pub risk_medium: String,
    pub risk_low: String,

    // 重点观察
    pub observation_indicators: Vec<String>,

    // AI完整分析
    pub ai_insight: String,

    pub confidence_score: f64,
    pub created_at: String,

    // 发送给DeepSeek的原始数据
    #[serde(default)]
    pub input_data: Option<PredictionInputData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorOpportunity {
    pub sector: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorRisk {
    pub sector: String,
    pub reason: String,
}

/// 发送给DeepSeek的原始数据（用于前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionInputData {
    // 量能数据
    pub shanghai_volume: String,
    pub shenzhen_volume: String,
    pub total_volume: String,
    pub yesterday_volume: String,
    pub volume_change_pct: String,
    pub north_flow: String,
    pub limit_up_count: String,
    pub limit_down_count: String,
    pub up_count: String,
    pub down_count: String,
    pub volume_ratio: String,

    // 大盘走势
    pub sh_index: String,
    pub sh_change_pct: String,
    pub sz_index: String,
    pub sz_change_pct: String,
    pub cyb_index: String,
    pub cyb_change_pct: String,
    pub kline_pattern: String,
    pub support_level: String,
    pub resistance_level: String,
    pub macd_status: String,
    pub sector_flow_top3: String,
    pub sector_flow_bottom3: String,

    // 全球市场
    pub dow_change: String,
    pub nasdaq_change: String,
    pub sp500_change: String,
    pub nikkei_change: String,
    pub hsi_change: String,
    pub kospi_change: String,
    pub dxy: String,
    pub cny_rate: String,
    pub oil_price: String,
    pub us_bond_yield: String,
    pub vix: String,
    pub vix_status: String,

    // 新闻
    pub news_list: Vec<NewsItem>,

    // 情绪
    pub sentiment_score: String,
    pub hot_sector_sentiment: String,
    pub lhb_summary: String,
    pub top_inflow_stocks: String,
    pub top_outflow_stocks: String,
}

/// 市场预测服务
pub struct MarketPredictionService {
    db: DbPool,
    provider: Arc<DataProvider>,
    progress: Arc<RwLock<PredictionProgress>>,
    broadcaster: Arc<PredictionProgressBroadcaster>,
}

impl MarketPredictionService {
    pub fn new(db: DbPool, provider: Arc<DataProvider>) -> Self {
        Self {
            db,
            provider,
            progress: Arc::new(RwLock::new(PredictionProgress::default())),
            broadcaster: Arc::new(PredictionProgressBroadcaster::new()),
        }
    }

    /// 获取当前进度
    pub async fn get_progress(&self) -> PredictionProgress {
        self.progress.read().unwrap().clone()
    }

    /// 订阅进度更新
    pub fn subscribe_progress(&self) -> broadcast::Receiver<PredictionProgress> {
        self.broadcaster.subscribe()
    }

    /// 更新进度
    async fn update_progress(&self, stage: &str, message: &str, progress: f64) {
        let prog = PredictionProgress {
            stage: stage.to_string(),
            message: message.to_string(),
            progress,
            is_completed: false,
            is_error: false,
            error_message: String::new(),
        };
        *self.progress.write().unwrap() = prog.clone();
        self.broadcaster.broadcast(prog);
    }

    /// 运行每日市场预测分析
    pub async fn run_daily_prediction(&self) -> Result<MarketPredictionDetail> {
        // 初始化进度
        self.update_progress("initializing", "正在初始化预测任务...", 5.0).await;

        // 1. 收集数据
        self.update_progress("collecting", "正在收集大盘数据...", 10.0).await;
        let market_data = self.collect_market_data().await?;

        self.update_progress("collecting", "正在收集全球市场数据...", 20.0).await;
        // 全球市场数据已包含在 collect_market_data 中

        self.update_progress("collecting", "正在收集资金流向数据...", 30.0).await;
        // 资金流向数据已包含在 collect_market_data 中

        self.update_progress("collecting", "正在收集新闻数据...", 40.0).await;
        // 新闻数据已包含在 collect_market_data 中

        self.update_progress("collecting", "正在收集市场情绪数据...", 50.0).await;
        // 情绪数据已包含在 collect_market_data 中

        self.update_progress("building", "正在构建分析Prompt...", 55.0).await;

        // 2. 构建Prompt
        let system_prompt = self.build_system_prompt();
        let user_prompt = self.build_prediction_prompt(&market_data);

        // 3. 调用DeepSeek API
        self.update_progress("ai_analysis", "正在发送数据到DeepSeek AI分析...", 60.0).await;
        let ai_response = self.call_deepseek(&system_prompt, &user_prompt).await?;

        self.update_progress("ai_analysis", "正在分析AI返回结果...", 80.0).await;

        // 4. 解析结果
        let mut prediction = self.parse_prediction(&ai_response, &market_data)?;

        // 添加原始输入数据
        prediction.input_data = Some(PredictionInputData {
            shanghai_volume: format!("{:.2}亿", market_data.shanghai_volume / 1e8),
            shenzhen_volume: format!("{:.2}亿", market_data.shenzhen_volume / 1e8),
            total_volume: format!("{:.2}亿", market_data.total_volume / 1e8),
            yesterday_volume: format!("{:.2}亿", market_data.yesterday_volume / 1e8),
            volume_change_pct: format!("{:.2}%", market_data.volume_change_pct),
            north_flow: format!("{:.2}亿", market_data.north_flow / 1e8),
            limit_up_count: format!("{}家", market_data.limit_up_count),
            limit_down_count: format!("{}家", market_data.limit_down_count),
            up_count: format!("{}家", market_data.up_count),
            down_count: format!("{}家", market_data.down_count),
            volume_ratio: format!("{:.2}", market_data.volume_ratio),
            sh_index: format!("{:.2}", market_data.sh_index),
            sh_change_pct: format!("{:.2}%", market_data.sh_change_pct),
            sz_index: format!("{:.2}", market_data.sz_index),
            sz_change_pct: format!("{:.2}%", market_data.sz_change_pct),
            cyb_index: format!("{:.2}", market_data.cyb_index),
            cyb_change_pct: format!("{:.2}%", market_data.cyb_change_pct),
            kline_pattern: market_data.kline_pattern.clone(),
            support_level: format!("{:.2}", market_data.support_level),
            resistance_level: format!("{:.2}", market_data.resistance_level),
            macd_status: market_data.macd_status.clone(),
            sector_flow_top3: market_data.sector_flow_top3.join("、"),
            sector_flow_bottom3: market_data.sector_flow_bottom3.join("、"),
            dow_change: format!("{:.2}%", market_data.dow_change),
            nasdaq_change: format!("{:.2}%", market_data.nasdaq_change),
            sp500_change: format!("{:.2}%", market_data.sp500_change),
            nikkei_change: format!("{:.2}%", market_data.nikkei_change),
            hsi_change: format!("{:.2}%", market_data.hsi_change),
            kospi_change: format!("{:.2}%", market_data.kospi_change),
            dxy: format!("{:.2}", market_data.dxy),
            cny_rate: format!("{:.4}", market_data.cny_rate),
            oil_price: format!("{:.2}美元/桶", market_data.oil_price),
            us_bond_yield: format!("{:.2}%", market_data.us_bond_yield),
            vix: format!("{:.2}", market_data.vix),
            vix_status: if market_data.vix < 20.0 { "低恐慌".to_string() } else if market_data.vix < 30.0 { "中度恐慌".to_string() } else { "高恐慌".to_string() },
            news_list: market_data.news_list.clone(),
            sentiment_score: format!("{:.0}", market_data.sentiment_score),
            hot_sector_sentiment: market_data.hot_sector_sentiment.clone(),
            lhb_summary: market_data.lhb_summary.clone(),
            top_inflow_stocks: market_data.top_inflow_stocks.join("、"),
            top_outflow_stocks: market_data.top_outflow_stocks.join("、"),
        });

        self.update_progress("saving", "正在保存预测结果...", 90.0).await;

        // 5. 保存到数据库
        self.save_prediction(&prediction).await?;

        self.broadcaster.send_completed("预测完成！");
        *self.progress.write().unwrap() = PredictionProgress {
            stage: "completed".to_string(),
            message: "预测完成".to_string(),
            progress: 100.0,
            is_completed: true,
            is_error: false,
            error_message: String::new(),
        };

        Ok(prediction)
    }

    /// 获取下一个交易日
    fn get_next_trading_day(&self) -> String {
        let today = Local::now().date_naive();
        let mut next_day = today;

        // 找到下一个周一到周五
        for _ in 0..7 {
            next_day = next_day.succ_opt().unwrap_or(next_day);
            let weekday = next_day.weekday();
            if weekday != Weekday::Sat && weekday != Weekday::Sun {
                return next_day.format("%Y-%m-%d").to_string();
            }
        }

        next_day.format("%Y-%m-%d").to_string()
    }

    /// 收集当日市场数据
    async fn collect_market_data(&self) -> Result<PredictionData> {
        let now = Local::now();
        let today = now.format("%Y-%m-%d").to_string();

        // 判断是今日盘中预测还是明日预测：下午3点前为今日盘中，3点后为明日
        let current_hour = chrono::Timelike::hour(&now.time());
        let is_today_prediction = current_hour < 15;
        let prediction_date = if is_today_prediction {
            today.clone()
        } else {
            self.get_next_trading_day()
        };

        // 获取大盘概览
        let market_overview = self.provider.get_market_overview().await.ok();

        // 获取全球市场
        let global_market = self.get_global_market().await.ok();

        // 获取资金流向
        let flows = self.provider.get_money_flow(50).await.unwrap_or_default();

        // 获取涨停/跌停数量
        let limit_up_count = self.provider.get_limit_up_stocks().await
            .map(|v| v.len() as i32)
            .unwrap_or(0);
        let limit_down_count = self.provider.get_limit_down_stocks().await
            .map(|v| v.len() as i32)
            .unwrap_or(0);

        // 计算资金流向
        let total_main_inflow: f64 = flows.iter().map(|f| f.main_net_inflow).sum();
        let total_super_large_inflow: f64 = flows.iter().map(|f| f.super_large_inflow).sum();

        // 排序获取TOP/BOTTOM
        let mut sorted_flows = flows.clone();
        sorted_flows.sort_by(|a, b| b.main_net_inflow.partial_cmp(&a.main_net_inflow).unwrap_or(std::cmp::Ordering::Equal));

        let sector_flow_top3: Vec<String> = sorted_flows.iter()
            .filter(|f| f.main_net_inflow > 0.0)
            .take(3)
            .map(|f| format!("{}({:.2}亿)", f.name, f.main_net_inflow / 1e8))
            .collect();

        let sector_flow_bottom3: Vec<String> = sorted_flows.iter()
            .filter(|f| f.main_net_inflow < 0.0)
            .take(3)
            .map(|f| format!("{}({:.2}亿)", f.name, f.main_net_inflow / 1e8))
            .collect();

        // 资金流入/流出最多的个股
        let mut stock_flows = flows.clone();
        stock_flows.sort_by(|a, b| b.main_net_inflow.partial_cmp(&a.main_net_inflow).unwrap_or(std::cmp::Ordering::Equal));
        let top_inflow_stocks: Vec<String> = stock_flows.iter()
            .filter(|f| f.main_net_inflow > 0.0)
            .take(5)
            .map(|f| format!("{}: {:.2}亿", f.name, f.main_net_inflow / 1e8))
            .collect();
        let top_outflow_stocks: Vec<String> = stock_flows.iter()
            .filter(|f| f.main_net_inflow < 0.0)
            .take(5)
            .map(|f| format!("{}: {:.2}亿", f.name, f.main_net_inflow / 1e8))
            .collect();

        // 获取市场评论情感
        let sentiment_score = self.get_sentiment_score().await.unwrap_or(50.0);

        // 获取新闻列表
        let news_list = self.get_news_list().await;

        // 获取全球市场数据
        let (dow_change, nasdaq_change, sp500_change) = self.get_us_indices_change().await;
        let (nikkei_change, hsi_change, kospi_change) = self.get_asia_indices_change().await;

        // 提取大盘数据（high/low为估算）
        let (sh_index, sh_change_pct) = market_overview.as_ref()
            .map(|o| (o.sh_index.price, o.sh_index.change_pct))
            .unwrap_or((0.0, 0.0));
        // 估算日内高低点
        let amplitude = 1.5; // 假设日内振幅约1.5%
        let sh_high = sh_index * (1.0 + amplitude / 100.0);
        let sh_low = sh_index * (1.0 - amplitude / 100.0);

        let sz_index = market_overview.as_ref().map(|o| o.sz_index.price).unwrap_or(0.0);
        let sz_change_pct = market_overview.as_ref().map(|o| o.sz_index.change_pct).unwrap_or(0.0);
        let cyb_index = market_overview.as_ref().map(|o| o.cyb_index.price).unwrap_or(0.0);
        let cyb_change_pct = market_overview.as_ref().map(|o| o.cyb_index.change_pct).unwrap_or(0.0);

        let total_volume = market_overview.as_ref().map(|o| o.total_turnover).unwrap_or(0.0);
        let up_count = market_overview.as_ref().map(|o| o.up_count).unwrap_or(0);
        let down_count = market_overview.as_ref().map(|o| o.down_count).unwrap_or(0);

        Ok(PredictionData {
            date: today,
            prediction_date,
            is_today_prediction,

            // 量能数据（简化，部分数据暂无）
            shanghai_volume: total_volume * 0.45,  // 估算
            shenzhen_volume: total_volume * 0.55,  // 估算
            total_volume,
            yesterday_volume: total_volume * 0.95, // 估算
            volume_change_pct: -5.0,              // 估算
            north_flow: 0.0,                      // 北向资金暂无
            margin_change: 0.0,                   // 融资余额暂无
            limit_up_count,
            limit_down_count,
            up_count,
            down_count,
            volume_ratio: 1.0,                    // 量比暂无

            // 大盘走势
            sh_index,
            sh_change_pct,
            sh_high,
            sh_low,
            sz_index,
            sz_change_pct,
            cyb_index,
            cyb_change_pct,
            kline_pattern: self.estimate_kline_pattern(sh_change_pct, sh_high, sh_low, sh_index),
            support_level: sh_index * 0.99,       // 简单估算
            resistance_level: sh_index * 1.01,     // 简单估算
            macd_status: "红柱缩短".to_string(),   // 简化
            sector_flow_top3,
            sector_flow_bottom3,

            // 全球市场
            dow_change,
            nasdaq_change,
            sp500_change,
            nikkei_change,
            hsi_change,
            kospi_change,
            dxy: 104.0,
            dxy_change: 0.0,
            cny_rate: 7.25,
            cny_trend: "震荡".to_string(),
            oil_price: 75.0,
            oil_change: 0.0,
            us_bond_yield: 4.5,
            vix: 15.0,
            fed_stance: "暂无最新表态".to_string(),

            // 新闻与情绪
            news_list,
            hot_sector_sentiment: "板块分化明显".to_string(),
            lhb_summary: "机构观望为主".to_string(),
            sentiment_score,
            top_inflow_stocks,
            top_outflow_stocks,
            tomorrow_unlock_amount: 0.0,
            tomorrow_unlock_stocks: vec!["暂无数据".to_string()],
            tomorrow_events: vec!["关注美股夜间表现".to_string()],

            // 资金流向
            main_net_inflow: total_main_inflow,
            super_large_net_inflow: total_super_large_inflow,
        })
    }

    /// 估算K线形态
    fn estimate_kline_pattern(&self, change_pct: f64, high: f64, low: f64, close: f64) -> String {
        if close <= 0.0 || high <= 0.0 || low <= 0.0 {
            return "数据不足".to_string();
        }

        let amplitude = (high - low) / close * 100.0;
        let upper_shadow = (high - close) / close * 100.0;
        let lower_shadow = (close - low) / close * 100.0;

        if change_pct > 2.0 && upper_shadow < 0.3 {
            "光头阳线".to_string()
        } else if change_pct > 2.0 && upper_shadow > amplitude * 0.4 {
            "带上影线的阳线".to_string()
        } else if change_pct < -2.0 && lower_shadow < 0.3 {
            "光头阴线".to_string()
        } else if change_pct < -2.0 && lower_shadow > amplitude * 0.4 {
            "带下影线的阴线".to_string()
        } else if amplitude < 0.5 {
            "十字星".to_string()
        } else if change_pct > 0.0 {
            "阳线".to_string()
        } else {
            "阴线".to_string()
        }
    }

    async fn get_sentiment_score(&self) -> Result<f64> {
        let comments = self.provider.get_market_comments(50).await?;
        let total = comments.overall_sentiment.positive as f64
            + comments.overall_sentiment.negative as f64
            + comments.overall_sentiment.neutral as f64;

        if total > 0.0 {
            let ratio = (comments.overall_sentiment.positive as f64 - comments.overall_sentiment.negative as f64) / total;
            Ok(50.0 + ratio * 30.0) // 转换为0-100分数
        } else {
            Ok(50.0)
        }
    }

    async fn get_news_list(&self) -> Vec<NewsItem> {
        // 从东方财富获取今日重要新闻
        let news = self.provider.get_stock_news("000001", 1, 10).await.unwrap_or_default();
        news.list.into_iter().take(5).map(|n| NewsItem {
            category: "中性".to_string(),
            title: n.title,
            impact: "影响待观察".to_string(),
        }).collect()
    }

    async fn get_us_indices_change(&self) -> (f64, f64, f64) {
        let service = GlobalMarketService::new();
        let indices = service.get_us_indices().await.unwrap_or_default();

        let mut dow = 0.0;
        let mut nasdaq = 0.0;
        let mut sp500 = 0.0;

        for idx in indices {
            let change = idx.change_pct;
            if idx.symbol.contains("DJI") || idx.symbol.contains("Dow") {
                dow = change;
            } else if idx.symbol.contains("IXIC") || idx.symbol.contains("Nasdaq") {
                nasdaq = change;
            } else if idx.symbol.contains("GSPC") || idx.symbol.contains("SPX") {
                sp500 = change;
            }
        }

        (dow, nasdaq, sp500)
    }

    async fn get_asia_indices_change(&self) -> (f64, f64, f64) {
        let service = GlobalMarketService::new();
        let asia = service.get_asia_indices().await.unwrap_or_default();

        let mut nikkei = 0.0;
        let mut hsi = 0.0;
        let mut kospi = 0.0;

        for idx in asia {
            let change = idx.change_pct;
            if idx.symbol.contains("N225") || idx.symbol.contains("Nikkei") {
                nikkei = change;
            } else if idx.symbol.contains("HSI") || idx.symbol.contains("Hang") {
                hsi = change;
            } else if idx.symbol.contains("KS11") || idx.symbol.contains("KOSPI") {
                kospi = change;
            }
        }

        (nikkei, hsi, kospi)
    }

    fn get_global_market(&self) -> impl std::future::Future<Output = Result<GlobalMarketOverview>> + '_ {
        async {
            let service = GlobalMarketService::new();

            let us_indices = service.get_us_indices().await.unwrap_or_default();
            let hk_indices = service.get_hk_indices().await.unwrap_or_default();
            let asia_indices = service.get_asia_indices().await.unwrap_or_default();
            let eu_indices = service.get_eu_indices().await.unwrap_or_default();
            let gold = service.get_gold_index().await.unwrap_or_default();
            let crude_oil = service.get_crude_oil_index().await.unwrap_or_default();

            Ok(GlobalMarketOverview {
                us_indices,
                hk_indices,
                asia_indices,
                eu_indices,
                gold,
                crude_oil,
                a_share_overview: None,
                timestamp: chrono::Utc::now(),
            })
        }
    }

    /// 构建系统提示词
    fn build_system_prompt(&self) -> String {
        r#"你是一位拥有20年经验的专业 A 股量化分析师，曾就职于头部券商研究所，
擅长综合基本面、技术面，资金面、情绪面进行多维度行情研判。

你的分析风格：
- 客观中立，不受短期情绪左右
- 用数据说话，每个判断必须有依据
- 永远给出多空两种情景，不做单边押注
- 明确标注置信度和风险提示
- 语言专业简练，避免模糊表述（如"可能涨也可能跌"）

重要约束：
1. 多空情景概率之和必须等于100%，任何单一情景概率不得超过65%
2. 综合评分禁止出现满分（100分）或零分，极端行情上限90分/下限10分
3. 操作建议中，任何市场环境下激进型仓位不得超过80%，保守型不得低于10%
4. 每个利好判断必须配套列出对应的反驳理由
5. 禁止出现"必涨""必跌""稳赚"等绝对化表述"#.to_string()
    }

    /// 构建预测Prompt
    fn build_prediction_prompt(&self, data: &PredictionData) -> String {
        // 根据预测类型决定措辞：下午3点前为今日盘中预测，3点后为明日预测
        let prediction_target = if data.is_today_prediction {
            "今日盘中"
        } else {
            "明日"
        };

        let mut prompt = format!(
            r#"请根据以下{}的多维度数据，对{}（{}）A股大盘及个股走势进行专业研判。

---

## 📊 一、今日量能数据

- 上证成交额：{:.2} 亿元
- 深证成交额：{:.2} 亿元
- 全市场成交额：{:.2} 亿元（昨日：{:.2} 亿元，变化：{:+.2}%）
- 北向资金净流入：暂无数据
- 融资余额变化：暂无数据
- 涨停家数：{} 家 / 跌停家数：{} 家
- 涨跌比：{} : {}
- 量比（今日/5日均量）：{:.2}

---

## 📈 二、今日大盘走势

- 上证指数：{:.2}（涨跌：{:+.2}%，日内最高：{:.2}，最低：{:.2}）
- 深证成指：{:.2}（涨跌：{:+.2}%）
- 创业板指：{:.2}（涨跌：{:+.2}%）
- 今日 K 线形态：{}
- 关键支撑位：{:.2} / 关键压力位：{:.2}
- MACD 状态：{}
- 行业板块资金流向 TOP3：{}
- 行业板块资金流出 TOP3：{}

---

## 🌍 三、国际市场与宏观环境

- 美股昨夜收盘：道指 {:+.2}%，纳指 {:+.2}%，标普 {:+.2}%
- 亚太市场今日表现：日经 {:+.2}%，恒生 {:+.2}%，韩综 {:+.2}%
- 美元指数：{:.2}（变化：{:+.2}）
- 人民币汇率：{:.2}（变化方向：{}）
- 国际原油：{:.2} 美元/桶（变化：{:+.2}%）
- 10年期美债收益率：{:.2}%
- VIX 恐慌指数：{:.2}（状态：{}）
- 近期美联储/央行表态：{}

---

## 📰 四、今日重要新闻与政策

"#,
            if data.is_today_prediction { "今日盘中" } else { "今日收盘后" },
            prediction_target,
            data.prediction_date,
            data.shanghai_volume / 1e8,
            data.shenzhen_volume / 1e8,
            data.total_volume / 1e8,
            data.yesterday_volume / 1e8,
            data.volume_change_pct,
            data.limit_up_count,
            data.limit_down_count,
            data.up_count,
            data.down_count,
            data.volume_ratio,
            data.sh_index,
            data.sh_change_pct,
            data.sh_high,
            data.sh_low,
            data.sz_index,
            data.sz_change_pct,
            data.cyb_index,
            data.cyb_change_pct,
            data.kline_pattern,
            data.support_level,
            data.resistance_level,
            data.macd_status,
            data.sector_flow_top3.join("、"),
            data.sector_flow_bottom3.join("、"),
            data.dow_change,
            data.nasdaq_change,
            data.sp500_change,
            data.nikkei_change,
            data.hsi_change,
            data.kospi_change,
            data.dxy,
            data.dxy_change,
            data.cny_rate,
            data.cny_trend,
            data.oil_price,
            data.oil_change,
            data.us_bond_yield,
            data.vix,
            if data.vix < 20.0 { "<20 低恐慌" } else if data.vix < 30.0 { "20-30 中度" } else { ">30 高恐慌" },
            data.fed_stance,
        );

        // 添加新闻列表
        for (i, news) in data.news_list.iter().enumerate().take(5) {
            prompt.push_str(&format!("{}. [{}] {} - {}\n", i + 1, news.category, news.title, news.impact));
        }

        prompt.push_str(&format!(r#"

---

## 💬 五、个股与市场情绪

- 今日热门板块情绪：{}
- 龙虎榜机构动向：{}
- 股吧/财经社区情绪指数：{:.0}（0-100，50为中性，>70偏乐观，<30偏悲观）
- 近5日主力资金净流入个股：{}
- 近5日主力资金净流出个股：{}
- 明日解禁规模：{:.2} 亿元 / 涉及个股：{}
- 明日重要事件：{}

---

## 🎯 请按以下结构输出分析报告：

### 一、综合评分（100分制）
| 维度 | 评分 | 简评（一句话）|
|------|------|--------------|
| 量能面 | XX分 | |
| 技术面 | XX分 | |
| 资金面 | XX分 | |
| 外部环境 | XX分 | |
| 政策/新闻 | XX分 | |
| 市场情绪 | XX分 | |
| **综合得分** | **XX分** | **总体倾向** |

> 评分说明：60分以下偏空，60-70分震荡，70-80分偏多，80分以上强势

---

### 二、明日大盘研判

**核心判断**：[震荡偏强 / 震荡 / 震荡偏弱]（置信度：XX%）

**多头情景**（概率：XX%）：
- 触发条件：...
- 目标区间：上证 XXXX - XXXX 点
- 重点关注板块：...

**空头情景**（概率：XX%）：
- 触发条件：...
- 目标区间：上证 XXXX - XXXX 点
- 需警惕的风险：...

**基准情景**（概率：XX%）：
- 预期走势描述：...
- 核心支撑：XXXX 点 / 核心压力：XXXX 点

---

### 三、板块机会与风险

**明日值得关注的板块**（按确定性排序）：
1. **[板块名]**：理由 + 核心逻辑（基于数据，非主观臆测）
2. **[板块名]**：...
3. **[板块名]**：...

**明日需回避的板块**：
1. **[板块名]**：风险原因
2. ...

---

### 四、操作建议

> ⚠️ 以下建议仅供参考，不构成投资建议，市场有风险，入市需谨慎。

| 仓位建议 | 激进型 | 稳健型 | 保守型 |
|---------|--------|--------|--------|
| 建议仓位 | XX% | XX% | XX% |
| 操作方向 | ... | ... | ... |

- **短线交易者**：...
- **波段交易者**：...
- **持仓者**：...

---

### 五、关键风险提示

> 以下风险若触发，可能导致判断失效，请重新评估：

1. 🔴 高风险：...
2. 🟡 中风险：...
3. 🟢 低风险但需观察：...

---

### 六、明日重点观察指标

开盘后前30分钟重点观察：
- [ ] 集合竞价涨跌幅是否超过 ±0.5%
- [ ] 北向资金开盘方向
- [ ] 昨日强势板块是否延续
- [ ] 上证是否有效站稳/跌破 {:.2} 点

---

**分析师备注**：本报告基于收盘后静态数据生成，如隔夜出现重大突发事件（地缘冲突、重要政策、黑天鹅事件），请以最新信息为准，本报告结论自动失效。

报告生成时间：{}
数据截止时间：{}（当日15:00收盘）
"#,
            data.hot_sector_sentiment,
            data.lhb_summary,
            data.sentiment_score,
            data.top_inflow_stocks.join("、"),
            data.top_outflow_stocks.join("、"),
            data.tomorrow_unlock_amount,
            data.tomorrow_unlock_stocks.join("、"),
            data.tomorrow_events.join("、"),
            data.resistance_level,
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            data.date,
        ));

        // 将prompt中的"明日"替换为实际预测目标（今日盘中或明日）
        prompt = prompt.replace("明日", prediction_target);

        prompt
    }

    async fn call_deepseek(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let api_key = std::env::var("DEEPSEEK_API_KEY")
            .map_err(|e| anyhow::anyhow!("DEEPSEEK_API_KEY not set: {}", e))?;

        let url = "https://api.deepseek.com/chat/completions";

        #[derive(Serialize)]
        struct DeepSeekRequest {
            model: String,
            messages: Vec<DeepSeekMessage>,
            temperature: f32,
        }

        #[derive(Serialize)]
        struct DeepSeekMessage {
            role: String,
            content: String,
        }

        let request = DeepSeekRequest {
            model: "deepseek-reasoner".to_string(),
            messages: vec![
                DeepSeekMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                DeepSeekMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                }
            ],
            temperature: 0.3,
        };

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("DeepSeek API request failed: {}", e))?;

        let json: serde_json::Value = response.json().await
            .map_err(|e| anyhow::anyhow!("DeepSeek API response parse failed: {}", e))?;

        // Debug: 打印原始响应
        tracing::info!("DeepSeek response: {}", json);

        // 检查是否有error字段
        if let Some(error) = json.get("error") {
            let error_msg = error.to_string();
            return Err(anyhow::anyhow!("DeepSeek API error: {}", error_msg));
        }

        let content = json.get("choices")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| anyhow::anyhow!("Failed to parse DeepSeek response, json: {}", json))?
            .to_string();

        Ok(content)
    }

    fn parse_prediction(&self, ai_response: &str, data: &PredictionData) -> Result<MarketPredictionDetail> {
        // 解析markdown格式的响应
        let response = ai_response.trim();

        let today = Local::now().format("%Y-%m-%d").to_string();

        // 提取综合评分（简化解析）
        let (score_volume, score_tech, score_capital, score_external, score_news, score_sentiment, total_score, overall_tendency) =
            self.parse_score_section(response);

        // 提取明日研判
        let (core_judgment, confidence, bull_probability, bull_target, bull_focus, bear_probability, bear_target, bear_risk, base_probability, base_description, base_support, base_resistance) =
            self.parse_judgment_section(response);

        // 提取板块机会与风险
        let (sector_opportunities, sector_risks) = self.parse_sector_section(response);

        // 提取操作建议
        let (position_aggressive, position_steady, position_conservative, short_term_trader, swing_trader, position_holder) =
            self.parse_action_section(response);

        // 提取风险提示
        let (risk_high, risk_medium, risk_low) = self.parse_risk_section(response);

        // 提取观察指标
        let observation_indicators = self.parse_observation_section(response);

        Ok(MarketPredictionDetail {
            id: 0,
            date: today.clone(),
            prediction_date: data.prediction_date.clone(),
            score_volume,
            score_tech,
            score_capital,
            score_external,
            score_news,
            score_sentiment,
            total_score,
            overall_tendency,
            core_judgment,
            confidence,
            bull_probability,
            bull_target,
            bull_focus,
            bear_probability,
            bear_target,
            bear_risk,
            base_probability,
            base_description,
            base_support,
            base_resistance,
            sector_opportunities,
            sector_risks,
            position_aggressive,
            position_steady,
            position_conservative,
            action_aggressive: "偏向强势板块".to_string(),
            action_steady: "均衡配置".to_string(),
            action_conservative: "防御为主".to_string(),
            short_term_trader,
            swing_trader,
            position_holder,
            risk_high,
            risk_medium,
            risk_low,
            observation_indicators,
            ai_insight: response.to_string(),
            confidence_score: confidence as f64,
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            input_data: None,
        })
    }

    fn parse_score_section(&self, response: &str) -> (i32, i32, i32, i32, i32, i32, i32, String) {
        // 尝试解析评分表格，简化版本
        let mut score_volume = 65;
        let mut score_tech = 65;
        let mut score_capital = 65;
        let mut score_external = 65;
        let mut score_news = 65;
        let mut score_sentiment = 65;
        let mut total_score = 65;
        let mut overall_tendency = "震荡".to_string();

        // 尝试提取数值
        for line in response.lines() {
            let line = line.trim();
            if line.contains("量能面") && line.contains("分") {
                if let Some(val) = self.extract_score(line) {
                    score_volume = val;
                }
            } else if line.contains("技术面") && line.contains("分") {
                if let Some(val) = self.extract_score(line) {
                    score_tech = val;
                }
            } else if line.contains("资金面") && line.contains("分") {
                if let Some(val) = self.extract_score(line) {
                    score_capital = val;
                }
            } else if line.contains("外部环境") && line.contains("分") {
                if let Some(val) = self.extract_score(line) {
                    score_external = val;
                }
            } else if line.contains("政策/新闻") && line.contains("分") {
                if let Some(val) = self.extract_score(line) {
                    score_news = val;
                }
            } else if line.contains("市场情绪") && line.contains("分") {
                if let Some(val) = self.extract_score(line) {
                    score_sentiment = val;
                }
            } else if line.contains("综合得分") && line.contains("分") {
                if let Some(val) = self.extract_score(line) {
                    total_score = val;
                }
                if line.contains("偏多") {
                    overall_tendency = "偏多".to_string();
                } else if line.contains("偏空") {
                    overall_tendency = "偏空".to_string();
                } else if line.contains("强势") {
                    overall_tendency = "强势".to_string();
                }
            }
        }

        (score_volume, score_tech, score_capital, score_external, score_news, score_sentiment, total_score, overall_tendency)
    }

    fn extract_score(&self, line: &str) -> Option<i32> {
        // 从行中提取分数
        let chars: Vec<char> = line.chars().collect();
        let mut num_str = String::new();

        for c in chars {
            if c.is_ascii_digit() {
                num_str.push(c);
            }
        }

        num_str.parse().ok().filter(|&n| n > 0 && n <= 100)
    }

    fn parse_judgment_section(&self, response: &str) -> (String, i32, i32, String, String, i32, String, String, i32, String, String, String) {
        let core_judgment = self.extract_between(response, "核心判断", "（置信度")
            .or_else(|| Some("震荡".to_string()))
            .unwrap();

        let confidence = self.extract_number_after(response, "置信度：").unwrap_or(60);
        let bull_probability = self.extract_number_after(response, "多头情景").unwrap_or(35);
        let bear_probability = self.extract_number_after(response, "空头情景").unwrap_or(25);
        let base_probability = 100 - bull_probability - bear_probability;

        let bull_target = self.extract_between(response, "目标区间", "重点关注").unwrap_or("上证 XX-XX 点".to_string());
        let bull_focus = self.extract_after(response, "重点关注板块").unwrap_or("待确认".to_string());
        let bear_target = self.extract_between(response, "空头情景", "需警惕").unwrap_or("上证 XX-XX 点".to_string());
        let bear_risk = self.extract_after(response, "需警惕的风险").unwrap_or("待观察".to_string());
        let base_description = self.extract_between(response, "基准情景", "核心支撑").unwrap_or("震荡为主".to_string());
        let base_support = self.extract_after(response, "核心支撑").unwrap_or("XX 点".to_string());
        let base_resistance = self.extract_after(response, "核心压力").unwrap_or("XX 点".to_string());

        (core_judgment, confidence, bull_probability, bull_target, bull_focus, bear_probability, bear_target, bear_risk, base_probability, base_description, base_support, base_resistance)
    }

    fn parse_sector_section(&self, response: &str) -> (Vec<SectorOpportunity>, Vec<SectorRisk>) {
        let mut opportunities = Vec::new();
        let mut risks = Vec::new();

        let mut in_opportunity = false;
        let mut in_risk = false;

        for line in response.lines() {
            let line = line.trim();

            if line.contains("明日值得关注的板块") {
                in_opportunity = true;
                in_risk = false;
            } else if line.contains("明日需回避的板块") {
                in_opportunity = false;
                in_risk = true;
            } else if line.contains("##") || line.contains("### 四") || line.contains("### 五") {
                in_opportunity = false;
                in_risk = false;
            } else if in_opportunity && line.starts_with(|c: char| c.is_ascii_digit() || c == '[') {
                if line.len() > 3 {
                    opportunities.push(SectorOpportunity {
                        sector: format!("板块{}", opportunities.len() + 1),
                        reason: line[line.find("】").map(|i| i+1).unwrap_or(3)..].trim().to_string(),
                    });
                }
            } else if in_risk && line.starts_with(|c: char| c.is_ascii_digit() || c == '[') {
                if line.len() > 3 {
                    risks.push(SectorRisk {
                        sector: format!("板块{}", risks.len() + 1),
                        reason: line[line.find("】").map(|i| i+1).unwrap_or(3)..].trim().to_string(),
                    });
                }
            }
        }

        (opportunities, risks)
    }

    fn parse_action_section(&self, response: &str) -> (i32, i32, i32, String, String, String) {
        let position_aggressive = 50;
        let position_steady = 30;
        let position_conservative = 20;
        let short_term_trader = "快进快出，关注强势板块".to_string();
        let swing_trader = "控制仓位，高抛低吸".to_string();
        let position_holder = "持有为主，谨慎加仓".to_string();

        (position_aggressive, position_steady, position_conservative, short_term_trader, swing_trader, position_holder)
    }

    fn parse_risk_section(&self, response: &str) -> (String, String, String) {
        let risk_high = "关注外部市场波动风险".to_string();
        let risk_medium = "注意板块轮动风险".to_string();
        let risk_low = "关注成交量变化".to_string();

        let mut in_risk = false;
        let mut risks = Vec::new();

        for line in response.lines() {
            let line = line.trim();

            if line.contains("关键风险提示") {
                in_risk = true;
            } else if in_risk && line.starts_with(|c: char| c.is_ascii_digit()) {
                if let Some(pos) = line.find('：') {
                    risks.push(line[pos+3..].trim().to_string());
                }
            } else if in_risk && (line.contains("##") || line.contains("### 六")) {
                in_risk = false;
            }
        }

        (
            risks.get(0).cloned().unwrap_or(risk_high),
            risks.get(1).cloned().unwrap_or(risk_medium),
            risks.get(2).cloned().unwrap_or(risk_low),
        )
    }

    fn parse_observation_section(&self, response: &str) -> Vec<String> {
        let mut indicators = Vec::new();
        let mut in_observation = false;

        for line in response.lines() {
            let line = line.trim();

            if line.contains("明日重点观察指标") {
                in_observation = true;
            } else if in_observation && line.contains("[ ]") {
                indicators.push(line.replace("[ ]", "").trim().to_string());
            } else if in_observation && (line.contains("##") || line.starts_with("**分析")) {
                in_observation = false;
            }
        }

        if indicators.is_empty() {
            indicators = vec![
                "集合竞价涨跌幅是否超过 ±0.5%".to_string(),
                "北向资金开盘方向".to_string(),
                "昨日强势板块是否延续".to_string(),
            ];
        }

        indicators
    }

    fn extract_between(&self, text: &str, start: &str, end: &str) -> Option<String> {
        let start_idx = text.find(start)?;
        let end_idx = text.find(end)?;

        if start_idx < end_idx {
            Some(text[start_idx + start.len()..end_idx].trim().to_string())
        } else {
            None
        }
    }

    fn extract_after(&self, text: &str, marker: &str) -> Option<String> {
        let idx = text.find(marker)?;
        let after = &text[idx + marker.len()..];
        let end_idx = after.find('\n').unwrap_or(after.len());
        Some(after[..end_idx].trim().to_string())
    }

    fn extract_number_after(&self, text: &str, marker: &str) -> Option<i32> {
        let after = self.extract_after(text, marker)?;
        let chars: Vec<char> = after.chars().collect();
        let mut num_str = String::new();

        for c in chars {
            if c.is_ascii_digit() {
                num_str.push(c);
            } else if !num_str.is_empty() {
                break;
            }
        }

        num_str.parse().ok()
    }

    async fn save_prediction(&self, prediction: &MarketPredictionDetail) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

        let sector_json = serde_json::to_string(&prediction.sector_opportunities)
            .unwrap_or_else(|_| "[]".to_string());
        let risks_json = serde_json::to_string(&prediction.sector_risks)
            .unwrap_or_else(|_| "[]".to_string());
        let indicators_json = serde_json::to_string(&prediction.observation_indicators)
            .unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            r#"INSERT OR REPLACE INTO market_prediction
               (date, prediction_date, market_outlook, market_outlook_detail, key_factors,
                sector_recommendations, risk_factors, hot_stocks, ai_insight, confidence_score, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
            params![
                prediction.date,
                prediction.prediction_date,
                format!("{}（置信度{}%）", prediction.core_judgment, prediction.confidence),
                prediction.ai_insight,
                format!("综合得分{}分，整体{}", prediction.total_score, prediction.overall_tendency),
                sector_json,
                format!("高风险:{}; 中风险:{}; 低风险:{}", prediction.risk_high, prediction.risk_medium, prediction.risk_low),
                risks_json,
                prediction.ai_insight,
                prediction.confidence_score,
                prediction.created_at,
            ],
        )?;

        Ok(())
    }

    /// 获取最新预测
    pub fn get_latest_prediction(&self) -> Result<Option<MarketPredictionDetail>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT id, date, prediction_date, market_outlook, market_outlook_detail, key_factors,
                    sector_recommendations, risk_factors, hot_stocks, ai_insight, confidence_score, created_at
             FROM market_prediction
             ORDER BY date DESC
             LIMIT 1"
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_prediction(row)?))
        } else {
            Ok(None)
        }
    }

    /// 获取预测历史
    pub fn get_prediction_history(&self, limit: usize) -> Result<Vec<MarketPredictionDetail>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT id, date, prediction_date, market_outlook, market_outlook_detail, key_factors,
                    sector_recommendations, risk_factors, hot_stocks, ai_insight, confidence_score, created_at
             FROM market_prediction
             ORDER BY date DESC
             LIMIT ?1"
        )?;

        let mut rows = stmt.query([limit as i64])?;
        let mut predictions = Vec::new();

        while let Some(row) = rows.next()? {
            predictions.push(self.row_to_prediction(row)?);
        }

        Ok(predictions)
    }

    fn row_to_prediction(&self, row: &rusqlite::Row) -> Result<MarketPredictionDetail> {
        let sector_json: String = row.get(6)?;
        let risks_json: String = row.get(8)?;

        let sector_opportunities: Vec<SectorOpportunity> = serde_json::from_str(&sector_json)
            .unwrap_or_default();
        let sector_risks: Vec<SectorRisk> = serde_json::from_str(&risks_json)
            .unwrap_or_default();

        Ok(MarketPredictionDetail {
            id: row.get(0)?,
            date: row.get(1)?,
            prediction_date: row.get(2)?,
            score_volume: 65,
            score_tech: 65,
            score_capital: 65,
            score_external: 65,
            score_news: 65,
            score_sentiment: 65,
            total_score: 65,
            overall_tendency: "震荡".to_string(),
            core_judgment: row.get(3)?,
            confidence: 60,
            bull_probability: 35,
            bull_target: "待分析".to_string(),
            bull_focus: "待分析".to_string(),
            bear_probability: 25,
            bear_target: "待分析".to_string(),
            bear_risk: "待分析".to_string(),
            base_probability: 40,
            base_description: "待分析".to_string(),
            base_support: "待分析".to_string(),
            base_resistance: "待分析".to_string(),
            sector_opportunities,
            sector_risks,
            position_aggressive: 50,
            position_steady: 30,
            position_conservative: 20,
            action_aggressive: "偏向强势板块".to_string(),
            action_steady: "均衡配置".to_string(),
            action_conservative: "防御为主".to_string(),
            short_term_trader: "快进快出".to_string(),
            swing_trader: "控制仓位".to_string(),
            position_holder: "持有为主".to_string(),
            risk_high: "关注外部风险".to_string(),
            risk_medium: "注意板块轮动".to_string(),
            risk_low: "关注成交量".to_string(),
            observation_indicators: vec![],
            ai_insight: row.get(9)?,
            confidence_score: row.get(10)?,
            created_at: row.get(11)?,
            input_data: None,
        })
    }
}
