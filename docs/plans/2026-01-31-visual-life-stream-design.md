# Visual Life Stream Design Document

**Document Version:** 1.0
**Date:** January 31, 2026
**Author:** JMWillis + Claude
**Status:** Draft for Review

---

## 1. Executive Summary

The Visual Life Stream transforms CodexMonitor from a collection of separate domain dashboards into a **unified chronological feed of visual cards** representing the user's day. Instead of navigating between Delivery, Nutrition, Media, and other domain views, users see a continuous stream where each life event—meals, deliveries, media consumption, thoughts, code work—appears as a rich visual card with images, stats, and AI-enriched context. Input happens through a chat/voice interface at the bottom, with the AI auto-detecting intent and rendering appropriate card types. Cards appear immediately with live processing states, update in-place when complete, and support non-blocking parallel execution. This design prioritizes mobile-first glanceability while retaining full power on desktop, unifying the Life OS experience into a single, scrollable, visual timeline.

---

## 2. Goals & Non-Goals

### 2.1 Goals

| ID | Goal | Success Metric |
|----|------|----------------|
| G1 | **Unified visual timeline** — Replace fragmented domain dashboards with one chronological stream | Single view shows all life domains |
| G2 | **Cards, not chat bubbles** — Every interaction produces a visual card with image + enriched data | 100% of logged events render as cards |
| G3 | **Immediate feedback** — Cards appear instantly with live processing indicators | < 100ms from input to card appearance |
| G4 | **Non-blocking parallel processing** — Multiple inputs process simultaneously | Can send 5+ messages while others process |
| G5 | **Voice-first mobile** — Optimized for speech input while driving/delivering | Voice works on iOS without keyboard |
| G6 | **AI auto-intent detection** — No explicit commands needed, natural language flows | "had omelette" auto-routes to nutrition |
| G7 | **Rich media cards** — Album art, movie posters, food photos, merchant logos | 90%+ of cards display relevant images |
| G8 | **Expandable details** — Tap to see full history, all mentions, entity stats | Every card supports expansion |
| G9 | **Day picker + emoji filters** — Easy navigation across days and domains | Filter by day + multiple emoji toggles |

### 2.2 Non-Goals

| ID | Non-Goal | Rationale |
|----|----------|-----------|
| NG1 | Real-time collaborative editing | Single-user system |
| NG2 | Full chat history preservation | Cards replace traditional chat bubbles |
| NG3 | Domain-specific dashboard views | Unified stream supersedes separate views |
| NG4 | Complex query builders | Natural language replaces structured queries |
| NG5 | Offline-first architecture | Requires Codex connection for AI features |
| NG6 | Third-party integrations (Slack, etc.) | Focus on personal life logging |
| NG7 | Multi-model switching in stream | Use single Codex model per workspace |
| NG8 | Automated background logging | All entries user-initiated or confirmed |

---

## 3. User Experience

### 3.1 Primary View Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  ◀ Jan 30  │  Jan 31  │  Feb 1 ▶   (day picker, horizontal)    │
├─────────────────────────────────────────────────────────────────┤
│  🚗  🍽️  🎬  💭  🔧  📝  💤  🏋️  (emoji toggle filters)         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ 🍽️ 10:20am Hot n Tot                               ✓    │  │
│  │ ┌─────────┐ Spinach Omelette x2                         │  │
│  │ │ omelette│ 🔥 840cal 💪 56g protein                     │  │
│  │ │  photo  │ Day total: 840/2000                         │  │
│  │ │         │ Shared with: Mom                            │  │
│  │ └─────────┘ 💡 High protein start!                      │  │
│  │             [tap to expand]                              │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ 🚗 11:15am Started lunch shift                      ⏳   │  │
│  │ ┌─────────┐ → Analyzing zone traffic...                 │  │
│  │ │ 🚗 icon │ → Checking merchant patterns...             │  │
│  │ └─────────┘                                              │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ 🎵 9:45am Music                                     ✓    │  │
│  │ ┌─────────┐ Blonde — Frank Ocean                        │  │
│  │ │ album   │ Added to listening queue                    │  │
│  │ │ cover   │ 🎧 2 listens this week                      │  │
│  │ └─────────┘                                              │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ... (scrollable stream of cards) ...                           │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ 🎤                                                   📎  │  │
│  │ "ordered 2 omelettes for me and mom at 10:20am sat"      │  │
│  │                                              [Send ➤]    │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 User Flow: Logging a Meal

```
1. USER SPEAKS: "ordered 2 spinach omelettes with home fries and
                 sourdough bread, 1 for me and 1 for my mom at
                 10:20am on sat 1/31"

2. CARD APPEARS IMMEDIATELY (processing state):
   ┌─────────────────────────────────────────┐
   │ 🍽️ 10:20am                        ⏳    │
   │ → Parsing input...                      │
   │ → Detected: meal log                    │
   │ → Looking up nutrition...               │
   └─────────────────────────────────────────┘

3. AI PROCESSES IN BACKGROUND:
   - Intent detection: LOG (nutrition domain)
   - Entity extraction: spinach omelette, home fries, sourdough
   - Nutrition lookup: calories, protein, carbs, fat
   - Context enrichment: shared with Mom, daily totals
   - Image fetch: check local cache → API if missing

4. CARD UPDATES IN-PLACE (complete state):
   ┌─────────────────────────────────────────┐
   │ 🍽️ 10:20am Hot n Tot               ✓   │
   │ ┌─────────┐ Spinach Omelette x2         │
   │ │ omelette│ 🔥 840cal 💪 56g protein    │
   │ │  photo  │ Day total: 840/2000         │
   │ │ or 📷   │ Shared with: Mom            │
   │ └─────────┘ 💡 High protein start!      │
   │             [tap to expand]             │
   └─────────────────────────────────────────┘

5. TAP TO EXPAND (inline expansion):
   ┌─────────────────────────────────────────┐
   │ 🍽️ 10:20am Hot n Tot               ▼   │
   │ ┌─────────┐ Spinach Omelette x2         │
   │ │ omelette│                             │
   │ │  photo  │ NUTRITION BREAKDOWN         │
   │ │ or 📷   │ ├ Calories: 840 (42% daily) │
   │ └─────────┘ ├ Protein: 56g (112% daily) │
   │             ├ Carbs: 48g                │
   │             └ Fat: 32g                  │
   │                                         │
   │ MEAL DETAILS                            │
   │ ├ 2x Spinach omelette (280cal each)     │
   │ ├ 2x Home fries (120cal each)           │
   │ └ 2x Sourdough toast (40cal each)       │
   │                                         │
   │ CONTEXT                                 │
   │ ├ Restaurant: Hot n Tot (inferred)      │
   │ ├ Shared with: Mom                      │
   │ └ Original input: "ordered 2 spinach..."│
   │                                         │
   │ HISTORY (last 5 at Hot n Tot)           │
   │ ├ Jan 24: Breakfast burrito             │
   │ ├ Jan 18: Pancakes                      │
   │ └ Jan 11: Omelette                      │
   │                                         │
   │ [📷 Add Photo]  [✏️ Edit]  [🗑️ Delete] │
   └─────────────────────────────────────────┘
```

