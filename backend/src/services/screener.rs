use crate::models::{
    ScreenerCatalogField, ScreenerCondition, ScreenerDefinition, ScreenerFieldStatus,
    ScreenerFieldValueType, ScreenerGroup, ScreenerNode, ScreenerOperator,
};

#[derive(Debug, Clone)]
pub struct ScreenerService {
    catalog: Vec<ScreenerCatalogField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenerValidationError {
    pub condition_id: String,
    pub field: String,
    pub code: String,
    pub message: String,
}

impl ScreenerService {
    pub fn new() -> Self {
        Self {
            catalog: vec![
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
        ScreenerDefinition, ScreenerGroup, ScreenerLogic, ScreenerNode, ScreenerSort,
        ScreenerSortDirection, ScreenerValue,
    };

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
}
