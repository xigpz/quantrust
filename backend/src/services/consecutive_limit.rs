use anyhow::Result;
use rusqlite::params;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::db::DbPool;
use crate::models::{LimitUpHistory, ConsecutiveLimitUp, StockQuote};

/// 连板天梯服务
pub struct ConsecutiveLimitService {
    db: DbPool,
    /// 缓存的连板数据
    cache: Arc<RwLock<Vec<ConsecutiveLimitUp>>>,
}

impl ConsecutiveLimitService {
    pub fn new(db: DbPool) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 获取缓存的连板数据（优先内存，内存为空则从数据库加载）
    pub async fn get_cache(&self) -> Vec<ConsecutiveLimitUp> {
        let cache = self.cache.read().await.clone();
        if !cache.is_empty() {
            return cache;
        }
        // 内存缓存为空，尝试从数据库加载
        if let Ok(db_cache) = self.load_cache_from_db().await {
            if !db_cache.is_empty() {
                *self.cache.write().await = db_cache.clone();
                tracing::info!("Loaded consecutive limit-up cache from database: {} stocks", db_cache.len());
                return db_cache;
            }
        }
        cache
    }

    /// 从数据库加载缓存（异步版本）
    async fn load_cache_from_db(&self) -> Result<Vec<ConsecutiveLimitUp>> {
        let conn = self.db.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT symbol, name, price, change_pct, consecutive_days, reason, turn_rate, amount FROM consecutive_limit_up_cache ORDER BY consecutive_days DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ConsecutiveLimitUp {
                symbol: row.get(0)?,
                name: row.get(1)?,
                price: row.get(2)?,
                change_pct: row.get(3)?,
                consecutive_days: row.get(4)?,
                reason: row.get(5)?,
                turn_rate: row.get(6)?,
                amount: row.get(7)?,
            })
        })?;

        let mut cache = Vec::new();
        for row in rows {
            if let Ok(item) = row {
                cache.push(item);
            }
        }

        Ok(cache)
    }

    /// 保存当日涨停快照
    pub async fn save_daily_limit_up(&self, limit_ups: &[StockQuote]) -> Result<()> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let conn = self.db.lock().unwrap();

        for stock in limit_ups {
            conn.execute(
                "INSERT OR REPLACE INTO limit_up_history (symbol, name, price, change_pct, date) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![stock.symbol, stock.name, stock.price, stock.change_pct, today],
            )?;
        }

        tracing::info!("Saved {} limit-up stocks for {}", limit_ups.len(), today);
        Ok(())
    }

    /// 计算连续涨停天数
    pub fn calculate_consecutive_days(&self, symbol: &str, current_date: &str) -> Result<i32> {
        let conn = self.db.lock().unwrap();

        // 获取最近30个交易日的涨停记录
        let mut stmt = conn.prepare(
            "SELECT date FROM limit_up_history WHERE symbol = ?1 ORDER BY date DESC LIMIT 30"
        )?;

        let dates: Vec<String> = stmt
            .query_map([symbol], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        if dates.is_empty() {
            return Ok(0);
        }

        // 检查是否连续涨停
        // 从最新日期开始，检查是否每天都涨停
        let mut consecutive_days = 0;
        let mut expected_date = current_date.to_string();

        for date in dates {
            // 简单检查：如果记录日期等于预期日期（或差1天，周末情况），则连续
            if Self::is_trading_day_sequence(&date, &expected_date) {
                consecutive_days += 1;
                expected_date = Self::prev_trading_day(&date);
            } else {
                break;
            }
        }

        Ok(consecutive_days)
    }

    /// 检查两个日期是否是连续的交易日（考虑周末）
    fn is_trading_day_sequence(date1: &str, date2: &str) -> bool {
        use chrono::{NaiveDate, Duration};

        let d1 = NaiveDate::parse_from_str(date1, "%Y-%m-%d").ok();
        let d2 = NaiveDate::parse_from_str(date2, "%Y-%m-%d").ok();

        match (d1, d2) {
            (Some(d1), Some(d2)) => {
                let diff = (d2 - d1).num_days();
                // 同一天或差1天（正常工作日），或差3天（跨周末）
                diff >= 0 && diff <= 3
            }
            _ => false,
        }
    }

    /// 获取前一个交易日
    fn prev_trading_day(date_str: &str) -> String {
        use chrono::{NaiveDate, Duration};

        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let prev = date - Duration::days(1);
            prev.format("%Y-%m-%d").to_string()
        } else {
            date_str.to_string()
        }
    }

    /// 更新连板缓存数据
    pub async fn update_consecutive_cache(&self, limit_ups: &[StockQuote], reasons: &std::collections::HashMap<String, String>) -> Result<()> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let mut consecutive_list = Vec::new();

        for stock in limit_ups {
            let consecutive_days = self.calculate_consecutive_days(&stock.symbol, &today)?;
            if consecutive_days >= 2 {
                let reason = reasons.get(&stock.symbol).cloned().unwrap_or_default();
                let item = ConsecutiveLimitUp {
                    symbol: stock.symbol.clone(),
                    name: stock.name.clone(),
                    price: stock.price,
                    change_pct: stock.change_pct,
                    consecutive_days,
                    reason,
                    turn_rate: stock.turnover_rate,
                    amount: stock.turnover,
                };
                consecutive_list.push(item);
            }
        }

        // 按连板天数降序排列
        consecutive_list.sort_by(|a, b| b.consecutive_days.cmp(&a.consecutive_days));

        // 保存到数据库缓存
        {
            let conn = self.db.lock().unwrap();
            for item in &consecutive_list {
                conn.execute(
                    "INSERT OR REPLACE INTO consecutive_limit_up_cache (symbol, name, price, change_pct, consecutive_days, reason, turn_rate, amount, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
                    params![item.symbol, item.name, item.price, item.change_pct, item.consecutive_days, item.reason, item.turn_rate, item.amount],
                )?;
            }
        }

        // 更新内存缓存
        *self.cache.write().await = consecutive_list;

        tracing::info!("Updated consecutive limit-up cache with {} stocks", limit_ups.len());
        Ok(())
    }
}
