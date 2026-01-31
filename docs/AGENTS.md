# CodexMonitor Agent Reference

> Quick reference for AI coding assistants working on this codebase.

## Architecture (1-paragraph summary)

CodexMonitor is a multi-client UI (Desktop Tauri+React, iOS SwiftUI, and an optional remote Rust daemon) for driving **Codex `app-server`** sessions. The core backend concept is **one `codex app-server` process per workspace**, with UI state driven by streamed JSON notifications from Codex. The desktop app can run the backend locally, or proxy all calls to a daemon over TCP. iOS is a thin remote client that always talks to the daemon over newline-delimited JSON "RPC" with a shared token.

## Two-Workspace Model

CodexMonitor uses a **simplified two-workspace architecture**:

| Workspace | Purpose | Features |
|-----------|---------|----------|
| **CodexMonitor** | Coding workspace | Standard Codex, git operations, terminal |
| **Life** | Unified life assistant | All 6 domains, combined prompt injection, domain dashboards |

### Life Workspace Domains

| Domain | Data Source | Dashboard Features |
|--------|-------------|-------------------|
| Delivery | Obsidian Sessions + Supabase | Earnings, orders, $/hr, merchant analysis |
| Nutrition | Obsidian Stream + Food entities | Macros, meals, weekly trends |
| Exercise | Obsidian Behaviors | Workouts, streaks, progress |
| Media | Obsidian Entities/Media | Bookshelf covers, ratings, type filters |
| YouTube | Obsidian Entities/YouTube | Pipeline by tier, ideas, stages |
| Finance | Obsidian Entities/Finance/Bills | Due dates, calendar, monthly totals |

### Life Workspace Behavior

1. **Chat Mode (default)**: Full life context (4 prompts combined) injected into every thread
2. **Dashboard Mode**: Clicking domain tab replaces center panel with rendered dashboard
3. **Domain prompts**: `workspace-delivery-finance.md`, `workspace-food-exercise.md`, `workspace-media.md`, `workspace-youtube.md`
4. **Poster selection rules**: TMDB `/images` posters are selected by language rules (anime prefers English except Cowboy Bebop; foreign films prefer origin-country language). If ambiguous, prompt the user before choosing.

### Data Flow

```
Obsidian (raw) → Daemon parses → Dashboard data
                      ↓
             Supabase (aggregations) → Week/Month/Lifetime stats
```

### Implementation Status

| Phase | Status | Description |
|-------|--------|-------------|
| **Phase 0** | ✅ Complete | `WorkspacePurpose` enum, `get_life_workspace_prompt`, thread-level injection, skip per-turn injection |
| **Phase 1** | ✅ Complete | Delivery Dashboard (React + Rust + iOS) - parses Obsidian sessions, center panel swap |
| **Phase 2** | ✅ Complete | Media Bookshelf (React + Rust + iOS) - poster cards with covers, filters, grid/list view |
| **Phase 2b** | ✅ Complete | YouTube Bookshelf (React + Rust + iOS) - tier cards (S/A/B/C), filters, no time range |
| **Phase 3** | ✅ Complete | Nutrition, Exercise, Finance dashboards - stream parsing, weekly trends, streaks, bill due dates |
| **Phase 4** | ✅ Complete | UI Polish - clean poster cards (title below), scrolling fix, weekly bar charts, domain accent colors, manual cover overrides |
| **Upstream Merge** | ✅ Complete | Features from upstream CodexMonitor v0.7.30 (see below) |

**Debug logging**: Set `LIFE_DEBUG=1` to see Life workspace logging.

### Upstream Features Merged

| Feature | Priority | Files |
|---------|----------|-------|
| **Prompt History** | 🔴 HIGH | `usePromptHistory.ts`, `Composer.tsx` - Arrow up/down through 200 previous prompts |
| **Image Diff** | 🔴 HIGH | `ImageDiffCard.tsx`, `git.rs` - Side-by-side image comparisons in git diffs |
| **Global AGENTS.md Editor** | 🔴 HIGH | `useGlobalAgentsMd.ts`, `SettingsView.tsx`, `codex_home.rs` |
| **Global config.toml Editor** | 🔴 HIGH | `useGlobalCodexConfigToml.ts`, `SettingsView.tsx`, `codex_home.rs` |
| **Sidebar Search** | 🟡 MEDIUM | `Sidebar.tsx`, `sidebar.css` - Search across workspaces |
| **Recent Threads on Home** | 🟡 MEDIUM | `WorkspaceHome.tsx`, `workspaces.rs` |
| **CODEX_HOME per workspace** | 🟡 MEDIUM | `workspaces.rs`, `types.rs` |

