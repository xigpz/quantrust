//! 自动研报生成服务 (Auto Research Report Generator)
//!
//! 基于市场数据、新闻、情绪分析自动生成投资研报

use serde::{Deserialize, Serialize};
use chrono::{Local, Duration};
use crate::services::agent_perception::PerceptionModule;
use crate::services::agent_reasoning::ReasoningEngine;

/// 研报类型
#[derive(Debug, Clone, Copy)]
pub enum ReportType {
    MorningBrief,    // 晨间简报
    IntraDay,        // 盘中快讯
    EveningSummary,  // 收盘总结
    WeeklyOutlook,   // 周报展望
    SectorDeep,     // 板块深度分析
}

/// 研报结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    pub id: String,
    pub report_type: String,
    pub title: String,
    pub date: String,
    pub generated_at: String,
    // 宏观分析
    pub macro_analysis: MacroAnalysis,
    // 板块分析
    pub sector_analysis: SectorAnalysis,
    // 个股分析
    pub stock_analysis: StockAnalysis,
    // 投资建议
    pub investment_suggestions: InvestmentSuggestions,
    // 风险提示
    pub risk_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroAnalysis {
    pub global_market: String,        // 全球市场概况
    pub capital_flow: String,          // 资金流向分析
    pub sentiment_indicator: String,     // 情绪指标
    pub policy_outlook: String,         // 政策面展望
    pub macro_summary: String,          // 宏观小结
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorAnalysis {
    pub hot_sectors: Vec<SectorInfo>,      // 热点板块
    pub money_flow_sectors: Vec<SectorInfo>, // 资金流入板块
    pub potential_sectors: Vec<SectorInfo>,  // 潜在机会板块
    pub risky_sectors: Vec<SectorInfo>,      // 风险板块
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorInfo {
    pub name: String,
    pub change_pct: f64,
    pub hot_score: f64,
    pub money_flow: f64,  // 资金净流入 (万元)
    pub lead_stock: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAnalysis {
    pub limit_up_stocks: Vec<StockInfo>,      // 涨停股票
    pub unusual_volume: Vec<StockInfo>,       // 异常放量
    pub news_driven: Vec<StockInfo>,         // 消息驱动
    pub technical_breakout: Vec<StockInfo>,   // 技术突破
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInfo {
    pub symbol: String,
    pub name: String,
    pub change_pct: f64,
    pub reason: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentSuggestions {
    pub focus_sectors: Vec<String>,     // 重点关注板块
    pub watch_list: Vec<StockPick>,     // 自选股列表
    pub buy_candidates: Vec<StockPick>, // 买入候选
    pub operation_hints: Vec<String>,   // 操作提示
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockPick {
    pub symbol: String,
    pub name: String,
    pub entry_point: f64,
    pub stop_loss: f64,
    pub target: f64,
    pub reason: String,
    pub confidence: f64,
}

/// 自动研报生成器
pub struct AutoResearch {
    perception: PerceptionModule,
    reasoning: ReasoningEngine,
}

impl AutoResearch {
    pub fn new() -> Self {
        Self {
            perception: PerceptionModule::new(),
            reasoning: ReasoningEngine::new(),
        }
    }

    /// 生成晨间简报 (8:30)
    pub async fn generate_morning_brief(&self) -> ResearchReport {
        let now = Local::now();
        let today = now.format("%Y-%m-%d").to_string();

        // 获取外盘数据
        let external = self.perception.scan_external().await;

        // 获取市场数据
        let market = self.perception.scan_market().await;

        // 生成宏观分析
        let macro_analysis = self.generate_macro_analysis(&external, &market);

        // 生成板块分析
        let sector_analysis = self.generate_sector_analysis(&market);

        // 生成个股分析
        let stock_analysis = self.generate_stock_analysis(&market);

        // 生成投资建议
        let investment_suggestions = self.generate_suggestions(&sector_analysis, &stock_analysis);

        // 生成风险提示
        let risk_warnings = self.generate_risk_warnings(&external, &market);

        ResearchReport {
            id: uuid::Uuid::new_v4().to_string(),
            report_type: "晨间简报".to_string(),
            title: format!("{} 晨间投资简报", today),
            date: today,
            generated_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            macro_analysis,
            sector_analysis,
            stock_analysis,
            investment_suggestions,
            risk_warnings,
        }
    }

    /// 生成收盘总结 (15:30)
    pub async fn generate_evening_summary(&self) -> ResearchReport {
        let now = Local::now();
        let today = now.format("%Y-%m-%d").to_string();

        let market = self.perception.scan_market().await;
        let news = self.perception.scan_news().await;

        // 今日宏观分析
        let macro_analysis = MacroAnalysis {
            global_market: format!(
                "今日A股收涨/跌，成交量{}，市场情绪{}",
                if market.market_breadth.up > market.market_breadth.down { "放大" } else { "萎缩" },
                if market.limit_up_count > 50 { "偏暖" } else if market.limit_up_count < 20 { "偏冷" } else { "中性" }
            ),
            capital_flow: format!("今日涨停 {} 家，跌停 {} 家", market.limit_up_count, market.limit_down_count),
            sentiment_indicator: format!("市场广度: 上涨 {} 家，下跌 {} 家", market.market_breadth.up, market.market_breadth.down),
            policy_outlook: "关注政策面动态，等待进一步信号".to_string(),
            macro_summary: "今日市场表现平稳，板块轮动明显".to_string(),
        };

        // 板块分析
        let sector_analysis = self.generate_sector_analysis(&market);

        // 个股分析
        let stock_analysis = self.generate_stock_analysis(&market);

        // 投资建议
        let investment_suggestions = self.generate_suggestions(&sector_analysis, &stock_analysis);

        // 风险提示
        let risk_warnings = vec![
            "注意高位股补跌风险".to_string(),
            "关注成交量变化".to_string(),
            "警惕外盘夜盘波动".to_string(),
        ];

        ResearchReport {
            id: uuid::Uuid::new_v4().to_string(),
            report_type: "收盘总结".to_string(),
            title: format!("{} 收盘投资总结", today),
            date: today,
            generated_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            macro_analysis,
            sector_analysis,
            stock_analysis,
            investment_suggestions,
            risk_warnings,
        }
    }

    /// 生成宏观分析
    fn generate_macro_analysis(
        &self,
        external: &crate::services::agent_perception::ExternalMarket,
        _market: &crate::services::agent_perception::MarketData,
    ) -> MacroAnalysis {
        // 计算美股平均涨跌
        let us_avg = if external.us_futures.is_empty() {
            0.0
        } else {
            external.us_futures.iter().map(|i| i.change_pct).sum::<f64>() / external.us_futures.len() as f64
        };

        let global_market = if us_avg > 1.0 {
            "隔夜美股上涨，外盘环境偏暖".to_string()
        } else if us_avg < -1.0 {
            "隔夜美股下跌，外盘环境偏冷".to_string()
        } else {
            "隔夜美股平稳，外盘影响中性".to_string()
        };

        let capital_flow = if external.commodities.iter().any(|c| c.name.contains("黄金") && c.change_pct > 1.0) {
            "黄金上涨，反映避险情绪升温".to_string()
        } else {
            "大宗商品整体平稳，资金风险偏好中性".to_string()
        };

        MacroAnalysis {
            global_market,
            capital_flow,
            sentiment_indicator: "情绪指标偏中性，需进一步观察".to_string(),
            policy_outlook: "政策面平稳，关注后续导向".to_string(),
            macro_summary: "宏观环境整体平稳，暂无重大风险".to_string(),
        }
    }

    /// 生成板块分析
    fn generate_sector_analysis(
        &self,
        market: &crate::services::agent_perception::MarketData,
    ) -> SectorAnalysis {
        let mut hot_sectors = vec![];
        let mut money_flow_sectors = vec![];
        let mut potential_sectors = vec![];

        for sector in &market.hot_sectors {
            let info = SectorInfo {
                name: sector.name.clone(),
                change_pct: sector.change_pct,
                hot_score: sector.hot_score,
                money_flow: sector.flow_in,
                lead_stock: sector.lead_stock.clone(),
                reason: if sector.hot_score > 80.0 {
                    "市场热点".to_string()
                } else if sector.flow_in > 10000.0 {
                    "资金大幅流入".to_string()
                } else {
                    "关注".to_string()
                },
            };

            if sector.hot_score > 70.0 {
                hot_sectors.push(info.clone());
            }
            if sector.flow_in > 5000.0 {
                money_flow_sectors.push(info.clone());
            }
            if sector.hot_score > 60.0 && sector.hot_score <= 70.0 {
                potential_sectors.push(info);
            }
        }

        SectorAnalysis {
            hot_sectors,
            money_flow_sectors,
            potential_sectors,
            risky_sectors: vec![], // 暂无风险板块
        }
    }

    /// 生成个股分析
    fn generate_stock_analysis(
        &self,
        market: &crate::services::agent_perception::MarketData,
    ) -> StockAnalysis {
        // 从市场数据中提取热点股票
        let limit_up_stocks: Vec<StockInfo> = market.hot_sectors.iter()
            .filter(|s| s.change_pct > 9.5) // 涨停
            .take(10)
            .map(|s| StockInfo {
                symbol: s.lead_stock.clone(),
                name: s.lead_stock.clone(),
                change_pct: s.change_pct,
                reason: format!("{} 板块龙头", s.name),
                confidence: s.hot_score / 100.0,
            })
            .collect();

        StockAnalysis {
            limit_up_stocks,
            unusual_volume: vec![],
            news_driven: vec![],
            technical_breakout: vec![],
        }
    }

    /// 生成投资建议
    fn generate_suggestions(
        &self,
        sector_analysis: &SectorAnalysis,
        _stock_analysis: &StockAnalysis,
    ) -> InvestmentSuggestions {
        let focus_sectors: Vec<String> = sector_analysis.hot_sectors.iter()
            .take(3)
            .map(|s| s.name.clone())
            .collect();

        let watch_list: Vec<StockPick> = sector_analysis.hot_sectors.iter()
            .take(5)
            .map(|s| StockPick {
                symbol: s.lead_stock.clone(),
                name: s.lead_stock.clone(),
                entry_point: 0.0, // 待定
                stop_loss: 0.0,
                target: 0.0,
                reason: format!("{} 板块热点", s.name),
                confidence: s.hot_score / 100.0,
            })
            .collect();

        let operation_hints = if !focus_sectors.is_empty() {
            vec![
                format!("今日重点关注: {}", focus_sectors.join(", ")),
                "建议控制仓位，不追高位股".to_string(),
                "关注板块轮动机会".to_string(),
            ]
        } else {
            vec![
                "市场热点不明确，保持观望".to_string(),
                "控制仓位，等待机会".to_string(),
            ]
        };

        InvestmentSuggestions {
            focus_sectors,
            watch_list,
            buy_candidates: vec![],
            operation_hints,
        }
    }

    /// 生成风险提示
    fn generate_risk_warnings(
        &self,
        external: &crate::services::agent_perception::ExternalMarket,
        market: &crate::services::agent_perception::MarketData,
    ) -> Vec<String> {
        let mut warnings = vec![];

        // 检测外盘风险
        if external.us_futures.iter().any(|i| i.change_pct < -2.0) {
            warnings.push("美股期货大跌，注意防范隔夜风险".to_string());
        }

        // 检测市场风险
        if market.limit_down_count > 30 {
            warnings.push("跌停家数较多，市场情绪偏弱".to_string());
        }

        // 检测板块风险
        let sector_analysis = self.generate_sector_analysis(market);
        if sector_analysis.risky_sectors.len() > 2 {
            warnings.push("多个板块出现资金大幅流出".to_string());
        }

        if warnings.is_empty() {
            warnings.push("市场整体平稳，无重大风险提示".to_string());
        }

        warnings
    }

    /// 生成板块深度分析
    pub async fn generate_sector_deep_report(&self, sector_name: &str) -> String {
        let market = self.perception.scan_market().await;

        // 查找指定板块
        let sector = market.hot_sectors.iter()
            .find(|s| s.name == sector_name);

        if let Some(s) = sector {
            format!(
                "# {} 深度分析报告\n\n\
                ## 板块概况\n\
                - 当前涨跌幅: {:.2}%\n\
                - 热度评分: {:.0}/100\n\
                - 资金流向: {:.0}万元\n\
                - 龙头股: {}\n\n\
                ## 板块分析\n\
                {}\n\n\
                ## 投资建议\n\
                {}\n",
                s.name,
                s.change_pct,
                s.hot_score,
                s.flow_in,
                s.lead_stock,
                if s.hot_score > 70.0 { "热点板块，可适当关注" } else { "板块热度一般，保持观望" },
                format!("建议关注龙头股{}的走势", s.lead_stock)
            )
        } else {
            format!("未找到板块「{}」的分析数据", sector_name)
        }
    }

    /// 生成周报展望
    pub async fn generate_weekly_outlook(&self) -> ResearchReport {
        let now = Local::now();
        let today = now.format("%Y-%m-%d").to_string();

        // 获取本周数据
        let market = self.perception.scan_market().await;

        ResearchReport {
            id: uuid::Uuid::new_v4().to_string(),
            report_type: "周报展望".to_string(),
            title: format!("第{}周市场展望", now.format("%W")),
            date: today,
            generated_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            macro_analysis: MacroAnalysis {
                global_market: "本周全球市场整体平稳".to_string(),
                capital_flow: "资金观望情绪较浓".to_string(),
                sentiment_indicator: "市场情绪中性偏暖".to_string(),
                policy_outlook: "政策面保持稳定".to_string(),
                macro_summary: "宏观环境平稳，市场以结构性机会为主".to_string(),
            },
            sector_analysis: self.generate_sector_analysis(&market),
            stock_analysis: self.generate_stock_analysis(&market),
            investment_suggestions: self.generate_suggestions(
                &self.generate_sector_analysis(&market),
                &self.generate_stock_analysis(&market),
            ),
            risk_warnings: vec![
                "注意月末资金面紧张".to_string(),
                "关注外盘不确定性风险".to_string(),
            ],
        }
    }
}

impl Default for AutoResearch {
    fn default() -> Self {
        Self::new()
    }
}

/// 格式化研报为 Markdown
pub fn format_report_markdown(report: &ResearchReport) -> String {
    let mut md = format!(
        "# {}\n\n",
        report.title
    );
    md += &format!("**类型:** {} | **日期:** {} | **生成时间:** {}\n\n",
        report.report_type, report.date, report.generated_at);

    // 宏观分析
    md += "## 宏观分析\n\n";
    md += &format!("- **全球市场:** {}\n", report.macro_analysis.global_market);
    md += &format!("- **资金流向:** {}\n", report.macro_analysis.capital_flow);
    md += &format!("- **情绪指标:** {}\n", report.macro_analysis.sentiment_indicator);
    md += &format!("- **政策展望:** {}\n", report.macro_analysis.policy_outlook);
    md += &format!("- **宏观小结:** {}\n\n", report.macro_analysis.macro_summary);

    // 板块分析
    md += "## 板块分析\n\n";
    if !report.sector_analysis.hot_sectors.is_empty() {
        md += "### 热点板块\n\n";
        for sector in &report.sector_analysis.hot_sectors {
            md += &format!(
                "- **{}** ({:.2}%) 热度:{:.0} 资金:{:.0}万 龙头:{}\n",
                sector.name, sector.change_pct, sector.hot_score, sector.money_flow, sector.lead_stock
            );
        }
        md += "\n";
    }

    if !report.sector_analysis.money_flow_sectors.is_empty() {
        md += "### 资金流入板块\n\n";
        for sector in &report.sector_analysis.money_flow_sectors {
            md += &format!(
                "- **{}** 资金流入 {:.0}万\n",
                sector.name, sector.money_flow
            );
        }
        md += "\n";
    }

    // 投资建议
    md += "## 投资建议\n\n";
    if !report.investment_suggestions.focus_sectors.is_empty() {
        md += &format!("**重点关注:** {}\n\n",
            report.investment_suggestions.focus_sectors.join(", "));
    }

    if !report.investment_suggestions.operation_hints.is_empty() {
        md += "### 操作提示\n\n";
        for hint in &report.investment_suggestions.operation_hints {
            md += &format!("- {}\n", hint);
        }
        md += "\n";
    }

    // 风险提示
    if !report.risk_warnings.is_empty() {
        md += "## 风险提示\n\n";
        for warning in &report.risk_warnings {
            md += &format!("- ⚠️ {}\n", warning);
        }
        md += "\n";
    }

    md += "---\n";
    md += &format!("*报告生成时间: {}*\n", report.generated_at);

    md
}
