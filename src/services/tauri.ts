import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  AppSettings,
  CodexDoctorResult,
  DictationModelStatus,
  DictationSessionState,
  LocalUsageSnapshot,
  MemoryEntry,
  MemorySearchResult,
  MemoryStatus,
  SessionThreadInfo,
  WorkspaceInfo,
  WorkspaceSettings,
  Domain,
  DomainTrendSnapshot,
  DeliveryDashboard,
  ExerciseDashboard,
  FinanceDashboard,
  MediaLibrary,
  MediaCoverSummary,
  FixCoversResult,
  CoverRebuildResult,
  LifeTimeRange,
  NutritionDashboard,
  YouTubeLibrary,
} from "../types";
import type {
  GitFileDiff,
  GitFileStatus,
  GitCommitDiff,
  GitHubIssuesResponse,
  GitHubPullRequestComment,
  GitHubPullRequestDiff,
  GitHubPullRequestsResponse,
  GitLogResponse,
  ReviewTarget,
} from "../types";

function isMissingTauriInvokeError(error: unknown) {
  return (
    error instanceof TypeError &&
    (error.message.includes("reading 'invoke'") ||
      error.message.includes("reading \"invoke\""))
  );
}

export type TextFileResponse = {
  exists: boolean;
  content: string;
  truncated: boolean;
};

export async function pickWorkspacePath(): Promise<string | null> {
  const selection = await open({ directory: true, multiple: false });
  if (!selection || Array.isArray(selection)) {
    return null;
  }
  return selection;
}

export async function pickImageFiles(): Promise<string[]> {
  const selection = await open({
    multiple: true,
    filters: [
      {
        name: "Images",
        extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp", "tiff", "tif"],
      },
    ],
  });
  if (!selection) {
    return [];
  }
  return Array.isArray(selection) ? selection : [selection];
}

export async function listWorkspaces(): Promise<WorkspaceInfo[]> {
  try {
    return await invoke<WorkspaceInfo[]>("list_workspaces");
  } catch (error) {
    if (isMissingTauriInvokeError(error)) {
      console.warn("Tauri invoke bridge unavailable; returning empty workspaces list.");
      return [];
    }
    throw error;
  }
}

export async function listDomains(): Promise<Domain[]> {
  return invoke<Domain[]>("domains_list");
}

export async function createDomain(domain: Domain): Promise<Domain> {
  return invoke<Domain>("domains_create", domain);
}

export async function updateDomain(domain: Domain): Promise<Domain> {
  return invoke<Domain>("domains_update", domain);
}

export async function deleteDomain(domainId: string): Promise<void> {
  return invoke("domains_delete", { domainId });
}

export async function getDomainTrends(
  workspaceId: string,
  domainId: string,
  range: "7d" | "30d" | "lifetime",
): Promise<DomainTrendSnapshot> {
  return invoke<DomainTrendSnapshot>("domain_trends", { workspaceId, domainId, range });
}

export async function getDeliveryDashboard(
  workspaceId: string,
  range: LifeTimeRange,
): Promise<DeliveryDashboard> {
  return invoke<DeliveryDashboard>("get_delivery_dashboard", { workspaceId, range });
}

export async function getNutritionDashboard(
  workspaceId: string,
  range: LifeTimeRange,
): Promise<NutritionDashboard> {
  return invoke<NutritionDashboard>("get_nutrition_dashboard", { workspaceId, range });
}

export async function getExerciseDashboard(
  workspaceId: string,
  range: LifeTimeRange,
): Promise<ExerciseDashboard> {
  return invoke<ExerciseDashboard>("get_exercise_dashboard", { workspaceId, range });
}

export async function getMediaDashboard(
  workspaceId: string,
): Promise<MediaLibrary> {
  return invoke<MediaLibrary>("get_media_dashboard", { workspaceId });
}

export async function getYouTubeDashboard(
  workspaceId: string,
): Promise<YouTubeLibrary> {
  return invoke<YouTubeLibrary>("get_youtube_dashboard", { workspaceId });
}

export async function getFinanceDashboard(
  workspaceId: string,
  range: LifeTimeRange,
): Promise<FinanceDashboard> {
  return invoke<FinanceDashboard>("get_finance_dashboard", { workspaceId, range });
}

export async function enrichMediaCovers(
  workspaceId: string,
  force = false,
): Promise<MediaCoverSummary> {
  return invoke<MediaCoverSummary>("enrich_media_covers", { workspaceId, force });
}

