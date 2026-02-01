use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use tokio::fs;

use crate::life_stream::types::EntityRef;

#[derive(Debug, Clone, Deserialize)]
struct MediaEntity {
    id: Option<String>,
    title: String,
    #[serde(rename = "type")]
    media_type: Option<String>,
    status: Option<String>,
    rating: Option<i64>,
    year: Option<i64>,
    creator: Option<String>,
}

pub struct MediaHandler {
    obsidian_root: String,
}

impl MediaHandler {
    pub fn new(obsidian_root: &str) -> Self {
        Self {
            obsidian_root: obsidian_root.to_string(),
        }
    }

    pub async fn process(&self, input: &str, occurred_at: &str) -> Result<ProcessedMedia, String> {
        let entities = self.extract_media_entities(input).await?;
        let time_label = occurred_at.get(11..16).unwrap_or("??:??");

        let title = if entities.len() == 1 {
            entities[0].title.clone()
        } else if entities.is_empty() {
            format!("Media log {}", time_label)
        } else {
            format!("Media log {}", time_label)
        };

        let subtitle = if entities.is_empty() {
            None
        } else {
            let summaries: Vec<String> = entities
                .iter()
                .map(|entity| {
                    if let Some(media_type) = &entity.media_type {
                        media_type.clone()
                    } else {
                        "Media".to_string()
                    }
                })
                .collect();
            Some(summaries.join(" • "))
        };

        let mut stats = HashMap::new();
        if !entities.is_empty() {
            stats.insert("items".to_string(), serde_json::json!(entities.len()));
        }

        let mut entity_refs = Vec::new();
        for entity in &entities {
            entity_refs.push(EntityRef {
                entity_type: "media".to_string(),
                id: entity.id.clone(),
                name: entity.title.clone(),
                link: Some(format!("[[Entities/Media/{}]]", entity.title)),
            });
        }

        Ok(ProcessedMedia {
            title,
            subtitle,
            summary: if entities.is_empty() {
                Some(input.to_string())
            } else {
                None
            },
            stats: if stats.is_empty() { None } else { Some(stats) },
            entities: if entity_refs.is_empty() {
                None
            } else {
                Some(entity_refs)
            },
        })
    }

    async fn extract_media_entities(&self, input: &str) -> Result<Vec<MediaEntity>, String> {
        let media_dir = Path::new(&self.obsidian_root).join("Entities/Media");
        if !media_dir.exists() {
            return Ok(Vec::new());
        }

        let lower_input = input.to_lowercase();
        let mut found = Vec::new();

        let mut entries = fs::read_dir(&media_dir)
            .await
            .map_err(|e| format!("Failed to read media directory: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read entry: {}", e))?
        {
            let path = entry.path();
            if !path.extension().map(|e| e == "md").unwrap_or(false) {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&path).await {
                if let Some(entity) = self.parse_media_entity(&content) {
                    if lower_input.contains(&entity.title.to_lowercase()) {
                        found.push(entity);
                    }
                }
            }
        }

        Ok(found)
    }

    fn parse_media_entity(&self, content: &str) -> Option<MediaEntity> {
        if !content.starts_with("---") {
            return None;
        }

        let end = content[3..].find("---")?;
        let frontmatter = &content[3..3 + end];

        let mut id = None;
        let mut title = None;
        let mut media_type = None;
        let mut status = None;
        let mut rating = None;
        let mut year = None;
        let mut creator = None;

        for line in frontmatter.lines() {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }
            let key = parts[0].trim();
            let value = parts[1].trim().trim_matches('"');

            match key {
                "id" => id = Some(value.to_string()),
                "title" => title = Some(value.to_string()),
                "type" => media_type = Some(value.to_string()),
                "status" => status = Some(value.to_string()),
                "rating" => rating = value.parse().ok(),
                "year" => year = value.parse().ok(),
                "creator" => creator = Some(value.to_string()),
                _ => {}
            }
        }

        Some(MediaEntity {
            id,
            title: title?,
            media_type,
            status,
            rating,
            year,
            creator,
        })
    }
}

pub struct ProcessedMedia {
    pub title: String,
    pub subtitle: Option<String>,
    pub summary: Option<String>,
    pub stats: Option<HashMap<String, serde_json::Value>>,
    pub entities: Option<Vec<EntityRef>>,
}
