import type { CardAction, ExpandedSection, StreamCard } from "../../types";
import "./StreamCardExtras.css";

type ExpandedCardProps = {
  card: StreamCard;
  onCollapse: () => void;
  onAction?: (action: CardAction) => void;
};

function renderSection(section: ExpandedSection) {
  return (
    <div key={section.title} className="life-stream-card-expanded__section">
      <div className="life-stream-card-expanded__title">{section.title}</div>
      <div
        className="life-stream-card-expanded__body"
        dangerouslySetInnerHTML={{ __html: section.body }}
      />
    </div>
  );
}

export function ExpandedCard({ card, onCollapse, onAction }: ExpandedCardProps) {
  const expanded = card.expanded;

  if (!expanded) return null;

  const sections = expanded.sections ?? [];
  const actions = expanded.actions ?? [];
  const entityLinks = expanded.entityLinks ?? [];
  const originalInput = expanded.originalInput ?? card.originalInput;

  return (
    <section className="life-stream-card-expanded">
      <button
        type="button"
        className="life-stream-card-expanded__close"
        onClick={onCollapse}
        aria-label="Collapse card"
      >
        ✕
      </button>

      {originalInput && (
        <div className="life-stream-card-expanded__section">
          <div className="life-stream-card-expanded__title">Original input</div>
          <div className="life-stream-card-expanded__body is-mono">{originalInput}</div>
        </div>
      )}

      {sections.map(renderSection)}

      {entityLinks.length > 0 && (
        <div className="life-stream-card-expanded__section">
          <div className="life-stream-card-expanded__title">Related</div>
          <div className="life-stream-card-expanded__links">
            {entityLinks.map((link) => (
              <span key={link.path} className="life-stream-card-expanded__link">
                {link.icon && <span aria-hidden>{link.icon}</span>}
                <span>{link.name}</span>
              </span>
            ))}
          </div>
        </div>
      )}

      {actions.length > 0 && (
        <div className="life-stream-card-expanded__actions">
          {actions.map((action) => (
            <button
              key={action.id}
              type="button"
              className={`life-stream-card-expanded__action${action.style ? ` is-${action.style}` : ""}`}
              onClick={() => onAction?.(action)}
            >
              {action.icon && <span aria-hidden>{action.icon}</span>}
              {action.label}
            </button>
          ))}
        </div>
      )}
    </section>
  );
}
