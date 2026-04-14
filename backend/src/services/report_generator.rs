//! 报告生成器 (Report Generator)
//!
//! 生成每日、每周、每月的交易报告

use serde::{Deserialize, Serialize};
use chrono::{Local, Duration};
use crate::db::DbPool;
use crate::db::autonomous_trading::{
    self, AiDecision, DailyReport as DbDailyReport, TradingStats,
    get_decisions_by_date, get_daily_report, get_history_reports,
    get_trading_stats, get_strategy_performance, get_recent_learning,
};

/// 报告配置
#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub include_positions: bool,
    pub include_trades: bool,
    pub include_strategy_analysis: bool,
    pub include_learning: bool,
    pub ai_insight_level: AInsightLevel,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            include_positions: true,
            include_trades: true,
            include_strategy_analysis: true,
            include_learning: true,
            ai_insight_level: AInsightLevel::Basic,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AInsightLevel {
    Basic,    // 基础分析
    Standard, // 标准分析
    Deep,     // 深度分析
}

/// 持仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionInfo {
    pub symbol: String,
    pub name: String,
    pub quantity: f64,
    pub avg_cost: f64,
    pub current_price: f64,
    pub market_value: f64,
    pub profit_loss: f64,
    pub profit_ratio: f64,
}

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeInfo {
    pub timestamp: String,
    pub symbol: String,
    pub name: String,
    pub action: String,
    pub price: f64,
    pub quantity: f64,
    pub amount: f64,
    pub pnl: Option<f64>,
    pub reason: String,
}

