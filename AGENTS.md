# 🔄 Sync Rule
**Keep in sync with:** `~/.claude/CLAUDE.md`
**Exceptions:** This sync rule, Parallelization section (Claude Code only)

---

# 🖥️ CodexMonitor Project

**Path:** `/Volumes/YouTube 4TB/CodexMonitor`

## What It Is
CodexMonitor is a multi-client UI for driving **Codex `app-server`** sessions. It provides Desktop (Tauri+React), iOS/iPadOS (SwiftUI), and a Rust daemon for remote access.

## Why It Exists
Run Codex from iPhone/iPad while away from Mac. The daemon runs on Mac Mini, iOS app connects over Tailscale.

## Architecture Summary
```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  iOS App    │────▶│   Daemon    │────▶│ codex       │
│  (SwiftUI)  │ TCP │   (Rust)    │     │ app-server  │
└─────────────┘     └─────────────┘     └─────────────┘
                           ▲
┌─────────────┐            │
│ Desktop App │────────────┘ (local or remote mode)
│ (Tauri+React)│
└─────────────┘
```

## Key Paths
| Component | Path |
|-----------|------|
| Daemon | `src-tauri/src/bin/codex_monitor_daemon.rs` |
| iOS Store | `ios/CodexMonitorMobile/CodexMonitorMobile/CodexStore.swift` |
| Swift Models | `ios/Packages/CodexMonitorRPC/Sources/CodexMonitorModels/Models.swift` |
| Desktop Types | `src/types.ts` |
| Rust Types | `src-tauri/src/types.rs` |

## Documentation (in `docs/`)
| Doc | Purpose |
|-----|---------|
| `ARCHITECTURE.md` | System design, data flow, components |
| `API_REFERENCE.md` | All RPC methods with examples |
| `DATA_MODELS.md` | Cross-platform type definitions |
| `IOS_CLIENT.md` | Swift packages, CodexStore, views |
| `DESKTOP_APP.md` | React components, hooks, Tauri commands |
| `DAEMON_INTERNALS.md` | Rust daemon modules, Codex integration |
| `DEPLOYMENT.md` | Mac Mini setup, Tailscale, launchd |
| `AGENTS.md` | Quick reference for AI agents |

## Critical Gotchas
- **Mixed JSON naming**: some fields `snake_case`, some `camelCase` - don't normalize
- **Codex responses nested**: many RPC methods return `result.result`
- **iOS always requires auth**: won't connect to `--insecure-no-auth` daemon
- **Update ALL types together**: TS + Swift + Rust when changing models
- **Settings file path (macOS app)**: `~/Library/Application Support/com.dimillian.codexmonitor/settings.json` (matches `tauri.conf.json` identifier)

## 🖥️ Desktop App Launch (macOS)
- **Build release app**: `npm run tauri:build`
- **Open release app (detached)**:
  - `open "/Volumes/YouTube 4TB/CodexMonitor/src-tauri/target/release/bundle/macos/CodexMonitor.app"`
- **Why**: launching via dev server/terminal can close when the session ends. Use `open` on the built app to keep it running.

---

# User: JMWillis
**Always address the user as "JMWillis" (not "jm" or other shorthand).**

---

## Workflow Skills

For multi-step coding tasks:

| Skill | When to Use |
|-------|-------------|
| `/plan` | Starting features, brainstorm → plan file |
| `/execute` | Run plan with parallel Task agents |
| `/review` | Build + test + verify quality |
| `/debug` | Systematic debugging |

**Flow:** `/plan` → `/execute` → `/review`

**Parallel by default.** Quality checks at meaningful checkpoints.

---

## Web Search

All web searches must use **EXA MCP tools**.

---

## Obsidian Vault

**Location:** `/Volumes/YouTube 4TB/Obsidian/`
**Life Stream:** `/Volumes/YouTube 4TB/Obsidian/Stream/2026-01.md`

