import { useCallback, useMemo, useState } from "react";
import { useLifeStream } from "../hooks/useLifeStream";
import { DayPicker } from "./navigation/DayPicker";
import { EmojiFilters } from "./navigation/EmojiFilters";
import { CardList } from "./stream/CardList";
import { StreamComposer } from "./composer/StreamComposer";
import type { DomainId } from "../types";
import "./LifeStreamView.css";

type LifeStreamViewProps = {
  workspaceId: string | null;
};

export function LifeStreamView({ workspaceId }: LifeStreamViewProps) {
  const {
    cards,
    isLoading,
    currentDate,
    submit,
    cancel,
    retry,
    clarify,
    goToPreviousDay,
    goToNextDay,
    goToToday,
  } = useLifeStream(workspaceId);
  const filterStorageKey = "life-stream-filters";

  const [activeFilters, setActiveFilters] = useState<Set<DomainId>>(() => {
    if (typeof window === "undefined") {
      return new Set();
    }
    const stored = window.localStorage.getItem(filterStorageKey);
    if (!stored) return new Set();
    try {
      const parsed = JSON.parse(stored) as DomainId[];
      return new Set(parsed);
    } catch {
      return new Set();
    }
  });

  const persistFilters = useCallback((next: Set<DomainId>) => {
    setActiveFilters(next);
    if (typeof window !== "undefined") {
      window.localStorage.setItem(filterStorageKey, JSON.stringify(Array.from(next)));
    }
  }, [filterStorageKey]);

  const toggleFilter = useCallback((domain: DomainId) => {
    const next = new Set(activeFilters);
    if (next.has(domain)) {
      next.delete(domain);
    } else {
      next.add(domain);
    }
    persistFilters(next);
  }, [activeFilters, persistFilters]);

  const clearFilters = useCallback(() => {
    persistFilters(new Set());
  }, [persistFilters]);

  // Filter cards by active domain filters
  const filteredCards = useMemo(() => (
    activeFilters.size === 0
      ? cards
      : cards.filter((card) => activeFilters.has(card.domain))
  ), [activeFilters, cards]);

  const cardIds = useMemo(() => filteredCards.map((card) => card.id), [filteredCards]);

  const handleSubmit = useCallback((text: string) => {
    return submit(text);
  }, [submit]);

  if (!workspaceId) {
    return <div className="life-stream-empty">Select a Life workspace</div>;
  }

  return (
    <div className="life-dashboard life-stream-dashboard">
      <div className="life-stream-header">
        <div>
          <div className="life-dashboard-title">Visual Life Stream</div>
          <div className="life-dashboard-subtitle">
            Unified timeline of your day.
          </div>
        </div>
      </div>

      <DayPicker
        currentDate={currentDate}
        onPrevious={goToPreviousDay}
        onNext={goToNextDay}
        onToday={goToToday}
      />

      <EmojiFilters
        activeFilters={activeFilters}
        onToggle={toggleFilter}
        onClear={clearFilters}
      />

      {isLoading ? (
        <div className="life-dashboard-status">Loading stream...</div>
      ) : (
        <CardList
          cardIds={cardIds}
          onCancel={cancel}
          onRetry={retry}
          onClarify={clarify}
        />
      )}

      <StreamComposer onSubmit={handleSubmit} />
    </div>
  );
}
