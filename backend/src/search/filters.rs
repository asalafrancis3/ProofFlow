use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Filter types for search queries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FilterType {
    Term {
        field: String,
        value: Value,
    },
    Range {
        field: String,
        min: Option<Value>,
        max: Option<Value>,
    },
    Exists {
        field: String,
    },
    Prefix {
        field: String,
        value: String,
    },
    Match {
        field: String,
        query: String,
    },
}

/// Search filter structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    pub filter: FilterType,
    pub operator: FilterOperator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FilterOperator {
    And,
    Or,
    Not,
}

impl SearchFilter {
    pub fn term(field: impl Into<String>, value: Value) -> Self {
        Self {
            filter: FilterType::Term {
                field: field.into(),
                value,
            },
            operator: FilterOperator::And,
        }
    }

    pub fn range(field: impl Into<String>, min: Option<Value>, max: Option<Value>) -> Self {
        Self {
            filter: FilterType::Range {
                field: field.into(),
                min,
                max,
            },
            operator: FilterOperator::And,
        }
    }

    pub fn exists(field: impl Into<String>) -> Self {
        Self {
            filter: FilterType::Exists { field: field.into() },
            operator: FilterOperator::And,
        }
    }
}