## Key paths

| What | Path |
|------|------|
| Daemon entry | `src-tauri/src/bin/codex_monitor_daemon.rs` |
| Desktop Rust backend entry | `src-tauri/src/main.rs` → `codex_monitor_lib::run()` |
| Tauri command registry | `src-tauri/src/lib.rs` |
| React entry | `src/main.tsx` |
| React root | `src/App.tsx` |
| Desktop types | `src/types.ts` |
| Shared Rust types | `src-tauri/src/types.rs` |
| Shared Swift models | `ios/Packages/CodexMonitorRPC/Sources/CodexMonitorModels/Models.swift` |
| iOS RPC client | `ios/Packages/CodexMonitorRPC/Sources/CodexMonitorRPC/RPCClient.swift` |
| iOS app store | `ios/CodexMonitorMobile/CodexMonitorMobile/CodexStore.swift` |
| **Life workspace (React)** | `src/features/life/` |
| **Life workspace (iOS)** | `ios/.../Views/LifeWorkspaceView.swift` |
| **Life Rust commands** | `src-tauri/src/life.rs` |
| **Life prompt builder** | `src-tauri/src/life_core.rs` |
| **Domain prompts** | `workspace-*.md` files |
| **Prompt history hook** | `src/features/composer/hooks/usePromptHistory.ts` |
| **Image diff component** | `src/features/git/components/ImageDiffCard.tsx` |
| **Global config editors** | `src-tauri/src/files.rs` |
| **Codex args parser** | `src-tauri/src/codex_args.rs` |
| **File editor hook** | `src/features/shared/hooks/useFileEditor.ts` |
| **Workspace home** | `src/features/workspaces/components/WorkspaceHome.tsx` |
| **Cover overrides** | `Obsidian/Indexes/media.covers.overrides.json` |
| **Merchant logos** | `Obsidian/Indexes/merchant.logos.json` |
| **Food images** | `Obsidian/Indexes/food.images.json` |
| **Company logos** | `Obsidian/Indexes/company.logos.json` |

## RPC Quick Reference (daemon)

Full details: `docs/API_REFERENCE.md`

| Method | Purpose |
|---|---|
| `auth` | Authenticate a TCP connection (token) |
| `list_workspaces` | List known workspaces |
| `add_workspace` / `remove_workspace` | Manage workspaces |
| `connect_workspace` | Spawn/attach `codex app-server` for a workspace |
| `list_threads` | List Codex threads for a workspace |
| `start_thread` / `resume_thread` / `archive_thread` | Thread lifecycle |
| `send_user_message` | Start a Codex turn (`turn/start`) |
| `turn_interrupt` | Interrupt a running turn |
| `respond_to_server_request` | Reply to Codex request (approvals) |
| `remember_approval_rule` | Append allowed-command rule to CODEX_HOME rules |
| `get_git_status` / `get_git_diffs` / `commit_git` | Git operations |
| `terminal_open` / `terminal_write` | PTY terminal sessions |
| `prompts_list` / `prompts_create` | Prompt library CRUD |
| `read_global_agents_md` / `write_global_agents_md` | Global AGENTS.md file |
| `read_global_config_toml` / `write_global_config_toml` | Global config.toml file |
| notifications: `app-server-event` | Streamed Codex events |
| notifications: `terminal-output` | Terminal output stream |

## Common tasks

### Add a new daemon RPC method

1. Implement it in `src-tauri/src/bin/codex_monitor_daemon.rs` inside `handle_rpc_request`:
   - Parse params with existing `parse_*` helpers (or add one).
   - If it’s Codex-backed, call `session.send_request(...)` with an app-server method string.
2. Add/extend typed wrappers:
   - Desktop remote mode: `src-tauri/src/remote_backend.rs` (method string + params)
   - iOS: `ios/Packages/CodexMonitorRPC/Sources/CodexMonitorRPC/CodexMonitorAPI.swift`
3. Update docs:
   - `docs/API_REFERENCE.md` (method entry)
4. If the method adds/changes payload types:
   - update `src/types.ts`
   - update Swift `Models.swift`
   - update Rust `types.rs`
   - update `docs/DATA_MODELS.md`

### Add a new iOS view