export async function fixBrokenCovers(
  workspaceId: string,
): Promise<FixCoversResult> {
  return invoke<FixCoversResult>("fix_broken_covers", { workspaceId });
}

export async function rebuildMediaCovers(
  workspaceId: string,
): Promise<CoverRebuildResult> {
  return invoke<CoverRebuildResult>("rebuild_media_covers", { workspaceId });
}

export async function refetchMediaCover(
  workspaceId: string,
  mediaId: string,
): Promise<string | null> {
  return invoke<string | null>("refetch_media_cover", { workspaceId, mediaId });
}

export async function addWorkspace(
  path: string,
  codex_bin: string | null,
): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("add_workspace", { path, codex_bin });
}

export async function isWorkspacePathDir(path: string): Promise<boolean> {
  return invoke<boolean>("is_workspace_path_dir", { path });
}

export async function addClone(
  sourceWorkspaceId: string,
  copiesFolder: string,
  copyName: string,
): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("add_clone", {
    sourceWorkspaceId,
    copiesFolder,
    copyName,
  });
}

export async function addWorktree(
  parentId: string,
  branch: string,
): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("add_worktree", { parentId, branch });
}

export async function updateWorkspaceSettings(
  id: string,
  settings: WorkspaceSettings,
): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("update_workspace_settings", { id, settings });
}

export async function updateWorkspaceCodexBin(
  id: string,
  codex_bin: string | null,
): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("update_workspace_codex_bin", { id, codex_bin });
}

export async function removeWorkspace(id: string): Promise<void> {
  return invoke("remove_workspace", { id });
}

export async function removeWorktree(id: string): Promise<void> {
  return invoke("remove_worktree", { id });
}

export async function renameWorktree(
  id: string,
  branch: string,
): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("rename_worktree", { id, branch });
}

export async function renameWorktreeUpstream(
  id: string,
  oldBranch: string,
  newBranch: string,
): Promise<void> {
  return invoke("rename_worktree_upstream", { id, oldBranch, newBranch });
}

export async function applyWorktreeChanges(workspaceId: string): Promise<void> {
  return invoke("apply_worktree_changes", { workspaceId });
}

export async function openWorkspaceIn(path: string, app: string): Promise<void> {
  return invoke("open_workspace_in", { path, app });
}

export async function connectWorkspace(id: string): Promise<void> {
  return invoke("connect_workspace", { id });
}

export async function startThread(workspaceId: string) {
  return invoke<any>("start_thread", { workspaceId });
}

export async function sendUserMessage(
  workspaceId: string,
  threadId: string,
  text: string,
  options?: {
    model?: string | null;
    effort?: string | null;
    accessMode?: "read-only" | "current" | "full-access";
    images?: string[];
    collaborationMode?: Record<string, unknown> | null;
  },
) {
  return invoke("send_user_message", {
    workspaceId,
    threadId,
    text,
    model: options?.model ?? null,
    effort: options?.effort ?? null,
    accessMode: options?.accessMode ?? null,
    images: options?.images ?? null,
    collaborationMode: options?.collaborationMode ?? null,
  });
}

export async function interruptTurn(
  workspaceId: string,
  threadId: string,
  turnId: string,
) {
  return invoke("turn_interrupt", { workspaceId, threadId, turnId });
}

export async function startReview(
  workspaceId: string,
  threadId: string,
  target: ReviewTarget,
  delivery?: "inline" | "detached",
) {
  const payload: Record<string, unknown> = { workspaceId, threadId, target };
  if (delivery) {
    payload.delivery = delivery;
  }
  return invoke("start_review", payload);
}

export async function respondToServerRequest(
  workspaceId: string,
  requestId: number | string,
  result: "accept" | "decline" | Record<string, unknown>,
) {
  const payload = typeof result === "string" ? { decision: result } : result;
  return invoke("respond_to_server_request", {
    workspaceId,
    requestId,
    result: payload,
  });
}

export async function rememberApprovalRule(
  workspaceId: string,
  command: string[],
) {
  return invoke("remember_approval_rule", { workspaceId, command });
}

export async function getGitStatus(workspace_id: string): Promise<{
  branchName: string;
  files: GitFileStatus[];
  stagedFiles: GitFileStatus[];
  unstagedFiles: GitFileStatus[];
  totalAdditions: number;
  totalDeletions: number;
}> {
  return invoke("get_git_status", { workspaceId: workspace_id });
}

