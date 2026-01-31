#[path = "../memory/auto_flush.rs"]
mod auto_flush;
#[allow(dead_code)]
#[path = "../backend/mod.rs"]
mod backend;
#[path = "../browser/mod.rs"]
mod browser;
#[path = "../codex_args.rs"]
mod codex_args;
#[path = "../codex_config.rs"]
mod codex_config;
#[path = "../codex_home.rs"]
mod codex_home;
#[path = "../codex_params.rs"]
mod codex_params;
#[path = "../git_utils.rs"]
mod git_utils;
#[path = "../life_core.rs"]
mod life;
#[path = "../local_usage_core.rs"]
mod local_usage_core;
#[path = "../memory/mod.rs"]
mod memory;
#[path = "../obsidian/mod.rs"]
mod obsidian;
#[path = "../rules.rs"]
mod rules;
#[path = "../skills/mod.rs"]
mod skills;
#[path = "../storage.rs"]
mod storage;
#[allow(dead_code)]
#[path = "../types.rs"]
mod types;
#[path = "../utils.rs"]
mod utils;

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use git2::{BranchType, DiffOptions, Repository, Sort, Status, StatusOptions};
use ignore::WalkBuilder;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Command;
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio::task;
use utils::{git_env_path, resolve_git_binary};
use uuid::Uuid;

use auto_flush::{
    build_snapshot, parse_memory_flush_result, run_memory_flush_summarizer, write_memory_flush,
    AutoMemoryRuntime,
};
use backend::app_server::{spawn_workspace_session, WorkspaceSession};
use backend::events::{AppServerEvent, EventSink, TerminalOutput};
use browser::service::BrowserService;
use codex_params::{build_turn_start_params, build_user_input};
use git_utils::{
    checkout_branch, commit_to_entry, diff_patch_to_string, diff_stats_for_path,
    list_git_roots as scan_git_roots, parse_github_repo, resolve_git_root,
};
use memory::MemoryService;
use skills::skill_md::{parse_skill_md, validate_skill};
use storage::{
    read_domains, read_settings, read_workspaces, seed_domains_from_files, write_domains,
    write_settings, write_workspaces,
};
use types::{
    AppSettings, AutoMemorySettings, BranchInfo, Domain, DomainTrendSnapshot, GitCommitDiff,
    GitFileDiff, GitFileStatus, GitHubIssue, GitHubIssuesResponse, GitHubPullRequest,
    GitHubPullRequestComment, GitHubPullRequestDiff, GitHubPullRequestsResponse, GitLogResponse,
    LocalUsageSnapshot, WorkspaceEntry, WorkspaceInfo, WorkspaceKind, WorkspaceSettings,
    WorktreeInfo,
};
use utils::normalize_git_path;

const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:4732";
const INDEX_SKIP_WORKTREE_FLAG: u16 = 0x4000;

#[derive(Clone)]
struct DaemonEventSink {
    tx: broadcast::Sender<DaemonEvent>,
}

#[derive(Clone)]
enum DaemonEvent {
    AppServer(AppServerEvent),
    #[allow(dead_code)]
    TerminalOutput(TerminalOutput),
}

impl EventSink for DaemonEventSink {
    fn emit_app_server_event(&self, event: AppServerEvent) {
        let _ = self.tx.send(DaemonEvent::AppServer(event));
    }

    fn emit_terminal_output(&self, event: TerminalOutput) {
        let _ = self.tx.send(DaemonEvent::TerminalOutput(event));
    }
}

struct DaemonConfig {
    listen: SocketAddr,
    token: Option<String>,
    data_dir: PathBuf,
}

struct DaemonState {
    data_dir: PathBuf,
    workspaces: Mutex<HashMap<String, WorkspaceEntry>>,
    sessions: Mutex<HashMap<String, Arc<WorkspaceSession>>>,
    terminal_sessions: Mutex<HashMap<String, Arc<TerminalSession>>>,
    storage_path: PathBuf,
    settings_path: PathBuf,
    domains_path: PathBuf,
    app_settings: Mutex<AppSettings>,
    domains: Mutex<Vec<Domain>>,
    memory: RwLock<Option<MemoryService>>,
    auto_memory_runtime: Mutex<AutoMemoryRuntime>,
    browser: BrowserService,
    event_sink: DaemonEventSink,
}

#[derive(Serialize, Deserialize)]
struct WorkspaceFileResponse {
    content: String,
    truncated: bool,
}

#[derive(Serialize, Deserialize)]
struct TextFileResponse {
    exists: bool,
    content: String,
    truncated: bool,
}

struct TerminalSession {
    id: String,
    master: Mutex<Box<dyn portable_pty::MasterPty + Send>>,
    writer: Mutex<Box<dyn Write + Send>>,
    child: Mutex<Box<dyn portable_pty::Child + Send>>,
}

#[derive(Debug, Serialize, Clone)]
struct TerminalSessionInfo {
    id: String,
}

#[derive(Serialize, Clone)]
struct CustomPromptEntry {
    name: String,
    path: String,
    description: Option<String>,
    #[serde(rename = "argumentHint")]
    argument_hint: Option<String>,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
}

impl DaemonState {
    fn load(config: &DaemonConfig, event_sink: DaemonEventSink) -> Self {
        let storage_path = config.data_dir.join("workspaces.json");
        let settings_path = config.data_dir.join("settings.json");
        let domains_path = config.data_dir.join("domains.json");
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
        Self {
            data_dir: config.data_dir.clone(),
            workspaces: Mutex::new(workspaces),
            sessions: Mutex::new(HashMap::new()),
            terminal_sessions: Mutex::new(HashMap::new()),
            storage_path,
            settings_path,
            domains_path,
            app_settings: Mutex::new(app_settings),
            domains: Mutex::new(domains),
            memory: RwLock::new(memory),
            auto_memory_runtime: Mutex::new(AutoMemoryRuntime::default()),
            browser: BrowserService::new(),
            event_sink,
        }
    }

    async fn kill_session(&self, workspace_id: &str) {
        let session = {
            let mut sessions = self.sessions.lock().await;
            sessions.remove(workspace_id)
        };

        let Some(session) = session else {
            return;
        };

        let mut child = session.child.lock().await;
        let _ = child.kill().await;
    }

    async fn list_workspaces(&self) -> Vec<WorkspaceInfo> {
        let workspaces = self.workspaces.lock().await;
        let sessions = self.sessions.lock().await;
        let mut result = Vec::new();
        for entry in workspaces.values() {
            result.push(WorkspaceInfo {
                id: entry.id.clone(),
                name: entry.name.clone(),
                path: entry.path.clone(),
                connected: sessions.contains_key(&entry.id),
                codex_bin: entry.codex_bin.clone(),
                kind: entry.kind.clone(),
                parent_id: entry.parent_id.clone(),
                worktree: entry.worktree.clone(),
                settings: entry.settings.clone(),
            });
        }
        sort_workspaces(&mut result);
        result
    }

    async fn domain_trends(
        &self,
        workspace_id: String,
        domain_id: String,
        range: String,
    ) -> Result<DomainTrendSnapshot, String> {
        let workspaces = self.workspaces.lock().await;
        let workspace = workspaces
            .get(&workspace_id)
            .ok_or_else(|| "workspace not found".to_string())?;
        obsidian::compute_domain_trends(&workspace.path, &domain_id, &range)
    }

    async fn is_workspace_path_dir(&self, path: String) -> bool {
        PathBuf::from(&path).is_dir()
    }

