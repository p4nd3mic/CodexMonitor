import { useState, useRef, useEffect, type MouseEvent } from "react";
import type { MediaItem, MediaType } from "../../types";

const TYPE_CLASS: Record<MediaType, string> = {
  Film: "media-gradient-film",
  TV: "media-gradient-tv",
  Anime: "media-gradient-anime",
  Game: "media-gradient-game",
  Book: "media-gradient-book",
  YouTube: "media-gradient-youtube",
};

type MediaCardProps = {
  item: MediaItem;
  viewMode: "grid" | "list";
  onRefetchCover?: (mediaId: string) => void;
  isRefetching?: boolean;
  onViewDetails?: (item: MediaItem) => void;
};

export function MediaCard({ item, viewMode, onRefetchCover, isRefetching, onViewDetails }: MediaCardProps) {
  const [menuOpen, setMenuOpen] = useState(false);
  const [menuPos, setMenuPos] = useState({ x: 0, y: 0 });
  const menuRef = useRef<HTMLDivElement>(null);
  const [imageFailed, setImageFailed] = useState(false);

  useEffect(() => {
    setImageFailed(false);
  }, [item.coverUrl]);

  // Close menu when clicking outside
  useEffect(() => {
    if (!menuOpen) return;
    function handleClickOutside(e: Event) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setMenuOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [menuOpen]);

  const handleContextMenu = (e: MouseEvent) => {
    e.preventDefault();
    setMenuPos({ x: e.clientX, y: e.clientY });
    setMenuOpen(true);
  };

  const handleRefetch = () => {
    setMenuOpen(false);
    onRefetchCover?.(item.id);
  };

  const handleClick = () => {
    // Don't trigger if clicking on context menu
    if (menuOpen) return;
    onViewDetails?.(item);
  };

  const statusIcon = item.status === "Completed" ? "✓" : "◎";
  return (
    <article
      className={`media-card ${viewMode === "list" ? "is-list" : ""} ${isRefetching ? "is-refetching" : ""}`}
      onContextMenu={handleContextMenu}
      onClick={handleClick}
    >
      <div className="media-card__poster">
        {item.coverUrl && !imageFailed ? (
          <img
            src={item.coverUrl}
            alt={item.title}
            loading="lazy"
            onError={() => setImageFailed(true)}
          />
        ) : (
          <div className={`media-card__fallback ${TYPE_CLASS[item.type]}`}>
            <span className="media-card__fallback-title">{item.title}</span>
          </div>
        )}
        {isRefetching && (
          <div className="media-card__refetching-overlay">
            <span className="media-card__refetching-spinner">↻</span>
          </div>
        )}
        <span
          className={`media-card__badge media-card__badge--status-${item.status}`}
        >
          {statusIcon}
        </span>
        {item.rating != null ? (
          <span className="media-card__badge media-card__badge--rating">
            {item.rating.toFixed(1)}
          </span>
        ) : null}
      </div>
      <div className="media-card__info">
        <h3 className="media-card__title">{item.title}</h3>
        <span className="media-card__type">{item.type}</span>
      </div>

      {menuOpen && (
        <div
          ref={menuRef}
          className="media-card__context-menu"
          style={{
            position: "fixed",
            left: menuPos.x,
            top: menuPos.y,
            zIndex: 1000,
          }}
        >
          <button
            type="button"
            className="media-card__context-menu-item"
            onClick={handleRefetch}
            disabled={isRefetching}
          >
            {isRefetching ? "Refetching..." : "Refetch Cover"}
          </button>
        </div>
      )}
    </article>
  );
}
