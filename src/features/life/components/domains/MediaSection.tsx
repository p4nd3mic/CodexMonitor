import type { MediaItem } from "../../types";
import { MediaCard } from "./MediaCard";

type MediaSectionProps = {
  title: string;
  count: number;
  items: MediaItem[];
  viewMode: "grid" | "list";
  onRefetchCover?: (mediaId: string) => void;
  refetchingIds?: Set<string>;
  onViewDetails?: (item: MediaItem) => void;
  coverEpoch?: number;
};

export function MediaSection({
  title,
  count,
  items,
  viewMode,
  onRefetchCover,
  refetchingIds,
  onViewDetails,
  coverEpoch,
}: MediaSectionProps) {
  const epoch = coverEpoch ?? 0;
  return (
    <section className="media-section">
      <div className="media-section-header">
        <div className="media-section-title">
          {title} <span className="media-section-count">({count})</span>
        </div>
      </div>
      <div className={`media-grid ${viewMode === "list" ? "is-list" : ""}`}>
        {items.map((item) => (
          <MediaCard
            key={`${item.id}-${epoch}`}
            item={item}
            viewMode={viewMode}
            onRefetchCover={onRefetchCover}
            isRefetching={refetchingIds?.has(item.id)}
            onViewDetails={onViewDetails}
          />
        ))}
      </div>
    </section>
  );
}
