use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::models::StockNews;

/// 市场类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarketType {
    #[serde(rename = "A股")]
    AShare,
    #[serde(rename = "港股")]
    HK,
    #[serde(rename = "美股")]
    US,
}

impl MarketType {
    pub fn from_column(column: &str) -> Option<Self> {
        if column.contains("100") {
            Some(MarketType::AShare)
        } else if column.contains("102") {
            Some(MarketType::HK)
        } else if column.contains("104") {
            Some(MarketType::US)
        } else {
            None
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            MarketType::AShare => "A股",
            MarketType::HK => "港股",
            MarketType::US => "美股",
        }
    }
}

/// 带市场的新闻
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketNews {
    #[serde(flatten)]
    pub news: StockNews,
    pub market: MarketType,
}

/// 新闻影响类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactType {
    #[serde(rename = "利好")]
    Positive,
    #[serde(rename = "利空")]
    Negative,
    #[serde(rename = "中性")]
    Neutral,
}

/// 受影响股票
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactedStock {
    pub symbol: String,
    pub name: String,
    pub impact_type: ImpactType,
    pub impact_strength: f64,  // 0-1
    pub confidence: f64,       // 置信度 0-1
}

/// 单条新闻分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsAnalysis {
    pub news: StockNews,
    pub impact_type: ImpactType,
    pub impact_stocks: Vec<ImpactedStock>,
    pub sectors: Vec<String>,
    pub strength: f64,        // 影响强度 0-1
    pub reason: String,        // 分析原因
}

/// 新闻影响分析结果
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NewsImpactResult {
    pub news_list: Vec<NewsAnalysis>,
    pub top_stocks: Vec<ImpactedStock>,                    // 按影响评分排序的股票
    pub sector_impacts: std::collections::HashMap<String, f64>,  // 板块影响评分
    pub market_sentiment: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    // 多市场支持
    pub market_news: std::collections::HashMap<String, Vec<NewsAnalysis>>, // 按市场分类的新闻
    pub cross_market_sentiment: String,  // 跨市场情绪
}

/// DeepSeek API调用
pub struct DeepSeekClient {
    client: Client,
    api_key: String,
}

