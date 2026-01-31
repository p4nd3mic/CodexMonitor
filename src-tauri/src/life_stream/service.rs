use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, Semaphore};
use tauri::Emitter;

use super::obsidian::ObsidianIO;
use super::types::*;

pub struct LifeStreamService {
    cards: Arc<Mutex<HashMap<String, StreamCard>>>,
    worker_semaphore: Arc<Semaphore>,
    write_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
    obsidian: ObsidianIO,
    emitter: Option<tauri::AppHandle>,
}

impl LifeStreamService {
    pub fn new(obsidian_root: Option<String>) -> Self {
        Self {
            cards: Arc::new(Mutex::new(HashMap::new())),
            worker_semaphore: Arc::new(Semaphore::new(5)),
            write_locks: Arc::new(Mutex::new(HashMap::new())),
            obsidian: ObsidianIO::new(obsidian_root),
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

        let card = StreamCard {
            id: card_id.to_string(),
            occurred_at: occurred.clone(),
            created_at: now.clone(),
            updated_at: now.clone(),
            version: 1,
            card_type: CardType::Generic,
            domain: DomainId::General,
            emoji: "📝".to_string(),
            state: CardState::Processing,
            processing_step: Some("Parsing input...".to_string()),
            processing_steps: Some(vec!["Parsing input...".to_string()]),
            title: truncate(input, 50),
            subtitle: None,
            summary: None,
            image: None,
            stats: None,
            entities: None,
            original_input: Some(input.to_string()),
            source: None,
            expanded: None,
            error_message: None,
        };

        {
            let mut cards = self.cards.lock().await;
            cards.insert(card_id.to_string(), card.clone());
        }

        self.emit_event(LifeStreamEvent::CardCreated { card: card.clone() });

        let card_id = card_id.to_string();
        let workspace_id = workspace_id.to_string();
        let workspace_path = workspace_path.to_string();
        let obsidian_root = obsidian_root.map(|value| value.to_string());
        let input = input.to_string();
        let cards = Arc::clone(&self.cards);
        let semaphore = Arc::clone(&self.worker_semaphore);
        let write_locks = Arc::clone(&self.write_locks);
        let obsidian = self.obsidian.clone();
        let emitter = self.emitter.clone();

        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let lock = {
                let mut locks = write_locks.lock().await;
                locks
                    .entry(workspace_id.clone())
                    .or_insert_with(|| Arc::new(Mutex::new(())))
                    .clone()
            };

            let result = process_card(
                &card_id,
                workspace_path.as_str(),
                obsidian_root.as_deref(),
                &input,
                &occurred,
                &cards,
                &lock,
                &obsidian,
                &emitter,
            )
            .await;

            if let Err(e) = result {
                let mut cards_guard = cards.lock().await;
                if let Some(card) = cards_guard.get_mut(&card_id) {
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

        Ok(())
    }

    fn emit_event(&self, event: LifeStreamEvent) {
        if let Some(app) = &self.emitter {
            let _ = app.emit("life_stream_event", event);
        }
    }
}

async fn process_card(
    card_id: &str,
    workspace_path: &str,
    obsidian_root: Option<&str>,
    input: &str,
    occurred_at: &str,
    cards: &Arc<Mutex<HashMap<String, StreamCard>>>,
    write_lock: &Arc<Mutex<()>>,
    obsidian: &ObsidianIO,
    emitter: &Option<tauri::AppHandle>,
) -> Result<(), String> {
    emit_step(card_id, "Detecting intent...", cards, emitter).await;
    let (card_type, domain, emoji) = detect_intent(input);

    emit_step(
        card_id,
        &format!("Processing as {:?}...", domain),
        cards,
        emitter,
    )
    .await;

    let enriched = match domain {
        DomainId::Nutrition => handle_nutrition(input, occurred_at).await?,
        _ => handle_generic(input).await?,
    };

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
            .await?;
    }

    let mut cards_guard = cards.lock().await;
    if let Some(card) = cards_guard.get_mut(card_id) {
        card.state = CardState::Complete;
        card.card_type = card_type;
        card.domain = domain;
        card.emoji = emoji;
        card.title = enriched.title;
        card.subtitle = enriched.subtitle;
        card.summary = enriched.summary;
        card.stats = enriched.stats;
        card.entities = enriched.entities;
        card.version += 1;

        if let Some(app) = emitter {
            let _ = app.emit(
                "life_stream_event",
                LifeStreamEvent::CardCompleted { card: card.clone() },
            );
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
            if let Some(ref mut steps) = card.processing_steps {
                steps.push(step.to_string());
            }
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

fn detect_intent(input: &str) -> (CardType, DomainId, String) {
    let lower = input.to_lowercase();

    if lower.contains("ate")
        || lower.contains("had")
        || lower.contains("breakfast")
        || lower.contains("lunch")
        || lower.contains("dinner")
        || lower.contains("snack")
        || lower.contains("meal")
        || lower.contains("calories")
        || lower.contains("omelette")
        || lower.contains("eggs")
    {
        return (CardType::Meal, DomainId::Nutrition, "🍽️".to_string());
    }

    if lower.contains("order")
        || lower.contains("delivery")
        || lower.contains("shift")
        || lower.contains("doordash")
    {
        return (CardType::DeliveryOrder, DomainId::Delivery, "🚗".to_string());
    }

    if lower.contains("watched") || lower.contains("movie") || lower.contains("show") {
        return (CardType::MediaAdd, DomainId::Media, "🎬".to_string());
    }

    (CardType::Generic, DomainId::General, "📝".to_string())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

async fn handle_nutrition(input: &str, occurred_at: &str) -> Result<EnrichedData, String> {
    let time_label = occurred_at.get(11..16).unwrap_or("??:??");
    let title = format!("Meal at {}", time_label);
    Ok(EnrichedData {
        title,
        subtitle: Some(input.to_string()),
        summary: None,
        stats: None,
        entities: None,
    })
}

async fn handle_generic(input: &str) -> Result<EnrichedData, String> {
    Ok(EnrichedData {
        title: truncate(input, 50),
        subtitle: None,
        summary: None,
        stats: None,
        entities: None,
    })
}

pub(crate) struct EnrichedData {
    pub(crate) title: String,
    pub(crate) subtitle: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) stats: Option<HashMap<String, serde_json::Value>>,
    pub(crate) entities: Option<Vec<EntityRef>>,
}
