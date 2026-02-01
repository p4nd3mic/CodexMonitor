use std::collections::HashMap;

use crate::life_stream::service::truncate;
use crate::life_stream::types::{CardStatValue, EntityRef};

pub struct QueryHandler;

impl QueryHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn process(&self, input: &str) -> Result<ProcessedQuery, String> {
        let normalized = input.trim();
        let title = if normalized.is_empty() {
            "Query".to_string()
        } else {
            format!("Query: {}", truncate(normalized, 48))
        };

        let mut stats = HashMap::new();
        if !normalized.is_empty() {
            stats.insert(
                "query_len".to_string(),
                CardStatValue::Integer(normalized.len() as i64),
            );
        }

        Ok(ProcessedQuery {
            title,
            subtitle: Some("Needs data lookup".to_string()),
            summary: if normalized.is_empty() {
                None
            } else {
                Some(normalized.to_string())
            },
            stats: if stats.is_empty() { None } else { Some(stats) },
            entities: None,
        })
    }
}

pub struct ProcessedQuery {
    pub title: String,
    pub subtitle: Option<String>,
    pub summary: Option<String>,
    pub stats: Option<HashMap<String, CardStatValue>>,
    pub entities: Option<Vec<EntityRef>>,
}
