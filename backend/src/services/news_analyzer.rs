use serde::{Deserialize, Serialize};

/// 情感分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    pub score: f64,
    pub label: String,
    pub keywords: Vec<String>,
}

/// 异动预测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyPrediction {
    pub symbol: String,
    pub name: String,
    pub change_pct: f64,
    pub pred_type: String,
    pub sentiment: Sentiment,
    pub urgency: String,
    pub timestamp: String,
    pub reason: String,
}

/// 新闻分析器
pub struct NewsAnalyzer;

impl NewsAnalyzer {
    /// 分析新闻情感
    pub fn analyze(title: &str, content: &str) -> Sentiment {
        let text = format!("{} {}", title, content).to_lowercase();
        
        let pos: Vec<&str> = vec!["利好","上涨","增长","盈利","突破","业绩预增","订单","中标","合作","回购","增持","买入","推荐","特大利好","暴增"];
        let neg: Vec<&str> = vec!["利空","下跌","亏损","风险","调查","处罚","减持","业绩预亏","跌停","暴跌","特大利空","暴亏"];
        
        let mut pc = 0; 
        let mut nc = 0; 
        let mut kw: Vec<String> = vec![];
        
        for w in &pos { 
            if text.contains(&w.to_lowercase()) { 
                pc += 1; 
                kw.push(w.to_string()); 
            } 
        }
        for w in &neg { 
            if text.contains(&w.to_lowercase()) { 
                nc += 1; 
                kw.push(w.to_string()); 
            } 
        }
        
        let tot = pc + nc;
        let sc = if tot > 0 { (pc as f64 - nc as f64) / (tot as f64) } else { 0.0 };
        
        let (lab, conf) = if tot == 0 {
            ("中性".to_string(), 0.3)
        } else if sc > 0.3 {
            ("利好".to_string(), 0.6_f64.min(0.4 + pc as f64 / 10.0))
        } else if sc < -0.3 {
            ("利空".to_string(), 0.6_f64.min(0.4 + nc as f64 / 10.0))
        } else {
            ("中性".to_string(), 0.4)
        };
        
        Sentiment { score: sc, label: lab, keywords: kw }
    }
    
    /// 从关键词预测异动类型
    pub fn predict_type(title: &str) -> Option<(String, Vec<String>)> {
        let t = title.to_lowercase();
        
        if t.contains("业绩") || t.contains("预增") || t.contains("暴增") || t.contains("扭亏") {
            return Some(("业绩预增".to_string(), vec!["年报".to_string(), "业绩".to_string()]));
        }
        if t.contains("订单") || t.contains("中标") || t.contains("签约") || t.contains("合作") {
            return Some(("订单利好".to_string(), vec!["订单".to_string()]));
        }
        if t.contains("回购") || t.contains("增持") {
            return Some(("回购增持".to_string(), vec!["股权激励".to_string()]));
        }
        if t.contains("政策") || t.contains("支持") || t.contains("补贴") {
            return Some(("政策利好".to_string(), vec!["政策".to_string()]));
        }
        if t.contains("AI") || t.contains("人工智能") || t.contains("大模型") {
            return Some(("AI利好".to_string(), vec!["科技".to_string()]));
        }
        if t.contains("减持") || t.contains("调查") || t.contains("处罚") {
            return Some(("风险警示".to_string(), vec!["利空".to_string()]));
        }
        if t.contains("重组") || t.contains("并购") || t.contains("借壳") {
            return Some(("并购重组".to_string(), vec!["重组".to_string()]));
        }
        
        None
    }
}
