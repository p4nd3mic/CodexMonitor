use std::collections::HashMap;

use crate::life_stream::service::truncate;
use crate::life_stream::types::{CardStatValue, EntityRef};

pub struct CodeTaskHandler;

impl CodeTaskHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn process(&self, input: &str) -> Result<ProcessedCodeTask, String> {
        let trimmed = input.trim();
        let title = if trimmed.is_empty() {
            "Code task".to_string()
        } else {
            truncate(trimmed, 64)
        };

        let mut stats = HashMap::new();
        if !trimmed.is_empty() {
            let word_count = trimmed.split_whitespace().count();
            stats.insert(
                "words".to_string(),
                CardStatValue::Integer(word_count as i64),
            );
        }

        Ok(ProcessedCodeTask {
            title,
            subtitle: Some("Code task captured".to_string()),
            summary: if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            },
            stats: if stats.is_empty() { None } else { Some(stats) },
            entities: None,
        })
    }
}

pub struct ProcessedCodeTask {
    pub title: String,
    pub subtitle: Option<String>,
    pub summary: Option<String>,
    pub stats: Option<HashMap<String, CardStatValue>>,
    pub entities: Option<Vec<EntityRef>>,
}
