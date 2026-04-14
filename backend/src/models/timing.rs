use chrono::{Datelike, Local, Timelike};
use serde::{Deserialize, Serialize};

/// 日内时段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntradayWindow {
    PreAuction,      // 9:15-9:25 集合竞价
    MorningPeak,     // 9:30-10:30 黄金一小时 ★最佳
    MiddayLull,      // 10:30-14:30 垃圾时间
    AfternoonActive,  // 14:30-15:00 尾盘活跃 ★次佳
    AfterHours,      // 15:05-15:30 盘后交易
    Closed,          // 非交易时间
}

impl IntradayWindow {
    /// 获取当前所处时段
    pub fn current() -> Self {
        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute();
        let time_minutes = hour * 60 + minute;

        match time_minutes {
            0..=0 if hour < 9 => Self::Closed,                                    // 未开盘
            555..=565 => Self::PreAuction,                                         // 9:15-9:25
            570..=630 => Self::MorningPeak,                                        // 9:30-10:30
            631..=870 => Self::MiddayLull,                                        // 10:30-14:30
            871..=900 => Self::AfternoonActive,                                   // 14:30-15:00
            901..=930 => Self::AfterHours,                                        // 15:05-15:30
            _ => Self::Closed,                                                    // 收盘后
        }
    }

    /// 获取时段名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::PreAuction => "集合竞价",
            Self::MorningPeak => "早盘高峰",
            Self::MiddayLull => "垃圾时间",
            Self::AfternoonActive => "尾盘活跃",
            Self::AfterHours => "盘后交易",
            Self::Closed => "非交易时间",
        }
    }

    /// 获取时段描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::PreAuction => "布局窗口，可挂单买入龙头股",
            Self::MorningPeak => "黄金一小时，最佳操作窗口",
            Self::MiddayLull => "震荡期，建议观望不操作",
            Self::AfternoonActive => "尾盘异动，可买入超跌股",
            Self::AfterHours => "盘后交易，大额订单最后机会",
            Self::Closed => "非交易时间",
        }
    }

    /// 获取时段评分 (0-100)
    pub fn score(&self) -> f64 {
        match self {
            Self::MorningPeak => 100.0,
            Self::AfternoonActive => 80.0,
            Self::PreAuction => 60.0,
            Self::AfterHours => 40.0,
            Self::MiddayLull => 20.0,
            Self::Closed => 0.0,
        }
    }

    /// 获取时段剩余分钟数
    pub fn remaining_minutes(&self) -> i32 {
        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute();
        let current_minutes = (hour * 60 + minute) as i32;

        match self {
            Self::PreAuction => (565 - current_minutes).max(0),     // 到9:25
            Self::MorningPeak => (630 - current_minutes).max(0),   // 到10:30
            Self::MiddayLull => (870 - current_minutes).max(0),   // 到14:30
            Self::AfternoonActive => (900 - current_minutes).max(0), // 到15:00
            Self::AfterHours => (930 - current_minutes).max(0),    // 到15:30
            Self::Closed => 0,
        }
    }

    /// 获取下一个时段
    pub fn next_window(&self) -> Option<Self> {
        match self {
            Self::Closed => Some(Self::PreAuction),
            Self::PreAuction => Some(Self::MorningPeak),
            Self::MorningPeak => Some(Self::MiddayLull),
            Self::MiddayLull => Some(Self::AfternoonActive),
            Self::AfternoonActive => Some(Self::AfterHours),
            Self::AfterHours => None,
        }
    }
}

/// 年度时段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnualWindow {
    SpringRally,   // 2-3月 春季躁动 ★最佳
    SummerDull,    // 5-6月 夏季淡季
    MidYear,       // 7-9月 中报行情 七翻身 ★次佳
    AutumnDull,    // 10月 秋季淡季
    YearEnd,       // 11-1月 跨年行情 妖股摇篮 ★第三
    OffSeason,     // 4月、11月 过渡期
}

impl AnnualWindow {
    /// 获取当前所处年度时段
    pub fn current() -> Self {
        let now = Local::now();
        let month = now.month();

        match month {
            2 | 3 => Self::SpringRally,
            5 | 6 => Self::SummerDull,
            7 | 8 | 9 => Self::MidYear,
            10 => Self::AutumnDull,
            11 | 12 | 1 => Self::YearEnd,
            _ => Self::OffSeason,
        }
    }

