use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use regex::Regex;
use tauri::Emitter;
use tokio::sync::{Mutex, Semaphore};

use super::handlers::code_task::CodeTaskHandler;
use super::handlers::delivery::DeliveryHandler;
use super::handlers::media::MediaHandler;
use super::handlers::nutrition::NutritionHandler;
use super::handlers::query::QueryHandler;
use super::handlers::thought::ThoughtHandler;
use super::images::ImageService;
use super::mcp_bridge::LifeMcpBridge;
use super::obsidian::ObsidianIO;
use super::types::*;

pub struct LifeStreamService {
    cards: Arc<Mutex<HashMap<String, StreamCard>>>,
    worker_semaphore: Arc<Semaphore>,
    write_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
    cancelled_cards: Arc<Mutex<HashSet<String>>>,
    obsidian: ObsidianIO,
    tmdb_api_key: Option<String>,
    mcp_bridge: LifeMcpBridge,
    emitter: Option<tauri::AppHandle>,
    event_sink: Option<Arc<dyn Fn(LifeStreamEvent) + Send + Sync>>,
}

impl LifeStreamService {
    pub fn new(obsidian_root: Option<String>, tmdb_api_key: Option<String>) -> Self {
        Self {
            cards: Arc::new(Mutex::new(HashMap::new())),
            worker_semaphore: Arc::new(Semaphore::new(5)),
            write_locks: Arc::new(Mutex::new(HashMap::new())),
            cancelled_cards: Arc::new(Mutex::new(HashSet::new())),
            obsidian: ObsidianIO::new(obsidian_root),
            tmdb_api_key,
            mcp_bridge: LifeMcpBridge::from_env(),
            emitter: None,
            event_sink: None,
        }
    }

    pub fn set_emitter(&mut self, app: tauri::AppHandle) {
        self.emitter = Some(app);
    }

    pub fn set_event_sink<F>(&mut self, sink: F)
    where
        F: Fn(LifeStreamEvent) + Send + Sync + 'static,
    {
        self.event_sink = Some(Arc::new(sink));
    }

    pub async fn load_day(
        &self,
        workspace_path: &str,
        obsidian_root: Option<&str>,
        date_iso: &str,
    ) -> Result<Vec<StreamCard>, String> {
        self.obsidian
            .load_cards_for_date(workspace_path, obsidian_root, date_iso)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn submit(
        &self,
        workspace_id: &str,
        workspace_path: &str,
        obsidian_root: Option<&str>,
        card_id: &str,
        input: &str,
        occurred_at: Option<&str>,
    ) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        let occurred = occurred_at.unwrap_or(&now).to_string();

        {
            let mut cancelled = self.cancelled_cards.lock().await;
            cancelled.remove(card_id);
        }

        let card = StreamCard {
            id: card_id.to_string(),
            occurred_at: occurred.clone(),
            created_at: now.clone(),
            updated_at: now.clone(),
            version: 1,
            card_type: CardType::Generic,
            domain: DomainId::General,
            emoji: "📝".to_string(),
            state: CardState::Pending,
            processing_step: Some("Queued...".to_string()),
            processing_steps: Some(vec!["Queued...".to_string()]),
            title: truncate(input, 50),
            subtitle: None,
            summary: None,
            image: None,
            stats: None,
            entities: None,
            original_input: Some(input.to_string()),
            source: None,
            expanded: None,
            clarification_options: None,
            error_message: None,
        };

        {
            let mut cards = self.cards.lock().await;
            cards.insert(card_id.to_string(), card.clone());
        }

        self.emit_event(LifeStreamEvent::CardCreated { card: card.clone() });

        self.spawn_processing(
            workspace_id.to_string(),
            workspace_path.to_string(),
            obsidian_root.map(|value| value.to_string()),
            card_id.to_string(),
            input.to_string(),
            occurred,
            None,
            self.tmdb_api_key.clone(),
        );

        Ok(())
    }

    pub async fn cancel(&self, card_id: &str) -> Result<(), String> {
        emit_patch(
            card_id,
            StreamCardPatch {
                state: Some(CardState::Cancelled),
                processing_step: Some("Cancelled".to_string()),
                error_message: Some(String::new()),
                ..Default::default()
            },
            &self.cards,
            &self.emitter,
            &self.event_sink,
        )
        .await
        .ok_or("card not found")?;

        let mut cancelled = self.cancelled_cards.lock().await;
        cancelled.insert(card_id.to_string());
        drop(cancelled);

        Ok(())
    }

