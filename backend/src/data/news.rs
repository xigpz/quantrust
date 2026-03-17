use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::models::{StockNews, StockNewsResponse};

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

/// 获取财经新闻列表
pub async fn get_stock_news(stock_code: &str, page_index: u32, page_size: u32) -> Result<StockNewsResponse> {
    // 东财没有公开的新闻API，返回空列表
    // 前端会回退到使用公告数据
    Ok(StockNewsResponse {
        list: vec![],
        total: 0,
    })
}

/// 获取新闻详情
pub async fn get_news_detail(news_id: &str) -> Result<StockNews> {
    // 东财没有公开的新闻API，返回提示信息
    Ok(StockNews {
        id: news_id.to_string(),
        title: "财经新闻".to_string(),
        content: "财经新闻详情暂时无法获取。东方财富未提供公开的新闻API，建议前往官网查看详情。".to_string(),
        pub_time: "".to_string(),
        source: "东方财富".to_string(),
        url: "".to_string(),
        category: "财经".to_string(),
    })
}
