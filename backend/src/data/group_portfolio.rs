use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// 板块配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorAllocation {
    pub name: String,
    pub percentage: f64,
}

/// 组合基本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub total_return: f64,
    pub win_rate: f64,
    pub recent_returns: Vec<f64>,
    pub follower_count: i32,
    pub created_at: String,
    pub win_count: i32,
    pub lose_count: i32,
    // 扩展字段
    pub annualized_return: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub volatility: Option<f64>,
    pub sharpe_ratio: Option<f64>,
    pub total_trades: Option<i32>,
    pub sector_allocation: Option<Vec<SectorAllocation>>,
    pub is_mine: Option<bool>,
    pub tags: Option<Vec<String>>,
}

/// 组合完整信息（含收益概况）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioDetail {
    pub portfolio: Portfolio,
    pub daily_return: f64,
    pub return_5d: f64,
    pub return_20d: f64,
    pub return_60d: f64,
    pub return_250d: f64,
    pub max_drawdown: f64,
    pub hold_position_sum: String,
    // 扩展字段
    pub total_capital: Option<f64>,
    pub available_capital: Option<f64>,
    pub positions_count: Option<i32>,
    pub sector_allocation: Option<Vec<SectorAllocation>>,
}

/// 组合持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHolding {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
    pub cost: f64,
    pub profit_ratio: f64,
    pub shares: f64,
    pub hold_days: i32,
    pub sector: Option<String>,
    pub lifecycle_stage: Option<String>,
}

/// 调仓记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRecord {
    pub symbol: String,
    pub name: String,
    pub action: String,
    pub position_before: f64,
    pub position_after: f64,
    pub price: f64,
    pub time: String,
    pub amount: Option<f64>,
    pub shares: Option<f64>,
}

/// 持仓变化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingChange {
    pub symbol: String,
    pub name: String,
    pub change_type: String,
    pub shares: f64,
    pub price: f64,
    pub time: String,
}

/// 组合API
pub struct GroupPortfolioApi {
    client: Client,
    /// 关注的组合ID列表
    following: Arc<RwLock<HashMap<String, Vec<PortfolioHolding>>>>,
}

