use crate::models::timing::*;
use chrono::{Datelike, Local, Timelike};

/// 时机优化服务
pub struct TimingOptimizer;

impl TimingOptimizer {
    /// 获取当前交易时机信号
    pub fn get_timing_signal() -> TimingSignal {
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let intraday = IntradayWindow::current();
        let annual = AnnualWindow::current();

        // 计算信号强度
        let intraday_score = intraday.score();
        let annual_score = annual.score();
        let action_strength = ((intraday_score * 0.6 + annual_score * 0.4) as i32).min(100);

        // 确定交易指令
        let action = Self::determine_action(intraday, annual, action_strength);

        // 仓位建议
        let position_advice = Self::calculate_position_advice(intraday, annual, action);

        // 板块推荐
        let sector_recommendation = Self::get_sector_recommendation(intraday, annual);

        // 股票推荐
        let stock_picks = Self::get_stock_picks(intraday, annual, &sector_recommendation);

        // 风险提示
        let risk_warnings = Self::get_risk_warnings(intraday, annual, action);

        // 操作步骤
        let actionable_steps = Self::get_actionable_steps(
            intraday,
            annual,
            action,
            &stock_picks,
            &position_advice,
        );

        TimingSignal {
            timestamp,
            action,
            action_strength,
            intraday_window: format!("{:?}", intraday),
            intraday_score: intraday_score as i32,
            intraday_remaining_minutes: intraday.remaining_minutes(),
            intraday_next: intraday
                .next_window()
                .map(|w| w.name().to_string())
                .unwrap_or_default(),
            annual_window: format!("{:?}", annual),
            annual_nickname: annual.nickname().to_string(),
            annual_score: annual_score as i32,
            annual_win_rate: annual.win_rate(),
            position_advice,
            sector_recommendation,
            stock_picks,
            risk_warnings,
            actionable_steps,
        }
    }

    /// 确定交易指令
    fn determine_action(intraday: IntradayWindow, annual: AnnualWindow, strength: i32) -> TradeAction {
        // 非交易时间观望
        if matches!(intraday, IntradayWindow::Closed) {
            return TradeAction::Watch;
        }

        // 垃圾时间只看不操作
        if matches!(intraday, IntradayWindow::MiddayLull) {
            return TradeAction::Watch;
        }

        // 根据信号强度和时段确定指令
        match strength {
            s if s >= 80 && matches!(intraday, IntradayWindow::MorningPeak | IntradayWindow::AfternoonActive) => {
                TradeAction::Buy
            }
            s if s >= 65 => TradeAction::Buy,
            s if s >= 45 => TradeAction::Hold,
            _ => TradeAction::Watch,
        }
    }

    /// 计算仓位建议
    fn calculate_position_advice(
        intraday: IntradayWindow,
        annual: AnnualWindow,
        action: TradeAction,
    ) -> PositionAdvice {
        if matches!(action, TradeAction::Watch | TradeAction::Sell) {
            return PositionAdvice {
                current_position: PositionLevel::Empty,
                max_position: PositionLevel::Light,
                stop_loss_pct: 0.0,
                take_profit_pct: 0.0,
            };
        }

        let current = annual.recommended_position();
        let (stop_loss, take_profit) = Self::get_stop_loss_take_profit(intraday, annual);

        PositionAdvice {
            current_position: current,
            max_position: PositionLevel::Full,
            stop_loss_pct: stop_loss,
            take_profit_pct: take_profit,
        }
    }

    /// 获取止损止盈位
    fn get_stop_loss_take_profit(intraday: IntradayWindow, annual: AnnualWindow) -> (f64, f64) {
        // 高波动期止损放大
        let base_stop_loss = match annual {
            AnnualWindow::YearEnd => 7.0,  // 跨年行情波动大
            AnnualWindow::SpringRally => 5.0,
            _ => 5.0,
        };

        // 垃圾时间不设止盈目标太远
        let base_take_profit = match intraday {
            IntradayWindow::MorningPeak => 15.0,
            IntradayWindow::AfternoonActive => 10.0,
            _ => 8.0,
        };

        (base_stop_loss, base_take_profit)
    }