    async fn add_workspace(
        &self,
        path: String,
        codex_bin: Option<String>,
        client_version: String,
    ) -> Result<WorkspaceInfo, String> {
        if !PathBuf::from(&path).is_dir() {
            return Err("Workspace path must be a folder.".to_string());
        }

        let name = PathBuf::from(&path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Workspace")
            .to_string();

        let entry = WorkspaceEntry {
            id: Uuid::new_v4().to_string(),
            name: name.clone(),
            path: path.clone(),
            codex_bin,
            kind: WorkspaceKind::Main,
            parent_id: None,
            worktree: None,
            settings: WorkspaceSettings::default(),
        };

        let default_bin = {
            let settings = self.app_settings.lock().await;
            settings.codex_bin.clone()
        };

        let codex_home = codex_home::resolve_workspace_codex_home(&entry, None);
        let codex_args = {
            let settings = self.app_settings.lock().await;
            codex_args::resolve_workspace_codex_args(&entry, None, Some(&settings))
        };
        let session = spawn_workspace_session(
            entry.clone(),
            default_bin,
            codex_args,
            codex_home,
            client_version,
            self.event_sink.clone(),
        )
        .await?;

        let list = {
            let mut workspaces = self.workspaces.lock().await;
            workspaces.insert(entry.id.clone(), entry.clone());
            workspaces.values().cloned().collect::<Vec<_>>()
        };
        write_workspaces(&self.storage_path, &list)?;

        self.sessions.lock().await.insert(entry.id.clone(), session);

        Ok(WorkspaceInfo {
            id: entry.id,
            name: entry.name,
            path: entry.path,
            connected: true,
            codex_bin: entry.codex_bin,
            kind: entry.kind,
            parent_id: entry.parent_id,
            worktree: entry.worktree,
            settings: entry.settings,
        })
    }

    async fn add_worktree(
        &self,
        parent_id: String,
        branch: String,
        client_version: String,
    ) -> Result<WorkspaceInfo, String> {
        let branch = branch.trim().to_string();
        if branch.trim().is_empty() {
            return Err("Branch name is required.".to_string());
        }

        let parent_entry = {
            let workspaces = self.workspaces.lock().await;
            workspaces
                .get(&parent_id)
                .cloned()
                .ok_or("parent workspace not found")?
        };

        if parent_entry.kind.is_worktree() {
            return Err("Cannot create a worktree from another worktree.".to_string());
        }

        let worktree_root = self.data_dir.join("worktrees").join(&parent_entry.id);
        std::fs::create_dir_all(&worktree_root)
            .map_err(|e| format!("Failed to create worktree directory: {e}"))?;

        let safe_name = sanitize_worktree_name(&branch);
        let worktree_path = unique_worktree_path(&worktree_root, &safe_name)?;
        let worktree_path_string = worktree_path.to_string_lossy().to_string();

        let repo_path = PathBuf::from(&parent_entry.path);
        let branch_exists = git_branch_exists(&repo_path, &branch).await?;
        if branch_exists {
            run_git_command(
                &repo_path,
                &["worktree", "add", &worktree_path_string, &branch],
            )
            .await?;
        } else if let Some(remote_ref) =
            git_find_remote_tracking_branch(&repo_path, &branch).await?
        {
            run_git_command(
                &repo_path,
                &[
                    "worktree",
                    "add",
                    "-b",
                    &branch,
                    &worktree_path_string,
                    &remote_ref,
                ],
            )
            .await?;
        } else {
            run_git_command(
                &repo_path,
                &["worktree", "add", "-b", &branch, &worktree_path_string],
            )
            .await?;
        }

        let entry = WorkspaceEntry {
            id: Uuid::new_v4().to_string(),
            name: branch.to_string(),
            path: worktree_path_string,
            codex_bin: parent_entry.codex_bin.clone(),
            kind: WorkspaceKind::Worktree,
            parent_id: Some(parent_entry.id.clone()),
            worktree: Some(WorktreeInfo {
                branch: branch.to_string(),
            }),
            settings: WorkspaceSettings::default(),
        };

        let default_bin = {
            let settings = self.app_settings.lock().await;
            settings.codex_bin.clone()
        };

        let codex_home = codex_home::resolve_workspace_codex_home(&entry, Some(&parent_entry));
        let codex_args = {
            let settings = self.app_settings.lock().await;
            codex_args::resolve_workspace_codex_args(&entry, Some(&parent_entry), Some(&settings))
        };
        let session = spawn_workspace_session(
            entry.clone(),
            default_bin,
            codex_args,
            codex_home,
            client_version,
            self.event_sink.clone(),
        )
        .await?;

        let list = {
            let mut workspaces = self.workspaces.lock().await;
            workspaces.insert(entry.id.clone(), entry.clone());
            workspaces.values().cloned().collect::<Vec<_>>()
        };
        write_workspaces(&self.storage_path, &list)?;

        self.sessions.lock().await.insert(entry.id.clone(), session);

        Ok(WorkspaceInfo {
            id: entry.id,
            name: entry.name,
            path: entry.path,
            connected: true,
            codex_bin: entry.codex_bin,
            kind: entry.kind,
            parent_id: entry.parent_id,
            worktree: entry.worktree,
            settings: entry.settings,
        })
    }

    async fn remove_workspace(&self, id: String) -> Result<(), String> {
        let (entry, child_worktrees) = {
            let workspaces = self.workspaces.lock().await;
            let entry = workspaces.get(&id).cloned().ok_or("workspace not found")?;
            if entry.kind.is_worktree() {
                return Err("Use remove_worktree for worktree agents.".to_string());
            }
            let children = workspaces
                .values()
                .filter(|workspace| workspace.parent_id.as_deref() == Some(&id))
                .cloned()
                .collect::<Vec<_>>();
            (entry, children)
        };

        let repo_path = PathBuf::from(&entry.path);
        let mut removed_child_ids = Vec::new();
        let mut failures = Vec::new();

        for child in &child_worktrees {
            let child_path = PathBuf::from(&child.path);
            if child_path.exists() {
                if let Err(err) =
                    run_git_command(&repo_path, &["worktree", "remove", "--force", &child.path])
                        .await
                {
                    if is_missing_worktree_error(&err) {
                        if let Err(fs_err) = std::fs::remove_dir_all(&child_path) {
                            failures.push((
                                child.id.clone(),
                                format!("Failed to remove worktree folder: {fs_err}"),
                            ));
                            continue;
                        }
                    } else {
                        failures.push((child.id.clone(), err));
                        continue;
                    }
                }
            }

            self.kill_session(&child.id).await;
            removed_child_ids.push(child.id.clone());
        }

        let _ = run_git_command(&repo_path, &["worktree", "prune", "--expire", "now"]).await;

        let mut ids_to_remove = removed_child_ids;
        if failures.is_empty() {
            self.kill_session(&id).await;
            ids_to_remove.push(id.clone());
        }

        if !ids_to_remove.is_empty() {
            let list = {
                let mut workspaces = self.workspaces.lock().await;
                for workspace_id in ids_to_remove {
                    workspaces.remove(&workspace_id);
                }
                workspaces.values().cloned().collect::<Vec<_>>()
            };
            write_workspaces(&self.storage_path, &list)?;
        }

        if failures.is_empty() {
            return Ok(());
        }

        let mut message =
            "Failed to remove one or more worktrees; parent workspace was not removed.".to_string();
        for (child_id, error) in failures {
            message.push_str(&format!("\n- {child_id}: {error}"));
        }
        Err(message)
    }

    async fn remove_worktree(&self, id: String) -> Result<(), String> {
        let (entry, parent) = {
            let workspaces = self.workspaces.lock().await;
            let entry = workspaces.get(&id).cloned().ok_or("workspace not found")?;
            if !entry.kind.is_worktree() {
                return Err("Not a worktree workspace.".to_string());
            }
            let parent_id = entry.parent_id.clone().ok_or("worktree parent not found")?;
            let parent = workspaces
                .get(&parent_id)
                .cloned()
                .ok_or("worktree parent not found")?;
            (entry, parent)
        };

        let parent_path = PathBuf::from(&parent.path);
        let entry_path = PathBuf::from(&entry.path);
        if entry_path.exists() {
            if let Err(err) = run_git_command(
                &parent_path,
                &["worktree", "remove", "--force", &entry.path],
            )
            .await
            {
                if is_missing_worktree_error(&err) {
                    if entry_path.exists() {
                        std::fs::remove_dir_all(&entry_path).map_err(|fs_err| {
                            format!("Failed to remove worktree folder: {fs_err}")
                        })?;
                    }
                } else {
                    return Err(err);
                }
            }
        }
        let _ = run_git_command(&parent_path, &["worktree", "prune", "--expire", "now"]).await;

        self.kill_session(&entry.id).await;

        let list = {
            let mut workspaces = self.workspaces.lock().await;
            workspaces.remove(&entry.id);
            workspaces.values().cloned().collect::<Vec<_>>()
        };
        write_workspaces(&self.storage_path, &list)?;

        Ok(())
    }

    async fn rename_worktree(
        &self,
        id: String,
        branch: String,
        client_version: String,
    ) -> Result<WorkspaceInfo, String> {
        let trimmed = branch.trim();
        if trimmed.is_empty() {
            return Err("Branch name is required.".to_string());
        }

        let (entry, parent) = {
            let workspaces = self.workspaces.lock().await;
            let entry = workspaces.get(&id).cloned().ok_or("workspace not found")?;
            if !entry.kind.is_worktree() {
                return Err("Not a worktree workspace.".to_string());
            }
            let parent_id = entry.parent_id.clone().ok_or("worktree parent not found")?;
            let parent = workspaces
                .get(&parent_id)
                .cloned()
                .ok_or("worktree parent not found")?;
            (entry, parent)
        };

        let old_branch = entry
            .worktree
            .as_ref()
            .map(|worktree| worktree.branch.clone())
            .ok_or("worktree metadata missing")?;
        if old_branch == trimmed {
            return Err("Branch name is unchanged.".to_string());
        }

        let parent_root = PathBuf::from(&parent.path);

        let (final_branch, _was_suffixed) = unique_branch_name(&parent_root, trimmed, None).await?;
        if final_branch == old_branch {
            return Err("Branch name is unchanged.".to_string());
        }

        run_git_command(&parent_root, &["branch", "-m", &old_branch, &final_branch]).await?;

        let worktree_root = self.data_dir.join("worktrees").join(&parent.id);
        std::fs::create_dir_all(&worktree_root)
            .map_err(|e| format!("Failed to create worktree directory: {e}"))?;

        let safe_name = sanitize_worktree_name(&final_branch);
        let current_path = PathBuf::from(&entry.path);
        let next_path = unique_worktree_path_for_rename(&worktree_root, &safe_name, &current_path)?;
        let next_path_string = next_path.to_string_lossy().to_string();
        if next_path_string != entry.path {
            if let Err(error) = run_git_command(
                &parent_root,
                &["worktree", "move", &entry.path, &next_path_string],
            )
            .await
            {
                let _ =
                    run_git_command(&parent_root, &["branch", "-m", &final_branch, &old_branch])
                        .await;
                return Err(error);
            }
        }

        let (entry_snapshot, list) = {
            let mut workspaces = self.workspaces.lock().await;
            let entry = match workspaces.get_mut(&id) {
                Some(entry) => entry,
                None => return Err("workspace not found".to_string()),
            };
            entry.name = final_branch.clone();
            entry.path = next_path_string.clone();
            match entry.worktree.as_mut() {
                Some(worktree) => {
                    worktree.branch = final_branch.clone();
                }
                None => {
                    entry.worktree = Some(WorktreeInfo {
                        branch: final_branch.clone(),
                    });
                }
            }
            let snapshot = entry.clone();
            let list: Vec<_> = workspaces.values().cloned().collect();
            (snapshot, list)
        };
        write_workspaces(&self.storage_path, &list)?;

        let was_connected = self.sessions.lock().await.contains_key(&entry_snapshot.id);
        if was_connected {
            self.kill_session(&entry_snapshot.id).await;
            let default_bin = {
                let settings = self.app_settings.lock().await;
                settings.codex_bin.clone()
            };
            let codex_home =
                codex_home::resolve_workspace_codex_home(&entry_snapshot, Some(&parent));
            let codex_args = {
                let settings = self.app_settings.lock().await;
                codex_args::resolve_workspace_codex_args(
                    &entry_snapshot,
                    Some(&parent),
                    Some(&settings),
                )
            };
            match spawn_workspace_session(
                entry_snapshot.clone(),
                default_bin,
                codex_args,
                codex_home,
                client_version,
                self.event_sink.clone(),
            )
            .await
            {
                Ok(session) => {
                    self.sessions
                        .lock()
                        .await
                        .insert(entry_snapshot.id.clone(), session);
                }
                Err(error) => {
                    eprintln!(
                        "rename_worktree: respawn failed for {} after rename: {error}",
                        entry_snapshot.id
                    );
                }
            }
        }

        let connected = self.sessions.lock().await.contains_key(&entry_snapshot.id);
        Ok(WorkspaceInfo {
            id: entry_snapshot.id,
            name: entry_snapshot.name,
            path: entry_snapshot.path,
            connected,
            codex_bin: entry_snapshot.codex_bin,
            kind: entry_snapshot.kind,
            parent_id: entry_snapshot.parent_id,
            worktree: entry_snapshot.worktree,
            settings: entry_snapshot.settings,
        })
    }

    async fn rename_worktree_upstream(
        &self,
        id: String,
        old_branch: String,
        new_branch: String,
    ) -> Result<(), String> {
        let old_branch = old_branch.trim();
        let new_branch = new_branch.trim();
        if old_branch.is_empty() || new_branch.is_empty() {
            return Err("Branch name is required.".to_string());
        }
        if old_branch == new_branch {
            return Err("Branch name is unchanged.".to_string());
        }

        let (_entry, parent) = {
            let workspaces = self.workspaces.lock().await;
            let entry = workspaces.get(&id).cloned().ok_or("workspace not found")?;
            if !entry.kind.is_worktree() {
                return Err("Not a worktree workspace.".to_string());
            }
            let parent_id = entry.parent_id.clone().ok_or("worktree parent not found")?;
            let parent = workspaces
                .get(&parent_id)
                .cloned()
                .ok_or("worktree parent not found")?;
            (entry, parent)
        };

        let parent_root = PathBuf::from(&parent.path);
        if !git_branch_exists(&parent_root, new_branch).await? {
            return Err("Local branch not found.".to_string());
        }

        let remote_for_old = git_find_remote_for_branch(&parent_root, old_branch).await?;
        let remote_name = match remote_for_old.as_ref() {
            Some(remote) => remote.clone(),
            None => {
                if git_remote_exists(&parent_root, "origin").await? {
                    "origin".to_string()
                } else {
                    return Err("No git remote configured for this worktree.".to_string());
                }
            }
        };

        if git_remote_branch_exists_live(&parent_root, &remote_name, new_branch).await? {
            return Err("Remote branch already exists.".to_string());
        }

        if remote_for_old.is_some() {
            run_git_command(
                &parent_root,
                &["push", &remote_name, &format!("{new_branch}:{new_branch}")],
            )
            .await?;
            run_git_command(
                &parent_root,
                &["push", &remote_name, &format!(":{old_branch}")],
            )
            .await?;
        } else {
            run_git_command(&parent_root, &["push", &remote_name, new_branch]).await?;
        }

        run_git_command(
            &parent_root,
            &[
                "branch",
                "--set-upstream-to",
                &format!("{remote_name}/{new_branch}"),
                new_branch,
            ],
        )
        .await?;

        Ok(())
    }

    async fn update_workspace_settings(
        &self,
        id: String,
        settings: WorkspaceSettings,
    ) -> Result<WorkspaceInfo, String> {
        let mut settings = settings;
        if matches!(settings.purpose, Some(types::WorkspacePurpose::Life))
            && settings.obsidian_root.is_none()
        {
            settings.obsidian_root = life::default_obsidian_root();
        }

        let (entry_snapshot, list) = {
            let mut workspaces = self.workspaces.lock().await;
            let entry_snapshot = match workspaces.get_mut(&id) {
                Some(entry) => {
                    entry.settings = settings.clone();
                    entry.clone()
                }
                None => return Err("workspace not found".to_string()),
            };
            let list: Vec<_> = workspaces.values().cloned().collect();
            (entry_snapshot, list)
        };
        write_workspaces(&self.storage_path, &list)?;

        let connected = self.sessions.lock().await.contains_key(&id);
        Ok(WorkspaceInfo {
            id: entry_snapshot.id,
            name: entry_snapshot.name,
            path: entry_snapshot.path,
            connected,
            codex_bin: entry_snapshot.codex_bin,
            kind: entry_snapshot.kind,
            parent_id: entry_snapshot.parent_id,
            worktree: entry_snapshot.worktree,
            settings: entry_snapshot.settings,
        })
    }

    async fn update_workspace_codex_bin(
        &self,
        id: String,
        codex_bin: Option<String>,
    ) -> Result<WorkspaceInfo, String> {
        let (entry_snapshot, list) = {
            let mut workspaces = self.workspaces.lock().await;
            let entry_snapshot = match workspaces.get_mut(&id) {
                Some(entry) => {
                    entry.codex_bin = codex_bin.clone();
                    entry.clone()
                }
                None => return Err("workspace not found".to_string()),
            };
            let list: Vec<_> = workspaces.values().cloned().collect();
            (entry_snapshot, list)
        };
        write_workspaces(&self.storage_path, &list)?;

        let connected = self.sessions.lock().await.contains_key(&id);
        Ok(WorkspaceInfo {
            id: entry_snapshot.id,
            name: entry_snapshot.name,
            path: entry_snapshot.path,
            connected,
            codex_bin: entry_snapshot.codex_bin,
            kind: entry_snapshot.kind,
            parent_id: entry_snapshot.parent_id,
            worktree: entry_snapshot.worktree,
            settings: entry_snapshot.settings,
        })
    }

    async fn connect_workspace(&self, id: String, client_version: String) -> Result<(), String> {
        {
            let sessions = self.sessions.lock().await;
            if sessions.contains_key(&id) {
                return Ok(());
            }
        }

        let entry = {
            let workspaces = self.workspaces.lock().await;
            workspaces.get(&id).cloned().ok_or("workspace not found")?
        };

        let default_bin = {
            let settings = self.app_settings.lock().await;
            settings.codex_bin.clone()
        };

        let parent_entry = if entry.kind.is_worktree() {
            let workspaces = self.workspaces.lock().await;
            entry
                .parent_id
                .as_deref()
                .and_then(|parent_id| workspaces.get(parent_id))
                .cloned()
        } else {
            None
        };
        let codex_home = codex_home::resolve_workspace_codex_home(&entry, parent_entry.as_ref());
        let codex_args = {
            let settings = self.app_settings.lock().await;
            codex_args::resolve_workspace_codex_args(&entry, parent_entry.as_ref(), Some(&settings))
        };
        let session = spawn_workspace_session(
            entry,
            default_bin,
            codex_args,
            codex_home,
            client_version,
            self.event_sink.clone(),
        )
        .await?;

        self.sessions.lock().await.insert(id, session);
        Ok(())
    }

    async fn update_app_settings(&self, settings: AppSettings) -> Result<AppSettings, String> {
        let _ = codex_config::write_collab_enabled(settings.experimental_collab_enabled);
        let _ = codex_config::write_steer_enabled(settings.experimental_steer_enabled);
        let _ =
            codex_config::write_unified_exec_enabled(settings.experimental_unified_exec_enabled);
        write_settings(&self.settings_path, &settings)?;
        let mut current = self.app_settings.lock().await;
        *current = settings.clone();
        let mut memory_lock = self.memory.write().await;
        *memory_lock = if settings.memory_enabled
            && !settings.supabase_url.is_empty()
            && !settings.supabase_anon_key.is_empty()
        {
            Some(MemoryService::new(
                &settings.supabase_url,
                &settings.supabase_anon_key,
                if settings.memory_embedding_enabled {
                    Some(&settings.minimax_api_key)
                } else {
                    None
                },
                true,
            ))
        } else {
            None
        };
        Ok(settings)
    }

    async fn domains_list(&self) -> Result<Vec<Domain>, String> {
        let domains = self.domains.lock().await;
        Ok(domains.clone())
    }

    async fn domains_create(&self, mut domain: Domain) -> Result<Domain, String> {
        domain.id = Uuid::new_v4().to_string();
        let domain = Self::normalize_domain(domain);
        let mut domains = self.domains.lock().await;
        domains.push(domain.clone());
        write_domains(&self.domains_path, &domains)?;
        Ok(domain)
    }

    async fn domains_update(&self, domain: Domain) -> Result<Domain, String> {
        let domain = Self::normalize_domain(domain);
        let mut domains = self.domains.lock().await;
        if let Some(idx) = domains.iter().position(|item| item.id == domain.id) {
            domains[idx] = domain.clone();
            write_domains(&self.domains_path, &domains)?;
            Ok(domain)
        } else {
            Err(format!("Domain not found: {}", domain.id))
        }
    }

    async fn domains_delete(&self, domain_id: String) -> Result<(), String> {
        let mut domains = self.domains.lock().await;
        domains.retain(|domain| domain.id != domain_id);
        write_domains(&self.domains_path, &domains)?;
        Ok(())
    }

    fn normalize_domain(mut domain: Domain) -> Domain {
        if domain.view_type.trim().is_empty() {
            domain.view_type = "chat".to_string();
        }
        domain
    }

    async fn memory_flush_now(
        &self,
        workspace_id: String,
        thread_id: String,
        force: bool,
    ) -> Result<Value, String> {
        let settings = self.app_settings.lock().await.clone();
        if !settings.auto_memory.enabled && !force {
            return Err("Auto memory disabled".to_string());
        }

        let memory = self
            .memory
            .read()
            .await
            .clone()
            .ok_or("Memory not enabled")?;
        let session = self.get_session(&workspace_id).await?;
        perform_memory_flush(
            session,
            memory,
            settings.auto_memory,
            workspace_id,
            thread_id,
            0,
            0,
        )
        .await
    }

    async fn get_session(&self, workspace_id: &str) -> Result<Arc<WorkspaceSession>, String> {
        let sessions = self.sessions.lock().await;
        sessions
            .get(workspace_id)
            .cloned()
            .ok_or("workspace not connected".to_string())
    }

    async fn list_workspace_files(&self, workspace_id: String) -> Result<Vec<String>, String> {
        let entry = {
            let workspaces = self.workspaces.lock().await;
            workspaces
                .get(&workspace_id)
                .cloned()
                .ok_or("workspace not found")?
        };

        let root = PathBuf::from(entry.path);
        Ok(list_workspace_files_inner(&root, 20000))
    }

    async fn read_workspace_file(
        &self,
        workspace_id: String,
        path: String,
    ) -> Result<WorkspaceFileResponse, String> {
        let entry = {
            let workspaces = self.workspaces.lock().await;
            workspaces
                .get(&workspace_id)
                .cloned()
                .ok_or("workspace not found")?
        };

        let root = PathBuf::from(entry.path);
        read_workspace_file_inner(&root, &path)
    }

    async fn read_global_agents_md(&self) -> Result<TextFileResponse, String> {
        read_global_file_inner("AGENTS.md")
    }

    async fn write_global_agents_md(&self, content: String) -> Result<(), String> {
        write_global_file_inner("AGENTS.md", &content)
    }

    async fn read_global_config_toml(&self) -> Result<TextFileResponse, String> {
        read_global_file_inner("config.toml")
    }

    async fn write_global_config_toml(&self, content: String) -> Result<(), String> {
        write_global_file_inner("config.toml", &content)
    }

    async fn get_life_workspace_prompt(&self) -> Result<String, String> {
        life::build_life_workspace_prompt()
    }

    async fn get_delivery_dashboard(
        &self,
        workspace_id: String,
        range: String,
    ) -> Result<Value, String> {
        let workspaces = self.workspaces.lock().await;
        let entry = workspaces
            .get(&workspace_id)
            .cloned()
            .ok_or("workspace not found")?;
        let supabase = {
            let settings = self.app_settings.lock().await;
            if settings.supabase_url.trim().is_empty()
                || settings.supabase_anon_key.trim().is_empty()
            {
                None
            } else {
                Some((
                    settings.supabase_url.clone(),
                    settings.supabase_anon_key.clone(),
                ))
            }
        };
        let dashboard = life::build_delivery_dashboard(
            &entry.path,
            entry.settings.obsidian_root.as_deref(),
            supabase.as_ref().map(|value| value.0.as_str()),
            supabase.as_ref().map(|value| value.1.as_str()),
            &range,
        )
        .await?;
        serde_json::to_value(dashboard).map_err(|err| err.to_string())
    }

    async fn get_nutrition_dashboard(
        &self,
        workspace_id: String,
        range: String,
    ) -> Result<Value, String> {
        let workspaces = self.workspaces.lock().await;
        let entry = workspaces
            .get(&workspace_id)
            .cloned()
            .ok_or("workspace not found")?;
        let dashboard = life::build_nutrition_dashboard(
            &entry.path,
            entry.settings.obsidian_root.as_deref(),
            &range,
        )
        .await?;
        serde_json::to_value(dashboard).map_err(|err| err.to_string())
    }

    async fn get_exercise_dashboard(
        &self,
        workspace_id: String,
        range: String,
    ) -> Result<Value, String> {
        let workspaces = self.workspaces.lock().await;
        let entry = workspaces
            .get(&workspace_id)
            .cloned()
            .ok_or("workspace not found")?;
        let dashboard = life::build_exercise_dashboard(
            &entry.path,
            entry.settings.obsidian_root.as_deref(),
            &range,
        )
        .await?;
        serde_json::to_value(dashboard).map_err(|err| err.to_string())
    }

    async fn get_media_dashboard(&self, workspace_id: String) -> Result<Value, String> {
        let workspaces = self.workspaces.lock().await;
        let entry = workspaces
            .get(&workspace_id)
            .cloned()
            .ok_or("workspace not found")?;
        let dashboard =
            life::build_media_library(&entry.path, entry.settings.obsidian_root.as_deref()).await?;
        serde_json::to_value(dashboard).map_err(|err| err.to_string())
    }

    async fn get_youtube_dashboard(&self, workspace_id: String) -> Result<Value, String> {
        let workspaces = self.workspaces.lock().await;
        let entry = workspaces
            .get(&workspace_id)
            .cloned()
            .ok_or("workspace not found")?;
        let dashboard =
            life::build_youtube_library(&entry.path, entry.settings.obsidian_root.as_deref())
                .await?;
        serde_json::to_value(dashboard).map_err(|err| err.to_string())
    }

    async fn enrich_media_covers(
        &self,
        workspace_id: String,
        force: bool,
    ) -> Result<Value, String> {
        let workspaces = self.workspaces.lock().await;
        let entry = workspaces
            .get(&workspace_id)
            .cloned()
            .ok_or("workspace not found")?;
        let settings = self.app_settings.lock().await;
        let tmdb_key = resolve_api_key(settings.tmdb_api_key.as_str(), "TMDB_API_KEY");
        let igdb_client_id = resolve_api_key(settings.igdb_client_id.as_str(), "IGDB_CLIENT_ID");
        let igdb_client_secret =
            resolve_api_key(settings.igdb_client_secret.as_str(), "IGDB_CLIENT_SECRET");
        let exa_api_key = if !settings.exa_api_key.trim().is_empty() {
            Some(settings.exa_api_key.clone())
        } else {
            resolve_api_key("", "EXA_API_KEY")
        };
        let summary = life::enrich_media_covers(
            &entry.path,
            entry.settings.obsidian_root.as_deref(),
            tmdb_key.as_deref(),
            igdb_client_id.as_deref(),
            igdb_client_secret.as_deref(),
            exa_api_key.as_deref(),
            force,
        )
        .await?;
        serde_json::to_value(summary).map_err(|err| err.to_string())
    }

    async fn fix_broken_covers(&self, workspace_id: String) -> Result<Value, String> {
        let workspaces = self.workspaces.lock().await;
        let entry = workspaces
            .get(&workspace_id)
            .cloned()
            .ok_or("workspace not found")?;
        let settings = self.app_settings.lock().await;
        let tmdb_key = resolve_api_key(settings.tmdb_api_key.as_str(), "TMDB_API_KEY");
        let igdb_client_id = resolve_api_key(settings.igdb_client_id.as_str(), "IGDB_CLIENT_ID");
        let igdb_client_secret =
            resolve_api_key(settings.igdb_client_secret.as_str(), "IGDB_CLIENT_SECRET");
        let exa_api_key = if !settings.exa_api_key.trim().is_empty() {
            Some(settings.exa_api_key.clone())
        } else {
            resolve_api_key("", "EXA_API_KEY")
        };
        let summary = life::fix_broken_covers(
            &entry.path,
            entry.settings.obsidian_root.as_deref(),
            tmdb_key.as_deref(),
            igdb_client_id.as_deref(),
            igdb_client_secret.as_deref(),
            exa_api_key.as_deref(),
        )
        .await?;
        serde_json::to_value(summary).map_err(|err| err.to_string())
    }

    async fn rebuild_media_covers(&self, workspace_id: String) -> Result<Value, String> {
        let workspaces = self.workspaces.lock().await;
        let entry = workspaces
            .get(&workspace_id)
            .cloned()
            .ok_or("workspace not found")?;
        let settings = self.app_settings.lock().await;
        let tmdb_key = resolve_api_key(settings.tmdb_api_key.as_str(), "TMDB_API_KEY");
        let igdb_client_id = resolve_api_key(settings.igdb_client_id.as_str(), "IGDB_CLIENT_ID");
        let igdb_client_secret =
            resolve_api_key(settings.igdb_client_secret.as_str(), "IGDB_CLIENT_SECRET");
        let exa_api_key = if !settings.exa_api_key.trim().is_empty() {
            Some(settings.exa_api_key.clone())
        } else {
            resolve_api_key("", "EXA_API_KEY")
        };
        let summary = life::rebuild_media_covers(
            &entry.path,
            entry.settings.obsidian_root.as_deref(),
            tmdb_key.as_deref(),
            igdb_client_id.as_deref(),
            igdb_client_secret.as_deref(),
            exa_api_key.as_deref(),
        )
        .await?;
        serde_json::to_value(summary).map_err(|err| err.to_string())
    }

    async fn get_finance_dashboard(
        &self,
        workspace_id: String,
        range: String,
    ) -> Result<Value, String> {
        let workspaces = self.workspaces.lock().await;
        let entry = workspaces
            .get(&workspace_id)
            .cloned()
            .ok_or("workspace not found")?;
        let dashboard = life::build_finance_dashboard(
            &entry.path,
            entry.settings.obsidian_root.as_deref(),
            &range,
        )
        .await?;
        serde_json::to_value(dashboard).map_err(|err| err.to_string())
    }

    async fn start_thread(&self, workspace_id: String) -> Result<Value, String> {
        let session = self.get_session(&workspace_id).await?;
        let is_life = {
            let workspaces = self.workspaces.lock().await;
            workspaces
                .get(&workspace_id)
                .map(|workspace| life::is_life_workspace(&workspace.settings))
                .unwrap_or(false)
        };

        let mut params = Map::new();
        params.insert("cwd".to_string(), json!(session.entry.path));
        params.insert("approvalPolicy".to_string(), json!("on-request"));
        if is_life {
            let prompt = life::build_life_workspace_prompt()?;
            if life::life_debug_enabled() {
                eprintln!(
                    "[life] start_thread: injecting systemPrompt (len={})",
                    prompt.len()
                );
            }
            params.insert("systemPrompt".to_string(), json!(prompt));
        }
        session
            .send_request("thread/start", Value::Object(params))
            .await
    }

    async fn resume_thread(
        &self,
        workspace_id: String,
        thread_id: String,
    ) -> Result<Value, String> {
        let session = self.get_session(&workspace_id).await?;
        let params = json!({
            "threadId": thread_id
        });
        session.send_request("thread/resume", params).await
    }

    async fn list_threads(
        &self,
        workspace_id: String,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> Result<Value, String> {
        let session = self.get_session(&workspace_id).await?;
        let params = json!({
            "cursor": cursor,
            "limit": limit
        });
        session.send_request("thread/list", params).await
    }

    async fn archive_thread(
        &self,
        workspace_id: String,
        thread_id: String,
    ) -> Result<Value, String> {
        let session = self.get_session(&workspace_id).await?;
        let params = json!({ "threadId": thread_id });
        session.send_request("thread/archive", params).await
    }

    async fn send_user_message(
        &self,
        workspace_id: String,
        thread_id: String,
        text: String,
        model: Option<String>,
        effort: Option<String>,
        access_mode: Option<String>,
        images: Option<Vec<String>>,
        collaboration_mode: Option<Value>,
    ) -> Result<Value, String> {
        let session = self.get_session(&workspace_id).await?;
        let access_mode = access_mode.unwrap_or_else(|| "current".to_string());
        let sandbox_policy = match access_mode.as_str() {
            "full-access" => json!({
                "type": "dangerFullAccess"
            }),
            "read-only" => json!({
                "type": "readOnly"
            }),
            _ => json!({
                "type": "workspaceWrite",
                "writableRoots": [session.entry.path],
                "networkAccess": true
            }),
        };

        let approval_policy = if access_mode == "full-access" {
            "never"
        } else {
            "on-request"
        };

        let input = build_user_input(&text, images.as_deref())?;

        let (is_life_workspace, domain_instructions) = {
            let workspaces = self.workspaces.lock().await;
            let workspace = workspaces.get(&workspace_id);
            if let Some(workspace) = workspace {
                let is_life_workspace = life::is_life_workspace(&workspace.settings);
                if is_life_workspace {
                    (true, None)
                } else {
                    let apply = workspace.settings.apply_domain_instructions.unwrap_or(true);
                    if apply {
                        let domains = self.domains.lock().await;
                        (
                            false,
                            workspace
                                .settings
                                .domain_id
                                .as_ref()
                                .and_then(|id| domains.iter().find(|domain| &domain.id == id))
                                .map(|domain| domain.system_prompt.clone()),
                        )
                    } else {
                        (false, None)
                    }
                }
            } else {
                (false, None)
            }
        };

        if is_life_workspace && life::life_debug_enabled() {
            eprintln!(
                "[life] send_user_message: skipping per-turn domain injection (thread={})",
                thread_id
            );
        }

        let params = build_turn_start_params(
            &thread_id,
            input,
            &session.entry.path,
            approval_policy,
            sandbox_policy,
            model,
            effort,
            collaboration_mode,
            domain_instructions,
        );
        session.send_request("turn/start", params).await
    }

    async fn turn_interrupt(
        &self,
        workspace_id: String,
        thread_id: String,
        turn_id: String,
    ) -> Result<Value, String> {
        let session = self.get_session(&workspace_id).await?;
        let params = json!({
            "threadId": thread_id,
            "turnId": turn_id
        });
        session.send_request("turn/interrupt", params).await
    }

    async fn start_review(
        &self,
        workspace_id: String,
        thread_id: String,
        target: Value,
        delivery: Option<String>,
    ) -> Result<Value, String> {
        let session = self.get_session(&workspace_id).await?;
        let mut params = Map::new();
        params.insert("threadId".to_string(), json!(thread_id));
        params.insert("target".to_string(), target);
        if let Some(delivery) = delivery {
            params.insert("delivery".to_string(), json!(delivery));
        }
        session
            .send_request("review/start", Value::Object(params))
            .await
    }

    async fn model_list(&self, workspace_id: String) -> Result<Value, String> {
        let session = self.get_session(&workspace_id).await?;
        session.send_request("model/list", json!({})).await
    }

    async fn collaboration_mode_list(&self, workspace_id: String) -> Result<Value, String> {
        let session = self.get_session(&workspace_id).await?;
        session
            .send_request("collaborationMode/list", json!({}))
            .await
    }

    async fn account_rate_limits(&self, workspace_id: String) -> Result<Value, String> {
        let session = self.get_session(&workspace_id).await?;
        session
            .send_request("account/rateLimits/read", Value::Null)
            .await
    }

    async fn skills_list(&self, workspace_id: String) -> Result<Value, String> {
        let session = self.get_session(&workspace_id).await?;
        let params = json!({
            "cwd": session.entry.path
        });
        session.send_request("skills/list", params).await
    }

    async fn skills_config_write(
        &self,
        workspace_id: String,
        config: Value,
    ) -> Result<Value, String> {
        let session = self.get_session(&workspace_id).await?;
        let mut payload = match config {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        payload
            .entry("cwd".to_string())
            .or_insert(json!(session.entry.path));
        let payload_value = Value::Object(payload.clone());
        let result = session
            .send_request("skills/config/write", payload_value)
            .await?;

        if let Ok(config_path) = self.skills_config_path(&workspace_id).await {
            let config_value = result
                .get("result")
                .and_then(|v| v.get("config"))
                .cloned()
                .unwrap_or_else(|| {
                    let mut clone = payload.clone();
                    clone.remove("cwd");
                    Value::Object(clone)
                });
            let _ = write_json_file(&config_path, &config_value);
        }

        Ok(result)
    }

    async fn skills_config_read(&self, workspace_id: String) -> Result<Value, String> {
        let config_path = self.skills_config_path(&workspace_id).await?;
        if let Ok(value) = read_json_file(&config_path) {
            return Ok(value);
        }
        Ok(json!({ "enabled": [], "disabled": [] }))
    }

    async fn skills_validate(&self, workspace_id: String) -> Result<Value, String> {
        let skills_list = self.skills_list(workspace_id).await?;
        let skills = skills_list
            .pointer("/result/skills")
            .or_else(|| skills_list.pointer("/skills"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut results = Vec::new();
        for entry in skills {
            let path = entry
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if path.is_empty() {
                continue;
            }
            let skill_md_path = if path.ends_with("SKILL.md") {
                PathBuf::from(&path)
            } else {
                PathBuf::from(&path).join("SKILL.md")
            };
            if !skill_md_path.exists() {
                continue;
            }
            if let Ok(desc) = parse_skill_md(&skill_md_path) {
                let issues = validate_skill(&desc);
                results.push(json!({
                    "name": desc.name,
                    "path": desc.path,
                    "issues": issues,
                    "description": desc.description
                }));
            }
        }

        Ok(json!(results))
    }

    async fn skills_install_from_git(
        &self,
        source_url: String,
        target: String,
        workspace_id: Option<String>,
    ) -> Result<Value, String> {
        let root = self
            .resolve_skill_root(&target, workspace_id.as_deref())
            .await?;
        std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;

        let repo_name = source_url
            .split('/')
            .last()
            .unwrap_or("skill")
            .trim_end_matches(".git")
            .to_string();
        let dest = root.join(repo_name);
        if dest.exists() {
            return Err("Destination already exists".to_string());
        }

        let git_bin = resolve_git_binary().map_err(|e| format!("Failed to run git: {e}"))?;
        let status = Command::new(git_bin)
            .arg("clone")
            .arg(&source_url)
            .arg(&dest)
            .env("PATH", git_env_path())
            .status()
            .await
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("git clone failed".to_string());
        }

        let skill_md = dest.join("SKILL.md");
        if !skill_md.exists() {
            return Err("SKILL.md not found in repo".to_string());
        }

        Ok(json!({ "ok": true, "path": dest }))
    }

    async fn skills_uninstall(
        &self,
        name: String,
        target: String,
        workspace_id: Option<String>,
    ) -> Result<Value, String> {
        let root = self
            .resolve_skill_root(&target, workspace_id.as_deref())
            .await?;
        let dest = root.join(&name);
        if !dest.exists() {
            return Err("Skill not found".to_string());
        }
        std::fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
        Ok(json!({ "ok": true }))
    }

    async fn resolve_skill_root(
        &self,
        target: &str,
        workspace_id: Option<&str>,
    ) -> Result<PathBuf, String> {
        match target {
            "global" => {
                let home = std::env::var("HOME").map_err(|_| "Missing HOME".to_string())?;
                Ok(PathBuf::from(home).join(".codex").join("skills"))
            }
            "workspace" => {
                let workspace_id =
                    workspace_id.ok_or("workspaceId required for workspace target")?;
                let workspaces = self.workspaces.lock().await;
                let entry = workspaces.get(workspace_id).ok_or("workspace not found")?;
                Ok(PathBuf::from(&entry.path).join(".codex").join("skills"))
            }
            _ => Err("Invalid target (use 'global' or 'workspace')".to_string()),
        }
    }

    async fn respond_to_server_request(
        &self,
        workspace_id: String,
        request_id: Value,
        result: Value,
    ) -> Result<Value, String> {
        let session = self.get_session(&workspace_id).await?;
        session.send_response(request_id, result).await?;
        Ok(json!({ "ok": true }))
    }

    async fn remember_approval_rule(
        &self,
        workspace_id: String,
        command: Vec<String>,
    ) -> Result<Value, String> {
        let command = command
            .into_iter()
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>();
        if command.is_empty() {
            return Err("empty command".to_string());
        }

        let (entry, parent_entry) = self.workspace_entry_with_parent(&workspace_id).await?;

        let codex_home = codex_home::resolve_workspace_codex_home(&entry, parent_entry.as_ref())
            .ok_or("Unable to resolve CODEX_HOME".to_string())?;
        let rules_path = rules::default_rules_path(&codex_home);
        rules::append_prefix_rule(&rules_path, &command)?;

        Ok(json!({
            "ok": true,
            "rulesPath": rules_path,
        }))
    }

    async fn skills_config_path(&self, workspace_id: &str) -> Result<PathBuf, String> {
        let (entry, parent_entry) = self.workspace_entry_with_parent(workspace_id).await?;
        let codex_home = codex_home::resolve_workspace_codex_home(&entry, parent_entry.as_ref())
            .ok_or("Unable to resolve CODEX_HOME")?;
        Ok(codex_home.join("skills").join("config.json"))
    }
}

async fn perform_memory_flush(
    session: Arc<WorkspaceSession>,
    memory: MemoryService,
    settings: AutoMemorySettings,
    workspace_id: String,
    thread_id: String,
    context_tokens: u32,
    model_context_window: u32,
) -> Result<Value, String> {
    let snapshot = build_snapshot(
        &session,
        &workspace_id,
        &thread_id,
        context_tokens,
        model_context_window,
        &settings,
    )
    .await?;

    let raw = run_memory_flush_summarizer(&session, &snapshot).await?;
    let result = parse_memory_flush_result(&raw);
    write_memory_flush(&memory, &snapshot, &result, &settings).await?;

    Ok(json!({
        "ok": true,
        "noReply": result.no_reply,
        "tags": result.tags,
    }))
}

fn sort_workspaces(workspaces: &mut [WorkspaceInfo]) {
    workspaces.sort_by(|a, b| {
        let a_order = a.settings.sort_order.unwrap_or(u32::MAX);
        let b_order = b.settings.sort_order.unwrap_or(u32::MAX);
        if a_order != b_order {
            return a_order.cmp(&b_order);
        }
        a.name.cmp(&b.name)
    });
}

fn should_skip_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | "node_modules" | "dist" | "target" | "release-artifacts"
    )
}

// normalize_git_path provided by utils module

fn list_workspace_files_inner(root: &PathBuf, max_files: usize) -> Vec<String> {
    let mut results = Vec::new();
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .follow_links(false)
        .require_git(false)
        .filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }
            if entry.file_type().is_some_and(|ft| ft.is_dir()) {
                let name = entry.file_name().to_string_lossy();
                return !should_skip_dir(&name);
            }
            true
        })
        .build();

    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        if let Ok(rel_path) = entry.path().strip_prefix(root) {
            let normalized = normalize_git_path(&rel_path.to_string_lossy());
            if !normalized.is_empty() {
                results.push(normalized);
            }
        }
        if results.len() >= max_files {
            break;
        }
    }

    results.sort();
    results
}