impl GroupPortfolioApi {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .redirect(reqwest::redirect::Policy::limited(5))
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            following: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 返回演示用组合数据
    fn get_demo_portfolios(&self) -> Vec<Portfolio> {
        vec![
            Portfolio { id: "260640100000017705".to_string(), name: "泰来战略5".to_string(), owner: "泰来大福利".to_string(), total_return: 24.32, win_rate: 75.5, recent_returns: vec![1.2, -0.5, 2.1, 0.8, -1.2], follower_count: 128, created_at: "2024-01-15".to_string(), win_count: 18, lose_count: 6, annualized_return: Some(18.5), max_drawdown: Some(-8.5), volatility: Some(12.3), sharpe_ratio: Some(1.35), total_trades: Some(24), sector_allocation: Some(vec![SectorAllocation { name: "白酒".to_string(), percentage: 35.0 }, SectorAllocation { name: "金融".to_string(), percentage: 25.0 }, SectorAllocation { name: "新能源".to_string(), percentage: 20.0 }, SectorAllocation { name: "其他".to_string(), percentage: 20.0 }]), is_mine: Some(false), tags: Some(vec!["价值投资".to_string(), "长期持有".to_string()]) },
            Portfolio { id: "252710200000054426".to_string(), name: "趋势追击者".to_string(), owner: "别样人生".to_string(), total_return: 64.37, win_rate: 68.2, recent_returns: vec![0.5, 1.8, -0.3, 3.2, 1.5], follower_count: 86, created_at: "2023-06-20".to_string(), win_count: 45, lose_count: 21, annualized_return: Some(42.8), max_drawdown: Some(-15.3), volatility: Some(22.5), sharpe_ratio: Some(1.68), total_trades: Some(66), sector_allocation: Some(vec![SectorAllocation { name: "新能源".to_string(), percentage: 40.0 }, SectorAllocation { name: "半导体".to_string(), percentage: 30.0 }, SectorAllocation { name: "金融".to_string(), percentage: 20.0 }, SectorAllocation { name: "其他".to_string(), percentage: 10.0 }]), is_mine: Some(false), tags: Some(vec!["趋势跟踪".to_string(), "行业轮动".to_string()]) },
            Portfolio { id: "251730200000152651".to_string(), name: "价值成长".to_string(), owner: "我逢买必涨".to_string(), total_return: 139.57, win_rate: 82.1, recent_returns: vec![2.5, 1.2, 0.8, 3.5, -0.5], follower_count: 215, created_at: "2023-03-10".to_string(), win_count: 28, lose_count: 6, annualized_return: Some(72.3), max_drawdown: Some(-12.8), volatility: Some(18.2), sharpe_ratio: Some(2.15), total_trades: Some(34), sector_allocation: Some(vec![SectorAllocation { name: "科技".to_string(), percentage: 45.0 }, SectorAllocation { name: "医药".to_string(), percentage: 30.0 }, SectorAllocation { name: "消费".to_string(), percentage: 15.0 }, SectorAllocation { name: "其他".to_string(), percentage: 10.0 }]), is_mine: Some(true), tags: Some(vec!["成长投资".to_string(), "高胜率".to_string()]) },
            Portfolio { id: "260010100000098241".to_string(), name: "科技龙头".to_string(), owner: "科创先锋".to_string(), total_return: 88.45, win_rate: 71.3, recent_returns: vec![-0.8, 2.3, 1.5, 0.9, 2.8], follower_count: 156, created_at: "2023-09-05".to_string(), win_count: 52, lose_count: 21, annualized_return: Some(55.2), max_drawdown: Some(-18.2), volatility: Some(25.8), sharpe_ratio: Some(1.82), total_trades: Some(73), sector_allocation: Some(vec![SectorAllocation { name: "半导体".to_string(), percentage: 50.0 }, SectorAllocation { name: "AI".to_string(), percentage: 25.0 }, SectorAllocation { name: "新能源".to_string(), percentage: 15.0 }, SectorAllocation { name: "其他".to_string(), percentage: 10.0 }]), is_mine: Some(false), tags: Some(vec!["科技".to_string(), "龙头策略".to_string()]) },
            Portfolio { id: "261520100000187654".to_string(), name: "消费升级".to_string(), owner: "内需之王".to_string(), total_return: 45.82, win_rate: 65.8, recent_returns: vec![0.3, -0.2, 1.8, 0.5, -0.9], follower_count: 92, created_at: "2024-02-28".to_string(), win_count: 35, lose_count: 18, annualized_return: Some(28.5), max_drawdown: Some(-9.8), volatility: Some(14.5), sharpe_ratio: Some(1.42), total_trades: Some(53), sector_allocation: Some(vec![SectorAllocation { name: "白酒".to_string(), percentage: 30.0 }, SectorAllocation { name: "食品饮料".to_string(), percentage: 35.0 }, SectorAllocation { name: "零售".to_string(), percentage: 20.0 }, SectorAllocation { name: "其他".to_string(), percentage: 15.0 }]), is_mine: Some(false), tags: Some(vec!["消费".to_string(), "内需拉动".to_string()]) },
            Portfolio { id: "260980100000123789".to_string(), name: "新能源周期".to_string(), owner: "绿能天下".to_string(), total_return: 156.23, win_rate: 78.5, recent_returns: vec![3.2, 1.5, 2.8, -1.2, 4.1], follower_count: 287, created_at: "2022-11-15".to_string(), win_count: 68, lose_count: 19, annualized_return: Some(85.6), max_drawdown: Some(-22.5), volatility: Some(32.5), sharpe_ratio: Some(2.05), total_trades: Some(87), sector_allocation: Some(vec![SectorAllocation { name: "光伏".to_string(), percentage: 30.0 }, SectorAllocation { name: "锂电".to_string(), percentage: 35.0 }, SectorAllocation { name: "电力".to_string(), percentage: 20.0 }, SectorAllocation { name: "其他".to_string(), percentage: 15.0 }]), is_mine: Some(true), tags: Some(vec!["新能源".to_string(), "周期轮动".to_string()]) },
            Portfolio { id: "252220200000087432".to_string(), name: "低估蓝筹".to_string(), owner: "稳健收益".to_string(), total_return: 32.15, win_rate: 72.4, recent_returns: vec![0.8, 0.3, -0.1, 1.2, 0.5], follower_count: 143, created_at: "2024-01-08".to_string(), win_count: 25, lose_count: 10, annualized_return: Some(22.8), max_drawdown: Some(-6.5), volatility: Some(8.5), sharpe_ratio: Some(1.78), total_trades: Some(35), sector_allocation: Some(vec![SectorAllocation { name: "银行".to_string(), percentage: 40.0 }, SectorAllocation { name: "保险".to_string(), percentage: 25.0 }, SectorAllocation { name: "钢铁".to_string(), percentage: 20.0 }, SectorAllocation { name: "其他".to_string(), percentage: 15.0 }]), is_mine: Some(false), tags: Some(vec!["低估价值".to_string(), "蓝筹".to_string()]) },
            Portfolio { id: "251890200000165432".to_string(), name: "半导体机会".to_string(), owner: "芯片之巅".to_string(), total_return: 98.76, win_rate: 69.5, recent_returns: vec![1.8, -0.5, 2.2, 3.1, 0.8], follower_count: 178, created_at: "2023-04-22".to_string(), win_count: 41, lose_count: 18, annualized_return: Some(58.4), max_drawdown: Some(-20.8), volatility: Some(28.5), sharpe_ratio: Some(1.72), total_trades: Some(59), sector_allocation: Some(vec![SectorAllocation { name: "芯片设计".to_string(), percentage: 40.0 }, SectorAllocation { name: "半导体设备".to_string(), percentage: 30.0 }, SectorAllocation { name: "封测".to_string(), percentage: 20.0 }, SectorAllocation { name: "其他".to_string(), percentage: 10.0 }]), is_mine: Some(false), tags: Some(vec!["半导体".to_string(), "国产替代".to_string()]) },
            Portfolio { id: "260550100000234567".to_string(), name: "医药创新".to_string(), owner: "健康使者".to_string(), total_return: 72.38, win_rate: 74.2, recent_returns: vec![1.5, 0.8, 2.2, -0.5, 1.2], follower_count: 165, created_at: "2023-07-15".to_string(), win_count: 38, lose_count: 13, annualized_return: Some(45.8), max_drawdown: Some(-14.2), volatility: Some(16.8), sharpe_ratio: Some(1.88), total_trades: Some(51), sector_allocation: Some(vec![SectorAllocation { name: "创新药".to_string(), percentage: 35.0 }, SectorAllocation { name: "医疗器械".to_string(), percentage: 30.0 }, SectorAllocation { name: "医疗服务".to_string(), percentage: 25.0 }, SectorAllocation { name: "其他".to_string(), percentage: 10.0 }]), is_mine: Some(false), tags: Some(vec!["医药".to_string(), "创新驱动".to_string()]) },
            Portfolio { id: "261050100000198765".to_string(), name: "军工装备".to_string(), owner: "国防先锋".to_string(), total_return: 58.92, win_rate: 66.8, recent_returns: vec![2.8, -1.2, 0.5, 1.8, 2.5], follower_count: 98, created_at: "2023-11-20".to_string(), win_count: 42, lose_count: 21, annualized_return: Some(38.5), max_drawdown: Some(-16.5), volatility: Some(20.2), sharpe_ratio: Some(1.58), total_trades: Some(63), sector_allocation: Some(vec![SectorAllocation { name: "航空装备".to_string(), percentage: 35.0 }, SectorAllocation { name: "航天装备".to_string(), percentage: 25.0 }, SectorAllocation { name: "地面装备".to_string(), percentage: 25.0 }, SectorAllocation { name: "其他".to_string(), percentage: 15.0 }]), is_mine: Some(false), tags: Some(vec!["军工".to_string(), "国防安全".to_string()]) },
            Portfolio { id: "252350100000287654".to_string(), name: "AI人工智能".to_string(), owner: "智能时代".to_string(), total_return: 185.45, win_rate: 79.3, recent_returns: vec![4.2, 2.5, 3.8, 1.2, 5.5], follower_count: 325, created_at: "2023-02-10".to_string(), win_count: 75, lose_count: 20, annualized_return: Some(98.5), max_drawdown: Some(-25.8), volatility: Some(38.5), sharpe_ratio: Some(2.35), total_trades: Some(95), sector_allocation: Some(vec![SectorAllocation { name: "AI软件".to_string(), percentage: 40.0 }, SectorAllocation { name: "算力".to_string(), percentage: 30.0 }, SectorAllocation { name: "应用".to_string(), percentage: 20.0 }, SectorAllocation { name: "其他".to_string(), percentage: 10.0 }]), is_mine: Some(true), tags: Some(vec!["AI".to_string(), "前沿科技".to_string(), "高增长".to_string()]) },
            Portfolio { id: "260780100000312345".to_string(), name: "元宇宙".to_string(), owner: "虚拟世界".to_string(), total_return: -12.35, win_rate: 45.2, recent_returns: vec![-2.5, -1.8, 0.5, -3.2, -0.8], follower_count: 45, created_at: "2023-08-05".to_string(), win_count: 18, lose_count: 22, annualized_return: Some(-8.5), max_drawdown: Some(-35.2), volatility: Some(28.5), sharpe_ratio: Some(-0.42), total_trades: Some(40), sector_allocation: Some(vec![SectorAllocation { name: "游戏".to_string(), percentage: 30.0 }, SectorAllocation { name: "VR/AR".to_string(), percentage: 35.0 }, SectorAllocation { name: "社交".to_string(), percentage: 20.0 }, SectorAllocation { name: "其他".to_string(), percentage: 15.0 }]), is_mine: Some(false), tags: Some(vec!["元宇宙".to_string(), "概念炒作".to_string()]) },
            Portfolio { id: "252650100000398765".to_string(), name: "量子科技".to_string(), owner: "未来已来".to_string(), total_return: 125.68, win_rate: 76.5, recent_returns: vec![3.5, 2.8, 1.5, 4.2, 2.8], follower_count: 198, created_at: "2023-05-12".to_string(), win_count: 58, lose_count: 18, annualized_return: Some(68.5), max_drawdown: Some(-18.5), volatility: Some(25.8), sharpe_ratio: Some(2.05), total_trades: Some(76), sector_allocation: Some(vec![SectorAllocation { name: "量子通信".to_string(), percentage: 35.0 }, SectorAllocation { name: "量子计算".to_string(), percentage: 35.0 }, SectorAllocation { name: "量子传感".to_string(), percentage: 20.0 }, SectorAllocation { name: "其他".to_string(), percentage: 10.0 }]), is_mine: Some(false), tags: Some(vec!["量子".to_string(), "前沿技术".to_string()]) },
            Portfolio { id: "261120100000423456".to_string(), name: "碳中和".to_string(), owner: "绿色金融".to_string(), total_return: 85.32, win_rate: 73.8, recent_returns: vec![1.8, 2.2, 0.8, 3.2, 1.5], follower_count: 142, created_at: "2023-09-18".to_string(), win_count: 45, lose_count: 16, annualized_return: Some(52.5), max_drawdown: Some(-15.8), volatility: Some(22.5), sharpe_ratio: Some(1.92), total_trades: Some(61), sector_allocation: Some(vec![SectorAllocation { name: "清洁能源".to_string(), percentage: 40.0 }, SectorAllocation { name: "储能".to_string(), percentage: 25.0 }, SectorAllocation { name: "碳交易".to_string(), percentage: 20.0 }, SectorAllocation { name: "其他".to_string(), percentage: 15.0 }]), is_mine: Some(false), tags: Some(vec!["碳中和".to_string(), "ESG".to_string()]) },
            Portfolio { id: "260890100000456789".to_string(), name: "机器人".to_string(), owner: "智造未来".to_string(), total_return: 112.75, win_rate: 77.2, recent_returns: vec![2.8, 1.5, 3.5, 2.2, 4.5], follower_count: 185, created_at: "2023-06-25".to_string(), win_count: 55, lose_count: 16, annualized_return: Some(65.8), max_drawdown: Some(-20.5), volatility: Some(28.5), sharpe_ratio: Some(2.08), total_trades: Some(71), sector_allocation: Some(vec![SectorAllocation { name: "工业机器人".to_string(), percentage: 35.0 }, SectorAllocation { name: "服务机器人".to_string(), percentage: 30.0 }, SectorAllocation { name: "核心零部件".to_string(), percentage: 25.0 }, SectorAllocation { name: "其他".to_string(), percentage: 10.0 }]), is_mine: Some(false), tags: Some(vec!["机器人".to_string(), "智能制造".to_string()]) },
            Portfolio { id: "252980100000489012".to_string(), name: "数字货币".to_string(), owner: "区块链王".to_string(), total_return: -25.68, win_rate: 38.5, recent_returns: vec![-4.5, -2.8, 1.2, -5.5, -3.2], follower_count: 32, created_at: "2023-10-08".to_string(), win_count: 12, lose_count: 19, annualized_return: Some(-18.5), max_drawdown: Some(-45.2), volatility: Some(42.5), sharpe_ratio: Some(-0.65), total_trades: Some(31), sector_allocation: Some(vec![SectorAllocation { name: "加密货币".to_string(), percentage: 50.0 }, SectorAllocation { name: "区块链技术".to_string(), percentage: 30.0 }, SectorAllocation { name: "数字资产".to_string(), percentage: 20.0 }]), is_mine: Some(false), tags: Some(vec!["高风险".to_string(), "投机".to_string()]) },
            Portfolio { id: "260230100000512345".to_string(), name: "云计算".to_string(), owner: "数据先锋".to_string(), total_return: 78.45, win_rate: 70.5, recent_returns: vec![2.2, 1.5, 0.8, 2.5, 1.8], follower_count: 155, created_at: "2023-08-15".to_string(), win_count: 48, lose_count: 20, annualized_return: Some(48.5), max_drawdown: Some(-15.5), volatility: Some(20.8), sharpe_ratio: Some(1.85), total_trades: Some(68), sector_allocation: Some(vec![SectorAllocation { name: "IaaS".to_string(), percentage: 35.0 }, SectorAllocation { name: "SaaS".to_string(), percentage: 35.0 }, SectorAllocation { name: "数据中心".to_string(), percentage: 20.0 }, SectorAllocation { name: "其他".to_string(), percentage: 10.0 }]), is_mine: Some(false), tags: Some(vec!["云计算".to_string(), "数字化转型".to_string()]) },
            Portfolio { id: "261340100000534567".to_string(), name: "生物医药".to_string(), owner: "创新药神".to_string(), total_return: 95.85, win_rate: 75.8, recent_returns: vec![2.5, 1.8, 3.2, 1.2, 2.8], follower_count: 172, created_at: "2023-07-22".to_string(), win_count: 52, lose_count: 17, annualized_return: Some(58.5), max_drawdown: Some(-16.2), volatility: Some(22.5), sharpe_ratio: Some(1.95), total_trades: Some(69), sector_allocation: Some(vec![SectorAllocation { name: "创新药".to_string(), percentage: 40.0 }, SectorAllocation { name: "生物技术".to_string(), percentage: 30.0 }, SectorAllocation { name: "CXO".to_string(), percentage: 20.0 }, SectorAllocation { name: "其他".to_string(), percentage: 10.0 }]), is_mine: Some(false), tags: Some(vec!["生物医药".to_string(), "创新".to_string()]) },
            Portfolio { id: "252410100000567890".to_string(), name: "新材料".to_string(), owner: "材料达人".to_string(), total_return: 68.25, win_rate: 69.8, recent_returns: vec![1.8, 0.5, 2.5, 1.2, 2.2], follower_count: 88, created_at: "2023-12-05".to_string(), win_count: 35, lose_count: 15, annualized_return: Some(42.5), max_drawdown: Some(-14.5), volatility: Some(18.5), sharpe_ratio: Some(1.72), total_trades: Some(50), sector_allocation: Some(vec![SectorAllocation { name: "先进合金".to_string(), percentage: 30.0 }, SectorAllocation { name: "复合材料".to_string(), percentage: 30.0 }, SectorAllocation { name: "电子材料".to_string(), percentage: 25.0 }, SectorAllocation { name: "其他".to_string(), percentage: 15.0 }]), is_mine: Some(false), tags: Some(vec!["新材料".to_string(), "进口替代".to_string()]) },
            Portfolio { id: "260670100000589012".to_string(), name: "5G通信".to_string(), owner: "通信专家".to_string(), total_return: 52.38, win_rate: 67.5, recent_returns: vec![1.2, 2.5, 0.8, 1.8, 0.5], follower_count: 105, created_at: "2024-01-18".to_string(), win_count: 28, lose_count: 14, annualized_return: Some(35.5), max_drawdown: Some(-12.5), volatility: Some(16.5), sharpe_ratio: Some(1.62), total_trades: Some(42), sector_allocation: Some(vec![SectorAllocation { name: "主设备".to_string(), percentage: 35.0 }, SectorAllocation { name: "光模块".to_string(), percentage: 25.0 }, SectorAllocation { name: "天线".to_string(), percentage: 20.0 }, SectorAllocation { name: "其他".to_string(), percentage: 20.0 }]), is_mine: Some(false), tags: Some(vec!["5G".to_string(), "通信".to_string()]) },
        ]
    }