/// 每日报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyTradingReport {
    pub date: String,
    pub generated_at: String,
    // 收益概览
    pub initial_capital: f64,
    pub final_capital: f64,
    pub total_pnl: f64,
    pub pnl_ratio: f64,
    // 持仓情况
    pub positions: Vec<PositionInfo>,
    pub positions_count: i32,
    // 交易情况
    pub trades: Vec<TradeInfo>,
    pub trades_count: i32,
    pub buy_count: i32,
    pub sell_count: i32,
    pub win_count: i32,
    pub lose_count: i32,
    pub win_rate: f64,
    // 策略表现
    pub strategy_stats: StrategyStatsSummary,
    // 市场分析
    pub hot_sectors: Vec<String>,
    pub market_summary: String,
    // AI 观察
    pub ai_observations: Vec<String>,
    pub tomorrow_outlook: String,
    // 学习记录
    pub recent_learning: Vec<LearningItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStatsSummary {
    pub total_trades: i32,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub max_drawdown: f64,
    pub avg_holding_days: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningItem {
    pub timestamp: String,
    pub event: String,
    pub lesson: String,
    pub adjustment: String,
}

/// 周报
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyReport {
    pub start_date: String,
    pub end_date: String,
    pub generated_at: String,
    pub daily_reports: Vec<DailyTradingReport>,
    pub total_pnl: f64,
    pub total_pnl_ratio: f64,
    pub avg_win_rate: f64,
    pub total_trades: i32,
    pub best_day: DayPerformance,
    pub worst_day: DayPerformance,
    pub most_profitable_stock: String,
    pub lessons_learned: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayPerformance {
    pub date: String,
    pub pnl: f64,
    pub pnl_ratio: f64,
}

/// 月报
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyReport {
    pub month: String,
    pub year: i32,
    pub generated_at: String,
    pub total_pnl: f64,
    pub total_pnl_ratio: f64,
    pub total_trades: i32,
    pub avg_win_rate: f64,
    pub total_profit: f64,
    pub total_loss: f64,
    pub net_profit: f64,
    pub best_strategy: String,
    pub lessons_learned: Vec<String>,
    pub monthly_summary: String,
}

/// 报告生成器
#[derive(Clone)]
pub struct ReportGenerator {
    db_pool: DbPool,
    config: ReportConfig,
}

impl ReportGenerator {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            db_pool,
            config: ReportConfig::default(),
        }
    }

    pub fn with_config(db_pool: DbPool, config: ReportConfig) -> Self {
        Self { db_pool, config }
    }

    /// 生成日报
    pub async fn generate_daily_report(&self, date: &str) -> Result<DailyTradingReport, String> {
        // 获取当日决策
        let decisions = get_decisions_by_date(&self.db_pool, date)
            .map_err(|e| format!("获取决策失败: {}", e))?;

        // 获取统计
        let stats = get_trading_stats(&self.db_pool, 1)
            .map_err(|e| format!("获取统计失败: {}", e))?;

        // 获取策略表现
        let strategy_perf = get_strategy_performance(&self.db_pool, "ai_strategy", 1)
            .map_err(|e| format!("获取策略表现失败: {}", e))?;

        // 获取学习记录
        let learning = get_recent_learning(&self.db_pool, 5)
            .map_err(|e| format!("获取学习记录失败: {}", e))?;

        // 获取持仓 (模拟数据，实际应从执行模块获取)
        let positions = vec![]; // TODO: 从 ExecutionModule 获取

        // 转换为交易信息
        let trades: Vec<TradeInfo> = decisions.iter()
            .filter(|d| d.executed)
            .map(|d| TradeInfo {
                timestamp: d.timestamp.clone(),
                symbol: d.symbol.clone(),
                name: d.name.clone(),
                action: d.action.clone(),
                price: d.price,
                quantity: d.quantity,
                amount: d.price * d.quantity,
                pnl: d.pnl,
                reason: d.reason.clone(),
            })
            .collect();

        let buy_count = trades.iter().filter(|t| t.action == "buy").count() as i32;
        let sell_count = trades.iter().filter(|t| t.action == "sell").count() as i32;
        let win_count = trades.iter().filter(|t| t.pnl.map_or(false, |p| p > 0.0)).count() as i32;
        let lose_count = trades.iter().filter(|t| t.pnl.map_or(false, |p| p < 0.0)).count() as i32;

        let strategy_stats = if let Some(perf) = strategy_perf.first() {
            StrategyStatsSummary {
                total_trades: perf.total_trades,
                win_rate: perf.win_rate,
                profit_factor: perf.profit_factor,
                max_drawdown: perf.max_drawdown,
                avg_holding_days: perf.avg_holding_days,
            }
        } else {
            StrategyStatsSummary {
                total_trades: stats.total_trades,
                win_rate: stats.win_rate,
                profit_factor: stats.profit_factor,
                max_drawdown: stats.max_drawdown,
                avg_holding_days: 0.0,
            }
        };

        let learning_items: Vec<LearningItem> = learning.iter().map(|l| LearningItem {
            timestamp: l.timestamp.clone(),
            event: l.event.clone(),
            lesson: l.lesson.clone(),
            adjustment: l.adjustment.clone(),
        }).collect();

        let initial_capital = 100000.0; // 假设初始资金
        let final_capital = initial_capital + stats.total_pnl;

        Ok(DailyTradingReport {
            date: date.to_string(),
            generated_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            initial_capital,
            final_capital,
            total_pnl: stats.total_pnl,
            pnl_ratio: if initial_capital > 0.0 { stats.total_pnl / initial_capital } else { 0.0 },
            positions,
            positions_count: 0,
            trades,
            trades_count: stats.total_trades,
            buy_count,
            sell_count,
            win_count,
            lose_count,
            win_rate: stats.win_rate,
            strategy_stats,
            hot_sectors: vec![],
            market_summary: "".to_string(),
            ai_observations: vec![],
            tomorrow_outlook: "".to_string(),
            recent_learning: learning_items,
        })
    }

    /// 生成周报
    pub async fn generate_weekly_report(&self, start_date: &str, end_date: &str) -> Result<WeeklyReport, String> {
        let mut daily_reports = vec![];
        let mut current = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|e| format!("日期解析失败: {}", e))?;
        let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
            .map_err(|e| format!("日期解析失败: {}", e))?;

        while current <= end {
            let date_str = current.format("%Y-%m-%d").to_string();
            if let Ok(report) = self.generate_daily_report(&date_str).await {
                daily_reports.push(report);
            }
            current += Duration::days(1);
        }

        let total_pnl: f64 = daily_reports.iter().map(|r| r.total_pnl).sum();
        let initial_capital = 100000.0;
        let total_pnl_ratio = total_pnl / initial_capital;
        let avg_win_rate = if daily_reports.is_empty() {
            0.0
        } else {
            daily_reports.iter().map(|r| r.win_rate).sum::<f64>() / daily_reports.len() as f64
        };
        let total_trades: i32 = daily_reports.iter().map(|r| r.trades_count).sum();

        let mut best_day = DayPerformance {
            date: start_date.to_string(),
            pnl: 0.0,
            pnl_ratio: 0.0,
        };
        let mut worst_day = DayPerformance {
            date: start_date.to_string(),
            pnl: 0.0,
            pnl_ratio: 0.0,
        };

        for report in &daily_reports {
            if report.total_pnl > best_day.pnl {
                best_day = DayPerformance {
                    date: report.date.clone(),
                    pnl: report.total_pnl,
                    pnl_ratio: report.pnl_ratio,
                };
            }
            if report.total_pnl < worst_day.pnl {
                worst_day = DayPerformance {
                    date: report.date.clone(),
                    pnl: report.total_pnl,
                    pnl_ratio: report.pnl_ratio,
                };
            }
        }

        Ok(WeeklyReport {
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            generated_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            daily_reports,
            total_pnl,
            total_pnl_ratio,
            avg_win_rate,
            total_trades,
            best_day,
            worst_day,
            most_profitable_stock: "".to_string(),
            lessons_learned: vec![],
        })
    }

    /// 生成月报
    pub async fn generate_monthly_report(&self, year: i32, month: u32) -> Result<MonthlyReport, String> {
        let start_date = format!("{:04}-{:02}-01", year, month);
        let end_date = if month == 12 {
            format!("{:04}-01-01", year + 1)
        } else {
            format!("{:04}-{:02}-01", year, month + 1)
        };

        let weekly = self.generate_weekly_report(&start_date, &end_date).await?;

        let total_profit: f64 = weekly.daily_reports.iter()
            .map(|r| r.trades.iter().filter(|t| t.pnl.map_or(false, |p| p > 0.0)).map(|t| t.pnl.unwrap_or(0.0)).sum::<f64>())
            .sum();

        let total_loss: f64 = weekly.daily_reports.iter()
            .map(|r| r.trades.iter().filter(|t| t.pnl.map_or(false, |p| p < 0.0)).map(|t| t.pnl.unwrap_or(0.0)).sum::<f64>())
            .sum();

        Ok(MonthlyReport {
            month: format!("{:04}-{:02}", year, month),
            year,
            generated_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            total_pnl: weekly.total_pnl,
            total_pnl_ratio: weekly.total_pnl_ratio,
            total_trades: weekly.total_trades,
            avg_win_rate: weekly.avg_win_rate,
            total_profit,
            total_loss,
            net_profit: total_profit + total_loss,
            best_strategy: "ai_strategy".to_string(),
            lessons_learned: weekly.lessons_learned,
            monthly_summary: format!(
                "{}月交易报告：总盈亏{:.2}元，胜率{:.1}%，共{}笔交易",
                month, weekly.total_pnl, weekly.avg_win_rate * 100.0, weekly.total_trades
            ),
        })
    }

    /// 生成 AI 观察 (使用 LLM 进行深度分析)
    pub async fn generate_ai_observations(&self, _report: &DailyTradingReport) -> Vec<String> {
        // TODO: 集成 LLM API 进行深度分析
        vec![
            "今日市场波动较大，建议关注明日开盘情况".to_string(),
            "热点板块轮动较快，注意控制仓位".to_string(),
        ]
    }

    /// 生成次日展望
    pub async fn generate_tomorrow_outlook(&self, _report: &DailyTradingReport) -> String {
        // TODO: 基于当日报告和 LLM 生成次日展望
        "明日关注政策导向和市场情绪变化".to_string()
    }
}

