import { useEffect, useRef } from "react";
import type { MediaItem, MediaType } from "../../types";

const TYPE_CLASS: Record<MediaType, string> = {
  Film: "media-gradient-film",
  TV: "media-gradient-tv",
  Anime: "media-gradient-anime",
  Game: "media-gradient-game",
  Book: "media-gradient-book",
  YouTube: "media-gradient-youtube",
};

const TYPE_EMOJI: Record<MediaType, string> = {
  Film: "🎬",
  TV: "📺",
  Anime: "🎌",
  Game: "🎮",
  Book: "📚",
  YouTube: "🎥",
};

type MediaDetailPanelProps = {
  item: MediaItem;
  onClose: () => void;
};

export function MediaDetailPanel({ item, onClose }: MediaDetailPanelProps) {
  const panelRef = useRef<HTMLDivElement>(null);

  // Close on escape key
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        onClose();
      }
    }
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  // Close when clicking outside the panel
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        onClose();
      }
    }
    // Use setTimeout to avoid the click that opened the panel from closing it
    const timer = setTimeout(() => {
      document.addEventListener("mousedown", handleClickOutside);
    }, 100);
    return () => {
      clearTimeout(timer);
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [onClose]);

  const statusIcon = item.status === "Completed" ? "✓" : "◎";
  const statusLabel = item.status === "Completed" ? "Completed" : "Backlog";

  return (
    <div className="media-detail-overlay">
      <div ref={panelRef} className="media-detail-panel">
        <button
          type="button"
          className="media-detail-close"
          onClick={onClose}
          aria-label="Close"
        >
          ×
        </button>

        <div className="media-detail-header">
          <div className="media-detail-poster">
            {item.coverUrl ? (
              <img src={item.coverUrl} alt={item.title} />
            ) : (
              <div className={`media-detail-fallback ${TYPE_CLASS[item.type]}`}>
                <span className="media-detail-fallback-title">{item.title}</span>
              </div>
            )}
          </div>

          <div className="media-detail-meta">
            <h2 className="media-detail-title">{item.title}</h2>
            <div className="media-detail-info">
              <span className="media-detail-type">
                {TYPE_EMOJI[item.type]} {item.type}
              </span>
              {item.year && (
                <span className="media-detail-year">{item.year}</span>
              )}
              <span className={`media-detail-status media-detail-status--${item.status}`}>
                {statusIcon} {statusLabel}
              </span>
              {item.rating != null && (
                <span className="media-detail-rating">
                  ⭐ {item.rating.toFixed(1)}
                </span>
              )}
            </div>
            <div className="media-detail-dates">
              {item.completedAt && (
                <span className="media-detail-date">
                  Completed: {formatDate(item.completedAt)}
                </span>
              )}
              <span className="media-detail-date">
                Updated: {formatDate(item.updatedAt)}
              </span>
            </div>
          </div>
        </div>

        <div className="media-detail-content">
          {item.notes ? (
            <div className="media-detail-notes">
              <h3 className="media-detail-notes-title">Notes</h3>
              <div className="media-detail-notes-body">{item.notes}</div>
            </div>
          ) : (
            <div className="media-detail-empty">
              No notes for this item yet.
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  } catch {
    return dateStr;
  }
}