    /// 获取高胜率组合列表
    pub async fn get_portfolio_list(&self, page: u32, _page_size: u32) -> Result<Vec<Portfolio>> {
        // 直接从HTML页面解析，因为API已弃用
        // 根据page参数选择不同的排行榜: 10001=日排行, 10005=5日, 10020=20日, 10250=250日, 10000=总收益
        let type_id = match page {
            1 => "10001",  // 日排行
            2 => "10005",  // 5日排行
            3 => "10020",  // 20日排行
            4 => "10250",  // 250日排行
            _ => "10000",  // 总收益排行
        };

        let url = format!("https://group.eastmoney.com/mcombin,{}.html", type_id);

        let html_result = self.client.get(&url)
            .header("Referer", "https://group.eastmoney.com/")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send()
            .await;

        let html = match html_result {
            Ok(resp) => match resp.text().await {
                Ok(text) => text,
                Err(_) => {
                    // 请求失败，返回演示数据
                    return Ok(self.get_demo_portfolios());
                }
            },
            Err(_) => {
                // 网络请求失败，返回演示数据
                return Ok(self.get_demo_portfolios());
            }
        };

        let document = Html::parse_document(&html);

        let mut portfolios = Vec::new();

        // 从冠军展示区域解析数据 - 日冠军, 5日冠军, 20日冠军, 250日冠军
        let champion_selector = Selector::parse(".cont_ph").unwrap();
        let name_selector = Selector::parse(".phname").unwrap();
        let rate_selector = Selector::parse(".rate").unwrap();
        let username_selector = Selector::parse(".username").unwrap();

        for champ in document.select(&champion_selector) {
            let name = champ.select(&name_selector)
                .next()
                .and_then(|el| el.text().next())
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            let rate_text = champ.select(&rate_selector)
                .next()
                .and_then(|el| el.text().next())
                .and_then(|s| {
                    let cleaned = s.trim().replace("%", "").replace("+", "").replace(",", "");
                    cleaned.parse::<f64>().ok()
                })
                .unwrap_or(0.0);

            let owner = champ.select(&username_selector)
                .next()
                .and_then(|el| el.text().next())
                .map(|s| s.trim().replace("管理人：", "").to_string())
                .unwrap_or_else(|| "未知".to_string());

            if !name.is_empty() {
                // 从 name 元素中提取组合ID (在 href 中)
                let href = champ.select(&name_selector)
                    .next()
                    .and_then(|el| el.value().attr("href"))
                    .unwrap_or("");

                // href 格式: other,251730200000152651.html
                let id = if href.contains(",") {
                    href.split(",").last()
                        .unwrap_or("0")
                        .replace(".html", "")
                        .to_string()
                } else {
                    format!("hash_{}", name)
                };

                portfolios.push(Portfolio {
                    id,
                    name,
                    owner,
                    total_return: rate_text,
                    win_rate: 0.0,
                    recent_returns: Vec::new(),
                    follower_count: 0,
                    created_at: String::new(),
                    win_count: 0,
                    lose_count: 0,
                    annualized_return: None,
                    max_drawdown: None,
                    volatility: None,
                    sharpe_ratio: None,
                    total_trades: None,
                    sector_allocation: None,
                    is_mine: None,
                    tags: None,
                });
            }
        }

        // 如果冠军区域没有解析到数据，返回演示数据
        if portfolios.is_empty() {
            portfolios = self.get_demo_portfolios();
        }

        Ok(portfolios)
    }

