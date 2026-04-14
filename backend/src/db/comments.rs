//! Comments database module for stock comments and market analysis
//! Uses SQLite for simplicity and reliability

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::db::DbPool;

/// Stock comment stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockComment {
    pub id: i64,
    pub symbol: String,
    pub name: String,
    pub content: String,
    pub nickname: String,
    pub pub_time: String,
    pub like_count: i32,
    pub reply_count: i32,
    pub sentiment: String,
    pub sentiment_score: f64,
    pub created_at: String,
}

/// Stock comment stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockCommentStats {
    pub symbol: String,
    pub name: String,
    pub comment_count: i64,
    pub positive_count: i64,
    pub negative_count: i64,
    pub neutral_count: i64,
    pub risk_level: String,
    pub risk_score: f64,
    pub sentiment: String,
    pub positive_ratio: f64,
    pub negative_ratio: f64,
}

/// Market-wide sentiment stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSentiment {
    pub total_comments: i64,
    pub positive_count: i64,
    pub negative_count: i64,
    pub neutral_count: i64,
    pub positive_ratio: f64,
    pub negative_ratio: f64,
    pub market_risk_level: String,
    pub market_risk_score: f64,
}

/// Initialize comments tables
pub fn init_comments_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS stock_comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            name TEXT NOT NULL,
            content TEXT NOT NULL,
            nickname TEXT NOT NULL DEFAULT '',
            pub_time TEXT NOT NULL,
            like_count INTEGER NOT NULL DEFAULT 0,
            reply_count INTEGER NOT NULL DEFAULT 0,
            sentiment TEXT NOT NULL DEFAULT 'neutral',
            sentiment_score REAL NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(symbol, content, pub_time)
        );

        CREATE INDEX IF NOT EXISTS idx_comments_symbol ON stock_comments(symbol);
        CREATE INDEX IF NOT EXISTS idx_comments_sentiment ON stock_comments(sentiment);
        CREATE INDEX IF NOT EXISTS idx_comments_created_at ON stock_comments(created_at);
        "#,
    )?;
    Ok(())
}

/// Insert a batch of comments
pub async fn insert_comments(db: &DbPool, comments: Vec<StockComment>) -> Result<()> {
    let conn = db.lock().unwrap();
    for comment in comments {
        conn.execute(
            r#"
            INSERT OR IGNORE INTO stock_comments (symbol, name, content, nickname, pub_time, like_count, reply_count, sentiment, sentiment_score, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'))
            "#,
            params![
                &comment.symbol,
                &comment.name,
                &comment.content,
                &comment.nickname,
                &comment.pub_time,
                comment.like_count,
                comment.reply_count,
                &comment.sentiment,
                comment.sentiment_score,
            ],
        )?;
    }
    Ok(())
}

/// Get all stock symbols that have comments
pub async fn get_stocks_with_comments(db: &DbPool) -> Result<Vec<(String, String, i64)>> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        r#"
        SELECT symbol, name, COUNT(*) as comment_count
        FROM stock_comments
        GROUP BY symbol, name
        ORDER BY comment_count DESC
        LIMIT 500
        "#,
    )?;

    let results = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;

    let mut stocks = Vec::new();
    for result in results {
        stocks.push(result?);
    }
    Ok(stocks)
}

/// Get recent comments for a symbol
pub async fn get_recent_comments(
    db: &DbPool,
    symbol: &str,
    limit: i32,
) -> Result<Vec<StockComment>> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        r#"
        SELECT id, symbol, name, content, nickname, pub_time, like_count, reply_count, sentiment, sentiment_score, created_at
        FROM stock_comments
        WHERE symbol = ?1
        ORDER BY created_at DESC
        LIMIT ?2
        "#,
    )?;

    let results = stmt.query_map(params![symbol, limit], |row| {
        Ok(StockComment {
            id: row.get(0)?,
            symbol: row.get(1)?,
            name: row.get(2)?,
            content: row.get(3)?,
            nickname: row.get(4)?,
            pub_time: row.get(5)?,
            like_count: row.get(6)?,
            reply_count: row.get(7)?,
            sentiment: row.get(8)?,
            sentiment_score: row.get(9)?,
            created_at: row.get(10)?,
        })
    })?;

    let mut comments = Vec::new();
    for result in results {
        comments.push(result?);
    }
    Ok(comments)
}