export async function listGitRoots(
  workspace_id: string,
  depth: number,
): Promise<string[]> {
  return invoke("list_git_roots", { workspaceId: workspace_id, depth });
}

export async function getGitDiffs(
  workspace_id: string,
): Promise<GitFileDiff[]> {
  return invoke("get_git_diffs", { workspaceId: workspace_id });
}

export async function getGitLog(
  workspace_id: string,
  limit = 40,
): Promise<GitLogResponse> {
  return invoke("get_git_log", { workspaceId: workspace_id, limit });
}

export async function getGitCommitDiff(
  workspace_id: string,
  sha: string,
): Promise<GitCommitDiff[]> {
  return invoke("get_git_commit_diff", { workspaceId: workspace_id, sha });
}

export async function getGitRemote(workspace_id: string): Promise<string | null> {
  return invoke("get_git_remote", { workspaceId: workspace_id });
}

export async function memoryStatus(): Promise<MemoryStatus> {
  return invoke<MemoryStatus>("memory_status");
}

export async function memorySearch(
  query: string,
  limit = 10,
): Promise<MemorySearchResult[]> {
  return invoke<MemorySearchResult[]>("memory_search", { query, limit });
}

export async function memoryAppend(
  type: "daily" | "curated",
  content: string,
  tags: string[] = [],
  workspaceId?: string | null,
): Promise<MemoryEntry> {
  return invoke<MemoryEntry>("memory_append", {
    type,
    content,
    tags,
    workspace_id: workspaceId ?? null,
  });
}

export async function memoryBootstrap(): Promise<MemorySearchResult[]> {
  return invoke<MemorySearchResult[]>("memory_bootstrap");
}

export async function memoryFlushNow(
  workspaceId: string,
  threadId: string,
  force = false,
): Promise<unknown> {
  return invoke("memory_flush_now", { workspaceId, threadId, force });
}

export async function browserCreateSession(params: Record<string, unknown> = {}) {
  return invoke("browser_create_session", params);
}

export async function browserListSessions() {
  return invoke("browser_list_sessions", {});
}

export async function browserCloseSession(sessionId: string) {
  return invoke("browser_close_session", { sessionId });
}

export async function browserNavigate(params: Record<string, unknown>) {
  return invoke("browser_navigate", params);
}

export async function browserScreenshot(params: Record<string, unknown>) {
  return invoke("browser_screenshot", params);
}

export async function browserClick(params: Record<string, unknown>) {
  return invoke("browser_click", params);
}

export async function browserType(params: Record<string, unknown>) {
  return invoke("browser_type", params);
}

export async function browserPress(params: Record<string, unknown>) {
  return invoke("browser_press", params);
}

export async function browserSnapshot(params: Record<string, unknown>) {
  return invoke("browser_snapshot", params);
}

export async function browserEvaluate(params: Record<string, unknown>) {
  return invoke("browser_evaluate", params);
}

export async function skillsConfigWrite(
  workspaceId: string,
  config: Record<string, unknown>,
) {
  return invoke("skills_config_write", { workspaceId, config });
}

export async function skillsConfigRead(workspaceId: string) {
  return invoke("skills_config_read", { workspaceId });
}

export async function skillsValidate(workspaceId: string) {
  return invoke("skills_validate", { workspaceId });
}

export async function skillsInstallFromGit(
  sourceUrl: string,
  target: "global" | "workspace",
  workspaceId?: string | null,
) {
  return invoke("skills_install_from_git", {
    sourceUrl,
    target,
    workspaceId: workspaceId ?? null,
  });
}

export async function skillsUninstall(
  name: string,
  target: "global" | "workspace",
  workspaceId?: string | null,
) {
  return invoke("skills_uninstall", {
    name,
    target,
    workspaceId: workspaceId ?? null,
  });
}

export async function stageGitFile(workspaceId: string, path: string) {
  return invoke("stage_git_file", { workspaceId, path });
}

export async function stageGitAll(workspaceId: string): Promise<void> {
  return invoke("stage_git_all", { workspaceId });
}

export async function unstageGitFile(workspaceId: string, path: string) {
  return invoke("unstage_git_file", { workspaceId, path });
}

export async function revertGitFile(workspaceId: string, path: string) {
  return invoke("revert_git_file", { workspaceId, path });
}

export async function revertGitAll(workspaceId: string) {
  return invoke("revert_git_all", { workspaceId });
}

