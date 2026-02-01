import { CardItem } from "./CardItem";
import { CardErrorBoundary } from "./CardErrorBoundary";

type CardListProps = {
  cardIds: string[];
  onCancel: (cardId: string) => void;
  onRetry: (cardId: string) => void;
  onClarify: (cardId: string, optionId: string) => void;
};

export function CardList({ cardIds, onCancel, onRetry, onClarify }: CardListProps) {
  if (cardIds.length === 0) {
    return <div className="life-dashboard-status">No cards for this day yet.</div>;
  }

  return (
    <section className="life-stream-card-list">
      {cardIds.map((cardId) => (
        <CardErrorBoundary key={cardId} cardId={cardId}>
          <CardItem
            cardId={cardId}
            onCancel={onCancel}
            onRetry={onRetry}
            onClarify={onClarify}
          />
        </CardErrorBoundary>
      ))}
    </section>
  );
}