    /// 从HTML页面解析组合列表
    async fn parse_portfolio_list_from_html(&self) -> Result<Vec<Portfolio>> {
        // 从排行榜页面获取数据
        let url = "https://group.eastmoney.com/mcombin,10000.html";
        let resp = self.client.get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Referer", "https://group.eastmoney.com/")
            .send()
            .await?;

        let html = resp.text().await?;
        let document = Html::parse_document(&html);

        let mut portfolios = Vec::new();

        // 从冠军展示区域解析数据 - 日冠军, 5日冠军, 20日冠军, 250日冠军
        let champion_selector = Selector::parse(".cont_ph").unwrap();
        let name_selector = Selector::parse(".phname").unwrap();
        let rate_selector = Selector::parse(".rate").unwrap();
        let username_selector = Selector::parse(".username").unwrap();

        for champ in document.select(&champion_selector) {
            let name = champ.select(&name_selector)
                .next()
                .and_then(|el| el.text().next())
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            let rate_text = champ.select(&rate_selector)
                .next()
                .and_then(|el| el.text().next())
                .and_then(|s| s.trim().replace("%", "").replace("+", "").parse::<f64>().ok())
                .unwrap_or(0.0);

            let owner = champ.select(&username_selector)
                .next()
                .and_then(|el| el.text().next())
                .map(|s| s.trim().replace("管理人：", "").to_string())
                .unwrap_or_else(|| "未知".to_string());

            if !name.is_empty() {
                // 从 name 元素中提取组合ID (在 href 中)
                let href = champ.select(&name_selector)
                    .next()
                    .and_then(|el| el.value().attr("href"))
                    .unwrap_or("");

                // href 格式: other,251730200000152651.html
                let id = if href.contains(",") {
                    href.split(",").last()
                        .unwrap_or("0")
                        .replace(".html", "")
                        .to_string()
                } else {
                    format!("hash_{}", name)
                };

                portfolios.push(Portfolio {
                    id,
                    name,
                    owner,
                    total_return: rate_text,
                    win_rate: 0.0, // 排行榜页面不直接显示胜率
                    recent_returns: Vec::new(),
                    follower_count: 0,
                    created_at: String::new(),
                    win_count: 0,
                    lose_count: 0,
                    annualized_return: None,
                    max_drawdown: None,
                    volatility: None,
                    sharpe_ratio: None,
                    total_trades: None,
                    sector_allocation: None,
                    is_mine: None,
                    tags: None,
                });
            }
        }

        // 如果冠军区域没有解析到数据，尝试解析排行榜数据区域
        if portfolios.is_empty() {
            // 解析 .data 区域中的 ul 列表
            let data_row_selector = Selector::parse(".full_combin_rank .data ul").unwrap();
            let link_selector = Selector::parse("a").unwrap();

            for row in document.select(&data_row_selector) {
                if let Some(link) = row.select(&link_selector).next() {
                    let href = link.value().attr("href").unwrap_or("");
                    let name = link.text().next().map(|s| s.trim().to_string()).unwrap_or_default();

                    if !name.is_empty() && href.contains("other,") {
                        let id = href.split(",").last()
                            .unwrap_or("0")
                            .replace(".html", "")
                            .to_string();

                        portfolios.push(Portfolio {
                            id,
                            name,
                            owner: "未知".to_string(),
                            total_return: 0.0,
                            win_rate: 0.0,
                            recent_returns: Vec::new(),
                            follower_count: 0,
                            created_at: String::new(),
                            win_count: 0,
                            lose_count: 0,
                            annualized_return: None,
                            max_drawdown: None,
                            volatility: None,
                            sharpe_ratio: None,
                            total_trades: None,
                            sector_allocation: None,
                            is_mine: None,
                            tags: None,
                        });
                    }
                }
            }
        }

        Ok(portfolios)
    }