const MAX_WORKSPACE_FILE_BYTES: u64 = 400_000;

fn read_workspace_file_inner(
    root: &PathBuf,
    relative_path: &str,
) -> Result<WorkspaceFileResponse, String> {
    let canonical_root = root
        .canonicalize()
        .map_err(|err| format!("Failed to resolve workspace root: {err}"))?;
    let candidate = canonical_root.join(relative_path);
    let canonical_path = candidate
        .canonicalize()
        .map_err(|err| format!("Failed to open file: {err}"))?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err("Invalid file path".to_string());
    }
    let metadata = std::fs::metadata(&canonical_path)
        .map_err(|err| format!("Failed to read file metadata: {err}"))?;
    if !metadata.is_file() {
        return Err("Path is not a file".to_string());
    }

    let mut file =
        File::open(&canonical_path).map_err(|err| format!("Failed to open file: {err}"))?;
    let mut buffer = Vec::new();
    file.take(MAX_WORKSPACE_FILE_BYTES + 1)
        .read_to_end(&mut buffer)
        .map_err(|err| format!("Failed to read file: {err}"))?;

    let truncated = buffer.len() > MAX_WORKSPACE_FILE_BYTES as usize;
    if truncated {
        buffer.truncate(MAX_WORKSPACE_FILE_BYTES as usize);
    }

    let content = String::from_utf8(buffer).map_err(|_| "File is not valid UTF-8".to_string())?;
    Ok(WorkspaceFileResponse { content, truncated })
}

fn read_global_file_inner(filename: &str) -> Result<TextFileResponse, String> {
    let Some(root) = resolve_codex_home() else {
        return Err("Unable to resolve CODEX_HOME".to_string());
    };
    let path = root.join(filename);
    if !path.exists() {
        return Ok(TextFileResponse {
            exists: false,
            content: String::new(),
            truncated: false,
        });
    }
    let content = std::fs::read_to_string(&path).map_err(|err| err.to_string())?;
    Ok(TextFileResponse {
        exists: true,
        content,
        truncated: false,
    })
}

fn write_global_file_inner(filename: &str, content: &str) -> Result<(), String> {
    let Some(root) = resolve_codex_home() else {
        return Err("Unable to resolve CODEX_HOME".to_string());
    };
    std::fs::create_dir_all(&root).map_err(|err| err.to_string())?;
    let path = root.join(filename);
    std::fs::write(path, content).map_err(|err| err.to_string())
}

async fn run_git_command(repo_path: &Path, args: &[&str]) -> Result<String, String> {
    let git_bin = resolve_git_binary().map_err(|e| format!("Failed to run git: {e}"))?;
    let output = Command::new(git_bin)
        .args(args)
        .current_dir(repo_path)
        .env("PATH", git_env_path())
        .output()
        .await
        .map_err(|e| format!("Failed to run git: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        if detail.is_empty() {
            Err("Git command failed.".to_string())
        } else {
            Err(detail.to_string())
        }
    }
}

fn is_missing_worktree_error(error: &str) -> bool {
    error.contains("is not a working tree")
}

async fn run_git_command_bytes(repo_path: &PathBuf, args: &[&str]) -> Result<Vec<u8>, String> {
    let git_bin = resolve_git_binary().map_err(|e| format!("Failed to run git: {e}"))?;
    let output = Command::new(git_bin)
        .args(args)
        .current_dir(repo_path)
        .env("PATH", git_env_path())
        .output()
        .await
        .map_err(|e| format!("Failed to run git: {e}"))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        if detail.is_empty() {
            Err("Git command failed.".to_string())
        } else {
            Err(detail.to_string())
        }
    }
}

async fn run_git_diff(repo_path: &PathBuf, args: &[&str]) -> Result<Vec<u8>, String> {
    let git_bin = resolve_git_binary().map_err(|e| format!("Failed to run git: {e}"))?;
    let output = Command::new(git_bin)
        .args(args)
        .current_dir(repo_path)
        .env("PATH", git_env_path())
        .output()
        .await
        .map_err(|e| format!("Failed to run git: {e}"))?;
    if output.status.success() || output.status.code() == Some(1) {
        Ok(output.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        if detail.is_empty() {
            Err("Git command failed.".to_string())
        } else {
            Err(detail.to_string())
        }
    }
}

fn terminal_key(workspace_id: &str, terminal_id: &str) -> String {
    format!("{workspace_id}:{terminal_id}")
}

fn shell_path() -> String {
    env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
}

fn resolve_locale() -> String {
    let candidate = env::var("LC_ALL")
        .or_else(|_| env::var("LANG"))
        .unwrap_or_else(|_| "en_US.UTF-8".to_string());
    let lower = candidate.to_lowercase();
    if lower.contains("utf-8") || lower.contains("utf8") {
        return candidate;
    }
    "en_US.UTF-8".to_string()
}

fn spawn_terminal_reader(
    event_sink: DaemonEventSink,
    workspace_id: String,
    terminal_id: String,
    mut reader: Box<dyn Read + Send>,
) {
    std::thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        let mut pending: Vec<u8> = Vec::new();
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    pending.extend_from_slice(&buffer[..count]);
                    loop {
                        match std::str::from_utf8(&pending) {
                            Ok(decoded) => {
                                if !decoded.is_empty() {
                                    let payload = TerminalOutput {
                                        workspace_id: workspace_id.clone(),
                                        terminal_id: terminal_id.clone(),
                                        data: decoded.to_string(),
                                    };
                                    event_sink.emit_terminal_output(payload);
                                }
                                pending.clear();
                                break;
                            }
                            Err(error) => {
                                let valid_up_to = error.valid_up_to();
                                if valid_up_to == 0 {
                                    if error.error_len().is_none() {
                                        break;
                                    }
                                    let invalid_len = error.error_len().unwrap_or(1);
                                    pending.drain(..invalid_len.min(pending.len()));
                                    continue;
                                }
                                let chunk =
                                    String::from_utf8_lossy(&pending[..valid_up_to]).to_string();
                                if !chunk.is_empty() {
                                    let payload = TerminalOutput {
                                        workspace_id: workspace_id.clone(),
                                        terminal_id: terminal_id.clone(),
                                        data: chunk,
                                    };
                                    event_sink.emit_terminal_output(payload);
                                }
                                pending.drain(..valid_up_to);
                                if error.error_len().is_none() {
                                    break;
                                }
                                let invalid_len = error.error_len().unwrap_or(1);
                                pending.drain(..invalid_len.min(pending.len()));
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });
}

fn resolve_home_dir() -> Option<PathBuf> {
    if let Ok(value) = env::var("HOME") {
        if !value.trim().is_empty() {
            return Some(PathBuf::from(value));
        }
    }
    if let Ok(value) = env::var("USERPROFILE") {
        if !value.trim().is_empty() {
            return Some(PathBuf::from(value));
        }
    }
    None
}

fn resolve_codex_home() -> Option<PathBuf> {
    if let Ok(value) = env::var("CODEX_HOME") {
        if !value.trim().is_empty() {
            let path = PathBuf::from(value.trim());
            if path.exists() {
                return path.canonicalize().ok().or(Some(path));
            }
            return Some(path);
        }
    }
    resolve_home_dir().map(|home| home.join(".codex"))
}

fn default_prompts_dir() -> Option<PathBuf> {
    resolve_codex_home().map(|home| home.join("prompts"))
}

fn workspace_prompts_dir(data_dir: &Path, entry: &WorkspaceEntry) -> Result<PathBuf, String> {
    Ok(data_dir.join("workspaces").join(&entry.id).join("prompts"))
}

fn prompt_roots_for_workspace(
    data_dir: &Path,
    entry: &WorkspaceEntry,
) -> Result<Vec<PathBuf>, String> {
    let mut roots = Vec::new();
    roots.push(workspace_prompts_dir(data_dir, entry)?);
    if let Some(global_dir) = default_prompts_dir() {
        roots.push(global_dir);
    }
    Ok(roots)
}

fn ensure_path_within_roots(path: &Path, roots: &[PathBuf]) -> Result<(), String> {
    let canonical_path = path
        .canonicalize()
        .map_err(|_| "Invalid prompt path.".to_string())?;
    for root in roots {
        if let Ok(canonical_root) = root.canonicalize() {
            if canonical_path.starts_with(&canonical_root) {
                return Ok(());
            }
        }
    }
    Err("Prompt path is not within allowed directories.".to_string())
}

#[cfg(unix)]
fn is_cross_device_error(err: &std::io::Error) -> bool {
    err.raw_os_error() == Some(libc::EXDEV)
}

#[cfg(not(unix))]
fn is_cross_device_error(_err: &std::io::Error) -> bool {
    false
}

fn move_file(src: &Path, dest: &Path) -> Result<(), String> {
    match std::fs::rename(src, dest) {
        Ok(()) => Ok(()),
        Err(err) if is_cross_device_error(&err) => {
            std::fs::copy(src, dest).map_err(|err| err.to_string())?;
            std::fs::remove_file(src).map_err(|err| err.to_string())
        }
        Err(err) => Err(err.to_string()),
    }
}

fn parse_frontmatter(content: &str) -> (Option<String>, Option<String>, String) {
    let mut segments = content.split_inclusive('\n');
    let Some(first_segment) = segments.next() else {
        return (None, None, String::new());
    };
    let first_line = first_segment.trim_end_matches(['\r', '\n']);
    if first_line.trim() != "---" {
        return (None, None, content.to_string());
    }

    let mut description: Option<String> = None;
    let mut argument_hint: Option<String> = None;
    let mut frontmatter_closed = false;
    let mut consumed = first_segment.len();

    for segment in segments {
        let line = segment.trim_end_matches(['\r', '\n']);
        let trimmed = line.trim();

        if trimmed == "---" {
            frontmatter_closed = true;
            consumed += segment.len();
            break;
        }

        if trimmed.is_empty() || trimmed.starts_with('#') {
            consumed += segment.len();
            continue;
        }

        if let Some((key, value)) = trimmed.split_once(':') {
            let mut val = value.trim().to_string();
            if val.len() >= 2 {
                let bytes = val.as_bytes();
                let first = bytes[0];
                let last = bytes[bytes.len() - 1];
                if (first == b'\"' && last == b'\"') || (first == b'\'' && last == b'\'') {
                    val = val[1..val.len().saturating_sub(1)].to_string();
                }
            }
            match key.trim().to_ascii_lowercase().as_str() {
                "description" => description = Some(val),
                "argument-hint" | "argument_hint" => argument_hint = Some(val),
                _ => {}
            }
        }

        consumed += segment.len();
    }

    if !frontmatter_closed {
        return (None, None, content.to_string());
    }

    let body = if consumed >= content.len() {
        String::new()
    } else {
        content[consumed..].to_string()
    };
    (description, argument_hint, body)
}

fn build_prompt_contents(
    description: Option<String>,
    argument_hint: Option<String>,
    content: String,
) -> String {
    let has_meta = description
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty())
        || argument_hint
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());
    if !has_meta {
        return content;
    }
    let mut output = String::from("---\n");
    if let Some(description) = description {
        let trimmed = description.trim();
        if !trimmed.is_empty() {
            output.push_str(&format!(
                "description: \"{}\"\n",
                trimmed.replace('\"', "\\\"")
            ));
        }
    }
    if let Some(argument_hint) = argument_hint {
        let trimmed = argument_hint.trim();
        if !trimmed.is_empty() {
            output.push_str(&format!(
                "argument-hint: \"{}\"\n",
                trimmed.replace('\"', "\\\"")
            ));
        }
    }
    output.push_str("---\n");
    output.push_str(&content);
    output
}