    /// 获取板块推荐
    fn get_sector_recommendation(intraday: IntradayWindow, annual: AnnualWindow) -> SectorRecommendation {
        let (hot, defensive, reason, risk, position) = match (intraday, annual) {
            // 早盘 + 春季躁动 = 最佳窗口
            (IntradayWindow::MorningPeak, AnnualWindow::SpringRally) => (
                vec!["人工智能".to_string(), "半导体".to_string(), "新能源车".to_string(), "消费电子".to_string()],
                vec!["银行".to_string()],
                "春季躁动叠加早盘高峰，进攻型仓位最佳窗口".to_string(),
                WarningLevel::Yellow,
                PositionLevel::Heavy,
            ),
            // 早盘 + 跨年行情
            (IntradayWindow::MorningPeak, AnnualWindow::YearEnd) => (
                vec!["元宇宙".to_string(), "AI".to_string(), "军工".to_string(), "数字经济".to_string()],
                vec!["医药".to_string()],
                "跨年行情游资活跃，主题炒作正当时".to_string(),
                WarningLevel::Orange,
                PositionLevel::Heavy,
            ),
            // 早盘 + 中报行情
            (IntradayWindow::MorningPeak, AnnualWindow::MidYear) => (
                vec!["券商".to_string(), "有色金属".to_string(), "业绩超预期".to_string()],
                vec!["银行".to_string(), "电力".to_string()],
                "中报业绩驱动，七翻身行情进行中".to_string(),
                WarningLevel::Yellow,
                PositionLevel::Half,
            ),
            // 尾盘活跃
            (IntradayWindow::AfternoonActive, _) => (
                vec!["券商".to_string(), "科技".to_string(), "新能源".to_string()],
                vec!["银行".to_string(), "消费".to_string()],
                "尾盘异动，机构调仓换股".to_string(),
                WarningLevel::Yellow,
                PositionLevel::Half,
            ),
            // 集合竞价
            (IntradayWindow::PreAuction, _) => (
                vec!["昨日涨停".to_string(), "龙头股".to_string(), "隔夜消息股".to_string()],
                vec!["银行".to_string()],
                "盘前布局，依据隔夜消息挂单".to_string(),
                WarningLevel::Orange,
                PositionLevel::Light,
            ),
            // 垃圾时间
            (IntradayWindow::MiddayLull, _) => (
                vec![],
                vec!["银行".to_string(), "白酒".to_string(), "医药".to_string(), "电力".to_string()],
                "垃圾时间，观望为主".to_string(),
                WarningLevel::Yellow,
                PositionLevel::Light,
            ),
            // 盘后交易
            (IntradayWindow::AfterHours, _) => (
                vec!["科创板".to_string(), "创业板".to_string()],
                vec!["银行".to_string()],
                "盘后交易，大额订单按收盘价成交".to_string(),
                WarningLevel::Yellow,
                PositionLevel::Half,
            ),
            // 默认
            _ => (
                vec!["科技".to_string(), "新能源".to_string()],
                vec!["银行".to_string(), "医药".to_string()],
                "常规时段，稳健布局".to_string(),
                WarningLevel::Yellow,
                PositionLevel::Half,
            ),
        };

        SectorRecommendation {
            hot_sectors: hot,
            defensive_sectors: defensive,
            sector_reason: reason,
            risk_level: risk,
            position_advice: position,
        }
    }

