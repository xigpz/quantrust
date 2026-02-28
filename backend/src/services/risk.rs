use serde::{Deserialize, Serialize};
use crate::models::BacktestResult;

/// 风险控制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// 最大仓位比例 (0.0-1.0)
    pub max_position_ratio: f64,
    /// 单只股票最大仓位
    pub max_single_position: f64,
    /// 止损比例 (如 0.05 = 5%)
    pub stop_loss_ratio: f64,
    /// 止盈比例
    pub take_profit_ratio: f64,
    /// 最大回撤阈值 (如 0.15 = 15%)
    pub max_drawdown_threshold: f64,
    /// 是否启用风控
    pub enabled: bool,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_position_ratio: 0.8,      // 最多80%仓位
            max_single_position: 0.2,     // 单股最多20%
            stop_loss_ratio: 0.05,        // 5%止损
            take_profit_ratio: 0.15,      // 15%止盈
            max_drawdown_threshold: 0.15,  // 15%最大回撤
            enabled: true,
        }
    }
}

/// 风险管理器
pub struct RiskManager {
    config: RiskConfig,
}

impl RiskManager {
    pub fn new(config: RiskConfig) -> Self {
        Self { config }
    }

    pub fn with_default() -> Self {
        Self::new(RiskConfig::default())
    }

    /// 检查是否可以买入
    pub fn can_buy(&self, current_position: f64, new_position: f64, total_capital: f64) -> (bool, String) {
        if !self.config.enabled {
            return (true, "风控已关闭".to_string());
        }

        // 检查总仓位
        let total_ratio = (current_position + new_position) / total_capital;
        if total_ratio > self.config.max_position_ratio {
            return (false, format!(
                "总仓位超限: {:.1}% > {:.1}%", 
                total_ratio * 100.0, 
                self.config.max_position_ratio * 100.0
            ));
        }

        // 检查单股仓位
        let single_ratio = new_position / total_capital;
        if single_ratio > self.config.max_single_position {
            return (false, format!(
                "单股仓位超限: {:.1}% > {:.1}%", 
                single_ratio * 100.0, 
                self.config.max_single_position * 100.0
            ));
        }

        (true, "可以买入".to_string())
    }

    /// 检查是否触发止损
    pub fn check_stop_loss(&self, entry_price: f64, current_price: f64) -> (bool, String) {
        if !self.config.enabled {
            return (false, "风控已关闭".to_string());
        }

        let loss_ratio = (entry_price - current_price) / entry_price;
        if loss_ratio >= self.config.stop_loss_ratio {
            return (true, format!(
                "触发止损: 亏损 {:.1}% >= {:.1}%", 
                loss_ratio * 100.0, 
                self.config.stop_loss_ratio * 100.0
            ));
        }

        (false, "未触发止损".to_string())
    }

    /// 检查是否触发止盈
    pub fn check_take_profit(&self, entry_price: f64, current_price: f64) -> (bool, String) {
        if !self.config.enabled {
            return (false, "风控已关闭".to_string());
        }

        let profit_ratio = (current_price - entry_price) / entry_price;
        if profit_ratio >= self.config.take_profit_ratio {
            return (true, format!(
                "触发止盈: 盈利 {:.1}% >= {:.1}%", 
                profit_ratio * 100.0, 
                self.config.take_profit_ratio * 100.0
            ));
        }

        (false, "未触发止盈".to_string())
    }

    /// 计算当前回撤
    pub fn calculate_drawdown(&self, equity_curve: &[f64]) -> (f64, f64, usize) {
        if equity_curve.is_empty() {
            return (0.0, 0.0, 0);
        }

        let mut peak = equity_curve[0];
        let mut max_drawdown = 0.0;
        let mut max_drawdown_idx = 0;

        for (i, &value) in equity_curve.iter().enumerate() {
            if value > peak {
                peak = value;
            }
            let drawdown = (peak - value) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
                max_drawdown_idx = i;
            }
        }