fn sanitize_prompt_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Prompt name is required.".to_string());
    }
    if trimmed.chars().any(|ch| ch.is_whitespace()) {
        return Err("Prompt name cannot include whitespace.".to_string());
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err("Prompt name cannot include path separators.".to_string());
    }
    Ok(trimmed.to_string())
}

fn discover_prompts_in(dir: &Path, scope: Option<&str>) -> Vec<CustomPromptEntry> {
    let mut out: Vec<CustomPromptEntry> = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return out,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let is_file = std::fs::metadata(&path)
            .map(|m| m.is_file())
            .unwrap_or(false);
        if !is_file {
            continue;
        }
        let is_md = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false);
        if !is_md {
            continue;
        }
        let Some(name) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        let content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => continue,
        };
        let (description, argument_hint, body) = parse_frontmatter(&content);
        out.push(CustomPromptEntry {
            name,
            path: path.to_string_lossy().to_string(),
            description,
            argument_hint,
            content: body,
            scope: scope.map(|value| value.to_string()),
        });
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn action_paths_for_file(repo_root: &Path, path: &str) -> Vec<String> {
    let target = normalize_git_path(path).trim().to_string();
    if target.is_empty() {
        return Vec::new();
    }

    let repo = match Repository::open(repo_root) {
        Ok(repo) => repo,
        Err(_) => return vec![target],
    };

    let mut status_options = StatusOptions::new();
    status_options
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true)
        .include_ignored(false);

    let statuses = match repo.statuses(Some(&mut status_options)) {
        Ok(statuses) => statuses,
        Err(_) => return vec![target],
    };

    for entry in statuses.iter() {
        let status = entry.status();
        if !(status.contains(Status::WT_RENAMED) || status.contains(Status::INDEX_RENAMED)) {
            continue;
        }
        let delta = entry.index_to_workdir().or_else(|| entry.head_to_index());
        let Some(delta) = delta else {
            continue;
        };
        let (Some(old_path), Some(new_path)) = (delta.old_file().path(), delta.new_file().path())
        else {
            continue;
        };
        let old_path = normalize_git_path(old_path.to_string_lossy().as_ref());
        let new_path = normalize_git_path(new_path.to_string_lossy().as_ref());
        if old_path != target && new_path != target {
            continue;
        }
        if old_path == new_path || new_path.is_empty() {
            return vec![target];
        }
        let mut result = Vec::new();
        if !old_path.is_empty() {
            result.push(old_path);
        }
        if !new_path.is_empty() && !result.contains(&new_path) {
            result.push(new_path);
        }
        return if result.is_empty() {
            vec![target]
        } else {
            result
        };
    }

    vec![target]
}

fn parse_upstream_ref(name: &str) -> Option<(String, String)> {
    let trimmed = name.strip_prefix("refs/remotes/").unwrap_or(name);
    let mut parts = trimmed.splitn(2, '/');
    let remote = parts.next()?;
    let branch = parts.next()?;
    if remote.is_empty() || branch.is_empty() {
        return None;
    }
    Some((remote.to_string(), branch.to_string()))
}

fn upstream_remote_and_branch(repo_root: &Path) -> Result<Option<(String, String)>, String> {
    let repo = Repository::open(repo_root).map_err(|e| e.to_string())?;
    let head = match repo.head() {
        Ok(head) => head,
        Err(_) => return Ok(None),
    };
    if !head.is_branch() {
        return Ok(None);
    }
    let branch_name = match head.shorthand() {
        Some(name) => name,
        None => return Ok(None),
    };
    let branch = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(|e| e.to_string())?;
    let upstream_branch = match branch.upstream() {
        Ok(upstream) => upstream,
        Err(_) => return Ok(None),
    };
    let upstream_ref = upstream_branch.get();
    let upstream_name = upstream_ref.name().or_else(|| upstream_ref.shorthand());
    Ok(upstream_name.and_then(parse_upstream_ref))
}

async fn push_with_upstream(repo_root: &Path) -> Result<(), String> {
    let upstream = upstream_remote_and_branch(repo_root)?;
    if let Some((remote, branch)) = upstream {
        let refspec = format!("HEAD:{branch}");
        return run_git_command(repo_root, &["push", remote.as_str(), refspec.as_str()])
            .await
            .map(|_| ());
    }
    run_git_command(repo_root, &["push"]).await.map(|_| ())
}

fn status_for_index(status: Status) -> Option<&'static str> {
    if status.contains(Status::INDEX_NEW) {
        Some("A")
    } else if status.contains(Status::INDEX_MODIFIED) {
        Some("M")
    } else if status.contains(Status::INDEX_DELETED) {
        Some("D")
    } else if status.contains(Status::INDEX_RENAMED) {
        Some("R")
    } else if status.contains(Status::INDEX_TYPECHANGE) {
        Some("T")
    } else {
        None
    }
}

fn status_for_workdir(status: Status) -> Option<&'static str> {
    if status.contains(Status::WT_NEW) {
        Some("A")
    } else if status.contains(Status::WT_MODIFIED) {
        Some("M")
    } else if status.contains(Status::WT_DELETED) {
        Some("D")
    } else if status.contains(Status::WT_RENAMED) {
        Some("R")
    } else if status.contains(Status::WT_TYPECHANGE) {
        Some("T")
    } else {
        None
    }
}

fn status_for_delta(status: git2::Delta) -> &'static str {
    match status {
        git2::Delta::Added => "A",
        git2::Delta::Modified => "M",
        git2::Delta::Deleted => "D",
        git2::Delta::Renamed => "R",
        git2::Delta::Typechange => "T",
        _ => "M",
    }
}

fn build_combined_diff(diff: &git2::Diff) -> String {
    let mut combined_diff = String::new();
    for (index, delta) in diff.deltas().enumerate() {
        let path = delta.new_file().path().or_else(|| delta.old_file().path());
        let Some(path) = path else {
            continue;
        };
        let patch = match git2::Patch::from_diff(diff, index) {
            Ok(patch) => patch,
            Err(_) => continue,
        };
        let Some(mut patch) = patch else {
            continue;
        };
        let content = match diff_patch_to_string(&mut patch) {
            Ok(content) => content,
            Err(_) => continue,
        };
        if content.trim().is_empty() {
            continue;
        }
        if !combined_diff.is_empty() {
            combined_diff.push_str("\n\n");
        }
        combined_diff.push_str(&format!("=== {} ===\n", path.display()));
        combined_diff.push_str(&content);
    }
    combined_diff
}

fn collect_workspace_diff(repo_root: &Path) -> Result<String, String> {
    let repo = Repository::open(repo_root).map_err(|e| e.to_string())?;
    let head_tree = repo.head().ok().and_then(|head| head.peel_to_tree().ok());

    let mut options = DiffOptions::new();
    let index = repo.index().map_err(|e| e.to_string())?;
    let diff = match head_tree.as_ref() {
        Some(tree) => repo
            .diff_tree_to_index(Some(tree), Some(&index), Some(&mut options))
            .map_err(|e| e.to_string())?,
        None => repo
            .diff_tree_to_index(None, Some(&index), Some(&mut options))
            .map_err(|e| e.to_string())?,
    };
    let combined_diff = build_combined_diff(&diff);
    if !combined_diff.trim().is_empty() {
        return Ok(combined_diff);
    }

    let mut options = DiffOptions::new();
    options
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .show_untracked_content(true);
    let diff = match head_tree.as_ref() {
        Some(tree) => repo
            .diff_tree_to_workdir_with_index(Some(tree), Some(&mut options))
            .map_err(|e| e.to_string())?,
        None => repo
            .diff_tree_to_workdir_with_index(None, Some(&mut options))
            .map_err(|e| e.to_string())?,
    };
    Ok(build_combined_diff(&diff))
}

fn github_repo_from_path(path: &Path) -> Result<String, String> {
    let repo = Repository::open(path).map_err(|e| e.to_string())?;
    let remotes = repo.remotes().map_err(|e| e.to_string())?;
    let name = if remotes.iter().any(|remote| remote == Some("origin")) {
        "origin".to_string()
    } else {
        remotes.iter().flatten().next().unwrap_or("").to_string()
    };
    if name.is_empty() {
        return Err("No git remote configured.".to_string());
    }
    let remote = repo.find_remote(&name).map_err(|e| e.to_string())?;
    let remote_url = remote.url().ok_or("Remote has no URL configured.")?;
    parse_github_repo(remote_url).ok_or("Remote is not a GitHub repository.".to_string())
}

fn parse_pr_diff(diff: &str) -> Vec<GitHubPullRequestDiff> {
    let mut entries = Vec::new();
    let mut current_lines: Vec<&str> = Vec::new();
    let mut current_old_path: Option<String> = None;
    let mut current_new_path: Option<String> = None;
    let mut current_status: Option<String> = None;

    let finalize = |lines: &Vec<&str>,
                    old_path: &Option<String>,
                    new_path: &Option<String>,
                    status: &Option<String>,
                    results: &mut Vec<GitHubPullRequestDiff>| {
        if lines.is_empty() {
            return;
        }
        let diff_text = lines.join("\n");
        if diff_text.trim().is_empty() {
            return;
        }
        let status_value = status.clone().unwrap_or_else(|| "M".to_string());
        let path = if status_value == "D" {
            old_path.clone().unwrap_or_default()
        } else {
            new_path
                .clone()
                .or_else(|| old_path.clone())
                .unwrap_or_default()
        };
        if path.is_empty() {
            return;
        }
        results.push(GitHubPullRequestDiff {
            path: normalize_git_path(&path),
            status: status_value,
            diff: diff_text,
        });
    };

    for line in diff.lines() {
        if line.starts_with("diff --git ") {
            finalize(
                &current_lines,
                &current_old_path,
                &current_new_path,
                &current_status,
                &mut entries,
            );
            current_lines = vec![line];
            current_old_path = None;
            current_new_path = None;
            current_status = None;

            let rest = line.trim_start_matches("diff --git ").trim();
            let mut parts = rest.split_whitespace();
            let old_part = parts.next().unwrap_or("").trim_start_matches("a/");
            let new_part = parts.next().unwrap_or("").trim_start_matches("b/");
            if !old_part.is_empty() {
                current_old_path = Some(old_part.to_string());
            }
            if !new_part.is_empty() {
                current_new_path = Some(new_part.to_string());
            }
            continue;
        }
        if line.starts_with("new file mode ") {
            current_status = Some("A".to_string());
        } else if line.starts_with("deleted file mode ") {
            current_status = Some("D".to_string());
        } else if line.starts_with("rename from ") {
            current_status = Some("R".to_string());
            let path = line.trim_start_matches("rename from ").trim();
            if !path.is_empty() {
                current_old_path = Some(path.to_string());
            }
        } else if line.starts_with("rename to ") {
            current_status = Some("R".to_string());
            let path = line.trim_start_matches("rename to ").trim();
            if !path.is_empty() {
                current_new_path = Some(path.to_string());
            }
        }
        current_lines.push(line);
    }

    finalize(
        &current_lines,
        &current_old_path,
        &current_new_path,
        &current_status,
        &mut entries,
    );

    entries
}

impl DaemonState {
    async fn workspace_entry(&self, workspace_id: &str) -> Result<WorkspaceEntry, String> {
        let workspaces = self.workspaces.lock().await;
        workspaces
            .get(workspace_id)
            .cloned()
            .ok_or("workspace not found".to_string())
    }

    async fn workspace_entry_with_parent(
        &self,
        workspace_id: &str,
    ) -> Result<(WorkspaceEntry, Option<WorkspaceEntry>), String> {
        let workspaces = self.workspaces.lock().await;
        let entry = workspaces
            .get(workspace_id)
            .cloned()
            .ok_or("workspace not found".to_string())?;
        let parent_entry = entry
            .parent_id
            .as_ref()
            .and_then(|parent_id| workspaces.get(parent_id))
            .cloned();
        Ok((entry, parent_entry))
    }
}

impl DaemonState {
    async fn add_clone(
        &self,
        source_workspace_id: String,
        copy_name: String,
        copies_folder: String,
        client_version: String,
    ) -> Result<WorkspaceInfo, String> {
        let copy_name = copy_name.trim().to_string();
        if copy_name.is_empty() {
            return Err("Copy name is required.".to_string());
        }

        let copies_folder = copies_folder.trim().to_string();
        if copies_folder.is_empty() {
            return Err("Copies folder is required.".to_string());
        }
        let copies_folder_path = PathBuf::from(&copies_folder);
        std::fs::create_dir_all(&copies_folder_path)
            .map_err(|e| format!("Failed to create copies folder: {e}"))?;
        if !copies_folder_path.is_dir() {
            return Err("Copies folder must be a directory.".to_string());
        }

        let (source_entry, inherited_group_id) = {
            let workspaces = self.workspaces.lock().await;
            let source_entry = workspaces
                .get(&source_workspace_id)
                .cloned()
                .ok_or("source workspace not found")?;
            let inherited_group_id = if source_entry.kind.is_worktree() {
                source_entry
                    .parent_id
                    .as_ref()
                    .and_then(|parent_id| workspaces.get(parent_id))
                    .and_then(|parent| parent.settings.group_id.clone())
            } else {
                source_entry.settings.group_id.clone()
            };
            (source_entry, inherited_group_id)
        };

        let destination_path = build_clone_destination_path(&copies_folder_path, &copy_name)?;
        let destination_path_string = destination_path.to_string_lossy().to_string();

        if let Err(error) = run_git_command(
            &copies_folder_path,
            &["clone", &source_entry.path, &destination_path_string],
        )
        .await
        {
            let _ = tokio::fs::remove_dir_all(&destination_path).await;
            return Err(error);
        }

        if let Some(origin_url) = git_get_origin_url(&PathBuf::from(&source_entry.path)).await {
            let _ = run_git_command(
                &destination_path,
                &["remote", "set-url", "origin", &origin_url],
            )
            .await;
        }

        let entry = WorkspaceEntry {
            id: Uuid::new_v4().to_string(),
            name: copy_name.clone(),
            path: destination_path_string,
            codex_bin: source_entry.codex_bin.clone(),
            kind: WorkspaceKind::Main,
            parent_id: None,
            worktree: None,
            settings: WorkspaceSettings {
                group_id: inherited_group_id,
                ..WorkspaceSettings::default()
            },
        };

        let default_bin = {
            let settings = self.app_settings.lock().await;
            settings.codex_bin.clone()
        };
        let codex_home = codex_home::resolve_workspace_codex_home(&entry, None);
        let codex_args = {
            let settings = self.app_settings.lock().await;
            codex_args::resolve_workspace_codex_args(&entry, None, Some(&settings))
        };
        let session = match spawn_workspace_session(
            entry.clone(),
            default_bin,
            codex_args,
            codex_home,
            client_version,
            self.event_sink.clone(),
        )
        .await
        {
            Ok(session) => session,
            Err(error) => {
                let _ = tokio::fs::remove_dir_all(&destination_path).await;
                return Err(error);
            }
        };

        if let Err(error) = {
            let mut workspaces = self.workspaces.lock().await;
            workspaces.insert(entry.id.clone(), entry.clone());
            let list: Vec<_> = workspaces.values().cloned().collect();
            write_workspaces(&self.storage_path, &list)
        } {
            {
                let mut workspaces = self.workspaces.lock().await;
                workspaces.remove(&entry.id);
            }
            let mut child = session.child.lock().await;
            let _ = child.kill().await;
            let _ = tokio::fs::remove_dir_all(&destination_path).await;
            return Err(error);
        }

        self.sessions.lock().await.insert(entry.id.clone(), session);

        Ok(WorkspaceInfo {
            id: entry.id,
            name: entry.name,
            path: entry.path,
            connected: true,
            codex_bin: entry.codex_bin,
            kind: entry.kind,
            parent_id: entry.parent_id,
            worktree: entry.worktree,
            settings: entry.settings,
        })
    }

    async fn apply_worktree_changes(&self, workspace_id: String) -> Result<(), String> {
        let (entry, parent) = {
            let workspaces = self.workspaces.lock().await;
            let entry = workspaces
                .get(&workspace_id)
                .cloned()
                .ok_or("workspace not found")?;
            if !entry.kind.is_worktree() {
                return Err("Not a worktree workspace.".to_string());
            }
            let parent_id = entry.parent_id.clone().ok_or("worktree parent not found")?;
            let parent = workspaces
                .get(&parent_id)
                .cloned()
                .ok_or("worktree parent not found")?;
            (entry, parent)
        };

        let worktree_root = resolve_git_root(&entry)?;
        let parent_root = resolve_git_root(&parent)?;

        let parent_status = run_git_command_bytes(&parent_root, &["status", "--porcelain"]).await?;
        if !String::from_utf8_lossy(&parent_status).trim().is_empty() {
            return Err(
                "Your current branch has uncommitted changes. Please commit, stash, or discard them before applying worktree changes."
                    .to_string(),
            );
        }

        let mut patch: Vec<u8> = Vec::new();
        let staged_patch = run_git_diff(
            &worktree_root,
            &["diff", "--binary", "--no-color", "--cached"],
        )
        .await?;
        patch.extend_from_slice(&staged_patch);
        let unstaged_patch =
            run_git_diff(&worktree_root, &["diff", "--binary", "--no-color"]).await?;
        patch.extend_from_slice(&unstaged_patch);

        let untracked_output = run_git_command_bytes(
            &worktree_root,
            &["ls-files", "--others", "--exclude-standard", "-z"],
        )
        .await?;
        for raw_path in untracked_output.split(|byte| *byte == 0) {
            if raw_path.is_empty() {
                continue;
            }
            let path = String::from_utf8_lossy(raw_path).to_string();
            let diff = run_git_diff(
                &worktree_root,
                &[
                    "diff",
                    "--binary",
                    "--no-color",
                    "--no-index",
                    "--",
                    null_device_path(),
                    &path,
                ],
            )
            .await?;
            patch.extend_from_slice(&diff);
        }

        if String::from_utf8_lossy(&patch).trim().is_empty() {
            return Err("No changes to apply.".to_string());
        }

        let git_bin = resolve_git_binary().map_err(|e| format!("Failed to run git: {e}"))?;
        let mut child = Command::new(git_bin)
            .args(["apply", "--3way", "--whitespace=nowarn", "-"])
            .current_dir(&parent_root)
            .env("PATH", git_env_path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to run git: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(&patch)
                .await
                .map_err(|e| format!("Failed to write git apply input: {e}"))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| format!("Failed to run git: {e}"))?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        if detail.is_empty() {
            return Err("Git apply failed.".to_string());
        }

        if detail.contains("Applied patch to") {
            if detail.contains("with conflicts") {
                return Err(
                    "Applied with conflicts. Resolve conflicts in the parent repo before retrying."
                        .to_string(),
                );
            }
            return Err(
                "Patch applied partially. Resolve changes in the parent repo before retrying."
                    .to_string(),
            );
        }

        Err(detail.to_string())
    }
}

