import type { StreamCard } from "../../types";
import { CardItem } from "./CardItem";

type CardListProps = {
  cards: StreamCard[];
  onCancel: (cardId: string) => void;
  onRetry: (cardId: string) => void;
};

export function CardList({ cards, onCancel, onRetry }: CardListProps) {
  if (cards.length === 0) {
    return <div className="life-dashboard-status">No cards for this day yet.</div>;
  }

  return (
    <section className="life-stream-card-list">
      {cards.map((card) => (
        <CardItem key={card.id} card={card} onCancel={onCancel} onRetry={onRetry} />
      ))}
    </section>
  );
}
