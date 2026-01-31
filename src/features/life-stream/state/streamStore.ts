import type { LifeStreamEvent, StreamCard } from "../types";

type Listener = () => void;
type CardListener = (card: StreamCard) => void;

class StreamStore {
  private cards: Map<string, StreamCard> = new Map();
  private cardListeners: Map<string, Set<CardListener>> = new Map();
  private globalListeners: Set<Listener> = new Set();
  private currentDate: string = new Date().toISOString().split("T")[0];

  // Global subscription (for CardList)
  subscribe(listener: Listener): () => void {
    this.globalListeners.add(listener);
    return () => this.globalListeners.delete(listener);
  }

  // Per-card subscription (for CardItem - prevents full list re-render)
  subscribeToCard(cardId: string, listener: CardListener): () => void {
    if (!this.cardListeners.has(cardId)) {
      this.cardListeners.set(cardId, new Set());
    }
    this.cardListeners.get(cardId)!.add(listener);
    return () => {
      const listeners = this.cardListeners.get(cardId);
      if (listeners) {
        listeners.delete(listener);
        if (listeners.size === 0) {
          this.cardListeners.delete(cardId);
        }
      }
    };
  }

  getSnapshot(): StreamCard[] {
    return Array.from(this.cards.values())
      .filter((card) => card.occurredAt.startsWith(this.currentDate))
      .sort((a, b) => b.occurredAt.localeCompare(a.occurredAt));
  }

  getCard(cardId: string): StreamCard | undefined {
    return this.cards.get(cardId);
  }

  getCurrentDate(): string {
    return this.currentDate;
  }

  // Actions
  setDate(dateIso: string): void {
    this.currentDate = dateIso;
    this.notifyGlobal();
  }

  loadCards(cards: StreamCard[]): void {
    this.cards.clear();
    for (const card of cards) {
      this.cards.set(card.id, card);
    }
    this.notifyGlobal();
  }

  addCard(card: StreamCard): void {
    this.cards.set(card.id, card);
    this.notifyGlobal();
  }

  updateCard(cardId: string, patch: Partial<StreamCard>, newVersion: number): void {
    const existing = this.cards.get(cardId);
    if (!existing) return;

    // Only apply if version is newer
    if (newVersion <= existing.version) return;

    const updated = { ...existing, ...patch, version: newVersion };
    this.cards.set(cardId, updated);
    this.notifyCard(cardId, updated);
  }

  setCardStep(cardId: string, step: string, newVersion: number): void {
    const existing = this.cards.get(cardId);
    if (!existing) return;
    if (newVersion <= existing.version) return;

    const steps = existing.processingSteps ?? [];
    const updated: StreamCard = {
      ...existing,
      processingStep: step,
      processingSteps: [...steps, step],
      version: newVersion,
    };
    this.cards.set(cardId, updated);
    this.notifyCard(cardId, updated);
  }

  completeCard(card: StreamCard): void {
    const existing = this.cards.get(card.id);
    if (existing && card.version <= existing.version) return;

    this.cards.set(card.id, card);
    this.notifyCard(card.id, card);
  }

  setCardError(cardId: string, message: string, newVersion: number): void {
    const existing = this.cards.get(cardId);
    if (!existing) return;
    if (newVersion <= existing.version) return;

    const updated: StreamCard = {
      ...existing,
      state: "error",
      errorMessage: message,
      version: newVersion,
    };
    this.cards.set(cardId, updated);
    this.notifyCard(cardId, updated);
  }

  // Handle incoming event
  handleEvent(event: LifeStreamEvent): void {
    switch (event.type) {
      case "card_created":
        this.addCard(event.card);
        break;
      case "card_step":
        this.setCardStep(event.cardId, event.step, event.version);
        break;
      case "card_updated":
        this.updateCard(event.cardId, event.patch as Partial<StreamCard>, event.version);
        break;
      case "card_completed":
        this.completeCard(event.card);
        break;
      case "card_error":
        this.setCardError(event.cardId, event.message, event.version);
        break;
    }
  }

  private notifyGlobal(): void {
    for (const listener of this.globalListeners) {
      listener();
    }
  }

  private notifyCard(cardId: string, card: StreamCard): void {
    const listeners = this.cardListeners.get(cardId);
    if (listeners) {
      for (const listener of listeners) {
        listener(card);
      }
    }
  }
}

export const streamStore = new StreamStore();
