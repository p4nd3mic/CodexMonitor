// Card states
export type CardState =
  | "pending"
  | "processing"
  | "awaiting_input"
  | "complete"
  | "error"
  | "cancelled";

// Card types
export type CardType =
  | "meal"
  | "delivery_order"
  | "media_add"
  | "music"
  | "thought"
  | "query"
  | "code_task"
  | "generic";

// Domain identifiers
export type DomainId =
  | "nutrition"
  | "delivery"
  | "media"
  | "youtube"
  | "finance"
  | "fitness"
  | "general";

// Image status
export type ImageStatus = "loading" | "ready" | "missing" | "upload_prompt";

// Entity reference (linked entities in cards)
export type EntityRef = {
  type: string;
  id?: string;
  name: string;
  link?: string; // Obsidian wiki-link [[Entities/Food/Omelette]]
};

// Expanded card content
export type ExpandedSection = {
  title: string;
  body: string;
};

export type CardAction = {
  id: string;
  label: string;
  icon?: string;
  style?: string;
};

export type ExpandedContent = {
  originalInput?: string;
  sections: ExpandedSection[];
  entityLinks?: EntityLink[];
  actions: CardAction[];
};

// Image metadata
export type CardImage = {
  url?: string;
  status: ImageStatus;
  source?: string;
};

export type EntityLink = {
  name: string;
  path: string;
  icon?: string;
};

export type ClarificationOption = {
  id: string;
  label: string;
  emoji?: string;
};

// Main StreamCard type
export type StreamCard = {
  id: string;
  occurredAt: string; // ISO timestamp - when event happened
  createdAt: string; // ISO timestamp - when card created
  updatedAt: string; // ISO timestamp - last update
  version: number; // For out-of-order event handling

  cardType: CardType;
  domain: DomainId;
  emoji: string;

  state: CardState;
  processingStep?: string; // Current step being shown
  processingSteps?: string[]; // History of steps

  title: string;
  subtitle?: string;
  summary?: string;

  image?: CardImage;

  stats?: Record<string, string | number>;
  entities?: EntityRef[];

  originalInput?: string;

  source?: {
    streamFile?: string;
    streamAnchor?: string;
  };

  expanded?: ExpandedContent;
  clarificationOptions?: ClarificationOption[];

  errorMessage?: string;
};

export type StreamCardPatch = {
  state?: CardState;
  title?: string;
  subtitle?: string;
  processingStep?: string;
  processingSteps?: string[];
  errorMessage?: string;
  stats?: Record<string, string | number>;
  image?: CardImage;
  expanded?: ExpandedContent;
  clarificationOptions?: ClarificationOption[];
};

// Event types for real-time updates
export type LifeStreamEvent =
  | { type: "card_created"; card: StreamCard }
  | { type: "card_step"; cardId: string; step: string; version: number }
  | {
      type: "card_updated";
      cardId: string;
      patch: StreamCardPatch;
      version: number;
    }
  | { type: "card_completed"; card: StreamCard }
  | { type: "card_error"; cardId: string; message: string; version: number };

// Filter state
export type StreamFilters = {
  date: string; // ISO date (YYYY-MM-DD)
  domains: Set<DomainId>;
};