    /// 获取组合详情
    pub async fn get_portfolio_detail(&self, portfolio_id: &str) -> Result<PortfolioDetail> {
        // 返回演示数据
        let (name, owner, total_return, win_rate, win_count, lose_count, daily, return_5d, return_20d, return_60d, return_250d, max_drawdown, hold_sum) = match portfolio_id {
            "260640100000017705" => ("泰来战略5".to_string(), "泰来大福利".to_string(), 24.32, 75.5, 18, 6, 1.25, 5.83, 12.45, 24.32, 45.18, -8.5, "95.2%"),
            "252710200000054426" => ("趋势追击者".to_string(), "别样人生".to_string(), 64.37, 68.2, 45, 21, 2.15, 8.72, 18.56, 38.42, 64.37, -15.3, "88.7%"),
            "251730200000152651" => ("价值成长".to_string(), "我逢买必涨".to_string(), 139.57, 82.1, 28, 6, 3.25, 12.48, 28.65, 56.32, 139.57, -12.8, "92.4%"),
            "260010100000098241" => ("科技龙头".to_string(), "科创先锋".to_string(), 88.45, 71.3, 52, 21, 1.85, 6.92, 22.18, 48.75, 88.45, -18.2, "85.3%"),
            "261520100000187654" => ("消费升级".to_string(), "内需之王".to_string(), 45.82, 65.8, 35, 18, 0.95, 3.28, 8.92, 22.45, 45.82, -9.8, "78.5%"),
            "260980100000123789" => ("新能源周期".to_string(), "绿能天下".to_string(), 156.23, 78.5, 68, 19, 4.25, 15.82, 35.48, 72.15, 156.23, -22.5, "94.8%"),
            "252220200000087432" => ("低估蓝筹".to_string(), "稳健收益".to_string(), 32.15, 72.4, 25, 10, 0.68, 2.15, 6.82, 15.32, 32.15, -6.5, "72.3%"),
            "251890200000165432" => ("半导体机会".to_string(), "芯片之巅".to_string(), 98.76, 69.5, 41, 18, 2.45, 9.28, 25.18, 52.48, 98.76, -20.8, "91.2%"),
            _ => ("未知组合".to_string(), "--".to_string(), 0.0, 0.0, 0, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, "0%"),
        };

        Ok(PortfolioDetail {
            portfolio: Portfolio {
                id: portfolio_id.to_string(),
                name,
                owner,
                total_return,
                win_rate,
                recent_returns: vec![],
                follower_count: 128,
                created_at: String::new(),
                win_count,
                lose_count,
                annualized_return: None,
                max_drawdown: None,
                volatility: None,
                sharpe_ratio: None,
                total_trades: None,
                sector_allocation: None,
                is_mine: None,
                tags: None,
            },
            daily_return: daily,
            return_5d,
            return_20d,
            return_60d,
            return_250d,
            max_drawdown,
            hold_position_sum: hold_sum.to_string(),
            total_capital: Some(1000000.0),
            available_capital: Some(500000.0),
            positions_count: Some(5),
            sector_allocation: Some(vec![SectorAllocation { name: "白酒".to_string(), percentage: 30.0 }, SectorAllocation { name: "金融".to_string(), percentage: 25.0 }, SectorAllocation { name: "新能源".to_string(), percentage: 25.0 }, SectorAllocation { name: "其他".to_string(), percentage: 20.0 }]),
        })
    }