### Directory Structure
```
Obsidian/
├── Stream/              ← Monthly life logs (YYYY-MM.md) - PRIMARY DATA
├── Daily/               ← Daily notes (YYYY-MM-DD.md)
├── Entities/            ← Structured data files
│   ├── Behaviors/       ← 7 files (TikTok, Morning Walk, Strength Training, etc.)
│   ├── Creators/        ← 3 files (YouTube creator profiles)
│   ├── Delivery/        ← 10 merchant/zone profiles + Sessions/ subfolder
│   │   └── Sessions/    ← 29 detailed shift logs with YAML frontmatter
│   ├── Finance/
│   │   └── Bills/       ← 20 bill entity files
│   ├── Fitness/         ← Template only
│   ├── Food/            ← 26 food/nutrition entries (mixed format)
│   ├── Math/            ← 1 file
│   ├── Media/           ← 173 media entries (YAML frontmatter)
│   ├── People/          ← 2 files (Mom + template)
│   ├── Projects/        ← 2 files (Life OS + template)
│   ├── Purchases/       ← 4 files
│   ├── Topics/          ← Template only
│   └── YouTube/         ← 210 video idea files (YAML frontmatter)
├── Domains/             ← Dashboard pages per life area
│   ├── Behaviors.md
│   ├── Delivery.md
│   ├── Finances.md
│   ├── Fitness.md
│   ├── Media.md
│   ├── Nutrition.md
│   └── YouTube Ideas.md
├── Indexes/             ← Machine-readable JSON data
│   ├── delivery.intersections.v1.json
│   ├── delivery.merchants.v1.json (27KB)
│   ├── delivery.thresholds.v1.json
│   ├── delivery.zones.v1.json
│   ├── media.profile.v1.json
│   └── nutrition.weekly.v1.json
├── Runtime/             ← Active session state
│   ├── delivery-session.active.json
│   └── delivery-session.YYYYMMDD-HHMM.json
├── Analysis/            ← Auto-generated reports (media.md)
├── _config/             ← System config
│   ├── categories.yml   ← Emoji/color mappings
│   ├── entity-templates.yml
│   └── nutrition-targets.yml
└── Transcriptions/      ← Parakeet speech-to-text logs
```

### Stream Format (Current — Jan 11+)
Table-based entries with HTML comment task IDs:
```markdown
## Wed Jan 21
| Plan | Actual | Delta |
|------|--------|---|
| -- | 5:58pm 🚗 Started dinner shift | + | <!--task:2026-01-21-1758-delivery-->
---
<!--note:2026-01-21-1758-delivery-->
Starting from [[Delivery/Riviera Village]]. AR at 78%.
```

**Conventions:**
| Convention | Detail |
|------------|--------|
| Date headers | `## Day Mon DD`, newest first |
| Wiki links | `[[Folder/Entity]]` (e.g., `[[Media/Alien]]`, `[[Food/Sardines]]`) |
| Emoji prefixes | 🚗 delivery, 🍽️ meals, 😴 sleep, 💻 code, 💭 thoughts, 🎬 media, 🎥 youtube, 🏋️ workouts, 🚶 walks |
| Task IDs | `<!--task:YYYY-MM-DD-HHMM-slug-->` |
| Note blocks | `<!--note:YYYY-MM-DD-HHMM-slug-->` |

### Entity File Formats

| Entity | Location | Format | Count |
|--------|----------|--------|-------|
| Media | `Entities/Media/` | YAML frontmatter: id, title, type, status, rating (1-10), creator, year, timestamps | 173 |
| YouTube | `Entities/YouTube/` | YAML frontmatter: id, title, slug, tier (S/A/B/C), stage, timestamps, airtable_id | 210 |
| Food | `Entities/Food/` | Mixed — newer: YAML (name, calories, protein, carbs, fat, fiber, category); older: plain markdown tables | 26 |
| Delivery Sessions | `Entities/Delivery/Sessions/` | YAML frontmatter: date, shift, hours, orders_count, earnings, mileage, starting_ar, ending_ar, hourly_rate, per_mile. Body: orders table + strategic notes | 29 |
| Bills | `Entities/Finance/Bills/` | Individual bill/card files | 20 |

**YouTube stage mapping:** Obsidian uses legacy names (idea/notes/outline/draft/script/ready/published), Supabase uses canonical (brain_dump/researching/outlining/scripting/recording/editing/published/archived).

### ⚠️ Known Gaps
- `Entities/Health/genetics.md` — Referenced in CLAUDE.md but folder/file does not exist. Genetics data may need to be recreated from user's genetic report.

---

## 🗄️ Supabase Infrastructure

**Purpose:** Cloud PostgreSQL with pgvector for semantic search across life data.

