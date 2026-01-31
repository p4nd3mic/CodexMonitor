import { useMemo } from "react";
import type { StreamCard } from "../../types";
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
  card: StreamCard;
};

export function CardItem({ card }: CardItemProps) {
  const timeLabel = useMemo(() => formatTime(card.occurredAt), [card.occurredAt]);

  return (
    <article className={`life-card life-stream-card state-${card.state}`}>
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
      </header>

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

      {card.errorMessage && (
        <div className="life-stream-card__error">
          {card.errorMessage}
        </div>
      )}

      <ProcessingIndicator card={card} />
    </article>
  );
}
