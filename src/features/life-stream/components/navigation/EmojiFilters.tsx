import type { DomainId } from "../../types";

const FILTERS: Array<{ domain: DomainId; label: string; emoji: string }> = [
  { domain: "nutrition", label: "Meals", emoji: "🍽️" },
  { domain: "delivery", label: "Delivery", emoji: "🚗" },
  { domain: "media", label: "Media", emoji: "🎬" },
  { domain: "youtube", label: "YouTube", emoji: "🎥" },
  { domain: "finance", label: "Finance", emoji: "💸" },
  { domain: "fitness", label: "Fitness", emoji: "🏋️" },
  { domain: "general", label: "Notes", emoji: "📝" },
];

type EmojiFiltersProps = {
  activeFilters: Set<DomainId>;
  onToggle: (domain: DomainId) => void;
  onClear: () => void;
};

export function EmojiFilters({ activeFilters, onToggle, onClear }: EmojiFiltersProps) {
  return (
    <section className="life-card life-stream-filters">
      <div className="life-stream-filters__header">
        <div className="life-section-title">Filters</div>
        {activeFilters.size > 0 && (
          <button
            type="button"
            className="life-stream-filters__clear"
            onClick={onClear}
          >
            Clear
          </button>
        )}
      </div>
      <div className="life-segment-control life-stream-filters__controls">
        {FILTERS.map((filter) => {
          const active = activeFilters.has(filter.domain);
          return (
            <button
              type="button"
              key={filter.domain}
              className={`life-segment-button ${active ? "is-active" : ""}`}
              onClick={() => onToggle(filter.domain)}
              aria-pressed={active}
              aria-label={`Filter ${filter.label.toLowerCase()}`}
            >
              <span className="life-stream-filters__emoji" aria-hidden>
                {filter.emoji}
              </span>
              {filter.label}
            </button>
          );
        })}
      </div>
    </section>
  );
}
