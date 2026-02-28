use anyhow::Result;
use crate::models::*;
use chrono::Utc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// 回测引擎 - 基于事件驱动的简单回测框架
pub struct BacktestEngine;

impl BacktestEngine {
    pub fn new() -> Self {
        Self
    }

    /// 执行双均线策略回测
    pub fn run_ma_crossover(
        &self,
        candles: &[Candle],
        params: &BacktestParams,
        short_period: usize,
        long_period: usize,
    ) -> Result<BacktestResult> {
        if candles.len() < long_period {
            return Err(anyhow::anyhow!("K线数据不足，至少需要 {} 根", long_period));
        }

        let mut cash = params.initial_capital;
        let mut position: f64 = 0.0;
        let mut trades = Vec::new();
        let mut equity_curve = Vec::new();
        let initial_price = candles[0].close;

        // 计算移动平均线
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

        // 需要从 long_period+1 开始，确保 i-1 时也有足够数据计算前一期均线
        let start = (long_period + 1).max(short_period + 1);
        for i in start..candles.len() {
            let short_ma: f64 = closes[i.saturating_sub(short_period)..i].iter().sum::<f64>() / short_period as f64;
            let long_ma: f64 = closes[i.saturating_sub(long_period)..i].iter().sum::<f64>() / long_period as f64;
            let prev_short_ma: f64 = closes[i.saturating_sub(short_period + 1)..i.saturating_sub(1)].iter().sum::<f64>() / short_period as f64;
            let prev_long_ma: f64 = closes[i.saturating_sub(long_period + 1)..i.saturating_sub(1)].iter().sum::<f64>() / long_period as f64;

            let price = candles[i].close;
            let commission = price * params.commission_rate;

            // 金叉买入
            if prev_short_ma <= prev_long_ma && short_ma > long_ma && position == 0.0 {
                let qty = (cash * 0.95 / (price + commission)).floor();
                if qty > 0.0 {
                    let cost = qty * price + qty * commission;
                    cash -= cost;
                    position = qty;
                    trades.push(BacktestTrade {
                        timestamp: candles[i].timestamp.clone(),
                        symbol: params.symbol.clone(),
                        direction: "BUY".to_string(),
                        price,
                        quantity: qty,
                        commission: qty * commission,
                        pnl: 0.0,
                    });
                }
            }

            // 死叉卖出
            if prev_short_ma >= prev_long_ma && short_ma < long_ma && position > 0.0 {
                let revenue = position * price - position * commission;
                let buy_cost = trades.last().map(|t| t.price * t.quantity + t.commission).unwrap_or(0.0);
                let pnl = revenue - buy_cost;
                cash += revenue;

                trades.push(BacktestTrade {
                    timestamp: candles[i].timestamp.clone(),
                    symbol: params.symbol.clone(),
                    direction: "SELL".to_string(),
                    price,
                    quantity: position,
                    commission: position * commission,
                    pnl,
                });
                position = 0.0;
            }

            // 记录净值
            let equity = cash + position * price;
            let benchmark = params.initial_capital * (price / initial_price);
            equity_curve.push(EquityPoint {
                timestamp: candles[i].timestamp.clone(),
                equity,
                benchmark,
            });
        }

        // 如果还有持仓，按最后价格计算
        let final_price = candles.last().map(|c| c.close).unwrap_or(0.0);
        let final_equity = cash + position * final_price;

        // 计算KPI
        let kpis = self.calculate_kpis(&trades, &equity_curve, params.initial_capital, final_equity);

        Ok(BacktestResult {
            id: Uuid::new_v4().to_string(),
            strategy_id: params.strategy_id.clone(),
            params: params.clone(),
            kpis,
            trades,
            equity_curve,
            created_at: Utc::now(),
        })
    }

