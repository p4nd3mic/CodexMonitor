use std::collections::HashMap;

use crate::life_stream::types::EntityRef;

pub struct ThoughtHandler;

impl ThoughtHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn process(&self, input: &str) -> Result<ProcessedThought, String> {
        let trimmed = input.trim();
        let title = if trimmed.len() > 64 {
            format!("{}...", &trimmed[..64])
        } else if trimmed.is_empty() {
            "Thought".to_string()
        } else {
            trimmed.to_string()
        };

        let summary = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };

        let mut stats = HashMap::new();
        if !trimmed.is_empty() {
            stats.insert(
                "chars".to_string(),
                serde_json::json!(trimmed.chars().count()),
            );
        }

        Ok(ProcessedThought {
            title,
            subtitle: Some("Captured thought".to_string()),
            summary,
            stats: if stats.is_empty() { None } else { Some(stats) },
            entities: None,
        })
    }
}

pub struct ProcessedThought {
    pub title: String,
    pub subtitle: Option<String>,
    pub summary: Option<String>,
    pub stats: Option<HashMap<String, serde_json::Value>>,
    pub entities: Option<Vec<EntityRef>>,
}