### 3.3 User Flow: Query

```
USER: "what songs did I listen to this week?"

1. PROCESSING CARD:
   ┌─────────────────────────────────────────┐
   │ 🎵 Query                           ⏳   │
   │ → Searching music logs...               │
   │ → Found 12 entries...                   │
   └─────────────────────────────────────────┘

2. RESULT CARD (collapsed):
   ┌─────────────────────────────────────────┐
   │ 🎵 This Week's Music               ✓    │
   │ ┌───┐┌───┐┌───┐┌───┐┌───┐              │
   │ │ 1 ││ 2 ││ 3 ││ 4 ││ 5 │ +7 more     │
   │ └───┘└───┘└───┘└───┘└───┘              │
   │ 12 tracks • 4.2 hours                   │
   │ Top: Frank Ocean (5), Radiohead (3)     │
   │             [tap to expand]             │
   └─────────────────────────────────────────┘

3. EXPANDED:
   ┌─────────────────────────────────────────┐
   │ 🎵 This Week's Music               ▼    │
   │                                         │
   │ FRIDAY                                  │
   │ ├ Blonde — Frank Ocean (3 plays)        │
   │ ├ Kid A — Radiohead                     │
   │ └ DAMN. — Kendrick Lamar                │
   │                                         │
   │ THURSDAY                                │
   │ ├ Channel Orange — Frank Ocean          │
   │ └ In Rainbows — Radiohead               │
   │ ... (full list)                         │
   └─────────────────────────────────────────┘
```

### 3.4 User Flow: Code Action

```
USER: "fix the login bug in CodexMonitor"

1. ACTION CARD (processing):
   ┌─────────────────────────────────────────┐
   │ 🔧 Code Task                       ⏳   │
   │ ┌─────────┐ Working on: login bug       │
   │ │   📁    │                             │
   │ │ Codex   │ → Reading auth module...    │
   │ │ Monitor │ → Found issue in session... │
   │ └─────────┘ → Applying fix...           │
   │                                         │
   │ ▸ Live reasoning (latest step)          │
   └─────────────────────────────────────────┘

2. COMPLETE CARD:
   ┌─────────────────────────────────────────┐
   │ 🔧 Fixed login bug                 ✓    │
   │ ┌─────────┐ CodexMonitor                │
   │ │   ✓     │                             │
   │ │  Done   │ Fixed session validation    │
   │ │         │ in auth.rs (3 files)        │
   │ └─────────┘                             │
   │ 📝 Summary: Token refresh was failing...│
   │             [tap to expand]             │
   └─────────────────────────────────────────┘

3. EXPANDED (full answer + reasoning toggle):
   ┌─────────────────────────────────────────┐
   │ 🔧 Fixed login bug                 ▼    │
   │                                         │
   │ SUMMARY                                 │
   │ The login bug was caused by expired     │
   │ session tokens not being refreshed...   │
   │                                         │
   │ FILES CHANGED                           │
   │ ├ src-tauri/src/auth.rs (+15, -3)       │
   │ ├ src-tauri/src/session.rs (+8, -2)     │
   │ └ src/hooks/useAuth.ts (+4, -1)         │
   │                                         │
   │ ▸ Show reasoning trace                  │
   │   └ [collapsed reasoning steps...]      │
   │                                         │
   │ ORIGINAL PROMPT                         │
   │ "fix the login bug in CodexMonitor"     │
   └─────────────────────────────────────────┘
```

---

## 4. Architecture Overview

### 4.1 System Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              USER LAYER                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌───────────────────┐              ┌───────────────────┐               │
│  │   iOS App         │              │   Desktop App     │               │
│  │   (SwiftUI)       │              │   (React/Tauri)   │               │
│  │                   │              │                   │               │
│  │ ┌───────────────┐ │              │ ┌───────────────┐ │               │
│  │ │ LifeStreamView│ │              │ │LifeStreamView │ │               │
│  │ │  - CardList   │ │              │ │  - CardList   │ │               │
│  │ │  - DayPicker  │ │              │ │  - DayPicker  │ │               │
│  │ │  - Filters    │ │              │ │  - Filters    │ │               │
│  │ │  - Composer   │ │              │ │  - Composer   │ │               │
│  │ └───────────────┘ │              │ └───────────────┘ │               │
│  │                   │              │                   │               │
│  │ ┌───────────────┐ │              │ ┌───────────────┐ │               │
│  │ │ CardRenderer  │ │              │ │ CardRenderer  │ │               │
│  │ │  - MealCard   │ │              │ │  - MealCard   │ │               │
│  │ │  - DelivCard  │ │              │ │  - DelivCard  │ │               │
│  │ │  - MediaCard  │ │              │ │  - MediaCard  │ │               │
│  │ │  - CodeCard   │ │              │ │  - CodeCard   │ │               │
│  │ │  - QueryCard  │ │              │ │  - QueryCard  │ │               │
│  │ └───────────────┘ │              │ └───────────────┘ │               │
│  └─────────┬─────────┘              └─────────┬─────────┘               │
│            │ TCP/JSON-RPC                     │ Tauri IPC                │
│            │ (Tailscale)                      │ (local or remote)        │
└────────────┼────────────────────────────────┼───────────────────────────┘
             │                                  │
             ▼                                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           DAEMON LAYER                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    CodexMonitor Daemon (Rust)                    │    │
│  │                                                                 │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │    │
│  │  │ RPC Server  │  │ Event       │  │ Stream Service          │  │    │
│  │  │ (TCP)       │  │ Broadcaster │  │  - CardStateManager     │  │    │
│  │  │             │  │             │  │  - BackgroundTaskQueue  │  │    │
│  │  └─────────────┘  └─────────────┘  │  - IntentRouter         │  │    │
│  │                                     └─────────────────────────┘  │    │
│  │                                                                 │    │
│  │  ┌─────────────────────────────────────────────────────────┐    │    │
│  │  │              Processing Pipeline                         │    │    │
│  │  │                                                         │    │    │
│  │  │  Input → Intent Detection → Entity Extraction →         │    │    │
│  │  │         Domain Router → Enrichment → Card Update        │    │    │
│  │  │                                                         │    │    │
│  │  └─────────────────────────────────────────────────────────┘    │    │
│  │                                                                 │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │    │
│  │  │ Codex       │  │ Image       │  │ Data Services           │  │    │
│  │  │ app-server  │  │ Resolver    │  │  - ObsidianReader       │  │    │
│  │  │ (subprocess)│  │ (cache+API) │  │  - SupabaseClient       │  │    │
│  │  └─────────────┘  └─────────────┘  │  - LifeMCPBridge        │  │    │
│  │                                     └─────────────────────────┘  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
             │                    │                    │
             ▼                    ▼                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                            DATA LAYER                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐    │
│  │ Obsidian Vault    │  │ Supabase          │  │ Image Cache       │    │
│  │                   │  │                   │  │                   │    │
│  │ ├─ Stream/        │  │ ├─ meals          │  │ ├─ media.covers   │    │
│  │ │  └─ 2026-01.md  │  │ ├─ deliveries     │  │ ├─ food.images    │    │
│  │ │                 │  │ ├─ media          │  │ ├─ merchant.logos │    │
│  │ ├─ Entities/      │  │ ├─ youtube_ideas  │  │ └─ album.art      │    │
│  │ │  ├─ Food/       │  │ ├─ notes          │  │                   │    │
│  │ │  ├─ Media/      │  │ ├─ memory         │  │ (JSON in Indexes/)│    │
│  │ │  ├─ Delivery/   │  │ │                 │  │                   │    │
│  │ │  └─ YouTube/    │  │ └─ aggregations   │  │                   │    │
│  │ │                 │  │                   │  │                   │    │
│  │ └─ Indexes/       │  │ (pgvector search) │  │                   │    │
│  │    (JSON caches)  │  │                   │  │                   │    │
│  └───────────────────┘  └───────────────────┘  └───────────────────┘    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         EXTERNAL SERVICES                                │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐                │
│  │ TMDB API      │  │ IGDB API      │  │ Nutritionix   │                │
│  │ (movie/TV)    │  │ (games)       │  │ (food lookup) │                │
│  └───────────────┘  └───────────────┘  └───────────────┘                │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐                │
│  │ Spotify/Apple │  │ Clearbit      │  │ MiniMax       │                │
│  │ (album art)   │  │ (company logo)│  │ (embeddings)  │                │
│  └───────────────┘  └───────────────┘  └───────────────┘                │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Component Responsibilities

| Component | Responsibility |
|-----------|----------------|
| **LifeStreamView** | Main UI container, renders card list, day picker, filters, composer |
| **CardRenderer** | Polymorphic card rendering based on card type and state |
| **CardStateManager** | Tracks all cards, their states (processing/complete/error), handles updates |
| **BackgroundTaskQueue** | Non-blocking task execution with parallel processing |
| **IntentRouter** | AI-powered intent detection, routes to correct domain handler |
| **ObsidianReader** | Reads Stream files, Entity files, parses markdown |
| **SupabaseClient** | Writes to cloud tables, semantic search, aggregations |
| **LifeMCPBridge** | Interfaces with life-mcp tools (advisor, nutrition, etc.) |
| **ImageResolver** | Multi-tier image resolution (cache → API → placeholder) |

---

## 5. Card System Design

### 5.1 Card Type Taxonomy

| Type | Emoji | Domain | Primary Image Source | Key Data |
|------|-------|--------|---------------------|----------|
| `meal` | 🍽️ | Nutrition | Food photo / restaurant logo | calories, macros, daily total |
| `delivery_shift` | 🚗 | Delivery | Zone map / car icon | start time, AR%, zone |
| `delivery_order` | 📦 | Delivery | Merchant logo | payout, mileage, $/mi |
| `media_add` | 🎬 | Media | Movie/show/game poster | title, type, status |
| `media_rate` | ⭐ | Media | Poster | rating, review snippet |
| `music` | 🎵 | Media | Album art | album, artist, play count |
| `youtube_idea` | 🎥 | YouTube | Thumbnail mock | title, tier, stage |
| `thought` | 💭 | Notes | Thought bubble icon | content snippet |
| `sleep` | 💤 | Health | Moon/bed icon | duration, quality |
| `workout` | 🏋️ | Fitness | Exercise icon | type, duration, sets |
| `walk` | 🚶 | Fitness | Map/route | distance, steps, duration |
| `code_task` | 🔧 | Code | Project icon / folder | task summary, files changed |
| `query_result` | 🔍 | Query | Grid of results | count, top items |
| `finance` | 💰 | Finance | Company logo | amount, category, due date |
| `generic` | 📝 | Any | Default icon | content |

### 5.2 Card States

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CARD STATE MACHINE                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│    ┌──────────┐      ┌──────────────┐      ┌───────────┐           │
│    │ PENDING  │─────▶│ PROCESSING   │─────▶│ COMPLETE  │           │
│    │          │      │              │      │           │           │
│    │ (queued) │      │ (live trace) │      │ (final)   │           │
│    └──────────┘      └──────┬───────┘      └───────────┘           │
│                             │                                       │
│                             │ error                                 │
│                             ▼                                       │
│                      ┌──────────────┐                               │
│                      │ ERROR        │                               │
│                      │              │                               │
│                      │ (retry btn)  │                               │
│                      └──────────────┘                               │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

| State | Visual Indicator | User Interaction |
|-------|------------------|------------------|
| `pending` | Faded card, queue position | Cancel |
| `processing` | Spinner, live trace lines | Watch progress |
| `complete` | Checkmark, full content | Expand, edit, delete |
| `error` | Red indicator, error message | Retry, dismiss |

### 5.3 Card Data Model

```typescript
interface StreamCard {
  id: string;                    // UUID
  timestamp: Date;               // When event occurred (user-specified or now)
  createdAt: Date;               // When card was created
  updatedAt: Date;               // Last state change

  // Type & Domain
  cardType: CardType;            // 'meal' | 'delivery_order' | 'media_add' | ...
  domain: Domain;                // 'nutrition' | 'delivery' | 'media' | ...
  emoji: string;                 // Visual emoji indicator

  // State
  state: CardState;              // 'pending' | 'processing' | 'complete' | 'error'
  processingStep?: string;       // Current step (for live trace)
  processingSteps?: string[];    // History of steps
  errorMessage?: string;         // If state === 'error'

  // Content
  title: string;                 // Card header
  subtitle?: string;             // Secondary info
  body?: string;                 // Main content (varies by type)
  tip?: string;                  // AI-generated insight

  // Image
  imageUrl?: string;             // Primary image
  imagePlaceholder?: string;     // Placeholder while loading / missing
  imageStatus: 'loading' | 'ready' | 'missing' | 'upload_prompt';

  // Stats (domain-specific)
  stats?: Record<string, string | number>;  // calories, $/mi, rating, etc.

  // Context
  entities?: EntityRef[];        // Referenced entities (food, media, merchant)
  originalInput?: string;        // User's raw input (hidden by default)

  // Expansion
  isExpanded: boolean;           // UI state
  expandedContent?: ExpandedCardContent;  // Full details when expanded
}

interface ExpandedCardContent {
  sections: ExpandedSection[];   // History, breakdown, notes, etc.
  actions: CardAction[];         // Edit, delete, add photo, etc.
}

interface EntityRef {
  type: 'food' | 'media' | 'merchant' | 'person' | 'place';
  id: string;
  name: string;
  link?: string;                 // [[Entities/Food/Omelette]] wiki-link
}
```

