import { useCallback, useEffect, useSyncExternalStore } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { streamStore } from "../state/streamStore";
import type { LifeStreamEvent, StreamCard } from "../types";

export function useLifeStream(workspaceId: string | null) {
  // Subscribe to card list changes
  const cards = useSyncExternalStore(
    (callback) => streamStore.subscribe(callback),
    () => streamStore.getSnapshot(),
  );

  const currentDate = streamStore.getCurrentDate();

  // Load cards for current date
  const loadDay = useCallback(async (dateIso: string) => {
    if (!workspaceId) return;

    streamStore.setDate(dateIso);

    try {
      const cards = await invoke<StreamCard[]>("life_stream_load_day", {
        workspaceId,
        dateIso,
      });
      streamStore.loadCards(cards);
    } catch (err) {
      console.error("Failed to load day:", err);
    }
  }, [workspaceId]);

  // Submit new input
  const submit = useCallback(async (input: string, occurredAtIso?: string) => {
    if (!workspaceId) return;

    const cardId = crypto.randomUUID();
    const now = new Date().toISOString();

    // Optimistic: add pending card immediately
    const pendingCard: StreamCard = {
      id: cardId,
      occurredAt: occurredAtIso ?? now,
      createdAt: now,
      updatedAt: now,
      version: 1,
      cardType: "generic",
      domain: "general",
      emoji: "📝",
      state: "pending",
      processingStep: "Submitting...",
      title: input.slice(0, 50) + (input.length > 50 ? "..." : ""),
      originalInput: input,
    };

    streamStore.addCard(pendingCard);

    try {
      await invoke("life_stream_submit", {
        workspaceId,
        cardId,
        input,
        occurredAtIso,
      });
    } catch (err) {
      streamStore.setCardError(cardId, String(err), 2);
    }
  }, [workspaceId]);

  // Navigate to previous/next day
  const goToPreviousDay = useCallback(() => {
    const date = new Date(currentDate);
    date.setDate(date.getDate() - 1);
    void loadDay(date.toISOString().split("T")[0]);
  }, [currentDate, loadDay]);

  const goToNextDay = useCallback(() => {
    const date = new Date(currentDate);
    date.setDate(date.getDate() + 1);
    void loadDay(date.toISOString().split("T")[0]);
  }, [currentDate, loadDay]);

  const goToToday = useCallback(() => {
    void loadDay(new Date().toISOString().split("T")[0]);
  }, [loadDay]);

  // Listen for stream events
  useEffect(() => {
    if (!workspaceId) return;

    const unlisten = listen<LifeStreamEvent>("life_stream_event", (event) => {
      streamStore.handleEvent(event.payload);
    });

    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [workspaceId]);

  // Load today on mount
  useEffect(() => {
    if (workspaceId) {
      void loadDay(new Date().toISOString().split("T")[0]);
    }
  }, [workspaceId, loadDay]);

  return {
    cards,
    currentDate,
    submit,
    loadDay,
    goToPreviousDay,
    goToNextDay,
    goToToday,
  };
}