    pub async fn retry(
        &self,
        workspace_id: &str,
        workspace_path: &str,
        obsidian_root: Option<&str>,
        card_id: &str,
    ) -> Result<(), String> {
        let (input, occurred_at) = {
            let cards_guard = self.cards.lock().await;
            let card = cards_guard.get(card_id).ok_or("card not found")?;
            let input = card
                .original_input
                .clone()
                .ok_or("card has no original input")?;
            (input, card.occurred_at.clone())
        };

        {
            let mut cancelled = self.cancelled_cards.lock().await;
            cancelled.remove(card_id);
        }

        emit_patch(
            card_id,
            StreamCardPatch {
                state: Some(CardState::Processing),
                processing_step: Some("Retrying...".to_string()),
                error_message: Some(String::new()),
                ..Default::default()
            },
            &self.cards,
            &self.emitter,
            &self.event_sink,
        )
        .await
        .ok_or("card not found")?;

        self.spawn_processing(
            workspace_id.to_string(),
            workspace_path.to_string(),
            obsidian_root.map(|value| value.to_string()),
            card_id.to_string(),
            input,
            occurred_at,
            None,
            self.tmdb_api_key.clone(),
        );

        Ok(())
    }

    pub async fn resume_with_clarification(
        &self,
        workspace_id: &str,
        workspace_path: &str,
        obsidian_root: Option<&str>,
        card_id: &str,
        option_id: &str,
    ) -> Result<(), String> {
        let (input, occurred_at) = {
            let cards_guard = self.cards.lock().await;
            let card = cards_guard.get(card_id).ok_or("card not found")?;
            let input = card
                .original_input
                .clone()
                .ok_or("card has no original input")?;
            (input, card.occurred_at.clone())
        };

        emit_patch(
            card_id,
            StreamCardPatch {
                state: Some(CardState::Processing),
                processing_step: Some("Resuming...".to_string()),
                clarification_options: Some(Vec::new()),
                error_message: Some(String::new()),
                ..Default::default()
            },
            &self.cards,
            &self.emitter,
            &self.event_sink,
        )
        .await
        .ok_or("card not found")?;

        self.spawn_processing(
            workspace_id.to_string(),
            workspace_path.to_string(),
            obsidian_root.map(|value| value.to_string()),
            card_id.to_string(),
            input,
            occurred_at,
            Some(option_id.to_string()),
            self.tmdb_api_key.clone(),
        );

        Ok(())
    }

    fn emit_event(&self, event: LifeStreamEvent) {
        if let Some(app) = &self.emitter {
            let _ = app.emit("life_stream_event", event.clone());
        }
        if let Some(sink) = &self.event_sink {
            sink(event);
        }
    }

    fn spawn_processing(
        &self,
        workspace_id: String,
        workspace_path: String,
        obsidian_root: Option<String>,
        card_id: String,
        input: String,
        occurred: String,
        clarification: Option<String>,
        tmdb_api_key: Option<String>,
    ) {
        let cards = Arc::clone(&self.cards);
        let semaphore = Arc::clone(&self.worker_semaphore);
        let write_locks = Arc::clone(&self.write_locks);
        let obsidian = self.obsidian.clone();
        let emitter = self.emitter.clone();
        let event_sink = self.event_sink.clone();
        let cancelled_cards = Arc::clone(&self.cancelled_cards);
        let clarification = clarification.clone();
        let mcp_bridge = self.mcp_bridge.clone();

        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let lock = {
                let mut locks = write_locks.lock().await;
                locks
                    .entry(workspace_id.clone())
                    .or_insert_with(|| Arc::new(Mutex::new(())))
                    .clone()
            };

            let _ = emit_patch(
                &card_id,
                StreamCardPatch {
                    state: Some(CardState::Processing),
                    processing_step: Some("Starting...".to_string()),
                    ..Default::default()
                },
                &cards,
                &emitter,
                &event_sink,
            )
            .await;

            let result = process_card(
                &card_id,
                workspace_path.as_str(),
                obsidian_root.as_deref(),
                &input,
                &occurred,
                clarification.as_deref(),
                &cards,
                &lock,
                &obsidian,
                &emitter,
                &event_sink,
                &cancelled_cards,
                &mcp_bridge,
                tmdb_api_key.as_deref(),
            )
            .await;

            if let Err(e) = result {
                let mut cards_guard = cards.lock().await;
                if let Some(card) = cards_guard.get_mut(&card_id) {
                    if card.state == CardState::Cancelled {
                        return;
                    }
                    card.state = CardState::Error;
                    card.error_message = Some(e.clone());
                    card.version += 1;

                    let event = LifeStreamEvent::CardError {
                        card_id: card_id.clone(),
                        message: e,
                        version: card.version,
                    };
                    if let Some(app) = &emitter {
                        let _ = app.emit("life_stream_event", event.clone());
                    }
                    if let Some(sink) = &event_sink {
                        sink(event);
                    }
                }
            }
        });
    }
}

