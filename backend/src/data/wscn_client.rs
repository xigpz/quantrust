use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::models::StockNews;

/// 华尔街见闻快讯项
#[derive(Debug, Clone, Deserialize)]
struct WscnLiveItem {
    #[serde(rename = "id")]
    id: i64,
    #[serde(rename = "title")]
    title: Option<String>,
    #[serde(rename = "content_text")]
    content_text: Option<String>,
    #[serde(rename = "display_time")]
    display_time: i64,
    #[serde(rename = "author")]
    author: Option<WscnAuthor>,
    #[serde(rename = "channels")]
    channels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
struct WscnAuthor {
    #[serde(rename = "display_name")]
    display_name: Option<String>,
}

/// 华尔街见闻API响应
#[derive(Debug, Deserialize)]
struct WscnResponse {
    #[serde(rename = "code")]
    code: i32,
    #[serde(rename = "message")]
    message: String,
    #[serde(rename = "data")]
    data: Option<WscnData>,
}

#[derive(Debug, Deserialize)]
struct WscnData {
    #[serde(rename = "items")]
    items: Option<Vec<WscnLiveItem>>,
    #[serde(rename = "next_cursor")]
    next_cursor: Option<String>,
}

/// 华尔街见闻新闻客户端
pub struct WscnClient {
    client: Client,
}

impl WscnClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    /// 获取全球快讯
    pub async fn get_global_lives(&self, limit: u32) -> Result<Vec<StockNews>> {
        let url = format!(
            "https://api-prod.wallstreetcn.com/apiv1/content/lives?channel=global-channel&client=pc&limit={}",
            limit
        );

        let resp = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await?;

        let text = resp.text().await?;
        let json: WscnResponse = serde_json::from_str(&text)?;

        let items = json.data.and_then(|d| d.items).unwrap_or_default();

        let news = items.into_iter().map(|item| {
            // 格式化时间
            let datetime = chrono::DateTime::from_timestamp(item.display_time, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default();

            // 获取频道作为分类
            let category = item.channels
                .as_ref()
                .and_then(|c| c.first())
                .map(|c| c.replace("-channel", "").replace("global", "全球"))
                .unwrap_or_else(|| "全球".to_string());

            // 作者名作为来源
            let source = item.author
                .as_ref()
                .and_then(|a| a.display_name.clone())
                .unwrap_or_else(|| "华尔街见闻".to_string());

            StockNews {
                id: item.id.to_string(),
                title: item.title.unwrap_or_default(),
                content: item.content_text.unwrap_or_default(),
                pub_time: datetime,
                source,
                url: format!("https://wallstreetcn.com/live"),
                category,
            }
        }).collect();

        Ok(news)
    }
}

impl Default for WscnClient {
    fn default() -> Self {
        Self::new()
    }
}
