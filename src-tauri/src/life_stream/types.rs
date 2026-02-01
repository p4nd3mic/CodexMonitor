use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CardState {
    Pending,
    Processing,
    AwaitingInput,
    Complete,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CardType {
    Meal,
    DeliveryOrder,
    MediaAdd,
    Music,
    Thought,
    Query,
    CodeTask,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DomainId {
    Nutrition,
    Delivery,
    Media,
    Youtube,
    Finance,
    Fitness,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ImageStatus {
    Loading,
    Ready,
    Missing,
    UploadPrompt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardImage {
    pub url: Option<String>,
    pub status: ImageStatus,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRef {
    #[serde(rename = "type")]
    pub entity_type: String,
    pub id: Option<String>,
    pub name: String,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandedSection {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardAction {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandedContent {
    #[serde(rename = "originalInput")]
    pub original_input: Option<String>,
    pub sections: Vec<ExpandedSection>,
    #[serde(rename = "entityLinks")]
    pub entity_links: Option<Vec<EntityLink>>,
    pub actions: Vec<CardAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityLink {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationOption {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub emoji: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardSource {
    #[serde(rename = "streamFile")]
    pub stream_file: Option<String>,
    #[serde(rename = "streamAnchor")]
    pub stream_anchor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamCard {
    pub id: String,
    #[serde(rename = "occurredAt")]
    pub occurred_at: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: u32,

    #[serde(rename = "cardType")]
    pub card_type: CardType,
    pub domain: DomainId,
    pub emoji: String,

    pub state: CardState,
    #[serde(rename = "processingStep")]
    pub processing_step: Option<String>,
    #[serde(rename = "processingSteps")]
    pub processing_steps: Option<Vec<String>>,

    pub title: String,
    pub subtitle: Option<String>,
    pub summary: Option<String>,

    pub image: Option<CardImage>,

    pub stats: Option<HashMap<String, serde_json::Value>>,
    pub entities: Option<Vec<EntityRef>>,

    #[serde(rename = "originalInput")]
    pub original_input: Option<String>,

    pub source: Option<CardSource>,

    pub expanded: Option<ExpandedContent>,

    #[serde(rename = "clarificationOptions")]
    pub clarification_options: Option<Vec<ClarificationOption>>,

    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
}

// Event types for broadcasting
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LifeStreamEvent {
    CardCreated {
        card: StreamCard,
    },
    CardStep {
        #[serde(rename = "cardId")]
        card_id: String,
        step: String,
        version: u32,
    },
    CardUpdated {
        #[serde(rename = "cardId")]
        card_id: String,
        patch: StreamCardPatch,
        version: u32,
    },
    CardCompleted {
        card: StreamCard,
    },
    CardError {
        #[serde(rename = "cardId")]
        card_id: String,
        message: String,
        version: u32,
    },
}

/// Typed patch for card updates via events.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StreamCardPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<CardState>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "processingStep")]
    pub processing_step: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "processingSteps")]
    pub processing_steps: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "errorMessage")]
    pub error_message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<HashMap<String, CardStatValue>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<CardImage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expanded: Option<ExpandedContent>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "clarificationOptions"
    )]
    pub clarification_options: Option<Vec<ClarificationOption>>,
}

/// Constrained stat value type for sync with TypeScript.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CardStatValue {
    String(String),
    Integer(i64),
    Float(f64),
}

#[derive(Debug, Error)]
pub enum LifeStreamError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Security violation: {0}")]
    Security(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

// Submit input parameters
#[derive(Debug, Clone, Deserialize)]
pub struct SubmitInput {
    #[serde(rename = "workspaceId")]
    pub workspace_id: String,
    #[serde(rename = "cardId")]
    pub card_id: String,
    pub input: String,
    #[serde(rename = "occurredAtIso")]
    pub occurred_at_iso: Option<String>,
}