    /// 获取股票推荐
    fn get_stock_picks(
        intraday: IntradayWindow,
        annual: AnnualWindow,
        sector: &SectorRecommendation,
    ) -> Vec<StockPick> {
        // 只有在可买入时段才推荐具体股票
        if !matches!(
            intraday,
            IntradayWindow::MorningPeak | IntradayWindow::AfternoonActive | IntradayWindow::PreAuction
        ) {
            return vec![];
        }

        // 根据年度窗口推荐不同标的
        let picks = match annual {
            AnnualWindow::SpringRally => vec![
                StockPick {
                    symbol: "002049".to_string(),
                    name: "紫光国微".to_string(),
                    reason: "AI芯片龙头，高弹性".to_string(),
                    entry_min: 85.0,
                    entry_max: 90.0,
                    stop_loss_pct: 5.0,
                    target_pct: 15.0,
                },
                StockPick {
                    symbol: "688256".to_string(),
                    name: "寒武纪".to_string(),
                    reason: "人工智能核心标的".to_string(),
                    entry_min: 100.0,
                    entry_max: 110.0,
                    stop_loss_pct: 7.0,
                    target_pct: 20.0,
                },
            ],
            AnnualWindow::MidYear => vec![
                StockPick {
                    symbol: "600030".to_string(),
                    name: "中信证券".to_string(),
                    reason: "券商龙头，业绩超预期".to_string(),
                    entry_min: 22.0,
                    entry_max: 24.0,
                    stop_loss_pct: 5.0,
                    target_pct: 12.0,
                },
            ],
            AnnualWindow::YearEnd => vec![
                StockPick {
                    symbol: "300750".to_string(),
                    name: "宁德时代".to_string(),
                    reason: "新能源龙头，主题炒作".to_string(),
                    entry_min: 180.0,
                    entry_max: 190.0,
                    stop_loss_pct: 7.0,
                    target_pct: 15.0,
                },
            ],
            _ => vec![],
        };

        picks
    }

    /// 获取风险提示
    fn get_risk_warnings(
        intraday: IntradayWindow,
        annual: AnnualWindow,
        action: TradeAction,
    ) -> Vec<RiskWarning> {
        let mut warnings = vec![];

        // 垃圾时间风险
        if matches!(intraday, IntradayWindow::MiddayLull) {
            warnings.push(RiskWarning {
                level: WarningLevel::Orange,
                message: "现在是垃圾时间(10:30-14:30)，不建议操作".to_string(),
                suggestion: "只看不动，等待下一个窗口".to_string(),
            });
        }

        // 追高风险
        if matches!(intraday, IntradayWindow::MorningPeak) {
            warnings.push(RiskWarning {
                level: WarningLevel::Orange,
                message: "早盘追高风险大".to_string(),
                suggestion: "回调后再买，不要追涨停".to_string(),
            });
        }

        // 跨年行情高风险
        if matches!(annual, AnnualWindow::YearEnd) {
            warnings.push(RiskWarning {
                level: WarningLevel::Yellow,
                message: "跨年行情妖股波动大".to_string(),
                suggestion: "控制仓位，快进快出".to_string(),
            });
        }

        // 非买入指令
        if matches!(action, TradeAction::Watch) {
            warnings.push(RiskWarning {
                level: WarningLevel::Yellow,
                message: "当前时机不适合操作".to_string(),
                suggestion: "等待更好的时机".to_string(),
            });
        }

        // 强制止损提示
        warnings.push(RiskWarning {
            level: WarningLevel::Yellow,
            message: "止损必须严格执行，不侥幸".to_string(),
            suggestion: "设置自动止损单".to_string(),
        });

        warnings
    }

    /// 获取具体操作步骤
    fn get_actionable_steps(
        intraday: IntradayWindow,
        annual: AnnualWindow,
        action: TradeAction,
        picks: &[StockPick],
        position: &PositionAdvice,
    ) -> Vec<String> {
        let mut steps = vec![];

        // 当前时机描述
        let time_desc = match intraday {
            IntradayWindow::MorningPeak => "现在是早盘高峰(9:30-10:30)，最佳操作窗口",
            IntradayWindow::AfternoonActive => "现在是尾盘活跃(14:30-15:00)，可操作",
            IntradayWindow::PreAuction => "现在是集合竞价(9:15-9:25)，可挂单布局",
            IntradayWindow::MiddayLull => "现在是垃圾时间(10:30-14:30)，建议观望",
            IntradayWindow::AfterHours => "现在是盘后交易(15:05-15:30)",
            IntradayWindow::Closed => "现在是非交易时间",
        };
        steps.push(time_desc.to_string());

        // 操作指令
        match action {
            TradeAction::Buy => {
                if let Some(pick) = picks.first() {
                    steps.push(format!(
                        "建议买入: {} ({}), 价格 {}-{}元",
                        pick.name, pick.symbol, pick.entry_min, pick.entry_max
                    ));
                    steps.push(format!("止损位: -{:.0}% ({:.1}元)", pick.stop_loss_pct, pick.entry_min * (1.0 - pick.stop_loss_pct / 100.0)));
                    steps.push(format!("目标位: +{:.0}% ({:.1}元)", pick.target_pct, pick.entry_max * (1.0 + pick.target_pct / 100.0)));
                }
                steps.push(format!("建议仓位: {} ({}%)", position.current_position.name(), position.current_position.percentage()));
                steps.push("10:30前完成建仓".to_string());
            }
            TradeAction::Sell => {
                steps.push("考虑卖出: 建议关注止盈时机".to_string());
                steps.push("止损单预设好".to_string());
            }
            TradeAction::Hold => {
                steps.push("建议持有: 等待更好时机再加仓".to_string());
            }
            TradeAction::Watch => {
                steps.push("建议观望: 当前时机不佳，不操作".to_string());
                if matches!(intraday, IntradayWindow::MiddayLull) {
                    steps.push("下一个操作窗口: 14:30 尾盘活跃期".to_string());
                }
            }
        }

        steps
    }

