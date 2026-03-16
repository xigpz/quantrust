use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScreenerDefinition {
    pub name: Option<String>,
    pub description: Option<String>,
    pub logic: ScreenerGroup,
    pub sorts: Vec<ScreenerSort>,
    pub columns: Vec<String>,
    pub source: Option<ScreenerSource>,
    #[serde(rename = "importMeta")]
    pub import_meta: Option<ScreenerImportMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScreenerGroup {
    pub id: String,
    pub operator: ScreenerLogic,
    pub children: Vec<ScreenerNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ScreenerNode {
    Group(ScreenerGroup),
    Condition(ScreenerCondition),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScreenerCondition {
    pub id: String,
    pub field: String,
    pub operator: ScreenerOperator,
    pub value: ScreenerValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScreenerSort {
    pub field: String,
    pub direction: ScreenerSortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScreenerImportMeta {
    #[serde(rename = "originalUrl")]
    pub original_url: Option<String>,
    #[serde(rename = "importedConditions")]
    pub imported_conditions: usize,
    #[serde(rename = "unsupportedConditions")]
    pub unsupported_conditions: Vec<ImportedConditionWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportedConditionWarning {
    pub key: String,
    pub reason: String,
    pub raw: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScreenerLogic {
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScreenerOperator {
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = ">=")]
    GreaterThanOrEqual,
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = "<=")]
    LessThanOrEqual,
    #[serde(rename = "=")]
    Equal,
    #[serde(rename = "between")]
    Between,
    #[serde(rename = "in")]
    In,
    #[serde(rename = "contains")]
    Contains,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScreenerSource {
    Manual,
    EastmoneyImport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScreenerSortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ScreenerValue {
    Number(f64),
    Text(String),
    Boolean(bool),
    StringList(Vec<String>),
    Range { min: f64, max: f64 },
    NumericList(Vec<f64>),
}

impl<'de> Deserialize<'de> for ScreenerValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::Number(number) => number
                .as_f64()
                .map(ScreenerValue::Number)
                .ok_or_else(|| serde::de::Error::custom("invalid numeric value")),
            serde_json::Value::String(text) => Ok(ScreenerValue::Text(text)),
            serde_json::Value::Bool(flag) => Ok(ScreenerValue::Boolean(flag)),
            serde_json::Value::Array(values) if values.len() == 2 && values.iter().all(|v| v.is_number()) => {
                Ok(ScreenerValue::Range {
                    min: values[0].as_f64().unwrap_or_default(),
                    max: values[1].as_f64().unwrap_or_default(),
                })
            }
            serde_json::Value::Array(values) if values.iter().all(|v| v.is_string()) => Ok(
                ScreenerValue::StringList(
                    values
                        .into_iter()
                        .filter_map(|value| value.as_str().map(str::to_string))
                        .collect(),
                ),
            ),
            serde_json::Value::Array(values) if values.iter().all(|v| v.is_number()) => Ok(
                ScreenerValue::NumericList(
                    values
                        .into_iter()
                        .filter_map(|value| value.as_f64())
                        .collect(),
                ),
            ),
            _ => Err(serde::de::Error::custom("unsupported screener value")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScreenerCatalogField {
    pub field: String,
    pub label: String,
    pub category: String,
    #[serde(rename = "valueType")]
    pub value_type: ScreenerFieldValueType,
    pub operators: Vec<ScreenerOperator>,
    #[serde(rename = "dataSource")]
    pub data_source: String,
    #[serde(rename = "eastmoneyCompatible")]
    pub eastmoney_compatible: bool,
    pub status: ScreenerFieldStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScreenerFieldValueType {
    Number,
    Range,
    Enum,
    Boolean,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScreenerFieldStatus {
    Ready,
    Derived,
    Unavailable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn screener_definition_preserves_nested_groups_and_import_warnings() {
        let payload = json!({
            "name": "Momentum candidates",
            "description": "Imported from EastMoney",
            "logic": {
                "id": "root",
                "operator": "AND",
                "children": [
                    {
                        "id": "price-range",
                        "field": "latest_price",
                        "operator": "between",
                        "value": [10.5, 25.0]
                    },
                    {
                        "id": "sub-group",
                        "operator": "OR",
                        "children": [
                            {
                                "id": "pct-up",
                                "field": "change_pct",
                                "operator": ">=",
                                "value": 3.2
                            },
                            {
                                "id": "turnover",
                                "field": "turnover_rate",
                                "operator": ">=",
                                "value": 5.0
                            }
                        ]
                    }
                ]
            },
            "sorts": [
                { "field": "change_pct", "direction": "desc" }
            ],
            "columns": ["symbol", "name", "latest_price", "change_pct"],
            "source": "eastmoney_import",
            "importMeta": {
                "originalUrl": "https://xuangu.eastmoney.com/example",
                "importedConditions": 3,
                "unsupportedConditions": [
                    {
                        "key": "fancy_metric",
                        "reason": "unsupported field",
                        "raw": "fancy_metric>1"
                    }
                ]
            }
        });

        let definition: ScreenerDefinition = serde_json::from_value(payload).unwrap();

        assert_eq!(definition.name.as_deref(), Some("Momentum candidates"));
        assert_eq!(definition.sorts.len(), 1);
        assert_eq!(definition.columns.len(), 4);
        assert_eq!(definition.import_meta.as_ref().unwrap().imported_conditions, 3);
        assert_eq!(
            definition
                .import_meta
                .as_ref()
                .unwrap()
                .unsupported_conditions
                .len(),
            1
        );

        match &definition.logic.children[1] {
            ScreenerNode::Group(group) => {
                assert_eq!(group.operator, ScreenerLogic::Or);
                assert_eq!(group.children.len(), 2);
            }
            ScreenerNode::Condition(_) => panic!("expected nested group"),
        }

        match &definition.logic.children[0] {
            ScreenerNode::Condition(condition) => {
                assert_eq!(condition.field, "latest_price");
                assert_eq!(condition.operator, ScreenerOperator::Between);
                assert_eq!(
                    condition.value,
                    ScreenerValue::Range {
                        min: 10.5,
                        max: 25.0
                    }
                );
            }
            ScreenerNode::Group(_) => panic!("expected condition"),
        }
    }
}