export async function commitGit(
  workspaceId: string,
  message: string,
): Promise<void> {
  return invoke("commit_git", { workspaceId, message });
}

export async function pushGit(workspaceId: string): Promise<void> {
  return invoke("push_git", { workspaceId });
}

export async function pullGit(workspaceId: string): Promise<void> {
  return invoke("pull_git", { workspaceId });
}

export async function syncGit(workspaceId: string): Promise<void> {
  return invoke("sync_git", { workspaceId });
}

export async function getGitHubIssues(
  workspace_id: string,
): Promise<GitHubIssuesResponse> {
  return invoke("get_github_issues", { workspaceId: workspace_id });
}

export async function getGitHubPullRequests(
  workspace_id: string,
): Promise<GitHubPullRequestsResponse> {
  return invoke("get_github_pull_requests", { workspaceId: workspace_id });
}

export async function getGitHubPullRequestDiff(
  workspace_id: string,
  prNumber: number,
): Promise<GitHubPullRequestDiff[]> {
  return invoke("get_github_pull_request_diff", {
    workspaceId: workspace_id,
    prNumber,
  });
}

export async function getGitHubPullRequestComments(
  workspace_id: string,
  prNumber: number,
): Promise<GitHubPullRequestComment[]> {
  return invoke("get_github_pull_request_comments", {
    workspaceId: workspace_id,
    prNumber,
  });
}

export async function localUsageSnapshot(
  days?: number,
  workspacePath?: string | null,
): Promise<LocalUsageSnapshot> {
  const payload: { days: number; workspacePath?: string } = { days: days ?? 30 };
  if (workspacePath) {
    payload.workspacePath = workspacePath;
  }
  return invoke("local_usage_snapshot", payload);
}

export async function getModelList(workspaceId: string) {
  return invoke<any>("model_list", { workspaceId });
}

export async function getCollaborationModes(workspaceId: string) {
  return invoke<any>("collaboration_mode_list", { workspaceId });
}

export async function getAccountRateLimits(workspaceId: string) {
  return invoke<any>("account_rate_limits", { workspaceId });
}

export async function getSkillsList(workspaceId: string) {
  return invoke<any>("skills_list", { workspaceId });
}

export async function getPromptsList(workspaceId: string) {
  return invoke<any>("prompts_list", { workspaceId });
}

export async function getWorkspacePromptsDir(workspaceId: string) {
  return invoke<string>("prompts_workspace_dir", { workspaceId });
}

export async function getGlobalPromptsDir() {
  return invoke<string>("prompts_global_dir");
}

export async function createPrompt(
  workspaceId: string,
  data: {
    scope: "workspace" | "global";
    name: string;
    description?: string | null;
    argumentHint?: string | null;
    content: string;
  },
) {
  return invoke<any>("prompts_create", {
    workspaceId,
    scope: data.scope,
    name: data.name,
    description: data.description ?? null,
    argumentHint: data.argumentHint ?? null,
    content: data.content,
  });
}

export async function updatePrompt(
  workspaceId: string,
  data: {
    path: string;
    name: string;
    description?: string | null;
    argumentHint?: string | null;
    content: string;
  },
) {
  return invoke<any>("prompts_update", {
    workspaceId,
    path: data.path,
    name: data.name,
    description: data.description ?? null,
    argumentHint: data.argumentHint ?? null,
    content: data.content,
  });
}

export async function deletePrompt(workspaceId: string, path: string) {
  return invoke<any>("prompts_delete", { workspaceId, path });
}

export async function movePrompt(
  workspaceId: string,
  data: { path: string; scope: "workspace" | "global" },
) {
  return invoke<any>("prompts_move", {
    workspaceId,
    path: data.path,
    scope: data.scope,
  });
}

export async function getAppSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_app_settings");
}

export async function updateAppSettings(settings: AppSettings): Promise<AppSettings> {
  return invoke<AppSettings>("update_app_settings", { settings });
}

type MenuAcceleratorUpdate = {
  id: string;
  accelerator: string | null;
};

export async function setMenuAccelerators(
  updates: MenuAcceleratorUpdate[],
): Promise<void> {
  return invoke("menu_set_accelerators", { updates });
}

export async function runCodexDoctor(
  codexBin: string | null,
): Promise<CodexDoctorResult> {
  return invoke<CodexDoctorResult>("codex_doctor", { codexBin });
}

export async function getWorkspaceFiles(workspaceId: string) {
  return invoke<string[]>("list_workspace_files", { workspaceId });
}