async fn emit_patch(
    card_id: &str,
    patch: StreamCardPatch,
    cards: &Arc<Mutex<HashMap<String, StreamCard>>>,
    emitter: &Option<tauri::AppHandle>,
    event_sink: &Option<Arc<dyn Fn(LifeStreamEvent) + Send + Sync>>,
) -> Option<u32> {
    let version = {
        let mut cards_guard = cards.lock().await;
        let card = cards_guard.get_mut(card_id)?;
        apply_patch_to_card(card, &patch);
        card.version += 1;
        card.updated_at = chrono::Utc::now().to_rfc3339();
        card.version
    };

    let event = LifeStreamEvent::CardUpdated {
        card_id: card_id.to_string(),
        patch,
        version,
    };

    if let Some(app) = emitter {
        let _ = app.emit("life_stream_event", event.clone());
    }
    if let Some(sink) = event_sink {
        sink(event);
    }

    Some(version)
}

fn apply_patch_to_card(card: &mut StreamCard, patch: &StreamCardPatch) {
    if let Some(state) = &patch.state {
        card.state = state.clone();
    }
    if let Some(title) = &patch.title {
        card.title = title.clone();
    }
    if let Some(subtitle) = &patch.subtitle {
        card.subtitle = Some(subtitle.clone());
    }
    if let Some(step) = &patch.processing_step {
        card.processing_step = Some(step.clone());
        let steps = card.processing_steps.get_or_insert_with(Vec::new);
        steps.push(step.clone());
    }
    if let Some(steps) = &patch.processing_steps {
        card.processing_steps = Some(steps.clone());
    }
    if let Some(error) = &patch.error_message {
        if error.is_empty() {
            card.error_message = None;
        } else {
            card.error_message = Some(error.clone());
        }
    }
    if let Some(stats) = &patch.stats {
        card.stats = Some(stats.clone());
    }
    if let Some(image) = &patch.image {
        card.image = Some(image.clone());
    }
    if let Some(expanded) = &patch.expanded {
        card.expanded = Some(expanded.clone());
    }
    if let Some(options) = &patch.clarification_options {
        if options.is_empty() {
            card.clarification_options = None;
        } else {
            card.clarification_options = Some(options.clone());
        }
    }
}

