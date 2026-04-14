//! Comment collector service
//! Continuously fetches stock comments from data sources and stores them in PostgreSQL

use crate::db::postgres_comments::{self, PostgresPool, StockComment};
use crate::data::DataProvider;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// Comment collector state
pub struct CommentCollector {
    pool: PostgresPool,
    provider: Arc<DataProvider>,
    last_fetch: RwLock<Option<chrono::DateTime<Utc>>>,
}

impl CommentCollector {
    pub fn new(pool: PostgresPool, provider: Arc<DataProvider>) -> Self {
        Self {
            pool,
            provider,
            last_fetch: RwLock::new(None),
        }
    }

    /// Start the background comment collection task
    pub async fn start(&self) {
        let pool = self.pool.clone();
        let provider = self.provider.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(120)); // Fetch every 2 minutes

            loop {
                ticker.tick().await;

                tracing::info!("Starting comment collection cycle...");

                if let Err(e) = fetch_and_store_comments(&pool, &provider).await {
                    tracing::error!("Comment collection failed: {}", e);
                } else {
                    tracing::info!("Comment collection completed successfully");
                }
            }
        });
    }
}

/// Fetch comments from provider and store in PostgreSQL
async fn fetch_and_store_comments(pool: &PostgresPool, provider: &Arc<DataProvider>) -> Result<(), anyhow::Error> {
    // Get market comments from provider (this fetches from EastMoney)
    let market_comments = provider.get_market_comments(300).await?;

    let total_stored = market_comments.hot_stocks.len();
    let mut total_comments = 0i64;

    for stock_summary in market_comments.hot_stocks {
        // Fetch detailed comments for this stock
        let comments_result = provider.get_stock_comments(&stock_summary.symbol, 1, 100).await;

        if let Ok(comments_data) = comments_result {
            for comment in comments_data.list {
                let stock_comment = StockComment {
                    id: 0, // Auto-generated
                    symbol: stock_summary.symbol.clone(),
                    name: stock_summary.name.clone(),
                    content: comment.content,
                    nickname: comment.nickname,
                    pub_time: Utc::now(),
                    like_count: comment.like_count,
                    reply_count: comment.reply_count,
                    sentiment: comment.sentiment,
                    sentiment_score: comment.sentiment_score,
                    created_at: Utc::now(),
                };

                if postgres_comments::insert_comment(pool.as_ref(), &stock_comment).await.is_ok() {
                    total_comments += 1;
                }
            }
        }
    }

    tracing::info!("Stored {} comments for {} stocks", total_comments, total_stored);
    Ok(())
}

/// Initialize comment collector with existing pool
pub fn init_comment_collector(pool: PostgresPool, provider: Arc<DataProvider>) -> CommentCollector {
    CommentCollector::new(pool, provider)
}
