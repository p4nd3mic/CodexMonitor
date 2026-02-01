import { beforeEach, describe, expect, it, vi } from "vitest";
import type { StreamCard } from "../types";
import { streamStore } from "../state/streamStore";

const makeCard = (overrides: Partial<StreamCard> = {}): StreamCard => ({
  id: overrides.id ?? "card-1",
  occurredAt: overrides.occurredAt ?? "2026-01-21T12:00:00",
  createdAt: overrides.createdAt ?? "2026-01-21T12:00:00",
  updatedAt: overrides.updatedAt ?? "2026-01-21T12:00:00",
  version: overrides.version ?? 1,
  cardType: overrides.cardType ?? "generic",
  domain: overrides.domain ?? "general",
  emoji: overrides.emoji ?? "📝",
  state: overrides.state ?? "processing",
  title: overrides.title ?? "Lunch log",
  subtitle: overrides.subtitle,
  summary: overrides.summary,
  image: overrides.image,
  stats: overrides.stats,
  entities: overrides.entities,
  originalInput: overrides.originalInput,
  source: overrides.source,
  expanded: overrides.expanded,
  processingStep: overrides.processingStep,
  processingSteps: overrides.processingSteps,
  errorMessage: overrides.errorMessage,
});

describe("streamStore", () => {
  beforeEach(() => {
    streamStore.loadCards([]);
    streamStore.setDate("2026-01-21");
  });

  it("adds cards and notifies global subscribers", () => {
    const listener = vi.fn();
    const unsubscribe = streamStore.subscribe(listener);

    const card = makeCard({ id: "card-add" });
    streamStore.addCard(card);

    expect(listener).toHaveBeenCalledTimes(1);
    expect(streamStore.getSnapshot()).toEqual([card]);

    unsubscribe();
  });

  it("updates cards with newer versions and notifies per-card listeners", () => {
    const card = makeCard({ id: "card-update", title: "Original" });
    streamStore.addCard(card);

    const globalListener = vi.fn();
    const unsubscribeGlobal = streamStore.subscribe(globalListener);
    const cardListener = vi.fn();
    const unsubscribeCard = streamStore.subscribeToCard("card-update", cardListener);

    streamStore.updateCard("card-update", { title: "Updated" }, 2);

    expect(cardListener).toHaveBeenCalledTimes(1);
    expect(streamStore.getCard("card-update")?.title).toBe("Updated");

    expect(globalListener).not.toHaveBeenCalled();

    streamStore.updateCard("card-update", { title: "Stale" }, 1);
    expect(cardListener).toHaveBeenCalledTimes(1);

    unsubscribeCard();
    unsubscribeGlobal();
  });

  it("only notifies per-card subscribers for matching cards", () => {
    const cardA = makeCard({ id: "card-a", title: "A" });
    const cardB = makeCard({ id: "card-b", title: "B", occurredAt: "2026-01-21T13:00:00" });
    streamStore.loadCards([cardA, cardB]);

    const listener = vi.fn();
    const unsubscribe = streamStore.subscribeToCard("card-a", listener);

    streamStore.updateCard("card-b", { title: "B+" }, 2);
    expect(listener).not.toHaveBeenCalled();

    streamStore.updateCard("card-a", { title: "A+" }, 2);
    expect(listener).toHaveBeenCalledTimes(1);

    unsubscribe();

    streamStore.updateCard("card-a", { title: "A++" }, 3);
    expect(listener).toHaveBeenCalledTimes(1);
  });
});