impl DaemonState {
    async fn workspace_path(&self, workspace_id: &str) -> Result<PathBuf, String> {
        let entry = self.workspace_entry(workspace_id).await?;
        Ok(PathBuf::from(&entry.path))
    }

    async fn terminal_open(
        &self,
        workspace_id: String,
        terminal_id: String,
        cols: u16,
        rows: u16,
    ) -> Result<TerminalSessionInfo, String> {
        if terminal_id.is_empty() {
            return Err("Terminal id is required".to_string());
        }
        let key = terminal_key(&workspace_id, &terminal_id);
        {
            let sessions = self.terminal_sessions.lock().await;
            if let Some(existing) = sessions.get(&key) {
                return Ok(TerminalSessionInfo {
                    id: existing.id.clone(),
                });
            }
        }

        let cwd = self.workspace_path(&workspace_id).await?;
        let pty_system = native_pty_system();
        let size = PtySize {
            rows: rows.max(2),
            cols: cols.max(2),
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = pty_system
            .openpty(size)
            .map_err(|e| format!("Failed to open pty: {e}"))?;

        let mut cmd = CommandBuilder::new(shell_path());
        cmd.cwd(cwd);
        cmd.arg("-i");
        cmd.env("TERM", "xterm-256color");
        let locale = resolve_locale();
        cmd.env("LANG", &locale);
        cmd.env("LC_ALL", &locale);
        cmd.env("LC_CTYPE", &locale);

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {e}"))?;
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to open pty reader: {e}"))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to open pty writer: {e}"))?;

        let session = Arc::new(TerminalSession {
            id: terminal_id.clone(),
            master: Mutex::new(pair.master),
            writer: Mutex::new(writer),
            child: Mutex::new(child),
        });
        let session_id = session.id.clone();

        {
            let mut sessions = self.terminal_sessions.lock().await;
            if let Some(existing) = sessions.get(&key) {
                let mut child = session.child.lock().await;
                let _ = child.kill();
                return Ok(TerminalSessionInfo {
                    id: existing.id.clone(),
                });
            }
            sessions.insert(key, session);
        }

        let event_sink = self.event_sink.clone();
        spawn_terminal_reader(event_sink, workspace_id, terminal_id, reader);

        Ok(TerminalSessionInfo { id: session_id })
    }

    async fn terminal_write(
        &self,
        workspace_id: String,
        terminal_id: String,
        data: String,
    ) -> Result<(), String> {
        let key = terminal_key(&workspace_id, &terminal_id);
        let sessions = self.terminal_sessions.lock().await;
        let session = sessions
            .get(&key)
            .ok_or_else(|| "Terminal session not found".to_string())?;
        let mut writer = session.writer.lock().await;
        writer
            .write_all(data.as_bytes())
            .map_err(|e| format!("Failed to write to pty: {e}"))?;
        writer
            .flush()
            .map_err(|e| format!("Failed to flush pty: {e}"))?;
        Ok(())
    }

    async fn terminal_resize(
        &self,
        workspace_id: String,
        terminal_id: String,
        cols: u16,
        rows: u16,
    ) -> Result<(), String> {
        let key = terminal_key(&workspace_id, &terminal_id);
        let sessions = self.terminal_sessions.lock().await;
        let session = sessions
            .get(&key)
            .ok_or_else(|| "Terminal session not found".to_string())?;
        let size = PtySize {
            rows: rows.max(2),
            cols: cols.max(2),
            pixel_width: 0,
            pixel_height: 0,
        };
        let master = session.master.lock().await;
        master
            .resize(size)
            .map_err(|e| format!("Failed to resize pty: {e}"))?;
        Ok(())
    }

    async fn terminal_close(
        &self,
        workspace_id: String,
        terminal_id: String,
    ) -> Result<(), String> {
        let key = terminal_key(&workspace_id, &terminal_id);
        let mut sessions = self.terminal_sessions.lock().await;
        let session = sessions
            .remove(&key)
            .ok_or_else(|| "Terminal session not found".to_string())?;
        let mut child = session.child.lock().await;
        let _ = child.kill();
        Ok(())
    }
}

impl DaemonState {
    async fn prompts_list(&self, workspace_id: String) -> Result<Vec<CustomPromptEntry>, String> {
        let (workspace_dir, global_dir) = {
            let workspaces = self.workspaces.lock().await;
            let entry = workspaces.get(&workspace_id).cloned();
            let workspace_dir = entry
                .as_ref()
                .and_then(|entry| workspace_prompts_dir(&self.data_dir, entry).ok());
            (workspace_dir, default_prompts_dir())
        };

        task::spawn_blocking(move || {
            let mut out = Vec::new();
            if let Some(dir) = workspace_dir {
                let _ = std::fs::create_dir_all(&dir);
                out.extend(discover_prompts_in(&dir, Some("workspace")));
            }
            if let Some(dir) = global_dir {
                let _ = std::fs::create_dir_all(&dir);
                out.extend(discover_prompts_in(&dir, Some("global")));
            }
            out
        })
        .await
        .map_err(|_| "prompt discovery failed".to_string())
    }

    async fn prompts_workspace_dir(&self, workspace_id: String) -> Result<String, String> {
        let dir = {
            let workspaces = self.workspaces.lock().await;
            let entry = workspaces
                .get(&workspace_id)
                .cloned()
                .ok_or("workspace not found")?;
            workspace_prompts_dir(&self.data_dir, &entry)?
        };
        std::fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
        Ok(dir.to_string_lossy().to_string())
    }

    async fn prompts_global_dir(&self) -> Result<String, String> {
        let dir = default_prompts_dir().ok_or("Unable to resolve CODEX_HOME".to_string())?;
        std::fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
        Ok(dir.to_string_lossy().to_string())
    }

    async fn prompts_create(
        &self,
        workspace_id: String,
        scope: String,
        name: String,
        description: Option<String>,
        argument_hint: Option<String>,
        content: String,
    ) -> Result<CustomPromptEntry, String> {
        let name = sanitize_prompt_name(&name)?;
        let (target_dir, resolved_scope) = {
            let workspaces = self.workspaces.lock().await;
            let entry = workspaces
                .get(&workspace_id)
                .cloned()
                .ok_or("workspace not found")?;
            match scope.as_str() {
                "workspace" => {
                    let dir = workspace_prompts_dir(&self.data_dir, &entry)?;
                    (dir, "workspace")
                }
                "global" => {
                    let dir =
                        default_prompts_dir().ok_or("Unable to resolve CODEX_HOME".to_string())?;
                    (dir, "global")
                }
                _ => return Err("Invalid scope.".to_string()),
            }
        };
        let path = target_dir.join(format!("{name}.md"));
        if path.exists() {
            return Err("Prompt already exists.".to_string());
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        let body =
            build_prompt_contents(description.clone(), argument_hint.clone(), content.clone());
        std::fs::write(&path, body).map_err(|err| err.to_string())?;
        Ok(CustomPromptEntry {
            name,
            path: path.to_string_lossy().to_string(),
            description,
            argument_hint,
            content,
            scope: Some(resolved_scope.to_string()),
        })
    }

    async fn prompts_update(
        &self,
        workspace_id: String,
        path: String,
        name: String,
        description: Option<String>,
        argument_hint: Option<String>,
        content: String,
    ) -> Result<CustomPromptEntry, String> {
        let name = sanitize_prompt_name(&name)?;
        let target_path = PathBuf::from(&path);
        if !target_path.exists() {
            return Err("Prompt not found.".to_string());
        }
        {
            let workspaces = self.workspaces.lock().await;
            let entry = workspaces
                .get(&workspace_id)
                .cloned()
                .ok_or("workspace not found")?;
            let roots = prompt_roots_for_workspace(&self.data_dir, &entry)?;
            ensure_path_within_roots(&target_path, &roots)?;
        }
        let dir = target_path
            .parent()
            .ok_or("Unable to resolve prompt directory.".to_string())?;
        let next_path = dir.join(format!("{name}.md"));
        if next_path != target_path && next_path.exists() {
            return Err("Prompt with that name already exists.".to_string());
        }
        let body =
            build_prompt_contents(description.clone(), argument_hint.clone(), content.clone());
        std::fs::write(&next_path, body).map_err(|err| err.to_string())?;
        if next_path != target_path {
            std::fs::remove_file(&target_path).map_err(|err| err.to_string())?;
        }
        let scope = {
            let workspaces = self.workspaces.lock().await;
            let entry = workspaces
                .get(&workspace_id)
                .cloned()
                .ok_or("workspace not found")?;
            let workspace_dir = workspace_prompts_dir(&self.data_dir, &entry)?;
            if next_path.starts_with(&workspace_dir) {
                Some("workspace".to_string())
            } else {
                Some("global".to_string())
            }
        };
        Ok(CustomPromptEntry {
            name,
            path: next_path.to_string_lossy().to_string(),
            description,
            argument_hint,
            content,
            scope,
        })
    }

    async fn prompts_delete(&self, workspace_id: String, path: String) -> Result<(), String> {
        let target = PathBuf::from(path);
        if !target.exists() {
            return Ok(());
        }
        {
            let workspaces = self.workspaces.lock().await;
            let entry = workspaces
                .get(&workspace_id)
                .cloned()
                .ok_or("workspace not found")?;
            let roots = prompt_roots_for_workspace(&self.data_dir, &entry)?;
            ensure_path_within_roots(&target, &roots)?;
        }
        std::fs::remove_file(&target).map_err(|err| err.to_string())
    }

    async fn prompts_move(
        &self,
        workspace_id: String,
        path: String,
        scope: String,
    ) -> Result<CustomPromptEntry, String> {
        let target_path = PathBuf::from(&path);
        if !target_path.exists() {
            return Err("Prompt not found.".to_string());
        }
        let roots = {
            let workspaces = self.workspaces.lock().await;
            let entry = workspaces
                .get(&workspace_id)
                .cloned()
                .ok_or("workspace not found")?;
            prompt_roots_for_workspace(&self.data_dir, &entry)?
        };
        ensure_path_within_roots(&target_path, &roots)?;
        let file_name = target_path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or("Invalid prompt path.".to_string())?;
        let target_dir = {
            let workspaces = self.workspaces.lock().await;
            let entry = workspaces
                .get(&workspace_id)
                .cloned()
                .ok_or("workspace not found")?;
            match scope.as_str() {
                "workspace" => workspace_prompts_dir(&self.data_dir, &entry)?,
                "global" => {
                    default_prompts_dir().ok_or("Unable to resolve CODEX_HOME".to_string())?
                }
                _ => return Err("Invalid scope.".to_string()),
            }
        };
        let next_path = target_dir.join(file_name);
        if next_path == target_path {
            return Err("Prompt is already in that scope.".to_string());
        }
        if next_path.exists() {
            return Err("Prompt with that name already exists.".to_string());
        }
        if let Some(parent) = next_path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        move_file(&target_path, &next_path)?;
        let content = std::fs::read_to_string(&next_path).unwrap_or_default();
        let (description, argument_hint, body) = parse_frontmatter(&content);
        let name = next_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_string();
        Ok(CustomPromptEntry {
            name,
            path: next_path.to_string_lossy().to_string(),
            description,
            argument_hint,
            content: body,
            scope: Some(scope),
        })
    }
}

impl DaemonState {
    async fn list_git_roots(
        &self,
        workspace_id: String,
        depth: Option<usize>,
    ) -> Result<Vec<String>, String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let root = PathBuf::from(&entry.path);
        let depth = depth.unwrap_or(2).clamp(1, 6);
        Ok(scan_git_roots(&root, depth, 200))
    }

    async fn get_workspace_diff(&self, workspace_id: &str) -> Result<String, String> {
        let entry = self.workspace_entry(workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        collect_workspace_diff(&repo_root)
    }

    async fn get_git_status(&self, workspace_id: String) -> Result<Value, String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        let repo = Repository::open(&repo_root).map_err(|e| e.to_string())?;

        let branch_name = repo
            .head()
            .ok()
            .and_then(|head| head.shorthand().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        let mut status_options = StatusOptions::new();
        status_options
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true)
            .include_ignored(false);

        let statuses = repo
            .statuses(Some(&mut status_options))
            .map_err(|e| e.to_string())?;

        let head_tree = repo.head().ok().and_then(|head| head.peel_to_tree().ok());
        let index = repo.index().ok();

        let mut files = Vec::new();
        let mut staged_files = Vec::new();
        let mut unstaged_files = Vec::new();
        let mut total_additions = 0i64;
        let mut total_deletions = 0i64;
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("");
            if path.is_empty() {
                continue;
            }
            if let Some(index) = index.as_ref() {
                if let Some(entry) = index.get_path(Path::new(path), 0) {
                    if entry.flags_extended & INDEX_SKIP_WORKTREE_FLAG != 0 {
                        continue;
                    }
                }
            }
            let status = entry.status();
            let normalized_path = normalize_git_path(path);
            let include_index = status.intersects(
                Status::INDEX_NEW
                    | Status::INDEX_MODIFIED
                    | Status::INDEX_DELETED
                    | Status::INDEX_RENAMED
                    | Status::INDEX_TYPECHANGE,
            );
            let include_workdir = status.intersects(
                Status::WT_NEW
                    | Status::WT_MODIFIED
                    | Status::WT_DELETED
                    | Status::WT_RENAMED
                    | Status::WT_TYPECHANGE,
            );
            let mut combined_additions = 0i64;
            let mut combined_deletions = 0i64;

            if include_index {
                let (additions, deletions) =
                    diff_stats_for_path(&repo, head_tree.as_ref(), path, true, false)
                        .unwrap_or((0, 0));
                if let Some(status_str) = status_for_index(status) {
                    staged_files.push(GitFileStatus {
                        path: normalized_path.clone(),
                        status: status_str.to_string(),
                        additions,
                        deletions,
                    });
                }
                combined_additions += additions;
                combined_deletions += deletions;
                total_additions += additions;
                total_deletions += deletions;
            }

            if include_workdir {
                let (additions, deletions) =
                    diff_stats_for_path(&repo, head_tree.as_ref(), path, false, true)
                        .unwrap_or((0, 0));
                if let Some(status_str) = status_for_workdir(status) {
                    unstaged_files.push(GitFileStatus {
                        path: normalized_path.clone(),
                        status: status_str.to_string(),
                        additions,
                        deletions,
                    });
                }
                combined_additions += additions;
                combined_deletions += deletions;
                total_additions += additions;
                total_deletions += deletions;
            }

            if include_index || include_workdir {
                let status_str = status_for_workdir(status)
                    .or_else(|| status_for_index(status))
                    .unwrap_or("--");
                files.push(GitFileStatus {
                    path: normalized_path,
                    status: status_str.to_string(),
                    additions: combined_additions,
                    deletions: combined_deletions,
                });
            }
        }

        Ok(json!({
            "branchName": branch_name,
            "files": files,
            "stagedFiles": staged_files,
            "unstagedFiles": unstaged_files,
            "totalAdditions": total_additions,
            "totalDeletions": total_deletions,
        }))
    }

    async fn get_git_diffs(&self, workspace_id: String) -> Result<Vec<GitFileDiff>, String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        let repo = Repository::open(&repo_root).map_err(|e| e.to_string())?;
        let head_tree = repo.head().ok().and_then(|head| head.peel_to_tree().ok());

        let mut options = DiffOptions::new();
        options
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .show_untracked_content(true);

        let diff = match head_tree.as_ref() {
            Some(tree) => repo
                .diff_tree_to_workdir_with_index(Some(tree), Some(&mut options))
                .map_err(|e| e.to_string())?,
            None => repo
                .diff_tree_to_workdir_with_index(None, Some(&mut options))
                .map_err(|e| e.to_string())?,
        };

        let mut results = Vec::new();
        for (index, delta) in diff.deltas().enumerate() {
            let path = delta.new_file().path().or_else(|| delta.old_file().path());
            let Some(path) = path else {
                continue;
            };
            let patch = match git2::Patch::from_diff(&diff, index) {
                Ok(patch) => patch,
                Err(_) => continue,
            };
            let Some(mut patch) = patch else {
                continue;
            };
            let content = match diff_patch_to_string(&mut patch) {
                Ok(content) => content,
                Err(_) => continue,
            };
            if content.trim().is_empty() {
                continue;
            }
            results.push(GitFileDiff {
                path: normalize_git_path(path.to_string_lossy().as_ref()),
                diff: content,
                is_binary: false,
                is_image: false,
                old_image_data: None,
                new_image_data: None,
                old_image_mime: None,
                new_image_mime: None,
            });
        }

        Ok(results)
    }

    async fn get_git_log(
        &self,
        workspace_id: String,
        limit: Option<usize>,
    ) -> Result<GitLogResponse, String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        let repo = Repository::open(&repo_root).map_err(|e| e.to_string())?;
        let max_items = limit.unwrap_or(40);
        let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
        revwalk.push_head().map_err(|e| e.to_string())?;
        revwalk.set_sorting(Sort::TIME).map_err(|e| e.to_string())?;

        let mut total = 0usize;
        for oid_result in revwalk {
            oid_result.map_err(|e| e.to_string())?;
            total += 1;
        }

        let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
        revwalk.push_head().map_err(|e| e.to_string())?;
        revwalk.set_sorting(Sort::TIME).map_err(|e| e.to_string())?;

        let mut entries = Vec::new();
        for oid_result in revwalk.take(max_items) {
            let oid = oid_result.map_err(|e| e.to_string())?;
            let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
            entries.push(commit_to_entry(commit));
        }

        let mut ahead = 0usize;
        let mut behind = 0usize;
        let mut ahead_entries = Vec::new();
        let mut behind_entries = Vec::new();
        let mut upstream = None;

        if let Ok(head) = repo.head() {
            if head.is_branch() {
                if let Some(branch_name) = head.shorthand() {
                    if let Ok(branch) = repo.find_branch(branch_name, BranchType::Local) {
                        if let Ok(upstream_branch) = branch.upstream() {
                            let upstream_ref = upstream_branch.get();
                            upstream = upstream_ref
                                .shorthand()
                                .map(|name| name.to_string())
                                .or_else(|| upstream_ref.name().map(|name| name.to_string()));
                            if let (Some(head_oid), Some(upstream_oid)) =
                                (head.target(), upstream_ref.target())
                            {
                                let (ahead_count, behind_count) = repo
                                    .graph_ahead_behind(head_oid, upstream_oid)
                                    .map_err(|e| e.to_string())?;
                                ahead = ahead_count;
                                behind = behind_count;

                                let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
                                revwalk.push(head_oid).map_err(|e| e.to_string())?;
                                revwalk.hide(upstream_oid).map_err(|e| e.to_string())?;
                                revwalk.set_sorting(Sort::TIME).map_err(|e| e.to_string())?;
                                for oid_result in revwalk.take(max_items) {
                                    let oid = oid_result.map_err(|e| e.to_string())?;
                                    let commit =
                                        repo.find_commit(oid).map_err(|e| e.to_string())?;
                                    ahead_entries.push(commit_to_entry(commit));
                                }

                                let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
                                revwalk.push(upstream_oid).map_err(|e| e.to_string())?;
                                revwalk.hide(head_oid).map_err(|e| e.to_string())?;
                                revwalk.set_sorting(Sort::TIME).map_err(|e| e.to_string())?;
                                for oid_result in revwalk.take(max_items) {
                                    let oid = oid_result.map_err(|e| e.to_string())?;
                                    let commit =
                                        repo.find_commit(oid).map_err(|e| e.to_string())?;
                                    behind_entries.push(commit_to_entry(commit));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(GitLogResponse {
            total,
            entries,
            ahead,
            behind,
            ahead_entries,
            behind_entries,
            upstream,
        })
    }

    async fn get_git_commit_diff(
        &self,
        workspace_id: String,
        sha: String,
    ) -> Result<Vec<GitCommitDiff>, String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        let repo = Repository::open(&repo_root).map_err(|e| e.to_string())?;
        let oid = git2::Oid::from_str(&sha).map_err(|e| e.to_string())?;
        let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
        let commit_tree = commit.tree().map_err(|e| e.to_string())?;
        let parent_tree = commit.parent(0).ok().and_then(|parent| parent.tree().ok());

        let mut options = DiffOptions::new();
        let diff = repo
            .diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), Some(&mut options))
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for (index, delta) in diff.deltas().enumerate() {
            let path = delta.new_file().path().or_else(|| delta.old_file().path());
            let Some(path) = path else {
                continue;
            };
            let patch = match git2::Patch::from_diff(&diff, index) {
                Ok(patch) => patch,
                Err(_) => continue,
            };
            let Some(mut patch) = patch else {
                continue;
            };
            let content = match diff_patch_to_string(&mut patch) {
                Ok(content) => content,
                Err(_) => continue,
            };
            if content.trim().is_empty() {
                continue;
            }
            results.push(GitCommitDiff {
                path: normalize_git_path(path.to_string_lossy().as_ref()),
                status: status_for_delta(delta.status()).to_string(),
                diff: content,
                is_binary: false,
                is_image: false,
                old_image_data: None,
                new_image_data: None,
                old_image_mime: None,
                new_image_mime: None,
            });
        }

        Ok(results)
    }

    async fn get_git_remote(&self, workspace_id: String) -> Result<Option<String>, String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        let repo = Repository::open(&repo_root).map_err(|e| e.to_string())?;
        let remotes = repo.remotes().map_err(|e| e.to_string())?;
        let name = if remotes.iter().any(|remote| remote == Some("origin")) {
            "origin".to_string()
        } else {
            remotes.iter().flatten().next().unwrap_or("").to_string()
        };
        if name.is_empty() {
            return Ok(None);
        }
        let remote = repo.find_remote(&name).map_err(|e| e.to_string())?;
        Ok(remote.url().map(|url| url.to_string()))
    }

    async fn list_git_branches(&self, workspace_id: String) -> Result<Value, String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        let repo = Repository::open(&repo_root).map_err(|e| e.to_string())?;
        let mut branches = Vec::new();
        let refs = repo
            .branches(Some(BranchType::Local))
            .map_err(|e| e.to_string())?;
        for branch_result in refs {
            let (branch, _) = branch_result.map_err(|e| e.to_string())?;
            let name = branch.name().ok().flatten().unwrap_or("").to_string();
            if name.is_empty() {
                continue;
            }
            let last_commit = branch
                .get()
                .target()
                .and_then(|oid| repo.find_commit(oid).ok())
                .map(|commit| commit.time().seconds())
                .unwrap_or(0);
            branches.push(BranchInfo { name, last_commit });
        }
        branches.sort_by(|a, b| b.last_commit.cmp(&a.last_commit));
        Ok(json!({ "branches": branches }))
    }

    async fn checkout_git_branch(&self, workspace_id: String, name: String) -> Result<(), String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        let repo = Repository::open(&repo_root).map_err(|e| e.to_string())?;
        checkout_branch(&repo, &name).map_err(|e| e.to_string())
    }

    async fn create_git_branch(&self, workspace_id: String, name: String) -> Result<(), String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        let repo = Repository::open(&repo_root).map_err(|e| e.to_string())?;
        let head = repo.head().map_err(|e| e.to_string())?;
        let target = head.peel_to_commit().map_err(|e| e.to_string())?;
        repo.branch(&name, &target, false)
            .map_err(|e| e.to_string())?;
        checkout_branch(&repo, &name).map_err(|e| e.to_string())
    }
}

