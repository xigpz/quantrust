//! AI 自主交易持久化模块
//!
//! 负责存储 AI 交易决策、每日报告、策略表现等数据

use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use chrono::Local;
use crate::db::DbPool;

/// AI 交易决策记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDecision {
    pub id: String,
    pub timestamp: String,
    pub symbol: String,
    pub name: String,
    pub action: String,        // buy | sell | hold
    pub quantity: f64,
    pub price: f64,
    pub confidence: f64,
    pub reason: String,
    pub strategies: String,   // JSON array
    pub market_context: String,
    pub executed: bool,
    pub pnl: Option<f64>,     // 盈亏 (卖出时记录)
}

/// 每日交易报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReport {
    pub id: String,
    pub date: String,
    pub total_pnl: f64,
    pub pnl_ratio: f64,
    pub initial_capital: f64,
    pub final_capital: f64,
    pub positions_count: i32,
    pub trades_count: i32,
    pub win_trades: i32,
    pub lose_trades: i32,
    pub hot_sectors: String,        // JSON array
    pub positions_json: String,      // JSON array
    pub trades_json: String,        // JSON array
    pub summary: String,
    pub ai_observations: String,    // JSON array
    pub next_outlook: String,
    pub created_at: String,
}

/// 策略表现记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub id: String,
    pub strategy_name: String,
    pub date: String,
    pub total_trades: i32,
    pub win_trades: i32,
    pub lose_trades: i32,
    pub win_rate: f64,
    pub avg_profit: f64,
    pub avg_loss: f64,
    pub max_drawdown: f64,
    pub profit_factor: f64,
    pub avg_holding_days: f64,
    pub updated_at: String,
}

/// 学习记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRecord {
    pub id: String,
    pub timestamp: String,
    pub event: String,
    pub lesson: String,
    pub adjustment: String,
}

/// 初始化 AI 自主交易相关表
pub fn init_autonomous_trading_tables(conn: &Connection) -> SqliteResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS ai_decisions (
            id TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL,
            symbol TEXT NOT NULL,
            name TEXT NOT NULL,
            action TEXT NOT NULL,
            quantity REAL NOT NULL DEFAULT 0,
            price REAL NOT NULL DEFAULT 0,
            confidence REAL NOT NULL DEFAULT 0,
            reason TEXT NOT NULL DEFAULT '',
            strategies TEXT NOT NULL DEFAULT '[]',
            market_context TEXT NOT NULL DEFAULT '',
            executed INTEGER NOT NULL DEFAULT 0,
            pnl REAL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS ai_daily_reports (
            id TEXT PRIMARY KEY,
            date TEXT NOT NULL UNIQUE,
            total_pnl REAL NOT NULL DEFAULT 0,
            pnl_ratio REAL NOT NULL DEFAULT 0,
            initial_capital REAL NOT NULL DEFAULT 100000,
            final_capital REAL NOT NULL DEFAULT 100000,
            positions_count INTEGER NOT NULL DEFAULT 0,
            trades_count INTEGER NOT NULL DEFAULT 0,
            win_trades INTEGER NOT NULL DEFAULT 0,
            lose_trades INTEGER NOT NULL DEFAULT 0,
            hot_sectors TEXT NOT NULL DEFAULT '[]',
            positions_json TEXT NOT NULL DEFAULT '[]',
            trades_json TEXT NOT NULL DEFAULT '[]',
            summary TEXT NOT NULL DEFAULT '',
            ai_observations TEXT NOT NULL DEFAULT '[]',
            next_outlook TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS ai_strategy_performance (
            id TEXT PRIMARY KEY,
            strategy_name TEXT NOT NULL,
            date TEXT NOT NULL,
            total_trades INTEGER NOT NULL DEFAULT 0,
            win_trades INTEGER NOT NULL DEFAULT 0,
            lose_trades INTEGER NOT NULL DEFAULT 0,
            win_rate REAL NOT NULL DEFAULT 0,
            avg_profit REAL NOT NULL DEFAULT 0,
            avg_loss REAL NOT NULL DEFAULT 0,
            max_drawdown REAL NOT NULL DEFAULT 0,
            profit_factor REAL NOT NULL DEFAULT 0,
            avg_holding_days REAL NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS ai_learning_records (
            id TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL,
            event TEXT NOT NULL,
            lesson TEXT NOT NULL,
            adjustment TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_ai_decisions_timestamp ON ai_decisions(timestamp);
        CREATE INDEX IF NOT EXISTS idx_ai_decisions_symbol ON ai_decisions(symbol);
        CREATE INDEX IF NOT EXISTS idx_ai_decisions_date ON ai_decisions(substr(timestamp, 1, 10));
        CREATE INDEX IF NOT EXISTS idx_ai_daily_reports_date ON ai_daily_reports(date);
        CREATE INDEX IF NOT EXISTS idx_ai_strategy_date ON ai_strategy_performance(date);
        CREATE INDEX IF NOT EXISTS idx_ai_learning_timestamp ON ai_learning_records(timestamp);
        ",
    )?;
    Ok(())
}

/// 保存 AI 交易决策
pub fn save_decision(pool: &DbPool, decision: &AiDecision) -> SqliteResult<()> {
    let conn = pool.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO ai_decisions
         (id, timestamp, symbol, name, action, quantity, price, confidence, reason, strategies, market_context, executed, pnl)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            decision.id,
            decision.timestamp,
            decision.symbol,
            decision.name,
            decision.action,
            decision.quantity,
            decision.price,
            decision.confidence,
            decision.reason,
            decision.strategies,
            decision.market_context,
            decision.executed as i32,
            decision.pnl,
        ],
    )?;
    Ok(())
}

