use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use regex::Regex;
use tauri::Emitter;
use tokio::sync::{Mutex, Semaphore};

use super::handlers::nutrition::NutritionHandler;
use super::images::ImageService;
use super::obsidian::ObsidianIO;
use super::types::*;

pub struct LifeStreamService {
    cards: Arc<Mutex<HashMap<String, StreamCard>>>,
    worker_semaphore: Arc<Semaphore>,
    write_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
    cancelled_cards: Arc<Mutex<HashSet<String>>>,
    obsidian: ObsidianIO,
    tmdb_api_key: Option<String>,
    emitter: Option<tauri::AppHandle>,
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
            emitter: None,
        }
    }

    pub fn set_emitter(&mut self, app: tauri::AppHandle) {
        self.emitter = Some(app);
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
            let _ = app.emit("life_stream_event", event);
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
        let cancelled_cards = Arc::clone(&self.cancelled_cards);
        let clarification = clarification.clone();

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
                &cancelled_cards,
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

                    if let Some(app) = &emitter {
                        let _ = app.emit(
                            "life_stream_event",
                            LifeStreamEvent::CardError {
                                card_id: card_id.clone(),
                                message: e,
                                version: card.version,
                            },
                        );
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
) -> Option<u32> {
    let version = {
        let mut cards_guard = cards.lock().await;
        let card = cards_guard.get_mut(card_id)?;
        apply_patch_to_card(card, &patch);
        card.version += 1;
        card.updated_at = chrono::Utc::now().to_rfc3339();
        card.version
    };

    if let Some(app) = emitter {
        let _ = app.emit(
            "life_stream_event",
            LifeStreamEvent::CardUpdated {
                card_id: card_id.to_string(),
                patch,
                version,
            },
        );
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
        let mut mapped = HashMap::new();
        for (key, value) in stats {
            mapped.insert(key.clone(), card_stat_value_to_json(value.clone()));
        }
        card.stats = Some(mapped);
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

fn card_stat_value_to_json(value: CardStatValue) -> serde_json::Value {
    match value {
        CardStatValue::String(value) => serde_json::json!(value),
        CardStatValue::Integer(value) => serde_json::json!(value),
        CardStatValue::Float(value) => serde_json::json!(value),
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
    cancelled_cards: &Arc<Mutex<HashSet<String>>>,
    tmdb_api_key: Option<&str>,
) -> Result<(), String> {
    if is_cancelled(card_id, cancelled_cards).await {
        return Ok(());
    }
    emit_step(card_id, "Detecting intent...", cards, emitter).await;
    let (card_type, domain, emoji) = detect_intent(input);

    if is_cancelled(card_id, cancelled_cards).await {
        return Ok(());
    }
    emit_step(
        card_id,
        &format!("Processing as {:?}...", domain),
        cards,
        emitter,
    )
    .await;

    if is_cancelled(card_id, cancelled_cards).await {
        return Ok(());
    }

    let enriched = match card_type {
        CardType::Meal => {
            emit_step(card_id, "Looking up nutrition...", cards, emitter).await;
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
            )
            .await?
            {
                Some(value) => value,
                None => return Ok(()),
            }
        }
        CardType::DeliveryOrder => handle_delivery(input).await?,
        CardType::MediaAdd => handle_media(input).await?,
        CardType::Thought => handle_thought(input).await?,
        CardType::Query => handle_query(input).await?,
        CardType::CodeTask => handle_code_task(input).await?,
        _ => handle_generic(input).await?,
    };

    if is_cancelled(card_id, cancelled_cards).await {
        return Ok(());
    }
    emit_step(card_id, "Saving...", cards, emitter).await;
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

        if let Some(app) = emitter {
            let _ = app.emit(
                "life_stream_event",
                LifeStreamEvent::CardCompleted { card: card.clone() },
            );
        }
    }

    if let Some(lookup) = enriched.image_lookup {
        if let Some(root) = obsidian_root {
            let cards = Arc::clone(cards);
            let emitter = emitter.clone();
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

    if let Some(app) = emitter {
        let _ = app.emit(
            "life_stream_event",
            LifeStreamEvent::CardStep {
                card_id: card_id.to_string(),
                step: step.to_string(),
                version,
            },
        );
    }
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
        "watched", "movie", "show", "anime", "film", "played", "game", "read", "book",
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

    if lower.contains("thought") || lower.contains("idea") || lower.contains("feeling") {
        return (CardType::Thought, DomainId::General, "💭".to_string());
    }

    (CardType::Generic, DomainId::General, "📝".to_string())
}

pub(crate) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }

    let end = s
        .char_indices()
        .take_while(|(i, _)| *i < max)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);

    format!("{}...", &s[..end])
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

async fn handle_delivery(input: &str) -> Result<EnrichedData, String> {
    let amount_re = Regex::new(r"\\$?(\\d+\\.?\\d*)").map_err(|e| e.to_string())?;
    let mile_re = Regex::new(r"(\\d+\\.?\\d*)\\s*(?:mi|miles?)").map_err(|e| e.to_string())?;
    let tip_re = Regex::new(r"tip\\s*\\$?(\\d+\\.?\\d*)").map_err(|e| e.to_string())?;

    let amount = amount_re
        .captures(input)
        .and_then(|c| c.get(1)?.as_str().parse::<f64>().ok())
        .unwrap_or(0.0);
    let mileage = mile_re
        .captures(input)
        .and_then(|c| c.get(1)?.as_str().parse::<f64>().ok())
        .unwrap_or(0.0);
    let tip = tip_re
        .captures(input)
        .and_then(|c| c.get(1)?.as_str().parse::<f64>().ok())
        .unwrap_or(0.0);
    let per_mile = if mileage > 0.0 { amount / mileage } else { 0.0 };

    let merchant = input
        .split_whitespace()
        .next()
        .filter(|word| {
            !word
                .chars()
                .next()
                .map(|c| c.is_numeric() || c == '$')
                .unwrap_or(false)
        })
        .map(|s| s.to_string());

    let rating = if per_mile >= 2.5 {
        "🟢 Great"
    } else if per_mile >= 2.0 {
        "🟡 Good"
    } else if per_mile >= 1.5 {
        "🟠 OK"
    } else {
        "🔴 Low"
    };

    let mut stats = HashMap::new();
    stats.insert("earnings".to_string(), serde_json::json!(amount));
    stats.insert("mileage".to_string(), serde_json::json!(mileage));
    stats.insert(
        "perMile".to_string(),
        serde_json::json!(format!("${:.2}/mi", per_mile)),
    );
    stats.insert("tip".to_string(), serde_json::json!(tip));
    stats.insert("rating".to_string(), serde_json::json!(rating));

    Ok(EnrichedData {
        title: merchant.unwrap_or_else(|| "Delivery".to_string()),
        subtitle: Some(format!("${:.2} • {:.1} mi", amount, mileage)),
        summary: None,
        stats: Some(stats),
        entities: None,
        image: None,
        expanded: Some(base_expanded(input)),
        image_lookup: None,
    })
}

async fn handle_media(input: &str) -> Result<EnrichedData, String> {
    let lower = input.to_lowercase();
    let rating_re = Regex::new(r"(\\d+)\\s*(?:/\\s*10)?").map_err(|e| e.to_string())?;
    let rating = rating_re
        .captures(&lower)
        .and_then(|c| c.get(1)?.as_str().parse::<u8>().ok())
        .filter(|&r| r <= 10);

    let title = input
        .split_whitespace()
        .filter(|word| {
            let lower = word.to_lowercase();
            !matches!(
                lower.as_str(),
                "movie"
                    | "film"
                    | "show"
                    | "series"
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
        .join(" ");

    let title = if title.is_empty() {
        truncate(input, 50)
    } else {
        title
    };

    let mut stats = HashMap::new();
    if let Some(rating) = rating {
        stats.insert(
            "rating".to_string(),
            serde_json::json!(format!("{}/10", rating)),
        );
    }

    let expanded = ExpandedContent {
        original_input: Some(input.to_string()),
        sections: Vec::new(),
        entity_links: Some(vec![EntityLink {
            name: title.clone(),
            path: format!("[[Media/{}]]", title),
            icon: Some("🎬".to_string()),
        }]),
        actions: Vec::new(),
    };

    Ok(EnrichedData {
        title: title.clone(),
        subtitle: None,
        summary: None,
        stats: if stats.is_empty() { None } else { Some(stats) },
        entities: Some(vec![EntityRef {
            entity_type: "media".to_string(),
            id: None,
            name: title.clone(),
            link: Some(format!("[[Media/{}]]", title)),
        }]),
        image: Some(CardImage {
            url: None,
            status: ImageStatus::Loading,
            source: None,
        }),
        expanded: Some(expanded),
        image_lookup: Some(ImageLookup {
            card_type: CardType::MediaAdd,
            entity_name: title,
        }),
    })
}

async fn handle_thought(input: &str) -> Result<EnrichedData, String> {
    let word_count = input.split_whitespace().count() as i64;
    let hashtag_re = Regex::new(r"#(\\w+)").map_err(|e| e.to_string())?;
    let wiki_re = Regex::new(r"\\[\\[([^\\]]+)\\]\\]").map_err(|e| e.to_string())?;

    let mut topics = Vec::new();
    for cap in hashtag_re.captures_iter(input) {
        if let Some(m) = cap.get(1) {
            topics.push(m.as_str().to_string());
        }
    }
    for cap in wiki_re.captures_iter(input) {
        if let Some(m) = cap.get(1) {
            topics.push(m.as_str().to_string());
        }
    }

    let mut stats = HashMap::new();
    stats.insert("words".to_string(), serde_json::json!(word_count));

    let expanded = ExpandedContent {
        original_input: Some(input.to_string()),
        sections: vec![ExpandedSection {
            title: "Full Note".to_string(),
            body: input.to_string(),
        }],
        entity_links: if topics.is_empty() {
            None
        } else {
            Some(
                topics
                    .iter()
                    .map(|topic| EntityLink {
                        name: topic.clone(),
                        path: format!("[[Topics/{}]]", topic),
                        icon: Some("🏷️".to_string()),
                    })
                    .collect(),
            )
        },
        actions: Vec::new(),
    };

    Ok(EnrichedData {
        title: truncate(input, 50),
        subtitle: if topics.is_empty() {
            None
        } else {
            Some(topics.join(" • "))
        },
        summary: None,
        stats: Some(stats),
        entities: None,
        image: None,
        expanded: Some(expanded),
        image_lookup: None,
    })
}

async fn handle_query(input: &str) -> Result<EnrichedData, String> {
    Ok(EnrichedData {
        title: truncate(input, 60),
        subtitle: Some("Searching...".to_string()),
        summary: None,
        stats: None,
        entities: None,
        image: None,
        expanded: Some(base_expanded(input)),
        image_lookup: None,
    })
}

async fn handle_code_task(input: &str) -> Result<EnrichedData, String> {
    let summary = truncate(input, 60);

    Ok(EnrichedData {
        title: summary,
        subtitle: Some("Queued for Codex".to_string()),
        summary: None,
        stats: None,
        entities: None,
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
    pub(crate) stats: Option<HashMap<String, serde_json::Value>>,
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