impl DeepSeekClient {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("DEEPSEEK_API_KEY")
            .map_err(|_| anyhow::anyhow!("DEEPSEEK_API_KEY not set"))?;
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?,
            api_key,
        })
    }

    /// 分析新闻对市场的影响
    pub async fn analyze_news_impact(&self, news: &StockNews, market: &MarketType) -> Result<NewsAnalysis> {
        let (system_prompt, user_prompt_template) = match market {
            MarketType::AShare => {
                (r#"你是一个客观、专业的A股市场分析师。你的分析必须基于事实，避免过度乐观或过度悲观。

分析原则：
1. 区分"实质性利好/利空"和"模糊的正面/负面"
   - 实质性利好：具体订单合同、业绩大幅增长、政策明确支持、核心技术突破等
   - 模糊正面：一般性合作意向、行业发展预期、区域政策等（影响不确定）
   - 同样原则适用于利空

2. 考虑市场整体环境：
   - 美股下跌、全球经济放缓等外部压力会抵消国内利好
   - 市场情绪极度悲观时，好消息可能无力反弹
   - 市场情绪过度亢奋时，利空可能被忽视

3. 评级标准：
   - 利好：必须是具体、可见的影响，关联度要高
   - 利空：同样需要具体、可见
   - 中性：影响不确定、关联度低、或正负参半

4. 输出要求：
   - 只返回JSON，不要有其他内容
   - A股股票代码使用6位数字，如000001表示平安银行，600519表示贵州茅台
   - 如果新闻没有明显影响的相关股票，返回空数组
   - 置信度体现分析确定性，0-1之间"#,
                r#"分析以下新闻对A股市场的影响：

标题：{}
内容：{}
时间：{}
来源：{}

请返回JSON格式：
{
    "impact_type": "利好/利空/中性",
    "impact_stocks": [
        {"symbol": "股票代码", "name": "股票名称", "impact_type": "利好/利空/中性", "impact_strength": 0.0-1.0, "confidence": 0.0-1.0}
    ],
    "sectors": ["相关板块1", "相关板块2"],
    "strength": 0.0-1.0,
    "reason": "简短分析原因"
}"#)
            },
            MarketType::HK => {
                (r#"你是一个客观、专业的港股市场分析师。你的分析必须基于事实，避免过度乐观或过度悲观。

分析原则：
1. 区分"实质性利好/利空"和"模糊的正面/负面"
2. 考虑美股夜盘、A股表现对港股的联动影响
3. 注意港股独特的交易制度（T+0、无涨跌停）

股票代码格式说明：
- 港股代码使用4位数字+.HK后缀，如0700.HK表示腾讯控股，9988.HK表示阿里巴巴
- 常见港股：0700(腾讯)、9988(阿里)、9618(京东)、3690(美团)、1810(小米)、0941(中国移动)、1398(工商银行)、3968(招商银行)

输出要求：
- 只返回JSON，不要有其他内容
- 如果新闻没有明显影响的相关股票，返回空数组
- 置信度体现分析确定性，0-1之间"#,
                r#"分析以下新闻对港股市场的影响：

标题：{}
内容：{}
时间：{}
来源：{}

请返回JSON格式：
{
    "impact_type": "利好/利空/中性",
    "impact_stocks": [
        {"symbol": "股票代码", "name": "股票名称", "impact_type": "利好/利空/中性", "impact_strength": 0.0-1.0, "confidence": 0.0-1.0}
    ],
    "sectors": ["相关板块1", "相关板块2"],
    "strength": 0.0-1.0,
    "reason": "简短分析原因"
}"#)
            },
            MarketType::US => {
                (r#"你是一个客观、专业的美股市场分析师。你的分析必须基于事实，避免过度乐观或过度悲观。

分析原则：
1. 区分"实质性利好/利空"和"模糊的正面/负面"
2. 考虑宏观经济环境、利率预期、地缘政治风险
3. 注意科技股走势对大盘的整体影响

股票代码格式说明：
- 美股代码使用英文字母，如AAPL表示苹果，GOOGL表示谷歌，TSLA表示特斯拉，NVDA表示英伟达
- 常见科技股：META(Meta)、GOOGL(谷歌)、AMZN(亚马逊)、MSFT(微软)、NFLX(奈飞)
- 指数：^GSPC(标普500)、^DJI(道琼斯)、^IXIC(纳斯达克)

输出要求：
- 只返回JSON，不要有其他内容
- 如果新闻没有明显影响的相关股票，返回空数组
- 置信度体现分析确定性，0-1之间"#,
                r#"分析以下新闻对美股市场的影响：

标题：{}
内容：{}
时间：{}
来源：{}

请返回JSON格式：
{
    "impact_type": "利好/利空/中性",
    "impact_stocks": [
        {"symbol": "股票代码", "name": "股票名称", "impact_type": "利好/利空/中性", "impact_strength": 0.0-1.0, "confidence": 0.0-1.0}
    ],
    "sectors": ["相关板块1", "相关板块2"],
    "strength": 0.0-1.0,
    "reason": "简短分析原因"
}"#)
            }
        };

        let user_prompt = format!(r#"分析以下新闻对美股市场的影响：

标题：{}
内容：{}
时间：{}
来源：{}

请返回JSON格式：
{{
    "impact_type": "利好/利空/中性",
    "impact_stocks": [
        {{"symbol": "股票代码", "name": "股票名称", "impact_type": "利好/利空/中性", "impact_strength": 0.0-1.0, "confidence": 0.0-1.0}}
    ],
    "sectors": ["相关板块1", "相关板块2"],
    "strength": 0.0-1.0,
    "reason": "简短分析原因"
}}"#, news.title, news.content, news.pub_time, news.source);

        let response = self.call_api(system_prompt, &user_prompt).await?;
        self.parse_analysis(&response, news)
    }

    async fn call_api(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let url = "https://api.deepseek.com/chat/completions";

        #[derive(Serialize)]
        struct Request {
            model: String,
            messages: Vec<Message>,
            temperature: f32,
        }

        #[derive(Serialize)]
        struct Message {
            role: String,
            content: String,
        }

        let request = Request {
            model: "deepseek-chat".to_string(),
            messages: vec![
                Message { role: "system".to_string(), content: system_prompt.to_string() },
                Message { role: "user".to_string(), content: user_prompt.to_string() },
            ],
            temperature: 0.3,
        };

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("DeepSeek API request failed: {}", e))?;

        let json: serde_json::Value = response.json().await
            .map_err(|e| anyhow::anyhow!("DeepSeek API response parse failed: {}", e))?;

        if let Some(error) = json.get("error") {
            return Err(anyhow::anyhow!("DeepSeek API error: {}", error));
        }

        let content = json.get("choices")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| anyhow::anyhow!("Failed to parse DeepSeek response"))?
            .to_string();

        Ok(content)
    }

    fn parse_analysis(&self, response: &str, news: &StockNews) -> Result<NewsAnalysis> {
        // 尝试提取JSON（可能包含在markdown代码块中）
        let json_str = if response.contains("```json") {
            response.split("```json").nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
        } else if response.contains("```") {
            response.split("```").nth(1)
                .unwrap_or(response)
        } else {
            response
        }.trim();

        #[derive(Deserialize)]
        struct RawAnalysis {
            impact_type: String,
            impact_stocks: Vec<RawImpactedStock>,
            sectors: Vec<String>,
            strength: f64,
            reason: String,
        }

        #[derive(Deserialize)]
        struct RawImpactedStock {
            symbol: String,
            name: String,
            impact_type: String,
            impact_strength: f64,
            confidence: f64,
        }

        let raw: RawAnalysis = serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse analysis JSON: {} - raw: {}", e, json_str))?;

        let impact_type = match raw.impact_type.as_str() {
            "利好" => ImpactType::Positive,
            "利空" => ImpactType::Negative,
            _ => ImpactType::Neutral,
        };

        let impact_stocks: Vec<ImpactedStock> = raw.impact_stocks.into_iter().map(|s| {
            ImpactedStock {
                symbol: s.symbol,
                name: s.name,
                impact_type: match s.impact_type.as_str() {
                    "利好" => ImpactType::Positive,
                    "利空" => ImpactType::Negative,
                    _ => ImpactType::Neutral,
                },
                impact_strength: s.impact_strength,
                confidence: s.confidence,
            }
        }).collect();

        Ok(NewsAnalysis {
            news: news.clone(),
            impact_type,
            impact_stocks,
            sectors: raw.sectors,
            strength: raw.strength,
            reason: raw.reason,
        })
    }
}

/// 重要新闻判断
fn is_important_news(news: &StockNews) -> bool {
    let important_keywords_title = [
        "涨停", "跌停", "利好", "利空", "政策", "重大", "首板", "连板",
        "特停", "问询", "监管", "罚款", "处罚", "立案", "调查",
        "业绩", "预增", "预亏", "扭亏", "大增", "大降", "财报", "营收",
        "订单", "中标", "签约", "合作", "重组", "并购", "收购",
        "增持", "减持", "回购", "分红", "AI", "人工智能", "大模型",
        "涨停", "跌停", "制裁", "出口", "进口", "关税", "核", "战",
        "总统", "谈判", "协议", "危机", "暴涨", "暴跌", "突破",
    ];

    let important_keywords_content = [
        "涨停", "跌停", "利好", "利空", "政策", "重大", "首板", "连板",
        "特停", "问询", "监管", "罚款", "处罚", "立案", "调查",
        "业绩", "预增", "预亏", "扭亏", "大增", "大降", "财报", "营收",
        "订单", "中标", "签约", "合作", "重组", "并购", "收购",
        "增持", "减持", "回购", "分红", "AI", "人工智能", "大模型",
        "制裁", "出口", "进口", "关税", "核", "战", "总统", "谈判",
        "协议", "危机", "暴涨", "暴跌", "突破", "上涨", "下跌",
    ];

    let title_lower = news.title.to_lowercase();
    let content_lower = news.content.to_lowercase();

    for kw in &important_keywords_title {
        if title_lower.contains(kw) {
            return true;
        }
    }

    // 检查内容中是否有重要关键词
    for kw in &important_keywords_content {
        if content_lower.contains(kw) {
            return true;
        }
    }

    false
}

/// 新闻影响分析服务
pub struct NewsImpactService {
    cls_client: crate::data::cls_client::ClsClient,
    eastmoney_client: reqwest::Client,
    deepseek_client: Option<DeepSeekClient>,
}

impl NewsImpactService {
    pub fn new() -> Self {
        Self {
            cls_client: crate::data::cls_client::ClsClient::new(),
            eastmoney_client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            deepseek_client: DeepSeekClient::new().ok(),
        }
    }

    /// 获取东方财富新闻（支持A股、港股、美股分类）
    async fn get_eastmoney_news(&self, page: u32, page_size: u32) -> Result<Vec<(StockNews, MarketType)>> {
        let url = format!(
            "https://newsapi.eastmoney.com/kuaixun/v1/getlist_101_ajaxResult_{}_{}_.html",
            page_size.min(50) as usize, page
        );

        let text = self.eastmoney_client
            .get(&url)
            .header("Referer", "https://finance.eastmoney.com/")
            .header("Accept", "*/*")
            .send()
            .await?
            .text()
            .await?;

        // 解析JSON（东方财富返回的是var格式）
        let json_str = if let Some(eq_pos) = text.find("=") {
            let after_eq = text[eq_pos + 1..].trim();
            if after_eq.starts_with('{') || after_eq.starts_with('[') {
                after_eq.trim()
            } else {
                &text
            }
        } else {
            &text
        };

        #[derive(Deserialize)]
        struct EmResponse {
            rc: i32,
            #[serde(rename = "LivesList")]
            lives_list: Option<Vec<EmItem>>,
        }

        #[derive(Deserialize)]
        struct EmItem {
            id: String,
            title: String,
            digest: Option<String>,
            showtime: String,
            url_w: Option<String>,
            column: Option<String>,
        }

        let response: EmResponse = serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse EastMoney news: {}", e))?;

        let mut result = Vec::new();

        if response.rc == 1 {
            if let Some(lives) = response.lives_list {
                for item in lives {
                    let market = match item.column.as_deref() {
                        Some(c) if c.contains("100") => MarketType::AShare,
                        Some(c) if c.contains("102") => MarketType::HK,
                        Some(c) if c.contains("104") => MarketType::US,
                        _ => continue, // 跳过其他分类
                    };

                    let news = StockNews {
                        id: item.id,
                        title: item.title,
                        content: item.digest.unwrap_or_default(),
                        pub_time: item.showtime,
                        source: "东方财富".to_string(),
                        url: item.url_w.unwrap_or_default(),
                        category: market.display_name().to_string(),
                    };

                    result.push((news, market));
                }
            }
        }

        Ok(result)
    }

    /// 获取新闻并分析
    pub async fn analyze_latest_news(&self, count: usize) -> Result<NewsImpactResult> {
        // 1. 获取财联社新闻（A股为主）
        let cls_news = self.cls_client.get_telegraphs(1, (count / 2) as u32).await?;
        let cls_news: Vec<_> = cls_news.into_iter()
            .map(|n| (n, MarketType::AShare)) // 财联社主要是A股
            .collect();

        // 2. 获取东方财富新闻（包含港股、美股分类）
        let em_news = self.get_eastmoney_news(1, (count / 2) as u32).await?;

        // 3. 合并新闻
        let all_news: Vec<_> = cls_news.into_iter()
            .chain(em_news)
            .collect();

        // 按市场分组
        let mut news_by_market: std::collections::HashMap<String, Vec<NewsAnalysis>> = std::collections::HashMap::new();
        news_by_market.insert("A股".to_string(), Vec::new());
        news_by_market.insert("港股".to_string(), Vec::new());
        news_by_market.insert("美股".to_string(), Vec::new());

        let mut all_analyses = Vec::new();

        // 4. 对重要新闻调用DeepSeek分析
        for (news, market) in &all_news {
            if is_important_news(news) {
                if let Some(ref client) = self.deepseek_client {
                    match client.analyze_news_impact(news, market).await {
                        Ok(analysis) => {
                            let market_key = market.display_name().to_string();
                            news_by_market.entry(market_key).or_insert_with(Vec::new).push(analysis.clone());
                            all_analyses.push(analysis);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to analyze news '{}': {}", news.title, e);
                            let basic = self.basic_analysis(news);
                            let market_key = market.display_name().to_string();
                            news_by_market.entry(market_key).or_insert_with(Vec::new).push(basic.clone());
                            all_analyses.push(basic);
                        }
                    }
                } else {
                    let basic = self.basic_analysis(news);
                    let market_key = market.display_name().to_string();
                    news_by_market.entry(market_key).or_insert_with(Vec::new).push(basic.clone());
                    all_analyses.push(basic);
                }
            }
        }

        // 计算股票评分
        let top_stocks = self.calculate_stock_scores(&all_analyses);

        // 计算板块影响
        let sector_impacts = self.calculate_sector_impacts(&all_analyses);

        // 判断市场情绪
        let market_sentiment = self.calculate_market_sentiment(&all_analyses);

        // 计算跨市场情绪
        let cross_market_sentiment = self.calculate_cross_market_sentiment(&news_by_market);

        Ok(NewsImpactResult {
            news_list: all_analyses,
            top_stocks,
            sector_impacts,
            market_sentiment,
            timestamp: chrono::Utc::now(),
            market_news: news_by_market,
            cross_market_sentiment,
        })
    }

    /// 基础分析（不调用AI）
    fn basic_analysis(&self, news: &StockNews) -> NewsAnalysis {
        let text = format!("{} {}", news.title, news.content).to_lowercase();

        let (impact_type, strength) = if text.contains("利好") || text.contains("上涨") || text.contains("增长") {
            (ImpactType::Positive, 0.6)
        } else if text.contains("利空") || text.contains("下跌") || text.contains("亏损") {
            (ImpactType::Negative, 0.6)
        } else {
            (ImpactType::Neutral, 0.3)
        };

        NewsAnalysis {
            news: news.clone(),
            impact_type,
            impact_stocks: Vec::new(),
            sectors: Vec::new(),
            strength,
            reason: "基础关键词分析".to_string(),
        }
    }

    /// 计算股票综合评分
    fn calculate_stock_scores(&self, analyses: &[NewsAnalysis]) -> Vec<ImpactedStock> {
        use std::collections::HashMap;

        let mut stock_scores: HashMap<String, (String, f64, f64, i32)> = HashMap::new(); // symbol -> (name, score, strength_sum, count)

        for analysis in analyses {
            for stock in &analysis.impact_stocks {
                let entry = stock_scores.entry(stock.symbol.clone()).or_insert_with(|| {
                    (stock.name.clone(), 0.0, 0.0, 0)
                });

                // 计算评分：利好为正，利空为负
                let score_delta = match stock.impact_type {
                    ImpactType::Positive => stock.impact_strength * stock.confidence * 10.0,
                    ImpactType::Negative => -stock.impact_strength * stock.confidence * 10.0,
                    ImpactType::Neutral => 0.0,
                };

                entry.1 += score_delta;
                entry.2 += stock.impact_strength;
                entry.3 += 1;
            }
        }

        // 转换为Vec并排序
        let mut stocks: Vec<_> = stock_scores.into_iter().map(|(symbol, (name, score, strength_sum, count))| {
            let avg_strength = if count > 0 { strength_sum / count as f64 } else { 0.0 };
            let final_score = score + avg_strength * 5.0; // 综合评分
            ImpactedStock {
                symbol,
                name,
                impact_type: if final_score > 0.0 { ImpactType::Positive }
                    else if final_score < 0.0 { ImpactType::Negative }
                    else { ImpactType::Neutral },
                impact_strength: final_score.abs() / 20.0, // 归一化到0-1
                confidence: 0.7,
            }
        }).collect();

        stocks.sort_by(|a, b| {
            b.impact_strength.partial_cmp(&a.impact_strength).unwrap_or(std::cmp::Ordering::Equal)
        });

        stocks.truncate(20); // 只返回前20个
        stocks
    }

    /// 计算板块影响评分
    fn calculate_sector_impacts(&self, analyses: &[NewsAnalysis]) -> std::collections::HashMap<String, f64> {
        use std::collections::HashMap;

        let mut sector_scores: HashMap<String, f64> = HashMap::new();

        for analysis in analyses {
            let weight = match analysis.impact_type {
                ImpactType::Positive => analysis.strength,
                ImpactType::Negative => -analysis.strength,
                ImpactType::Neutral => 0.0,
            };

            for sector in &analysis.sectors {
                *sector_scores.entry(sector.clone()).or_insert(0.0) += weight;
            }
        }

        // 归一化到 -1 到 1
        let max_score: f64 = sector_scores.values().map(|v| v.abs()).fold(1.0f64, |a, b| a.max(b));
        for score in sector_scores.values_mut() {
            *score /= max_score;
        }

        sector_scores
    }

    /// 计算市场情绪
    fn calculate_market_sentiment(&self, analyses: &[NewsAnalysis]) -> String {
        let positive = analyses.iter().filter(|a| a.impact_type == ImpactType::Positive).count();
        let negative = analyses.iter().filter(|a| a.impact_type == ImpactType::Negative).count();
        let total = analyses.len();

        if total == 0 {
            return "中性".to_string();
        }

        let ratio = (positive as f64 - negative as f64) / total as f64;

        if ratio > 0.3 {
            "偏多".to_string()
        } else if ratio < -0.3 {
            "偏空".to_string()
        } else {
            "中性".to_string()
        }
    }

    /// 计算跨市场情绪
    fn calculate_cross_market_sentiment(&self, news_by_market: &std::collections::HashMap<String, Vec<NewsAnalysis>>) -> String {
        let mut parts = Vec::new();

        for (market, analyses) in news_by_market {
            if analyses.is_empty() {
                continue;
            }

            let sentiment = self.calculate_market_sentiment(analyses);
            parts.push(format!("{}:{}", market, sentiment));
        }

        if parts.is_empty() {
            return "暂无数据".to_string();
        }

        parts.join(", ")
    }
}

impl Default for NewsImpactService {
    fn default() -> Self {
        Self::new()
    }
}
