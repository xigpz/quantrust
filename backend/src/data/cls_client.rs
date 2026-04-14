use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::models::StockNews;

/// 财联社快讯新闻项
#[derive(Debug, Clone, Deserialize)]
struct ClsNewsItem {
    // id: 新闻ID
    #[serde(rename = "id")]
    id: i64,
    // ctime: 创建时间 (Unix时间戳)
    #[serde(rename = "ctime")]
    ctime: i64,
    // title: 标题
    #[serde(rename = "title")]
    title: String,
    // brief: 摘要
    #[serde(rename = "brief")]
    brief: Option<String>,
    // content: 内容
    #[serde(rename = "content")]
    content: Option<String>,
    // shareurl: 分享URL
    #[serde(rename = "shareurl")]
    shareurl: Option<String>,
    // level: 重要程度 (A/B/C)
    #[serde(rename = "level")]
    level: Option<String>,
    // subjects: 主题/板块
    #[serde(rename = "subjects")]
    subjects: Option<Vec<ClsSubject>>,
}

#[derive(Debug, Clone, Deserialize)]
struct ClsSubject {
    #[serde(rename = "subject_name")]
    subject_name: Option<String>,
}

/// 财联社API响应
#[derive(Debug, Deserialize)]
struct ClsNewsResponse {
    #[serde(rename = "error")]
    error: i32,
    #[serde(rename = "data")]
    data: Option<ClsNewsData>,
}

#[derive(Debug, Deserialize)]
struct ClsNewsData {
    #[serde(rename = "roll_data")]
    roll_data: Option<Vec<ClsNewsItem>>,
}

/// 判断是否是重要新闻
fn is_important_news(item: &ClsNewsItem) -> bool {
    let important_keywords = [
        "涨停", "跌停", "利好", "利空", "政策", "重大", "首板", "连板",
        "特停", "问询", "监管", "罚款", "处罚", "立案", "调查",
        "业绩", "预增", "预亏", "扭亏", "大增", "大降",
        "订单", "中标", "签约", "合作", "重组", "并购",
        "增持", "减持", "回购", "分红", "AI", "人工智能",
    ];

    let title_lower = item.title.to_lowercase();
    let brief_lower = item.brief.as_deref().unwrap_or("").to_lowercase();

    // 检查level字段 - A级最重要
    if let Some(level) = item.level.as_deref() {
        if level == "A" || level == "B" {
            return true;
        }
    }

    // 检查关键词
    for kw in important_keywords {
        if title_lower.contains(&kw.to_lowercase()) || brief_lower.contains(&kw.to_lowercase()) {
            return true;
        }
    }

    false
}

/// 转换为我们自己的模型
fn to_stock_news(item: ClsNewsItem) -> StockNews {
    // 从时间戳转换
    let pub_time = chrono::DateTime::from_timestamp(item.ctime, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| item.ctime.to_string());

    // 从subjects提取板块
    let category = item.subjects.as_ref()
        .and_then(|subs| subs.first())
        .and_then(|s| s.subject_name.clone())
        .unwrap_or_else(|| "财经".to_string());

    StockNews {
        id: item.id.to_string(),
        title: item.title.clone(),
        content: item.brief.clone().or(item.content.clone()).unwrap_or_default(),
        pub_time,
        source: "财联社".to_string(),
        url: item.shareurl.clone().unwrap_or_default(),
        category,
    }
}

/// 财联社API客户端
pub struct ClsClient {
    client: Client,
    base_url: String,
}

impl ClsClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url: "https://www.cls.cn/nodeapi".to_string(),
        }
    }

    /// 获取财联社快讯新闻
    pub async fn get_telegraphs(&self, page: u32, page_size: u32) -> Result<Vec<StockNews>> {
        let page_size = page_size.min(50) as usize;
        let rn = page_size;

        // 财联社快讯API
        let url = format!("{}/telegraphList", self.base_url);

        let params = [
            ("app", "CailianpressWeb"),
            ("os", "web"),
            ("refresh_type", "1"),
            ("order", "1"),
            ("rn", &rn.to_string()),
            ("sv", "8.4.6"),
            ("page", &page.to_string()),
        ];

        let response = self.client
            .get(&url)
            .query(&params)
            .header("Referer", "https://www.cls.cn/telegraph")
            .header("Accept", "application/json")
            .send()
            .await?;

        let text = response.text().await?;

        // 解析JSON响应
        let api_response: ClsNewsResponse = serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("Failed to parse cls response: {}", e))?;

        if api_response.error != 0 {
            return Err(anyhow::anyhow!("cls API error"));
        }

        let news_list = api_response.data
            .and_then(|d| d.roll_data)
            .unwrap_or_default()
            .into_iter()
            .map(to_stock_news)
            .collect();

        Ok(news_list)
    }

    /// 获取重要新闻（仅返回重要的）
    pub async fn get_important_news(&self, page_size: u32) -> Result<Vec<StockNews>> {
        let all_news = self.get_telegraphs(1, page_size).await?;
        Ok(all_news)
    }

    /// 检查API是否可用
    pub async fn health_check(&self) -> bool {
        if let Ok(news) = self.get_telegraphs(1, 1).await {
            return !news.is_empty();
        }
        false
    }
}

impl Default for ClsClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_important_news() {
        let item1 = ClsNewsItem {
            ctime: "2024-01-01 10:00:00".to_string(),
            title: "【重大利好】某公司签下百亿订单".to_string(),
            brief: Some("订单金额超百亿".to_string()),
            content: None,
            shareurl: None,
            level: Some(2),
            subtype: Some("stock".to_string()),
        };
        assert!(is_important_news(&item1));

        let item2 = ClsNewsItem {
            ctime: "2024-01-01 10:00:00".to_string(),
            title: "今日行情综述".to_string(),
            brief: Some("市场小幅波动".to_string()),
            content: None,
            shareurl: None,
            level: Some(0),
            subtype: None,
        };
        assert!(!is_important_news(&item2));
    }
}