### 5.4 Card Rendering (React Component Structure)

```
src/features/life-stream/
├── components/
│   ├── LifeStreamView.tsx           # Main container
│   │
│   ├── stream/
│   │   ├── CardList.tsx             # Virtualized card list
│   │   ├── CardItem.tsx             # Single card container
│   │   ├── CardStateIndicator.tsx   # Processing/complete/error badge
│   │   └── ProcessingTrace.tsx      # Live reasoning steps
│   │
│   ├── cards/                       # Card type renderers
│   │   ├── MealCard.tsx
│   │   ├── DeliveryShiftCard.tsx
│   │   ├── DeliveryOrderCard.tsx
│   │   ├── MediaCard.tsx
│   │   ├── MusicCard.tsx
│   │   ├── YouTubeIdeaCard.tsx
│   │   ├── ThoughtCard.tsx
│   │   ├── WorkoutCard.tsx
│   │   ├── CodeTaskCard.tsx
│   │   ├── QueryResultCard.tsx
│   │   ├── FinanceCard.tsx
│   │   └── GenericCard.tsx
│   │
│   ├── expanded/                    # Expanded card sections
│   │   ├── ExpandedContainer.tsx
│   │   ├── NutritionBreakdown.tsx
│   │   ├── EntityHistory.tsx
│   │   ├── CodeReasoningTrace.tsx
│   │   └── CardActions.tsx
│   │
│   ├── navigation/
│   │   ├── DayPicker.tsx            # Horizontal date scroller
│   │   └── EmojiFilters.tsx         # Toggle buttons for domains
│   │
│   ├── composer/
│   │   ├── StreamComposer.tsx       # Input area
│   │   ├── VoiceButton.tsx          # Speech input
│   │   └── AttachmentButton.tsx     # Image upload
│   │
│   └── shared/
│       ├── CardImage.tsx            # Image with fallback/placeholder
│       ├── StatBadge.tsx            # Small stat display
│       └── EmojiIcon.tsx            # Consistent emoji rendering
│
├── hooks/
│   ├── useStreamCards.ts            # Card list state management
│   ├── useCardProcessing.ts         # Background task tracking
│   ├── useDayFilter.ts              # Date navigation
│   ├── useEmojiFilters.ts           # Domain filters
│   └── useStreamComposer.ts         # Input handling
│
└── types/
    └── stream.ts                    # TypeScript interfaces
```

---

## 6. Input Processing Pipeline

### 6.1 Pipeline Overview

```
┌───────────────────────────────────────────────────────────────────────┐
│                      INPUT PROCESSING PIPELINE                         │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  USER INPUT (text/voice)                                              │
│       │                                                               │
│       ▼                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ 1. IMMEDIATE CARD CREATION                                      │  │
│  │    - Generate card ID                                           │  │
│  │    - Extract timestamp (if mentioned) or use now()              │  │
│  │    - Set state = 'processing'                                   │  │
│  │    - Render placeholder card                                    │  │
│  │    - Broadcast: card_created event                              │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│       │                                                               │
│       ▼ (background, non-blocking)                                    │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ 2. INTENT DETECTION (Codex)                                     │  │
│  │    - Classify: LOG | QUERY | ACTION | CODE                      │  │
│  │    - Detect domain: nutrition | delivery | media | etc.         │  │
│  │    - Confidence score                                           │  │
│  │    - Broadcast: processing_step "Detected: meal log"            │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│       │                                                               │
│       ▼                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ 3. ENTITY EXTRACTION                                            │  │
│  │    - Parse entities: food items, media titles, merchants        │  │
│  │    - Parse quantities: "2 omelettes", "3 episodes"              │  │
│  │    - Parse context: "for mom", "at Hot n Tot"                   │  │
│  │    - Broadcast: processing_step "Found: spinach omelette x2"    │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│       │                                                               │
│       ▼                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ 4. DOMAIN ROUTING                                               │  │
│  │    - Route to appropriate handler based on intent + domain      │  │
│  │    - Handlers: NutritionHandler, DeliveryHandler, MediaHandler  │  │
│  │    - Handler fetches additional data, performs lookups          │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│       │                                                               │
│       ▼                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ 5. DATA ENRICHMENT                                              │  │
│  │    - Nutrition: lookup calories, macros, daily totals           │  │
│  │    - Delivery: fetch merchant info, calculate $/mi              │  │
│  │    - Media: fetch poster, runtime, ratings                      │  │
│  │    - Generate AI tip/insight                                    │  │
│  │    - Broadcast: processing_step "Calculating nutrition..."      │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│       │                                                               │
│       ▼                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ 6. IMAGE RESOLUTION                                             │  │
│  │    - Check local cache (Indexes/food.images.json, etc.)         │  │
│  │    - If miss: fetch from API (TMDB, Nutritionix, etc.)          │  │
│  │    - Update cache for future use                                │  │
│  │    - If all fail: set imageStatus = 'upload_prompt'             │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│       │                                                               │
│       ▼                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ 7. PERSISTENCE                                                  │  │
│  │    - Write to Obsidian Stream (markdown entry)                  │  │
│  │    - Write to Supabase table (structured data)                  │  │
│  │    - Update aggregations if needed                              │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│       │                                                               │
│       ▼                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ 8. CARD COMPLETION                                              │  │
│  │    - Update card with final data                                │  │
│  │    - Set state = 'complete'                                     │  │
│  │    - Broadcast: card_updated event                              │  │
│  │    - UI updates in-place (no page refresh)                      │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

### 6.2 Intent Classification

| Intent | Description | Example Inputs |
|--------|-------------|----------------|
| `LOG` | Record a life event | "had omelette", "watched Dune", "walked 2 miles" |
| `QUERY` | Retrieve/summarize past data | "what did I eat this week", "show recent movies" |
| `ACTION` | Modify existing data or state | "add Blond to queue", "mark Dune as watched" |
| `CODE` | Programming task (routes to Codex) | "fix login bug", "add feature X" |
| `THOUGHT` | Capture an idea or reflection | "thinking about...", "idea: ..." |
| `UNKNOWN` | Requires clarification | Ambiguous input → ask user |

### 6.3 Domain Detection Keywords

| Domain | Trigger Keywords |
|--------|------------------|
| `nutrition` | ate, had, breakfast, lunch, dinner, snack, meal, calories, protein |
| `delivery` | order, shift, doordash, uber, grubhub, merchant, $/mi, AR, acceptance |
| `media` | watched, finished, playing, rating, movie, show, anime, game, book |
| `music` | listening, played, album, song, artist, track |
| `youtube` | video idea, pipeline, thesis, hook, script, tier, youtube |
| `finance` | bill, paid, due, expense, income, spent |
| `fitness` | workout, walk, gym, strength, run, exercise, miles, steps |
| `sleep` | slept, woke, sleep, nap, hours of sleep |
| `thought` | thinking, idea, thought, wondering, reflection |
| `code` | fix, bug, implement, feature, code, commit, deploy |

---

## 7. Data Layer

### 7.1 Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        DATA FLOW                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  USER INPUT ─────────────────────────────────────────────────────▶  │
│                                                                     │
│      ┌─────────────────────────────────────────────────────────┐    │
│      │                    DAEMON                                │    │
│      │                                                         │    │
│      │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐  │    │
│      │  │ Intent      │    │ Domain      │    │ Enrichment  │  │    │
│      │  │ Detection   │───▶│ Handler     │───▶│ Service     │  │    │
│      │  │ (Codex)     │    │             │    │             │  │    │
│      │  └─────────────┘    └─────────────┘    └──────┬──────┘  │    │
│      │                                               │         │    │
│      └───────────────────────────────────────────────┼─────────┘    │
│                                                      │              │
│                     ┌────────────────────────────────┼───────────┐  │
│                     │                                │           │  │
│                     ▼                                ▼           │  │
│      ┌─────────────────────────┐      ┌─────────────────────────┐│  │
│      │     OBSIDIAN VAULT      │      │       SUPABASE          ││  │
│      │                         │      │                         ││  │
│      │  Stream/2026-01.md      │      │  meals (structured)     ││  │
│      │  ├─ | 10:20am 🍽️ ... │      │  deliveries             ││  │
│      │  └─ <!--task:id-->      │      │  media                  ││  │
│      │                         │      │  youtube_ideas          ││  │
│      │  Entities/Food/...      │      │  notes                  ││  │
│      │  ├─ spinach-omelette.md │      │                         ││  │
│      │  └─ (YAML frontmatter)  │      │  + pgvector search      ││  │
│      │                         │      │  + aggregations         ││  │
│      └────────────┬────────────┘      └────────────┬────────────┘│  │
│                   │                                │              │  │
│                   │         READ PATH              │              │  │
│                   │◀───────────────────────────────│              │  │
│                   │                                               │  │
│                   └─────────────────────────────────────────────▶│  │
│                              QUERY RESULTS                        │  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 7.2 Obsidian Integration

**Write Path (Stream Entry)**

```markdown
## Fri Jan 31
| Plan | Actual | Delta |
|------|--------|---|
| -- | 10:20am 🍽️ Spinach omelette x2 @ Hot n Tot (w/ Mom) | + | <!--task:2026-01-31-1020-meal-->
---
<!--note:2026-01-31-1020-meal-->
840cal, 56g protein. Day total: 840/2000.
```

**Read Path (Entity Lookup)**

When looking up food entity `Entities/Food/Spinach Omelette.md`:

```markdown
---
name: Spinach Omelette
category: breakfast
calories: 280
protein: 18
carbs: 4
fat: 22
fiber: 1
image: omelette-spinach.jpg
---