    /// 计算回测绩效指标
    fn calculate_kpis(
        &self,
        trades: &[BacktestTrade],
        equity_curve: &[EquityPoint],
        initial_capital: f64,
        final_equity: f64,
    ) -> BacktestKpis {
        let total_return = (final_equity - initial_capital) / initial_capital * 100.0;

        // 计算最大回撤
        let mut max_equity = initial_capital;
        let mut max_drawdown = 0.0_f64;
        for point in equity_curve {
            if point.equity > max_equity {
                max_equity = point.equity;
            }
            let drawdown = (max_equity - point.equity) / max_equity * 100.0;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        // 统计交易
        let sell_trades: Vec<&BacktestTrade> = trades.iter().filter(|t| t.direction == "SELL").collect();
        let winning_trades = sell_trades.iter().filter(|t| t.pnl > 0.0).count() as i32;
        let losing_trades = sell_trades.iter().filter(|t| t.pnl <= 0.0).count() as i32;
        let total_trades = sell_trades.len() as i32;
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64 * 100.0
        } else {
            0.0
        };

        // 盈亏比
        let avg_win = if winning_trades > 0 {
            sell_trades.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).sum::<f64>() / winning_trades as f64
        } else {
            0.0
        };
        let avg_loss = if losing_trades > 0 {
            sell_trades.iter().filter(|t| t.pnl <= 0.0).map(|t| t.pnl.abs()).sum::<f64>() / losing_trades as f64
        } else {
            1.0
        };
        let profit_loss_ratio = if avg_loss > 0.0 { avg_win / avg_loss } else { 0.0 };

        // 简化的年化收益和夏普比率
        let trading_days = equity_curve.len().max(1) as f64;
        let annual_return = total_return / trading_days * 252.0;

        // 计算日收益率标准差
        let daily_returns: Vec<f64> = equity_curve.windows(2)
            .map(|w| (w[1].equity - w[0].equity) / w[0].equity)
            .collect();
        let avg_daily_return = if !daily_returns.is_empty() {
            daily_returns.iter().sum::<f64>() / daily_returns.len() as f64
        } else {
            0.0
        };
        let std_dev = if daily_returns.len() > 1 {
            (daily_returns.iter().map(|r| (r - avg_daily_return).powi(2)).sum::<f64>()
                / (daily_returns.len() - 1) as f64).sqrt()
        } else {
            1.0
        };
        let sharpe_ratio = if std_dev > 0.0 {
            (avg_daily_return - 0.0001) / std_dev * (252.0_f64).sqrt()
        } else {
            0.0
        };

        // Sortino比率 (只考虑下行风险)
        let downside_returns: Vec<f64> = daily_returns.iter().filter(|&&r| r < 0.0).copied().collect();
        let downside_dev = if downside_returns.len() > 1 {
            (downside_returns.iter().map(|r| r.powi(2)).sum::<f64>()
                / (downside_returns.len() - 1) as f64).sqrt()
        } else {
            1.0
        };
        let sortino_ratio = if downside_dev > 0.0 {
            (avg_daily_return - 0.0001) / downside_dev * (252.0_f64).sqrt()
        } else {
            0.0
        };

        BacktestKpis {
            total_return,
            annual_return,
            max_drawdown,
            sharpe_ratio,
            sortino_ratio,
            win_rate,
            profit_loss_ratio,
            total_trades,
            winning_trades,
            losing_trades,
        }
    }
}

// ============ 参数优化 - 网格搜索 ============

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub params: serde_json::Value,
    pub total_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
}

pub fn optimize_ma_params(candles: &[Candle], params: &BacktestParams) -> Vec<OptimizationResult> {
    let mut results = Vec::new();
    
    // 网格搜索 short_period 和 long_period
    for short in 3..=20 {
        for long in (short + 5)..=60 {
            let engine = BacktestEngine::new();
            if let Ok(result) = engine.run_ma_crossover(candles, params, short, long) {
                results.push(OptimizationResult {
                    params: serde_json::json!({"short_period": short, "long_period": long}),
                    total_return: result.kpis.total_return,
                    sharpe_ratio: result.kpis.sharpe_ratio,
                    max_drawdown: result.kpis.max_drawdown,
                    win_rate: result.kpis.win_rate,
                });
            }
        }
    }
    
    // 按总收益排序
    results.sort_by(|a, b| b.total_return.partial_cmp(&a.total_return).unwrap());
    results
}