export async function readWorkspaceFile(
  workspaceId: string,
  path: string,
): Promise<{ content: string; truncated: boolean }> {
  return invoke<{ content: string; truncated: boolean }>("read_workspace_file", {
    workspaceId,
    path,
  });
}

export async function readGlobalAgentsMd(): Promise<TextFileResponse> {
  return invoke<TextFileResponse>("read_global_agents_md");
}

export async function writeGlobalAgentsMd(content: string): Promise<void> {
  return invoke("write_global_agents_md", { content });
}

export async function readGlobalCodexConfigToml(): Promise<TextFileResponse> {
  return invoke<TextFileResponse>("read_global_config_toml");
}

export async function writeGlobalCodexConfigToml(content: string): Promise<void> {
  return invoke("write_global_config_toml", { content });
}

export async function listGitBranches(workspaceId: string) {
  return invoke<any>("list_git_branches", { workspaceId });
}

export async function checkoutGitBranch(workspaceId: string, name: string) {
  return invoke("checkout_git_branch", { workspaceId, name });
}

export async function createGitBranch(workspaceId: string, name: string) {
  return invoke("create_git_branch", { workspaceId, name });
}

function withModelId(modelId?: string | null) {
  return modelId ? { modelId } : {};
}

export async function getDictationModelStatus(
  modelId?: string | null,
): Promise<DictationModelStatus> {
  return invoke<DictationModelStatus>(
    "dictation_model_status",
    withModelId(modelId),
  );
}

export async function downloadDictationModel(
  modelId?: string | null,
): Promise<DictationModelStatus> {
  return invoke<DictationModelStatus>(
    "dictation_download_model",
    withModelId(modelId),
  );
}

export async function cancelDictationDownload(
  modelId?: string | null,
): Promise<DictationModelStatus> {
  return invoke<DictationModelStatus>(
    "dictation_cancel_download",
    withModelId(modelId),
  );
}

export async function removeDictationModel(
  modelId?: string | null,
): Promise<DictationModelStatus> {
  return invoke<DictationModelStatus>(
    "dictation_remove_model",
    withModelId(modelId),
  );
}

export async function startDictation(
  preferredLanguage: string | null,
): Promise<DictationSessionState> {
  return invoke("dictation_start", { preferredLanguage });
}

export async function stopDictation(): Promise<DictationSessionState> {
  return invoke("dictation_stop");
}

export async function cancelDictation(): Promise<DictationSessionState> {
  return invoke("dictation_cancel");
}

export async function openTerminalSession(
  workspaceId: string,
  terminalId: string,
  cols: number,
  rows: number,
): Promise<{ id: string }> {
  return invoke("terminal_open", { workspaceId, terminalId, cols, rows });
}

export async function writeTerminalSession(
  workspaceId: string,
  terminalId: string,
  data: string,
): Promise<void> {
  return invoke("terminal_write", { workspaceId, terminalId, data });
}

export async function resizeTerminalSession(
  workspaceId: string,
  terminalId: string,
  cols: number,
  rows: number,
): Promise<void> {
  return invoke("terminal_resize", { workspaceId, terminalId, cols, rows });
}

export async function closeTerminalSession(
  workspaceId: string,
  terminalId: string,
): Promise<void> {
  return invoke("terminal_close", { workspaceId, terminalId });
}

export async function listThreads(
  workspaceId: string,
  cursor?: string | null,
  limit?: number | null,
) {
  return invoke<any>("list_threads", { workspaceId, cursor, limit });
}

export async function listSessionThreads(
  workspacePath: string,
  limit?: number | null,
): Promise<SessionThreadInfo[]> {
  const response = await invoke<any>("list_session_threads", {
    workspacePath,
    limit,
  });
  const data = (response?.data ?? response) as SessionThreadInfo[] | undefined;
  return Array.isArray(data) ? data : [];
}

export async function resumeThread(workspaceId: string, threadId: string) {
  return invoke<any>("resume_thread", { workspaceId, threadId });
}

export async function archiveThread(workspaceId: string, threadId: string) {
  return invoke<any>("archive_thread", { workspaceId, threadId });
}

export async function getCommitMessagePrompt(
  workspaceId: string,
): Promise<string> {
  return invoke("get_commit_message_prompt", { workspaceId });
}

export async function generateCommitMessage(
  workspaceId: string,
): Promise<string> {
  return invoke("generate_commit_message", { workspaceId });
}
