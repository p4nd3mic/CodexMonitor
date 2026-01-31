import { useCallback, useState } from "react";
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
    currentDate,
    submit,
    goToPreviousDay,
    goToNextDay,
    goToToday,
  } = useLifeStream(workspaceId);

  const [activeFilters, setActiveFilters] = useState<Set<DomainId>>(new Set());

  const toggleFilter = useCallback((domain: DomainId) => {
    setActiveFilters((prev) => {
      const next = new Set(prev);
      if (next.has(domain)) {
        next.delete(domain);
      } else {
        next.add(domain);
      }
      return next;
    });
  }, []);

  const clearFilters = useCallback(() => {
    setActiveFilters(new Set());
  }, []);

  // Filter cards by active domain filters
  const filteredCards = activeFilters.size === 0
    ? cards
    : cards.filter((card) => activeFilters.has(card.domain));

  const handleSubmit = useCallback((text: string) => {
    void submit(text);
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

      <CardList cards={filteredCards} />

      <StreamComposer onSubmit={handleSubmit} />
    </div>
  );
}
