mod events;
mod handlers;
mod images;
mod mcp_bridge;
mod obsidian;
mod service;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod service_test;
mod types;

pub use service::LifeStreamService;
pub use types::*;

use serde_json::json;
use tauri::{AppHandle, State};

use crate::remote_backend;
use crate::state::AppState;

#[tauri::command]
pub async fn life_stream_load_day(
    workspace_id: String,
    date_iso: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<Vec<StreamCard>, String> {
    if remote_backend::is_remote_mode(&*state).await {
        let response = remote_backend::call_remote(
            &*state,
            app,
            "life_stream_load_day",
            json!({ "workspaceId": workspace_id, "dateIso": date_iso }),
        )
        .await?;
        return serde_json::from_value(response).map_err(|err| err.to_string());
    }

    let workspaces = state.workspaces.lock().await;
    let entry = workspaces.get(&workspace_id).ok_or("workspace not found")?;
    let obsidian_root = entry.settings.obsidian_root.as_deref();

    let life_stream = state.life_stream_service.lock().await;
    life_stream
        .load_day(&entry.path, obsidian_root, &date_iso)
        .await
}

#[tauri::command]
pub async fn life_stream_submit(
    workspace_id: String,
    card_id: String,
    input: String,
    occurred_at_iso: Option<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    if remote_backend::is_remote_mode(&*state).await {
        remote_backend::call_remote(
            &*state,
            app,
            "life_stream_submit",
            json!({
                "workspaceId": workspace_id,
                "cardId": card_id,
                "input": input,
                "occurredAtIso": occurred_at_iso,
            }),
        )
        .await?;
        return Ok(());
    }

    let workspaces = state.workspaces.lock().await;
    let entry = workspaces.get(&workspace_id).ok_or("workspace not found")?;
    let obsidian_root = entry.settings.obsidian_root.as_deref();

    let life_stream = state.life_stream_service.lock().await;
    life_stream
        .submit(
            &workspace_id,
            &entry.path,
            obsidian_root,
            &card_id,
            &input,
            occurred_at_iso.as_deref(),
        )
        .await
}

#[tauri::command]
pub async fn life_stream_cancel(
    workspace_id: String,
    card_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    if remote_backend::is_remote_mode(&*state).await {
        remote_backend::call_remote(
            &*state,
            app,
            "life_stream_cancel",
            json!({ "workspaceId": workspace_id, "cardId": card_id }),
        )
        .await?;
        return Ok(());
    }

    let life_stream = state.life_stream_service.lock().await;
    life_stream.cancel(&card_id).await
}

#[tauri::command]
pub async fn life_stream_retry(
    workspace_id: String,
    card_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    if remote_backend::is_remote_mode(&*state).await {
        remote_backend::call_remote(
            &*state,
            app,
            "life_stream_retry",
            json!({ "workspaceId": workspace_id, "cardId": card_id }),
        )
        .await?;
        return Ok(());
    }

    let workspaces = state.workspaces.lock().await;
    let entry = workspaces.get(&workspace_id).ok_or("workspace not found")?;
    let obsidian_root = entry.settings.obsidian_root.as_deref();

    let life_stream = state.life_stream_service.lock().await;
    life_stream
        .retry(&workspace_id, &entry.path, obsidian_root, &card_id)
        .await
}

#[tauri::command]
pub async fn life_stream_clarify(
    workspace_id: String,
    card_id: String,
    option_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    if remote_backend::is_remote_mode(&*state).await {
        remote_backend::call_remote(
            &*state,
            app,
            "life_stream_clarify",
            json!({ "workspaceId": workspace_id, "cardId": card_id, "optionId": option_id }),
        )
        .await?;
        return Ok(());
    }

    let workspaces = state.workspaces.lock().await;
    let entry = workspaces.get(&workspace_id).ok_or("workspace not found")?;
    let obsidian_root = entry.settings.obsidian_root.as_deref();

    let life_stream = state.life_stream_service.lock().await;
    life_stream
        .resume_with_clarification(
            &workspace_id,
            &entry.path,
            obsidian_root,
            &card_id,
            &option_id,
        )
        .await
}
