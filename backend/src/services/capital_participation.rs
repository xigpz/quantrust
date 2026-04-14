use crate::models::*;
use crate::services::DragonTigerService;

/// 资金参与度分析服务
pub struct CapitalParticipationService;

impl CapitalParticipationService {
    /// 分析股票的资金参与度
    pub async fn analyze(
        provider: &crate::data::DataProvider,
        symbol: &str,
    ) -> Result<CapitalParticipation, String> {
        // 获取股票基本信息
        let quote = match provider.get_stock_detail(symbol).await {
            Ok(q) => q,
            Err(_) => return Err("获取股票信息失败".to_string()),
        };

        // 获取资金流向
        let flows = match provider.get_money_flow(100).await {
            Ok(f) => f,
            Err(_) => Vec::new(),
        };
        // 尝试多种格式匹配
        let stock_code = symbol.split('.').next().unwrap_or(symbol);
        let stock_flow = flows.iter().find(|f| {
            f.symbol == symbol ||
            f.symbol == format!("{}.SH", stock_code) ||
            f.symbol == format!("{}.SZ", stock_code) ||
            f.symbol.contains(stock_code)
        });

        // 获取机构持仓
        let holdings = match provider.get_institutional_holdings(symbol, 1).await {
            Ok(h) => h.list,
            Err(_) => Vec::new(),
        };

        // 获取龙虎榜数据
        let dragon_tiger_count = match DragonTigerService::new().get_daily_list().await {
            Ok(data) => {
                let stock_code = symbol.split('.').next().unwrap_or(symbol);
                data.into_iter().filter(|d| d.symbol.contains(stock_code)).count() as i32
            }
            Err(_) => 0,
        };

        // 获取涨停次数(近5日)
        let limit_up_count = if quote.change_pct >= 9.5 { 1 } else { 0 };

        // 计算量化资金评分 (0-100)
        let (quant_score, quant_inflow, quant_ratio, institution_hold_ratio, institution_change, quant_trend) =
            Self::calc_quant_score(&quote, stock_flow, &holdings);

        // 计算游资评分 (0-100)
        let (hot_money_score, hot_money_inflow, hot_money_ratio, super_large_inflow, large_inflow, hot_trend) =
            Self::calc_hot_money_score(&quote, stock_flow, limit_up_count, dragon_tiger_count);

        // 综合评分
        let total_score = (quant_score * 0.4 + hot_money_score * 0.6).min(100.0);

        // 确定参与类型
        let participation_type = Self::calc_participation_type(quant_score, hot_money_score, institution_hold_ratio);

        // 生成投资建议
        let advice = Self::generate_advice(
            quant_score,
            hot_money_score,
            institution_hold_ratio,
            &participation_type,
            quote.change_pct,
        );

        Ok(CapitalParticipation {
            symbol: symbol.to_string(),
            name: quote.name,
            quant_score,
            quant_inflow,
            quant_ratio,
            institution_hold_ratio,
            institution_change,
            quant_trend,
            hot_money_score,
            hot_money_inflow,
            hot_money_ratio,
            super_large_inflow,
            large_inflow,
            limit_up_count,
            dragon_tiger_count,
            hot_trend,
            total_score,
            participation_type,
            advice,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// 计算量化资金评分
    fn calc_quant_score(
        quote: &StockQuote,
        flow: Option<&MoneyFlow>,
        holdings: &[InstitutionalHolding],
    ) -> (f64, f64, f64, f64, f64, String) {
        // 机构持股比例
        let institution_hold_ratio: f64 = holdings
            .iter()
            .map(|h| h.hold_ratio)
            .sum::<f64>()
            .min(50.0); // 最高50%

        // 机构持股变化
        let institution_change = holdings
            .first()
            .map(|h| h.change_ratio)
            .unwrap_or(0.0);

        // 量化资金净流入 (超大单+大单 - 这些通常是量化基金)
        let quant_inflow = flow
            .map(|f| f.super_large_inflow + f.large_inflow)
            .unwrap_or(0.0);

        // 量化资金占比
        let total_inflow = flow
            .map(|f| {
                f.main_net_inflow.abs()
                    + f.super_large_inflow.abs()
                    + f.large_inflow.abs()
                    + f.medium_inflow.abs()
                    + f.small_inflow.abs()
            })
            .unwrap_or(1.0);

        let quant_ratio = if total_inflow > 0.0 {
            ((quant_inflow.abs() / total_inflow) * 100.0).min(100.0)
        } else {
            0.0
        };

        // 量化资金评分计算
        let mut score = 0.0;

        // 机构持股比例评分 (0-40分)
        score += (institution_hold_ratio / 50.0 * 40.0).min(40.0);

        // 机构持股变化评分 (0-20分)
        if institution_change > 0.0 {
            score += (institution_change.min(10.0) / 10.0 * 20.0).min(20.0);
        }

        // 量化资金净流入方向评分 (0-20分)
        if quant_inflow > 0.0 {
            score += 10.0; // 净流入
            // 净流入金额评分
            if quant_inflow > 1e8 {
                score += 10.0;
            } else if quant_inflow > 5e7 {
                score += 5.0;
            }
        }

        // 换手率评分 (0-20分)
        if quote.turnover_rate > 20.0 {
            score += 20.0;
        } else if quote.turnover_rate > 10.0 {
            score += 15.0;
        } else if quote.turnover_rate > 5.0 {
            score += 10.0;
        } else {
            score += 5.0;
        }

        // 趋势判断
        let trend = if institution_change > 5.0 && quant_inflow > 0.0 {
            "increasing".to_string()
        } else if institution_change < -5.0 || quant_inflow < -1e7 {
            "decreasing".to_string()
        } else {
            "stable".to_string()
        };

        (
            score.min(100.0),
            quant_inflow,
            quant_ratio,
            institution_hold_ratio,
            institution_change,
            trend,
        )
    }

    /// 计算游资评分
    fn calc_hot_money_score(
        quote: &StockQuote,
        flow: Option<&MoneyFlow>,
        limit_up_count: i32,
        dragon_tiger_count: i32,
    ) -> (f64, f64, f64, f64, f64, String) {
        // 游资净流入 (中单+小单)
        let hot_money_inflow = flow
            .map(|f| f.medium_inflow + f.small_inflow)
            .unwrap_or(0.0);

        // 超大单净流入
        let super_large_inflow = flow.map(|f| f.super_large_inflow).unwrap_or(0.0);

        // 大单净流入
        let large_inflow = flow.map(|f| f.large_inflow).unwrap_or(0.0);

        // 游资占比
        let total_inflow = flow
            .map(|f| {
                f.main_net_inflow.abs()
                    + f.super_large_inflow.abs()
                    + f.large_inflow.abs()
                    + f.medium_inflow.abs()
                    + f.small_inflow.abs()
            })
            .unwrap_or(1.0);

        let hot_money_ratio = if total_inflow > 0.0 {
            ((hot_money_inflow.abs() / total_inflow) * 100.0).min(100.0)
        } else {
            0.0
        };

        // 游资评分计算
        let mut score = 0.0;

        // 涨幅评分 (0-25分)
        if quote.change_pct >= 9.5 {
            score += 25.0;
        } else if quote.change_pct >= 5.0 {
            score += 20.0;
        } else if quote.change_pct >= 3.0 {
            score += 15.0;
        } else if quote.change_pct >= 0.0 {
            score += 10.0;
        } else {
            score += 5.0;
        }

        // 振幅评分 (0-15分)
        if quote.amplitude >= 10.0 {
            score += 15.0;
        } else if quote.amplitude >= 5.0 {
            score += 10.0;
        } else {
            score += 5.0;
        }

        // 换手率评分 (0-20分)
        if quote.turnover_rate >= 30.0 {
            score += 20.0;
        } else if quote.turnover_rate >= 20.0 {
            score += 15.0;
        } else if quote.turnover_rate >= 10.0 {
            score += 10.0;
        } else {
            score += 5.0;
        }

        // 涨停次数评分 (0-15分)
        score += (limit_up_count as f64 * 5.0).min(15.0);

        // 龙虎榜上榜评分 (0-10分)
        score += (dragon_tiger_count as f64 * 3.0).min(10.0);

        // 大单活跃评分 (0-15分)
        let big_order_total = super_large_inflow.abs() + large_inflow.abs();
        if big_order_total > 1e8 {
            score += 15.0;
        } else if big_order_total > 5e7 {
            score += 10.0;
        } else if big_order_total > 1e7 {
            score += 5.0;
        }

        // 趋势判断
        let trend = if (quote.change_pct > 5.0 || limit_up_count > 0) && dragon_tiger_count > 0 {
            "increasing".to_string()
        } else if quote.change_pct < -3.0 || dragon_tiger_count == 0 {
            "decreasing".to_string()
        } else {
            "stable".to_string()
        };

        (
            score.min(100.0),
            hot_money_inflow,
            hot_money_ratio,
            super_large_inflow,
            large_inflow,
            trend,
        )
    }

    /// 计算参与类型
    fn calc_participation_type(quant_score: f64, hot_money_score: f64, institution_hold: f64) -> String {
        if quant_score > 60.0 && institution_hold > 20.0 {
            "机构主导".to_string()
        } else if quant_score > 50.0 && hot_money_score < 40.0 {
            "量化主导".to_string()
        } else if hot_money_score > 60.0 && quant_score < 40.0 {
            "游资主导".to_string()
        } else if (quant_score - hot_money_score).abs() < 15.0 {
            "均衡".to_string()
        } else {
            "清淡".to_string()
        }
    }

    /// 生成投资建议
    fn generate_advice(
        quant_score: f64,
        hot_money_score: f64,
        institution_hold: f64,
        participation_type: &str,
        change_pct: f64,
    ) -> String {
        match participation_type {
            "机构主导" => {
                if change_pct > 5.0 {
                    "机构大幅买入，短期有拉升，可适当关注".to_string()
                } else if change_pct > 0.0 {
                    "机构持续吸筹，股价稳步上涨，可中线持有".to_string()
                } else {
                    "机构逆势加仓，可能在底部吸筹，可关注".to_string()
                }
            }
            "量化主导" => {
                if quant_score > 70.0 {
                    "量化资金高度活跃，波动可能较大，适合量化策略".to_string()
                } else {
                    "量化资金稳定参与，适合中长线布局".to_string()
                }
            }
            "游资主导" => {
                if change_pct >= 9.5 {
                    "游资强势拉升，接近涨停，注意风险".to_string()
                } else if change_pct >= 5.0 {
                    "游资积极运作，短期波动大，谨慎追高".to_string()
                } else {
                    "游资试盘，可能有短线机会".to_string()
                }
            }
            "均衡" => {
                "多空力量均衡，观望为主".to_string()
            }
            _ => {
                "资金参与度较低，保持观望".to_string()
            }
        }
    }
}