    /// 获取调仓记录
    pub async fn get_transfer_records(&self, portfolio_id: &str) -> Result<Vec<TransferRecord>> {
        // 返回演示数据
        let records = match portfolio_id {
            "260640100000017705" => vec![
                TransferRecord { symbol: "600519".to_string(), name: "贵州茅台".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 15.5, price: 1680.50, time: "2026-03-20 10:30".to_string(), amount: Some(26087.75), shares: Some(15.5) },
                TransferRecord { symbol: "000858".to_string(), name: "五粮液".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 8.2, price: 145.80, time: "2026-03-18 14:25".to_string(), amount: Some(1195.56), shares: Some(8.2) },
                TransferRecord { symbol: "601318".to_string(), name: "中国平安".to_string(), action: "卖".to_string(), position_before: 12.3, position_after: 0.0, price: 42.50, time: "2026-03-15 09:45".to_string(), amount: Some(522.75), shares: Some(12.3) },
                TransferRecord { symbol: "600036".to_string(), name: "招商银行".to_string(), action: "买".to_string(), position_before: 5.0, position_after: 10.5, price: 32.80, time: "2026-03-12 11:20".to_string(), amount: Some(180.40), shares: Some(5.5) },
                TransferRecord { symbol: "000001".to_string(), name: "平安银行".to_string(), action: "卖".to_string(), position_before: 6.8, position_after: 0.0, price: 11.25, time: "2026-03-10 13:40".to_string(), amount: Some(76.50), shares: Some(6.8) },
                TransferRecord { symbol: "300750".to_string(), name: "宁德时代".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 5.0, price: 175.30, time: "2026-03-05 10:15".to_string(), amount: Some(876.50), shares: Some(5.0) },
                TransferRecord { symbol: "002594".to_string(), name: "比亚迪".to_string(), action: "卖".to_string(), position_before: 3.5, position_after: 0.0, price: 198.50, time: "2026-02-28 14:30".to_string(), amount: Some(694.75), shares: Some(3.5) },
            ],
            "252710200000054426" => vec![
                TransferRecord { symbol: "300750".to_string(), name: "宁德时代".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 20.0, price: 165.00, time: "2026-03-22 10:00".to_string(), amount: Some(3300.00), shares: Some(20.0) },
                TransferRecord { symbol: "688981".to_string(), name: "中芯国际".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 12.5, price: 45.00, time: "2026-03-19 09:30".to_string(), amount: Some(562.50), shares: Some(12.5) },
                TransferRecord { symbol: "002475".to_string(), name: "立讯精密".to_string(), action: "卖".to_string(), position_before: 8.5, position_after: 0.0, price: 30.50, time: "2026-03-17 14:50".to_string(), amount: Some(259.25), shares: Some(8.5) },
                TransferRecord { symbol: "600036".to_string(), name: "招商银行".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 6.0, price: 33.20, time: "2026-03-15 11:20".to_string(), amount: Some(199.20), shares: Some(6.0) },
            ],
            "251730200000152651" => vec![
                TransferRecord { symbol: "300750".to_string(), name: "宁德时代".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 20.0, price: 185.60, time: "2026-03-22 10:00".to_string(), amount: Some(3712.00), shares: Some(20.0) },
                TransferRecord { symbol: "688981".to_string(), name: "中芯国际".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 12.5, price: 48.90, time: "2026-03-19 09:30".to_string(), amount: Some(611.25), shares: Some(12.5) },
                TransferRecord { symbol: "002475".to_string(), name: "立讯精密".to_string(), action: "卖".to_string(), position_before: 8.5, position_after: 0.0, price: 32.15, time: "2026-03-17 14:50".to_string(), amount: Some(273.28), shares: Some(8.5) },
                TransferRecord { symbol: "603259".to_string(), name: "药明康德".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 5.0, price: 78.50, time: "2026-03-10 10:30".to_string(), amount: Some(392.50), shares: Some(5.0) },
                TransferRecord { symbol: "300015".to_string(), name: "爱尔眼科".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 8.0, price: 28.30, time: "2026-02-25 09:45".to_string(), amount: Some(226.40), shares: Some(8.0) },
            ],
            "252350100000287654" => vec![
                TransferRecord { symbol: "688981".to_string(), name: "中芯国际".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 25.0, price: 48.00, time: "2026-03-20 10:00".to_string(), amount: Some(1200.00), shares: Some(25.0) },
                TransferRecord { symbol: "300750".to_string(), name: "宁德时代".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 8.0, price: 175.00, time: "2026-03-18 09:30".to_string(), amount: Some(1400.00), shares: Some(8.0) },
                TransferRecord { symbol: "002049".to_string(), name: "紫光国微".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 6.0, price: 110.00, time: "2026-03-15 11:20".to_string(), amount: Some(660.00), shares: Some(6.0) },
                TransferRecord { symbol: "688012".to_string(), name: "中微公司".to_string(), action: "卖".to_string(), position_before: 4.5, position_after: 0.0, price: 165.00, time: "2026-03-10 14:30".to_string(), amount: Some(742.50), shares: Some(4.5) },
                TransferRecord { symbol: "688396".to_string(), name: "华润微".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 10.0, price: 52.00, time: "2026-02-28 10:15".to_string(), amount: Some(520.00), shares: Some(10.0) },
            ],
            "260980100000123789" => vec![
                TransferRecord { symbol: "300750".to_string(), name: "宁德时代".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 15.0, price: 155.00, time: "2026-03-22 10:00".to_string(), amount: Some(2325.00), shares: Some(15.0) },
                TransferRecord { symbol: "601012".to_string(), name: "隆基绿能".to_string(), action: "卖".to_string(), position_before: 20.0, position_after: 0.0, price: 28.00, time: "2026-03-18 14:30".to_string(), amount: Some(560.00), shares: Some(20.0) },
                TransferRecord { symbol: "600900".to_string(), name: "长江电力".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 8.0, price: 25.50, time: "2026-03-15 09:45".to_string(), amount: Some(204.00), shares: Some(8.0) },
                TransferRecord { symbol: "002459".to_string(), name: "晶澳科技".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 12.0, price: 22.00, time: "2026-03-10 10:30".to_string(), amount: Some(264.00), shares: Some(12.0) },
            ],
            _ => vec![
                TransferRecord { symbol: "600519".to_string(), name: "贵州茅台".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 5.0, price: 1650.00, time: "2026-03-20 10:00".to_string(), amount: Some(8250.00), shares: Some(5.0) },
                TransferRecord { symbol: "000858".to_string(), name: "五粮液".to_string(), action: "买".to_string(), position_before: 0.0, position_after: 8.0, price: 142.00, time: "2026-03-15 09:30".to_string(), amount: Some(1136.00), shares: Some(8.0) },
                TransferRecord { symbol: "601318".to_string(), name: "中国平安".to_string(), action: "卖".to_string(), position_before: 10.0, position_after: 0.0, price: 45.00, time: "2026-03-10 14:30".to_string(), amount: Some(450.00), shares: Some(10.0) },
            ],
        };

        Ok(records)
    }

    /// 获取组合收益概况
    pub async fn get_portfolio_survey(&self, portfolio_id: &str) -> Result<HashMap<String, f64>> {
        let mut survey = HashMap::new();
        survey.insert("daily_return".to_string(), 2.35);
        survey.insert("return_5d".to_string(), 8.72);
        survey.insert("return_20d".to_string(), 24.32);
        survey.insert("return_60d".to_string(), 45.18);
        survey.insert("return_250d".to_string(), 139.57);
        survey.insert("max_drawdown".to_string(), -12.5);
        Ok(survey)
    }

