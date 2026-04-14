use anyhow::{Result, anyhow};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::db::DbPool;
use crate::models::portfolio::*;

/// 组合服务
pub struct PortfolioService {
    db: DbPool,
}

impl PortfolioService {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// 创建组合
    pub fn create_portfolio(
        &self,
        user_id: &str,
        req: &CreatePortfolioRequest,
    ) -> Result<Portfolio> {
        let id = Uuid::new_v4().to_string();
        let initial_capital = req.initial_capital.unwrap_or(1_000_000.0);
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        db.execute(
            "INSERT INTO portfolios (id, user_id, name, description, initial_capital, current_capital, total_value, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &id,
                user_id,
                &req.name,
                req.description.as_deref().unwrap_or(""),
                initial_capital,
                initial_capital,
                initial_capital,
                &now,
                &now
            ],
        )?;

        Ok(Portfolio {
            id,
            user_id: user_id.to_string(),
            name: req.name.clone(),
            description: req.description.clone(),
            initial_capital,
            current_capital: initial_capital,
            total_value: initial_capital,
            total_return_rate: 0.0,
            positions_value: 0.0,
            positions_count: 0,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// 获取用户的所有组合
    pub fn get_user_portfolios(&self, user_id: &str) -> Result<Vec<Portfolio>> {
        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = db.prepare(
            "SELECT id, user_id, name, description, initial_capital, current_capital, total_value, 
                    total_return_rate, positions_value, positions_count, created_at, updated_at 
             FROM portfolios WHERE user_id = ?1 ORDER BY created_at DESC"
        )?;

        let portfolios = stmt
            .query_map(params![user_id], |row| {
                Ok(Portfolio {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    initial_capital: row.get(4)?,
                    current_capital: row.get(5)?,
                    total_value: row.get(6)?,
                    total_return_rate: row.get(7)?,
                    positions_value: row.get(8)?,
                    positions_count: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(portfolios)
    }

    /// 获取组合详情
    pub fn get_portfolio(&self, portfolio_id: &str, user_id: &str) -> Result<Option<Portfolio>> {
        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = db.prepare(
            "SELECT id, user_id, name, description, initial_capital, current_capital, total_value, 
                    total_return_rate, positions_value, positions_count, created_at, updated_at 
             FROM portfolios WHERE id = ?1 AND user_id = ?2"
        )?;

        let portfolio = stmt
            .query_row(params![portfolio_id, user_id], |row| {
                Ok(Portfolio {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    initial_capital: row.get(4)?,
                    current_capital: row.get(5)?,
                    total_value: row.get(6)?,
                    total_return_rate: row.get(7)?,
                    positions_value: row.get(8)?,
                    positions_count: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })
            .ok();

        Ok(portfolio)
    }

    /// 删除组合
    pub fn delete_portfolio(&self, portfolio_id: &str, user_id: &str) -> Result<bool> {
        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        // 开启事务
        let tx = db.unchecked_transaction()?;

        // 删除相关的持仓、交易记录、日志、收益记录
        tx.execute("DELETE FROM portfolio_positions WHERE portfolio_id = ?1", params![portfolio_id])?;
        tx.execute("DELETE FROM portfolio_trades WHERE portfolio_id = ?1", params![portfolio_id])?;
        tx.execute("DELETE FROM portfolio_logs WHERE portfolio_id = ?1", params![portfolio_id])?;
        tx.execute("DELETE FROM portfolio_returns WHERE portfolio_id = ?1", params![portfolio_id])?;

        // 删除组合
        let rows = tx.execute(
            "DELETE FROM portfolios WHERE id = ?1 AND user_id = ?2",
            params![portfolio_id, user_id],
        )?;

        tx.commit()?;

        Ok(rows > 0)
    }

    /// 买入股票
    pub fn buy_stock(&self, portfolio_id: &str, req: &BuyStockRequest) -> Result<PortfolioTrade> {
        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let tx = db.unchecked_transaction()?;

        // 1. 检查组合是否存在并获取当前资金
        let portfolio: Option<(f64, f64)> = tx.query_row(
            "SELECT current_capital, total_value FROM portfolios WHERE id = ?1",
            params![portfolio_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).ok();

        let (current_capital, total_value) = portfolio.ok_or_else(|| anyhow!("组合不存在"))?;

        // 2. 计算成交金额和手续费（万分之三）
        let amount = req.price * req.quantity;
        let commission = amount * 0.0003;
        let total_cost = amount + commission;

        // 3. 检查资金是否足够
        if total_cost > current_capital {
            return Err(anyhow!("资金不足，需要 {:.2}，可用 {:.2}", total_cost, current_capital));
        }

        // 4. 获取或创建持仓
        let position: Option<(String, f64, f64)> = tx.query_row(
            "SELECT id, quantity, avg_cost FROM portfolio_positions WHERE portfolio_id = ?1 AND symbol = ?2",
            params![portfolio_id, &req.symbol],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).ok();

        let now = Utc::now();
        let trade_date = req.trade_date.clone();
        let trade_time = now.format("%H:%M:%S").to_string();
        let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let (position_before, position_after, avg_cost_after): (f64, f64, f64);

        if let Some((pos_id, old_quantity, old_avg_cost)) = position {
            // 更新已有持仓
            position_before = old_quantity;
            position_after = old_quantity + req.quantity;
            let total_cost_old = old_avg_cost * old_quantity;
            let total_cost_new = total_cost_old + amount;
            avg_cost_after = total_cost_new / position_after;

            tx.execute(
                "UPDATE portfolio_positions SET 
                    quantity = ?1, avg_cost = ?2, last_trade_date = ?3, updated_at = ?4
                 WHERE id = ?5",
                params![position_after, avg_cost_after, &trade_date, &now_str, &pos_id],
            )?;
        } else {
            // 新建持仓
            position_before = 0.0;
            position_after = req.quantity;
            avg_cost_after = req.price;

            let pos_id = Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO portfolio_positions 
                    (id, portfolio_id, symbol, name, quantity, avg_cost, first_buy_date, last_trade_date, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    &pos_id, portfolio_id, &req.symbol, &req.name, req.quantity, 
                    avg_cost_after, &trade_date, &trade_date, &now_str
                ],
            )?;
        }

        // 5. 计算权重变化
        let weight_before = position_before * req.price / total_value;
        let weight_after = position_after * req.price / (total_value - commission);

        // 6. 创建交易记录
        let trade_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO portfolio_trades 
                (id, portfolio_id, symbol, name, direction, price, quantity, amount, commission,
                 position_before, position_after, weight_before, weight_after, reason, trade_date, trade_time, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                &trade_id, portfolio_id, &req.symbol, &req.name, "buy", req.price, req.quantity,
                amount, commission, position_before, position_after, weight_before, weight_after,
                req.reason.as_deref().unwrap_or(""), &trade_date, &trade_time, &now_str
            ],
        )?;

        // 7. 更新组合资金
        let new_capital = current_capital - total_cost;
        tx.execute(
            "UPDATE portfolios SET current_capital = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_capital, &now_str, portfolio_id],
        )?;

        tx.commit()?;

        Ok(PortfolioTrade {
            id: trade_id,
            portfolio_id: portfolio_id.to_string(),
            symbol: req.symbol.clone(),
            name: req.name.clone(),
            direction: TradeDirection::Buy,
            price: req.price,
            quantity: req.quantity,
            amount,
            commission,
            position_before: Some(position_before),
            position_after: Some(position_after),
            weight_before: Some(weight_before),
            weight_after: Some(weight_after),
            reason: req.reason.clone(),
            trade_date,
            trade_time,
            created_at: now_str,
        })
    }

    /// 卖出股票
    pub fn sell_stock(&self, portfolio_id: &str, req: &SellStockRequest) -> Result<PortfolioTrade> {
        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let tx = db.unchecked_transaction()?;

        // 1. 获取当前持仓
        let position: Option<(String, f64, f64, String)> = tx.query_row(
            "SELECT id, quantity, avg_cost, name FROM portfolio_positions 
             WHERE portfolio_id = ?1 AND symbol = ?2",
            params![portfolio_id, &req.symbol],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).ok();

        let (pos_id, position_before, avg_cost, name) = position.ok_or_else(|| anyhow!("没有该股票持仓"))?;

        // 2. 检查持仓是否足够
        if req.quantity > position_before {
            return Err(anyhow!(
                "持仓不足，持有 {:.0}，试图卖出 {:.0}",
                position_before,
                req.quantity
            ));
        }

        // 3. 计算成交金额和手续费
        let amount = req.price * req.quantity;
        let commission = amount * 0.0003;
        let net_amount = amount - commission;

        // 4. 获取组合信息
        let (current_capital, total_value): (f64, f64) = tx.query_row(
            "SELECT current_capital, total_value FROM portfolios WHERE id = ?1",
            params![portfolio_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| anyhow!("组合不存在"))?;

        // 5. 更新或删除持仓
        let position_after = position_before - req.quantity;
        let now = Utc::now();
        let trade_date = req.trade_date.clone();
        let trade_time = now.format("%H:%M:%S").to_string();
        let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        if position_after > 0.0 {
            // 部分卖出，保留持仓
            tx.execute(
                "UPDATE portfolio_positions SET 
                    quantity = ?1, last_trade_date = ?2, updated_at = ?3
                 WHERE id = ?4",
                params![position_after, &trade_date, &now_str, &pos_id],
            )?;
        } else {
            // 全部卖出，删除持仓
            tx.execute(
                "DELETE FROM portfolio_positions WHERE id = ?1",
                params![&pos_id],
            )?;
        }

        // 6. 计算权重变化
        let weight_before = position_before * req.price / total_value;
        let weight_after = if position_after > 0.0 {
            position_after * req.price / (total_value + net_amount)
        } else {
            0.0
        };

        // 7. 创建交易记录
        let trade_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO portfolio_trades 
                (id, portfolio_id, symbol, name, direction, price, quantity, amount, commission,
                 position_before, position_after, weight_before, weight_after, reason, trade_date, trade_time, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                &trade_id, portfolio_id, &req.symbol, &name, "sell", req.price, req.quantity,
                amount, commission, position_before, position_after, weight_before, weight_after,
                req.reason.as_deref().unwrap_or(""), &trade_date, &trade_time, &now_str
            ],
        )?;

        // 8. 更新组合资金
        let new_capital = current_capital + net_amount;
        tx.execute(
            "UPDATE portfolios SET current_capital = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_capital, &now_str, portfolio_id],
        )?;

        tx.commit()?;

        Ok(PortfolioTrade {
            id: trade_id,
            portfolio_id: portfolio_id.to_string(),
            symbol: req.symbol.clone(),
            name,
            direction: TradeDirection::Sell,
            price: req.price,
            quantity: req.quantity,
            amount,
            commission,
            position_before: Some(position_before),
            position_after: Some(position_after),
            weight_before: Some(weight_before),
            weight_after: Some(weight_after),
            reason: req.reason.clone(),
            trade_date,
            trade_time,
            created_at: now_str,
        })
    }

    /// 获取持仓列表
    pub fn get_positions(&self, portfolio_id: &str) -> Result<Vec<PortfolioPosition>> {
        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = db.prepare(
            "SELECT id, portfolio_id, symbol, name, quantity, avg_cost, current_price, 
                    market_value, total_profit, profit_rate, weight, first_buy_date, 
                    last_trade_date, updated_at 
             FROM portfolio_positions 
             WHERE portfolio_id = ?1 AND quantity > 0
             ORDER BY market_value DESC"
        )?;

        let positions = stmt
            .query_map(params![portfolio_id], |row| {
                Ok(PortfolioPosition {
                    id: row.get(0)?,
                    portfolio_id: row.get(1)?,
                    symbol: row.get(2)?,
                    name: row.get(3)?,
                    quantity: row.get(4)?,
                    avg_cost: row.get(5)?,
                    current_price: row.get(6)?,
                    market_value: row.get(7)?,
                    total_profit: row.get(8)?,
                    profit_rate: row.get(9)?,
                    weight: row.get(10)?,
                    first_buy_date: row.get(11)?,
                    last_trade_date: row.get(12)?,
                    updated_at: row.get(13)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(positions)
    }

    /// 获取调仓记录
    pub fn get_trades(
        &self,
        portfolio_id: &str,
        params: &TradeQueryParams,
    ) -> Result<Vec<PortfolioTrade>> {
        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let page = params.page.unwrap_or(1);
        let page_size = params.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        let mut sql = String::from(
            "SELECT id, portfolio_id, symbol, name, direction, price, quantity, amount, commission,
                    position_before, position_after, weight_before, weight_after, reason, 
                    trade_date, trade_time, created_at 
             FROM portfolio_trades 
             WHERE portfolio_id = ?1"
        );

        if let Some(ref symbol) = params.symbol {
            sql.push_str(&format!(" AND symbol = '{}'", symbol));
        }
        if let Some(ref start) = params.start_date {
            sql.push_str(&format!(" AND trade_date >= '{}'", start));
        }
        if let Some(ref end) = params.end_date {
            sql.push_str(&format!(" AND trade_date <= '{}'", end));
        }

        sql.push_str(" ORDER BY trade_date DESC, trade_time DESC LIMIT ?2 OFFSET ?3");

        let mut stmt = db.prepare(&sql)?;

        let trades = stmt
            .query_map(params![portfolio_id, page_size, offset], |row| {
                let direction_str: String = row.get(4)?;
                Ok(PortfolioTrade {
                    id: row.get(0)?,
                    portfolio_id: row.get(1)?,
                    symbol: row.get(2)?,
                    name: row.get(3)?,
                    direction: TradeDirection::from_str(&direction_str)
                        .unwrap_or(TradeDirection::Buy),
                    price: row.get(5)?,
                    quantity: row.get(6)?,
                    amount: row.get(7)?,
                    commission: row.get(8)?,
                    position_before: row.get(9)?,
                    position_after: row.get(10)?,
                    weight_before: row.get(11)?,
                    weight_after: row.get(12)?,
                    reason: row.get(13)?,
                    trade_date: row.get(14)?,
                    trade_time: row.get(15)?,
                    created_at: row.get(16)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(trades)
    }

    /// 更新持仓价格和市值（由外部定时任务调用）
    pub fn update_positions_price(
        &self,
        portfolio_id: &str,
        prices: &[(String, f64)], // (symbol, price)
    ) -> Result<()> {
        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let tx = db.unchecked_transaction()?;

        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        for (symbol, price) in prices {
            // 获取当前持仓
            let position: Option<(String, f64, f64)> = tx.query_row(
                "SELECT id, quantity, avg_cost FROM portfolio_positions 
                 WHERE portfolio_id = ?1 AND symbol = ?2",
                params![portfolio_id, symbol],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            ).ok();

            if let Some((pos_id, quantity, avg_cost)) = position {
                let market_value = quantity * price;
                let total_profit = (price - avg_cost) * quantity;
                let profit_rate = if avg_cost > 0.0 {
                    (price - avg_cost) / avg_cost * 100.0
                } else {
                    0.0
                };

                tx.execute(
                    "UPDATE portfolio_positions SET 
                        current_price = ?1, market_value = ?2, total_profit = ?3, 
                        profit_rate = ?4, updated_at = ?5
                     WHERE id = ?6",
                    params![price, market_value, total_profit, profit_rate, &now, &pos_id],
                )?;
            }
        }

        // 更新组合总市值和持仓数量
        let positions_value: f64 = tx.query_row(
            "SELECT COALESCE(SUM(market_value), 0) FROM portfolio_positions 
             WHERE portfolio_id = ?1",
            params![portfolio_id],
            |row| row.get(0),
        )?;

        let positions_count: i32 = tx.query_row(
            "SELECT COUNT(*) FROM portfolio_positions 
             WHERE portfolio_id = ?1 AND quantity > 0",
            params![portfolio_id],
            |row| row.get(0),
        )?;

        let (current_capital, initial_capital): (f64, f64) = tx.query_row(
            "SELECT current_capital, initial_capital FROM portfolios WHERE id = ?1",
            params![portfolio_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let total_value = current_capital + positions_value;
        let total_return_rate = if initial_capital > 0.0 {
            (total_value - initial_capital) / initial_capital * 100.0
        } else {
            0.0
        };

        // 更新权重
        if total_value > 0.0 {
            tx.execute(
                "UPDATE portfolio_positions SET weight = market_value / ?1 
                 WHERE portfolio_id = ?2",
                params![total_value, portfolio_id],
            )?;
        }

        tx.execute(
            "UPDATE portfolios SET 
                total_value = ?1, positions_value = ?2, positions_count = ?3,
                total_return_rate = ?4, updated_at = ?5
             WHERE id = ?6",
            params![total_value, positions_value, positions_count, total_return_rate, &now, portfolio_id],
        )?;

        tx.commit()?;
        Ok(())
    }

    /// 记录每日收益
    pub fn record_daily_return(&self, portfolio_id: &str) -> Result<()> {
        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let today = Utc::now().format("%Y-%m-%d").to_string();

        // 检查今天是否已记录
        let exists: bool = db.query_row(
            "SELECT 1 FROM portfolio_returns WHERE portfolio_id = ?1 AND date = ?2",
            params![portfolio_id, &today],
            |_| Ok(true),
        ).unwrap_or(false);

        if exists {
            return Ok(());
        }

        // 获取组合当前数据
        let (total_value, cash, positions_value, total_return_rate): (f64, f64, f64, f64) = db.query_row(
            "SELECT total_value, current_capital, positions_value, total_return_rate 
             FROM portfolios WHERE id = ?1",
            params![portfolio_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;

        // 计算日收益
        let daily_return = db.query_row(
            "SELECT total_value FROM portfolio_returns 
             WHERE portfolio_id = ?1 ORDER BY date DESC LIMIT 1",
            params![portfolio_id],
            |row| -> rusqlite::Result<f64> {
                let prev_value: f64 = row.get(0)?;
                Ok(total_value - prev_value)
            },
        ).unwrap_or(0.0);

        db.execute(
            "INSERT INTO portfolio_returns 
                (portfolio_id, date, total_value, cash, positions_value, daily_return, total_return_rate)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![portfolio_id, &today, total_value, cash, positions_value, daily_return, total_return_rate],
        )?;

        Ok(())
    }

    /// 获取收益走势
    pub fn get_returns(
        &self,
        portfolio_id: &str,
        params: &ReturnQueryParams,
    ) -> Result<Vec<PortfolioReturn>> {
        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let period = params.period.as_deref().unwrap_or("all");
        let limit = match period {
            "1d" => 1,
            "5d" => 5,
            "20d" => 20,
            "60d" => 60,
            "250d" => 250,
            _ => 10000, // all
        };

        let mut stmt = db.prepare(
            "SELECT id, portfolio_id, date, total_value, cash, positions_value, 
                    daily_return, total_return_rate, benchmark_return 
             FROM portfolio_returns 
             WHERE portfolio_id = ?1 
             ORDER BY date DESC LIMIT ?2"
        )?;

        let returns: Vec<PortfolioReturn> = stmt
            .query_map(params![portfolio_id, limit], |row| {
                Ok(PortfolioReturn {
                    id: row.get(0)?,
                    portfolio_id: row.get(1)?,
                    date: row.get(2)?,
                    total_value: row.get(3)?,
                    cash: row.get(4)?,
                    positions_value: row.get(5)?,
                    daily_return: row.get(6)?,
                    total_return_rate: row.get(7)?,
                    benchmark_return: row.get(8)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        // 倒序排列（从早到晚）
        let mut returns = returns;
        returns.reverse();

        Ok(returns)
    }

    /// 获取组合统计信息
    pub fn get_portfolio_stats(&self, portfolio_id: &str) -> Result<PortfolioStats> {
        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        // 获取基础信息
        let (initial_capital, total_value, created_at): (f64, f64, String) = db.query_row(
            "SELECT initial_capital, total_value, created_at FROM portfolios WHERE id = ?1",
            params![portfolio_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        let total_return = total_value - initial_capital;
        let total_return_rate = if initial_capital > 0.0 {
            total_return / initial_capital * 100.0
        } else {
            0.0
        };

        // 计算持仓天数和年化收益
        let days_elapsed = chrono::Utc::now()
            .signed_duration_since(chrono::DateTime::parse_from_rfc3339(&created_at).unwrap_or_else(|_| chrono::Utc::now().into()))
            .num_days()
            .max(1) as f64;
        let years = days_elapsed / 365.0;
        let annualized_return = if years > 0.0 && initial_capital > 0.0 {
            ((total_value / initial_capital).powf(1.0 / years) - 1.0) * 100.0
        } else {
            0.0
        };

        // 获取交易统计
        let total_trades: i64 = db.query_row(
            "SELECT COUNT(*) FROM portfolio_trades WHERE portfolio_id = ?1",
            params![portfolio_id],
            |row| row.get(0),
        )?;

        // 计算胜率（基于买入交易）
        let buy_trades: i64 = db.query_row(
            "SELECT COUNT(*) FROM portfolio_trades WHERE portfolio_id = ?1 AND direction = 'buy'",
            params![portfolio_id],
            |row| row.get(0),
        ).unwrap_or(0);

        // 简化估算胜率
        let win_trades = buy_trades / 2;
        let win_rate = if total_trades > 0 {
            (win_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        // 计算日收益（最近一天）
        let (daily_return, daily_return_rate): (f64, f64) = db.query_row(
            "SELECT COALESCE(daily_return, 0),
                    CASE WHEN total_value - daily_return > 0
                         THEN daily_return / (total_value - daily_return) * 100
                         ELSE 0 END
             FROM portfolio_returns
             WHERE portfolio_id = ?1 ORDER BY date DESC LIMIT 1",
            params![portfolio_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap_or((0.0, 0.0));

        // 计算最大回撤
        let mut returns: Vec<f64> = Vec::new();
        let mut stmt = db.prepare(
            "SELECT total_return_rate FROM portfolio_returns WHERE portfolio_id = ?1 ORDER BY date ASC"
        )?;
        let rows = stmt.query_map(params![portfolio_id], |row| row.get(0))?;
        for row in rows {
            if let Ok(r) = row {
                returns.push(r);
            }
        }

        let max_drawdown = calculate_max_drawdown(&returns);

        // 计算波动率（年化）
        let volatility = calculate_volatility(&returns);

        // 计算夏普比率 (假设无风险利率为3%)
        let risk_free_rate = 3.0;
        let sharpe_ratio = if volatility > 0.0 {
            (annualized_return - risk_free_rate) / volatility
        } else {
            0.0
        };

        // 计算基准收益（简化：假设HS300同期收益为组合收益的0.7倍作为估算）
        let benchmark_return = total_return_rate * 0.7;
        let alpha = annualized_return - benchmark_return - risk_free_rate;

        // 计算Beta（简化估算）
        let beta = if benchmark_return != 0.0 {
            (total_return_rate / benchmark_return).max(0.5).min(2.0)
        } else {
            1.0
        };

        // 获取持仓集中度（最大仓位占比）
        let max_position_weight: f64 = db.query_row(
            "SELECT COALESCE(MAX(weight), 0) FROM portfolio_positions WHERE portfolio_id = ?1",
            params![portfolio_id],
            |row| row.get(0),
        ).unwrap_or(0.0);

        // 计算连胜连亏
        let (win_streak, lose_streak) = calculate_streaks(&returns);

        Ok(PortfolioStats {
            total_return,
            total_return_rate,
            daily_return,
            daily_return_rate,
            total_trades,
            win_trades,
            loss_trades: total_trades.saturating_sub(win_trades),
            win_rate,
            max_drawdown,
            sharpe_ratio,
            annualized_return,
            volatility,
            benchmark_return,
            alpha,
            beta,
            position_concentration: max_position_weight * 100.0,
            win_streak,
            lose_streak,
        })
    }

    /// 获取月度收益
    pub fn get_monthly_returns(&self, portfolio_id: &str) -> Result<Vec<MonthlyReturn>> {
        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = db.prepare(
            "SELECT date, total_value, benchmark_return FROM portfolio_returns
             WHERE portfolio_id = ?1 ORDER BY date ASC"
        )?;

        let returns: Vec<(String, f64, f64)> = stmt
            .query_map(params![portfolio_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        // 按月分组计算
        let mut monthly_map: std::collections::HashMap<String, Vec<(String, f64, f64)>> = std::collections::HashMap::new();
        for (date, value, benchmark) in returns {
            if let (Some(year), Some(month)) = (date.get(0..4), date.get(5..7)) {
                let key = format!("{}-{}", year, month);
                monthly_map.entry(key).or_default().push((date, value, benchmark));
            }
        }

        let mut result: Vec<MonthlyReturn> = Vec::new();
        let mut sorted_keys: Vec<_> = monthly_map.keys().cloned().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            if let Some(month_data) = monthly_map.get(&key) {
                if month_data.len() >= 2 {
                    let (year, month) = {
                        let parts: Vec<&str> = key.split('-').collect();
                        (parts[0].parse().unwrap_or(2024), parts[1].parse().unwrap_or(1))
                    };
                    let first = &month_data[0];
                    let last = &month_data[month_data.len() - 1];
                    let start_value = first.1;
                    let end_value = last.1;
                    let monthly_return = end_value - start_value;
                    let monthly_return_rate = if start_value > 0.0 {
                        (end_value - start_value) / start_value * 100.0
                    } else {
                        0.0
                    };
                    let benchmark_return = last.2;

                    result.push(MonthlyReturn {
                        year: year as i32,
                        month: month as i32,
                        start_value,
                        end_value,
                        monthly_return,
                        monthly_return_rate,
                        benchmark_return,
                    });
                }
            }
        }

        Ok(result)
    }

    /// 获取年度收益
    pub fn get_annual_returns(&self, portfolio_id: &str) -> Result<Vec<AnnualReturn>> {
        let db = self.db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = db.prepare(
            "SELECT date, total_value, benchmark_return FROM portfolio_returns
             WHERE portfolio_id = ?1 ORDER BY date ASC"
        )?;

        let returns: Vec<(String, f64, f64)> = stmt
            .query_map(params![portfolio_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        // 按年分组计算
        let mut yearly_map: std::collections::HashMap<i32, Vec<(String, f64, f64)>> = std::collections::HashMap::new();
        for (date, value, benchmark) in returns {
            if let Some(year) = date.get(0..4) {
                let year_int: i32 = year.parse().unwrap_or(2024);
                yearly_map.entry(year_int).or_default().push((date, value, benchmark));
            }
        }

        let mut result: Vec<AnnualReturn> = Vec::new();
        let mut sorted_years: Vec<_> = yearly_map.keys().cloned().collect();
        sorted_years.sort();

        for year in sorted_years {
            if let Some(year_data) = yearly_map.get(&year) {
                if year_data.len() >= 2 {
                    let start_value = year_data[0].1;
                    let end_value = year_data[year_data.len() - 1].1;
                    let annual_return = end_value - start_value;
                    let annual_return_rate = if start_value > 0.0 {
                        (end_value - start_value) / start_value * 100.0
                    } else {
                        0.0
                    };
                    let benchmark_return = year_data[year_data.len() - 1].2;

                    // 统计年度交易次数
                    let trades_count: i32 = db.query_row(
                        "SELECT COUNT(*) FROM portfolio_trades
                         WHERE portfolio_id = ?1 AND strftime('%Y', trade_date) = ?2",
                        params![portfolio_id, year.to_string()],
                        |row| row.get(0),
                    ).unwrap_or(0);

                    result.push(AnnualReturn {
                        year,
                        start_value,
                        end_value,
                        annual_return,
                        annual_return_rate,
                        benchmark_return,
                        trades_count,
                        win_count: trades_count / 2,
                    });
                }
            }
        }

        Ok(result)
    }
}

// 计算最大回撤
fn calculate_max_drawdown(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let mut peak = returns[0];
    let mut max_dd = 0.0;

    for &ret in returns {
        if ret > peak {
            peak = ret;
        }
        let drawdown = peak - ret;
        if drawdown > max_dd {
            max_dd = drawdown;
        }
    }

    -max_dd // 返回负值表示回撤
}

// 计算波动率（年化）
fn calculate_volatility(returns: &[f64]) -> f64 {
    if returns.len() < 2 {
        return 0.0;
    }

    let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance: f64 = returns.iter()
        .map(|r| {
            let diff = r - mean;
            diff * diff
        })
        .sum::<f64>() / returns.len() as f64;

    let std_dev = variance.sqrt();
    std_dev * (250.0_f64).sqrt() // 年化波动率
}

// 计算连胜连亏
fn calculate_streaks(returns: &[f64]) -> (i32, i32) {
    if returns.is_empty() {
        return (0, 0);
    }

    let mut win_streak = 0;
    let mut lose_streak = 0;
    let mut current_win = 0;
    let mut current_lose = 0;

    for &ret in returns {
        if ret > 0.0 {
            current_win += 1;
            current_lose = 0;
            win_streak = win_streak.max(current_win);
        } else if ret < 0.0 {
            current_lose += 1;
            current_win = 0;
            lose_streak = lose_streak.max(current_lose);
        }
    }

    (win_streak, lose_streak)
}
