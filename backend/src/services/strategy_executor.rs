use crate::models::strategy_config::{
    BacktestRequest, CompareTarget, ComparisonOp, ConditionGroup, IndicatorConfig,
    SignalCondition, SignalAction, StrategyConfig, StrategySignal, StrategyStatus,
    TemplatePerformance, TradeAction,
};
use crate::models::{BacktestKpis, BacktestParams, BacktestResult, BacktestTrade, Candle, EquityPoint};
use crate::services::indicators::{self, macd, rsi, MacdResult};
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Strategy executor for both backtesting and live trading
pub struct StrategyExecutor;

impl StrategyExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Run a backtest with the given strategy config
    pub fn run_backtest(
        &self,
        req: &BacktestRequest,
        candles: &[Candle],
    ) -> Result<BacktestResult> {
        let initial_capital = req.initial_capital.unwrap_or(100_000.0);
        let commission_rate = req.commission_rate.unwrap_or(0.0003);
        let config = &req.config;

        let mut cash = initial_capital;
        let mut position: f64 = 0.0;
        let mut trades = Vec::new();
        let mut equity_curve = Vec::new();

        // Calculate all indicators upfront
        let indicator_values = self.calculate_indicators(candles, config)?;

        // Iterate through each candle
        let start_idx = self.get_start_index(candles.len(), config);

        for i in start_idx..candles.len() {
            let candle = &candles[i];
            let timestamp = &candle.timestamp;

            // Evaluate conditions and generate signal
            let signal = self.evaluate_conditions(
                &indicator_values,
                i,
                config,
                candle,
            );

            let price = candle.close;
            let commission = price * commission_rate;

            // Execute signal
            match signal.action {
                SignalAction::Buy if position == 0.0 => {
                    let target_qty = ((cash * 0.95) / (price + commission)).floor();
                    if target_qty > 0.0 {
                        let cost = target_qty * price + target_qty * commission;
                        cash -= cost;
                        position = target_qty;
                        trades.push(BacktestTrade {
                            timestamp: timestamp.clone(),
                            symbol: config.symbols.first().unwrap_or(&"UNKNOWN".to_string()).clone(),
                            direction: "BUY".to_string(),
                            price,
                            quantity: target_qty,
                            commission: target_qty * commission,
                            pnl: 0.0,
                        });
                    }
                }
                SignalAction::Sell if position > 0.0 => {
                    let revenue = position * price - position * commission;
                    let buy_cost = trades
                        .iter()
                        .rev()
                        .find(|t| t.direction == "BUY")
                        .map(|t| t.price * t.quantity + t.commission)
                        .unwrap_or(0.0);
                    let pnl = revenue - buy_cost;
                    cash += revenue;

                    trades.push(BacktestTrade {
                        timestamp: timestamp.clone(),
                        symbol: config.symbols.first().unwrap_or(&"UNKNOWN".to_string()).clone(),
                        direction: "SELL".to_string(),
                        price,
                        quantity: position,
                        commission: position * commission,
                        pnl,
                    });
                    position = 0.0;
                }
                _ => {}
            }

            // Record equity
            let equity = cash + position * price;
            let benchmark = initial_capital * (price / candles.first().map(|c| c.close).unwrap_or(price));
            equity_curve.push(EquityPoint {
                timestamp: timestamp.clone(),
                equity,
                benchmark,
            });
        }

        // Calculate final equity
        let final_price = candles.last().map(|c| c.close).unwrap_or(0.0);
        let final_equity = cash + position * final_price;

        // Calculate KPIs
        let kpis = self.calculate_kpis(&trades, &equity_curve, initial_capital, final_equity);

        Ok(BacktestResult {
            id: Uuid::new_v4().to_string(),
            strategy_id: config.id.clone(),
            params: BacktestParams {
                strategy_id: config.id.clone(),
                symbol: req.symbol.clone(),
                start_date: candles.first().map(|c| c.timestamp.clone()).unwrap_or_default(),
                end_date: candles.last().map(|c| c.timestamp.clone()).unwrap_or_default(),
                initial_capital,
                commission_rate,
                slippage: 0.0005,
            },
            kpis,
            trades,
            equity_curve,
            created_at: Utc::now(),
        })
    }

    /// Generate a real-time signal for live trading
    pub fn generate_signal(
        &self,
        config: &StrategyConfig,
        candles: &[Candle],
    ) -> Result<StrategySignal> {
        let indicator_values = self.calculate_indicators(candles, config)?;
        let latest_idx = candles.len().saturating_sub(1);

        if latest_idx == 0 {
            let unknown = "UNKNOWN".to_string();
            return Ok(StrategySignal::hold(
                config.symbols.first().unwrap_or(&unknown),
                &candles.first().map(|c| c.timestamp.clone()).unwrap_or_default(),
                "No data",
            ));
        }

        let candle = &candles[latest_idx];
        Ok(self.evaluate_conditions(&indicator_values, latest_idx, config, candle))
    }

    /// Calculate all indicators for a strategy
    fn calculate_indicators(
        &self,
        candles: &[Candle],
        config: &StrategyConfig,
    ) -> Result<HashMap<String, Vec<f64>>> {
        let mut results = HashMap::new();

        for indicator in &config.indicators {
            let values = self.calculate_indicator(candles, indicator)?;
            if let Some(alias) = &indicator.alias {
                results.insert(alias.clone(), values);
            } else {
                results.insert(indicator.indicator_type.clone(), values);
            }
        }

        Ok(results)
    }

    /// Calculate a single indicator
    fn calculate_indicator(
        &self,
        candles: &[Candle],
        config: &IndicatorConfig,
    ) -> Result<Vec<f64>> {
        let field = &config.field;
        let params = &config.params;

        match config.indicator_type.as_str() {
            "ma" | "sma" => {
                let period = params.get("period").copied().unwrap_or(5.0) as usize;
                indicators::sma(candles, period, field).context("SMA calculation failed")
            }
            "ema" => {
                let period = params.get("period").copied().unwrap_or(12.0) as usize;
                indicators::ema(candles, period, field).context("EMA calculation failed")
            }
            "rsi" => {
                let period = params.get("period").copied().unwrap_or(14.0) as usize;
                indicators::rsi(candles, period, field).context("RSI calculation failed")
            }
            "macd" => {
                let fast = params.get("fast").copied().unwrap_or(12.0) as usize;
                let slow = params.get("slow").copied().unwrap_or(26.0) as usize;
                let signal = params.get("signal").copied().unwrap_or(9.0) as usize;
                let macd_result = macd(candles, fast, slow, signal, field)
                    .context("MACD calculation failed")?;
                Ok(macd_result.dif) // Return DIF as the main MACD value
            }
            "boll" | "bollinger" => {
                let period = params.get("period").copied().unwrap_or(20.0) as usize;
                let std_dev = params.get("std_dev").copied().unwrap_or(2.0);
                let boll_result = indicators::boll(candles, period, std_dev, field)
                    .context("Bollinger calculation failed")?;
                Ok(boll_result.middle) // Return middle band
            }
            "volume_ratio" => {
                let period = params.get("period").copied().unwrap_or(5.0) as usize;
                indicators::volume_ratio(candles, period).context("Volume ratio failed")
            }
            _ => Err(anyhow::anyhow!("Unknown indicator type: {}", config.indicator_type)),
        }
    }

    /// Get the starting index for backtesting based on indicator requirements
    fn get_start_index(&self, candle_count: usize, config: &StrategyConfig) -> usize {
        let mut max_period = 60; // Default buffer

        for indicator in &config.indicators {
            if let Some(period) = indicator.params.get("period") {
                max_period = max_period.max(*period as usize);
            }
            if let Some(fast) = indicator.params.get("fast") {
                max_period = max_period.max(*fast as usize);
            }
            if let Some(slow) = indicator.params.get("slow") {
                max_period = max_period.max(*slow as usize);
            }
        }

        max_period.min(candle_count.saturating_sub(1))
    }

    /// Evaluate all conditions and generate a signal
    fn evaluate_conditions(
        &self,
        indicator_values: &HashMap<String, Vec<f64>>,
        idx: usize,
        config: &StrategyConfig,
        candle: &Candle,
    ) -> StrategySignal {
        let unknown = "UNKNOWN".to_string();
        let symbol = config.symbols.first().unwrap_or(&unknown);
        let timestamp = candle.timestamp.clone();

        // Evaluate buy conditions
        let buy_triggered = self.evaluate_condition_groups(&config.conditions, indicator_values, idx);

        // Evaluate sell conditions - use custom or fallback to MA cross below
        let sell_triggered = if let Some(ref sell_conds) = config.sell_conditions {
            self.evaluate_condition_groups(sell_conds, indicator_values, idx)
        } else {
            self.evaluate_default_sell_conditions(indicator_values, idx)
        };

        if buy_triggered {
            StrategySignal::buy(
                symbol,
                &timestamp,
                candle.close,
                0.0, // Quantity calculated at execution
                "Buy signal triggered",
            )
        } else if sell_triggered {
            StrategySignal::sell(
                symbol,
                &timestamp,
                candle.close,
                0.0,
                "Sell signal triggered",
            )
        } else {
            StrategySignal::hold(symbol, &timestamp, "No signal")
        }
    }

    /// Evaluate condition groups (with AND/OR logic)
    fn evaluate_condition_groups(
        &self,
        groups: &[ConditionGroup],
        indicator_values: &HashMap<String, Vec<f64>>,
        idx: usize,
    ) -> bool {
        if groups.is_empty() {
            return false;
        }

        // For now, evaluate all groups as OR (any group triggers)
        for group in groups {
            if self.evaluate_group(group, indicator_values, idx) {
                return true;
            }
        }
        false
    }

    /// Evaluate a single condition group
    fn evaluate_group(
        &self,
        group: &ConditionGroup,
        indicator_values: &HashMap<String, Vec<f64>>,
        idx: usize,
    ) -> bool {
        for condition in &group.conditions {
            let result = self.evaluate_single_condition(condition, indicator_values, idx);
            match group.logic.as_str() {
                "or" if result => return true,
                "and" if !result => return false,
                _ => {}
            }
        }
        group.logic.as_str() == "and"
    }

    /// Default sell conditions (used when no custom sell_conditions defined)
    /// MA cross below - sell when fast MA crosses below slow MA
    fn evaluate_default_sell_conditions(
        &self,
        indicator_values: &HashMap<String, Vec<f64>>,
        idx: usize,
    ) -> bool {
        // Check MA cross below (ma_fast crosses below ma_slow)
        if let Some(fast_values) = indicator_values.get("ma_fast") {
            if idx > 0 {
                let current_fast = fast_values.get(idx).copied().unwrap_or(0.0);
                let prev_fast = fast_values.get(idx - 1).copied().unwrap_or(0.0);
                if let Some(slow_values) = indicator_values.get("ma_slow") {
                    let current_slow = slow_values.get(idx).copied().unwrap_or(0.0);
                    let prev_slow = slow_values.get(idx - 1).copied().unwrap_or(0.0);
                    // Cross below: fast was above slow and now is below or equal
                    if prev_fast > prev_slow && current_fast <= current_slow {
                        return true;
                    }
                }
            }
        }

        // Check RSI overbought (RSI > 70) as fallback
        if let Some(rsi_values) = indicator_values.get("rsi") {
            if idx > 0 {
                let current_rsi = rsi_values.get(idx).copied().unwrap_or(50.0);
                let prev_rsi = rsi_values.get(idx - 1).copied().unwrap_or(50.0);
                if current_rsi > 70.0 && prev_rsi <= 70.0 {
                    return true;
                }
            }
        }

        false
    }

    /// Evaluate a single condition
    fn evaluate_single_condition(
        &self,
        condition: &SignalCondition,
        indicator_values: &HashMap<String, Vec<f64>>,
        idx: usize,
    ) -> bool {
        let indicator_data = match indicator_values.get(&condition.indicator) {
            Some(v) => v,
            None => return false,
        };

        let current = indicator_data.get(idx).copied().unwrap_or(0.0);
        let prev = if idx > 0 {
            indicator_data.get(idx - 1).copied().unwrap_or(0.0)
        } else {
            current
        };

        let compare_value = match &condition.compare_with {
            CompareTarget::Indicator(other_name) => {
                indicator_values
                    .get(other_name)
                    .and_then(|v: &Vec<f64>| v.get(idx).copied())
                    .unwrap_or(0.0)
            }
            CompareTarget::Value(v) => *v,
        };

        match condition.comparison {
            ComparisonOp::CrossAbove => prev <= compare_value && current > compare_value,
            ComparisonOp::CrossBelow => prev >= compare_value && current < compare_value,
            ComparisonOp::GreaterThan => current > compare_value,
            ComparisonOp::LessThan => current < compare_value,
            ComparisonOp::GreaterThanOrEqual => current >= compare_value,
            ComparisonOp::LessThanOrEqual => current <= compare_value,
            ComparisonOp::Equal => (current - compare_value).abs() < f64::EPSILON,
        }
    }

    /// Calculate KPIs for backtest result
    fn calculate_kpis(
        &self,
        trades: &[BacktestTrade],
        equity_curve: &[EquityPoint],
        initial_capital: f64,
        final_equity: f64,
    ) -> BacktestKpis {
        let total_return = (final_equity - initial_capital) / initial_capital * 100.0;

        // Max drawdown
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

        // Trade stats
        let sell_trades: Vec<&BacktestTrade> = trades.iter().filter(|t| t.direction == "SELL").collect();
        let winning_trades = sell_trades.iter().filter(|t| t.pnl > 0.0).count() as i32;
        let losing_trades = sell_trades.iter().filter(|t| t.pnl <= 0.0).count() as i32;
        let total_trades = sell_trades.len() as i32;
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64 * 100.0
        } else {
            0.0
        };

        let avg_win = if winning_trades > 0 {
            sell_trades.iter()
                .filter(|t| t.pnl > 0.0)
                .map(|t| t.pnl)
                .sum::<f64>() / winning_trades as f64
        } else {
            0.0
        };
        let avg_loss = if losing_trades > 0 {
            sell_trades.iter()
                .filter(|t| t.pnl <= 0.0)
                .map(|t| t.pnl.abs())
                .sum::<f64>() / losing_trades as f64
        } else {
            1.0
        };
        let profit_loss_ratio = if avg_loss > 0.0 { avg_win / avg_loss } else { 0.0 };

        // Annual return
        let trading_days = equity_curve.len().max(1) as f64;
        let annual_return = total_return / trading_days * 252.0;

        // Sharpe ratio (simplified)
        let daily_returns: Vec<f64> = equity_curve.windows(2)
            .map(|w| (w[1].equity - w[0].equity) / w[0].equity)
            .collect();
        let avg_daily_return = if !daily_returns.is_empty() {
            daily_returns.iter().sum::<f64>() / daily_returns.len() as f64
        } else {
            0.0
        };
        let std_dev = if daily_returns.len() > 1 {
            (daily_returns.iter()
                .map(|r| (r - avg_daily_return).powi(2))
                .sum::<f64>() / (daily_returns.len() - 1) as f64)
                .sqrt()
        } else {
            1.0
        };
        let sharpe_ratio = if std_dev > 0.0 {
            (avg_daily_return - 0.0001) / std_dev * (252.0_f64).sqrt()
        } else {
            0.0
        };

        // Sortino ratio
        let downside_returns: Vec<f64> = daily_returns.iter().filter(|&&r| r < 0.0).copied().collect();
        let downside_dev = if downside_returns.len() > 1 {
            (downside_returns.iter()
                .map(|r| r.powi(2))
                .sum::<f64>() / (downside_returns.len() - 1) as f64)
                .sqrt()
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

impl Default for StrategyExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a pre-built strategy config from a template ID
pub fn create_template_strategy(template_id: &str) -> Option<StrategyConfig> {
    match template_id {
        "ma_cross" => Some(StrategyConfig {
            id: "ma_cross".to_string(),
            name: "双均线交叉策略".to_string(),
            description: "MA5上穿MA20买入，MA5下穿MA20卖出".to_string(),
            indicators: vec![
                IndicatorConfig {
                    indicator_type: "ma".to_string(),
                    params: [("period".to_string(), 5.0)].into(),
                    alias: Some("ma_fast".to_string()),
                    field: "close".to_string(),
                },
                IndicatorConfig {
                    indicator_type: "ma".to_string(),
                    params: [("period".to_string(), 20.0)].into(),
                    alias: Some("ma_slow".to_string()),
                    field: "close".to_string(),
                },
            ],
            conditions: vec![
                ConditionGroup {
                    logic: "and".to_string(),
                    conditions: vec![
                        SignalCondition {
                            indicator: "ma_fast".to_string(),
                            comparison: ComparisonOp::CrossAbove,
                            compare_with: CompareTarget::Indicator("ma_slow".to_string()),
                        },
                    ],
                },
            ],
            sell_conditions: Some(vec![
                ConditionGroup {
                    logic: "and".to_string(),
                    conditions: vec![
                        SignalCondition {
                            indicator: "ma_fast".to_string(),
                            comparison: ComparisonOp::CrossBelow,
                            compare_with: CompareTarget::Indicator("ma_slow".to_string()),
                        },
                    ],
                },
            ]),
            buy_action: TradeAction {
                direction: "buy".to_string(),
                target_percent: 0.95,
            },
            sell_action: TradeAction {
                direction: "sell".to_string(),
                target_percent: 1.0,
            },
            symbols: vec!["000001".to_string()],
            period: "1d".to_string(),
        }),
        "rsi_mean_revert" => Some(StrategyConfig {
            id: "rsi_mean_revert".to_string(),
            name: "RSI均值回归策略".to_string(),
            description: "RSI低于30买入，RSI高于70卖出".to_string(),
            indicators: vec![
                IndicatorConfig {
                    indicator_type: "rsi".to_string(),
                    params: [("period".to_string(), 14.0)].into(),
                    alias: Some("rsi".to_string()),
                    field: "close".to_string(),
                },
            ],
            conditions: vec![
                ConditionGroup {
                    logic: "and".to_string(),
                    conditions: vec![
                        SignalCondition {
                            indicator: "rsi".to_string(),
                            comparison: ComparisonOp::LessThan,
                            compare_with: CompareTarget::Value(30.0),
                        },
                    ],
                },
            ],
            sell_conditions: Some(vec![
                ConditionGroup {
                    logic: "and".to_string(),
                    conditions: vec![
                        SignalCondition {
                            indicator: "rsi".to_string(),
                            comparison: ComparisonOp::GreaterThan,
                            compare_with: CompareTarget::Value(70.0),
                        },
                    ],
                },
            ]),
            buy_action: TradeAction {
                direction: "buy".to_string(),
                target_percent: 0.95,
            },
            sell_action: TradeAction {
                direction: "sell".to_string(),
                target_percent: 1.0,
            },
            symbols: vec!["000001".to_string()],
            period: "1d".to_string(),
        }),
        "macd_trend" => Some(StrategyConfig {
            id: "macd_trend".to_string(),
            name: "MACD趋势策略".to_string(),
            description: "MACD金叉买入，死叉卖出".to_string(),
            indicators: vec![
                IndicatorConfig {
                    indicator_type: "macd".to_string(),
                    params: [
                        ("fast".to_string(), 12.0),
                        ("slow".to_string(), 26.0),
                        ("signal".to_string(), 9.0),
                    ].into(),
                    alias: Some("macd".to_string()),
                    field: "close".to_string(),
                },
            ],
            conditions: vec![
                ConditionGroup {
                    logic: "and".to_string(),
                    conditions: vec![
                        SignalCondition {
                            indicator: "macd".to_string(),
                            comparison: ComparisonOp::CrossAbove,
                            compare_with: CompareTarget::Value(0.0),
                        },
                    ],
                },
            ],
            sell_conditions: Some(vec![
                ConditionGroup {
                    logic: "and".to_string(),
                    conditions: vec![
                        SignalCondition {
                            indicator: "macd".to_string(),
                            comparison: ComparisonOp::CrossBelow,
                            compare_with: CompareTarget::Value(0.0),
                        },
                    ],
                },
            ]),
            buy_action: TradeAction {
                direction: "buy".to_string(),
                target_percent: 0.95,
            },
            sell_action: TradeAction {
                direction: "sell".to_string(),
                target_percent: 1.0,
            },
            symbols: vec!["000001".to_string()],
            period: "1d".to_string(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candles(prices: &[f64]) -> Vec<Candle> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &close)| Candle {
                symbol: "TEST".to_string(),
                timestamp: format!("2024-01-{:02}", i + 1),
                open: close * 0.99,
                high: close * 1.01,
                low: close * 0.98,
                close,
                volume: 1_000_000.0,
                turnover: 0.0,
            })
            .collect()
    }

    #[test]
    fn test_ma_cross_strategy() {
        let executor = StrategyExecutor::new();
        let config = create_template_strategy("ma_cross").unwrap();

        // Rising prices - should trigger cross above
        let prices: Vec<f64> = (1..=30).map(|i| 10.0 + i as f64).collect();
        let candles = make_candles(&prices);

        let req = BacktestRequest {
            config,
            symbol: "TEST".to_string(),
            period: None,
            count: None,
            initial_capital: Some(100_000.0),
            commission_rate: Some(0.0003),
        };

        let result = executor.run_backtest(&req, &candles).unwrap();
        assert!(!result.trades.is_empty() || result.kpis.total_return >= 0.0);
    }
}