/// 格式化报告为 Markdown
pub fn format_report_markdown(report: &DailyTradingReport) -> String {
    let mut md = format!(
        "# {} 每日交易报告\n\n",
        report.date
    );

    // 收益概览
    md += "## 收益概览\n\n";
    md += &format!("- 初始资金: {:.2}\n", report.initial_capital);
    md += &format!("- 最终资金: {:.2}\n", report.final_capital);
    md += &format!("- 总盈亏: {:.2} ({:+.2}%)\n\n", report.total_pnl, report.pnl_ratio * 100.0);

    // 交易统计
    md += "## 交易统计\n\n";
    md += &format!("- 总交易数: {}\n", report.trades_count);
    md += &format!("- 买入: {} | 卖出: {}\n", report.buy_count, report.sell_count);
    md += &format!("- 盈利: {} | 亏损: {} | 胜率: {:.1}%\n\n", report.win_count, report.lose_count, report.win_rate * 100.0);

    // 策略表现
    md += "## 策略表现\n\n";
    md += &format!("- 总交易数: {}\n", report.strategy_stats.total_trades);
    md += &format!("- 胜率: {:.1}%\n", report.strategy_stats.win_rate * 100.0);
    md += &format!("- 盈亏比: {:.2}\n", report.strategy_stats.profit_factor);
    md += &format!("- 最大回撤: {:.1}%\n", report.strategy_stats.max_drawdown * 100.0);
    md += &format!("- 平均持仓: {:.1}天\n\n", report.strategy_stats.avg_holding_days);

    // 交易记录
    if !report.trades.is_empty() {
        md += "## 交易记录\n\n";
        md += "| 时间 | 股票 | 操作 | 价格 | 数量 | 盈亏 |\n";
        md += "|------|------|------|------|------|------|\n";
        for trade in &report.trades {
            let pnl_str = match trade.pnl {
                Some(p) => format!("{:+.2}", p),
                None => "-".to_string(),
            };
            md += &format!(
                "| {} | {} | {} | {:.2} | {} | {} |\n",
                &trade.timestamp[11..16], trade.symbol, trade.action, trade.price, trade.quantity, pnl_str
            );
        }
        md += "\n";
    }

    // AI 观察
    if !report.ai_observations.is_empty() {
        md += "## AI 观察\n\n";
        for obs in &report.ai_observations {
            md += &format!("- {}\n", obs);
        }
        md += "\n";
    }

    // 次日展望
    if !report.tomorrow_outlook.is_empty() {
        md += &format!("## 次日展望\n\n{}\n\n", report.tomorrow_outlook);
    }

    // 学习记录
    if !report.recent_learning.is_empty() {
        md += "## 最近学习\n\n";
        for item in &report.recent_learning {
            md += &format!("- **[{}]** {}: {}\n", item.timestamp, item.event, item.lesson);
        }
        md += "\n";
    }

    md += &format!("\n---\n*报告生成时间: {}*\n", report.generated_at);

    md
}
