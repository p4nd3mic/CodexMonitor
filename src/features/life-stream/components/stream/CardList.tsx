import type { StreamCard } from "../../types";
import { CardItem } from "./CardItem";

type CardListProps = {
  cards: StreamCard[];
};

export function CardList({ cards }: CardListProps) {
  if (cards.length === 0) {
    return <div className="life-dashboard-status">No cards for this day yet.</div>;
  }

  return (
    <section className="life-stream-card-list">
      {cards.map((card) => (
        <CardItem key={card.id} card={card} />
      ))}
    </section>
  );
}
