use serde::{Deserialize, Serialize};
use crate::models::Candle;

/// 交易规则类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeRule {
    /// 不追高 - 涨幅超过阈值不买入
    NoChaseHigh { max_change_pct: f64 },
    /// 趋势交易 - MA5 > MA10 > MA20
    TrendFollowing { short_ma: u32, mid_ma: u32, long_ma: u32 },
    /// 精确进出点
    ExactEntry { min_risk_reward: f64 },
    /// 止损纪律
    StopLoss { max_loss_pct: f64 },
    /// 止盈纪律
    TakeProfit { min_profit_pct: f64 },
}

/// 交易规则检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCheckResult {
    pub passed: bool,
    pub rule_name: String,
    pub message: String,
    pub details: Vec<String>,
}

/// 交易信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSignal {
    pub action: TradeAction,
    pub symbol: String,
    pub price: f64,
    pub reason: String,
    pub rules_checked: Vec<RuleCheckResult>,
    pub risk_reward_ratio: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeAction {
    Buy,
    Sell,
    Hold,
    Wait,
}

/// 交易纪律服务
pub struct TradingDisciplineService {
    rules: Vec<TradeRule>,
}

impl TradingDisciplineService {
    pub fn new() -> Self {
        Self {
            rules: vec![
                TradeRule::NoChaseHigh { max_change_pct: 5.0 },
                TradeRule::TrendFollowing { short_ma: 5, mid_ma: 10, long_ma: 20 },
                TradeRule::ExactEntry { min_risk_reward: 2.0 },
                TradeRule::StopLoss { max_loss_pct: -7.0 },
                TradeRule::TakeProfit { min_profit_pct: 15.0 },
            ],
        }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: TradeRule) {
        self.rules.push(rule);
    }

    /// 移除规则
    pub fn remove_rule(&mut self, index: usize) {
        if index < self.rules.len() {
            self.rules.remove(index);
        }
    }

