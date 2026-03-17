use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::models::{StockNotice, StockNoticesResponse, StockNoticeDetail};

/// 解析JSONP
fn strip_jsonp(text: &str) -> &str {
    if let Some(start) = text.find('(') {
        if let Some(end) = text.rfind(')') {
            if end > start {
                return &text[start + 1..end];
            }
        }
    }
    text
}

/// 获取个股新闻公告列表
pub async fn get_stock_notices(stock_code: &str, page_index: u32, page_size: u32) -> Result<StockNoticesResponse> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let callback = format!("jQuery11230617983012953938_{}", timestamp);

    let url = format!(
        "https://np-anotice-stock.eastmoney.com/api/security/ann?\
        cb={}&sr=-1&page_size={}&page_index={}&ann_type=A&client_source=web&stock_list={}&f_node=0&s_node=0",
        callback, page_size, page_index, stock_code
    );

    let text = client.get(&url)
        .header("Referer", "https://data.eastmoney.com/notices/stock/001309.html")
        .header("Accept", "*/*")
        .send()
        .await?
        .text()
        .await?;

    let json_str = strip_jsonp(&text);
    let value: serde_json::Value = serde_json::from_str(json_str)?;

    let data = value.get("data").ok_or_else(|| anyhow::anyhow!("No data field"))?;
    let total_hits = data.get("total_hits").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let page_index = data.get("page_index").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
    let page_size = data.get("page_size").and_then(|v| v.as_i64()).unwrap_or(50) as i32;

    let mut notices = Vec::new();
    if let Some(list) = data.get("list").and_then(|v| v.as_array()) {
        for item in list {
            let title = item.get("title_ch").or_else(|| item.get("title")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let notice_date = item.get("notice_date").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let display_time = item.get("display_time").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let art_code = item.get("art_code").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let source_type = item.get("source_type").and_then(|v| v.as_str()).unwrap_or("").to_string();

            let column_name = if let Some(columns) = item.get("columns").and_then(|v| v.as_array()) {
                columns.iter()
                    .filter_map(|c| c.get("column_name").and_then(|v| v.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                String::new()
            };

            notices.push(StockNotice {
                art_code,
                title,
                notice_date,
                display_time,
                column_name,
                source_type,
            });
        }
    }

    Ok(StockNoticesResponse {
        list: notices,
        total_hits,
        page_index,
        page_size,
    })
}

/// 获取公告详情
pub async fn get_notice_detail(art_code: &str) -> Result<StockNoticeDetail> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // 尝试多个 API 端点
    let urls = vec![
        format!("https://np-anotice-stock.eastmoney.com/api/security/ann/detail?art_code={}&_={}", art_code, timestamp),
        format!("https://datacenter.eastmoney.com/api/data/v1/get?reportName=RPT_BONDPAPER_DETAILS&columns=ALL&filter=(ART_CODE%3D%27{}%27)&pageNumber=1&pageSize=1&source=WEB", art_code),
    ];

    for url in &urls {
        let text = match client.get(url)
            .header("Referer", "https://data.eastmoney.com/")
            .header("Accept", "application/json, text/plain, */*")
            .send()
            .await {
                Ok(resp) => resp.text().await.unwrap_or_default(),
                Err(_) => continue,
            };

        if text.is_empty() {
            continue;
        }

        // 尝试解析 JSON
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
            // 尝试从各种可能的响应结构中提取数据
            let data = value.get("data")
                .or_else(|| value.get("result"))
                .or_else(|| value.get("list"))
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .cloned()
                .unwrap_or(value.clone());

            if let Some(title) = data.get("title_ch").or_else(|| data.get("title")).and_then(|v| v.as_str()) {
                let content = data.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let notice_date = data.get("notice_date").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let display_time = data.get("display_time").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let source = data.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let column_name = data.get("columns")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|c| c.get("column_name").and_then(|v| v.as_str())).collect::<Vec<_>>().join(", "))
                    .unwrap_or_default();

                return Ok(StockNoticeDetail {
                    title: title.to_string(),
                    content,
                    notice_date: notice_date.to_string(),
                    display_time: display_time.to_string(),
                    source: source.to_string(),
                    column_name,
                });
            }
        }
    }

    // 如果所有 API 都失败，返回一个提示信息
    Ok(StockNoticeDetail {
        title: "公告详情".to_string(),
        content: format!("无法获取公告详情 (art_code: {}). 该公告可能已被删除或不存在。", art_code),
        notice_date: "".to_string(),
        display_time: "".to_string(),
        source: "".to_string(),
        column_name: "".to_string(),
    })
}
