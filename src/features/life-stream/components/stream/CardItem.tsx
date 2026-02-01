import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { streamStore } from "../../state/streamStore";
import type { ClarificationOption } from "../../types";
import { CardImage } from "./CardImage";
import { ExpandedCard } from "./ExpandedCard";
import { ProcessingIndicator } from "./ProcessingIndicator";

const timeFormatter = new Intl.DateTimeFormat(undefined, {
  hour: "numeric",
  minute: "2-digit",
});

function formatTime(iso: string) {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return "";
  }
  return timeFormatter.format(date);
}

type CardItemProps = {
  cardId: string;
  onCancel: (cardId: string) => void;
  onRetry: (cardId: string) => void;
  onClarify: (cardId: string, optionId: string) => void;
};

export function CardItem({ cardId, onCancel, onRetry, onClarify }: CardItemProps) {
  const card = useSyncExternalStore(
    (listener) => streamStore.subscribeToCard(cardId, listener),
    () => streamStore.getCard(cardId),
    () => undefined,
  );
  const [isExpanded, setIsExpanded] = useState(false);

  useEffect(() => {
    if (!card?.expanded || card.state !== "complete") {
      setIsExpanded(false);
    }
  }, [card?.expanded, card?.state]);

  if (!card) return null;

  const timeLabel = useMemo(() => formatTime(card.occurredAt), [card.occurredAt]);
  const canCancel = card.state === "pending" || card.state === "processing" || card.state === "awaiting_input";
  const canRetry = card.state === "error";
  const canExpand = card.state === "complete" && Boolean(card.expanded);
  const clarificationOptions = card.clarificationOptions ?? [];

  const toggleExpanded = useCallback(() => {
    if (!canExpand) return;
    setIsExpanded((prev) => !prev);
  }, [canExpand]);

  const handleClarify = useCallback((option: ClarificationOption) => {
    onClarify(card.id, option.id);
  }, [card.id, onClarify]);

  return (
    <article
      className={`life-card life-stream-card state-${card.state}${isExpanded ? " is-expanded" : ""}`}
      aria-expanded={isExpanded}
    >
      <header className="life-stream-card__header">
        <div className="life-stream-card__emoji" aria-hidden>
          {card.emoji}
        </div>
        <div className="life-stream-card__meta">
          <div className="life-stream-card__title">{card.title}</div>
          {card.subtitle && (
            <div className="life-stream-card__subtitle">{card.subtitle}</div>
          )}
        </div>
        <div className="life-stream-card__time">{timeLabel}</div>
        {canExpand && (
          <button
            type="button"
            className="life-stream-card__expand"
            onClick={toggleExpanded}
            aria-label={isExpanded ? "Collapse card" : "Expand card"}
            aria-expanded={isExpanded}
          >
            {isExpanded ? "▼" : "▶"}
          </button>
        )}
      </header>

      {card.image && (
        <CardImage
          image={card.image}
          title={card.title}
          emoji={card.emoji}
          size={isExpanded ? "expanded" : "compact"}
        />
      )}

      {card.summary && (
        <div className="life-stream-card__summary">{card.summary}</div>
      )}

      {card.stats && (
        <div className="life-stream-card__stats">
          {Object.entries(card.stats).map(([key, value]) => (
            <div key={key} className="life-stream-card__stat">
              <span className="life-stream-card__stat-label">{key}</span>
              <span className="life-stream-card__stat-value">{String(value)}</span>
            </div>
          ))}
        </div>
      )}

      {card.entities && card.entities.length > 0 && (
        <div className="life-stream-card__entities">
          {card.entities.map((entity) => (
            <span key={`${entity.type}-${entity.name}`} className="life-stream-card__entity">
              {entity.name}
            </span>
          ))}
        </div>
      )}

      {card.state === "awaiting_input" && clarificationOptions.length > 0 && (
        <div className="life-stream-card__clarification">
          <div className="life-stream-card__clarification-message">
            {card.processingStep ?? "Need more info"}
          </div>
          <div className="life-stream-card__clarification-options">
            {clarificationOptions.map((option) => (
              <button
                key={option.id}
                type="button"
                className="life-stream-card__clarification-option"
                onClick={() => handleClarify(option)}
              >
                {option.emoji && <span aria-hidden>{option.emoji}</span>}
                {option.label}
              </button>
            ))}
          </div>
        </div>
      )}

      {card.errorMessage && (
        <div role="alert" className="life-stream-card__error">
          {card.errorMessage}
        </div>
      )}

      {(canCancel || canRetry) && (
        <div className="life-stream-card__actions">
          {canCancel && (
            <button
              type="button"
              className="life-stream-card__action"
              onClick={() => onCancel(card.id)}
            >
              Cancel
            </button>
          )}
          {canRetry && (
            <button
              type="button"
              className="life-stream-card__action is-primary"
              onClick={() => onRetry(card.id)}
            >
              Retry
            </button>
          )}
        </div>
      )}

      <ProcessingIndicator card={card} />

      {isExpanded && canExpand && (
        <ExpandedCard card={card} onCollapse={() => setIsExpanded(false)} />
      )}
    </article>
  );
}
