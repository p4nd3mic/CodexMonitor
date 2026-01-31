use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tokio::sync::{Mutex, RwLock};

use crate::auto_flush::AutoMemoryRuntime;
use crate::dictation::DictationState;
use crate::life_core::default_obsidian_root;
use crate::life_stream::LifeStreamService;
use crate::memory::MemoryService;
use crate::storage::{
    read_domains, read_settings, read_workspaces, seed_domains_from_files, write_domains,
};
use crate::types::{AppSettings, Domain, WorkspaceEntry};

pub(crate) struct AppState {
    pub(crate) workspaces: Mutex<HashMap<String, WorkspaceEntry>>,
    pub(crate) sessions: Mutex<HashMap<String, Arc<crate::codex::WorkspaceSession>>>,
    pub(crate) terminal_sessions: Mutex<HashMap<String, Arc<crate::terminal::TerminalSession>>>,
    pub(crate) remote_backend: Mutex<Option<crate::remote_backend::RemoteBackend>>,
    pub(crate) storage_path: PathBuf,
    pub(crate) settings_path: PathBuf,
    pub(crate) domains_path: PathBuf,
    pub(crate) app_settings: Mutex<AppSettings>,
    pub(crate) domains: Mutex<Vec<Domain>>,
    pub(crate) dictation: Mutex<DictationState>,
    pub(crate) memory: RwLock<Option<MemoryService>>,
    pub(crate) auto_memory_runtime: Mutex<AutoMemoryRuntime>,
    pub(crate) life_stream_service: Mutex<LifeStreamService>,
}

impl AppState {
    pub(crate) fn load(app: &AppHandle) -> Self {
        let data_dir = app
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| ".".into()));
        let storage_path = data_dir.join("workspaces.json");
        let settings_path = data_dir.join("settings.json");
        let domains_path = data_dir.join("domains.json");
        let workspaces = read_workspaces(&storage_path).unwrap_or_default();
        let app_settings = read_settings(&settings_path).unwrap_or_default();
        let mut domains = read_domains(&domains_path).unwrap_or_default();
        if domains.is_empty() {
            let seeded = seed_domains_from_files();
            if !seeded.is_empty() {
                let _ = write_domains(&domains_path, &seeded);
                domains = seeded;
            }
        }
        let memory = if app_settings.memory_enabled
            && !app_settings.supabase_url.is_empty()
            && !app_settings.supabase_anon_key.is_empty()
        {
            Some(MemoryService::new(
                &app_settings.supabase_url,
                &app_settings.supabase_anon_key,
                if app_settings.memory_embedding_enabled {
                    Some(&app_settings.minimax_api_key)
                } else {
                    None
                },
                true,
            ))
        } else {
            None
        };
        let mut life_stream_service = LifeStreamService::new(default_obsidian_root());
        life_stream_service.set_emitter(app.clone());

        Self {
            workspaces: Mutex::new(workspaces),
            sessions: Mutex::new(HashMap::new()),
            terminal_sessions: Mutex::new(HashMap::new()),
            remote_backend: Mutex::new(None),
            storage_path,
            settings_path,
            domains_path,
            app_settings: Mutex::new(app_settings),
            domains: Mutex::new(domains),
            dictation: Mutex::new(DictationState::default()),
            memory: RwLock::new(memory),
            auto_memory_runtime: Mutex::new(AutoMemoryRuntime::default()),
            life_stream_service: Mutex::new(life_stream_service),
        }
    }
}