async fn process_card(
    card_id: &str,
    workspace_path: &str,
    obsidian_root: Option<&str>,
    input: &str,
    occurred_at: &str,
    clarification: Option<&str>,
    cards: &Arc<Mutex<HashMap<String, StreamCard>>>,
    write_lock: &Arc<Mutex<()>>,
    obsidian: &ObsidianIO,
    emitter: &Option<tauri::AppHandle>,
    event_sink: &Option<Arc<dyn Fn(LifeStreamEvent) + Send + Sync>>,
    cancelled_cards: &Arc<Mutex<HashSet<String>>>,
    mcp_bridge: &LifeMcpBridge,
    tmdb_api_key: Option<&str>,
) -> Result<(), String> {
    if is_cancelled(card_id, cancelled_cards).await {
        return Ok(());
    }
    emit_step(card_id, "Detecting intent...", cards, emitter, event_sink).await;
    let (card_type, domain, emoji) = detect_intent(input);

    if is_cancelled(card_id, cancelled_cards).await {
        return Ok(());
    }
    emit_step(
        card_id,
        &format!("Processing as {:?}...", domain),
        cards,
        emitter,
        event_sink,
    )
    .await;

    if is_cancelled(card_id, cancelled_cards).await {
        return Ok(());
    }

    let mut enriched = match card_type {
        CardType::Meal => {
            emit_step(card_id, "Looking up nutrition...", cards, emitter, event_sink).await;
            match handle_nutrition(
                card_id,
                input,
                occurred_at,
                workspace_path,
                obsidian_root,
                clarification,
                obsidian,
                cards,
                emitter,
                event_sink,
            )
            .await?
            {
                Some(value) => value,
                None => return Ok(()),
            }
        }
        CardType::DeliveryOrder => {
            handle_delivery(input, occurred_at, workspace_path, obsidian_root, obsidian).await?
        }
        CardType::MediaAdd => {
            handle_media(input, occurred_at, workspace_path, obsidian_root, obsidian).await?
        }
        CardType::Thought => handle_thought(input).await?,
        CardType::Query => handle_query(input).await?,
        CardType::CodeTask => handle_code_task(input).await?,
        _ => handle_generic(input).await?,
    };

    if is_cancelled(card_id, cancelled_cards).await {
        return Ok(());
    }

    if let Some(mcp_output) = maybe_call_mcp_tool(
        mcp_bridge,
        card_id,
        &card_type,
        input,
        cards,
        emitter,
        event_sink,
    )
    .await
    {
        apply_mcp_output(&mut enriched, mcp_output, input);
    }

    if is_cancelled(card_id, cancelled_cards).await {
        return Ok(());
    }
    emit_step(card_id, "Saving...", cards, emitter, event_sink).await;
    {
        let _guard = write_lock.lock().await;
        obsidian
            .write_card(
                workspace_path,
                obsidian_root,
                card_id,
                occurred_at,
                &enriched,
            )
            .await
            .map_err(|err| err.to_string())?;
    }

    let mut cards_guard = cards.lock().await;
    if let Some(card) = cards_guard.get_mut(card_id) {
        if card.state == CardState::Cancelled {
            return Ok(());
        }
        card.state = CardState::Complete;
        card.card_type = card_type;
        card.domain = domain;
        card.emoji = emoji;
        card.title = enriched.title;
        card.subtitle = enriched.subtitle;
        card.summary = enriched.summary;
        card.stats = enriched.stats;
        card.entities = enriched.entities;
        card.image = enriched.image;
        card.expanded = enriched.expanded;
        card.clarification_options = None;
        card.version += 1;

        let event = LifeStreamEvent::CardCompleted { card: card.clone() };
        if let Some(app) = emitter {
            let _ = app.emit("life_stream_event", event.clone());
        }
        if let Some(sink) = event_sink {
            sink(event);
        }
    }

    if let Some(lookup) = enriched.image_lookup {
        if let Some(root) = obsidian_root {
            let cards = Arc::clone(cards);
            let emitter = emitter.clone();
            let event_sink = event_sink.clone();
            let card_id = card_id.to_string();
            let tmdb_key = tmdb_api_key.map(|value| value.to_string());
            let root = PathBuf::from(root);
            tokio::spawn(async move {
                let image_service = ImageService::new(root, tmdb_key);
                let image = image_service
                    .fetch_image(card_type_label(&lookup.card_type), &lookup.entity_name)
                    .await;
                let _ = emit_patch(
                    &card_id,
                    StreamCardPatch {
                        image: Some(image),
                        ..Default::default()
                    },
                    &cards,
                    &emitter,
                    &event_sink,
                )
                .await;
            });
        }
    }

    Ok(())
}

async fn emit_step(
    card_id: &str,
    step: &str,
    cards: &Arc<Mutex<HashMap<String, StreamCard>>>,
    emitter: &Option<tauri::AppHandle>,
    event_sink: &Option<Arc<dyn Fn(LifeStreamEvent) + Send + Sync>>,
) {
    let version = {
        let mut cards_guard = cards.lock().await;
        if let Some(card) = cards_guard.get_mut(card_id) {
            card.processing_step = Some(step.to_string());
            let steps = card.processing_steps.get_or_insert_with(Vec::new);
            steps.push(step.to_string());
            card.version += 1;
            card.version
        } else {
            return;
        }
    };

    let event = LifeStreamEvent::CardStep {
        card_id: card_id.to_string(),
        step: step.to_string(),
        version,
    };
    if let Some(app) = emitter {
        let _ = app.emit("life_stream_event", event.clone());
    }
    if let Some(sink) = event_sink {
        sink(event);
    }
}

#[derive(Debug, Clone)]
struct McpToolOutput {
    tool: String,
    text: Option<String>,
    raw: serde_json::Value,
}