        (max_drawdown, peak, max_drawdown_idx)
    }

    /// 检查是否触发最大回撤告警
    pub fn check_max_drawdown(&self, equity_curve: &[f64]) -> (bool, String, f64) {
        if !self.config.enabled || equity_curve.is_empty() {
            return (false, "未触发".to_string(), 0.0);
        }

        let (drawdown, _, _) = self.calculate_drawdown(equity_curve);
        
        if drawdown >= self.config.max_drawdown_threshold {
            return (true, format!(
                "触发最大回撤告警: {:.1}% >= {:.1}%", 
                drawdown * 100.0, 
                self.config.max_drawdown_threshold * 100.0
            ), drawdown);
        }

        (false, "未触发".to_string(), drawdown)
    }

    /// 综合信号检查（用于交易前）
    pub fn check_trade_signal(
        &self,
        position: f64,
        entry_price: f64,
        current_price: f64,
        total_capital: f64,
        trade_type: TradeType,
    ) -> TradeSignal {
        match trade_type {
            TradeType::Buy => {
                let (allowed, reason) = self.can_buy(position, position * 0.1, total_capital);
                TradeSignal {
                    allowed,
                    action: TradeAction::Buy,
                    reason,
                    stop_loss_price: Some(entry_price * (1.0 - self.config.stop_loss_ratio)),
                    take_profit_price: Some(entry_price * (1.0 + self.config.take_profit_ratio)),
                }
            }
            TradeType::Sell => {
                let (sl_triggered, sl_reason) = self.check_stop_loss(entry_price, current_price);
                let (tp_triggered, tp_reason) = self.check_take_profit(entry_price, current_price);
                
                let allowed = sl_triggered || tp_triggered;
                let reason = if sl_triggered { sl_reason } else { tp_reason };
                
                TradeSignal {
                    allowed,
                    action: if allowed { TradeAction::Sell } else { TradeAction::Hold },
                    reason,
                    stop_loss_price: None,
                    take_profit_price: None,
                }
            }
        }
    }

    /// 从回测结果生成风控报告
    pub fn generate_risk_report(&self, result: &BacktestResult) -> RiskReport {
        let equity: Vec<f64> = result.equity_curve.iter().map(|e| e.equity).collect();
        
        let (max_drawdown, peak, idx) = self.calculate_drawdown(&equity);
        let (drawdown_alert, drawdown_msg, _) = self.check_max_drawdown(&equity);
        
        let final_equity = equity.last().copied().unwrap_or(0.0);
        let total_return = if result.params.initial_capital > 0.0 {
            (final_equity - result.params.initial_capital) / result.params.initial_capital
        } else {
            0.0
        };

        RiskReport {
            config: self.config.clone(),
            total_return,
            max_drawdown,
            drawdown_peak_idx: idx,
            drawdown_alert_triggered: drawdown_alert,
            drawdown_alert_message: drawdown_msg,
            final_equity,
            risk_score: self.calculate_risk_score(max_drawdown, total_return, result.kpis.win_rate),
        }
    }

    /// 计算风险评分 (0-100, 越高风险越大)
    fn calculate_risk_score(&self, max_drawdown: f64, total_return: f64, win_rate: f64) -> f64 {
        let mut score = 0.0;
        
        // 回撤贡献
        score += max_drawdown * 100.0 * 0.5;
        
        // 收益贡献 (负收益增加风险)
        if total_return < 0.0 {
            score += (-total_return) * 100.0 * 0.3;
        }
        
        // 胜率贡献 (低胜率增加风险)
        if win_rate < 0.5 {
            score += (0.5 - win_rate) * 100.0 * 0.2;
        }
        
        score.min(100.0)
    }
}

impl Default for RiskManager {
    fn default() -> Self {
        Self::with_default()
    }
}

/// 交易类型
#[derive(Debug, Clone, Copy)]
pub enum TradeType {
    Buy,
    Sell,
}

/// 交易动作
#[derive(Debug, Clone, Copy)]
pub enum TradeAction {
    Buy,
    Sell,
    Hold,
}

/// 交易信号
#[derive(Debug, Clone)]
pub struct TradeSignal {
    pub allowed: bool,
    pub action: TradeAction,
    pub reason: String,
    pub stop_loss_price: Option<f64>,
    pub take_profit_price: Option<f64>,
}

/// 风控报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskReport {
    pub config: RiskConfig,
    pub total_return: f64,
    pub max_drawdown: f64,
    pub drawdown_peak_idx: usize,
    pub drawdown_alert_triggered: bool,
    pub drawdown_alert_message: String,
    pub final_equity: f64,
    pub risk_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_loss() {
        let manager = RiskManager::with_default();
        let (triggered, reason) = manager.check_stop_loss(100.0, 94.0);
        assert!(triggered, "5%亏损应触发止损");
    }

    #[test]
    fn test_take_profit() {
        let manager = RiskManager::with_default();
        let (triggered, reason) = manager.check_take_profit(100.0, 116.0);
        assert!(triggered, "16%盈利应触发止盈");
    }
}
