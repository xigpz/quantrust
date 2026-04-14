use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 主题定义
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeAnalysis {
    pub theme: String,                    // 主题名称
    pub logic: Vec<ThemeLogic>,          // 利好逻辑链
    pub sectors: Vec<ThemeSector>,       // 受益板块
    pub stocks: Vec<ThemeStock>,         // 受益个股
    pub ai_insight: Option<String>,       // AI分析洞察
}

/// 利好逻辑
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeLogic {
    pub cause: String,        // 原因
    pub effect: String,       // 结果
    pub mechanism: String,    // 传导机制
}

/// 受益板块
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeSector {
    pub code: String,
    pub name: String,
    pub relevance: f64,       // 相关度 0-1
    pub change_pct: f64,     // 今日涨跌幅
    pub money_flow: f64,      // 成交额
}

/// 受益个股
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeStock {
    pub symbol: String,
    pub name: String,
    pub change_pct: f64,
    pub main_net_inflow: f64,
    pub reason: String,
}

/// 主题映射规则库 - 使用板块名称关键词匹配
pub struct ThemeMapping {
    /// 主题到利好逻辑的映射
    theme_logics: HashMap<String, Vec<ThemeLogic>>,
    /// 主题到板块名称关键词的映射
    theme_sector_keywords: HashMap<String, Vec<&'static str>>,
}

impl ThemeMapping {
    pub fn new() -> Self {
        let mut theme_logics = HashMap::new();
        let mut theme_sector_keywords = HashMap::new();

        // ============ 石油战争主题 ============
        theme_logics.insert("石油战争".to_string(), vec![
            ThemeLogic {
                cause: "地缘冲突导致原油供应紧张".to_string(),
                effect: "原油价格上涨".to_string(),
                mechanism: "中东/俄乌冲突 → 制裁+运输受阻 → 原油供需缺口 → 油价上涨".to_string(),
            },
            ThemeLogic {
                cause: "能源安全关注度提升".to_string(),
                effect: "替代能源需求增加".to_string(),
                mechanism: "传统能源风险 → 新能源、储能需求上升".to_string(),
            },
            ThemeLogic {
                cause: "化工原料成本上升".to_string(),
                effect: "化工品价格上涨".to_string(),
                mechanism: "原油→石脑油→乙烯/丙烯→化工品".to_string(),
            },
        ]);
        theme_sector_keywords.insert("石油战争".to_string(), vec![
            "石油", "油气", "能源", "原油", "采油", "油田", "化工", "炼油", "油服",
        ]);

        // ============ 原材料短缺主题 ============
        theme_logics.insert("原材料短缺".to_string(), vec![
            ThemeLogic {
                cause: "供应链中断".to_string(),
                effect: "原材料价格上涨".to_string(),
                mechanism: "突发事件 → 供应短缺 → 价格弹性 → 价格上涨".to_string(),
            },
            ThemeLogic {
                cause: "国产替代加速".to_string(),
                effect: "国内厂商受益".to_string(),
                mechanism: "外部供应风险 → 国产替代 → 国内企业订单增加".to_string(),
            },
        ]);
        theme_sector_keywords.insert("原材料短缺".to_string(), vec![
            "煤炭", "有色", "金属", "钢铁", "稀土", "矿产", "铜", "铝", "锂", "钴",
        ]);

        // ============ 粮食危机主题 ============
        theme_logics.insert("粮食危机".to_string(), vec![
            ThemeLogic {
                cause: "气候异常导致减产".to_string(),
                effect: "粮价上涨".to_string(),
                mechanism: "干旱/洪涝 → 粮食减产 → 供需紧张 → 价格上涨".to_string(),
            },
            ThemeLogic {
                cause: "农业投入品需求增加".to_string(),
                effect: "化肥农药受益".to_string(),
                mechanism: "粮价上涨 → 种植积极性 → 化肥农药需求".to_string(),
            },
        ]);
        theme_sector_keywords.insert("粮食危机".to_string(), vec![
            "种业", "化肥", "农药", "农机", "农业", "粮食", "种植", "农副",
        ]);

        // ============ 芯片战争主题 ============
        theme_logics.insert("芯片战争".to_string(), vec![
            ThemeLogic {
                cause: "半导体设备出口管制".to_string(),
                effect: "国产替代加速".to_string(),
                mechanism: "美国制裁 → 设备进口受阻 → 国产设备替代".to_string(),
            },
            ThemeLogic {
                cause: "成熟制程需求旺盛".to_string(),
                effect: "成熟制程代工厂受益".to_string(),
                mechanism: "消费电子需求 + 汽车芯片需求 → 成熟制程产能紧张".to_string(),
            },
        ]);
        theme_sector_keywords.insert("芯片战争".to_string(), vec![
            "半导体", "芯片", "集成电路", "IC", "封测", "晶圆", "光刻", "设备",
        ]);

        // ============ 新能源主题 ============
        theme_logics.insert("新能源".to_string(), vec![
            ThemeLogic {
                cause: "政策支持".to_string(),
                effect: "装机量增加".to_string(),
                mechanism: "碳中和政策 → 新能源补贴 → 装机加速".to_string(),
            },
            ThemeLogic {
                cause: "成本下降".to_string(),
                effect: "平价上网".to_string(),
                mechanism: "技术进步 → 成本下降 → 竞争力提升".to_string(),
            },
        ]);
        theme_sector_keywords.insert("新能源".to_string(), vec![
            "光伏", "风电", "储能", "锂电池", "新能源", "电力设备", "电池",
        ]);

        // ============ 军工主题 ============
        theme_logics.insert("军工".to_string(), vec![
            ThemeLogic {
                cause: "地缘风险上升".to_string(),
                effect: "军费开支增加".to_string(),
                mechanism: "国际局势紧张 → 国防预算 ↑ → 军工订单 ↑".to_string(),
            },
            ThemeLogic {
                cause: "装备升级换代".to_string(),
                effect: "先进装备需求".to_string(),
                mechanism: "现代化战争 → 新型装备 → 军工研发生产".to_string(),
            },
        ]);
        theme_sector_keywords.insert("军工".to_string(), vec![
            "军工", "航天", "航空", "船舶", "兵装", "国防", "雷达", "导航",
        ]);

        // ============ AI主题 ============
        theme_logics.insert("AI".to_string(), vec![
            ThemeLogic {
                cause: "技术突破".to_string(),
                effect: "应用场景拓展".to_string(),
                mechanism: "大模型能力提升 → 各行业AI应用落地".to_string(),
            },
            ThemeLogic {
                cause: "算力需求爆发".to_string(),
                effect: "算力基础设施".to_string(),
                mechanism: "AI训练推理 → 服务器/芯片/光模块需求".to_string(),
            },
        ]);
        theme_sector_keywords.insert("AI".to_string(), vec![
            "AI", "人工智能", "算力", "云计算", "软件", "计算机", "服务器", "数据", "智能", "云", "东数西算",
        ]);

        // ============ 房地产政策松动主题 ============
        theme_logics.insert("房地产".to_string(), vec![
            ThemeLogic {
                cause: "政策支持".to_string(),
                effect: "融资环境改善".to_string(),
                mechanism: "限购松绑 + 融资支持 → 房地产现金流改善".to_string(),
            },
            ThemeLogic {
                cause: "需求复苏".to_string(),
                effect: "竣工增加".to_string(),
                mechanism: "销售回暖 → 竣工加速 → 家居建材需求".to_string(),
            },
        ]);
        theme_sector_keywords.insert("房地产".to_string(), vec![
            "房地产", "建筑", "建材", "家居", "装修", "家电", "物业",
        ]);

        // ============ 医药主题 ============
        theme_logics.insert("医药".to_string(), vec![
            ThemeLogic {
                cause: "人口老龄化".to_string(),
                effect: "医疗需求增加".to_string(),
                mechanism: "老年人口增加 → 医疗支出 ↑ → 医药需求".to_string(),
            },
            ThemeLogic {
                cause: "创新药突破".to_string(),
                effect: "国产替代".to_string(),
                mechanism: "研发能力提升 → 创新药上市 → 进口替代".to_string(),
            },
        ]);
        theme_sector_keywords.insert("医药".to_string(), vec![
            "医药", "医疗", "中药", "化学制药", "生物制药", "医疗器械", "医院", "疫苗",
        ]);

        Self { theme_logics, theme_sector_keywords }
    }