/// 获取指定日期的所有决策
pub fn get_decisions_by_date(pool: &DbPool, date: &str) -> SqliteResult<Vec<AiDecision>> {
    let conn = pool.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, symbol, name, action, quantity, price, confidence, reason, strategies, market_context, executed, pnl
         FROM ai_decisions WHERE substr(timestamp, 1, 10) = ?1 ORDER BY timestamp",
    )?;

    let rows = stmt.query_map([date], |row| {
        Ok(AiDecision {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            symbol: row.get(2)?,
            name: row.get(3)?,
            action: row.get(4)?,
            quantity: row.get(5)?,
            price: row.get(6)?,
            confidence: row.get(7)?,
            reason: row.get(8)?,
            strategies: row.get(9)?,
            market_context: row.get(10)?,
            executed: row.get::<_, i32>(11)? != 0,
            pnl: row.get(12)?,
        })
    })?;

    rows.collect()
}

/// 获取最近的 N 条决策
pub fn get_recent_decisions(pool: &DbPool, limit: i32) -> SqliteResult<Vec<AiDecision>> {
    let conn = pool.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, symbol, name, action, quantity, price, confidence, reason, strategies, market_context, executed, pnl
         FROM ai_decisions ORDER BY timestamp DESC LIMIT ?1",
    )?;

    let rows = stmt.query_map([limit], |row| {
        Ok(AiDecision {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            symbol: row.get(2)?,
            name: row.get(3)?,
            action: row.get(4)?,
            quantity: row.get(5)?,
            price: row.get(6)?,
            confidence: row.get(7)?,
            reason: row.get(8)?,
            strategies: row.get(9)?,
            market_context: row.get(10)?,
            executed: row.get::<_, i32>(11)? != 0,
            pnl: row.get(12)?,
        })
    })?;

    rows.collect()
}