impl DaemonState {
    async fn stage_git_file(&self, workspace_id: String, path: String) -> Result<(), String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        for path in action_paths_for_file(&repo_root, &path) {
            run_git_command(&repo_root, &["add", "-A", "--", &path])
                .await
                .map(|_| ())?;
        }
        Ok(())
    }

    async fn stage_git_all(&self, workspace_id: String) -> Result<(), String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        run_git_command(&repo_root, &["add", "-A"])
            .await
            .map(|_| ())
    }

    async fn unstage_git_file(&self, workspace_id: String, path: String) -> Result<(), String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        for path in action_paths_for_file(&repo_root, &path) {
            run_git_command(&repo_root, &["restore", "--staged", "--", &path])
                .await
                .map(|_| ())?;
        }
        Ok(())
    }

    async fn revert_git_file(&self, workspace_id: String, path: String) -> Result<(), String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        for path in action_paths_for_file(&repo_root, &path) {
            if run_git_command(
                &repo_root,
                &["restore", "--staged", "--worktree", "--", &path],
            )
            .await
            .is_ok()
            {
                continue;
            }
            run_git_command(&repo_root, &["clean", "-f", "--", &path])
                .await
                .map(|_| ())?;
        }
        Ok(())
    }

    async fn revert_git_all(&self, workspace_id: String) -> Result<(), String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        run_git_command(
            &repo_root,
            &["restore", "--staged", "--worktree", "--", "."],
        )
        .await
        .map(|_| ())?;
        run_git_command(&repo_root, &["clean", "-f", "-d"])
            .await
            .map(|_| ())
    }

    async fn commit_git(&self, workspace_id: String, message: String) -> Result<(), String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        run_git_command(&repo_root, &["commit", "-m", &message])
            .await
            .map(|_| ())
    }

    async fn push_git(&self, workspace_id: String) -> Result<(), String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        push_with_upstream(&repo_root).await
    }

    async fn pull_git(&self, workspace_id: String) -> Result<(), String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        run_git_command(&repo_root, &["pull"]).await.map(|_| ())
    }

    async fn sync_git(&self, workspace_id: String) -> Result<(), String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        run_git_command(&repo_root, &["pull"]).await.map(|_| ())?;
        push_with_upstream(&repo_root).await
    }
}

impl DaemonState {
    async fn get_github_issues(
        &self,
        workspace_id: String,
    ) -> Result<GitHubIssuesResponse, String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        let repo_name = github_repo_from_path(&repo_root)?;

        let output = Command::new("gh")
            .args([
                "issue",
                "list",
                "--repo",
                &repo_name,
                "--limit",
                "50",
                "--json",
                "number,title,url,updatedAt",
            ])
            .current_dir(&repo_root)
            .output()
            .await
            .map_err(|e| format!("Failed to run gh: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if stderr.trim().is_empty() {
                stdout.trim()
            } else {
                stderr.trim()
            };
            if detail.is_empty() {
                return Err("GitHub CLI command failed.".to_string());
            }
            return Err(detail.to_string());
        }

        let issues: Vec<GitHubIssue> =
            serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;

        let search_query = format!("repo:{repo_name} is:issue is:open");
        let search_query = search_query.replace(' ', "+");
        let total = match Command::new("gh")
            .args([
                "api",
                &format!("/search/issues?q={search_query}"),
                "--jq",
                ".total_count",
            ])
            .current_dir(&repo_root)
            .output()
            .await
        {
            Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<usize>()
                .unwrap_or(issues.len()),
            _ => issues.len(),
        };

        Ok(GitHubIssuesResponse { total, issues })
    }

    async fn get_github_pull_requests(
        &self,
        workspace_id: String,
    ) -> Result<GitHubPullRequestsResponse, String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        let repo_name = github_repo_from_path(&repo_root)?;

        let output = Command::new("gh")
            .args([
                "pr",
                "list",
                "--repo",
                &repo_name,
                "--state",
                "open",
                "--limit",
                "50",
                "--json",
                "number,title,url,updatedAt,createdAt,body,headRefName,baseRefName,isDraft,author",
            ])
            .current_dir(&repo_root)
            .output()
            .await
            .map_err(|e| format!("Failed to run gh: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if stderr.trim().is_empty() {
                stdout.trim()
            } else {
                stderr.trim()
            };
            if detail.is_empty() {
                return Err("GitHub CLI command failed.".to_string());
            }
            return Err(detail.to_string());
        }

        let pull_requests: Vec<GitHubPullRequest> =
            serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;

        let search_query = format!("repo:{repo_name} is:pr is:open");
        let search_query = search_query.replace(' ', "+");
        let total = match Command::new("gh")
            .args([
                "api",
                &format!("/search/issues?q={search_query}"),
                "--jq",
                ".total_count",
            ])
            .current_dir(&repo_root)
            .output()
            .await
        {
            Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<usize>()
                .unwrap_or(pull_requests.len()),
            _ => pull_requests.len(),
        };

        Ok(GitHubPullRequestsResponse {
            total,
            pull_requests,
        })
    }

    async fn get_github_pull_request_diff(
        &self,
        workspace_id: String,
        pr_number: u64,
    ) -> Result<Vec<GitHubPullRequestDiff>, String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        let repo_name = github_repo_from_path(&repo_root)?;

        let output = Command::new("gh")
            .args([
                "pr",
                "diff",
                &pr_number.to_string(),
                "--repo",
                &repo_name,
                "--color",
                "never",
            ])
            .current_dir(&repo_root)
            .output()
            .await
            .map_err(|e| format!("Failed to run gh: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if stderr.trim().is_empty() {
                stdout.trim()
            } else {
                stderr.trim()
            };
            if detail.is_empty() {
                return Err("GitHub CLI command failed.".to_string());
            }
            return Err(detail.to_string());
        }

        let diff_text = String::from_utf8_lossy(&output.stdout);
        Ok(parse_pr_diff(&diff_text))
    }

    async fn get_github_pull_request_comments(
        &self,
        workspace_id: String,
        pr_number: u64,
    ) -> Result<Vec<GitHubPullRequestComment>, String> {
        let entry = self.workspace_entry(&workspace_id).await?;
        let repo_root = resolve_git_root(&entry)?;
        let repo_name = github_repo_from_path(&repo_root)?;

        let comments_endpoint =
            format!("/repos/{repo_name}/issues/{pr_number}/comments?per_page=30");
        let jq_filter = r#"[.[] | {id, body, createdAt: .created_at, url: .html_url, author: (if .user then {login: .user.login} else null end)}]"#;

        let output = Command::new("gh")
            .args(["api", &comments_endpoint, "--jq", jq_filter])
            .current_dir(&repo_root)
            .output()
            .await
            .map_err(|e| format!("Failed to run gh: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if stderr.trim().is_empty() {
                stdout.trim()
            } else {
                stderr.trim()
            };
            if detail.is_empty() {
                return Err("GitHub CLI command failed.".to_string());
            }
            return Err(detail.to_string());
        }

        let comments: Vec<GitHubPullRequestComment> =
            serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;

        Ok(comments)
    }
}

impl DaemonState {
    async fn codex_doctor(&self, codex_bin: Option<String>) -> Result<Value, String> {
        let default_bin = {
            let settings = self.app_settings.lock().await;
            settings.codex_bin.clone()
        };
        let resolved = codex_bin
            .clone()
            .filter(|value| !value.trim().is_empty())
            .or(default_bin);
        let path_env = backend::app_server::build_codex_path_env(resolved.as_deref());
        let version = backend::app_server::check_codex_installation(resolved.clone()).await?;
        let mut command = backend::app_server::build_codex_command_with_bin(resolved.clone());
        command.arg("app-server");
        command.arg("--help");
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        let app_server_ok =
            match tokio::time::timeout(Duration::from_secs(5), command.output()).await {
                Ok(result) => result
                    .map(|output| output.status.success())
                    .unwrap_or(false),
                Err(_) => false,
            };
        let (node_ok, node_version, node_details) = {
            let mut node_command = Command::new("node");
            if let Some(ref path_env) = path_env {
                node_command.env("PATH", path_env);
            }
            node_command.arg("--version");
            node_command.stdout(std::process::Stdio::piped());
            node_command.stderr(std::process::Stdio::piped());
            match tokio::time::timeout(Duration::from_secs(5), node_command.output()).await {
                Ok(result) => match result {
                    Ok(output) => {
                        if output.status.success() {
                            let version =
                                String::from_utf8_lossy(&output.stdout).trim().to_string();
                            (
                                !version.is_empty(),
                                if version.is_empty() {
                                    None
                                } else {
                                    Some(version)
                                },
                                None,
                            )
                        } else {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let detail = if stderr.trim().is_empty() {
                                stdout.trim()
                            } else {
                                stderr.trim()
                            };
                            (
                                false,
                                None,
                                Some(if detail.is_empty() {
                                    "Node failed to start.".to_string()
                                } else {
                                    detail.to_string()
                                }),
                            )
                        }
                    }
                    Err(err) => {
                        if err.kind() == std::io::ErrorKind::NotFound {
                            (false, None, Some("Node not found on PATH.".to_string()))
                        } else {
                            (false, None, Some(err.to_string()))
                        }
                    }
                },
                Err(_) => (
                    false,
                    None,
                    Some("Timed out while checking Node.".to_string()),
                ),
            }
        };
        let details = if app_server_ok {
            None
        } else {
            Some("Failed to run `codex app-server --help`.".to_string())
        };
        Ok(json!({
            "ok": version.is_some() && app_server_ok,
            "codexBin": resolved,
            "version": version,
            "appServerOk": app_server_ok,
            "details": details,
            "path": path_env,
            "nodeOk": node_ok,
            "nodeVersion": node_version,
            "nodeDetails": node_details,
        }))
    }

    async fn get_commit_message_prompt(&self, workspace_id: String) -> Result<String, String> {
        let diff = self.get_workspace_diff(&workspace_id).await?;
        if diff.trim().is_empty() {
            return Err("No changes to generate commit message for".to_string());
        }
        let prompt = format!(
            "Generate a concise git commit message for the following changes. \
Follow conventional commit format (e.g., feat:, fix:, refactor:, docs:, etc.). \
Focus on the 'why' rather than the 'what'. Keep the summary line under 72 characters. \
Only output the commit message, nothing else.\n\n\
Changes:\n{diff}"
        );
        Ok(prompt)
    }

    async fn generate_commit_message(&self, workspace_id: String) -> Result<String, String> {
        let diff = self.get_workspace_diff(&workspace_id).await?;
        if diff.trim().is_empty() {
            return Err("No changes to generate commit message for".to_string());
        }

        let prompt = format!(
            "Generate a concise git commit message for the following changes. \
Follow conventional commit format (e.g., feat:, fix:, refactor:, docs:, etc.). \
Focus on the 'why' rather than the 'what'. Keep the summary line under 72 characters. \
Only output the commit message, nothing else.\n\n\
Changes:\n{diff}"
        );

        let session = {
            let sessions = self.sessions.lock().await;
            sessions
                .get(&workspace_id)
                .ok_or("workspace not connected")?
                .clone()
        };

        let thread_params = json!({
            "cwd": session.entry.path,
            "approvalPolicy": "never"
        });
        let thread_result = session.send_request("thread/start", thread_params).await?;

        if let Some(error) = thread_result.get("error") {
            let error_msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error starting thread");
            return Err(error_msg.to_string());
        }

        let thread_id = thread_result
            .get("result")
            .and_then(|r| r.get("threadId"))
            .or_else(|| {
                thread_result
                    .get("result")
                    .and_then(|r| r.get("thread"))
                    .and_then(|t| t.get("id"))
            })
            .or_else(|| thread_result.get("threadId"))
            .or_else(|| thread_result.get("thread").and_then(|t| t.get("id")))
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                format!(
                    "Failed to get threadId from thread/start response: {:?}",
                    thread_result
                )
            })?
            .to_string();

        let (tx, mut rx) = mpsc::unbounded_channel::<Value>();

        {
            let mut callbacks = session.background_thread_callbacks.lock().await;
            callbacks.insert(thread_id.clone(), tx);
        }

        let turn_params = build_turn_start_params(
            &thread_id,
            vec![json!({ "type": "text", "text": prompt })],
            &session.entry.path,
            "never",
            json!({ "type": "readOnly" }),
            None,
            None,
            None,
            None,
        );
        let turn_result = session.send_request("turn/start", turn_params).await;
        let turn_result = match turn_result {
            Ok(result) => result,
            Err(error) => {
                {
                    let mut callbacks = session.background_thread_callbacks.lock().await;
                    callbacks.remove(&thread_id);
                }
                let archive_params = json!({ "threadId": thread_id.as_str() });
                let _ = session.send_request("thread/archive", archive_params).await;
                return Err(error);
            }
        };

        if let Some(error) = turn_result.get("error") {
            let error_msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error starting turn");
            {
                let mut callbacks = session.background_thread_callbacks.lock().await;
                callbacks.remove(&thread_id);
            }
            let archive_params = json!({ "threadId": thread_id.as_str() });
            let _ = session.send_request("thread/archive", archive_params).await;
            return Err(error_msg.to_string());
        }

        let mut commit_message = String::new();
        let timeout_duration = Duration::from_secs(60);
        let collect_result = tokio::time::timeout(timeout_duration, async {
            while let Some(event) = rx.recv().await {
                let method = event.get("method").and_then(|m| m.as_str()).unwrap_or("");

                match method {
                    "item/agentMessage/delta" => {
                        if let Some(params) = event.get("params") {
                            if let Some(delta) = params.get("delta").and_then(|d| d.as_str()) {
                                commit_message.push_str(delta);
                            }
                        }
                    }
                    "turn/completed" => {
                        break;
                    }
                    "turn/error" => {
                        let error_msg = event
                            .get("params")
                            .and_then(|p| p.get("error"))
                            .and_then(|e| e.as_str())
                            .unwrap_or("Unknown error during commit message generation");
                        return Err(error_msg.to_string());
                    }
                    _ => {}
                }
            }
            Ok(())
        })
        .await;

        {
            let mut callbacks = session.background_thread_callbacks.lock().await;
            callbacks.remove(&thread_id);
        }

        let archive_params = json!({ "threadId": thread_id });
        let _ = session.send_request("thread/archive", archive_params).await;

        match collect_result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err("Timeout waiting for commit message generation".to_string()),
        }

        let trimmed = commit_message.trim().to_string();
        if trimmed.is_empty() {
            return Err("No commit message was generated".to_string());
        }

        Ok(trimmed)
    }

    async fn local_usage_snapshot(
        &self,
        days: Option<u32>,
        workspace_path: Option<String>,
    ) -> Result<LocalUsageSnapshot, String> {
        local_usage_core::local_usage_snapshot_core(days, workspace_path).await
    }
}

async fn git_branch_exists(repo_path: &PathBuf, branch: &str) -> Result<bool, String> {
    let git_bin = resolve_git_binary().map_err(|e| format!("Failed to run git: {e}"))?;
    let status = Command::new(git_bin)
        .args(["show-ref", "--verify", &format!("refs/heads/{branch}")])
        .current_dir(repo_path)
        .env("PATH", git_env_path())
        .status()
        .await
        .map_err(|e| format!("Failed to run git: {e}"))?;
    Ok(status.success())
}

async fn git_remote_exists(repo_path: &PathBuf, remote: &str) -> Result<bool, String> {
    let git_bin = resolve_git_binary().map_err(|e| format!("Failed to run git: {e}"))?;
    let status = Command::new(git_bin)
        .args(["remote", "get-url", remote])
        .current_dir(repo_path)
        .env("PATH", git_env_path())
        .status()
        .await
        .map_err(|e| format!("Failed to run git: {e}"))?;
    Ok(status.success())
}

async fn git_remote_branch_exists_live(
    repo_path: &PathBuf,
    remote: &str,
    branch: &str,
) -> Result<bool, String> {
    let git_bin = resolve_git_binary().map_err(|e| format!("Failed to run git: {e}"))?;
    let output = Command::new(git_bin)
        .args([
            "ls-remote",
            "--heads",
            remote,
            &format!("refs/heads/{branch}"),
        ])
        .current_dir(repo_path)
        .env("PATH", git_env_path())
        .output()
        .await
        .map_err(|e| format!("Failed to run git: {e}"))?;
    if output.status.success() {
        Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        if detail.is_empty() {
            Err("Git command failed.".to_string())
        } else {
            Err(detail.to_string())
        }
    }
}

