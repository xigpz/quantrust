//! PostgreSQL comments database module for stock comments and market analysis
//! Uses PostgreSQL for scalability and better performance

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::FromRow;
use std::sync::Arc;

/// Stock comment stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StockComment {
    pub id: i64,
    pub symbol: String,
    pub name: String,
    pub content: String,
    pub nickname: String,
    pub pub_time: DateTime<Utc>,
    pub like_count: i32,
    pub reply_count: i32,
    pub sentiment: String,
    pub sentiment_score: f64,
    pub created_at: DateTime<Utc>,
}

/// Stock comment stats
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

/// Stock basic info for comment tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StockInfo {
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub updated_at: DateTime<Utc>,
}

pub type PostgresPool = Arc<PgPool>;

/// Initialize PostgreSQL connection pool
pub async fn init_postgres_pool(database_url: &str) -> Result<PostgresPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .connect(database_url)
        .await?;

    // Run migrations
    run_migrations(&pool).await?;

    tracing::info!("PostgreSQL pool initialized successfully");
    Ok(Arc::new(pool))
}

/// Run database migrations
async fn run_migrations(pool: &PgPool) -> Result<()> {
    // Create stock_info table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS stock_info (
            symbol VARCHAR(20) PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            sector VARCHAR(50) NOT NULL DEFAULT 'unknown',
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create stock_comments table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS stock_comments (
            id BIGSERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL,
            name VARCHAR(100) NOT NULL,
            content TEXT NOT NULL,
            nickname VARCHAR(100) NOT NULL DEFAULT '',
            pub_time TIMESTAMPTZ NOT NULL,
            like_count INTEGER NOT NULL DEFAULT 0,
            reply_count INTEGER NOT NULL DEFAULT 0,
            sentiment VARCHAR(20) NOT NULL DEFAULT 'neutral',
            sentiment_score DOUBLE PRECISION NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create indexes for efficient queries
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_comments_symbol ON stock_comments(symbol);
        CREATE INDEX IF NOT EXISTS idx_comments_pub_time ON stock_comments(pub_time DESC);
        CREATE INDEX IF NOT EXISTS idx_comments_sentiment ON stock_comments(sentiment);
        CREATE INDEX IF NOT EXISTS idx_comments_created_at ON stock_comments(created_at DESC);
        "#,
    )
    .execute(pool)
    .await?;

    tracing::info!("PostgreSQL migrations completed");
    Ok(())
}

// ============== Stock Info Operations ==============

/// Insert or update stock info
pub async fn upsert_stock_info(
    pool: &PgPool,
    symbol: &str,
    name: &str,
    sector: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO stock_info (symbol, name, sector, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (symbol) DO UPDATE SET
            name = EXCLUDED.name,
            sector = EXCLUDED.sector,
            updated_at = NOW()
        "#,
    )
    .bind(symbol)
    .bind(name)
    .bind(sector)
    .execute(pool)
    .await?;
    Ok(())
}

// ============== Comment Operations ==============

/// Insert a single comment
pub async fn insert_comment(pool: &PgPool, comment: &StockComment) -> Result<i64> {
    let id: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO stock_comments (symbol, name, content, nickname, pub_time, like_count, reply_count, sentiment, sentiment_score, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
        ON CONFLICT DO NOTHING
        RETURNING id
        "#,
    )
    .bind(&comment.symbol)
    .bind(&comment.name)
    .bind(&comment.content)
    .bind(&comment.nickname)
    .bind(&comment.pub_time)
    .bind(comment.like_count)
    .bind(comment.reply_count)
    .bind(&comment.sentiment)
    .bind(comment.sentiment_score)
    .fetch_one(pool)
    .await?;

    Ok(id.0)
}

/// Insert multiple comments in a batch
pub async fn insert_comments_batch(pool: &PgPool, comments: &[StockComment]) -> Result<usize> {
    let mut inserted = 0;
    for comment in comments {
        if insert_comment(pool, comment).await.is_ok() {
            inserted += 1;
        }
    }
    Ok(inserted)
}

/// Get comment count for a symbol
pub async fn get_comment_count(pool: &PgPool, symbol: &str) -> Result<i64> {
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM stock_comments WHERE symbol = $1",
    )
    .bind(symbol)
    .fetch_one(pool)
    .await?;
    Ok(result.0)
}

/// Get all stock symbols that have comments
pub async fn get_stocks_with_comments(pool: &PgPool) -> Result<Vec<(String, String, i64)>> {
    let results: Vec<(String, String, i64)> = sqlx::query_as(
        r#"
        SELECT symbol, name, COUNT(*) as comment_count
        FROM stock_comments
        GROUP BY symbol, name
        ORDER BY comment_count DESC
        LIMIT 500
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(results)
}

/// Get recent comments for a symbol
pub async fn get_recent_comments(
    pool: &PgPool,
    symbol: &str,
    limit: i32,
) -> Result<Vec<StockComment>> {
    let comments: Vec<StockComment> = sqlx::query_as(
        r#"
        SELECT id, symbol, name, content, nickname, pub_time, like_count, reply_count, sentiment, sentiment_score, created_at
        FROM stock_comments
        WHERE symbol = $1
        ORDER BY pub_time DESC
        LIMIT $2
        "#,
    )
    .bind(symbol)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(comments)
}

/// Get recent comments across all stocks
pub async fn get_all_recent_comments(
    pool: &PgPool,
    limit: i32,
) -> Result<Vec<StockComment>> {
    let comments: Vec<StockComment> = sqlx::query_as(
        r#"
        SELECT id, symbol, name, content, nickname, pub_time, like_count, reply_count, sentiment, sentiment_score, created_at
        FROM stock_comments
        ORDER BY created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(comments)
}

/// Get stock comment stats (aggregated)
pub async fn get_stock_comment_stats(pool: &PgPool) -> Result<Vec<StockCommentStats>> {
    let stats: Vec<StockCommentStats> = sqlx::query_as(
        r#"
        SELECT
            symbol,
            name,
            COUNT(*) as comment_count,
            SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) as positive_count,
            SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) as negative_count,
            SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END) as neutral_count,
            CASE
                WHEN CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) > 0.4 THEN 'high'
                WHEN CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) > 0.25 THEN 'medium'
                ELSE 'low'
            END as risk_level,
            50 + (CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) * 50) as risk_score,
            CASE
                WHEN SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) >= SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END)
                     AND SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) >= SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END)
                THEN 'positive'
                WHEN SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) >= SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END)
                THEN 'negative'
                ELSE 'neutral'
            END as sentiment,
            CAST(SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) as positive_ratio,
            CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) as negative_ratio
        FROM stock_comments
        GROUP BY symbol, name
        HAVING COUNT(*) >= 10
        ORDER BY comment_count DESC
        LIMIT 500
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(stats)
}