/// 保存每日报告
pub fn save_daily_report(pool: &DbPool, report: &DailyReport) -> SqliteResult<()> {
    let conn = pool.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO ai_daily_reports
         (id, date, total_pnl, pnl_ratio, initial_capital, final_capital, positions_count, trades_count, win_trades, lose_trades, hot_sectors, positions_json, trades_json, summary, ai_observations, next_outlook)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            report.id,
            report.date,
            report.total_pnl,
            report.pnl_ratio,
            report.initial_capital,
            report.final_capital,
            report.positions_count,
            report.trades_count,
            report.win_trades,
            report.lose_trades,
            report.hot_sectors,
            report.positions_json,
            report.trades_json,
            report.summary,
            report.ai_observations,
            report.next_outlook,
        ],
    )?;
    Ok(())
}

/// 获取每日报告
pub fn get_daily_report(pool: &DbPool, date: &str) -> SqliteResult<Option<DailyReport>> {
    let conn = pool.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, date, total_pnl, pnl_ratio, initial_capital, final_capital, positions_count, trades_count, win_trades, lose_trades, hot_sectors, positions_json, trades_json, summary, ai_observations, next_outlook, created_at
         FROM ai_daily_reports WHERE date = ?1",
    )?;

    let mut rows = stmt.query([date])?;
    if let Some(row) = rows.next()? {
        Ok(Some(DailyReport {
            id: row.get(0)?,
            date: row.get(1)?,
            total_pnl: row.get(2)?,
            pnl_ratio: row.get(3)?,
            initial_capital: row.get(4)?,
            final_capital: row.get(5)?,
            positions_count: row.get(6)?,
            trades_count: row.get(7)?,
            win_trades: row.get(8)?,
            lose_trades: row.get(9)?,
            hot_sectors: row.get(10)?,
            positions_json: row.get(11)?,
            trades_json: row.get(12)?,
            summary: row.get(13)?,
            ai_observations: row.get(14)?,
            next_outlook: row.get(15)?,
            created_at: row.get(16)?,
        }))
    } else {
        Ok(None)
    }
}

/// 获取历史报告
pub fn get_history_reports(pool: &DbPool, limit: i32) -> SqliteResult<Vec<DailyReport>> {
    let conn = pool.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, date, total_pnl, pnl_ratio, initial_capital, final_capital, positions_count, trades_count, win_trades, lose_trades, hot_sectors, positions_json, trades_json, summary, ai_observations, next_outlook, created_at
         FROM ai_daily_reports ORDER BY date DESC LIMIT ?1",
    )?;

    let rows = stmt.query_map([limit], |row| {
        Ok(DailyReport {
            id: row.get(0)?,
            date: row.get(1)?,
            total_pnl: row.get(2)?,
            pnl_ratio: row.get(3)?,
            initial_capital: row.get(4)?,
            final_capital: row.get(5)?,
            positions_count: row.get(6)?,
            trades_count: row.get(7)?,
            win_trades: row.get(8)?,
            lose_trades: row.get(9)?,
            hot_sectors: row.get(10)?,
            positions_json: row.get(11)?,
            trades_json: row.get(12)?,
            summary: row.get(13)?,
            ai_observations: row.get(14)?,
            next_outlook: row.get(15)?,
            created_at: row.get(16)?,
        })
    })?;

    rows.collect()
}

/// 保存策略表现
pub fn save_strategy_performance(pool: &DbPool, perf: &StrategyPerformance) -> SqliteResult<()> {
    let conn = pool.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO ai_strategy_performance
         (id, strategy_name, date, total_trades, win_trades, lose_trades, win_rate, avg_profit, avg_loss, max_drawdown, profit_factor, avg_holding_days, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            perf.id,
            perf.strategy_name,
            perf.date,
            perf.total_trades,
            perf.win_trades,
            perf.lose_trades,
            perf.win_rate,
            perf.avg_profit,
            perf.avg_loss,
            perf.max_drawdown,
            perf.profit_factor,
            perf.avg_holding_days,
            perf.updated_at,
        ],
    )?;
    Ok(())
}

