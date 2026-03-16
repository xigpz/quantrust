use crate::models::{
    ScreenerCatalogField, ScreenerCondition, ScreenerDefinition, ScreenerFieldStatus,
    ScreenerFieldValueType, ScreenerGroup, ScreenerImportMeta, ScreenerLogic,
    ScreenerNode, ScreenerOperator, ScreenerSortDirection, ScreenerSource,
    ScreenerValue, StockQuote,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use reqwest::Url;

pub type ScreenerRow = BTreeMap<String, Value>;

#[derive(Debug, Clone)]
pub struct ScreenerService {
    catalog: Vec<ScreenerCatalogField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScreenerValidationError {
    pub condition_id: String,
    pub field: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScreenerExecutionResult {
    pub total_count: usize,
    pub rows: Vec<ScreenerRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScreenerImportError {
    pub code: String,
    pub message: String,
}

impl ScreenerService {
    pub fn new() -> Self {
        Self {
            catalog: vec![
                Self::text_field("symbol", "Symbol", "identity", false, ScreenerFieldStatus::Ready),
                Self::text_field("name", "Name", "identity", false, ScreenerFieldStatus::Ready),
                Self::number_field("latest_price", "Latest Price", "quote", true, ScreenerFieldStatus::Ready),
                Self::number_field("change_pct", "Change %", "quote", true, ScreenerFieldStatus::Ready),
                Self::number_field("volume", "Volume", "quote", true, ScreenerFieldStatus::Ready),
                Self::number_field("turnover_rate", "Turnover Rate", "quote", true, ScreenerFieldStatus::Ready),
                Self::number_field("pe_ratio", "PE Ratio", "valuation", true, ScreenerFieldStatus::Ready),
                Self::number_field("total_market_cap", "Total Market Cap", "valuation", true, ScreenerFieldStatus::Ready),
                Self::number_field("roe", "ROE", "financial", false, ScreenerFieldStatus::Unavailable),
            ],
        }
    }

    pub fn catalog(&self) -> &[ScreenerCatalogField] {
        &self.catalog
    }

    pub fn validate_definition(
        &self,
        definition: &ScreenerDefinition,
    ) -> Result<(), Vec<ScreenerValidationError>> {
        let mut errors = Vec::new();
        self.validate_group(&definition.logic, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn execute(
        &self,
        definition: &ScreenerDefinition,
        quotes: &[StockQuote],
        limit: Option<usize>,
    ) -> Result<ScreenerExecutionResult, Vec<ScreenerValidationError>> {
        self.validate_definition(definition)?;

        let mut matched = quotes
            .iter()
            .filter(|quote| self.matches_group(&definition.logic, quote))
            .cloned()
            .collect::<Vec<_>>();

        self.sort_quotes(&mut matched, definition);

        let total_count = matched.len();
        let rows = matched
            .into_iter()
            .take(limit.unwrap_or(total_count))
            .map(|quote| self.project_quote(&quote, &definition.columns))
            .collect();

        Ok(ScreenerExecutionResult { total_count, rows })
    }

    pub fn import_eastmoney_url(&self, url: &str) -> Result<ScreenerDefinition, ScreenerImportError> {
        let parsed = Url::parse(url).map_err(|_| ScreenerImportError {
            code: "invalid_url".to_string(),
            message: "URL is not valid".to_string(),
        })?;

        let host = parsed.host_str().unwrap_or_default();
        if !host.contains("eastmoney.com") {
            return Err(ScreenerImportError {
                code: "unsupported_host".to_string(),
                message: "Only EastMoney screener URLs are supported".to_string(),
            });
        }

        let mut unsupported = Vec::new();
        let mut children = Vec::new();
        let mut imported_conditions = 0usize;
        let mut condition_index = 0usize;
        let filters = parsed
            .query_pairs()
            .find(|(key, _)| key == "filters" || key == "em_filters")
            .map(|pair| pair.1.into_owned());

        if let Some(filter_string) = filters {
            for raw_filter in filter_string.split(';').filter(|part: &&str| !part.trim().is_empty()) {
                let parts: Vec<&str> = raw_filter.splitn(3, ':').collect();
                if parts.len() != 3 {
                    unsupported.push(Self::warning(raw_filter, "unsupported filter format", raw_filter));
                    continue;
                }

                let field = Self::map_import_field(parts[0]);
                let operator = Self::parse_import_operator(parts[1]);
                let value = Self::parse_import_value(parts[1], parts[2]);

                match (field, operator, value) {
                    (Some(field_name), Some(operator), Some(value)) => {
                        if let Some(catalog_field) = self.catalog.iter().find(|entry| entry.field == field_name) {
                            if catalog_field.status == ScreenerFieldStatus::Unavailable {
                                unsupported.push(Self::warning(parts[0], "field unavailable", raw_filter));
                                continue;
                            }
                            if !catalog_field.operators.contains(&operator) {
                                unsupported.push(Self::warning(parts[0], "operator unsupported", raw_filter));
                                continue;
                            }
                        }

                        condition_index += 1;
                        imported_conditions += 1;
                        children.push(ScreenerNode::Condition(ScreenerCondition {
                            id: format!("imported-{}", condition_index),
                            field: field_name.to_string(),
                            operator,
                            value,
                        }));
                    }
                    _ => unsupported.push(Self::warning(parts[0], "could not map condition", raw_filter)),
                }
            }
        } else if parsed.query_pairs().any(|(key, _)| key == "id") {
            unsupported.push(Self::warning(
                "id",
                "remote strategy id import is not supported yet",
                parsed.as_str(),
            ));
        } else {
            return Err(ScreenerImportError {
                code: "missing_filters".to_string(),
                message: "No importable EastMoney filters were found".to_string(),
            });
        }

        Ok(ScreenerDefinition {
            name: Some("Imported EastMoney Screener".to_string()),
            description: Some("Imported from EastMoney URL".to_string()),
            logic: ScreenerGroup {
                id: "root".to_string(),
                operator: ScreenerLogic::And,
                children,
            },
            sorts: vec![],
            columns: vec![
                "symbol".to_string(),
                "name".to_string(),
                "latest_price".to_string(),
                "change_pct".to_string(),
            ],
            source: Some(ScreenerSource::EastmoneyImport),
            import_meta: Some(ScreenerImportMeta {
                original_url: Some(url.to_string()),
                imported_conditions,
                unsupported_conditions: unsupported,
            }),
        })
    }

    fn validate_group(&self, group: &ScreenerGroup, errors: &mut Vec<ScreenerValidationError>) {
        for child in &group.children {
            match child {
                ScreenerNode::Group(group) => self.validate_group(group, errors),
                ScreenerNode::Condition(condition) => self.validate_condition(condition, errors),
            }
        }
    }

    fn validate_condition(
        &self,
        condition: &ScreenerCondition,
        errors: &mut Vec<ScreenerValidationError>,
    ) {
        let Some(field) = self.catalog.iter().find(|entry| entry.field == condition.field) else {
            errors.push(ScreenerValidationError {
                condition_id: condition.id.clone(),
                field: condition.field.clone(),
                code: "unknown_field".to_string(),
                message: format!("Unknown screener field: {}", condition.field),
            });
            return;
        };

        if field.status == ScreenerFieldStatus::Unavailable {
            errors.push(ScreenerValidationError {
                condition_id: condition.id.clone(),
                field: condition.field.clone(),
                code: "field_unavailable".to_string(),
                message: format!("Field {} is not available yet", condition.field),
            });
            return;
        }

        if !field.operators.contains(&condition.operator) {
            errors.push(ScreenerValidationError {
                condition_id: condition.id.clone(),
                field: condition.field.clone(),
                code: "invalid_operator".to_string(),
                message: format!("Operator is not supported for {}", condition.field),
            });
        }
    }

    fn matches_group(&self, group: &ScreenerGroup, quote: &StockQuote) -> bool {
        match group.operator {
            ScreenerLogic::And => group.children.iter().all(|child| self.matches_node(child, quote)),
            ScreenerLogic::Or => group.children.iter().any(|child| self.matches_node(child, quote)),
        }
    }

    fn matches_node(&self, node: &ScreenerNode, quote: &StockQuote) -> bool {
        match node {
            ScreenerNode::Group(group) => self.matches_group(group, quote),
            ScreenerNode::Condition(condition) => self.matches_condition(condition, quote),
        }
    }

    fn matches_condition(&self, condition: &ScreenerCondition, quote: &StockQuote) -> bool {
        match condition.operator {
            ScreenerOperator::Contains => self
                .field_text_value(quote, &condition.field)
                .zip(match &condition.value {
                    ScreenerValue::Text(text) => Some(text.as_str()),
                    _ => None,
                })
                .map(|(candidate, needle)| candidate.contains(needle))
                .unwrap_or(false),
            ScreenerOperator::In => false,
            _ => self.evaluate_numeric_condition(
                self.field_numeric_value(quote, &condition.field),
                &condition.operator,
                &condition.value,
            ),
        }
    }

    fn evaluate_numeric_condition(
        &self,
        current: Option<f64>,
        operator: &ScreenerOperator,
        value: &ScreenerValue,
    ) -> bool {
        let Some(current) = current else {
            return false;
        };

        match (operator, value) {
            (ScreenerOperator::GreaterThan, ScreenerValue::Number(threshold)) => current > *threshold,
            (ScreenerOperator::GreaterThanOrEqual, ScreenerValue::Number(threshold)) => current >= *threshold,
            (ScreenerOperator::LessThan, ScreenerValue::Number(threshold)) => current < *threshold,
            (ScreenerOperator::LessThanOrEqual, ScreenerValue::Number(threshold)) => current <= *threshold,
            (ScreenerOperator::Equal, ScreenerValue::Number(threshold)) => (current - *threshold).abs() < f64::EPSILON,
            (ScreenerOperator::Between, ScreenerValue::Range { min, max }) => current >= *min && current <= *max,
            _ => false,
        }
    }

    fn sort_quotes(&self, quotes: &mut [StockQuote], definition: &ScreenerDefinition) {
        quotes.sort_by(|left, right| {
            for sort in &definition.sorts {
                let ordering = self.compare_field(left, right, &sort.field);
                if ordering != Ordering::Equal {
                    return match sort.direction {
                        ScreenerSortDirection::Asc => ordering,
                        ScreenerSortDirection::Desc => ordering.reverse(),
                    };
                }
            }
            Ordering::Equal
        });
    }

    fn compare_field(&self, left: &StockQuote, right: &StockQuote, field: &str) -> Ordering {
        match (self.field_numeric_value(left, field), self.field_numeric_value(right, field)) {
            (Some(lhs), Some(rhs)) => lhs.partial_cmp(&rhs).unwrap_or(Ordering::Equal),
            _ => self.field_text_value(left, field).cmp(&self.field_text_value(right, field)),
        }
    }

    fn project_quote(&self, quote: &StockQuote, columns: &[String]) -> ScreenerRow {
        let selected_columns = if columns.is_empty() {
            vec!["symbol".to_string(), "name".to_string(), "latest_price".to_string()]
        } else {
            columns.to_vec()
        };

        selected_columns
            .into_iter()
            .filter_map(|column| self.column_value(quote, &column).map(|value| (column, value)))
            .collect()
    }

    fn column_value(&self, quote: &StockQuote, field: &str) -> Option<Value> {
        if let Some(value) = self.field_numeric_value(quote, field) {
            return Some(json!(value));
        }

        self.field_text_value(quote, field).map(|value| json!(value))
    }

    fn field_numeric_value(&self, quote: &StockQuote, field: &str) -> Option<f64> {
        match field {
            "latest_price" => Some(quote.price),
            "change_pct" => Some(quote.change_pct),
            "volume" => Some(quote.volume),
            "turnover_rate" => Some(quote.turnover_rate),
            "pe_ratio" => Some(quote.pe_ratio),
            "total_market_cap" => Some(quote.total_market_cap),
            _ => None,
        }
    }

    fn field_text_value(&self, quote: &StockQuote, field: &str) -> Option<String> {
        match field {
            "symbol" => Some(quote.symbol.clone()),
            "name" => Some(quote.name.clone()),
            _ => None,
        }
    }

    fn number_field(
        field: &str,
        label: &str,
        category: &str,
        eastmoney_compatible: bool,
        status: ScreenerFieldStatus,
    ) -> ScreenerCatalogField {
        ScreenerCatalogField {
            field: field.to_string(),
            label: label.to_string(),
            category: category.to_string(),
            value_type: if field == "latest_price" {
                ScreenerFieldValueType::Range
            } else {
                ScreenerFieldValueType::Number
            },
            operators: vec![
                ScreenerOperator::GreaterThan,
                ScreenerOperator::GreaterThanOrEqual,
                ScreenerOperator::LessThan,
                ScreenerOperator::LessThanOrEqual,
                ScreenerOperator::Equal,
                ScreenerOperator::Between,
            ],
            data_source: "quote_cache".to_string(),
            eastmoney_compatible,
            status,
        }
    }

    fn text_field(
        field: &str,
        label: &str,
        category: &str,
        eastmoney_compatible: bool,
        status: ScreenerFieldStatus,
    ) -> ScreenerCatalogField {
        ScreenerCatalogField {
            field: field.to_string(),
            label: label.to_string(),
            category: category.to_string(),
            value_type: ScreenerFieldValueType::Text,
            operators: vec![ScreenerOperator::Equal, ScreenerOperator::Contains],
            data_source: "quote_cache".to_string(),
            eastmoney_compatible,
            status,
        }
    }

    fn map_import_field(field: &str) -> Option<&'static str> {
        match field {
            "latest_price" | "price" => Some("latest_price"),
            "change_pct" | "zdf" => Some("change_pct"),
            "volume" => Some("volume"),
            "turnover_rate" | "hsl" => Some("turnover_rate"),
            "pe_ratio" | "pe" => Some("pe_ratio"),
            "total_market_cap" | "market_cap" => Some("total_market_cap"),
            "roe" => Some("roe"),
            _ => None,
        }
    }

    fn parse_import_operator(operator: &str) -> Option<ScreenerOperator> {
        match operator {
            ">" => Some(ScreenerOperator::GreaterThan),
            ">=" => Some(ScreenerOperator::GreaterThanOrEqual),
            "<" => Some(ScreenerOperator::LessThan),
            "<=" => Some(ScreenerOperator::LessThanOrEqual),
            "=" => Some(ScreenerOperator::Equal),
            "between" => Some(ScreenerOperator::Between),
            "contains" => Some(ScreenerOperator::Contains),
            _ => None,
        }
    }

    fn parse_import_value(operator: &str, value: &str) -> Option<ScreenerValue> {
        match operator {
            "between" => {
                let parts: Vec<&str> = value.split("..").collect();
                if parts.len() != 2 {
                    return None;
                }
                Some(ScreenerValue::Range {
                    min: parts[0].parse().ok()?,
                    max: parts[1].parse().ok()?,
                })
            }
            "contains" => Some(ScreenerValue::Text(value.to_string())),
            _ => value.parse().ok().map(ScreenerValue::Number),
        }
    }

    fn warning(key: &str, reason: &str, raw: &str) -> crate::models::ImportedConditionWarning {
        crate::models::ImportedConditionWarning {
            key: key.to_string(),
            reason: reason.to_string(),
            raw: Some(raw.to_string()),
        }
    }
}

impl Default for ScreenerService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        ScreenerCondition, ScreenerDefinition, ScreenerGroup, ScreenerLogic, ScreenerNode,
        ScreenerSort, ScreenerSortDirection, ScreenerValue,
    };
    use chrono::Utc;

    fn definition_for(field: &str, operator: ScreenerOperator) -> ScreenerDefinition {
        ScreenerDefinition {
            name: Some("catalog check".to_string()),
            description: None,
            logic: ScreenerGroup {
                id: "root".to_string(),
                operator: ScreenerLogic::And,
                children: vec![ScreenerNode::Condition(ScreenerCondition {
                    id: "condition-1".to_string(),
                    field: field.to_string(),
                    operator,
                    value: ScreenerValue::Number(10.0),
                })],
            },
            sorts: vec![ScreenerSort {
                field: "change_pct".to_string(),
                direction: ScreenerSortDirection::Desc,
            }],
            columns: vec!["symbol".to_string(), "latest_price".to_string()],
            source: None,
            import_meta: None,
        }
    }

    fn sample_quotes() -> Vec<StockQuote> {
        vec![
            StockQuote {
                symbol: "000001.SZ".to_string(),
                name: "Alpha".to_string(),
                price: 12.0,
                change: 0.5,
                change_pct: 4.0,
                open: 11.4,
                high: 12.2,
                low: 11.1,
                pre_close: 11.5,
                volume: 1_000_000.0,
                turnover: 12_000_000.0,
                turnover_rate: 2.0,
                amplitude: 5.0,
                pe_ratio: 18.0,
                total_market_cap: 50_000_000.0,
                circulating_market_cap: 40_000_000.0,
                timestamp: Utc::now(),
                bid_prices: vec![],
                bid_volumes: vec![],
                ask_prices: vec![],
                ask_volumes: vec![],
            },
            StockQuote {
                symbol: "000002.SZ".to_string(),
                name: "Beta".to_string(),
                price: 18.0,
                change: 0.1,
                change_pct: 1.0,
                open: 17.9,
                high: 18.5,
                low: 17.8,
                pre_close: 17.9,
                volume: 900_000.0,
                turnover: 15_000_000.0,
                turnover_rate: 9.0,
                amplitude: 3.0,
                pe_ratio: 22.0,
                total_market_cap: 70_000_000.0,
                circulating_market_cap: 55_000_000.0,
                timestamp: Utc::now(),
                bid_prices: vec![],
                bid_volumes: vec![],
                ask_prices: vec![],
                ask_volumes: vec![],
            },
            StockQuote {
                symbol: "600000.SH".to_string(),
                name: "Gamma".to_string(),
                price: 30.0,
                change: 1.0,
                change_pct: 5.0,
                open: 29.0,
                high: 30.5,
                low: 28.8,
                pre_close: 29.0,
                volume: 2_000_000.0,
                turnover: 60_000_000.0,
                turnover_rate: 10.0,
                amplitude: 5.5,
                pe_ratio: 30.0,
                total_market_cap: 90_000_000.0,
                circulating_market_cap: 80_000_000.0,
                timestamp: Utc::now(),
                bid_prices: vec![],
                bid_volumes: vec![],
                ask_prices: vec![],
                ask_volumes: vec![],
            },
        ]
    }

    #[test]
    fn screener_catalog_contains_ready_quote_fields() {
        let service = ScreenerService::new();
        let catalog = service.catalog();

        for field in [
            "latest_price",
            "change_pct",
            "volume",
            "turnover_rate",
            "pe_ratio",
            "total_market_cap",
        ] {
            assert!(catalog.iter().any(|entry| entry.field == field), "missing {field}");
        }
    }

    #[test]
    fn screener_catalog_rejects_unknown_field_and_bad_operator_pairs() {
        let service = ScreenerService::new();

        let unknown_field = definition_for("not_real", ScreenerOperator::GreaterThanOrEqual);
        let errors = service.validate_definition(&unknown_field).unwrap_err();
        assert!(errors.iter().any(|error| error.condition_id == "condition-1" && error.code == "unknown_field"));

        let invalid_operator = definition_for("latest_price", ScreenerOperator::Contains);
        let errors = service.validate_definition(&invalid_operator).unwrap_err();
        assert!(errors.iter().any(|error| error.condition_id == "condition-1" && error.code == "invalid_operator"));
    }

    #[test]
    fn screener_catalog_flags_unavailable_fields() {
        let service = ScreenerService::new();
        let unavailable = definition_for("roe", ScreenerOperator::GreaterThanOrEqual);

        let errors = service.validate_definition(&unavailable).unwrap_err();
        assert!(errors.iter().any(|error| error.condition_id == "condition-1" && error.code == "field_unavailable"));
    }

    #[test]
    fn screener_execution_filters_nested_groups_and_projects_selected_columns() {
        let service = ScreenerService::new();
        let definition = ScreenerDefinition {
            name: Some("execution".to_string()),
            description: None,
            logic: ScreenerGroup {
                id: "root".to_string(),
                operator: ScreenerLogic::And,
                children: vec![
                    ScreenerNode::Condition(ScreenerCondition {
                        id: "price-band".to_string(),
                        field: "latest_price".to_string(),
                        operator: ScreenerOperator::Between,
                        value: ScreenerValue::Range { min: 10.0, max: 20.0 },
                    }),
                    ScreenerNode::Group(ScreenerGroup {
                        id: "nested".to_string(),
                        operator: ScreenerLogic::Or,
                        children: vec![
                            ScreenerNode::Condition(ScreenerCondition {
                                id: "pct-up".to_string(),
                                field: "change_pct".to_string(),
                                operator: ScreenerOperator::GreaterThanOrEqual,
                                value: ScreenerValue::Number(3.0),
                            }),
                            ScreenerNode::Condition(ScreenerCondition {
                                id: "turnover-hot".to_string(),
                                field: "turnover_rate".to_string(),
                                operator: ScreenerOperator::GreaterThanOrEqual,
                                value: ScreenerValue::Number(8.0),
                            }),
                        ],
                    }),
                ],
            },
            sorts: vec![ScreenerSort {
                field: "change_pct".to_string(),
                direction: ScreenerSortDirection::Desc,
            }],
            columns: vec![
                "symbol".to_string(),
                "latest_price".to_string(),
                "change_pct".to_string(),
            ],
            source: None,
            import_meta: None,
        };

        let result = service.execute(&definition, &sample_quotes(), None).unwrap();

        assert_eq!(result.total_count, 2);
        assert_eq!(result.rows[0].get("symbol"), Some(&json!("000001.SZ")));
        assert_eq!(result.rows[1].get("symbol"), Some(&json!("000002.SZ")));
        assert!(!result.rows[0].contains_key("volume"));
    }

    #[test]
    fn eastmoney_import_maps_supported_conditions_and_collects_warnings() {
        let service = ScreenerService::new();
        let definition = service
            .import_eastmoney_url(
                "https://xuangu.eastmoney.com/result?filters=latest_price:between:10..20;change_pct:>=:3;unsupported_metric:>=:1",
            )
            .unwrap();

        assert_eq!(definition.source, Some(ScreenerSource::EastmoneyImport));
        assert_eq!(definition.logic.children.len(), 2);
        assert_eq!(definition.import_meta.as_ref().unwrap().imported_conditions, 2);
        assert_eq!(definition.import_meta.as_ref().unwrap().unsupported_conditions.len(), 1);
    }

    #[test]
    fn eastmoney_import_rejects_malformed_urls() {
        let service = ScreenerService::new();
        let error = service.import_eastmoney_url("not-a-url").unwrap_err();

        assert_eq!(error.code, "invalid_url");
    }
}