    /// 分析主题
    pub fn analyze(&self, theme: &str) -> Option<(Vec<ThemeLogic>, Vec<&'static str>)> {
        // 精确匹配
        if let Some(logics) = self.theme_logics.get(theme) {
            let keywords = self.theme_sector_keywords.get(theme).cloned().unwrap_or_default();
            return Some((logics.clone(), keywords));
        }

        // 模糊匹配
        let theme_lower = theme.to_lowercase();
        for (key, logics) in &self.theme_logics {
            if theme_lower.contains(&key.to_lowercase()) || key.to_lowercase().contains(&theme_lower) {
                let keywords = self.theme_sector_keywords.get(key).cloned().unwrap_or_default();
                return Some((logics.clone(), keywords));
            }
        }

        None
    }

    /// 获取所有支持的主题
    pub fn get_supported_themes(&self) -> Vec<String> {
        self.theme_logics.keys().cloned().collect()
    }

    /// 检查板块名称是否匹配关键词
    pub fn sector_matches_keywords(sector_name: &str, keywords: &[&str]) -> bool {
        let name_lower = sector_name.to_lowercase();
        for kw in keywords {
            if name_lower.contains(&(*kw).to_lowercase()) {
                return true;
            }
        }
        false
    }
}

impl Default for ThemeMapping {
    fn default() -> Self {
        Self::new()
    }
}