async fn maybe_call_mcp_tool(
    mcp_bridge: &LifeMcpBridge,
    card_id: &str,
    card_type: &CardType,
    input: &str,
    cards: &Arc<Mutex<HashMap<String, StreamCard>>>,
    emitter: &Option<tauri::AppHandle>,
    event_sink: &Option<Arc<dyn Fn(LifeStreamEvent) + Send + Sync>>,
) -> Option<McpToolOutput> {
    if !mcp_bridge.is_enabled() {
        return None;
    }

    let (tool, params) = mcp_tool_for_card(card_type, input)?;

    emit_step(card_id, "Syncing with life-mcp...", cards, emitter, event_sink).await;

    let Ok(result) = mcp_bridge.call_tool(&tool, params).await else {
        return None;
    };
    let Some(raw) = result else {
        return None;
    };

    let text = extract_mcp_text(&raw);

    Some(McpToolOutput { tool, text, raw })
}

fn mcp_tool_for_card(
    card_type: &CardType,
    input: &str,
) -> Option<(String, serde_json::Value)> {
    match card_type {
        CardType::Meal => {
            let mut payload = serde_json::json!({ "input": input });
            if let Some(meal_type) = infer_meal_type(input) {
                if let Some(obj) = payload.as_object_mut() {
                    obj.insert("meal_type".to_string(), serde_json::json!(meal_type));
                }
            }
            Some(("log_meal_quick".to_string(), payload))
        }
        CardType::DeliveryOrder => Some((
            "advise_order".to_string(),
            serde_json::json!({ "stt_text": input, "format": "text" }),
        )),
        CardType::MediaAdd => {
            let title = parse_media_title(input);
            if title.is_empty() {
                return None;
            }
            let mut payload = serde_json::json!({
                "title": title,
                "type": infer_media_type(input),
            });
            if let Some(rating) = parse_media_rating(input) {
                if let Some(obj) = payload.as_object_mut() {
                    obj.insert("rating".to_string(), serde_json::json!(rating));
                }
            }
            Some(("media_add".to_string(), payload))
        }
        _ => None,
    }
}

fn infer_meal_type(input: &str) -> Option<&'static str> {
    let lower = input.to_lowercase();
    if lower.contains("breakfast") {
        Some("breakfast")
    } else if lower.contains("lunch") {
        Some("lunch")
    } else if lower.contains("dinner") {
        Some("dinner")
    } else if lower.contains("snack") {
        Some("snack")
    } else {
        None
    }
}

fn parse_media_title(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|word| {
            let lower = word.to_lowercase();
            !matches!(
                lower.as_str(),
                "movie"
                    | "film"
                    | "show"
                    | "series"
                    | "tv"
                    | "anime"
                    | "game"
                    | "book"
                    | "watched"
                    | "played"
                    | "read"
                    | "rating"
            )
        })
        .filter(|word| !word.parse::<u8>().map(|n| n <= 10).unwrap_or(false))
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_media_rating(input: &str) -> Option<u8> {
    let lower = input.to_lowercase();
    let rating_re = Regex::new(r"(\\d+)\\s*(?:/\\s*10)?").ok()?;
    rating_re
        .captures(&lower)
        .and_then(|c| c.get(1)?.as_str().parse::<u8>().ok())
        .filter(|&rating| rating <= 10)
}

fn infer_media_type(input: &str) -> &'static str {
    let lower = input.to_lowercase();
    if lower.contains("anime") {
        "anime"
    } else if lower.contains("animation") {
        "animation"
    } else if lower.contains("comic") {
        "comic"
    } else if lower.contains("youtube") {
        "youtube"
    } else if lower.contains("book") || lower.contains("read") {
        "book"
    } else if lower.contains("game") || lower.contains("played") {
        "game"
    } else if lower.contains("show") || lower.contains("series") || lower.contains("tv") {
        "tv"
    } else {
        "film"
    }
}

fn extract_mcp_text(value: &serde_json::Value) -> Option<String> {
    if let Some(text) = value.as_str() {
        return Some(text.to_string());
    }

    if let Some(text) = value.get("text").and_then(|value| value.as_str()) {
        return Some(text.to_string());
    }

    let content = value.get("content")?;
    if let Some(array) = content.as_array() {
        let mut parts = Vec::new();
        for item in array {
            if let Some(text) = item.get("text").and_then(|value| value.as_str()) {
                parts.push(text.to_string());
            }
        }
        if !parts.is_empty() {
            return Some(parts.join("\n"));
        }
    } else if let Some(text) = content.get("text").and_then(|value| value.as_str()) {
        return Some(text.to_string());
    }

    None
}

fn apply_mcp_output(enriched: &mut EnrichedData, output: McpToolOutput, input: &str) {
    let text = output
        .text
        .unwrap_or_else(|| output.raw.to_string());
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }

    if enriched.summary.is_none() {
        enriched.summary = Some(truncate_summary(trimmed, 160));
    }

    let section = ExpandedSection {
        title: format!("life-mcp ({})", output.tool),
        body: text,
    };

    let expanded = enriched
        .expanded
        .get_or_insert_with(|| base_expanded(input));
    expanded.sections.push(section);
}