## Notes
High protein, low carb. Good for morning start.
```

### 7.3 Supabase Tables

| Table | Purpose | Key Fields |
|-------|---------|------------|
| `meals` | Structured meal logs | id, user_id, timestamp, items[], total_calories, total_protein, notes |
| `deliveries` | Delivery orders | id, session_id, merchant, payout, mileage, tip, timestamp |
| `delivery_sessions` | Shift logs | id, start_time, end_time, zone, starting_ar, ending_ar, total_earnings |
| `media` | Media library | id, title, type, status, rating, poster_url, year, creator |
| `youtube_ideas` | Video pipeline | id, title, tier, stage, thesis, hooks[], timestamps |
| `notes` | Quick notes | id, content, tags[], embedding, created_at |
| `stream_cards` | Card metadata | id, card_type, domain, timestamp, state, data_json |

### 7.4 life-mcp Bridge

The daemon interfaces with existing life-mcp tools through a bridge layer:

```rust
// src-tauri/src/life_mcp/bridge.rs

pub struct LifeMCPBridge {
    // Connection to life-mcp server (if running)
    client: Option<MCPClient>,
    // Fallback to direct Supabase/Obsidian access
    supabase: SupabaseClient,
    obsidian: ObsidianReader,
}

impl LifeMCPBridge {
    // Use life-mcp tool if available, else fallback
    pub async fn call_tool(&self, tool: &str, params: Value) -> Result<Value> {
        if let Some(client) = &self.client {
            client.call(tool, params).await
        } else {
            self.fallback_handler(tool, params).await
        }
    }

    // Direct implementations for critical paths
    async fn fallback_handler(&self, tool: &str, params: Value) -> Result<Value> {
        match tool {
            "log_meal_quick" => self.log_meal_direct(params).await,
            "advise_order" => self.advise_order_direct(params).await,
            "media_add" => self.media_add_direct(params).await,
            _ => Err(anyhow!("Tool not available: {}", tool)),
        }
    }
}
```

---

## 8. Background Task System

### 8.1 Non-Blocking Execution Model

```
┌───────────────────────────────────────────────────────────────────────┐
│                    BACKGROUND TASK SYSTEM                              │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  USER INPUT → TaskQueue → Worker Pool (parallel execution)            │
│                    │                                                  │
│                    ▼                                                  │
│            ┌──────────────┐                                           │
│            │ TASK QUEUE   │                                           │
│            │              │                                           │
│            │ [task-1] ────┼─────▶ Worker 1 (processing)               │
│            │ [task-2] ────┼─────▶ Worker 2 (processing)               │
│            │ [task-3] ────┼─────▶ Worker 3 (processing)               │
│            │ [task-4]     │      (waiting)                            │
│            │ [task-5]     │      (waiting)                            │
│            └──────────────┘                                           │
│                    │                                                  │
│                    ▼                                                  │
│            ┌──────────────┐                                           │
│            │ EVENT STREAM │                                           │
│            │              │                                           │
│            │ task-1: step "Parsing..."                                │
│            │ task-2: step "Fetching poster..."                        │
│            │ task-3: complete                                         │
│            │ task-1: step "Looking up calories..."                    │
│            │ task-2: complete                                         │
│            │ task-1: complete                                         │
│            └──────────────┘                                           │
│                    │                                                  │
│                    ▼ (broadcast to all clients)                       │
│            ┌──────────────┐                                           │
│            │    UI        │                                           │
│            │              │                                           │
│            │ Cards update │                                           │
│            │ independently│                                           │
│            └──────────────┘                                           │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

