use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandedContent {
    pub sections: Vec<ExpandedSection>,
    pub actions: Vec<CardAction>,
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

    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
}

// Event types for broadcasting
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LifeStreamEvent {
    CardCreated { card: StreamCard },
    CardStep {
        #[serde(rename = "cardId")]
        card_id: String,
        step: String,
        version: u32,
    },
    CardUpdated {
        #[serde(rename = "cardId")]
        card_id: String,
        patch: serde_json::Value,
        version: u32,
    },
    CardCompleted { card: StreamCard },
    CardError {
        #[serde(rename = "cardId")]
        card_id: String,
        message: String,
        version: u32,
    },
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