async fn git_remote_branch_exists(
    repo_path: &PathBuf,
    remote: &str,
    branch: &str,
) -> Result<bool, String> {
    let git_bin = resolve_git_binary().map_err(|e| format!("Failed to run git: {e}"))?;
    let status = Command::new(git_bin)
        .args([
            "show-ref",
            "--verify",
            &format!("refs/remotes/{remote}/{branch}"),
        ])
        .current_dir(repo_path)
        .env("PATH", git_env_path())
        .status()
        .await
        .map_err(|e| format!("Failed to run git: {e}"))?;
    Ok(status.success())
}

async fn unique_branch_name(
    repo_path: &PathBuf,
    desired: &str,
    remote: Option<&str>,
) -> Result<(String, bool), String> {
    let mut candidate = desired.to_string();
    if desired.is_empty() {
        return Ok((candidate, false));
    }
    if !git_branch_exists(repo_path, &candidate).await?
        && match remote {
            Some(remote) => !git_remote_branch_exists_live(repo_path, remote, &candidate).await?,
            None => true,
        }
    {
        return Ok((candidate, false));
    }
    for index in 2..1000 {
        candidate = format!("{desired}-{index}");
        let local_exists = git_branch_exists(repo_path, &candidate).await?;
        let remote_exists = match remote {
            Some(remote) => git_remote_branch_exists_live(repo_path, remote, &candidate).await?,
            None => false,
        };
        if !local_exists && !remote_exists {
            return Ok((candidate, true));
        }
    }
    Err("Unable to find an available branch name.".to_string())
}

async fn git_list_remotes(repo_path: &PathBuf) -> Result<Vec<String>, String> {
    let output = run_git_command(repo_path, &["remote"]).await?;
    Ok(output
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect())
}

async fn git_find_remote_for_branch(
    repo_path: &PathBuf,
    branch: &str,
) -> Result<Option<String>, String> {
    if git_remote_exists(repo_path, "origin").await?
        && git_remote_branch_exists_live(repo_path, "origin", branch).await?
    {
        return Ok(Some("origin".to_string()));
    }

    for remote in git_list_remotes(repo_path).await? {
        if remote == "origin" {
            continue;
        }
        if git_remote_branch_exists_live(repo_path, &remote, branch).await? {
            return Ok(Some(remote));
        }
    }

    Ok(None)
}

async fn git_find_remote_tracking_branch(
    repo_path: &PathBuf,
    branch: &str,
) -> Result<Option<String>, String> {
    if git_remote_branch_exists(repo_path, "origin", branch).await? {
        return Ok(Some(format!("origin/{branch}")));
    }

    for remote in git_list_remotes(repo_path).await? {
        if remote == "origin" {
            continue;
        }
        if git_remote_branch_exists(repo_path, &remote, branch).await? {
            return Ok(Some(format!("{remote}/{branch}")));
        }
    }

    Ok(None)
}

fn sanitize_worktree_name(branch: &str) -> String {
    let mut result = String::new();
    for ch in branch.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            result.push(ch);
        } else {
            result.push('-');
        }
    }
    let trimmed = result.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "worktree".to_string()
    } else {
        trimmed
    }
}

fn unique_worktree_path(base_dir: &PathBuf, name: &str) -> Result<PathBuf, String> {
    let candidate = base_dir.join(name);
    if !candidate.exists() {
        return Ok(candidate);
    }

    for index in 2..1000 {
        let next = base_dir.join(format!("{name}-{index}"));
        if !next.exists() {
            return Ok(next);
        }
    }

    Err(format!(
        "Failed to find an available worktree path under {}.",
        base_dir.display()
    ))
}

fn unique_worktree_path_for_rename(
    base_dir: &PathBuf,
    name: &str,
    current_path: &PathBuf,
) -> Result<PathBuf, String> {
    let candidate = base_dir.join(name);
    if candidate == *current_path {
        return Ok(candidate);
    }
    if !candidate.exists() {
        return Ok(candidate);
    }
    for index in 2..1000 {
        let next = base_dir.join(format!("{name}-{index}"));
        if next == *current_path || !next.exists() {
            return Ok(next);
        }
    }
    Err(format!(
        "Failed to find an available worktree path under {}.",
        base_dir.display()
    ))
}

fn sanitize_clone_dir_name(name: &str) -> String {
    let mut result = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            result.push(ch);
        } else {
            result.push('-');
        }
    }
    let trimmed = result.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "copy".to_string()
    } else {
        trimmed
    }
}

fn build_clone_destination_path(
    copies_folder: &PathBuf,
    copy_name: &str,
) -> Result<PathBuf, String> {
    let safe_name = sanitize_clone_dir_name(copy_name);
    unique_worktree_path(copies_folder, &safe_name)
}

async fn git_get_origin_url(repo_path: &PathBuf) -> Option<String> {
    match run_git_command(repo_path, &["config", "--get", "remote.origin.url"]).await {
        Ok(url) if !url.trim().is_empty() => Some(url),
        _ => None,
    }
}

fn null_device_path() -> &'static str {
    if cfg!(windows) {
        "NUL"
    } else {
        "/dev/null"
    }
}

fn default_data_dir() -> PathBuf {
    if let Ok(xdg) = env::var("XDG_DATA_HOME") {
        let trimmed = xdg.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("codex-monitor-daemon");
        }
    }
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("codex-monitor-daemon")
}

fn usage() -> String {
    format!(
        "\
USAGE:\n  codex-monitor-daemon [--listen <addr>] [--data-dir <path>] [--token <token> | --insecure-no-auth]\n\n\
OPTIONS:\n  --listen <addr>        Bind address (default: {DEFAULT_LISTEN_ADDR})\n  --data-dir <path>      Data dir holding workspaces.json/settings.json\n  --token <token>        Shared token required by clients\n  --insecure-no-auth      Disable auth (dev only)\n  -h, --help             Show this help\n"
    )
}

fn parse_args() -> Result<DaemonConfig, String> {
    let mut listen = DEFAULT_LISTEN_ADDR
        .parse::<SocketAddr>()
        .map_err(|err| err.to_string())?;
    let mut token = env::var("CODEX_MONITOR_DAEMON_TOKEN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let mut insecure_no_auth = false;
    let mut data_dir: Option<PathBuf> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{}", usage());
                std::process::exit(0);
            }
            "--listen" => {
                let value = args.next().ok_or("--listen requires a value")?;
                listen = value.parse::<SocketAddr>().map_err(|err| err.to_string())?;
            }
            "--token" => {
                let value = args.next().ok_or("--token requires a value")?;
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err("--token requires a non-empty value".to_string());
                }
                token = Some(trimmed.to_string());
            }
            "--data-dir" => {
                let value = args.next().ok_or("--data-dir requires a value")?;
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err("--data-dir requires a non-empty value".to_string());
                }
                data_dir = Some(PathBuf::from(trimmed));
            }
            "--insecure-no-auth" => {
                insecure_no_auth = true;
                token = None;
            }
            _ => return Err(format!("Unknown argument: {arg}")),
        }
    }

    if token.is_none() && !insecure_no_auth {
        return Err(
            "Missing --token (or set CODEX_MONITOR_DAEMON_TOKEN). Use --insecure-no-auth for local dev only."
                .to_string(),
        );
    }

    Ok(DaemonConfig {
        listen,
        token,
        data_dir: data_dir.unwrap_or_else(default_data_dir),
    })
}

fn build_error_response(id: Option<u64>, message: &str) -> Option<String> {
    let id = id?;
    Some(
        serde_json::to_string(&json!({
            "id": id,
            "error": { "message": message }
        }))
        .unwrap_or_else(|_| {
            "{\"id\":0,\"error\":{\"message\":\"serialization failed\"}}".to_string()
        }),
    )
}

fn build_result_response(id: Option<u64>, result: Value) -> Option<String> {
    let id = id?;
    Some(
        serde_json::to_string(&json!({ "id": id, "result": result })).unwrap_or_else(|_| {
            "{\"id\":0,\"error\":{\"message\":\"serialization failed\"}}".to_string()
        }),
    )
}

fn build_event_notification(event: DaemonEvent) -> Option<String> {
    let payload = match event {
        DaemonEvent::AppServer(payload) => json!({
            "method": "app-server-event",
            "params": payload,
        }),
        DaemonEvent::TerminalOutput(payload) => json!({
            "method": "terminal-output",
            "params": payload,
        }),
    };
    serde_json::to_string(&payload).ok()
}

fn parse_auth_token(params: &Value) -> Option<String> {
    match params {
        Value::String(value) => Some(value.clone()),
        Value::Object(map) => map
            .get("token")
            .and_then(|value| value.as_str())
            .map(|v| v.to_string()),
        _ => None,
    }
}

fn parse_string(value: &Value, key: &str) -> Result<String, String> {
    match value {
        Value::Object(map) => map
            .get(key)
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
            .ok_or_else(|| format!("missing or invalid `{key}`")),
        _ => Err(format!("missing `{key}`")),
    }
}

fn resolve_api_key(value: &str, env_key: &str) -> Option<String> {
    if !value.trim().is_empty() {
        return Some(value.to_string());
    }
    std::env::var(env_key).ok().filter(|v| !v.trim().is_empty())
}

fn parse_optional_string(value: &Value, key: &str) -> Option<String> {
    match value {
        Value::Object(map) => map
            .get(key)
            .and_then(|value| value.as_str())
            .map(|v| v.to_string()),
        _ => None,
    }
}

fn parse_optional_u32(value: &Value, key: &str) -> Option<u32> {
    match value {
        Value::Object(map) => map.get(key).and_then(|value| value.as_u64()).and_then(|v| {
            if v > u32::MAX as u64 {
                None
            } else {
                Some(v as u32)
            }
        }),
        _ => None,
    }
}

fn parse_optional_usize(value: &Value, key: &str) -> Option<usize> {
    match value {
        Value::Object(map) => map
            .get(key)
            .and_then(|value| value.as_u64())
            .and_then(|v| usize::try_from(v).ok()),
        _ => None,
    }
}

fn read_json_file(path: &Path) -> Result<Value, String> {
    let mut file = File::open(path).map_err(|err| err.to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|err| err.to_string())?;
    serde_json::from_str(&contents).map_err(|err| err.to_string())
}

fn write_json_file(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let contents = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    let mut file = File::create(path).map_err(|err| err.to_string())?;
    file.write_all(contents.as_bytes())
        .map_err(|err| err.to_string())
}

#[cfg(test)]
mod daemon_tests {
    use super::{read_json_file, write_json_file};
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn json_file_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("config.json");
        let value = json!({ "enabled": [{ "name": "a", "path": "/a" }], "disabled": [] });
        write_json_file(&path, &value).expect("write");
        let loaded = read_json_file(&path).expect("read");
        assert_eq!(loaded, value);
    }
}

fn parse_optional_string_array(value: &Value, key: &str) -> Option<Vec<String>> {
    match value {
        Value::Object(map) => map
            .get(key)
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(|value| value.to_string()))
                    .collect::<Vec<_>>()
            }),
        _ => None,
    }
}

fn parse_string_array(value: &Value, key: &str) -> Result<Vec<String>, String> {
    parse_optional_string_array(value, key).ok_or_else(|| format!("missing `{key}`"))
}

fn parse_optional_value(value: &Value, key: &str) -> Option<Value> {
    match value {
        Value::Object(map) => map.get(key).cloned(),
        _ => None,
    }
}