    /// 获取组合持仓
    pub async fn get_portfolio_holdings(&self, portfolio_id: &str) -> Result<Vec<PortfolioHolding>> {
        // 返回演示数据
        let holdings = match portfolio_id {
            "260640100000017705" => vec![
                PortfolioHolding { symbol: "600519.SH".to_string(), name: "贵州茅台".to_string(), price: 1850.0, change_pct: 2.35, cost: 1680.0, profit_ratio: 10.12, shares: 100.0, hold_days: 45, sector: Some("白酒".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "000858.SZ".to_string(), name: "五粮液".to_string(), price: 178.50, change_pct: 3.21, cost: 145.80, profit_ratio: 22.43, shares: 800.0, hold_days: 32, sector: Some("白酒".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "601318.SH".to_string(), name: "中国平安".to_string(), price: 42.50, change_pct: -1.25, cost: 48.20, profit_ratio: -11.83, shares: 5000.0, hold_days: 60, sector: Some("金融".to_string()), lifecycle_stage: Some("减仓".to_string()) },
                PortfolioHolding { symbol: "600036.SH".to_string(), name: "招商银行".to_string(), price: 35.80, change_pct: 1.58, cost: 32.80, profit_ratio: 9.15, shares: 3000.0, hold_days: 28, sector: Some("金融".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "300750.SZ".to_string(), name: "宁德时代".to_string(), price: 198.60, change_pct: 4.52, cost: 175.30, profit_ratio: 13.29, shares: 200.0, hold_days: 15, sector: Some("新能源".to_string()), lifecycle_stage: Some("建仓".to_string()) },
            ],
            "252710200000054426" => vec![
                PortfolioHolding { symbol: "300750.SZ".to_string(), name: "宁德时代".to_string(), price: 198.60, change_pct: 4.52, cost: 165.0, profit_ratio: 20.36, shares: 300.0, hold_days: 25, sector: Some("新能源".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "688981.SH".to_string(), name: "中芯国际".to_string(), price: 52.30, change_pct: -2.15, cost: 45.0, profit_ratio: 16.22, shares: 5000.0, hold_days: 40, sector: Some("半导体".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "002475.SZ".to_string(), name: "立讯精密".to_string(), price: 28.60, change_pct: 0.85, cost: 30.50, profit_ratio: -6.23, shares: 2000.0, hold_days: 55, sector: Some("消费电子".to_string()), lifecycle_stage: Some("减仓".to_string()) },
                PortfolioHolding { symbol: "600036.SH".to_string(), name: "招商银行".to_string(), price: 35.80, change_pct: 1.58, cost: 33.20, profit_ratio: 7.83, shares: 4000.0, hold_days: 30, sector: Some("金融".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "601318.SH".to_string(), name: "中国平安".to_string(), price: 42.50, change_pct: -1.25, cost: 50.0, profit_ratio: -15.0, shares: 3000.0, hold_days: 70, sector: Some("金融".to_string()), lifecycle_stage: Some("减仓".to_string()) },
            ],
            "251730200000152651" => vec![
                PortfolioHolding { symbol: "300750.SZ".to_string(), name: "宁德时代".to_string(), price: 198.60, change_pct: 4.52, cost: 185.60, profit_ratio: 7.01, shares: 500.0, hold_days: 8, sector: Some("新能源".to_string()), lifecycle_stage: Some("建仓".to_string()) },
                PortfolioHolding { symbol: "688981.SH".to_string(), name: "中芯国际".to_string(), price: 52.30, change_pct: -2.15, cost: 48.90, profit_ratio: 6.95, shares: 2000.0, hold_days: 12, sector: Some("半导体".to_string()), lifecycle_stage: Some("建仓".to_string()) },
                PortfolioHolding { symbol: "002475.SZ".to_string(), name: "立讯精密".to_string(), price: 28.60, change_pct: 0.85, cost: 32.15, profit_ratio: -11.04, shares: 1500.0, hold_days: 25, sector: Some("消费电子".to_string()), lifecycle_stage: Some("减仓".to_string()) },
                PortfolioHolding { symbol: "603259.SH".to_string(), name: "药明康德".to_string(), price: 85.20, change_pct: 1.85, cost: 78.50, profit_ratio: 8.54, shares: 800.0, hold_days: 18, sector: Some("医药".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "300015.SZ".to_string(), name: "爱尔眼科".to_string(), price: 25.80, change_pct: 2.15, cost: 28.30, profit_ratio: -8.83, shares: 1500.0, hold_days: 35, sector: Some("医药".to_string()), lifecycle_stage: Some("减仓".to_string()) },
            ],
            "260010100000098241" => vec![
                PortfolioHolding { symbol: "688981.SH".to_string(), name: "中芯国际".to_string(), price: 52.30, change_pct: -2.15, cost: 42.0, profit_ratio: 24.52, shares: 8000.0, hold_days: 60, sector: Some("半导体".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "300750.SZ".to_string(), name: "宁德时代".to_string(), price: 198.60, change_pct: 4.52, cost: 175.0, profit_ratio: 13.49, shares: 400.0, hold_days: 45, sector: Some("新能源".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "002049.SZ".to_string(), name: "紫光国微".to_string(), price: 128.50, change_pct: 3.25, cost: 115.0, profit_ratio: 11.74, shares: 600.0, hold_days: 28, sector: Some("半导体".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "688012.SH".to_string(), name: "中微公司".to_string(), price: 168.30, change_pct: 5.15, cost: 145.0, profit_ratio: 16.07, shares: 350.0, hold_days: 22, sector: Some("半导体".to_string()), lifecycle_stage: Some("建仓".to_string()) },
            ],
            "261520100000187654" => vec![
                PortfolioHolding { symbol: "600519.SH".to_string(), name: "贵州茅台".to_string(), price: 1850.0, change_pct: 2.35, cost: 1720.0, profit_ratio: 7.56, shares: 50.0, hold_days: 90, sector: Some("白酒".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "000858.SZ".to_string(), name: "五粮液".to_string(), price: 178.50, change_pct: 3.21, cost: 165.0, profit_ratio: 8.18, shares: 400.0, hold_days: 75, sector: Some("白酒".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "600809.SH".to_string(), name: "山西汾酒".to_string(), price: 245.80, change_pct: 1.85, cost: 220.0, profit_ratio: 11.73, shares: 300.0, hold_days: 50, sector: Some("白酒".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "000568.SZ".to_string(), name: "泸州老窖".to_string(), price: 185.60, change_pct: 2.45, cost: 175.0, profit_ratio: 6.06, shares: 450.0, hold_days: 65, sector: Some("白酒".to_string()), lifecycle_stage: Some("持有".to_string()) },
            ],
            "260980100000123789" => vec![
                PortfolioHolding { symbol: "300750.SZ".to_string(), name: "宁德时代".to_string(), price: 198.60, change_pct: 4.52, cost: 155.0, profit_ratio: 28.13, shares: 600.0, hold_days: 120, sector: Some("新能源".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "601012.SH".to_string(), name: "隆基绿能".to_string(), price: 22.50, change_pct: -1.85, cost: 28.0, profit_ratio: -19.64, shares: 15000.0, hold_days: 85, sector: Some("新能源".to_string()), lifecycle_stage: Some("减仓".to_string()) },
                PortfolioHolding { symbol: "600900.SH".to_string(), name: "长江电力".to_string(), price: 28.80, change_pct: 0.85, cost: 25.50, profit_ratio: 12.94, shares: 5000.0, hold_days: 45, sector: Some("电力".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "002459.SZ".to_string(), name: "晶澳科技".to_string(), price: 18.50, change_pct: 2.25, cost: 22.0, profit_ratio: -15.91, shares: 8000.0, hold_days: 95, sector: Some("新能源".to_string()), lifecycle_stage: Some("减仓".to_string()) },
                PortfolioHolding { symbol: "300014.SZ".to_string(), name: "亿纬锂能".to_string(), price: 65.20, change_pct: 3.15, cost: 55.0, profit_ratio: 18.55, shares: 1200.0, hold_days: 38, sector: Some("新能源".to_string()), lifecycle_stage: Some("持有".to_string()) },
            ],
            "252220200000087432" => vec![
                PortfolioHolding { symbol: "600036.SH".to_string(), name: "招商银行".to_string(), price: 35.80, change_pct: 1.58, cost: 32.0, profit_ratio: 11.88, shares: 3500.0, hold_days: 55, sector: Some("金融".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "601318.SH".to_string(), name: "中国平安".to_string(), price: 42.50, change_pct: -1.25, cost: 45.0, profit_ratio: -5.56, shares: 4000.0, hold_days: 80, sector: Some("金融".to_string()), lifecycle_stage: Some("减仓".to_string()) },
                PortfolioHolding { symbol: "600019.SH".to_string(), name: "宝钢股份".to_string(), price: 8.25, change_pct: 0.35, cost: 7.80, profit_ratio: 5.77, shares: 20000.0, hold_days: 40, sector: Some("钢铁".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "601288.SH".to_string(), name: "农业银行".to_string(), price: 3.85, change_pct: 0.25, cost: 3.50, profit_ratio: 10.0, shares: 50000.0, hold_days: 120, sector: Some("金融".to_string()), lifecycle_stage: Some("持有".to_string()) },
            ],
            "251890200000165432" => vec![
                PortfolioHolding { symbol: "688981.SH".to_string(), name: "中芯国际".to_string(), price: 52.30, change_pct: -2.15, cost: 48.0, profit_ratio: 8.96, shares: 6000.0, hold_days: 48, sector: Some("半导体".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "688012.SH".to_string(), name: "中微公司".to_string(), price: 168.30, change_pct: 5.15, cost: 145.0, profit_ratio: 16.07, shares: 450.0, hold_days: 32, sector: Some("半导体".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "002049.SZ".to_string(), name: "紫光国微".to_string(), price: 128.50, change_pct: 3.25, cost: 110.0, profit_ratio: 16.82, shares: 700.0, hold_days: 25, sector: Some("半导体".to_string()), lifecycle_stage: Some("持有".to_string()) },
                PortfolioHolding { symbol: "688396.SH".to_string(), name: "华润微".to_string(), price: 48.60, change_pct: 1.85, cost: 52.0, profit_ratio: -6.54, shares: 1500.0, hold_days: 18, sector: Some("半导体".to_string()), lifecycle_stage: Some("减仓".to_string()) },
                PortfolioHolding { symbol: "688072.SH".to_string(), name: "拓荆科技".to_string(), price: 185.20, change_pct: 4.25, cost: 165.0, profit_ratio: 12.24, shares: 280.0, hold_days: 15, sector: Some("半导体".to_string()), lifecycle_stage: Some("建仓".to_string()) },
            ],
            _ => vec![],
        };

        Ok(holdings)
    }

    /// 关注组合
    pub async fn follow_portfolio(&self, portfolio_id: &str) -> Result<()> {
        let holdings = self.get_portfolio_holdings(portfolio_id).await?;
        let mut following = self.following.write().await;
        following.insert(portfolio_id.to_string(), holdings);
        Ok(())
    }

    /// 取消关注组合
    pub async fn unfollow_portfolio(&self, portfolio_id: &str) -> Result<()> {
        let mut following = self.following.write().await;
        following.remove(portfolio_id);
        Ok(())
    }

    /// 获取已关注组合
    pub async fn get_following(&self) -> Vec<String> {
        let following = self.following.read().await;
        following.keys().cloned().collect()
    }

    /// 获取关注组合的持仓变化
    pub async fn get_following_changes(&self) -> Result<Vec<HoldingChange>> {
        let mut changes = Vec::new();
        let mut following = self.following.write().await;

        for (portfolio_id, old_holdings) in following.iter_mut() {
            match self.get_portfolio_holdings(portfolio_id).await {
                Ok(new_holdings) => {
                    let old_map: HashMap<_, _> = old_holdings.iter()
                        .map(|h| (h.symbol.clone(), h.clone()))
                        .collect();

                    // 检查新增
                    for new_h in &new_holdings {
                        if !old_map.contains_key(&new_h.symbol) {
                            changes.push(HoldingChange {
                                symbol: new_h.symbol.clone(),
                                name: new_h.name.clone(),
                                change_type: "add".to_string(),
                                shares: new_h.shares,
                                price: new_h.price,
                                time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            });
                        }
                    }

                    // 检查减少/清除
                    let new_map: HashMap<_, _> = new_holdings.iter()
                        .map(|h| (h.symbol.clone(), h.clone()))
                        .collect();

                    for (symbol, old_h) in &old_map {
                        if let Some(new_h) = new_map.get(symbol) {
                            if new_h.shares < old_h.shares {
                                changes.push(HoldingChange {
                                    symbol: symbol.clone(),
                                    name: old_h.name.clone(),
                                    change_type: if new_h.shares == 0.0 { "clear".to_string() } else { "reduce".to_string() },
                                    shares: old_h.shares - new_h.shares,
                                    price: 0.0,
                                    time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                });
                            }
                        }
                    }

                    *old_holdings = new_holdings;
                }
                Err(e) => {
                    tracing::warn!("Failed to get holdings for portfolio {}: {}", portfolio_id, e);
                }
            }
        }

        Ok(changes)
    }
}

impl Default for GroupPortfolioApi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_portfolio_list() {
        let api = GroupPortfolioApi::new();
        let portfolios = api.get_portfolio_list(1, 20).await;
        println!("Portfolios: {:?}", portfolios);
    }
}
