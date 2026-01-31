use std::path::{Path, PathBuf};

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use tokio::fs;

use super::service::EnrichedData;
use super::types::{CardImage, CardState, CardType, DomainId, StreamCard};

#[derive(Clone)]
pub struct ObsidianIO {
    obsidian_root: Option<String>,
}

impl ObsidianIO {
    pub fn new(obsidian_root: Option<String>) -> Self {
        Self { obsidian_root }
    }

    pub async fn load_cards_for_date(
        &self,
        workspace_path: &str,
        obsidian_root: Option<&str>,
        date_iso: &str,
    ) -> Result<Vec<StreamCard>, String> {
        let root = self.resolve_root(workspace_path, obsidian_root)?;
        let date = NaiveDate::parse_from_str(date_iso, "%Y-%m-%d")
            .map_err(|err| format!("Invalid date {date_iso}: {err}"))?;
        let stream_file = stream_file_path(&root, date.year(), date.month());
        if !stream_file.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&stream_file)
            .await
            .map_err(|err| format!("Failed to read stream file: {err}"))?;
        Ok(parse_cards_from_stream(&content, date, &stream_file))
    }

    pub async fn write_card(
        &self,
        workspace_path: &str,
        obsidian_root: Option<&str>,
        card_id: &str,
        occurred_at: &str,
        enriched: &EnrichedData,
    ) -> Result<(), String> {
        let root = self.resolve_root(workspace_path, obsidian_root)?;
        let date_time = chrono::DateTime::parse_from_rfc3339(occurred_at)
            .map(|value| value.naive_local())
            .or_else(|_| NaiveDateTime::parse_from_str(occurred_at, "%Y-%m-%dT%H:%M:%S%z"))
            .or_else(|_| NaiveDateTime::parse_from_str(occurred_at, "%Y-%m-%dT%H:%M:%S%.f%z"))
            .or_else(|_| NaiveDateTime::parse_from_str(occurred_at, "%Y-%m-%dT%H:%M:%S"))
            .or_else(|_| NaiveDateTime::parse_from_str(occurred_at, "%Y-%m-%dT%H:%M:%S%.f"))
            .map_err(|err| format!("Invalid timestamp {occurred_at}: {err}"))?;
        let date = date_time.date();
        let stream_file = stream_file_path(&root, date.year(), date.month());
        if let Some(parent) = stream_file.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|err| format!("Failed to create stream directory: {err}"))?;
        }

        let content = if stream_file.exists() {
            fs::read_to_string(&stream_file)
                .await
                .map_err(|err| format!("Failed to read stream file: {err}"))?
        } else {
            String::new()
        };

        let updated = append_card_entry(&content, date, card_id, occurred_at, enriched);
        fs::write(&stream_file, updated)
            .await
            .map_err(|err| format!("Failed to write stream file: {err}"))?;

        Ok(())
    }

    fn resolve_root(
        &self,
        workspace_path: &str,
        obsidian_root: Option<&str>,
    ) -> Result<PathBuf, String> {
        if let Some(root) = obsidian_root {
            return Ok(PathBuf::from(root));
        }
        if let Some(root) = &self.obsidian_root {
            return Ok(PathBuf::from(root));
        }
        let fallback = PathBuf::from(workspace_path);
        if fallback.exists() {
            return Ok(fallback);
        }
        let default = PathBuf::from("/Volumes/YouTube 4TB/Obsidian");
        Ok(default)
    }
}

fn stream_file_path(root: &Path, year: i32, month: u32) -> PathBuf {
    let filename = format!("{year}-{month:02}.md");
    root.join("Stream").join(filename)
}

fn append_card_entry(
    content: &str,
    date: NaiveDate,
    card_id: &str,
    occurred_at: &str,
    enriched: &EnrichedData,
) -> String {
    let header = format!("## {}", date.format("%a %b %d"));
    let time_label = occurred_at.get(11..16).unwrap_or("00:00");
    let compact_time = time_label.replace(':', "");
    let task_id = format!("{}-{}-{}", date.format("%Y-%m-%d"), compact_time, card_id);
    let entry_line = format!(
        "| -- | {} {} | + | <!--task:{}-->",
        time_label, enriched.title, task_id
    );
    let note_line = format!("<!--note:{}-->", task_id);
    let note_body = enriched
        .summary
        .clone()
        .unwrap_or_else(|| enriched.subtitle.clone().unwrap_or_default());

    let mut lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
    if lines.is_empty() {
        lines.push(header.clone());
        lines.push("| Plan | Actual | Delta |".to_string());
        lines.push("|------|--------|---|".to_string());
        lines.push(entry_line);
        lines.push("---".to_string());
        lines.push(note_line);
        if !note_body.is_empty() {
            lines.push(note_body);
        }
        return lines.join("\n") + "\n";
    }

    let mut output = Vec::new();
    let mut inserted = false;
    let mut idx = 0;
    while idx < lines.len() {
        let line = &lines[idx];
        if !inserted && line.trim() == header {
            output.push(line.clone());
            idx += 1;
            while idx < lines.len() {
                let current = &lines[idx];
                if current.starts_with("## ") && current.trim() != header {
                    break;
                }
                if !inserted && current.trim() == "---" {
                    output.push(entry_line.clone());
                    inserted = true;
                }
                output.push(current.clone());
                idx += 1;
            }
            if !inserted {
                output.push(entry_line.clone());
                inserted = true;
            }
            output.push("---".to_string());
            output.push(note_line.clone());
            if !note_body.is_empty() {
                output.push(note_body.clone());
            }
            continue;
        }
        output.push(line.clone());
        idx += 1;
    }

    if !inserted {
        output.push("".to_string());
        output.push(header);
        output.push("| Plan | Actual | Delta |".to_string());
        output.push("|------|--------|---|".to_string());
        output.push(entry_line);
        output.push("---".to_string());
        output.push(note_line);
        if !note_body.is_empty() {
            output.push(note_body);
        }
    }

    output.join("\n") + "\n"
}