### 8.2 Task Types and Priorities

| Task Type | Priority | Max Concurrent | Timeout |
|-----------|----------|----------------|---------|
| `intent_detection` | HIGH | 5 | 10s |
| `entity_extraction` | HIGH | 5 | 10s |
| `data_enrichment` | MEDIUM | 3 | 30s |
| `image_fetch` | LOW | 10 | 60s |
| `code_execution` | LOW | 1 | 300s |
| `persistence` | MEDIUM | 5 | 15s |

### 8.3 Event Broadcasting

```rust
// src-tauri/src/stream/events.rs

#[derive(Serialize, Clone)]
pub enum StreamEvent {
    CardCreated { card: StreamCard },
    CardUpdated { card_id: String, updates: CardUpdate },
    CardProcessingStep { card_id: String, step: String },
    CardCompleted { card_id: String, final_card: StreamCard },
    CardError { card_id: String, error: String },
}

// Broadcast to all connected clients
pub async fn broadcast_event(event: StreamEvent, clients: &ClientPool) {
    let payload = serde_json::to_string(&event).unwrap();
    for client in clients.iter() {
        client.send(&payload).await;
    }
}
```

---

## 9. Filtering & Navigation

### 9.1 Day Picker

```
┌──────────────────────────────────────────────────────────────────┐
│  ◀  │ Wed 29 │ Thu 30 │ [Fri 31] │ Sat 1 │ Sun 2 │  ▶          │
└──────────────────────────────────────────────────────────────────┘
        │          │         ▲          │         │
        │          │         │          │         │
      (past)    (past)   (selected)  (future)  (future)
                             │
                     (current selection)
```

**Behavior:**
- Horizontal scrollable list of dates
- Current selection highlighted
- Tap date → filter stream to that day
- Swipe left/right to navigate
- Long press → jump to date picker modal
- "Today" button always accessible

### 9.2 Emoji Toggle Filters

```
┌──────────────────────────────────────────────────────────────────┐
│  [🚗]  [🍽️]  🎬   💭   🔧   📝   💤   🏋️                        │
│  (on)  (on)  (off) (off) (off) (off) (off) (off)                │
└──────────────────────────────────────────────────────────────────┘
```

**Behavior:**
- Toggle any emoji on/off
- Multiple can be active simultaneously
- All off = show all cards
- Any on = show only matching domains
- Tapping active filter turns it off
- Filter state persists in localStorage

### 9.3 Combined Filtering Logic

```typescript
function filterCards(
  cards: StreamCard[],
  selectedDate: Date,
  activeFilters: Set<string>
): StreamCard[] {
  return cards.filter(card => {
    // Date filter (required)
    const cardDate = startOfDay(card.timestamp);
    const filterDate = startOfDay(selectedDate);
    if (!isSameDay(cardDate, filterDate)) return false;

    // Emoji filter (if any active)
    if (activeFilters.size > 0) {
      return activeFilters.has(card.emoji);
    }

    return true;
  });
}
```

---

## 10. Image & Asset Management

### 10.1 Image Resolution Strategy

```
┌───────────────────────────────────────────────────────────────────────┐
│                    IMAGE RESOLUTION FLOW                               │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  Need image for "Spinach Omelette"                                    │
│       │                                                               │
│       ▼                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ 1. CHECK LOCAL CACHE                                            │  │
│  │    Path: Obsidian/Indexes/food.images.json                      │  │
│  │    Key: "spinach-omelette"                                      │  │
│  │                                                                 │  │
│  │    Found? ───▶ Return cached URL ───▶ DONE                      │  │
│  │    Not found? ▼                                                 │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│       │                                                               │
│       ▼                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ 2. CHECK ENTITY FILE                                            │  │
│  │    Path: Obsidian/Entities/Food/Spinach Omelette.md             │  │
│  │    Look for: image field in YAML frontmatter                    │  │
│  │                                                                 │  │
│  │    Found? ───▶ Return URL, update cache ───▶ DONE               │  │
│  │    Not found? ▼                                                 │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│       │                                                               │
│       ▼                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ 3. FETCH FROM API (background task)                             │  │
│  │                                                                 │  │
│  │    Domain-specific API:                                         │  │
│  │    - Food: Nutritionix / Unsplash food search                   │  │
│  │    - Media: TMDB (movies/TV), IGDB (games)                      │  │
│  │    - Music: Spotify / Apple Music / Last.fm                     │  │
│  │    - Merchant: Clearbit logo API                                │  │
│  │                                                                 │  │
│  │    Found? ───▶ Save to cache, return URL ───▶ DONE              │  │
│  │    Not found? ▼                                                 │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│       │                                                               │
│       ▼                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ 4. FALLBACK                                                     │  │
│  │                                                                 │  │
│  │    - Use domain-specific placeholder icon                       │  │
│  │    - Show "📷 Upload when food arrives" prompt                  │  │
│  │    - User can tap to upload photo later                         │  │
│  │                                                                 │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

### 10.2 Image Cache Schema

```json
// Obsidian/Indexes/food.images.json
{
  "spinach-omelette": {
    "url": "https://...",
    "source": "nutritionix",
    "fetchedAt": "2026-01-15T10:00:00Z",
    "width": 400,
    "height": 300
  },
  "home-fries": {
    "url": "https://...",
    "source": "unsplash",
    "fetchedAt": "2026-01-20T14:30:00Z"
  }
}

// Obsidian/Indexes/media.covers.v1.json
{
  "dune-2021": {
    "url": "https://image.tmdb.org/...",
    "source": "tmdb",
    "tmdbId": 438631,
    "fetchedAt": "2026-01-10T12:00:00Z"
  }
}

// Obsidian/Indexes/merchant.logos.json
{
  "hot-n-tot": {
    "url": "https://logo.clearbit.com/hotntot.com",
    "source": "clearbit",
    "fetchedAt": "2026-01-25T08:00:00Z"
  },
  "chipotle": {
    "url": "https://logo.clearbit.com/chipotle.com",
    "source": "clearbit"
  }
}
```

### 10.3 User-Uploaded Images

```
User taps "📷 Add Photo" on meal card
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHOTO UPLOAD FLOW                                               │
│                                                                 │
│ 1. Open camera/gallery picker                                   │
│ 2. User selects/takes photo                                     │
│ 3. Resize to max 1200px wide                                    │
│ 4. Upload to Supabase Storage                                   │
│ 5. Get public URL                                               │
│ 6. Update card with imageUrl                                    │
│ 7. Update local cache                                           │
│ 8. Optionally update entity file in Obsidian                    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 11. Code Action Integration

