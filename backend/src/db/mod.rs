use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub type DbPool = Arc<Mutex<Connection>>;

pub fn init_db() -> Result<DbPool> {
    init_db_at("quantrust.db")
}

pub fn init_db_at<P: AsRef<Path>>(path: P) -> Result<DbPool> {
    let conn = Connection::open(path)?;
    configure_db(&conn)?;
    tracing::info!("Database initialized successfully");
    Ok(Arc::new(Mutex::new(conn)))
}

fn configure_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
        PRAGMA foreign_keys=ON;
    ",
    )?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL,
            email TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

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

        CREATE TABLE IF NOT EXISTS strategy_versions (
            id TEXT PRIMARY KEY,
            strategy_id TEXT NOT NULL,
            version INTEGER NOT NULL,
            code TEXT NOT NULL DEFAULT '',
            description TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS watchlist (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            symbol TEXT NOT NULL,
            name TEXT NOT NULL,
            group_name TEXT NOT NULL DEFAULT '默认',
            added_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(user_id, symbol)
        );

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

        CREATE TABLE IF NOT EXISTS alert_records (
            id TEXT PRIMARY KEY,
            rule_id TEXT,
            symbol TEXT NOT NULL,
            name TEXT NOT NULL,
            message TEXT NOT NULL,
            triggered_at TEXT NOT NULL DEFAULT (datetime('now')),
            read INTEGER NOT NULL DEFAULT 0
        );

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

        CREATE TABLE IF NOT EXISTS backtest_results (
            id TEXT PRIMARY KEY,
            strategy_id TEXT NOT NULL,
            params TEXT NOT NULL,
            kpis TEXT NOT NULL,
            trades TEXT NOT NULL,
            equity_curve TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS sim_leaderboard (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            initial_capital REAL NOT NULL DEFAULT 1000000.0,
            current_capital REAL NOT NULL,
            total_return REAL NOT NULL DEFAULT 0.0,
            return_rate REAL NOT NULL DEFAULT 0.0,
            total_trades INTEGER NOT NULL DEFAULT 0,
            win_count INTEGER NOT NULL DEFAULT 0,
            loss_count INTEGER NOT NULL DEFAULT 0,
            win_rate REAL NOT NULL DEFAULT 0.0,
            positions_count INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(username)
        );

        CREATE TABLE IF NOT EXISTS screener_templates (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            name TEXT NOT NULL,
            description TEXT,
            definition_json TEXT NOT NULL,
            source_type TEXT NOT NULL DEFAULT 'manual',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS screener_runs (
            id TEXT PRIMARY KEY,
            template_id TEXT,
            definition_json TEXT NOT NULL,
            result_count INTEGER NOT NULL DEFAULT 0,
            warning_json TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
    ",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Row;
    use std::path::PathBuf;

    fn temp_db_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("quantrust-{}-{}.db", name, uuid::Uuid::new_v4()));
        path
    }

    #[test]
    fn screener_templates_table_exists_after_init() {
        let db_path = temp_db_path("screener-templates");
        let pool = init_db_at(&db_path).expect("db init should succeed");
        let conn = pool.lock().unwrap();

        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(1) FROM sqlite_master WHERE type = 'table' AND name = 'screener_templates'",
                [],
                |row: &Row<'_>| row.get(0),
            )
            .unwrap();

        drop(conn);
        std::fs::remove_file(db_path).ok();

        assert_eq!(exists, 1);
    }
}