fn parse_cards_from_stream(
    content: &str,
    date: NaiveDate,
    stream_file: &Path,
) -> Vec<StreamCard> {
    let mut cards = Vec::new();
    let mut current_date: Option<NaiveDate> = None;
    for line in content.lines() {
        if let Some(parsed) = parse_header_date(line, date.year()) {
            current_date = Some(parsed);
            continue;
        }
        if current_date != Some(date) {
            continue;
        }
        if let Some(entry) = parse_table_entry(line, date) {
            cards.push(StreamCard {
                id: entry.task_id.clone(),
                occurred_at: entry.occurred_at.clone(),
                created_at: entry.occurred_at.clone(),
                updated_at: entry.occurred_at.clone(),
                version: 1,
                card_type: CardType::Generic,
                domain: DomainId::General,
                emoji: "📝".to_string(),
                state: CardState::Complete,
                processing_step: None,
                processing_steps: None,
                title: entry.title.clone(),
                subtitle: None,
                summary: None,
                image: Some(CardImage {
                    url: None,
                    status: super::types::ImageStatus::Missing,
                }),
                stats: None,
                entities: None,
                original_input: None,
                source: Some(super::types::CardSource {
                    stream_file: Some(stream_file.to_string_lossy().to_string()),
                    stream_anchor: Some(entry.task_id.clone()),
                }),
                expanded: None,
                error_message: None,
            });
        }
    }
    cards
}

struct ParsedEntry {
    task_id: String,
    occurred_at: String,
    title: String,
}

fn parse_table_entry(line: &str, date: NaiveDate) -> Option<ParsedEntry> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.contains("<!--task:") {
        return None;
    }
    let parts: Vec<&str> = trimmed.split('|').collect();
    if parts.len() < 4 {
        return None;
    }
    let actual = parts.get(2)?.trim();
    let time_title: Vec<&str> = actual.split_whitespace().collect();
    if time_title.is_empty() {
        return None;
    }
    let time = time_title[0];
    let title = time_title[1..].join(" ");
    let task_id = trimmed
        .split("<!--task:")
        .nth(1)
        .and_then(|v| v.split("-->").next())
        .unwrap_or("")
        .to_string();
    if task_id.is_empty() {
        return None;
    }
    let time_24 = normalize_time(time).unwrap_or_else(|| "00:00".to_string());
    let occurred_at = format!("{}T{}:00", date.format("%Y-%m-%d"), time_24);
    Some(ParsedEntry {
        task_id,
        occurred_at,
        title,
    })
}

fn parse_header_date(line: &str, year: i32) -> Option<NaiveDate> {
    let trimmed = line.trim();
    if !trimmed.starts_with("## ") {
        return None;
    }
    let parts: Vec<&str> = trimmed.trim_start_matches("## ").split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let month = month_number(parts[1])?;
    let day: u32 = parts[2].parse().ok()?;
    NaiveDate::from_ymd_opt(year, month, day)
}

fn month_number(name: &str) -> Option<u32> {
    match name {
        "Jan" => Some(1),
        "Feb" => Some(2),
        "Mar" => Some(3),
        "Apr" => Some(4),
        "May" => Some(5),
        "Jun" => Some(6),
        "Jul" => Some(7),
        "Aug" => Some(8),
        "Sep" => Some(9),
        "Oct" => Some(10),
        "Nov" => Some(11),
        "Dec" => Some(12),
        _ => None,
    }
}

fn normalize_time(value: &str) -> Option<String> {
    let lower = value.to_lowercase();
    let (time_part, suffix) = if lower.ends_with("am") {
        (lower.trim_end_matches("am"), "am")
    } else if lower.ends_with("pm") {
        (lower.trim_end_matches("pm"), "pm")
    } else {
        (lower.as_str(), "")
    };
    let parts: Vec<&str> = time_part.split(':').collect();
    if parts.len() < 2 {
        return None;
    }
    let mut hour: i32 = parts[0].parse().ok()?;
    let minute = parts[1].parse::<i32>().ok()?;
    if suffix == "pm" && hour < 12 {
        hour += 12;
    }
    if suffix == "am" && hour == 12 {
        hour = 0;
    }
    Some(format!("{:02}:{:02}", hour, minute))
}
