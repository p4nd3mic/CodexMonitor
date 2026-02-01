import type { StreamCard } from "../../types";

const PROCESSING_STATES = new Set([
  "pending",
  "processing",
  "awaiting_input",
]);

type ProcessingIndicatorProps = {
  card: StreamCard;
};

export function ProcessingIndicator({ card }: ProcessingIndicatorProps) {
  if (!PROCESSING_STATES.has(card.state)) {
    return null;
  }

  return (
    <div
      className="life-stream-processing"
      role="status"
      aria-live="polite"
      aria-label={`Processing: ${card.processingStep ?? "Processing..."}`}
    >
      <span className="life-stream-spinner" aria-hidden />
      <span className="life-stream-processing__text">
        {card.processingStep ?? "Processing..."}
      </span>
    </div>
  );
}