    /// 获取日内时段详情
    pub fn get_intraday_detail() -> IntradayWindowDetail {
        let intraday = IntradayWindow::current();
        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute();
        let time_minutes = hour * 60 + minute;

        let (remaining, next_time) = match intraday {
            IntradayWindow::PreAuction => (565 - time_minutes, "9:25 开盘"),
            IntradayWindow::MorningPeak => (630 - time_minutes, "10:30 垃圾时间"),
            IntradayWindow::MiddayLull => (870 - time_minutes, "14:30 尾盘"),
            IntradayWindow::AfternoonActive => (900 - time_minutes, "15:00 收盘"),
            IntradayWindow::AfterHours => (930 - time_minutes, "15:30 结束"),
            IntradayWindow::Closed => (0, "等待开盘"),
        };

        IntradayWindowDetail {
            current: format!("{:?}", intraday),
            current_name: intraday.name().to_string(),
            current_score: intraday.score() as i32,
            remaining_minutes: remaining.max(0) as i32,
            next: intraday.next_window().map(|w| format!("{:?}", w)).unwrap_or_default(),
            next_name: next_time.to_string(),
            description: intraday.description().to_string(),
        }
    }

    /// 获取年度窗口详情
    pub fn get_annual_detail() -> AnnualWindowDetail {
        let annual = AnnualWindow::current();
        let now = Local::now();
        let month = now.month();

        // 计算到下一个窗口的月数
        let (next, countdown) = match annual {
            AnnualWindow::SpringRally => (Some("5-6月夏季淡季"), 2),
            AnnualWindow::SummerDull => (Some("7-9月中报行情"), 1),
            AnnualWindow::MidYear => (Some("10月秋季淡季"), 1),
            AnnualWindow::AutumnDull => (Some("11-1月跨年行情"), 1),
            AnnualWindow::YearEnd => (Some("2-3月春季躁动"), 1),
            AnnualWindow::OffSeason => (None, 0),
        };

        AnnualWindowDetail {
            current: format!("{:?}", annual),
            current_name: annual.name().to_string(),
            current_nickname: annual.nickname().to_string(),
            current_score: annual.score() as i32,
            win_rate: annual.win_rate(),
            recommended_position: annual.recommended_position().name().to_string(),
            next: next.map(|s| s.to_string()),
            next_name: next.map(|s| s.to_string()),
            countdown_months: countdown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intraday_windows() {
        let intraday = IntradayWindow::current();
        println!("Current intraday window: {:?}", intraday);
        println!("Name: {}", intraday.name());
        println!("Score: {}", intraday.score());
    }

    #[test]
    fn test_annual_windows() {
        let annual = AnnualWindow::current();
        println!("Current annual window: {:?}", annual);
        println!("Name: {}", annual.name());
        println!("Win rate: {}%", annual.win_rate());
    }

    #[test]
    fn test_timing_signal() {
        let signal = TimingOptimizer::get_timing_signal();
        println!("Action: {:?}", signal.action);
        println!("Action strength: {}", signal.action_strength);
        println!("Position: {}", signal.position_advice.current_position.name());
    }
}