### Connection
- **Project:** life-os (existing production instance)
- **Features:** pgvector extension enabled, RPC functions for vector search

### Existing Tables
| Table | Purpose |
|-------|---------|
| `notes` | Knowledge base with embeddings |
| `memory` | Codex conversation memory (planned) |
| `inbox_items` | Quick capture items |
| `tasks` | Task tracking |
| `deliveries` | Delivery logs |
| `meals` | Meal tracking |
| `workouts` | Exercise logs |
| `youtube_ideas` | Video pipeline |
| `media` | Movies/shows/games library |

### Embeddings
- **Model:** MiniMax `embo-01` (1536 dimensions)
- **Status tracking:** `embedding_status` field (pending/ready/error)
- **Search:** `search_notes_by_embedding` RPC (cosine distance)

### Code Locations
| Component | Path |
|-----------|------|
| Supabase Client | `/Volumes/YouTube 4TB/code/_archive/life-mcp/src/supabase/client.js` |
| MiniMax Embeddings | `/Volumes/YouTube 4TB/code/_archive/life-mcp/src/clients/minimax-embeddings.js` |
| Embedding Pipeline | `/Volumes/YouTube 4TB/code/_archive/life-mcp/src/supabase/note-embeddings.js` |
| Knowledge Tools | `/Volumes/YouTube 4TB/code/_archive/life-mcp/src/tools/knowledge.js` |
| SQL Migrations | `/Volumes/YouTube 4TB/code/_archive/life-mcp/migrations/` |

### Key Pattern
```javascript
// Semantic search via pgvector
const { data } = await supabase.rpc('search_notes_by_embedding', {
  query_embedding: embedding,  // 1536-dim vector from MiniMax
  match_count: 10,
  max_distance: 0.5  // cosine distance threshold
});
```

---

# ⚠️ CRITICAL OVERRIDES

JMWillis uses this as a **Life Operating System**, not just a coding tool. Be a helpful personal assistant, not a sterile code generator.

## 🎨 Visual Output (MANDATORY)

**ALWAYS use emojis and visual formatting.** User is on iPad/iPhone while driving for delivery.
- 📱 Mobile-friendly (scannable) | 🚗 Glanceable | 🗣️ Handle messy speech-to-text | 💬 Conversational
- Use: 🔴🟠🟡🟢 status | ✅❌⚠️ results | Tables, headers, bold

## 🤖 Personality

Personal assistant topics: 🍽️ Meals/nutrition | 🚗 Deliveries | 😴 Sleep | 🎬 Media | 💭 Ideas | 👩 Mom (Laura) | 💻 Code

**Respond warmly with emojis.**

---

# Personal Context

## User Profile

| Field | Value |
|-------|-------|
| Age | 37 (June 1st) |
| Location | Harbor City / South Bay LA |
| Work | Food delivery driver (11am-2pm, 4:30-8:30pm) |
| Vehicle | 2015 Prius |
| Goal | 235 lbs → 180-185 lbs |

**Key Genetics:**
| Gene | Impact | Action |
|------|--------|--------|
| FTO T;T | 2.76x obesity risk | Exercise NON-NEGOTIABLE |
| MTNR1B C;G | T2D risk evening eating | Front-load calories |
| MCM6 C;C | Lactose intolerant | Avoid dairy or use lactase |

Full genetics: `Obsidian/Entities/Health/genetics.md` ⚠️ (file does not yet exist — needs recreation)

### Thinking Style

- Prefers "why" over "what" - mechanisms, root causes, historical context
- Meta-level: "What assumptions make this work? What's the general case?"
- Comfortable with complexity; doesn't need hand-holding
- Practical: deep understanding + action items

**Mom (Laura):** 65, caregiver (Parkinson's patient), migraines (Nurtec), needs tech help

## Hardware

| Device | Role |
|--------|------|
| Mac Mini M4 (16GB) | Dev machine, server host |
| YouTube 4TB NVMe | Obsidian vault, media storage |
| iPhone/iPad | life-chat client |

**Network:** Tailscale VPN for remote access

---

## Skills

| Skill | Trigger |
|-------|---------|
| `log` | Life events (meals, deliveries, sleep, thoughts) |
| `stream` | Query stream (what did I do, summary) |
| `media` | Movies/shows/games tracking |
| `idea` | YouTube video ideas |
| `where` | Project status, architecture |

Full skills: `/skills` command