/// 获取策略表现历史
pub fn get_strategy_performance(pool: &DbPool, strategy_name: &str, days: i32) -> SqliteResult<Vec<StrategyPerformance>> {
    let conn = pool.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, strategy_name, date, total_trades, win_trades, lose_trades, win_rate, avg_profit, avg_loss, max_drawdown, profit_factor, avg_holding_days, updated_at
         FROM ai_strategy_performance
         WHERE strategy_name = ?1 AND date >= date('now', ?2)
         ORDER BY date DESC",
    )?;

    let days_param = format!("-{} days", days);
    let rows = stmt.query_map(params![strategy_name, days_param], |row| {
        Ok(StrategyPerformance {
            id: row.get(0)?,
            strategy_name: row.get(1)?,
            date: row.get(2)?,
            total_trades: row.get(3)?,
            win_trades: row.get(4)?,
            lose_trades: row.get(5)?,
            win_rate: row.get(6)?,
            avg_profit: row.get(7)?,
            avg_loss: row.get(8)?,
            max_drawdown: row.get(9)?,
            profit_factor: row.get(10)?,
            avg_holding_days: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    rows.collect()
}

/// 保存学习记录
pub fn save_learning_record(pool: &DbPool, record: &LearningRecord) -> SqliteResult<()> {
    let conn = pool.lock().unwrap();
    conn.execute(
        "INSERT INTO ai_learning_records (id, timestamp, event, lesson, adjustment) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![record.id, record.timestamp, record.event, record.lesson, record.adjustment],
    )?;
    Ok(())
}

/// 获取最近的学习记录
pub fn get_recent_learning(pool: &DbPool, limit: i32) -> SqliteResult<Vec<LearningRecord>> {
    let conn = pool.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, event, lesson, adjustment FROM ai_learning_records ORDER BY timestamp DESC LIMIT ?1",
    )?;

    let rows = stmt.query_map([limit], |row| {
        Ok(LearningRecord {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            event: row.get(2)?,
            lesson: row.get(3)?,
            adjustment: row.get(4)?,
        })
    })?;

    rows.collect()
}

/// 计算统计信息
#[derive(Debug, Clone)]
pub struct TradingStats {
    pub total_trades: i32,
    pub win_trades: i32,
    pub lose_trades: i32,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub avg_profit: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub max_drawdown: f64,
}

pub fn get_trading_stats(pool: &DbPool, days: i32) -> SqliteResult<TradingStats> {
    let conn = pool.lock().unwrap();
    let date_limit = format!("-{} days", days);

    // 获取所有已执行的卖出决策
    let mut stmt = conn.prepare(
        "SELECT pnl FROM ai_decisions
         WHERE executed = 1 AND action = 'sell' AND pnl IS NOT NULL AND timestamp >= date('now', ?1)",
    )?;

    let pnls: Vec<f64> = stmt.query_map([date_limit], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    let total_trades = pnls.len() as i32;
    let win_trades = pnls.iter().filter(|&&p| p > 0.0).count() as i32;
    let lose_trades = pnls.iter().filter(|&&p| p < 0.0).count() as i32;
    let total_pnl: f64 = pnls.iter().sum();

    let wins: Vec<f64> = pnls.iter().filter(|&&p| p > 0.0).copied().collect();
    let losses: Vec<f64> = pnls.iter().filter(|&&p| p < 0.0).copied().collect();

    let avg_profit = if wins.is_empty() { 0.0 } else { wins.iter().sum::<f64>() / wins.len() as f64 };
    let avg_loss = if losses.is_empty() { 0.0 } else { losses.iter().sum::<f64>() / losses.len() as f64 };

    let win_rate = if total_trades > 0 { win_trades as f64 / total_trades as f64 } else { 0.0 };
    let profit_factor = if avg_loss != 0.0 { -avg_profit / avg_loss } else { 0.0 };

    Ok(TradingStats {
        total_trades,
        win_trades,
        lose_trades,
        win_rate,
        total_pnl,
        avg_profit,
        avg_loss,
        profit_factor,
        max_drawdown: 0.0,
    })
}