### 11.1 Code Task Flow

```
USER: "fix the login bug in CodexMonitor"
        │
        ▼
┌───────────────────────────────────────────────────────────────────────┐
│ CODE ACTION PIPELINE                                                   │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  1. IMMEDIATE CARD (action type)                                      │
│     ┌─────────────────────────────────────────┐                       │
│     │ 🔧 Code Task                       ⏳   │                       │
│     │ Working on: fix login bug               │                       │
│     │ Workspace: CodexMonitor                 │                       │
│     └─────────────────────────────────────────┘                       │
│                                                                       │
│  2. ROUTE TO CODEX APP-SERVER                                         │
│     - Detect workspace: CodexMonitor (from mention or current)        │
│     - Create new thread or continue existing                          │
│     - Send prompt: "fix the login bug"                                │
│                                                                       │
│  3. STREAM LIVE REASONING (show latest step only)                     │
│     ┌─────────────────────────────────────────┐                       │
│     │ 🔧 Code Task                       ⏳   │                       │
│     │ Working on: fix login bug               │                       │
│     │                                         │                       │
│     │ → Reading src-tauri/src/auth.rs...      │ ← updates live       │
│     └─────────────────────────────────────────┘                       │
│                                                                       │
│     Card shows only LATEST step (not full trace)                      │
│     Full trace available in expanded view                             │
│                                                                       │
│  4. COMPLETION                                                        │
│     ┌─────────────────────────────────────────┐                       │
│     │ 🔧 Fixed login bug                 ✓    │                       │
│     │ CodexMonitor                            │                       │
│     │                                         │                       │
│     │ Fixed session token refresh issue.      │                       │
│     │ Changed 3 files.                        │                       │
│     │             [tap to expand]             │                       │
│     └─────────────────────────────────────────┘                       │
│                                                                       │
│  5. EXPANDED VIEW                                                     │
│     - Full summary of changes                                         │
│     - List of files modified                                          │
│     - Toggle to show full reasoning trace                             │
│     - Link to view diff                                               │
│     - Original prompt (hidden by default)                             │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

### 11.2 Code Card Data Model Extension

```typescript
interface CodeTaskCard extends StreamCard {
  cardType: 'code_task';

  // Code-specific fields
  workspace: string;             // Target workspace name
  workspaceId: string;           // UUID
  threadId?: string;             // Codex thread ID

  // Completion data
  summary?: string;              // AI-generated summary
  filesChanged?: FileChange[];   // List of modified files

  // Reasoning
  reasoningSteps?: ReasoningStep[];  // Full trace
  showReasoning?: boolean;           // UI toggle state
}

interface FileChange {
  path: string;
  additions: number;
  deletions: number;
  status: 'added' | 'modified' | 'deleted';
}

interface ReasoningStep {
  timestamp: Date;
  content: string;
  type: 'thinking' | 'tool_call' | 'tool_result';
}
```

---

## 12. Migration Strategy

### 12.1 Migration Path

```
┌───────────────────────────────────────────────────────────────────────┐
│                    MIGRATION TIMELINE                                  │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  CURRENT STATE                                                        │
│  └─ Separate domain dashboards (Delivery, Nutrition, Media, etc.)     │
│  └─ Traditional chat interface for Life workspace                     │
│  └─ Domain data in Obsidian + Supabase                                │
│                                                                       │
│  ══════════════════════════════════════════════════════════════════   │
│                                                                       │
│  PHASE 1: Foundation (Week 1-2)                                       │
│  ├─ Add StreamCard types and CardStateManager                         │
│  ├─ Implement basic CardList rendering                                │
│  ├─ Add day picker and emoji filters                                  │
│  ├─ Create StreamComposer (text input only)                           │
│  └─ LifeStreamView available alongside existing dashboards            │
│                                                                       │
│  PHASE 2: Processing Pipeline (Week 3-4)                              │
│  ├─ Implement IntentRouter with Codex                                 │
│  ├─ Add domain handlers (nutrition, delivery, media)                  │
│  ├─ Background task queue with parallel execution                     │
│  ├─ Live processing step broadcasting                                 │
│  └─ Card state updates (processing → complete)                        │
│                                                                       │
│  PHASE 3: Rich Cards (Week 5-6)                                       │
│  ├─ Implement all card type renderers                                 │
│  ├─ Add image resolution system                                       │
│  ├─ Expanded card views with full details                             │
│  ├─ Entity linking and history                                        │
│  └─ AI tips and insights                                              │
│                                                                       │
│  PHASE 4: Voice & Mobile (Week 7-8)                                   │
│  ├─ Voice input integration (iOS)                                     │
│  ├─ SwiftUI LifeStreamView implementation                             │
│  ├─ Mobile-optimized card layouts                                     │
│  └─ Gesture support (swipe, long press)                               │
│                                                                       │
│  PHASE 5: Polish & Deprecation (Week 9-10)                            │
│  ├─ Performance optimization (virtualization)                         │
│  ├─ Accessibility improvements                                        │
│  ├─ User preference migration                                         │
│  ├─ Deprecate old domain dashboard views                              │
│  └─ Update documentation                                              │
│                                                                       │
│  ══════════════════════════════════════════════════════════════════   │
│                                                                       │
│  FINAL STATE                                                          │
│  └─ Single Visual Life Stream as primary interface                    │
│  └─ All life domains unified in chronological cards                   │
│  └─ Voice-first mobile experience                                     │
│  └─ Old dashboards removed or accessible via settings                 │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

### 12.2 Data Migration

**No data migration required** - the Visual Life Stream reads from the same sources:
- Obsidian Stream files (unchanged format)
- Supabase tables (unchanged schema)
- Image caches (unchanged)

The migration is purely UI/UX - data layer remains stable.

### 12.3 Feature Flags

```typescript
// src/features/settings/featureFlags.ts

export const FEATURE_FLAGS = {
  // Phase 1: Enable stream view (alongside dashboards)
  LIFE_STREAM_VIEW: 'life_stream_view',

  // Phase 2: Use new processing pipeline
  STREAM_PROCESSING_PIPELINE: 'stream_processing_pipeline',

  // Phase 4: Voice input
  STREAM_VOICE_INPUT: 'stream_voice_input',

  // Phase 5: Deprecate old views
  DEPRECATE_DOMAIN_DASHBOARDS: 'deprecate_domain_dashboards',
};

// In settings.json
{
  "featureFlags": {
    "life_stream_view": true,
    "stream_processing_pipeline": false,
    "stream_voice_input": false,
    "deprecate_domain_dashboards": false
  }
}
```

