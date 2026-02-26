use anyhow::Result;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub type DbPool = Arc<Mutex<Connection>>;

pub fn init_db() -> Result<DbPool> {
    let conn = Connection::open("quantrust.db")?;
    
    conn.execute_batch("
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
        PRAGMA foreign_keys=ON;
    ")?;

    // Users table
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            hashed_password TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
    ")?;

    // Strategies table
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS strategies (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            name TEXT NOT NULL,
            description TEXT,
            code TEXT NOT NULL DEFAULT '',
            language TEXT NOT NULL DEFAULT 'python',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
    ")?;

    // Watchlist table
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS watchlist (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            symbol TEXT NOT NULL,
            name TEXT NOT NULL,
            group_name TEXT NOT NULL DEFAULT '默认',
            added_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(user_id, symbol)
        );
    ")?;

    // Alert rules table
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS alert_rules (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            name TEXT NOT NULL,
            symbol TEXT,
            rule_type TEXT NOT NULL,
            threshold REAL NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
    ")?;

    // Alert records table
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS alert_records (
            id TEXT PRIMARY KEY,
            rule_id TEXT,
            symbol TEXT NOT NULL,
            name TEXT NOT NULL,
            message TEXT NOT NULL,
            triggered_at TEXT NOT NULL DEFAULT (datetime('now')),
            read INTEGER NOT NULL DEFAULT 0
        );
    ")?;

    // Candles cache table
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS candles (
            symbol TEXT NOT NULL,
            period TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            open REAL NOT NULL,
            high REAL NOT NULL,
            low REAL NOT NULL,
            close REAL NOT NULL,
            volume REAL NOT NULL,
            turnover REAL NOT NULL DEFAULT 0,
            PRIMARY KEY (symbol, period, timestamp)
        );
    ")?;

    // Backtest results table
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS backtest_results (
            id TEXT PRIMARY KEY,
            strategy_id TEXT NOT NULL,
            params TEXT NOT NULL,
            kpis TEXT NOT NULL,
            trades TEXT NOT NULL,
            equity_curve TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
    ")?;

    tracing::info!("Database initialized successfully");
    Ok(Arc::new(Mutex::new(conn)))
}