1. Create the view in `ios/CodexMonitorMobile/CodexMonitorMobile/Views/`.
2. Add navigation entry in `RootView` / `PhoneRootView` / `TabletRootView`.
3. Bind state via `@EnvironmentObject var store: CodexStore`.
4. If the view needs new backend behavior, add RPC methods and typed wrappers.

### Modify a shared data model

1. Decide the *wire name* of each field (camelCase vs snake_case — existing code mixes both).
2. Update:
   - TypeScript: `src/types.ts`
   - Swift: `Models.swift` (+ `CodingKeys` if wire name differs)
   - Rust: `src-tauri/src/types.rs` (`#[serde(rename="...")]` if needed)
3. Confirm daemon serialization matches clients:
   - daemon RPC payloads in `handle_rpc_request`
4. Update `docs/DATA_MODELS.md`.

### Add a new Life workspace domain dashboard

1. Create the React component in `src/features/life/components/domains/`:
   - Follow pattern from existing dashboards (stats cards, time range, data table)
   - Use shared components from `src/features/life/components/shared/`
2. Create the data hook in `src/features/life/hooks/`:
   - Pattern: `useDomainData.ts` with `invoke('get_domain_dashboard', {...})`
3. Add Rust backend command:
   - Implement `get_[domain]_dashboard` in daemon
   - Parse Obsidian entities and/or query Supabase aggregations
4. Add to `DomainViewContainer.tsx` switch statement
5. Create iOS equivalent:
   - SwiftUI view in `Views/domains/`
   - CodexStore method for fetching
6. Add dashboard types to `docs/DATA_MODELS.md`

### Modify Life workspace prompt injection

1. Edit the relevant workspace prompt file (`workspace-*.md`)
2. Combined prompt is built in `get_life_workspace_prompt()` Rust command
3. Prompt files are concatenated with `---` separators
4. Test with thread/start to verify full injection

## Gotchas & landmines

- **Mixed JSON naming**: some payloads are snake_case (`codex_bin`, `workspace_id`), others camelCase (`parentId`, request params). Don't "normalize" without changing all clients.
- **Codex responses are nested**: many RPC methods return the raw app-server response envelope; clients often need `result.result`.
- **Auth vs insecure mode**: iOS always performs `auth` and will not connect to a daemon started with `--insecure-no-auth` (unless mobile is changed).
- **Terminal output is unbounded**: output streams can flood UI; trim buffers and consider backpressure if adding new streams.
- **Approval rules are security-sensitive**: `remember_approval_rule` changes CODEX_HOME rules; treat it like editing sudoers.
- **Life workspace prompts are large**: Combined 4-prompt injection can be 3000+ lines. Monitor token usage.
- **Domain dashboards replace center panel**: Not a modal/overlay. Similar to Git diff view pattern.
- **Two data sources for Life**: Obsidian for real-time/today, Supabase for aggregations.

---

## Supabase + Embeddings Infrastructure

> Existing infrastructure for semantic search - reuse for memory features.

### What exists
| Component | Location |
|-----------|----------|
| Supabase client | `/Volumes/YouTube 4TB/code/_archive/life-mcp/src/supabase/client.js` |
| MiniMax embeddings | `/Volumes/YouTube 4TB/code/_archive/life-mcp/src/clients/minimax-embeddings.js` |
| Embedding pipeline | `/Volumes/YouTube 4TB/code/_archive/life-mcp/src/supabase/note-embeddings.js` |
| Knowledge MCP tools | `/Volumes/YouTube 4TB/code/_archive/life-mcp/src/tools/knowledge.js` |
| SQL migrations | `/Volumes/YouTube 4TB/code/_archive/life-mcp/migrations/` |

### Supabase tables (existing)
`notes`, `inbox_items`, `tasks`, `deliveries`, `meals`, `workouts`, `youtube_ideas`, `media`

### Embedding pattern
```javascript
// 1. Generate embedding via MiniMax embo-01
const embedding = await minimax.createEmbedding(text);  // returns 1536-dim vector

// 2. Search via pgvector RPC
const { data } = await supabase.rpc('search_notes_by_embedding', {
  query_embedding: embedding,
  match_count: 10,
  max_distance: 0.5
});
```

### For memory integration
See `docs/PLAN-memory-integration.md` for the phased implementation plan. Key points:
- Add `memory` table to Supabase (not SQLite)
- Port MiniMax client to Rust
- Add memory RPC methods to daemon
- Add iOS Memory tab

---