---

## 13. Phase 1 Scope (MVP)

### 13.1 MVP Features

| Feature | Priority | Notes |
|---------|----------|-------|
| Basic card list rendering | P0 | Static cards, no processing |
| Day picker navigation | P0 | Simple date selection |
| Emoji filter toggles | P0 | Single-select initially |
| Text composer | P0 | No voice yet |
| Card state indicators | P0 | Processing spinner, complete checkmark |
| Meal card type | P0 | First domain implementation |
| Processing step display | P0 | Live "→ Parsing..." updates |
| Intent detection (basic) | P1 | LOG vs QUERY vs CODE |
| Nutrition handler | P1 | Meal logging end-to-end |
| Image placeholders | P1 | Show placeholder, no API fetch |
| Card expansion (basic) | P1 | Show more details on tap |

### 13.2 MVP Non-Features (Deferred)

| Feature | Deferred To |
|---------|-------------|
| Voice input | Phase 4 |
| All card types | Phase 3 |
| Image API fetching | Phase 3 |
| Code action cards | Phase 3 |
| Full entity history | Phase 3 |
| iOS implementation | Phase 4 |
| Multi-filter selection | Phase 2 |
| User photo upload | Phase 3 |

### 13.3 MVP Success Criteria

1. **User can open LifeStreamView** from Life workspace
2. **User can type "had omelette for breakfast"** and see card appear
3. **Card shows processing state** with live steps
4. **Card updates to complete** with calories, macros
5. **User can filter by day** using day picker
6. **User can filter by emoji** (single selection)
7. **Card data persists** to Obsidian Stream file
8. **Multiple inputs process in parallel** without blocking

---

## 14. Open Questions

| ID | Question | Options | Decision Needed By |
|----|----------|---------|-------------------|
| OQ1 | Where should card state be persisted? | A) Only in-memory, B) Supabase `stream_cards`, C) Obsidian JSON | Phase 2 |
| OQ2 | How to handle timezone for date picker? | A) User's local, B) UTC, C) Configurable | Phase 1 |
| OQ3 | Should filters persist across sessions? | A) Yes (localStorage), B) No (reset daily), C) Configurable | Phase 1 |
| OQ4 | Max concurrent background tasks? | A) 3, B) 5, C) 10, D) Configurable | Phase 2 |
| OQ5 | How to handle very long code task reasoning? | A) Truncate, B) Collapse, C) Separate view | Phase 3 |
| OQ6 | Image cache invalidation strategy? | A) TTL-based, B) Manual refresh, C) Never expire | Phase 3 |
| OQ7 | Should deleted cards show in stream? | A) No, B) Yes (grayed out), C) Trash view | Phase 2 |
| OQ8 | How to handle offline state on mobile? | A) Queue inputs, B) Disable input, C) Local-only mode | Phase 4 |
| OQ9 | Card ordering for same timestamp? | A) Creation order, B) Alphabetical, C) Domain priority | Phase 1 |
| OQ10 | Should expanded cards stay expanded on scroll? | A) Yes, B) No (auto-collapse), C) Configurable | Phase 3 |

---

## 15. Appendix: life-mcp Assessment

### 15.1 Current life-mcp Status

**Location:** `/Volumes/YouTube 4TB/code/_archive/life-mcp/`

**Architecture:**
- Node.js MCP server
- 17 tool categories, 50+ individual tools
- Supabase integration for persistence
- MiniMax embeddings for semantic search

### 15.2 Hybrid Approach Recommendation

| Component | Recommendation | Rationale |
|-----------|----------------|-----------|
| **Tool Registry** | REBUILD in Rust | Current JS registry is complex; simpler Rust version for daemon |
| **Advisor Engine** | KEEP | Delivery order advice logic is mature and valuable |
| **Nutrition Tools** | KEEP | Food lookup, macro calculation works well |
| **Delivery Tools** | KEEP | Session management, order logging is solid |
| **Analysis Tools** | KEEP | Pattern detection, suggestions are useful |
| **Media Tools** | MIGRATE to Rust | Simpler CRUD, better to have native in daemon |
| **YouTube Tools** | MIGRATE to Rust | Same as Media |
| **Knowledge Tools** | KEEP | pgvector search is working |
| **HTTP Server** | REBUILD | Current Fastify server replaced by daemon RPC |
| **Feature Flags** | REBUILD | Simpler config in daemon settings |
| **Supabase Client** | KEEP | Existing migrations, schema, client work well |

### 15.3 Integration Strategy

```
┌───────────────────────────────────────────────────────────────────────┐
│                    life-mcp INTEGRATION STRATEGY                       │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  OPTION A: MCP Server Bridge (Recommended for Phase 1)                │
│  ──────────────────────────────────────────────────────               │
│  - Daemon spawns life-mcp as subprocess                               │
│  - Communication via MCP protocol (stdio)                             │
│  - Existing tools work unchanged                                      │
│  - Gradual migration to native Rust                                   │
│                                                                       │
│  ┌─────────────┐      MCP      ┌─────────────────┐                    │
│  │   Daemon    │ ◀───────────▶ │   life-mcp      │                    │
│  │   (Rust)    │   (stdio)     │   (Node.js)     │                    │
│  └─────────────┘               └─────────────────┘                    │
│                                                                       │
│  ────────────────────────────────────────────────────────────────     │
│                                                                       │
│  OPTION B: Direct Supabase Access (Recommended for Phase 2+)          │
│  ─────────────────────────────────────────────────────────────        │
│  - Daemon accesses Supabase directly (Rust HTTP client)               │
│  - Reimplement core tools in Rust                                     │
│  - life-mcp becomes optional/deprecated                               │
│                                                                       │
│  ┌─────────────┐    HTTP       ┌─────────────────┐                    │
│  │   Daemon    │ ◀───────────▶ │   Supabase      │                    │
│  │   (Rust)    │               │   (PostgreSQL)  │                    │
│  └─────────────┘               └─────────────────┘                    │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

### 15.4 Tools to Prioritize

**High Priority (Phase 1-2):**
1. `log_meal_quick` - Core nutrition logging
2. `advise_order` - Delivery decision support
3. `get_session_context` - Delivery session state
4. `media_add` - Media logging
5. `knowledge_search` - Semantic search

**Medium Priority (Phase 3):**
6. `food_lookup` - Nutrition data fetch
7. `yt_add_idea` - YouTube pipeline
8. `note_add` - Quick notes
9. `delivery_bulk_add` - Batch logging

**Low Priority (Phase 4+):**
10. Analysis tools
11. Relationship tools
12. Goals/rewards system

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-31 | JMWillis + Claude | Initial draft |

---

*This document is intended to be a living specification. Updates should be tracked in the Document History section above.*