    /// 检查是否应该买入
    pub fn check_buy(&self, symbol: &str, price: f64, change_pct: f64, candles: &[Candle]) -> TradeSignal {
        let mut results = Vec::new();
        let mut all_passed = true;

        for rule in &self.rules {
            let result = match rule {
                TradeRule::NoChaseHigh { max_change_pct } => {
                    let passed = change_pct <= *max_change_pct;
                    if !passed { all_passed = false; }
                    RuleCheckResult {
                        passed,
                        rule_name: "不追高".to_string(),
                        message: if passed { "涨幅在可追范围内".to_string() } else { format!("涨幅{:.2}%超过阈值{:.2}%", change_pct, max_change_pct) },
                        details: vec![format!("当前涨幅: {:.2}%", change_pct)],
                    }
                }
                TradeRule::TrendFollowing { short_ma, mid_ma, long_ma } => {
                    let ma_check = self.check_ma_trend(candles, *short_ma, *mid_ma, *long_ma);
                    if !ma_check.0 { all_passed = false; }
                    RuleCheckResult {
                        passed: ma_check.0,
                        rule_name: "趋势交易".to_string(),
                        message: ma_check.1,
                        details: ma_check.2,
                    }
                }
                TradeRule::ExactEntry { min_risk_reward } => {
                    // 需要结合止损止盈计算
                    RuleCheckResult {
                        passed: true,
                        rule_name: "精确进出点".to_string(),
                        message: "需要结合具体止损止盈位计算".to_string(),
                        details: vec![],
                    }
                }
                _ => {
                    RuleCheckResult {
                        passed: true,
                        rule_name: "其他规则".to_string(),
                        message: "买入时不检查".to_string(),
                        details: vec![],
                    }
                }
            };
            results.push(result);
        }

        let action = if all_passed { TradeAction::Buy } else { TradeAction::Wait };

        TradeSignal {
            action,
            symbol: symbol.to_string(),
            price,
            reason: if all_passed { "所有规则检查通过".to_string() } else { "部分规则未通过".to_string() },
            rules_checked: results,
            risk_reward_ratio: 0.0,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 检查是否应该卖出
    pub fn check_sell(&self, symbol: &str, price: f64, entry_price: f64, change_pct: f64, candles: &[Candle]) -> TradeSignal {
        let mut results = Vec::new();
        let mut all_passed = true;
        let profit_pct = (price - entry_price) / entry_price * 100.0;

        for rule in &self.rules {
            let result = match rule {
                TradeRule::StopLoss { max_loss_pct } => {
                    let passed = profit_pct >= *max_loss_pct;
                    if !passed { all_passed = false; }
                    RuleCheckResult {
                        passed,
                        rule_name: "止损纪律".to_string(),
                        message: if passed { "触发止损".to_string() } else { "未触发止损".to_string() },
                        details: vec![format!("当前盈亏: {:.2}%", profit_pct)],
                    }
                }
                TradeRule::TakeProfit { min_profit_pct } => {
                    let passed = profit_pct >= *min_profit_pct;
                    RuleCheckResult {
                        passed,
                        rule_name: "止盈纪律".to_string(),
                        message: if passed { "达到止盈目标".to_string() } else { "未达到止盈".to_string() },
                        details: vec![format!("当前盈利: {:.2}%", profit_pct)],
                    }
                }
                TradeRule::TrendFollowing { short_ma, mid_ma, long_ma } => {
                    let ma_check = self.check_ma_trend(candles, *short_ma, *mid_ma, *long_ma);
                    if ma_check.0 {
                        // 趋势还在，继续持有
                        RuleCheckResult {
                            passed: true,
                            rule_name: "趋势交易".to_string(),
                            message: "均线多头，继续持有".to_string(),
                            details: ma_check.2,
                        }
                    } else {
                        all_passed = false;
                        RuleCheckResult {
                            passed: false,
                            rule_name: "趋势交易".to_string(),
                            message: "均线死叉，考虑卖出".to_string(),
                            details: ma_check.2,
                        }
                    }
                }
                _ => {
                    RuleCheckResult {
                        passed: true,
                        rule_name: "其他规则".to_string(),
                        message: "卖出时不检查".to_string(),
                        details: vec![],
                    }
                }
            };
            results.push(result);
        }

        let action = if all_passed { TradeAction::Sell } else { TradeAction::Hold };

        TradeSignal {
            action,
            symbol: symbol.to_string(),
            price,
            reason: if all_passed { "触发卖出规则".to_string() } else { "继续持有".to_string() },
            rules_checked: results,
            risk_reward_ratio: 0.0,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 检查均线趋势
    fn check_ma_trend(&self, candles: &[Candle], short_ma: u32, mid_ma: u32, long_ma: u32) -> (bool, String, Vec<String>) {
        if candles.len() < long_ma as usize {
            return (true, "数据不足，无法判断趋势".to_string(), vec![]);
        }

        let calc_ma = |n: u32| -> f64 {
            let n = n as usize;
            if candles.len() < n { return 0.0; }
            candles.iter().rev().take(n).map(|c| c.close).sum::<f64>() / n as f64
        };

        let ma5 = calc_ma(short_ma);
        let ma10 = calc_ma(mid_ma);
        let ma20 = calc_ma(long_ma);

        let is_bullish = ma5 > ma10 && ma10 > ma20;
        let message = if is_bullish {
            format!("MA{}>MA{}>MA{} 多头排列", short_ma, mid_ma, long_ma)
        } else {
            format!("MA{}>MA{}>MA20 空头排列", short_ma, mid_ma)
        };

        (is_bullish, message, vec![
            format!("MA{}: {:.2}", short_ma, ma5),
            format!("MA{}: {:.2}", mid_ma, ma10),
            format!("MA{}: {:.2}", long_ma, ma20),
        ])
    }

    /// 获取所有规则
    pub fn get_rules(&self) -> Vec<TradeRule> {
        self.rules.clone()
    }

    /// 生成检查清单
    pub fn generate_checklist(&self, action: &str) -> Vec<String> {
        let mut checklist = vec![];

        for rule in &self.rules {
            match rule {
                TradeRule::NoChaseHigh { max_change_pct } => {
                    checklist.push(format!("[ ] 涨幅不超过{:.1}% (不追高)", max_change_pct));
                }
                TradeRule::TrendFollowing { short_ma, mid_ma, long_ma } => {
                    checklist.push(format!("[ ] MA{} > MA{} > MA{} (趋势交易)", short_ma, mid_ma, long_ma));
                }
                TradeRule::ExactEntry { min_risk_reward } => {
                    checklist.push(format!("[ ] 盈亏比 >= {:.1} (精确进出)", min_risk_reward));
                }
                TradeRule::StopLoss { max_loss_pct } => {
                    checklist.push(format!("[ ] 止损线 {:.1}%", max_loss_pct));
                }
                TradeRule::TakeProfit { min_profit_pct } => {
                    checklist.push(format!("[ ] 止盈线 +{:.1}%", min_profit_pct));
                }
            }
        }

        checklist
    }
}

impl Default for TradingDisciplineService {
    fn default() -> Self {
        Self::new()
    }
}