fn truncate_summary(text: &str, max: usize) -> String {
    let line = text.lines().next().unwrap_or(text);
    if line.chars().count() <= max {
        return line.to_string();
    }
    let truncated: String = line.chars().take(max).collect();
    format!("{truncated}...")
}

pub(crate) fn detect_intent(input: &str) -> (CardType, DomainId, String) {
    let lower = input.to_lowercase();

    let nutrition_keywords = [
        "ate",
        "had",
        "breakfast",
        "lunch",
        "dinner",
        "snack",
        "meal",
        "calories",
        "omelette",
        "eggs",
        "food",
    ];
    if nutrition_keywords.iter().any(|kw| lower.contains(kw)) {
        return (CardType::Meal, DomainId::Nutrition, "🍽️".to_string());
    }

    let delivery_keywords = [
        "order",
        "delivery",
        "shift",
        "doordash",
        "uber",
        "grubhub",
        "instacart",
        "took",
        "declined",
    ];
    if delivery_keywords.iter().any(|kw| lower.contains(kw)) {
        return (
            CardType::DeliveryOrder,
            DomainId::Delivery,
            "🚗".to_string(),
        );
    }

    let media_keywords = [
        "watched",
        "watching",
        "watch",
        "movie",
        "show",
        "anime",
        "film",
        "played",
        "game",
        "read",
        "book",
    ];
    if media_keywords.iter().any(|kw| lower.contains(kw)) {
        return (CardType::MediaAdd, DomainId::Media, "🎬".to_string());
    }

    let code_keywords = [
        "fix",
        "implement",
        "bug",
        "refactor",
        "add feature",
        "create",
        "update",
        "delete",
        "modify",
        "change",
        "write code",
        "code task",
        "programming",
    ];
    let tech_terms = [
        "function",
        "component",
        "file",
        "module",
        "class",
        "api",
        "endpoint",
        "database",
        "query",
        "rust",
        "react",
        "typescript",
        "swift",
        ".rs",
        ".ts",
        ".tsx",
        ".swift",
    ];
    if code_keywords.iter().any(|kw| lower.contains(kw))
        && tech_terms.iter().any(|term| lower.contains(term))
    {
        return (CardType::CodeTask, DomainId::General, "💻".to_string());
    }

    if lower.ends_with('?') || lower.starts_with("what ") || lower.starts_with("how ") {
        return (CardType::Query, DomainId::General, "🔍".to_string());
    }

    if lower.contains("thought")
        || lower.contains("thinking")
        || lower.contains("think")
        || lower.contains("idea")
        || lower.contains("feeling")
    {
        return (CardType::Thought, DomainId::General, "💭".to_string());
    }

    (CardType::Generic, DomainId::General, "📝".to_string())
}

pub(crate) fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }

    let mut iter = s.chars();
    let truncated: String = iter.by_ref().take(max).collect();

    if truncated.chars().count() < s.chars().count() {
        truncated
    } else {
        s.to_string()
    }
}