/// Get market-wide sentiment
pub async fn get_market_sentiment(pool: &PgPool) -> Result<MarketSentiment> {
    let result: Option<MarketSentiment> = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) as total_comments,
            COALESCE(SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END), 0) as positive_count,
            COALESCE(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END), 0) as negative_count,
            COALESCE(SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END), 0) as neutral_count,
            CASE
                WHEN COUNT(*) > 0 AND COALESCE(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END), 0)::float / COUNT(*) > 0.35 THEN 'high'
                WHEN COUNT(*) > 0 AND COALESCE(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END), 0)::float / COUNT(*) > 0.25 THEN 'medium'
                ELSE 'low'
            END as market_risk_level,
            50 + (COALESCE(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END), 0)::float / NULLIF(COUNT(*), 0) * 50) as market_risk_score,
            0.0 as positive_ratio,
            0.0 as negative_ratio
        FROM stock_comments
        "#,
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.unwrap_or(MarketSentiment {
        total_comments: 0,
        positive_count: 0,
        negative_count: 0,
        neutral_count: 0,
        positive_ratio: 0.0,
        negative_ratio: 0.0,
        market_risk_level: "medium".to_string(),
        market_risk_score: 50.0,
    }))
}

/// Get total comment count
pub async fn get_total_comment_count(pool: &PgPool) -> Result<i64> {
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM stock_comments")
        .fetch_one(pool)
        .await?;
    Ok(result.0)
}

/// Delete old comments (keep only last N days)
pub async fn cleanup_old_comments(pool: &PgPool, days_to_keep: i32) -> Result<u64> {
    let result = sqlx::query(
        r#"
        DELETE FROM stock_comments
        WHERE created_at < NOW() - INTERVAL '1 day' * $1
        "#,
    )
    .bind(days_to_keep)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Get high risk stocks (negative_ratio > 0.35)
pub async fn get_high_risk_stocks(pool: &PgPool, min_count: i64) -> Result<Vec<StockCommentStats>> {
    let stats: Vec<StockCommentStats> = sqlx::query_as(
        r#"
        SELECT
            symbol,
            name,
            COUNT(*) as comment_count,
            SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) as positive_count,
            SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) as negative_count,
            SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END) as neutral_count,
            'high' as risk_level,
            50 + (CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) * 50) as risk_score,
            CASE
                WHEN SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) >= SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END)
                     AND SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) >= SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END)
                THEN 'positive'
                WHEN SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) >= SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END)
                THEN 'negative'
                ELSE 'neutral'
            END as sentiment,
            CAST(SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) as positive_ratio,
            CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) as negative_ratio
        FROM stock_comments
        GROUP BY symbol, name
        HAVING COUNT(*) >= $1 AND CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) > 0.35
        ORDER BY negative_ratio DESC, comment_count DESC
        LIMIT 50
        "#,
    )
    .bind(min_count)
    .fetch_all(pool)
    .await?;
    Ok(stats)
}

/// Get low risk stocks (negative_ratio < 0.2 and count >= 50)
pub async fn get_low_risk_stocks(pool: &PgPool, min_count: i64) -> Result<Vec<StockCommentStats>> {
    let stats: Vec<StockCommentStats> = sqlx::query_as(
        r#"
        SELECT
            symbol,
            name,
            COUNT(*) as comment_count,
            SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) as positive_count,
            SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) as negative_count,
            SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END) as neutral_count,
            'low' as risk_level,
            50 + (CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) * 50) as risk_score,
            CASE
                WHEN SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) >= SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END)
                     AND SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) >= SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END)
                THEN 'positive'
                WHEN SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) >= SUM(CASE WHEN sentiment = 'neutral' THEN 1 ELSE 0 END)
                THEN 'negative'
                ELSE 'neutral'
            END as sentiment,
            CAST(SUM(CASE WHEN sentiment = 'positive' THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) as positive_ratio,
            CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) as negative_ratio
        FROM stock_comments
        GROUP BY symbol, name
        HAVING COUNT(*) >= $1 AND CAST(SUM(CASE WHEN sentiment = 'negative' THEN 1 ELSE 0 END) AS FLOAT) / COUNT(*) < 0.2
        ORDER BY positive_ratio DESC, comment_count DESC
        LIMIT 50
        "#,
    )
    .bind(min_count)
    .fetch_all(pool)
    .await?;
    Ok(stats)
}
