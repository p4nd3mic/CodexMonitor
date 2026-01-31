# Data Models (Unified Reference)

This file is the **cross-platform type reference** for CodexMonitor.

Sources of truth by platform:

- **Desktop (TypeScript):** `src/types.ts` and a few service-local event types (e.g. `src/services/events.ts`)

- **iOS (Swift):** `ios/Packages/CodexMonitorRPC/Sources/CodexMonitorModels/Models.swift`

- **Daemon / Backend (Rust):** `src-tauri/src/types.rs` plus a few backend structs (`src-tauri/src/backend/events.rs`, `src-tauri/src/terminal.rs`, `src-tauri/src/workspaces.rs`)


Conventions:

- Wire field names are a **mix** of `camelCase` and `snake_case`.

  - Example: `WorkspaceInfo.codex_bin` uses snake_case.

  - Example: `WorkspaceInfo.parentId` uses camelCase.

- The daemon API also mixes styles in request params (mostly camelCase).

- Swift uses `CodingKeys` to map to these wire names.


> Tip: When adding/changing a type, update **all three**: TS (`src/types.ts`), Swift (`Models.swift`), and Rust (`types.rs`), then verify serialization names.


---

## Index

- [WorkspaceSettings](#workspacesettings)

- [WorkspaceGroup](#workspacegroup)

- [WorkspaceKind](#workspacekind)

- [WorktreeInfo](#worktreeinfo)

- [WorkspaceInfo](#workspaceinfo)

- [AppServerEvent](#appserverevent)

- [Message](#message)

- [ConversationItem](#conversationitem)

- [ThreadSummary](#threadsummary)

- [ReviewTarget](#reviewtarget)

- [AccessMode](#accessmode)

- [BackendMode](#backendmode)

- [ThemePreference](#themepreference)

- [ComposerEditorPreset](#composereditorpreset)

- [ComposerEditorSettings](#composereditorsettings)

- [AppSettings](#appsettings)

- [CodexDoctorResult](#codexdoctorresult)

- [ApprovalRequest](#approvalrequest)

- [RequestUserInputOption](#requestuserinputoption)

- [RequestUserInputQuestion](#requestuserinputquestion)

- [RequestUserInputParams](#requestuserinputparams)

- [RequestUserInputRequest](#requestuserinputrequest)

- [RequestUserInputAnswer](#requestuserinputanswer)

- [RequestUserInputResponse](#requestuserinputresponse)

- [GitFileStatus](#gitfilestatus)

- [GitFileDiff](#gitfilediff)

- [GitCommitDiff](#gitcommitdiff)

- [GitLogEntry](#gitlogentry)

- [GitLogResponse](#gitlogresponse)

- [GitHubIssue](#githubissue)

- [GitHubIssuesResponse](#githubissuesresponse)

- [GitHubUser](#githubuser)

- [GitHubPullRequest](#githubpullrequest)

- [GitHubPullRequestsResponse](#githubpullrequestsresponse)

- [GitHubPullRequestDiff](#githubpullrequestdiff)

- [GitHubPullRequestComment](#githubpullrequestcomment)

- [TokenUsageBreakdown](#tokenusagebreakdown)

- [ThreadTokenUsage](#threadtokenusage)

- [LocalUsageDay](#localusageday)

- [LocalUsageTotals](#localusagetotals)

- [LocalUsageModel](#localusagemodel)

- [LocalUsageSnapshot](#localusagesnapshot)

- [TurnPlanStepStatus](#turnplanstepstatus)

- [TurnPlanStep](#turnplanstep)

- [TurnPlan](#turnplan)

- [RateLimitWindow](#ratelimitwindow)

- [CreditsSnapshot](#creditssnapshot)

- [RateLimitSnapshot](#ratelimitsnapshot)

- [QueuedMessage](#queuedmessage)

- [ModelOption](#modeloption)

- [CollaborationModeOption](#collaborationmodeoption)

- [SkillOption](#skilloption)

- [CustomPromptOption](#custompromptoption)

- [BranchInfo](#branchinfo)

- [DebugEntry](#debugentry)

- [TerminalStatus](#terminalstatus)

- [DictationModelState](#dictationmodelstate)

- [DictationDownloadProgress](#dictationdownloadprogress)

- [DictationModelStatus](#dictationmodelstatus)

- [DictationSessionState](#dictationsessionstate)

- [DictationEvent](#dictationevent)

- [DictationTranscript](#dictationtranscript)

- [TerminalOutputEvent](#terminaloutputevent)

- [MessageRole](#messagerole)

- [ConversationItemKind](#conversationitemkind)

- [ReviewState](#reviewstate)

- [ToolChange](#toolchange)

- [ThreadTurn](#threadturn)

- [ThreadRecord](#threadrecord)

- [ThreadListResponse](#threadlistresponse)

- [ThreadStartResponse](#threadstartresponse)

- [ThreadResumeResponse](#threadresumeresponse)

- [ApprovalDecision](#approvaldecision)

- [ReviewDelivery](#reviewdelivery)

- [PingResponse](#pingresponse)

- [GitStatusResponse](#gitstatusresponse)

- [ReasoningEffortOption](#reasoningeffortoption)

- [PromptScope](#promptscope)

- [TerminalSessionInfo](#terminalsessioninfo)

- [WorkspaceFileResponse](#workspacefileresponse)

- [GitHubPullRequestAuthor](#githubpullrequestauthor)

- [WorkspaceEntry](#workspaceentry)

- [TerminalOutput](#terminaloutput)

- [TerminalSession](#terminalsession)

- [JSONValue](#jsonvalue)

- [RPCRequest](#rpcrequest)

- [RPCResponse](#rpcresponse)

- [RPCNotification](#rpcnotification)

- [RPCError](#rpcerror)

### Life Workspace Types

- [LifeDomain](#lifedomain)

- [DomainViewState](#domainviewstate)

- [TimeRange](#timerange)

- [DeliveryDashboard](#deliverydashboard)

- [NutritionDashboard](#nutritiondashboard)

- [MediaDashboard](#mediadashboard)

- [YouTubeDashboard](#youtubedashboard)

- [FinanceDashboard](#financedashboard)

- [TextFileResponse](#textfileresponse)

- [CoverOverrides](#coveroverrides)


---

## WorkspaceSettings

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `sidebarCollapsed` | `boolean` | no |  |
| `sortOrder` | `number \| null` | yes |  |
| `groupId` | `string \| null` | yes |  |
| `gitRoot` | `string \| null` | yes |  |


**Swift**

```swift
public struct WorkspaceSettings: Codable, Hashable, Sendable {
    public var sidebarCollapsed: Bool
    public var sortOrder: Int?
    public var groupId: String?
    public var gitRoot: String?

    public init(sidebarCollapsed: Bool = false, sortOrder: Int? = nil, groupId: String? = nil, gitRoot: String? = nil) {
        self.sidebarCollapsed = sidebarCollapsed
        self.sortOrder = sortOrder
        self.groupId = groupId
        self.gitRoot = gitRoot
    }

    enum CodingKeys: String, CodingKey {
        case sidebarCollapsed
        case sortOrder
        case groupId
        case gitRoot
    }
}
```

**TypeScript**

```ts
export type WorkspaceSettings = {
  sidebarCollapsed: boolean;
  sortOrder?: number | null;
  groupId?: string | null;
  gitRoot?: string | null;
};
```

**Rust**

```rust
pub(crate) struct WorkspaceSettings {
    #[serde(default, rename = "sidebarCollapsed")]
    pub(crate) sidebar_collapsed: bool,
    #[serde(default, rename = "sortOrder")]
    pub(crate) sort_order: Option<u32>,
    #[serde(default, rename = "groupId")]
    pub(crate) group_id: Option<String>,
    #[serde(default, rename = "gitRoot")]
    pub(crate) git_root: Option<String>,
}
```

---

## WorkspaceGroup

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `string` | no | Unique identifier. |
| `name` | `string` | no | Human-friendly name. |
| `sortOrder` | `number \| null` | yes |  |
| `copiesFolder` | `string \| null` | yes |  |


**Swift**

```swift
public struct WorkspaceGroup: Codable, Hashable, Sendable, Identifiable {
    public var id: String
    public var name: String
    public var sortOrder: Int?
    public var copiesFolder: String?
}
```

**TypeScript**

```ts
export type WorkspaceGroup = {
  id: string;
  name: string;
  sortOrder?: number | null;
  copiesFolder?: string | null;
};
```

**Rust**

```rust
pub(crate) struct WorkspaceGroup {
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(default, rename = "sortOrder")]
    pub(crate) sort_order: Option<u32>,
    #[serde(default, rename = "copiesFolder")]
    pub(crate) copies_folder: Option<String>,
}
```

---

## WorkspaceKind

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum WorkspaceKind: String, Codable, Sendable {
    case main
    case worktree
}
```

**TypeScript**

```ts
export type WorkspaceKind = "main" | "worktree";
```

**Rust**

```rust
pub(crate) enum WorkspaceKind {
    Main,
    Worktree,
}
```

---

## WorktreeInfo

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `branch` | `string` | no | Git branch name. |


**Swift**

```swift
public struct WorktreeInfo: Codable, Hashable, Sendable {
    public var branch: String
}
```

**TypeScript**

```ts
export type WorktreeInfo = {
  branch: string;
};
```

**Rust**

```rust
pub(crate) struct WorktreeInfo {
    pub(crate) branch: String,
}
```

---

## WorkspaceInfo

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `string` | no | Unique identifier. |
| `name` | `string` | no | Human-friendly name. |
| `path` | `string` | no | Filesystem path (absolute unless documented otherwise). |
| `connected` | `boolean` | no |  |
| `codex_bin` | `string \| null` | yes |  |
| `kind` | `WorkspaceKind` | yes |  |
| `parentId` | `string \| null` | yes |  |
| `worktree` | `WorktreeInfo \| null` | yes |  |
| `settings` | `WorkspaceSettings` | no |  |


**Swift**

```swift
public struct WorkspaceInfo: Codable, Hashable, Sendable, Identifiable {
    public var id: String
    public var name: String
    public var path: String
    public var connected: Bool
    public var codexBin: String?
    public var kind: WorkspaceKind?
    public var parentId: String?
    public var worktree: WorktreeInfo?
    public var settings: WorkspaceSettings

    enum CodingKeys: String, CodingKey {
        case id, name, path, connected, kind, worktree, settings
        case codexBin = "codex_bin"
        case parentId = "parentId"
    }
}
```

**TypeScript**

```ts
export type WorkspaceInfo = {
  id: string;
  name: string;
  path: string;
  connected: boolean;
  codex_bin?: string | null;
  kind?: WorkspaceKind;
  parentId?: string | null;
  worktree?: WorktreeInfo | null;
  settings: WorkspaceSettings;
};
```

**Rust**

```rust
pub(crate) struct WorkspaceInfo {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) connected: bool,
    pub(crate) codex_bin: Option<String>,
    #[serde(default)]
    pub(crate) kind: WorkspaceKind,
    #[serde(default, rename = "parentId")]
    pub(crate) parent_id: Option<String>,
    #[serde(default)]
    pub(crate) worktree: Option<WorktreeInfo>,
    #[serde(default)]
    pub(crate) settings: WorkspaceSettings,
}
```

**Notes**

- `codex_bin` is snake_case on the wire; `parentId` is camelCase. This mixed naming is intentional and implemented via Swift `CodingKeys` and Rust `#[serde(rename=...)]`.

- `kind`, `parentId`, and `worktree` are optional in Swift and TS; `kind` defaults to `main` in some creation paths.



---

## AppServerEvent

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `workspace_id` | `string` | no | Workspace identifier. |
| `message` | `Record<string, unknown>` | no | Message payload (often arbitrary JSON). |


**Swift**

```swift
public struct AppServerEvent: Codable, Sendable {
    public let workspaceId: String
    public let message: JSONValue

    enum CodingKeys: String, CodingKey {
        case workspaceId = "workspace_id"
        case message
    }
}
```

**TypeScript**

```ts
export type AppServerEvent = {
  workspace_id: string;
  message: Record<string, unknown>;
};
```

**Rust**

```rust
pub(crate) struct AppServerEvent {
    pub(crate) workspace_id: String,
    pub(crate) message: Value,
}
```

**Notes**

- `message` is **opaque** Codex app-server JSON. Desktop parses it by `method` string (see `useAppServerEvents`); iOS uses `JSONValue` / helper decoders in the store.



---

## Message

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `string` | no | Unique identifier. |
| `role` | `"user" \| "assistant"` | no |  |
| `text` | `string` | no |  |


**Swift**

_N/A_


**TypeScript**

```ts
export type Message = {
  id: string;
  role: "user" | "assistant";
  text: string;
};
```

**Rust**

_N/A_


---

## ConversationItem

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `String` | no | Unique identifier. |
| `kind` | `ConversationItemKind` | no |  |
| `role` | `MessageRole?` | yes |  |
| `text` | `String?` | yes |  |
| `summary` | `String?` | yes |  |
| `content` | `String?` | yes | Content payload (often text). |
| `title` | `String?` | yes | Title string. |
| `diff` | `String?` | yes |  |
| `state` | `ReviewState?` | yes | State enum. |
| `toolType` | `String?` | yes |  |
| `detail` | `String?` | yes |  |
| `status` | `String?` | yes | Status enum. |
| `output` | `String?` | yes |  |
| `durationMs` | `Double?` | yes |  |
| `changes` | `[ToolChange]?` | yes |  |


**Swift**

```swift
public struct ConversationItem: Identifiable, Hashable, Sendable {
    public var id: String
    public var kind: ConversationItemKind
    public var role: MessageRole? = nil
    public var text: String? = nil
    public var summary: String? = nil
    public var content: String? = nil
    public var title: String? = nil
    public var diff: String? = nil
    public var state: ReviewState? = nil
    public var toolType: String? = nil
    public var detail: String? = nil
    public var status: String? = nil
    public var output: String? = nil
    public var durationMs: Double? = nil
    public var changes: [ToolChange]? = nil

    public init(
        id: String,
        kind: ConversationItemKind,
        role: MessageRole? = nil,
        text: String? = nil,
        summary: String? = nil,
        content: String? = nil,
        title: String? = nil,
        diff: String? = nil,
        state: ReviewState? = nil,
        toolType: String? = nil,
        detail: String? = nil,
        status: String? = nil,
        output: String? = nil,
        durationMs: Double? = nil,
        changes: [ToolChange]? = nil
    ) {
        self.id = id
        self.kind = kind
        self.role = role
        self.text = text
        self.summary = summary
        self.content = content
        self.title = title
        self.diff = diff
        self.state = state
        self.toolType = toolType
        self.detail = detail
        self.status = status
        self.output = output
        self.durationMs = durationMs
        self.changes = changes
    }
}
```

**TypeScript**

```ts
export type ConversationItem = | { id: string; kind: "message"; role: "user" | "assistant"; text: string }
  | { id: string; kind: "reasoning"; summary: string; content: string }
  | { id: string; kind: "diff"; title: string; diff: string; status?: string }
  | { id: string; kind: "review"; state: "started" | "completed"; text: string }
  | {
      id: string;
      kind: "tool";
      toolType: string;
      title: string;
      detail: string;
      status?: string;
      output?: string;
      durationMs?: number | null;
      changes?: { path: string; kind?: string; diff?: string }[];
    };
```

**Rust**

_N/A_


---

## ThreadSummary

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `string` | no | Unique identifier. |
| `name` | `string` | no | Human-friendly name. |
| `updatedAt` | `number` | no | Last-updated timestamp. |


**Swift**

```swift
public struct ThreadSummary: Codable, Hashable, Sendable, Identifiable {
    public var id: String
    public var name: String
    public var updatedAt: Double

    public init(id: String, name: String, updatedAt: Double) {
        self.id = id
        self.name = name
        self.updatedAt = updatedAt
    }
}
```

**TypeScript**

```ts
export type ThreadSummary = {
  id: string;
  name: string;
  updatedAt: number;
};
```

**Rust**

_N/A_


---

## ReviewTarget

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `type` | `String` | no |  |
| `branch` | `String?` | yes | Git branch name. |
| `sha` | `String?` | yes | Git commit SHA. |
| `title` | `String?` | yes | Title string. |
| `instructions` | `String?` | yes |  |


**Swift**

```swift
public struct ReviewTarget: Codable, Hashable, Sendable {
    public var type: String
    public var branch: String?
    public var sha: String?
    public var title: String?
    public var instructions: String?

    public static func uncommittedChanges() -> ReviewTarget { .init(type: "uncommittedChanges", branch: nil, sha: nil, title: nil, instructions: nil) }
    public static func baseBranch(_ branch: String) -> ReviewTarget { .init(type: "baseBranch", branch: branch, sha: nil, title: nil, instructions: nil) }
    public static func commit(sha: String, title: String?) -> ReviewTarget { .init(type: "commit", branch: nil, sha: sha, title: title, instructions: nil) }
    public static func custom(_ instructions: String) -> ReviewTarget { .init(type: "custom", branch: nil, sha: nil, title: nil, instructions: instructions) }
}
```

**TypeScript**

```ts
export type ReviewTarget = | { type: "uncommittedChanges" }
  | { type: "baseBranch"; branch: string }
  | { type: "commit"; sha: string; title?: string }
  | { type: "custom"; instructions: string };
```

**Rust**

_N/A_


---

## AccessMode

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum AccessMode: String, Codable, Sendable {
    case readOnly = "read-only"
    case current = "current"
    case fullAccess = "full-access"
}
```

**TypeScript**

```ts
export type AccessMode = "read-only" | "current" | "full-access";
```

**Rust**

_N/A_


---

## BackendMode

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum BackendMode: String, Codable, Sendable {
    case local
    case remote
}
```

**TypeScript**

```ts
export type BackendMode = "local" | "remote";
```

**Rust**

```rust
pub(crate) enum BackendMode {
    Local,
    Remote,
}
```

---

## ThemePreference

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum ThemePreference: String, Codable, Sendable {
    case system
    case light
    case dark
}
```

**TypeScript**

```ts
export type ThemePreference = "system" | "light" | "dark";
```

**Rust**

_N/A_


---

## ComposerEditorPreset

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum ComposerEditorPreset: String, Codable, Sendable {
    case `default`
    case helpful
    case smart
}
```

**TypeScript**

```ts
export type ComposerEditorPreset = "default" | "helpful" | "smart";
```

**Rust**

_N/A_


---

## ComposerEditorSettings

**Used in:** iOS ❌, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `preset` | `ComposerEditorPreset` | no |  |
| `expandFenceOnSpace` | `boolean` | no |  |
| `expandFenceOnEnter` | `boolean` | no |  |
| `fenceLanguageTags` | `boolean` | no |  |
| `fenceWrapSelection` | `boolean` | no |  |
| `autoWrapPasteMultiline` | `boolean` | no |  |
| `autoWrapPasteCodeLike` | `boolean` | no |  |
| `continueListOnShiftEnter` | `boolean` | no |  |


**Swift**

_N/A_


**TypeScript**

```ts
export type ComposerEditorSettings = {
  preset: ComposerEditorPreset;
  expandFenceOnSpace: boolean;
  expandFenceOnEnter: boolean;
  fenceLanguageTags: boolean;
  fenceWrapSelection: boolean;
  autoWrapPasteMultiline: boolean;
  autoWrapPasteCodeLike: boolean;
  continueListOnShiftEnter: boolean;
};
```

**Rust**

_N/A_


---

## AppSettings

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `codexBin` | `string \| null` | no |  |
| `backendMode` | `BackendMode` | no |  |
| `remoteBackendHost` | `string` | no |  |
| `remoteBackendToken` | `string \| null` | no |  |
| `defaultAccessMode` | `AccessMode` | no |  |
| `composerModelShortcut` | `string \| null` | no |  |
| `composerAccessShortcut` | `string \| null` | no |  |
| `composerReasoningShortcut` | `string \| null` | no |  |
| `newAgentShortcut` | `string \| null` | no |  |
| `newWorktreeAgentShortcut` | `string \| null` | no |  |
| `newCloneAgentShortcut` | `string \| null` | no |  |
| `toggleProjectsSidebarShortcut` | `string \| null` | no |  |
| `toggleGitSidebarShortcut` | `string \| null` | no |  |
| `toggleDebugPanelShortcut` | `string \| null` | no |  |
| `toggleTerminalShortcut` | `string \| null` | no |  |
| `cycleAgentNextShortcut` | `string \| null` | no |  |
| `cycleAgentPrevShortcut` | `string \| null` | no |  |
| `cycleWorkspaceNextShortcut` | `string \| null` | no |  |
| `cycleWorkspacePrevShortcut` | `string \| null` | no |  |
| `lastComposerModelId` | `string \| null` | no |  |
| `lastComposerReasoningEffort` | `string \| null` | no |  |
| `uiScale` | `number` | no |  |
| `theme` | `ThemePreference` | no |  |
| `uiFontFamily` | `string` | no |  |
| `codeFontFamily` | `string` | no |  |
| `codeFontSize` | `number` | no |  |
| `notificationSoundsEnabled` | `boolean` | no |  |
| `experimentalCollabEnabled` | `boolean` | no |  |
| `experimentalSteerEnabled` | `boolean` | no |  |
| `experimentalUnifiedExecEnabled` | `boolean` | no |  |
| `dictationEnabled` | `boolean` | no |  |
| `dictationModelId` | `string` | no |  |
| `dictationPreferredLanguage` | `string \| null` | no |  |
| `dictationHoldKey` | `string \| null` | no |  |
| `composerEditorPreset` | `ComposerEditorPreset` | no |  |
| `composerFenceExpandOnSpace` | `boolean` | no |  |
| `composerFenceExpandOnEnter` | `boolean` | no |  |
| `composerFenceLanguageTags` | `boolean` | no |  |
| `composerFenceWrapSelection` | `boolean` | no |  |
| `composerFenceAutoWrapPasteMultiline` | `boolean` | no |  |
| `composerFenceAutoWrapPasteCodeLike` | `boolean` | no |  |
| `composerListContinuation` | `boolean` | no |  |
| `composerCodeBlockCopyUseModifier` | `boolean` | no |  |
| `workspaceGroups` | `WorkspaceGroup[]` | no |  |


**Swift**

```swift
public struct AppSettings: Codable, Hashable, Sendable {
    public var codexBin: String?
    public var backendMode: BackendMode
    public var remoteBackendHost: String
    public var remoteBackendToken: String?
    public var defaultAccessMode: String
    public var composerModelShortcut: String?
    public var composerAccessShortcut: String?
    public var composerReasoningShortcut: String?
    public var newAgentShortcut: String?
    public var newWorktreeAgentShortcut: String?
    public var newCloneAgentShortcut: String?
    public var toggleProjectsSidebarShortcut: String?
    public var toggleGitSidebarShortcut: String?
    public var toggleDebugPanelShortcut: String?
    public var toggleTerminalShortcut: String?
    public var cycleAgentNextShortcut: String?
    public var cycleAgentPrevShortcut: String?
    public var cycleWorkspaceNextShortcut: String?
    public var cycleWorkspacePrevShortcut: String?
    public var lastComposerModelId: String?
    public var lastComposerReasoningEffort: String?
    public var uiScale: Double
    public var theme: ThemePreference
    public var uiFontFamily: String
    public var codeFontFamily: String
    public var codeFontSize: Int
    public var notificationSoundsEnabled: Bool
    public var experimentalCollabEnabled: Bool
    public var experimentalSteerEnabled: Bool
    public var experimentalUnifiedExecEnabled: Bool
    public var dictationEnabled: Bool
    public var dictationModelId: String
    public var dictationPreferredLanguage: String?
    public var dictationHoldKey: String
    public var composerEditorPreset: ComposerEditorPreset
    public var composerFenceExpandOnSpace: Bool
    public var composerFenceExpandOnEnter: Bool
    public var composerFenceLanguageTags: Bool
    public var composerFenceWrapSelection: Bool
    public var composerFenceAutoWrapPasteMultiline: Bool
    public var composerFenceAutoWrapPasteCodeLike: Bool
    public var composerListContinuation: Bool
    public var composerCodeBlockCopyUseModifier: Bool
    public var workspaceGroups: [WorkspaceGroup]
}
```

**TypeScript**

```ts
export type AppSettings = {
  codexBin: string | null;
  backendMode: BackendMode;
  remoteBackendHost: string;
  remoteBackendToken: string | null;
  defaultAccessMode: AccessMode;
  composerModelShortcut: string | null;
  composerAccessShortcut: string | null;
  composerReasoningShortcut: string | null;
  newAgentShortcut: string | null;
  newWorktreeAgentShortcut: string | null;
  newCloneAgentShortcut: string | null;
  toggleProjectsSidebarShortcut: string | null;
  toggleGitSidebarShortcut: string | null;
  toggleDebugPanelShortcut: string | null;
  toggleTerminalShortcut: string | null;
  cycleAgentNextShortcut: string | null;
  cycleAgentPrevShortcut: string | null;
  cycleWorkspaceNextShortcut: string | null;
  cycleWorkspacePrevShortcut: string | null;
  lastComposerModelId: string | null;
  lastComposerReasoningEffort: string | null;
  uiScale: number;
  theme: ThemePreference;
  uiFontFamily: string;
  codeFontFamily: string;
  codeFontSize: number;
  notificationSoundsEnabled: boolean;
  experimentalCollabEnabled: boolean;
  experimentalSteerEnabled: boolean;
  experimentalUnifiedExecEnabled: boolean;
  dictationEnabled: boolean;
  dictationModelId: string;
  dictationPreferredLanguage: string | null;
  dictationHoldKey: string | null;
  composerEditorPreset: ComposerEditorPreset;
  composerFenceExpandOnSpace: boolean;
  composerFenceExpandOnEnter: boolean;
  composerFenceLanguageTags: boolean;
  composerFenceWrapSelection: boolean;
  composerFenceAutoWrapPasteMultiline: boolean;
  composerFenceAutoWrapPasteCodeLike: boolean;
  composerListContinuation: boolean;
  composerCodeBlockCopyUseModifier: boolean;
  workspaceGroups: WorkspaceGroup[];
};
```

**Rust**

```rust
pub(crate) struct AppSettings {
    #[serde(default, rename = "codexBin")]
    pub(crate) codex_bin: Option<String>,
    #[serde(default, rename = "backendMode")]
    pub(crate) backend_mode: BackendMode,
    #[serde(default = "default_remote_backend_host", rename = "remoteBackendHost")]
    pub(crate) remote_backend_host: String,
    #[serde(default, rename = "remoteBackendToken")]
    pub(crate) remote_backend_token: Option<String>,
    #[serde(default = "default_access_mode", rename = "defaultAccessMode")]
    pub(crate) default_access_mode: String,
    #[serde(
        default = "default_composer_model_shortcut",
        rename = "composerModelShortcut"
    )]
    pub(crate) composer_model_shortcut: Option<String>,
    #[serde(
        default = "default_composer_access_shortcut",
        rename = "composerAccessShortcut"
    )]
    pub(crate) composer_access_shortcut: Option<String>,
    #[serde(
        default = "default_composer_reasoning_shortcut",
        rename = "composerReasoningShortcut"
    )]
    pub(crate) composer_reasoning_shortcut: Option<String>,
    #[serde(default = "default_new_agent_shortcut", rename = "newAgentShortcut")]
    pub(crate) new_agent_shortcut: Option<String>,
    #[serde(
        default = "default_new_worktree_agent_shortcut",
        rename = "newWorktreeAgentShortcut"
    )]
    pub(crate) new_worktree_agent_shortcut: Option<String>,
    #[serde(
        default = "default_new_clone_agent_shortcut",
        rename = "newCloneAgentShortcut"
    )]
    pub(crate) new_clone_agent_shortcut: Option<String>,
    #[serde(
        default = "default_toggle_projects_sidebar_shortcut",
        rename = "toggleProjectsSidebarShortcut"
    )]
    pub(crate) toggle_projects_sidebar_shortcut: Option<String>,
    #[serde(
        default = "default_toggle_git_sidebar_shortcut",
        rename = "toggleGitSidebarShortcut"
    )]
    pub(crate) toggle_git_sidebar_shortcut: Option<String>,
    #[serde(
        default = "default_toggle_debug_panel_shortcut",
        rename = "toggleDebugPanelShortcut"
    )]
    pub(crate) toggle_debug_panel_shortcut: Option<String>,
    #[serde(
        default = "default_toggle_terminal_shortcut",
        rename = "toggleTerminalShortcut"
    )]
    pub(crate) toggle_terminal_shortcut: Option<String>,
    #[serde(
        default = "default_cycle_agent_next_shortcut",
        rename = "cycleAgentNextShortcut"
    )]
    pub(crate) cycle_agent_next_shortcut: Option<String>,
    #[serde(
        default = "default_cycle_agent_prev_shortcut",
        rename = "cycleAgentPrevShortcut"
    )]
    pub(crate) cycle_agent_prev_shortcut: Option<String>,
    #[serde(
        default = "default_cycle_workspace_next_shortcut",
        rename = "cycleWorkspaceNextShortcut"
    )]
    pub(crate) cycle_workspace_next_shortcut: Option<String>,
    #[serde(
        default = "default_cycle_workspace_prev_shortcut",
        rename = "cycleWorkspacePrevShortcut"
    )]
    pub(crate) cycle_workspace_prev_shortcut: Option<String>,
    #[serde(default, rename = "lastComposerModelId")]
    pub(crate) last_composer_model_id: Option<String>,
    #[serde(default, rename = "lastComposerReasoningEffort")]
    pub(crate) last_composer_reasoning_effort: Option<String>,
    #[serde(default = "default_ui_scale", rename = "uiScale")]
    pub(crate) ui_scale: f64,
    #[serde(default = "default_theme", rename = "theme")]
    pub(crate) theme: String,
    #[serde(default = "default_ui_font_family", rename = "uiFontFamily")]
    pub(crate) ui_font_family: String,
    #[serde(default = "default_code_font_family", rename = "codeFontFamily")]
    pub(crate) code_font_family: String,
    #[serde(default = "default_code_font_size", rename = "codeFontSize")]
    pub(crate) code_font_size: u8,
    #[serde(
        default = "default_notification_sounds_enabled",
        rename = "notificationSoundsEnabled"
    )]
    pub(crate) notification_sounds_enabled: bool,
    #[serde(
        default = "default_experimental_collab_enabled",
        rename = "experimentalCollabEnabled"
    )]
    pub(crate) experimental_collab_enabled: bool,
    #[serde(
        default = "default_experimental_steer_enabled",
        rename = "experimentalSteerEnabled"
    )]
    pub(crate) experimental_steer_enabled: bool,
    #[serde(
        default = "default_experimental_unified_exec_enabled",
        rename = "experimentalUnifiedExecEnabled"
    )]
    pub(crate) experimental_unified_exec_enabled: bool,
    #[serde(default = "default_dictation_enabled", rename = "dictationEnabled")]
    pub(crate) dictation_enabled: bool,
    #[serde(default = "default_dictation_model_id", rename = "dictationModelId")]
    pub(crate) dictation_model_id: String,
    #[serde(default, rename = "dictationPreferredLanguage")]
    pub(crate) dictation_preferred_language: Option<String>,
    #[serde(default = "default_dictation_hold_key", rename = "dictationHoldKey")]
    pub(crate) dictation_hold_key: String,
    #[serde(
        default = "default_composer_editor_preset",
        rename = "composerEditorPreset"
    )]
    pub(crate) composer_editor_preset: String,
    #[serde(
        default = "default_composer_fence_expand_on_space",
        rename = "composerFenceExpandOnSpace"
    )]
    pub(crate) composer_fence_expand_on_space: bool,
    #[serde(
        default = "default_composer_fence_expand_on_enter",
        rename = "composerFenceExpandOnEnter"
    )]
    pub(crate) composer_fence_expand_on_enter: bool,
    #[serde(
        default = "default_composer_fence_language_tags",
        rename = "composerFenceLanguageTags"
    )]
    pub(crate) composer_fence_language_tags: bool,
    #[serde(
        default = "default_composer_fence_wrap_selection",
        rename = "composerFenceWrapSelection"
    )]
    pub(crate) composer_fence_wrap_selection: bool,
    #[serde(
        default = "default_composer_fence_auto_wrap_paste_multiline",
        rename = "composerFenceAutoWrapPasteMultiline"
    )]
    pub(crate) composer_fence_auto_wrap_paste_multiline: bool,
    #[serde(
        default = "default_composer_fence_auto_wrap_paste_code_like",
        rename = "composerFenceAutoWrapPasteCodeLike"
    )]
    pub(crate) composer_fence_auto_wrap_paste_code_like: bool,
    #[serde(
        default = "default_composer_list_continuation",
        rename = "composerListContinuation"
    )]
    pub(crate) composer_list_continuation: bool,
    #[serde(
        default = "default_composer_code_block_copy_use_modifier",
        rename = "composerCodeBlockCopyUseModifier"
    )]
    pub(crate) composer_code_block_copy_use_modifier: bool,
    #[serde(default = "default_workspace_groups", rename = "workspaceGroups")]
    pub(crate) workspace_groups: Vec<WorkspaceGroup>,
}
```

**Notes**

- iOS does not currently have a full `AppSettings` model; the mobile app stores its own daemon host/port/token settings separately.



---

## CodexDoctorResult

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `ok` | `boolean` | no | Operation success flag. |
| `codexBin` | `string \| null` | no |  |
| `version` | `string \| null` | no |  |
| `appServerOk` | `boolean` | no |  |
| `details` | `string \| null` | no |  |
| `path` | `string \| null` | no | Filesystem path (absolute unless documented otherwise). |
| `nodeOk` | `boolean` | no |  |
| `nodeVersion` | `string \| null` | no |  |
| `nodeDetails` | `string \| null` | no |  |


**Swift**

```swift
public struct CodexDoctorResult: Codable, Hashable, Sendable {
    public var ok: Bool
    public var codexBin: String?
    public var version: String?
    public var appServerOk: Bool
    public var details: String?
    public var path: String?
    public var nodeOk: Bool
    public var nodeVersion: String?
    public var nodeDetails: String?
}
```

**TypeScript**

```ts
export type CodexDoctorResult = {
  ok: boolean;
  codexBin: string | null;
  version: string | null;
  appServerOk: boolean;
  details: string | null;
  path: string | null;
  nodeOk: boolean;
  nodeVersion: string | null;
  nodeDetails: string | null;
};
```

**Rust**

_N/A_


---

## ApprovalRequest

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `workspace_id` | `string` | no | Workspace identifier. |
| `request_id` | `number` | no |  |
| `method` | `string` | no |  |
| `params` | `Record<string, unknown>` | no | Parameters payload (often arbitrary JSON). |


**Swift**

```swift
public struct ApprovalRequest: Codable, Hashable, Sendable, Identifiable {
    public var workspaceId: String
    public var requestId: Int
    public var method: String
    public var params: [String: JSONValue]

    public var id: String { "\(workspaceId)-\(requestId)" }

    public init(workspaceId: String, requestId: Int, method: String, params: [String: JSONValue]) {
        self.workspaceId = workspaceId
        self.requestId = requestId
        self.method = method
        self.params = params
    }

    enum CodingKeys: String, CodingKey {
        case workspaceId = "workspace_id"
        case requestId = "request_id"
        case method
        case params
    }
}
```

**TypeScript**

```ts
export type ApprovalRequest = {
  workspace_id: string;
  request_id: number;
  method: string;
  params: Record<string, unknown>;
};
```

**Rust**

_N/A_


---

## RequestUserInputOption

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `label` | `string` | no | Display label. |
| `description` | `string` | no | Helper text for the option. |


**Swift**

```swift
public struct RequestUserInputOption: Codable, Hashable, Sendable {
    public var label: String
    public var description: String
}
```

**TypeScript**

```ts
export type RequestUserInputOption = {
  label: string;
  description: string;
};
```

**Rust**

_N/A_


---

## RequestUserInputQuestion

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `string` | no | Question identifier. |
| `header` | `string` | no | Short label for grouping. |
| `question` | `string` | no | Prompt text shown to the user. |
| `options` | `RequestUserInputOption[]` | yes | Multiple-choice options (if omitted, use free text). |


**Swift**

```swift
public struct RequestUserInputQuestion: Codable, Hashable, Sendable {
    public var id: String
    public var header: String
    public var question: String
    public var options: [RequestUserInputOption]?
}
```

**TypeScript**

```ts
export type RequestUserInputQuestion = {
  id: string;
  header: string;
  question: string;
  options?: RequestUserInputOption[];
};
```

**Rust**

_N/A_


---

## RequestUserInputParams

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `thread_id` | `string` | no | Thread identifier. |
| `turn_id` | `string` | no | Turn identifier. |
| `item_id` | `string` | no | Item identifier. |
| `questions` | `RequestUserInputQuestion[]` | no | Questions to present to the user. |


**Swift**

```swift
public struct RequestUserInputParams: Codable, Hashable, Sendable {
    public var threadId: String
    public var turnId: String
    public var itemId: String
    public var questions: [RequestUserInputQuestion]

    enum CodingKeys: String, CodingKey {
        case threadId = "thread_id"
        case turnId = "turn_id"
        case itemId = "item_id"
        case questions
    }
}
```

**TypeScript**

```ts
export type RequestUserInputParams = {
  thread_id: string;
  turn_id: string;
  item_id: string;
  questions: RequestUserInputQuestion[];
};
```

**Rust**

_N/A_


---

## RequestUserInputRequest

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `workspace_id` | `string` | no | Workspace identifier. |
| `request_id` | `number | string` | no | Request identifier (JSON-RPC id). |
| `params` | `RequestUserInputParams` | no | Request payload. |


**Swift**

```swift
public struct RequestUserInputRequest: Codable, Hashable, Sendable, Identifiable {
    public var workspaceId: String
    public var requestId: JSONValue
    public var params: RequestUserInputParams

    public var id: String { "\(workspaceId)-\(requestId.asString())" }

    enum CodingKeys: String, CodingKey {
        case workspaceId = "workspace_id"
        case requestId = "request_id"
        case params
    }
}
```

**TypeScript**

```ts
export type RequestUserInputRequest = {
  workspace_id: string;
  request_id: number | string;
  params: RequestUserInputParams;
};
```

**Rust**

_N/A_


---

## RequestUserInputAnswer

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `answers` | `string[]` | no | Answers for a single question. |


**Swift**

```swift
public struct RequestUserInputAnswer: Codable, Hashable, Sendable {
    public var answers: [String]
}
```

**TypeScript**

```ts
export type RequestUserInputAnswer = {
  answers: string[];
};
```

**Rust**

_N/A_


---

## RequestUserInputResponse

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `answers` | `Record<string, RequestUserInputAnswer>` | no | Answers keyed by question id. |


**Swift**

```swift
public struct RequestUserInputResponse: Codable, Hashable, Sendable {
    public var answers: [String: RequestUserInputAnswer]
}
```

**TypeScript**

```ts
export type RequestUserInputResponse = {
  answers: Record<string, RequestUserInputAnswer>;
};
```

**Rust**

_N/A_


---

## GitFileStatus

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `path` | `string` | no | Filesystem path (absolute unless documented otherwise). |
| `status` | `string` | no | Status enum. |
| `additions` | `number` | no |  |
| `deletions` | `number` | no |  |


**Swift**

```swift
public struct GitFileStatus: Codable, Hashable, Sendable {
    public var path: String
    public var status: String
    public var additions: Int
    public var deletions: Int
}
```

**TypeScript**

```ts
export type GitFileStatus = {
  path: string;
  status: string;
  additions: number;
  deletions: number;
};
```

**Rust**

```rust
pub(crate) struct GitFileStatus {
    pub(crate) path: String,
    pub(crate) status: String,
    pub(crate) additions: i64,
    pub(crate) deletions: i64,
}
```

---

## GitFileDiff

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `path` | `string` | no | Filesystem path (absolute unless documented otherwise). |
| `diff` | `string` | no |  |


**Swift**

```swift
public struct GitFileDiff: Codable, Hashable, Sendable {
    public var path: String
    public var diff: String
}
```

**TypeScript**

```ts
export type GitFileDiff = {
  path: string;
  diff: string;
};
```

**Rust**

```rust
pub(crate) struct GitFileDiff {
    pub(crate) path: String,
    pub(crate) diff: String,
}
```

---

## GitCommitDiff

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `path` | `string` | no | Filesystem path (absolute unless documented otherwise). |
| `status` | `string` | no | Status enum. |
| `diff` | `string` | no |  |


**Swift**

```swift
public struct GitCommitDiff: Codable, Hashable, Sendable {
    public var path: String
    public var status: String
    public var diff: String
}
```

**TypeScript**

```ts
export type GitCommitDiff = {
  path: string;
  status: string;
  diff: string;
};
```

**Rust**

```rust
pub(crate) struct GitCommitDiff {
    pub(crate) path: String,
    pub(crate) status: String,
    pub(crate) diff: String,
}
```

---

## GitLogEntry

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `sha` | `string` | no | Git commit SHA. |
| `summary` | `string` | no |  |
| `author` | `string` | no |  |
| `timestamp` | `number` | no |  |


**Swift**

```swift
public struct GitLogEntry: Codable, Hashable, Sendable {
    public var sha: String
    public var summary: String
    public var author: String
    public var timestamp: Double
}
```

**TypeScript**

```ts
export type GitLogEntry = {
  sha: string;
  summary: string;
  author: string;
  timestamp: number;
};
```

**Rust**

```rust
pub(crate) struct GitLogEntry {
    pub(crate) sha: String,
    pub(crate) summary: String,
    pub(crate) author: String,
    pub(crate) timestamp: i64,
}
```

---

## GitLogResponse

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `total` | `number` | no |  |
| `entries` | `GitLogEntry[]` | no | List entries. |
| `ahead` | `number` | no |  |
| `behind` | `number` | no |  |
| `aheadEntries` | `GitLogEntry[]` | no |  |
| `behindEntries` | `GitLogEntry[]` | no |  |
| `upstream` | `string \| null` | no |  |


**Swift**

```swift
public struct GitLogResponse: Codable, Hashable, Sendable {
    public var total: Int
    public var entries: [GitLogEntry]
    public var ahead: Int
    public var behind: Int
    public var aheadEntries: [GitLogEntry]
    public var behindEntries: [GitLogEntry]
    public var upstream: String?
}
```

**TypeScript**

```ts
export type GitLogResponse = {
  total: number;
  entries: GitLogEntry[];
  ahead: number;
  behind: number;
  aheadEntries: GitLogEntry[];
  behindEntries: GitLogEntry[];
  upstream: string | null;
};
```

**Rust**

```rust
pub(crate) struct GitLogResponse {
    pub(crate) total: usize,
    pub(crate) entries: Vec<GitLogEntry>,
    #[serde(default)]
    pub(crate) ahead: usize,
    #[serde(default)]
    pub(crate) behind: usize,
    #[serde(default, rename = "aheadEntries")]
    pub(crate) ahead_entries: Vec<GitLogEntry>,
    #[serde(default, rename = "behindEntries")]
    pub(crate) behind_entries: Vec<GitLogEntry>,
    #[serde(default)]
    pub(crate) upstream: Option<String>,
}
```

---

## GitHubIssue

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `number` | `number` | no |  |
| `title` | `string` | no | Title string. |
| `url` | `string` | no | URL string. |
| `updatedAt` | `string` | no | Last-updated timestamp. |


**Swift**

```swift
public struct GitHubIssue: Codable, Hashable, Sendable {
    public var number: Int
    public var title: String
    public var url: String
    public var updatedAt: String
}
```

**TypeScript**

```ts
export type GitHubIssue = {
  number: number;
  title: string;
  url: string;
  updatedAt: string;
};
```

**Rust**

```rust
pub(crate) struct GitHubIssue {
    pub(crate) number: u64,
    pub(crate) title: String,
    pub(crate) url: String,
    #[serde(rename = "updatedAt")]
    pub(crate) updated_at: String,
}
```

---

## GitHubIssuesResponse

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `total` | `number` | no |  |
| `issues` | `GitHubIssue[]` | no |  |


**Swift**

```swift
public struct GitHubIssuesResponse: Codable, Hashable, Sendable {
    public var total: Int
    public var issues: [GitHubIssue]
}
```

**TypeScript**

```ts
export type GitHubIssuesResponse = {
  total: number;
  issues: GitHubIssue[];
};
```

**Rust**

```rust
pub(crate) struct GitHubIssuesResponse {
    pub(crate) total: usize,
    pub(crate) issues: Vec<GitHubIssue>,
}
```

---

## GitHubUser

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `login` | `string` | no |  |


**Swift**

```swift
public struct GitHubUser: Codable, Hashable, Sendable {
    public var login: String
}
```

**TypeScript**

```ts
export type GitHubUser = {
  login: string;
};
```

**Rust**

_N/A_


---

## GitHubPullRequest

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `number` | `number` | no |  |
| `title` | `string` | no | Title string. |
| `url` | `string` | no | URL string. |
| `updatedAt` | `string` | no | Last-updated timestamp. |
| `createdAt` | `string` | no | Creation timestamp. |
| `body` | `string` | no |  |
| `headRefName` | `string` | no |  |
| `baseRefName` | `string` | no |  |
| `isDraft` | `boolean` | no |  |
| `author` | `GitHubUser \| null` | no |  |


**Swift**

```swift
public struct GitHubPullRequest: Codable, Hashable, Sendable {
    public var number: Int
    public var title: String
    public var url: String
    public var updatedAt: String
    public var createdAt: String
    public var body: String
    public var headRefName: String
    public var baseRefName: String
    public var isDraft: Bool
    public var author: GitHubUser?
}
```

**TypeScript**

```ts
export type GitHubPullRequest = {
  number: number;
  title: string;
  url: string;
  updatedAt: string;
  createdAt: string;
  body: string;
  headRefName: string;
  baseRefName: string;
  isDraft: boolean;
  author: GitHubUser | null;
};
```

**Rust**

```rust
pub(crate) struct GitHubPullRequest {
    pub(crate) number: u64,
    pub(crate) title: String,
    pub(crate) url: String,
    #[serde(rename = "updatedAt")]
    pub(crate) updated_at: String,
    #[serde(rename = "createdAt")]
    pub(crate) created_at: String,
    pub(crate) body: String,
    #[serde(rename = "headRefName")]
    pub(crate) head_ref_name: String,
    #[serde(rename = "baseRefName")]
    pub(crate) base_ref_name: String,
    #[serde(rename = "isDraft")]
    pub(crate) is_draft: bool,
    #[serde(default)]
    pub(crate) author: Option<GitHubPullRequestAuthor>,
}
```

---

## GitHubPullRequestsResponse

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `total` | `number` | no |  |
| `pullRequests` | `GitHubPullRequest[]` | no |  |


**Swift**

```swift
public struct GitHubPullRequestsResponse: Codable, Hashable, Sendable {
    public var total: Int
    public var pullRequests: [GitHubPullRequest]

    enum CodingKeys: String, CodingKey {
        case total
        case pullRequests = "pullRequests"
    }
}
```

**TypeScript**

```ts
export type GitHubPullRequestsResponse = {
  total: number;
  pullRequests: GitHubPullRequest[];
};
```

**Rust**

```rust
pub(crate) struct GitHubPullRequestsResponse {
    pub(crate) total: usize,
    #[serde(rename = "pullRequests")]
    pub(crate) pull_requests: Vec<GitHubPullRequest>,
}
```

---

## GitHubPullRequestDiff

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `path` | `string` | no | Filesystem path (absolute unless documented otherwise). |
| `status` | `string` | no | Status enum. |
| `diff` | `string` | no |  |


**Swift**

```swift
public struct GitHubPullRequestDiff: Codable, Hashable, Sendable {
    public var path: String
    public var status: String
    public var diff: String
}
```

**TypeScript**

```ts
export type GitHubPullRequestDiff = {
  path: string;
  status: string;
  diff: string;
};
```

**Rust**

```rust
pub(crate) struct GitHubPullRequestDiff {
    pub(crate) path: String,
    pub(crate) status: String,
    pub(crate) diff: String,
}
```

---

## GitHubPullRequestComment

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `number` | no | Unique identifier. |
| `body` | `string` | no |  |
| `createdAt` | `string` | no | Creation timestamp. |
| `url` | `string` | no | URL string. |
| `author` | `GitHubUser \| null` | no |  |


**Swift**

```swift
public struct GitHubPullRequestComment: Codable, Hashable, Sendable {
    public var id: Int
    public var body: String
    public var createdAt: String
    public var url: String
    public var author: GitHubUser?
}
```

**TypeScript**

```ts
export type GitHubPullRequestComment = {
  id: number;
  body: string;
  createdAt: string;
  url: string;
  author: GitHubUser | null;
};
```

**Rust**

```rust
pub(crate) struct GitHubPullRequestComment {
    pub(crate) id: u64,
    #[serde(default)]
    pub(crate) body: String,
    #[serde(rename = "createdAt")]
    pub(crate) created_at: String,
    #[serde(default)]
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) author: Option<GitHubPullRequestAuthor>,
}
```

---

## TokenUsageBreakdown

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `totalTokens` | `number` | no |  |
| `inputTokens` | `number` | no |  |
| `cachedInputTokens` | `number` | no |  |
| `outputTokens` | `number` | no |  |
| `reasoningOutputTokens` | `number` | no |  |


**Swift**

```swift
public struct TokenUsageBreakdown: Codable, Hashable, Sendable {
    public var totalTokens: Int
    public var inputTokens: Int
    public var cachedInputTokens: Int
    public var outputTokens: Int
    public var reasoningOutputTokens: Int
}
```

**TypeScript**

```ts
export type TokenUsageBreakdown = {
  totalTokens: number;
  inputTokens: number;
  cachedInputTokens: number;
  outputTokens: number;
  reasoningOutputTokens: number;
};
```

**Rust**

_N/A_


---

## ThreadTokenUsage

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `total` | `TokenUsageBreakdown` | no |  |
| `last` | `TokenUsageBreakdown` | no |  |
| `modelContextWindow` | `number \| null` | no |  |


**Swift**

```swift
public struct ThreadTokenUsage: Codable, Hashable, Sendable {
    public var total: TokenUsageBreakdown
    public var last: TokenUsageBreakdown
    public var modelContextWindow: Int?
}
```

**TypeScript**

```ts
export type ThreadTokenUsage = {
  total: TokenUsageBreakdown;
  last: TokenUsageBreakdown;
  modelContextWindow: number | null;
};
```

**Rust**

_N/A_


---

## LocalUsageDay

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `day` | `string` | no |  |
| `inputTokens` | `number` | no |  |
| `cachedInputTokens` | `number` | no |  |
| `outputTokens` | `number` | no |  |
| `totalTokens` | `number` | no |  |
| `agentTimeMs` | `number` | no |  |
| `agentRuns` | `number` | no |  |


**Swift**

```swift
public struct LocalUsageDay: Codable, Hashable, Sendable {
    public var day: String
    public var inputTokens: Int
    public var cachedInputTokens: Int
    public var outputTokens: Int
    public var totalTokens: Int
    public var agentTimeMs: Int
    public var agentRuns: Int
}
```

**TypeScript**

```ts
export type LocalUsageDay = {
  day: string;
  inputTokens: number;
  cachedInputTokens: number;
  outputTokens: number;
  totalTokens: number;
  agentTimeMs: number;
  agentRuns: number;
};
```

**Rust**

```rust
pub(crate) struct LocalUsageDay {
    pub(crate) day: String,
    pub(crate) input_tokens: i64,
    pub(crate) cached_input_tokens: i64,
    pub(crate) output_tokens: i64,
    pub(crate) total_tokens: i64,
    #[serde(default)]
    pub(crate) agent_time_ms: i64,
    #[serde(default)]
    pub(crate) agent_runs: i64,
}
```

---

## LocalUsageTotals

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `last7DaysTokens` | `number` | no |  |
| `last30DaysTokens` | `number` | no |  |
| `averageDailyTokens` | `number` | no |  |
| `cacheHitRatePercent` | `number` | no |  |
| `peakDay` | `string \| null` | no |  |
| `peakDayTokens` | `number` | no |  |


**Swift**

```swift
public struct LocalUsageTotals: Codable, Hashable, Sendable {
    public var last7DaysTokens: Int
    public var last30DaysTokens: Int
    public var averageDailyTokens: Int
    public var cacheHitRatePercent: Double
    public var peakDay: String?
    public var peakDayTokens: Int
}
```

**TypeScript**

```ts
export type LocalUsageTotals = {
  last7DaysTokens: number;
  last30DaysTokens: number;
  averageDailyTokens: number;
  cacheHitRatePercent: number;
  peakDay: string | null;
  peakDayTokens: number;
};
```

**Rust**

```rust
pub(crate) struct LocalUsageTotals {
    pub(crate) last7_days_tokens: i64,
    pub(crate) last30_days_tokens: i64,
    pub(crate) average_daily_tokens: i64,
    pub(crate) cache_hit_rate_percent: f64,
    pub(crate) peak_day: Option<String>,
    pub(crate) peak_day_tokens: i64,
}
```

---

## LocalUsageModel

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `model` | `string` | no |  |
| `tokens` | `number` | no |  |
| `sharePercent` | `number` | no |  |


**Swift**

```swift
public struct LocalUsageModel: Codable, Hashable, Sendable {
    public var model: String
    public var tokens: Int
    public var sharePercent: Double
}
```

**TypeScript**

```ts
export type LocalUsageModel = {
  model: string;
  tokens: number;
  sharePercent: number;
};
```

**Rust**

```rust
pub(crate) struct LocalUsageModel {
    pub(crate) model: String,
    pub(crate) tokens: i64,
    pub(crate) share_percent: f64,
}
```

---

## LocalUsageSnapshot

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `updatedAt` | `number` | no | Last-updated timestamp. |
| `days` | `LocalUsageDay[]` | no |  |
| `totals` | `LocalUsageTotals` | no |  |
| `topModels` | `LocalUsageModel[]` | no |  |


**Swift**

```swift
public struct LocalUsageSnapshot: Codable, Hashable, Sendable {
    public var updatedAt: Double
    public var days: [LocalUsageDay]
    public var totals: LocalUsageTotals
    public var topModels: [LocalUsageModel]
}
```

**TypeScript**

```ts
export type LocalUsageSnapshot = {
  updatedAt: number;
  days: LocalUsageDay[];
  totals: LocalUsageTotals;
  topModels: LocalUsageModel[];
};
```

**Rust**

```rust
pub(crate) struct LocalUsageSnapshot {
    pub(crate) updated_at: i64,
    pub(crate) days: Vec<LocalUsageDay>,
    pub(crate) totals: LocalUsageTotals,
    #[serde(default)]
    pub(crate) top_models: Vec<LocalUsageModel>,
}
```

---

## TurnPlanStepStatus

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum TurnPlanStepStatus: String, Codable, Sendable {
    case pending
    case inProgress
    case completed
}
```

**TypeScript**

```ts
export type TurnPlanStepStatus = "pending" | "inProgress" | "completed";
```

**Rust**

_N/A_


---

## TurnPlanStep

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `step` | `string` | no |  |
| `status` | `TurnPlanStepStatus` | no | Status enum. |


**Swift**

```swift
public struct TurnPlanStep: Codable, Hashable, Sendable {
    public var step: String
    public var status: TurnPlanStepStatus
}
```

**TypeScript**

```ts
export type TurnPlanStep = {
  step: string;
  status: TurnPlanStepStatus;
};
```

**Rust**

_N/A_


---

## TurnPlan

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `turnId` | `string` | no | Turn identifier. |
| `explanation` | `string \| null` | no |  |
| `steps` | `TurnPlanStep[]` | no |  |


**Swift**

```swift
public struct TurnPlan: Codable, Hashable, Sendable {
    public var turnId: String
    public var explanation: String?
    public var steps: [TurnPlanStep]

    public init(turnId: String, explanation: String?, steps: [TurnPlanStep]) {
        self.turnId = turnId
        self.explanation = explanation
        self.steps = steps
    }
}
```

**TypeScript**

```ts
export type TurnPlan = {
  turnId: string;
  explanation: string | null;
  steps: TurnPlanStep[];
};
```

**Rust**

_N/A_


---

## RateLimitWindow

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `usedPercent` | `number` | no |  |
| `windowDurationMins` | `number \| null` | no |  |
| `resetsAt` | `number \| null` | no |  |


**Swift**

```swift
public struct RateLimitWindow: Codable, Hashable, Sendable {
    public var usedPercent: Double
    public var windowDurationMins: Double?
    public var resetsAt: Double?
}
```

**TypeScript**

```ts
export type RateLimitWindow = {
  usedPercent: number;
  windowDurationMins: number | null;
  resetsAt: number | null;
};
```

**Rust**

_N/A_


---

## CreditsSnapshot

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `hasCredits` | `boolean` | no |  |
| `unlimited` | `boolean` | no |  |
| `balance` | `string \| null` | no |  |


**Swift**

```swift
public struct CreditsSnapshot: Codable, Hashable, Sendable {
    public var hasCredits: Bool
    public var unlimited: Bool
    public var balance: String?
}
```

**TypeScript**

```ts
export type CreditsSnapshot = {
  hasCredits: boolean;
  unlimited: boolean;
  balance: string | null;
};
```

**Rust**

_N/A_


---

## RateLimitSnapshot

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `primary` | `RateLimitWindow \| null` | no |  |
| `secondary` | `RateLimitWindow \| null` | no |  |
| `credits` | `CreditsSnapshot \| null` | no |  |
| `planType` | `string \| null` | no |  |


**Swift**

```swift
public struct RateLimitSnapshot: Codable, Hashable, Sendable {
    public var primary: RateLimitWindow?
    public var secondary: RateLimitWindow?
    public var credits: CreditsSnapshot?
    public var planType: String?
}
```

**TypeScript**

```ts
export type RateLimitSnapshot = {
  primary: RateLimitWindow | null;
  secondary: RateLimitWindow | null;
  credits: CreditsSnapshot | null;
  planType: string | null;
};
```

**Rust**

_N/A_


---

## QueuedMessage

**Used in:** iOS ❌, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `string` | no | Unique identifier. |
| `text` | `string` | no |  |
| `createdAt` | `number` | no | Creation timestamp. |
| `images` | `string[]` | yes |  |


**Swift**

_N/A_


**TypeScript**

```ts
export type QueuedMessage = {
  id: string;
  text: string;
  createdAt: number;
  images?: string[];
};
```

**Rust**

_N/A_


---

## ModelOption

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `string` | no | Unique identifier. |
| `model` | `string` | no |  |
| `displayName` | `string` | no |  |
| `description` | `string` | no | Human-readable description. |
| `defaultReasoningEffort` | `string` | no |  |
| `isDefault` | `boolean` | no |  |


**Swift**

```swift
public struct ModelOption: Codable, Hashable, Sendable {
    public var id: String
    public var model: String
    public var displayName: String
    public var description: String
    public var supportedReasoningEfforts: [ReasoningEffortOption]
    public var defaultReasoningEffort: String
    public var isDefault: Bool
}
```

**TypeScript**

```ts
export type ModelOption = {
  id: string;
  model: string;
  displayName: string;
  description: string;
  supportedReasoningEfforts: { reasoningEffort: string; description: string }[];
  defaultReasoningEffort: string;
  isDefault: boolean;
};
```

**Rust**

_N/A_


---

## CollaborationModeOption

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `string` | no | Unique identifier. |
| `label` | `string` | no |  |
| `mode` | `string` | no |  |
| `model` | `string` | no |  |
| `reasoningEffort` | `string \| null` | no |  |
| `developerInstructions` | `string \| null` | no |  |
| `value` | `Record<string, unknown>` | no |  |


**Swift**

```swift
public struct CollaborationModeOption: Codable, Hashable, Sendable {
    public var id: String
    public var label: String
    public var mode: String
    public var model: String
    public var reasoningEffort: String?
    public var developerInstructions: String?
    public var value: [String: JSONValue]
}
```

**TypeScript**

```ts
export type CollaborationModeOption = {
  id: string;
  label: string;
  mode: string;
  model: string;
  reasoningEffort: string | null;
  developerInstructions: string | null;
  value: Record<string, unknown>;
};
```

**Rust**

_N/A_


---

## SkillOption

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `name` | `string` | no | Human-friendly name. |
| `path` | `string` | no | Filesystem path (absolute unless documented otherwise). |
| `description` | `string` | yes | Human-readable description. |


**Swift**

```swift
public struct SkillOption: Codable, Hashable, Sendable {
    public var name: String
    public var path: String
    public var description: String?
}
```

**TypeScript**

```ts
export type SkillOption = {
  name: string;
  path: string;
  description?: string;
};
```

**Rust**

_N/A_


---

## CustomPromptOption

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `name` | `string` | no | Human-friendly name. |
| `path` | `string` | no | Filesystem path (absolute unless documented otherwise). |
| `description` | `string` | yes | Human-readable description. |
| `argumentHint` | `string` | yes |  |
| `content` | `string` | no | Content payload (often text). |
| `scope` | `"workspace" \| "global"` | yes | Prompt scope (global/workspace). |


**Swift**

```swift
public struct CustomPromptOption: Codable, Hashable, Sendable {
    public var name: String
    public var path: String
    public var description: String?
    public var argumentHint: String?
    public var content: String
    public var scope: PromptScope?

    enum CodingKeys: String, CodingKey {
        case name, path, description, content, scope
        case argumentHint = "argumentHint"
    }
}
```

**TypeScript**

```ts
export type CustomPromptOption = {
  name: string;
  path: string;
  description?: string;
  argumentHint?: string;
  content: string;
  scope?: "workspace" | "global";
};
```

**Rust**

_N/A_


---

## BranchInfo

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `name` | `string` | no | Human-friendly name. |
| `lastCommit` | `number` | no |  |


**Swift**

```swift
public struct BranchInfo: Codable, Hashable, Sendable {
    public var name: String
    public var lastCommit: Double
}
```

**TypeScript**

```ts
export type BranchInfo = {
  name: string;
  lastCommit: number;
};
```

**Rust**

```rust
pub(crate) struct BranchInfo {
    pub(crate) name: String,
    pub(crate) last_commit: i64,
}
```

---

## DebugEntry

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `string` | no | Unique identifier. |
| `timestamp` | `number` | no |  |
| `source` | `"client" \| "server" \| "event" \| "stderr" \| "error"` | no |  |
| `label` | `string` | no |  |
| `payload` | `unknown` | yes |  |


**Swift**

_N/A_


**TypeScript**

```ts
export type DebugEntry = {
  id: string;
  timestamp: number;
  source: "client" | "server" | "event" | "stderr" | "error";
  label: string;
  payload?: unknown;
};
```

**Rust**

_N/A_


---

## TerminalStatus

**Used in:** iOS ❌, Desktop ✅, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

_N/A_


**TypeScript**

```ts
export type TerminalStatus = "idle" | "connecting" | "ready" | "error";
```

**Rust**

_N/A_


---

## DictationModelState

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum DictationModelState: String, Codable, Sendable {
    case missing
    case downloading
    case ready
    case error
}
```

**TypeScript**

```ts
export type DictationModelState = "missing" | "downloading" | "ready" | "error";
```

**Rust**

_N/A_


---

## DictationDownloadProgress

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `totalBytes` | `number \| null` | yes |  |
| `downloadedBytes` | `number` | no |  |


**Swift**

```swift
public struct DictationDownloadProgress: Codable, Hashable, Sendable {
    public var totalBytes: Double?
    public var downloadedBytes: Double
}
```

**TypeScript**

```ts
export type DictationDownloadProgress = {
  totalBytes?: number | null;
  downloadedBytes: number;
};
```

**Rust**

_N/A_


---

## DictationModelStatus

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `state` | `DictationModelState` | no | State enum. |
| `modelId` | `string` | no |  |
| `progress` | `DictationDownloadProgress \| null` | yes |  |
| `error` | `string \| null` | yes |  |
| `path` | `string \| null` | yes | Filesystem path (absolute unless documented otherwise). |


**Swift**

```swift
public struct DictationModelStatus: Codable, Hashable, Sendable {
    public var state: DictationModelState
    public var modelId: String
    public var progress: DictationDownloadProgress?
    public var error: String?
    public var path: String?
}
```

**TypeScript**

```ts
export type DictationModelStatus = {
  state: DictationModelState;
  modelId: string;
  progress?: DictationDownloadProgress | null;
  error?: string | null;
  path?: string | null;
};
```

**Rust**

_N/A_


---

## DictationSessionState

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum DictationSessionState: String, Codable, Sendable {
    case idle
    case listening
    case processing
}
```

**TypeScript**

```ts
export type DictationSessionState = "idle" | "listening" | "processing";
```

**Rust**

_N/A_


---

## DictationEvent

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `type` | `String` | no |  |
| `state` | `DictationSessionState?` | yes | State enum. |
| `value` | `Double?` | yes |  |
| `text` | `String?` | yes |  |
| `message` | `String?` | yes | Message payload (often arbitrary JSON). |


**Swift**

```swift
public struct DictationEvent: Codable, Hashable, Sendable {
    public var type: String
    public var state: DictationSessionState?
    public var value: Double?
    public var text: String?
    public var message: String?
}
```

**TypeScript**

```ts
export type DictationEvent = | { type: "state"; state: DictationSessionState }
  | { type: "level"; value: number }
  | { type: "transcript"; text: string }
  | { type: "error"; message: string }
  | { type: "canceled"; message: string };
```

**Rust**

_N/A_


---

## DictationTranscript

**Used in:** iOS ❌, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `string` | no | Unique identifier. |
| `text` | `string` | no |  |


**Swift**

_N/A_


**TypeScript**

```ts
export type DictationTranscript = {
  id: string;
  text: string;
};
```

**Rust**

_N/A_


---

## TerminalOutputEvent

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `workspaceId` | `String` | no | Workspace identifier. |
| `terminalId` | `String` | no |  |
| `data` | `String` | no | Opaque data blob (often UTF-8 text). |


**Swift**

```swift
public struct TerminalOutputEvent: Codable, Sendable {
    public let workspaceId: String
    public let terminalId: String
    public let data: String
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## MessageRole

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum MessageRole: String, Codable, Sendable {
    case user
    case assistant
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## ConversationItemKind

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum ConversationItemKind: String, Codable, Sendable {
    case message
    case reasoning
    case diff
    case review
    case tool
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## ReviewState

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum ReviewState: String, Codable, Sendable {
    case started
    case completed
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## ToolChange

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `path` | `String` | no | Filesystem path (absolute unless documented otherwise). |
| `kind` | `String?` | yes |  |
| `diff` | `String?` | yes |  |


**Swift**

```swift
public struct ToolChange: Codable, Hashable, Sendable {
    public var path: String
    public var kind: String?
    public var diff: String?
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## ThreadTurn

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `String?` | yes | Unique identifier. |
| `items` | `[JSONValue]?` | yes |  |
| `error` | `JSONValue?` | yes |  |


**Swift**

```swift
public struct ThreadTurn: Codable, Hashable, Sendable {
    public var id: String?
    public var items: [JSONValue]?
    public var error: JSONValue?

    public init(id: String? = nil, items: [JSONValue]? = nil, error: JSONValue? = nil) {
        self.id = id
        self.items = items
        self.error = error
    }
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## ThreadRecord

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `String` | no | Unique identifier. |
| `name` | `String?` | yes | Human-friendly name. |
| `title` | `String?` | yes | Title string. |
| `preview` | `String?` | yes |  |
| `cwd` | `String?` | yes | Current working directory for Codex / shells. |
| `createdAt` | `Double?` | yes | Creation timestamp. |
| `updatedAt` | `Double?` | yes | Last-updated timestamp. |
| `created_at` | `Double?` | yes |  |
| `updated_at` | `Double?` | yes |  |
| `turns` | `[ThreadTurn]?` | yes |  |


**Swift**

```swift
public struct ThreadRecord: Codable, Hashable, Sendable, Identifiable {
    public var id: String
    public var name: String?
    public var title: String?
    public var preview: String?
    public var cwd: String?
    public var createdAt: Double?
    public var updatedAt: Double?
    public var created_at: Double?
    public var updated_at: Double?
    public var turns: [ThreadTurn]?
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## ThreadListResponse

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `data` | `[ThreadRecord]` | no | Opaque data blob (often UTF-8 text). |
| `nextCursor` | `String?` | yes |  |
| `next_cursor` | `String?` | yes |  |


**Swift**

```swift
public struct ThreadListResponse: Codable, Sendable {
    public var data: [ThreadRecord]
    public var nextCursor: String?
    public var next_cursor: String?
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## ThreadStartResponse

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `result` | `Result?` | yes |  |
| `thread` | `ThreadRecord?` | yes |  |


**Swift**

```swift
public struct ThreadStartResponse: Codable, Sendable {
    public struct Result: Codable, Sendable {
        public var thread: ThreadRecord?
    }
    public var result: Result?
    public var thread: ThreadRecord?
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## ThreadResumeResponse

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `result` | `Result?` | yes |  |
| `thread` | `ThreadRecord?` | yes |  |


**Swift**

```swift
public struct ThreadResumeResponse: Codable, Sendable {
    public struct Result: Codable, Sendable {
        public var thread: ThreadRecord?
    }
    public var result: Result?
    public var thread: ThreadRecord?
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## ApprovalDecision

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum ApprovalDecision: String, Codable, Sendable {
    case accept
    case decline
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## ReviewDelivery

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum ReviewDelivery: String, Codable, Sendable {
    case inline
    case detached
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## PingResponse

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `ok` | `Bool` | no | Operation success flag. |


**Swift**

```swift
public struct PingResponse: Codable, Sendable {
    public var ok: Bool
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## GitStatusResponse

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `branchName` | `String` | no |  |
| `files` | `[GitFileStatus]` | no | List of file statuses. |
| `stagedFiles` | `[GitFileStatus]` | no | Files currently staged for commit. |
| `unstagedFiles` | `[GitFileStatus]` | no | Files with unstaged changes. |
| `totalAdditions` | `Int` | no | Total added lines across diffs. |
| `totalDeletions` | `Int` | no | Total removed lines across diffs. |


**Swift**

```swift
public struct GitStatusResponse: Codable, Hashable, Sendable {
    public var branchName: String
    public var files: [GitFileStatus]
    public var stagedFiles: [GitFileStatus]
    public var unstagedFiles: [GitFileStatus]
    public var totalAdditions: Int
    public var totalDeletions: Int
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## ReasoningEffortOption

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `reasoningEffort` | `String` | no |  |
| `description` | `String` | no | Human-readable description. |


**Swift**

```swift
public struct ReasoningEffortOption: Codable, Hashable, Sendable {
    public var reasoningEffort: String
    public var description: String
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## PromptScope

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum PromptScope: String, Codable, Sendable {
    case workspace
    case global
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## TerminalSessionInfo

**Used in:** iOS ✅, Desktop ❌, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `String` | no | Unique identifier. |


**Swift**

```swift
public struct TerminalSessionInfo: Codable, Hashable, Sendable {
    public var id: String
}
```

**TypeScript**

_N/A_


**Rust**

```rust
pub(crate) struct TerminalSessionInfo {
    id: String,
}
```

---

## WorkspaceFileResponse

**Used in:** iOS ✅, Desktop ❌, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `content` | `String` | no | Content payload (often text). |
| `truncated` | `bool` | no |  |


**Swift**

```swift
public struct WorkspaceFileResponse: Codable, Hashable, Sendable {
    public var content: String
    public var truncated: Bool
}
```

**TypeScript**

_N/A_


**Rust**

```rust
pub(crate) struct WorkspaceFileResponse {
    content: String,
    truncated: bool,
}
```

---

## GitHubPullRequestAuthor

**Used in:** iOS ❌, Desktop ❌, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `login` | `String` | no |  |


**Swift**

_N/A_


**TypeScript**

_N/A_


**Rust**

```rust
pub(crate) struct GitHubPullRequestAuthor {
    pub(crate) login: String,
}
```

---

## WorkspaceEntry

**Used in:** iOS ❌, Desktop ❌, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `String` | no | Unique identifier. |
| `name` | `String` | no | Human-friendly name. |
| `path` | `String` | no | Filesystem path (absolute unless documented otherwise). |
| `codex_bin` | `Option<String>` | yes |  |
| `kind` | `WorkspaceKind` | no |  |
| `parent_id` | `Option<String>` | yes |  |
| `worktree` | `Option<WorktreeInfo>` | yes |  |
| `settings` | `WorkspaceSettings` | no |  |


**Swift**

_N/A_


**TypeScript**

_N/A_


**Rust**

```rust
pub(crate) struct WorkspaceEntry {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) codex_bin: Option<String>,
    #[serde(default)]
    pub(crate) kind: WorkspaceKind,
    #[serde(default, rename = "parentId")]
    pub(crate) parent_id: Option<String>,
    #[serde(default)]
    pub(crate) worktree: Option<WorktreeInfo>,
    #[serde(default)]
    pub(crate) settings: WorkspaceSettings,
}
```

---

## TerminalOutput

**Used in:** iOS ❌, Desktop ❌, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `workspaceId` | `String` | no | Workspace identifier. |
| `terminalId` | `String` | no |  |
| `data` | `String` | no | Opaque data blob (often UTF-8 text). |


**Swift**

_N/A_


**TypeScript**

_N/A_


**Rust**

```rust
pub(crate) struct TerminalOutput {
    #[serde(rename = "workspaceId")]
    pub(crate) workspace_id: String,
    #[serde(rename = "terminalId")]
    pub(crate) terminal_id: String,
    pub(crate) data: String,
}
```

**Notes**

- Wire keys are `workspaceId`, `terminalId`, `data` (camelCase). Rust backend uses `terminal_id` field with `#[serde(rename="terminalId")]` etc.



---

## TerminalSession

**Used in:** iOS ❌, Desktop ❌, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `id` | `String` | no | Unique identifier. |
| `master` | `Mutex<Box<dyn portable_pty::MasterPty + Send>>` | no |  |
| `writer` | `Mutex<Box<dyn Write + Send>>` | no |  |
| `child` | `Mutex<Box<dyn portable_pty::Child + Send>>` | no |  |


**Swift**

_N/A_


**TypeScript**

_N/A_


**Rust**

```rust
pub(crate) struct TerminalSession {
    pub(crate) id: String,
    pub(crate) master: Mutex<Box<dyn portable_pty::MasterPty + Send>>,
    pub(crate) writer: Mutex<Box<dyn Write + Send>>,
    pub(crate) child: Mutex<Box<dyn portable_pty::Child + Send>>,
}
```

---

## JSONValue

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum JSONValue: Codable, Equatable, Hashable, Sendable {
    case string(String)
    case number(Double)
    case bool(Bool)
    case object([String: JSONValue])
    case array([JSONValue])
    case null

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if container.decodeNil() {
            self = .null
            return
        }
        if let value = try? container.decode(Bool.self) {
            self = .bool(value)
            return
        }
        if let value = try? container.decode(Double.self) {
            self = .number(value)
            return
        }
        if let value = try? container.decode(String.self) {
            self = .string(value)
            return
        }
        if let value = try? container.decode([String: JSONValue].self) {
            self = .object(value)
            return
        }
        if let value = try? container.decode([JSONValue].self) {
            self = .array(value)
            return
        }
        throw DecodingError.typeMismatch(
            JSONValue.self,
            DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Unsupported JSON value")
        )
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .string(let value):
            try container.encode(value)
        case .number(let value):
            try container.encode(value)
        case .bool(let value):
            try container.encode(value)
        case .object(let value):
            try container.encode(value)
        case .array(let value):
            try container.encode(value)
        case .null:
            try container.encodeNil()
        }
    }

    public var stringValue: String? {
        if case .string(let value) = self { return value }
        return nil
    }

    public var boolValue: Bool? {
        if case .bool(let value) = self { return value }
        return nil
    }

    public var numberValue: Double? {
        if case .number(let value) = self { return value }
        return nil
    }

    public var objectValue: [String: JSONValue]? {
        if case .object(let value) = self { return value }
        return nil
    }

    public var arrayValue: [JSONValue]? {
        if case .array(let value) = self { return value }
        return nil
    }

    public subscript(key: String) -> JSONValue? {
        guard case .object(let dict) = self else { return nil }
        return dict[key]
    }

    public func asString() -> String {
        switch self {
        case .string(let value):
            return value
        case .number(let value):
            if value.rounded(.towardZero) == value {
                return String(Int(value))
            }
            return String(value)
        case .bool(let value):
            return value ? "true" : "false"
        case .null:
            return ""
        case .array(let value):
            return value.map { $0.asString() }.joined(separator: " ")
        case .object:
            return ""
        }
    }

    public func asNumber() -> Double? {
        switch self {
        case .number(let value):
            return value
        case .string(let value):
            return Double(value)
        default:
            return nil
        }
    }

    public static func fromEncodable<T: Encodable>(_ value: T) throws -> JSONValue {
        let data = try JSONEncoder().encode(value)
        return try JSONDecoder().decode(JSONValue.self, from: data)
    }

    public func decode<T: Decodable>(_ type: T.Type, decoder: JSONDecoder = JSONDecoder()) throws -> T {
        let data = try JSONEncoder().encode(self)
        return try decoder.decode(T.self, from: data)
    }
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## RPCRequest

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public struct RPCRequest: Codable, Sendable {
    public let id: UInt64
    public let method: String
    public let params: JSONValue?

    public init(id: UInt64, method: String, params: JSONValue?) {
        self.id = id
        self.method = method
        self.params = params
    }
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## RPCResponse

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public struct RPCResponse: Codable, Sendable {
    public let id: UInt64
    public let result: JSONValue?
    public let error: RPCError?

    public init(id: UInt64, result: JSONValue?, error: RPCError?) {
        self.id = id
        self.result = result
        self.error = error
    }
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## RPCNotification

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public struct RPCNotification: Codable, Sendable {
    public let method: String
    public let params: JSONValue?

    public init(method: String, params: JSONValue?) {
        self.method = method
        self.params = params
    }
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

## RPCError

**Used in:** iOS ✅, Desktop ❌, Daemon ❌


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public struct RPCError: Codable, Error, Sendable {
    public let message: String

    public init(message: String) {
        self.message = message
    }
}
```

**TypeScript**

_N/A_


**Rust**

_N/A_


---

# Life Workspace Types

The following types support the Life workspace domain dashboard views.

---

## LifeDomain

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum LifeDomain: String, Codable, CaseIterable, Identifiable, Sendable {
    case delivery
    case nutrition
    case exercise
    case media
    case youtube
    case finance

    public var id: String { rawValue }
}
```

**TypeScript**

```ts
export type LifeDomain =
  | 'delivery'
  | 'nutrition'
  | 'exercise'
  | 'media'
  | 'youtube'
  | 'finance';
```

**Rust**

```rust
pub enum LifeDomain {
    Delivery,
    Nutrition,
    Exercise,
    Media,
    YouTube,
    Finance,
}
```

---

## DomainViewState

**Used in:** iOS ✅, Desktop ✅, Daemon ❌


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `activeDomain` | `LifeDomain \| null` | yes | Currently selected domain (null = show chat) |
| `timeRange` | `TimeRange` | no | Selected time range filter |
| `filters` | `Record<string, string>` | no | Domain-specific filters |
| `sortBy` | `string` | no | Sort field |
| `sortDirection` | `'asc' \| 'desc'` | no | Sort direction |


**Swift**

```swift
public struct DomainViewState: Codable, Sendable {
    public var activeDomain: LifeDomain?
    public var timeRange: TimeRange
    public var filters: [String: String]
    public var sortBy: String
    public var sortDirection: SortDirection

    public enum SortDirection: String, Codable, Sendable {
        case asc
        case desc
    }
}
```

**TypeScript**

```ts
export type DomainViewState = {
  activeDomain: LifeDomain | null;
  timeRange: 'today' | 'week' | 'month' | 'lifetime';
  filters: Record<string, string>;
  sortBy: string;
  sortDirection: 'asc' | 'desc';
};
```

**Rust**

_N/A_


---

## TimeRange

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

_Not a simple object type (union/enum/utility). See code snippets below._

**Swift**

```swift
public enum TimeRange: String, Codable, CaseIterable, Sendable {
    case today
    case week
    case month
    case lifetime
}
```

**TypeScript**

```ts
export type TimeRange = 'today' | 'week' | 'month' | 'lifetime';
```

**Rust**

```rust
pub enum TimeRange {
    Today,
    Week,
    Month,
    Lifetime,
}
```

---

## DeliveryDashboard

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `stats` | `DeliveryStats` | no | Aggregated delivery statistics |
| `orders` | `DeliveryOrder[]` | no | List of orders |
| `topMerchants` | `MerchantStats[]` | no | Top performing merchants |
| `timeRange` | `string` | no | Time range for this data |


**TypeScript**

```ts
export type DeliveryDashboard = {
  meta: DashboardMeta;
  stats: DeliveryStats;
  orders: DeliveryOrder[];
  topMerchants: MerchantStats[];
};

export type DeliveryOrder = {
  id: string;
  startedAt: string;
  merchantName: string;
  logoUrl?: string;
  payout: number;
  miles?: number;
  durationMinutes?: number;
  platform?: string;
  notes?: string;
  rewardTag?: string;
  warningTag?: string;
};

export type MerchantStats = {
  merchantName: string;
  logoUrl?: string;
  orderCount: number;
  totalEarnings: number;
  avgPayout: number;
  avgMiles?: number;
  avgPerMile?: number;
  tier?: string;
};
```

**Swift**

```swift
public struct DeliveryDashboard: Codable, Sendable {
    public var meta: DashboardMeta
    public var stats: DeliveryStats
    public var orders: [DeliveryOrder]
    public var topMerchants: [MerchantStats]
}

public struct DeliveryStats: Codable, Sendable {
    public var totalEarnings: Double
    public var orderCount: Int
    public var activeHours: Double
    public var totalMiles: Double?
    public var hourlyRate: Double
    public var perMileRate: Double
    public var avgOrderValue: Double?
    public var startingAr: Double?
    public var endingAr: Double?
    public var whaleCatches: Int?
}

public struct DeliveryOrder: Codable, Identifiable, Sendable {
    public var id: String
    public var startedAt: String
    public var merchantName: String
    public var logoUrl: String?
    public var payout: Double
    public var miles: Double?
    public var durationMinutes: Double?
    public var platform: String?
    public var notes: String?
    public var rewardTag: String?
    public var warningTag: String?
}

public struct MerchantStats: Codable, Sendable {
    public var merchantName: String
    public var logoUrl: String?
    public var orderCount: Int
    public var totalEarnings: Double
    public var avgPayout: Double
    public var avgMiles: Double?
    public var avgPerMile: Double?
    public var tier: String?
}
```

**Rust**

```rust
pub struct DeliveryDashboard {
    pub stats: DeliveryStats,
    pub orders: Vec<DeliveryOrder>,
    #[serde(rename = "topMerchants")]
    pub top_merchants: Vec<MerchantStats>,
    #[serde(rename = "timeRange")]
    pub time_range: String,
}
```

---

## NutritionDashboard

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `today` | `DailyNutrition` | no | Today's nutrition data |
| `targets` | `NutritionTargets` | no | Daily nutrition targets |
| `weeklyTrend` | `DailyNutrition[]` | no | Last 7 days of nutrition |


**TypeScript**

```ts
export type NutritionDashboard = {
  meta: DashboardMeta;
  stats: NutritionStats;
  meals: MealEntry[];
  weeklyTrend?: Record<string, number>;
};

export type NutritionStats = {
  calories: number;
  protein: number;
  carbs: number;
  fat: number;
  fiber?: number;
  mealCount: number;
};

export type MealEntry = {
  id: string;
  timestamp: string;
  mealType: string;
  description: string;
  foods: string[];
  imageUrl?: string;
  protein?: number;
  carbs?: number;
  fat?: number;
  fiber?: number;
  estimatedCalories?: number;
};
```

**Swift**

```swift
public struct NutritionDashboard: Codable, Sendable {
    public var meta: DashboardMeta
    public var stats: NutritionStats
    public var meals: [MealEntry]
    public var weeklyTrend: [String: Double]?
}

public struct NutritionStats: Codable, Sendable {
    public var calories: Double
    public var protein: Double
    public var carbs: Double
    public var fat: Double
    public var fiber: Double?
    public var mealCount: Int
}

public struct MealEntry: Codable, Identifiable, Sendable {
    public var id: String
    public var timestamp: String
    public var mealType: String
    public var description: String
    public var foods: [String]
    public var imageUrl: String?
    public var protein: Double?
    public var carbs: Double?
    public var fat: Double?
    public var fiber: Double?
    public var estimatedCalories: Double?
}
```

**Rust**

```rust
pub struct NutritionDashboard {
    pub today: DailyNutritionDetail,
    pub targets: NutritionTargets,
    #[serde(rename = "weeklyTrend")]
    pub weekly_trend: Vec<DailyNutrition>,
}
```

---

## MediaDashboard

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `recentlyWatched` | `MediaItem[]` | no | Recently consumed media |
| `stats` | `MediaStats` | no | Aggregated media statistics |
| `byType` | `Record<MediaType, number>` | no | Count by media type |


**TypeScript**

```ts
export type MediaDashboard = {
  recentlyWatched: MediaItem[];
  stats: {
    backlogCount: number;
    completedCount: number;
    avgRating: number;
  };
  byType: Record<MediaType, number>;
};

export type MediaType = 'film' | 'tv' | 'game' | 'anime' | 'book';

export type MediaItem = {
  id: string;
  title: string;
  type: MediaType;
  status: 'backlog' | 'watching' | 'completed' | 'dropped';
  rating?: number;
  coverUrl?: string;
  year?: number;
};
```

**Swift**

```swift
public struct MediaDashboard: Codable, Sendable {
    public var recentlyWatched: [MediaItem]
    public var stats: MediaStats
    public var byType: [String: Int]
}

public struct MediaStats: Codable, Sendable {
    public var backlogCount: Int
    public var completedCount: Int
    public var avgRating: Double
}

public struct MediaItem: Codable, Identifiable, Sendable {
    public var id: String
    public var title: String
    public var type: String
    public var status: String
    public var rating: Double?
    public var coverUrl: String?
    public var year: Int?
}
```

**Rust**

```rust
pub struct MediaDashboard {
    #[serde(rename = "recentlyWatched")]
    pub recently_watched: Vec<MediaItem>,
    pub stats: MediaStats,
    #[serde(rename = "byType")]
    pub by_type: HashMap<String, i32>,
}
```

---

## YouTubeDashboard

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `pipelineStats` | `Record<PipelineStage, number>` | no | Count by pipeline stage |
| `sTierIdeas` | `VideoIdea[]` | no | S-tier video ideas |
| `inProgress` | `VideoIdea[]` | no | Currently in-progress ideas |
| `recentActivity` | `PipelineEvent[]` | no | Recent pipeline activity |


**TypeScript**

```ts
export type YouTubeDashboard = {
  pipelineStats: Record<PipelineStage, number>;
  sTierIdeas: VideoIdea[];
  inProgress: VideoIdea[];
  recentActivity: PipelineEvent[];
};

export type PipelineStage =
  | 'brain_dump'
  | 'researching'
  | 'outlining'
  | 'scripting'
  | 'recording'
  | 'editing'
  | 'published'
  | 'archived';

export type VideoIdea = {
  id: string;
  title: string;
  tier: 'S' | 'A' | 'B' | 'C';
  stage: PipelineStage;
  thesis?: string;
  updatedAt: number;
};

export type PipelineEvent = {
  id: string;
  ideaId: string;
  ideaTitle: string;
  fromStage: PipelineStage;
  toStage: PipelineStage;
  timestamp: number;
};
```

**Swift**

```swift
public struct YouTubeDashboard: Codable, Sendable {
    public var pipelineStats: [String: Int]
    public var sTierIdeas: [VideoIdea]
    public var inProgress: [VideoIdea]
    public var recentActivity: [PipelineEvent]
}

public struct VideoIdea: Codable, Identifiable, Sendable {
    public var id: String
    public var title: String
    public var tier: String
    public var stage: String
    public var thesis: String?
    public var updatedAt: Double
}

public struct PipelineEvent: Codable, Identifiable, Sendable {
    public var id: String
    public var ideaId: String
    public var ideaTitle: String
    public var fromStage: String
    public var toStage: String
    public var timestamp: Double
}
```

**Rust**

```rust
pub struct YouTubeDashboard {
    #[serde(rename = "pipelineStats")]
    pub pipeline_stats: HashMap<String, i32>,
    #[serde(rename = "sTierIdeas")]
    pub s_tier_ideas: Vec<VideoIdea>,
    #[serde(rename = "inProgress")]
    pub in_progress: Vec<VideoIdea>,
    #[serde(rename = "recentActivity")]
    pub recent_activity: Vec<PipelineEvent>,
}
```

---

## FinanceDashboard

**Used in:** iOS ✅, Desktop ✅, Daemon ✅


**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `upcomingBills` | `Bill[]` | no | Bills due soon |
| `monthlyTotal` | `number` | no | Total monthly bills |
| `paidThisMonth` | `number` | no | Amount paid this month |
| `remainingThisMonth` | `number` | no | Amount remaining this month |
| `calendarView` | `BillCalendarDay[]` | no | Calendar view of bills |


**TypeScript**

```ts
export type FinanceDashboard = {
  meta: DashboardMeta;
  stats: FinanceStats;
  bills: Bill[];
  byCategory: Record<string, number>;
  statusMessage?: string;
};

export type FinanceStats = {
  monthlyTotal: number;
  dueSoonCount: number;
  autoPayCount: number;
};

export type Bill = {
  id: string;
  name: string;
  logoUrl?: string;
  amount: number;
  dueDay: number;
  frequency: string;
  category: string;
  autoPay: boolean;
  nextDueDate: string;
};
```

**Swift**

```swift
public struct FinanceDashboard: Codable, Sendable {
    public var meta: DashboardMeta
    public var stats: FinanceStats
    public var bills: [Bill]
    public var byCategory: [String: Double]
    public var statusMessage: String?
}

public struct FinanceStats: Codable, Sendable {
    public var monthlyTotal: Double
    public var dueSoonCount: Int
    public var autoPayCount: Int
}

public struct Bill: Codable, Identifiable, Sendable {
    public var id: String
    public var name: String
    public var logoUrl: String?
    public var amount: Double
    public var dueDay: Int
    public var frequency: String
    public var category: String
    public var autoPay: Bool
    public var nextDueDate: String
}
```

**Rust**

```rust
pub struct FinanceDashboard {
    pub meta: DashboardMeta,
    pub stats: FinanceStats,
    pub bills: Vec<Bill>,
    #[serde(rename = "byCategory")]
    pub by_category: std::collections::HashMap<String, f64>,
    #[serde(rename = "statusMessage", skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
}

pub struct FinanceStats {
    #[serde(rename = "monthlyTotal")]
    pub monthly_total: f64,
    #[serde(rename = "dueSoonCount")]
    pub due_soon_count: i32,
    #[serde(rename = "autoPayCount")]
    pub auto_pay_count: i32,
}

pub struct Bill {
    pub id: String,
    pub name: String,
    #[serde(rename = "logoUrl", skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    pub amount: f64,
    #[serde(rename = "dueDay")]
    pub due_day: i32,
    pub frequency: String,
    pub category: String,
    #[serde(rename = "autoPay")]
    pub auto_pay: bool,
    #[serde(rename = "nextDueDate")]
    pub next_due_date: String,
}
```

---

## TextFileResponse

**Used in:** Desktop ✅, Daemon ✅

Response type for global configuration file read operations.

**Definition (wire shape)**

| Field | Type | Optional | Description |
|---|---|---|---|
| `exists` | `boolean` | no | Whether the file exists on disk |
| `content` | `string` | no | File contents (empty if not exists) |
| `truncated` | `boolean` | no | Whether content was truncated |


**TypeScript**

```ts
export type TextFileResponse = {
  exists: boolean;
  content: string;
  truncated: boolean;
};
```

**Rust**

```rust
pub(crate) struct TextFileResponse {
    pub exists: bool,
    pub content: String,
    pub truncated: bool,
}
```

**Notes**

- Used by `read_global_agents_md` and `read_global_config_toml` RPC methods.
- `content` is empty string when `exists` is false.
- `truncated` is reserved for future large file handling.

---

## CoverOverrides

**Used in:** Desktop ✅, Daemon ✅

Manual cover image overrides for Media and YouTube dashboards.

**Definition (wire shape)**

A JSON object mapping entity IDs to cover image URLs.

```json
{
  "media-id-123": "https://example.com/custom-cover.jpg",
  "youtube-idea-456": "/local/path/to/cover.png"
}
```

**TypeScript**

```ts
export type CoverOverrides = Record<string, string>;
```

**File Location**

`Obsidian/Indexes/media.covers.overrides.json`

**Notes**

- Keys are entity IDs (media or YouTube idea IDs).
- Values are URLs (http/https) or local file paths.
- Overrides take precedence over API-fetched covers.

---