async fn handle_nutrition(
    card_id: &str,
    input: &str,
    occurred_at: &str,
    workspace_path: &str,
    obsidian_root: Option<&str>,
    clarification: Option<&str>,
    obsidian: &ObsidianIO,
    cards: &Arc<Mutex<HashMap<String, StreamCard>>>,
    emitter: &Option<tauri::AppHandle>,
    event_sink: &Option<Arc<dyn Fn(LifeStreamEvent) + Send + Sync>>,
) -> Result<Option<EnrichedData>, String> {
    let root = obsidian
        .resolve_root_path(workspace_path, obsidian_root)
        .map_err(|err| err.to_string())?;
    let handler = NutritionHandler::new(&root.to_string_lossy());
    let processed = handler.process(input, occurred_at).await?;

    if processed.foods.is_empty() {
        match clarification {
            Some("skip") => {
                return Ok(Some(EnrichedData {
                    title: processed.title,
                    subtitle: processed.subtitle,
                    summary: Some("Logged without nutrition details.".to_string()),
                    stats: None,
                    entities: None,
                    image: None,
                    expanded: Some(base_expanded(input)),
                    image_lookup: None,
                }));
            }
            Some("add") | Some("photo") => {
                return Ok(Some(EnrichedData {
                    title: processed.title,
                    subtitle: processed.subtitle,
                    summary: Some("Needs manual nutrition entry.".to_string()),
                    stats: None,
                    entities: None,
                    image: Some(CardImage {
                        url: None,
                        status: ImageStatus::UploadPrompt,
                        source: None,
                    }),
                    expanded: Some(base_expanded(input)),
                    image_lookup: None,
                }));
            }
            _ => {
                request_clarification(
                    card_id,
                    "I couldn't find that food. What would you like to do?",
                    vec![
                        ClarificationOption {
                            id: "add".into(),
                            label: "Add new food".into(),
                            emoji: Some("➕".into()),
                        },
                        ClarificationOption {
                            id: "photo".into(),
                            label: "Upload photo".into(),
                            emoji: Some("📷".into()),
                        },
                        ClarificationOption {
                            id: "skip".into(),
                            label: "Log without nutrition".into(),
                            emoji: Some("⏭️".into()),
                        },
                    ],
                    cards,
                    emitter,
                    event_sink,
                )
                .await;
                return Ok(None);
            }
        }
    }

    Ok(Some(EnrichedData {
        title: processed.title,
        subtitle: processed.subtitle,
        summary: None,
        stats: processed.stats,
        entities: processed.entities,
        image: Some(CardImage {
            url: None,
            status: ImageStatus::UploadPrompt,
            source: None,
        }),
        expanded: Some(ExpandedContent {
            original_input: Some(input.to_string()),
            sections: Vec::new(),
            entity_links: None,
            actions: Vec::new(),
        }),
        image_lookup: None,
    }))
}

async fn request_clarification(
    card_id: &str,
    message: &str,
    options: Vec<ClarificationOption>,
    cards: &Arc<Mutex<HashMap<String, StreamCard>>>,
    emitter: &Option<tauri::AppHandle>,
    event_sink: &Option<Arc<dyn Fn(LifeStreamEvent) + Send + Sync>>,
) {
    let _ = emit_patch(
        card_id,
        StreamCardPatch {
            state: Some(CardState::AwaitingInput),
            processing_step: Some(message.to_string()),
            clarification_options: Some(options),
            ..Default::default()
        },
        cards,
        emitter,
        event_sink,
    )
    .await;
}

async fn handle_generic(input: &str) -> Result<EnrichedData, String> {
    Ok(EnrichedData {
        title: truncate(input, 50),
        subtitle: None,
        summary: None,
        stats: None,
        entities: None,
        image: None,
        expanded: Some(base_expanded(input)),
        image_lookup: None,
    })
}

async fn handle_delivery(
    input: &str,
    occurred_at: &str,
    workspace_path: &str,
    obsidian_root: Option<&str>,
    obsidian: &ObsidianIO,
) -> Result<EnrichedData, String> {
    let root = obsidian
        .resolve_root_path(workspace_path, obsidian_root)
        .map_err(|err| err.to_string())?;
    let handler = DeliveryHandler::new(&root.to_string_lossy());
    let processed = handler.process(input, occurred_at).await?;

    let entity_links = processed.entities.as_ref().map(|entities| {
        entities
            .iter()
            .map(|entity| EntityLink {
                name: entity.name.clone(),
                path: entity
                    .link
                    .clone()
                    .unwrap_or_else(|| format!("[[Entities/Delivery/{}]]", entity.name)),
                icon: Some("🚗".to_string()),
            })
            .collect::<Vec<_>>()
    });

    Ok(EnrichedData {
        title: processed.title,
        subtitle: processed.subtitle,
        summary: processed.summary,
        stats: processed.stats,
        entities: processed.entities,
        image: None,
        expanded: Some(ExpandedContent {
            original_input: Some(input.to_string()),
            sections: Vec::new(),
            entity_links,
            actions: Vec::new(),
        }),
        image_lookup: None,
    })
}