    /// 获取时段名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::SpringRally => "春季躁动",
            Self::SummerDull => "夏季淡季",
            Self::MidYear => "中报行情",
            Self::AutumnDull => "秋季淡季",
            Self::YearEnd => "跨年行情",
            Self::OffSeason => "过渡期",
        }
    }

    /// 获取时段俗称
    pub fn nickname(&self) -> &'static str {
        match self {
            Self::SpringRally => "吃饭行情",
            Self::SummerDull => "",
            Self::MidYear => "七翻身",
            Self::AutumnDull => "",
            Self::YearEnd => "妖股摇篮",
            Self::OffSeason => "",
        }
    }

    /// 获取时段评分 (0-100)
    pub fn score(&self) -> f64 {
        match self {
            Self::SpringRally => 100.0,
            Self::YearEnd => 85.0,
            Self::MidYear => 75.0,
            Self::AutumnDull => 40.0,
            Self::SummerDull => 30.0,
            Self::OffSeason => 50.0,
        }
    }

    /// 获取上涨概率
    pub fn win_rate(&self) -> f64 {
        match self {
            Self::SpringRally => 93.0,
            Self::MidYear => 75.0,
            Self::YearEnd => 70.0,
            Self::OffSeason => 55.0,
            Self::AutumnDull => 50.0,
            Self::SummerDull => 45.0,
        }
    }

    /// 获取推荐仓位
    pub fn recommended_position(&self) -> PositionLevel {
        match self {
            Self::SpringRally => PositionLevel::Heavy,
            Self::YearEnd => PositionLevel::Light,
            Self::MidYear => PositionLevel::Half,
            Self::OffSeason => PositionLevel::Half,
            Self::SummerDull | Self::AutumnDull => PositionLevel::Light,
        }
    }
}

/// 交易指令
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeAction {
    Buy,   // 买入 ⭐ - 积极建仓
    Sell,  // 卖出 ⚠️ - 考虑止盈/止损
    Hold,  // 持有 🔄 - 等待更好时机
    Watch, // 观望 👀 - 不操作
}

impl TradeAction {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Buy => "买入",
            Self::Sell => "卖出",
            Self::Hold => "持有",
            Self::Watch => "观望",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Buy => "⭐",
            Self::Sell => "⚠️",
            Self::Hold => "🔄",
            Self::Watch => "👀",
        }
    }
}

/// 仓位等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionLevel {
    Full,   // 满仓 100%
    Heavy,  // 重仓 70-80%
    Half,   // 半仓 50%
    Light,  // 轻仓 20-30%
    Empty,  // 空仓 0%
}

impl PositionLevel {
    pub fn percentage(&self) -> i32 {
        match self {
            Self::Full => 100,
            Self::Heavy => 75,
            Self::Half => 50,
            Self::Light => 25,
            Self::Empty => 0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Full => "满仓",
            Self::Heavy => "重仓",
            Self::Half => "半仓",
            Self::Light => "轻仓",
            Self::Empty => "空仓",
        }
    }
}

/// 风险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningLevel {
    Red,    // 极高风险
    Orange, // 高风险
    Yellow, // 中风险
}

impl WarningLevel {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Red => "极高",
            Self::Orange => "高",
            Self::Yellow => "中",
        }
    }
}

// ============ 核心数据结构 ============

/// 仓位建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionAdvice {
    pub current_position: PositionLevel,
    pub max_position: PositionLevel,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
}

/// 股票推荐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockPick {
    pub symbol: String,
    pub name: String,
    pub reason: String,
    pub entry_min: f64,
    pub entry_max: f64,
    pub stop_loss_pct: f64,
    pub target_pct: f64,
}

/// 风险提示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskWarning {
    pub level: WarningLevel,
    pub message: String,
    pub suggestion: String,
}

/// 板块推荐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorRecommendation {
    pub hot_sectors: Vec<String>,
    pub defensive_sectors: Vec<String>,
    pub sector_reason: String,
    pub risk_level: WarningLevel,
    pub position_advice: PositionLevel,
}

/// ⭐ 核心输出：交易时机信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingSignal {
    pub timestamp: String,

    // ========== 核心指导 ==========
    pub action: TradeAction,
    pub action_strength: i32,

    // ========== 时段信息 ==========
    pub intraday_window: String,
    pub intraday_score: i32,
    pub intraday_remaining_minutes: i32,
    pub intraday_next: String,

    pub annual_window: String,
    pub annual_nickname: String,
    pub annual_score: i32,
    pub annual_win_rate: f64,

    // ========== 仓位指导 ==========
    pub position_advice: PositionAdvice,

    // ========== 板块推荐 ==========
    pub sector_recommendation: SectorRecommendation,

    // ========== 具体推荐 ==========
    pub stock_picks: Vec<StockPick>,

    // ========== 风险提示 ==========
    pub risk_warnings: Vec<RiskWarning>,

    // ========== 操作步骤 ==========
    pub actionable_steps: Vec<String>,
}

/// 日内时段详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntradayWindowDetail {
    pub current: String,
    pub current_name: String,
    pub current_score: i32,
    pub remaining_minutes: i32,
    pub next: String,
    pub next_name: String,
    pub description: String,
}

/// 年度窗口详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnualWindowDetail {
    pub current: String,
    pub current_name: String,
    pub current_nickname: String,
    pub current_score: i32,
    pub win_rate: f64,
    pub recommended_position: String,
    pub next: Option<String>,
    pub next_name: Option<String>,
    pub countdown_months: i32,
}