async fn handle_rpc_request(
    state: &DaemonState,
    method: &str,
    params: Value,
    client_version: String,
) -> Result<Value, String> {
    match method {
        "ping" => Ok(json!({ "ok": true })),
        "list_workspaces" => {
            let workspaces = state.list_workspaces().await;
            serde_json::to_value(workspaces).map_err(|err| err.to_string())
        }
        "is_workspace_path_dir" => {
            let path = parse_string(&params, "path")?;
            let is_dir = state.is_workspace_path_dir(path).await;
            serde_json::to_value(is_dir).map_err(|err| err.to_string())
        }
        "add_workspace" => {
            let path = parse_string(&params, "path")?;
            let codex_bin = parse_optional_string(&params, "codex_bin");
            let workspace = state.add_workspace(path, codex_bin, client_version).await?;
            serde_json::to_value(workspace).map_err(|err| err.to_string())
        }
        "add_clone" => {
            let source_workspace_id = parse_string(&params, "sourceWorkspaceId")?;
            let copies_folder = parse_string(&params, "copiesFolder")?;
            let copy_name = parse_string(&params, "copyName")?;
            let workspace = state
                .add_clone(
                    source_workspace_id,
                    copy_name,
                    copies_folder,
                    client_version,
                )
                .await?;
            serde_json::to_value(workspace).map_err(|err| err.to_string())
        }
        "add_worktree" => {
            let parent_id = parse_string(&params, "parentId")?;
            let branch = parse_string(&params, "branch")?;
            let workspace = state
                .add_worktree(parent_id, branch, client_version)
                .await?;
            serde_json::to_value(workspace).map_err(|err| err.to_string())
        }
        "connect_workspace" => {
            let id = parse_string(&params, "id")?;
            state.connect_workspace(id, client_version).await?;
            Ok(json!({ "ok": true }))
        }
        "remove_workspace" => {
            let id = parse_string(&params, "id")?;
            state.remove_workspace(id).await?;
            Ok(json!({ "ok": true }))
        }
        "remove_worktree" => {
            let id = parse_string(&params, "id")?;
            state.remove_worktree(id).await?;
            Ok(json!({ "ok": true }))
        }
        "rename_worktree" => {
            let id = parse_string(&params, "id")?;
            let branch = parse_string(&params, "branch")?;
            let workspace = state.rename_worktree(id, branch, client_version).await?;
            serde_json::to_value(workspace).map_err(|err| err.to_string())
        }
        "rename_worktree_upstream" => {
            let id = parse_string(&params, "id")?;
            let old_branch = parse_string(&params, "oldBranch")?;
            let new_branch = parse_string(&params, "newBranch")?;
            state
                .rename_worktree_upstream(id, old_branch, new_branch)
                .await?;
            Ok(json!({ "ok": true }))
        }
        "apply_worktree_changes" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.apply_worktree_changes(workspace_id).await?;
            Ok(json!({ "ok": true }))
        }
        "open_workspace_in" => {
            Err("open_workspace_in is not supported in daemon mode.".to_string())
        }
        "update_workspace_settings" => {
            let id = parse_string(&params, "id")?;
            let settings_value = match params {
                Value::Object(map) => map.get("settings").cloned().unwrap_or(Value::Null),
                _ => Value::Null,
            };
            let settings: WorkspaceSettings =
                serde_json::from_value(settings_value).map_err(|err| err.to_string())?;
            let workspace = state.update_workspace_settings(id, settings).await?;
            serde_json::to_value(workspace).map_err(|err| err.to_string())
        }
        "update_workspace_codex_bin" => {
            let id = parse_string(&params, "id")?;
            let codex_bin = parse_optional_string(&params, "codex_bin");
            let workspace = state.update_workspace_codex_bin(id, codex_bin).await?;
            serde_json::to_value(workspace).map_err(|err| err.to_string())
        }
        "list_workspace_files" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let files = state.list_workspace_files(workspace_id).await?;
            serde_json::to_value(files).map_err(|err| err.to_string())
        }
        "read_workspace_file" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let path = parse_string(&params, "path")?;
            let response = state.read_workspace_file(workspace_id, path).await?;
            serde_json::to_value(response).map_err(|err| err.to_string())
        }
        "read_global_agents_md" => {
            let response = state.read_global_agents_md().await?;
            serde_json::to_value(response).map_err(|err| err.to_string())
        }
        "write_global_agents_md" => {
            let content = parse_string(&params, "content")?;
            state.write_global_agents_md(content).await?;
            Ok(json!({ "ok": true }))
        }
        "read_global_config_toml" => {
            let response = state.read_global_config_toml().await?;
            serde_json::to_value(response).map_err(|err| err.to_string())
        }
        "write_global_config_toml" => {
            let content = parse_string(&params, "content")?;
            state.write_global_config_toml(content).await?;
            Ok(json!({ "ok": true }))
        }
        "get_app_settings" => {
            let mut settings = state.app_settings.lock().await.clone();
            if let Ok(Some(collab_enabled)) = codex_config::read_collab_enabled() {
                settings.experimental_collab_enabled = collab_enabled;
            }
            if let Ok(Some(steer_enabled)) = codex_config::read_steer_enabled() {
                settings.experimental_steer_enabled = steer_enabled;
            }
            if let Ok(Some(unified_exec_enabled)) = codex_config::read_unified_exec_enabled() {
                settings.experimental_unified_exec_enabled = unified_exec_enabled;
            }
            serde_json::to_value(settings).map_err(|err| err.to_string())
        }
        "update_app_settings" => {
            let settings_value = match params {
                Value::Object(map) => map.get("settings").cloned().unwrap_or(Value::Null),
                _ => Value::Null,
            };
            let settings: AppSettings =
                serde_json::from_value(settings_value).map_err(|err| err.to_string())?;
            let updated = state.update_app_settings(settings).await?;
            serde_json::to_value(updated).map_err(|err| err.to_string())
        }
        "domains_list" => {
            let domains = state.domains_list().await?;
            serde_json::to_value(domains).map_err(|err| err.to_string())
        }
        "domains_create" => {
            let domain: Domain =
                serde_json::from_value(params).map_err(|err| format!("Invalid domain: {err}"))?;
            let created = state.domains_create(domain).await?;
            serde_json::to_value(created).map_err(|err| err.to_string())
        }
        "domains_update" => {
            let domain: Domain =
                serde_json::from_value(params).map_err(|err| format!("Invalid domain: {err}"))?;
            let updated = state.domains_update(domain).await?;
            serde_json::to_value(updated).map_err(|err| err.to_string())
        }
        "domains_delete" => {
            let domain_id = parse_string(&params, "domainId")?;
            state.domains_delete(domain_id).await?;
            Ok(json!({ "ok": true }))
        }
        "memory_status" => {
            let memory = state.memory.read().await;
            match memory.as_ref() {
                Some(mem) => mem.status().await.map(|s| serde_json::to_value(s).unwrap()),
                None => Ok(json!({
                    "enabled": false,
                    "embeddings_enabled": false,
                    "total": 0,
                    "pending": 0,
                    "ready": 0,
                    "error": 0
                })),
            }
        }
        "memory_search" => {
            let query = params
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or("Missing query")?;
            let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

            let memory = state.memory.read().await;
            match memory.as_ref() {
                Some(mem) => mem
                    .search(query, limit)
                    .await
                    .map(|r| serde_json::to_value(r).unwrap()),
                None => Ok(json!([])),
            }
        }
        "memory_append" => {
            let memory_type = params
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("daily");
            let content = params
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing content")?;
            let tags: Vec<String> = params
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let workspace_id = params
                .get("workspace_id")
                .and_then(|v| v.as_str())
                .map(String::from);

            let memory = state.memory.read().await;
            match memory.as_ref() {
                Some(mem) => mem
                    .append(memory_type, content, tags, workspace_id)
                    .await
                    .map(|e| serde_json::to_value(e).unwrap()),
                None => Err("Memory not enabled".to_string()),
            }
        }
        "memory_bootstrap" => {
            let memory = state.memory.read().await;
            match memory.as_ref() {
                Some(mem) => mem
                    .bootstrap()
                    .await
                    .map(|r| serde_json::to_value(r).unwrap()),
                None => Ok(json!([])),
            }
        }
        "memory_flush_now" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let thread_id = parse_string(&params, "threadId")?;
            let force = params
                .get("force")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            state.memory_flush_now(workspace_id, thread_id, force).await
        }
        "browser_create_session" => {
            let params = if params.is_object() {
                params
            } else {
                json!({})
            };
            state.browser.request("browser.create", params).await
        }
        "browser_list_sessions" => {
            let params = if params.is_object() {
                params
            } else {
                json!({})
            };
            state.browser.request("browser.list", params).await
        }
        "browser_close_session" => {
            let params = if params.is_object() {
                params
            } else {
                json!({})
            };
            state.browser.request("browser.close", params).await
        }
        "browser_navigate" => {
            let params = if params.is_object() {
                params
            } else {
                json!({})
            };
            state.browser.request("browser.navigate", params).await
        }
        "browser_screenshot" => {
            let params = if params.is_object() {
                params
            } else {
                json!({})
            };
            state.browser.request("browser.screenshot", params).await
        }
        "browser_click" => {
            let params = if params.is_object() {
                params
            } else {
                json!({})
            };
            state.browser.request("browser.click", params).await
        }
        "browser_type" => {
            let params = if params.is_object() {
                params
            } else {
                json!({})
            };
            state.browser.request("browser.type", params).await
        }
        "browser_press" => {
            let params = if params.is_object() {
                params
            } else {
                json!({})
            };
            state.browser.request("browser.press", params).await
        }
        "browser_snapshot" => {
            let params = if params.is_object() {
                params
            } else {
                json!({})
            };
            state.browser.request("browser.snapshot", params).await
        }
        "browser_evaluate" => {
            let params = if params.is_object() {
                params
            } else {
                json!({})
            };
            state.browser.request("browser.evaluate", params).await
        }
        "codex_doctor" => {
            let codex_bin = parse_optional_string(&params, "codexBin");
            let result = state.codex_doctor(codex_bin).await?;
            Ok(result)
        }
        "get_life_workspace_prompt" => {
            let prompt = state.get_life_workspace_prompt().await?;
            serde_json::to_value(prompt).map_err(|err| err.to_string())
        }
        "get_delivery_dashboard" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let range = parse_string(&params, "range")?;
            state.get_delivery_dashboard(workspace_id, range).await
        }
        "get_nutrition_dashboard" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let range = parse_string(&params, "range")?;
            state.get_nutrition_dashboard(workspace_id, range).await
        }
        "get_exercise_dashboard" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let range = parse_string(&params, "range")?;
            state.get_exercise_dashboard(workspace_id, range).await
        }
        "get_media_dashboard" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.get_media_dashboard(workspace_id).await
        }
        "get_youtube_dashboard" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.get_youtube_dashboard(workspace_id).await
        }
        "enrich_media_covers" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let force = params
                .get("force")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            state.enrich_media_covers(workspace_id, force).await
        }
        "fix_broken_covers" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.fix_broken_covers(workspace_id).await
        }
        "rebuild_media_covers" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.rebuild_media_covers(workspace_id).await
        }
        "get_finance_dashboard" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let range = parse_string(&params, "range")?;
            state.get_finance_dashboard(workspace_id, range).await
        }
        "get_commit_message_prompt" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let prompt = state.get_commit_message_prompt(workspace_id).await?;
            serde_json::to_value(prompt).map_err(|err| err.to_string())
        }
        "generate_commit_message" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let message = state.generate_commit_message(workspace_id).await?;
            serde_json::to_value(message).map_err(|err| err.to_string())
        }
        "start_thread" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.start_thread(workspace_id).await
        }
        "resume_thread" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let thread_id = parse_string(&params, "threadId")?;
            state.resume_thread(workspace_id, thread_id).await
        }
        "list_threads" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let cursor = parse_optional_string(&params, "cursor");
            let limit = parse_optional_u32(&params, "limit");
            state.list_threads(workspace_id, cursor, limit).await
        }
        "archive_thread" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let thread_id = parse_string(&params, "threadId")?;
            state.archive_thread(workspace_id, thread_id).await
        }
        "send_user_message" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let thread_id = parse_string(&params, "threadId")?;
            let text = parse_string(&params, "text")?;
            let model = parse_optional_string(&params, "model");
            let effort = parse_optional_string(&params, "effort");
            let access_mode = parse_optional_string(&params, "accessMode");
            let images = parse_optional_string_array(&params, "images");
            let collaboration_mode = parse_optional_value(&params, "collaborationMode");
            state
                .send_user_message(
                    workspace_id,
                    thread_id,
                    text,
                    model,
                    effort,
                    access_mode,
                    images,
                    collaboration_mode,
                )
                .await
        }
        "turn_interrupt" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let thread_id = parse_string(&params, "threadId")?;
            let turn_id = parse_string(&params, "turnId")?;
            state.turn_interrupt(workspace_id, thread_id, turn_id).await
        }
        "start_review" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let thread_id = parse_string(&params, "threadId")?;
            let target = params
                .as_object()
                .and_then(|map| map.get("target"))
                .cloned()
                .ok_or("missing `target`")?;
            let delivery = parse_optional_string(&params, "delivery");
            state
                .start_review(workspace_id, thread_id, target, delivery)
                .await
        }
        "model_list" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.model_list(workspace_id).await
        }
        "collaboration_mode_list" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.collaboration_mode_list(workspace_id).await
        }
        "account_rate_limits" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.account_rate_limits(workspace_id).await
        }
        "skills_list" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.skills_list(workspace_id).await
        }
        "skills_config_read" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.skills_config_read(workspace_id).await
        }
        "skills_config_write" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let config = match params {
                Value::Object(map) => map.get("config").cloned().unwrap_or_else(|| json!({})),
                _ => json!({}),
            };
            state.skills_config_write(workspace_id, config).await
        }
        "skills_validate" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.skills_validate(workspace_id).await
        }
        "skills_install_from_git" => {
            let source_url = parse_string(&params, "sourceUrl")?;
            let target = parse_string(&params, "target")?;
            let workspace_id = parse_optional_string(&params, "workspaceId");
            state
                .skills_install_from_git(source_url, target, workspace_id)
                .await
        }
        "skills_uninstall" => {
            let name = parse_string(&params, "name")?;
            let target = parse_string(&params, "target")?;
            let workspace_id = parse_optional_string(&params, "workspaceId");
            state.skills_uninstall(name, target, workspace_id).await
        }
        "domain_trends" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let domain_id = parse_string(&params, "domainId")?;
            let range = parse_string(&params, "range")?;
            let snapshot = state.domain_trends(workspace_id, domain_id, range).await?;
            serde_json::to_value(snapshot).map_err(|e| e.to_string())
        }
        "list_git_roots" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let depth = parse_optional_usize(&params, "depth");
            let roots = state.list_git_roots(workspace_id, depth).await?;
            serde_json::to_value(roots).map_err(|err| err.to_string())
        }
        "get_git_status" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.get_git_status(workspace_id).await
        }
        "get_git_diffs" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let diffs = state.get_git_diffs(workspace_id).await?;
            serde_json::to_value(diffs).map_err(|err| err.to_string())
        }
        "get_git_log" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let limit = parse_optional_usize(&params, "limit");
            let log = state.get_git_log(workspace_id, limit).await?;
            serde_json::to_value(log).map_err(|err| err.to_string())
        }
        "get_git_commit_diff" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let sha = parse_string(&params, "sha")?;
            let diffs = state.get_git_commit_diff(workspace_id, sha).await?;
            serde_json::to_value(diffs).map_err(|err| err.to_string())
        }
        "get_git_remote" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let remote = state.get_git_remote(workspace_id).await?;
            serde_json::to_value(remote).map_err(|err| err.to_string())
        }
        "stage_git_file" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let path = parse_string(&params, "path")?;
            state.stage_git_file(workspace_id, path).await?;
            Ok(json!({ "ok": true }))
        }
        "stage_git_all" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.stage_git_all(workspace_id).await?;
            Ok(json!({ "ok": true }))
        }
        "unstage_git_file" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let path = parse_string(&params, "path")?;
            state.unstage_git_file(workspace_id, path).await?;
            Ok(json!({ "ok": true }))
        }
        "revert_git_file" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let path = parse_string(&params, "path")?;
            state.revert_git_file(workspace_id, path).await?;
            Ok(json!({ "ok": true }))
        }
        "revert_git_all" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.revert_git_all(workspace_id).await?;
            Ok(json!({ "ok": true }))
        }
        "commit_git" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let message = parse_string(&params, "message")?;
            state.commit_git(workspace_id, message).await?;
            Ok(json!({ "ok": true }))
        }
        "pull_git" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.pull_git(workspace_id).await?;
            Ok(json!({ "ok": true }))
        }
        "push_git" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.push_git(workspace_id).await?;
            Ok(json!({ "ok": true }))
        }
        "sync_git" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.sync_git(workspace_id).await?;
            Ok(json!({ "ok": true }))
        }
        "list_git_branches" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            state.list_git_branches(workspace_id).await
        }
        "checkout_git_branch" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let name = parse_string(&params, "name")?;
            state.checkout_git_branch(workspace_id, name).await?;
            Ok(json!({ "ok": true }))
        }
        "create_git_branch" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let name = parse_string(&params, "name")?;
            state.create_git_branch(workspace_id, name).await?;
            Ok(json!({ "ok": true }))
        }
        "get_github_issues" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let issues = state.get_github_issues(workspace_id).await?;
            serde_json::to_value(issues).map_err(|err| err.to_string())
        }
        "get_github_pull_requests" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let prs = state.get_github_pull_requests(workspace_id).await?;
            serde_json::to_value(prs).map_err(|err| err.to_string())
        }
        "get_github_pull_request_diff" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let pr_number = params
                .as_object()
                .and_then(|map| map.get("prNumber"))
                .and_then(|value| value.as_u64())
                .ok_or("missing `prNumber`")?;
            let diffs = state
                .get_github_pull_request_diff(workspace_id, pr_number)
                .await?;
            serde_json::to_value(diffs).map_err(|err| err.to_string())
        }
        "get_github_pull_request_comments" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let pr_number = params
                .as_object()
                .and_then(|map| map.get("prNumber"))
                .and_then(|value| value.as_u64())
                .ok_or("missing `prNumber`")?;
            let comments = state
                .get_github_pull_request_comments(workspace_id, pr_number)
                .await?;
            serde_json::to_value(comments).map_err(|err| err.to_string())
        }
        "prompts_list" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let prompts = state.prompts_list(workspace_id).await?;
            serde_json::to_value(prompts).map_err(|err| err.to_string())
        }
        "prompts_create" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let scope = parse_string(&params, "scope")?;
            let name = parse_string(&params, "name")?;
            let description = parse_optional_string(&params, "description");
            let argument_hint = parse_optional_string(&params, "argumentHint");
            let content = parse_string(&params, "content")?;
            let prompt = state
                .prompts_create(
                    workspace_id,
                    scope,
                    name,
                    description,
                    argument_hint,
                    content,
                )
                .await?;
            serde_json::to_value(prompt).map_err(|err| err.to_string())
        }
        "prompts_update" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let path = parse_string(&params, "path")?;
            let name = parse_string(&params, "name")?;
            let description = parse_optional_string(&params, "description");
            let argument_hint = parse_optional_string(&params, "argumentHint");
            let content = parse_string(&params, "content")?;
            let prompt = state
                .prompts_update(
                    workspace_id,
                    path,
                    name,
                    description,
                    argument_hint,
                    content,
                )
                .await?;
            serde_json::to_value(prompt).map_err(|err| err.to_string())
        }
        "prompts_delete" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let path = parse_string(&params, "path")?;
            state.prompts_delete(workspace_id, path).await?;
            Ok(json!({ "ok": true }))
        }
        "prompts_move" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let path = parse_string(&params, "path")?;
            let scope = parse_string(&params, "scope")?;
            let prompt = state.prompts_move(workspace_id, path, scope).await?;
            serde_json::to_value(prompt).map_err(|err| err.to_string())
        }
        "prompts_workspace_dir" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let dir = state.prompts_workspace_dir(workspace_id).await?;
            serde_json::to_value(dir).map_err(|err| err.to_string())
        }
        "prompts_global_dir" => {
            let dir = state.prompts_global_dir().await?;
            serde_json::to_value(dir).map_err(|err| err.to_string())
        }
        "terminal_open" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let terminal_id = parse_string(&params, "terminalId")?;
            let cols = parse_optional_u32(&params, "cols").ok_or("missing `cols`")?;
            let rows = parse_optional_u32(&params, "rows").ok_or("missing `rows`")?;
            let info = state
                .terminal_open(
                    workspace_id,
                    terminal_id,
                    cols.min(u16::MAX as u32) as u16,
                    rows.min(u16::MAX as u32) as u16,
                )
                .await?;
            serde_json::to_value(info).map_err(|err| err.to_string())
        }
        "terminal_write" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let terminal_id = parse_string(&params, "terminalId")?;
            let data = parse_string(&params, "data")?;
            state
                .terminal_write(workspace_id, terminal_id, data)
                .await?;
            Ok(json!({ "ok": true }))
        }
        "terminal_resize" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let terminal_id = parse_string(&params, "terminalId")?;
            let cols = parse_optional_u32(&params, "cols").ok_or("missing `cols`")?;
            let rows = parse_optional_u32(&params, "rows").ok_or("missing `rows`")?;
            state
                .terminal_resize(
                    workspace_id,
                    terminal_id,
                    cols.min(u16::MAX as u32) as u16,
                    rows.min(u16::MAX as u32) as u16,
                )
                .await?;
            Ok(json!({ "ok": true }))
        }
        "terminal_close" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let terminal_id = parse_string(&params, "terminalId")?;
            state.terminal_close(workspace_id, terminal_id).await?;
            Ok(json!({ "ok": true }))
        }
        "local_usage_snapshot" => {
            let days = parse_optional_u32(&params, "days");
            let workspace_path = parse_optional_string(&params, "workspacePath");
            let snapshot = state.local_usage_snapshot(days, workspace_path).await?;
            serde_json::to_value(snapshot).map_err(|err| err.to_string())
        }
        "respond_to_server_request" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let map = params.as_object().ok_or("missing requestId")?;
            let request_id = map
                .get("requestId")
                .cloned()
                .filter(|value| value.is_number() || value.is_string())
                .ok_or("missing requestId")?;
            let result = map.get("result").cloned().ok_or("missing `result`")?;
            state
                .respond_to_server_request(workspace_id, request_id, result)
                .await
        }
        "remember_approval_rule" => {
            let workspace_id = parse_string(&params, "workspaceId")?;
            let command = parse_string_array(&params, "command")?;
            state.remember_approval_rule(workspace_id, command).await
        }
        _ => Err(format!("unknown method: {method}")),
    }
}

async fn forward_events(
    mut rx: broadcast::Receiver<DaemonEvent>,
    out_tx_events: mpsc::UnboundedSender<String>,
) {
    loop {
        let event = match rx.recv().await {
            Ok(event) => event,
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => break,
        };

        let Some(payload) = build_event_notification(event) else {
            continue;
        };

        if out_tx_events.send(payload).is_err() {
            break;
        }
    }
}

async fn maybe_trigger_auto_memory(
    state: Arc<DaemonState>,
    workspace_id: String,
    thread_id: String,
    context_tokens: u32,
    model_context_window: u32,
) {
    let settings = state.app_settings.lock().await.clone();
    if !settings.auto_memory.enabled {
        return;
    }

    let key = format!("{workspace_id}:{thread_id}");
    let should_flush = {
        let mut runtime = state.auto_memory_runtime.lock().await;
        runtime.update_and_check(
            &key,
            context_tokens,
            model_context_window,
            &settings.auto_memory,
        )
    };

    if !should_flush {
        return;
    }

    let memory = match state.memory.read().await.clone() {
        Some(mem) => mem,
        None => return,
    };

    let session = match state.get_session(&workspace_id).await {
        Ok(session) => session,
        Err(_) => return,
    };

    let auto_settings = settings.auto_memory.clone();
    tokio::spawn(async move {
        let result = perform_memory_flush(
            session,
            memory,
            auto_settings,
            workspace_id,
            thread_id,
            context_tokens,
            model_context_window,
        )
        .await;
        if let Err(err) = result {
            eprintln!("Auto memory flush failed: {err}");
        }
    });
}

async fn handle_client(
    socket: TcpStream,
    config: Arc<DaemonConfig>,
    state: Arc<DaemonState>,
    events: broadcast::Sender<DaemonEvent>,
) {
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();

    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();
    let write_task = tokio::spawn(async move {
        while let Some(message) = out_rx.recv().await {
            if writer.write_all(message.as_bytes()).await.is_err() {
                break;
            }
            if writer.write_all(b"\n").await.is_err() {
                break;
            }
        }
    });

    let mut authenticated = config.token.is_none();
    let mut events_task: Option<tokio::task::JoinHandle<()>> = None;

    if authenticated {
        let rx = events.subscribe();
        let out_tx_events = out_tx.clone();
        events_task = Some(tokio::spawn(forward_events(rx, out_tx_events)));
    }

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let message: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };

        let id = message.get("id").and_then(|value| value.as_u64());
        let method = message
            .get("method")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        let params = message.get("params").cloned().unwrap_or(Value::Null);

        if !authenticated {
            if method != "auth" {
                if let Some(response) = build_error_response(id, "unauthorized") {
                    let _ = out_tx.send(response);
                }
                continue;
            }

            let expected = config.token.clone().unwrap_or_default();
            let provided = parse_auth_token(&params).unwrap_or_default();
            if expected != provided {
                if let Some(response) = build_error_response(id, "invalid token") {
                    let _ = out_tx.send(response);
                }
                continue;
            }

            authenticated = true;
            if let Some(response) = build_result_response(id, json!({ "ok": true })) {
                let _ = out_tx.send(response);
            }

            let rx = events.subscribe();
            let out_tx_events = out_tx.clone();
            events_task = Some(tokio::spawn(forward_events(rx, out_tx_events)));

            continue;
        }

        let client_version = format!("daemon-{}", env!("CARGO_PKG_VERSION"));
        let result = handle_rpc_request(&state, &method, params, client_version).await;
        let response = match result {
            Ok(result) => build_result_response(id, result),
            Err(message) => build_error_response(id, &message),
        };
        if let Some(response) = response {
            let _ = out_tx.send(response);
        }
    }

    drop(out_tx);
    if let Some(task) = events_task {
        task.abort();
    }
    write_task.abort();
}

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{err}\n\n{}", usage());
            std::process::exit(2);
        }
    };

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    runtime.block_on(async move {
        let (events_tx, _events_rx) = broadcast::channel::<DaemonEvent>(2048);
        let event_sink = DaemonEventSink {
            tx: events_tx.clone(),
        };
        let state = Arc::new(DaemonState::load(&config, event_sink));
        let config = Arc::new(config);

        {
            let state = Arc::clone(&state);
            let mut rx = events_tx.subscribe();
            tokio::spawn(async move {
                loop {
                    let event = match rx.recv().await {
                        Ok(event) => event,
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => break,
                    };
                    let DaemonEvent::AppServer(app_event) = event else {
                        continue;
                    };
                    let method = app_event
                        .message
                        .get("method")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if method != "thread/tokenUsage/updated" {
                        continue;
                    }
                    let params = app_event
                        .message
                        .get("params")
                        .and_then(|v| v.as_object())
                        .cloned()
                        .unwrap_or_default();
                    let thread_id = params
                        .get("threadId")
                        .or_else(|| params.get("thread_id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if thread_id.is_empty() {
                        continue;
                    }
                    let token_usage = params
                        .get("tokenUsage")
                        .or_else(|| params.get("token_usage"))
                        .cloned()
                        .unwrap_or(Value::Null);
                    let total_tokens = token_usage
                        .pointer("/total/totalTokens")
                        .or_else(|| token_usage.pointer("/total/total_tokens"))
                        .or_else(|| token_usage.get("totalTokens"))
                        .or_else(|| token_usage.get("total_tokens"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32;
                    let model_context_window = token_usage
                        .get("modelContextWindow")
                        .or_else(|| token_usage.get("model_context_window"))
                        .or_else(|| params.get("modelContextWindow"))
                        .or_else(|| params.get("model_context_window"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32;
                    if total_tokens == 0 || model_context_window == 0 {
                        continue;
                    }
                    maybe_trigger_auto_memory(
                        Arc::clone(&state),
                        app_event.workspace_id.clone(),
                        thread_id,
                        total_tokens,
                        model_context_window,
                    )
                    .await;
                }
            });
        }

        let listener = TcpListener::bind(config.listen)
            .await
            .unwrap_or_else(|err| panic!("failed to bind {}: {err}", config.listen));
        eprintln!(
            "codex-monitor-daemon listening on {} (data dir: {})",
            config.listen,
            state
                .storage_path
                .parent()
                .unwrap_or(&state.storage_path)
                .display()
        );

        loop {
            match listener.accept().await {
                Ok((socket, _addr)) => {
                    let config = Arc::clone(&config);
                    let state = Arc::clone(&state);
                    let events = events_tx.clone();
                    tokio::spawn(async move {
                        handle_client(socket, config, state, events).await;
                    });
                }
                Err(_) => continue,
            }
        }
    });
}