async fn handle_media(
    input: &str,
    occurred_at: &str,
    workspace_path: &str,
    obsidian_root: Option<&str>,
    obsidian: &ObsidianIO,
) -> Result<EnrichedData, String> {
    let root = obsidian
        .resolve_root_path(workspace_path, obsidian_root)
        .map_err(|err| err.to_string())?;
    let handler = MediaHandler::new(&root.to_string_lossy());
    let processed = handler.process(input, occurred_at).await?;
    let title = processed.title.clone();

    let entity_links = processed.entities.as_ref().map(|entities| {
        entities
            .iter()
            .map(|entity| EntityLink {
                name: entity.name.clone(),
                path: entity
                    .link
                    .clone()
                    .unwrap_or_else(|| format!("[[Media/{}]]", entity.name)),
                icon: Some("🎬".to_string()),
            })
            .collect::<Vec<_>>()
    });

    let image_lookup = if title.is_empty() {
        None
    } else {
        Some(ImageLookup {
            card_type: CardType::MediaAdd,
            entity_name: title.clone(),
        })
    };

    let image = image_lookup.as_ref().map(|_| CardImage {
        url: None,
        status: ImageStatus::Loading,
        source: None,
    });

    Ok(EnrichedData {
        title: processed.title,
        subtitle: processed.subtitle,
        summary: processed.summary,
        stats: processed.stats,
        entities: processed.entities,
        image,
        expanded: Some(ExpandedContent {
            original_input: Some(input.to_string()),
            sections: Vec::new(),
            entity_links,
            actions: Vec::new(),
        }),
        image_lookup,
    })
}

async fn handle_thought(input: &str) -> Result<EnrichedData, String> {
    let handler = ThoughtHandler::new();
    let processed = handler.process(input).await?;

    let expanded = ExpandedContent {
        original_input: Some(input.to_string()),
        sections: if input.trim().is_empty() {
            Vec::new()
        } else {
            vec![ExpandedSection {
                title: "Full Note".to_string(),
                body: input.to_string(),
            }]
        },
        entity_links: None,
        actions: Vec::new(),
    };

    Ok(EnrichedData {
        title: processed.title,
        subtitle: processed.subtitle,
        summary: processed.summary,
        stats: processed.stats,
        entities: processed.entities,
        image: None,
        expanded: Some(expanded),
        image_lookup: None,
    })
}

async fn handle_query(input: &str) -> Result<EnrichedData, String> {
    let handler = QueryHandler::new();
    let processed = handler.process(input).await?;

    Ok(EnrichedData {
        title: processed.title,
        subtitle: processed.subtitle,
        summary: processed.summary,
        stats: processed.stats,
        entities: processed.entities,
        image: None,
        expanded: Some(base_expanded(input)),
        image_lookup: None,
    })
}

async fn handle_code_task(input: &str) -> Result<EnrichedData, String> {
    let handler = CodeTaskHandler::new();
    let processed = handler.process(input).await?;

    Ok(EnrichedData {
        title: processed.title,
        subtitle: processed.subtitle,
        summary: processed.summary,
        stats: processed.stats,
        entities: processed.entities,
        image: None,
        expanded: Some(ExpandedContent {
            original_input: Some(input.to_string()),
            sections: Vec::new(),
            entity_links: None,
            actions: vec![
                CardAction {
                    id: "view_thread".to_string(),
                    label: "View Thread".to_string(),
                    icon: Some("🔗".to_string()),
                    style: Some("primary".to_string()),
                },
                CardAction {
                    id: "cancel".to_string(),
                    label: "Cancel".to_string(),
                    icon: Some("✕".to_string()),
                    style: Some("danger".to_string()),
                },
            ],
        }),
        image_lookup: None,
    })
}

fn base_expanded(input: &str) -> ExpandedContent {
    ExpandedContent {
        original_input: Some(input.to_string()),
        sections: Vec::new(),
        entity_links: None,
        actions: Vec::new(),
    }
}

pub(crate) struct EnrichedData {
    pub(crate) title: String,
    pub(crate) subtitle: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) stats: Option<HashMap<String, CardStatValue>>,
    pub(crate) entities: Option<Vec<EntityRef>>,
    pub(crate) image: Option<CardImage>,
    pub(crate) expanded: Option<ExpandedContent>,
    pub(crate) image_lookup: Option<ImageLookup>,
}

async fn is_cancelled(card_id: &str, cancelled_cards: &Arc<Mutex<HashSet<String>>>) -> bool {
    let guard = cancelled_cards.lock().await;
    guard.contains(card_id)
}

#[derive(Debug, Clone)]
pub(crate) struct ImageLookup {
    pub(crate) card_type: CardType,
    pub(crate) entity_name: String,
}

fn card_type_label(card_type: &CardType) -> &'static str {
    match card_type {
        CardType::Meal => "meal",
        CardType::DeliveryOrder => "delivery",
        CardType::MediaAdd => "media",
        CardType::Music => "media",
        CardType::Thought => "thought",
        CardType::Query => "query",
        CardType::CodeTask => "code_task",
        CardType::Generic => "generic",
    }
}