/// Get recent comments across all stocks
pub async fn get_all_recent_comments(
    db: &DbPool,
    limit: i32,
) -> Result<Vec<StockComment>> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        r#"
        SELECT id, symbol, name, content, nickname, pub_time, like_count, reply_count, sentiment, sentiment_score, created_at
        FROM stock_comments
        ORDER BY created_at DESC
        LIMIT ?1
        "#,
    )?;

    let results = stmt.query_map(params![limit], |row| {
        Ok(StockComment {
            id: row.get(0)?,
            symbol: row.get(1)?,
            name: row.get(2)?,
            content: row.get(3)?,
            nickname: row.get(4)?,
            pub_time: row.get(5)?,
            like_count: row.get(6)?,
            reply_count: row.get(7)?,
            sentiment: row.get(8)?,
            sentiment_score: row.get(9)?,
            created_at: row.get(10)?,
        })
    })?;

    let mut comments = Vec::new();
    for result in results {
        comments.push(result?);
    }
    Ok(comments)
}

/// Get stock comment stats
pub async fn get_stock_comment_stats(db: &DbPool) -> Result<Vec<StockCommentStats>> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        r#"
        SELECT
            symbol,
            name,
            COUNT(*) as comment_count,
            SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) as positive_count,
            SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) as negative_count,
            SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END) as neutral_count,
            CASE
                WHEN CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS REAL) / COUNT(*) > 0.4 THEN 'high'
                WHEN CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS REAL) / COUNT(*) > 0.25 THEN 'medium'
                ELSE 'low'
            END as risk_level,
            50 + (CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS REAL) / COUNT(*) * 50) as risk_score,
            CASE
                WHEN SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) >= SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END)
                     AND SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) >= SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END)
                THEN 'positive'
                WHEN SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) >= SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END)
                THEN 'negative'
                ELSE 'neutral'
            END as sentiment,
            CAST(SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) AS REAL) / COUNT(*) as positive_ratio,
            CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS REAL) / COUNT(*) as negative_ratio
        FROM stock_comments
        GROUP BY symbol, name
        HAVING COUNT(*) >= 10
        ORDER BY comment_count DESC
        LIMIT 500
        "#,
    )?;

    let results = stmt.query_map([], |row| {
        Ok(StockCommentStats {
            symbol: row.get(0)?,
            name: row.get(1)?,
            comment_count: row.get(2)?,
            positive_count: row.get(3)?,
            negative_count: row.get(4)?,
            neutral_count: row.get(5)?,
            risk_level: row.get(6)?,
            risk_score: row.get(7)?,
            sentiment: row.get(8)?,
            positive_ratio: row.get(9)?,
            negative_ratio: row.get(10)?,
        })
    })?;

    let mut stats = Vec::new();
    for result in results {
        stats.push(result?);
    }
    Ok(stats)
}

/// Get market-wide sentiment
pub async fn get_market_sentiment(db: &DbPool) -> Result<MarketSentiment> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        r#"
        SELECT
            COUNT(*) as total_comments,
            SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) as positive_count,
            SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) as negative_count,
            SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END) as neutral_count
        FROM stock_comments
        "#,
    )?;

    let result = stmt.query_row([], |row| {
        let total: i64 = row.get(0)?;
        let positive: i64 = row.get(1)?;
        let negative: i64 = row.get(2)?;
        let neutral: i64 = row.get(3)?;

        let positive_ratio = if total > 0 { positive as f64 / total as f64 } else { 0.0 };
        let negative_ratio = if total > 0 { negative as f64 / total as f64 } else { 0.0 };

        let market_risk_level = if negative_ratio > 0.35 {
            "high"
        } else if negative_ratio > 0.25 {
            "medium"
        } else {
            "low"
        };

        let market_risk_score = 50.0 + (negative_ratio * 50.0);

        Ok(MarketSentiment {
            total_comments: total,
            positive_count: positive,
            negative_count: negative,
            neutral_count: neutral,
            positive_ratio,
            negative_ratio,
            market_risk_level: market_risk_level.to_string(),
            market_risk_score,
        })
    })?;

    Ok(result)
}

/// Delete old comments (keep only last N days)
pub async fn cleanup_old_comments(db: &DbPool, days_to_keep: i32) -> Result<u64> {
    let conn = db.lock().unwrap();
    let affected = conn.execute(
        r#"
        DELETE FROM stock_comments
        WHERE created_at < datetime('now', ?1 || ' days')
        "#,
        params![-days_to_keep],
    )?;
    Ok(affected as u64)
}
