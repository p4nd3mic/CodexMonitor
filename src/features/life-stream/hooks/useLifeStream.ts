import { useCallback, useEffect, useRef, useState, useSyncExternalStore } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { streamStore } from "../state/streamStore";
import type { LifeStreamEvent, StreamCard } from "../types";

export function useLifeStream(workspaceId: string | null) {
  const [isLoading, setIsLoading] = useState(false);
  const requestIdRef = useRef(0);

  // Subscribe to card list changes
  const cards = useSyncExternalStore(
    (callback) => streamStore.subscribe(callback),
    () => streamStore.getSnapshot(),
    () => [],
  );

  const currentDate = streamStore.getCurrentDate();

  // Load cards for current date
  const loadDay = useCallback(async (dateIso: string) => {
    if (!workspaceId) return;

    streamStore.setDate(dateIso);
    const requestId = ++requestIdRef.current;
    setIsLoading(true);

    try {
      const cards = await invoke<StreamCard[]>("life_stream_load_day", {
        workspaceId,
        dateIso,
      });
      if (requestIdRef.current === requestId) {
        streamStore.loadCards(cards);
      }
    } catch (err) {
      console.error("Failed to load day:", err);
    } finally {
      if (requestIdRef.current === requestId) {
        setIsLoading(false);
      }
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
      processingSteps: ["Submitting..."],
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

  const cancel = useCallback(async (cardId: string) => {
    if (!workspaceId) return;
    const existing = streamStore.getCard(cardId);
    if (existing) {
      streamStore.updateCard(
        cardId,
        { state: "cancelled", processingStep: "Cancelled" },
        existing.version + 1,
      );
    }
    try {
      await invoke("life_stream_cancel", { workspaceId, cardId });
    } catch (err) {
      console.error("Failed to cancel card:", err);
    }
  }, [workspaceId]);

  const retry = useCallback(async (cardId: string) => {
    if (!workspaceId) return;
    const existing = streamStore.getCard(cardId);
    if (existing) {
      streamStore.updateCard(
        cardId,
        { state: "processing", processingStep: "Retrying...", errorMessage: undefined },
        existing.version + 1,
      );
    }
    try {
      await invoke("life_stream_retry", { workspaceId, cardId });
    } catch (err) {
      console.error("Failed to retry card:", err);
      if (existing) {
        streamStore.setCardError(cardId, String(err), existing.version + 2);
      }
    }
  }, [workspaceId]);

  const clarify = useCallback(async (cardId: string, optionId: string) => {
    if (!workspaceId) return;
    const existing = streamStore.getCard(cardId);
    if (existing) {
      streamStore.updateCard(
        cardId,
        {
          state: "processing",
          processingStep: "Resuming...",
          clarificationOptions: [],
        },
        existing.version + 1,
      );
    }
    try {
      await invoke("life_stream_clarify", { workspaceId, cardId, optionId });
    } catch (err) {
      console.error("Failed to clarify card:", err);
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
    isLoading,
    currentDate,
    submit,
    cancel,
    retry,
    clarify,
    loadDay,
    goToPreviousDay,
    goToNextDay,
    goToToday,
  };
}
