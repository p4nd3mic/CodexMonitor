import { useMemo, useState, useCallback, useEffect, useRef } from "react";
import type { MediaFilterState, MediaItem, MediaType } from "../../types";
import { useMediaLibrary } from "../../hooks/useMediaLibrary";
import {
  rebuildMediaCovers,
  refetchMediaCover,
} from "../../../../services/tauri";
import { MediaFilterBar } from "./MediaFilterBar";
import { MediaSection } from "./MediaSection";
import { MediaDetailPanel } from "./MediaDetailPanel";

const MEDIA_TYPES: MediaType[] = [
  "Anime",
  "Film",
  "TV",
  "Game",
  "Book",
  "YouTube",
];

const TYPE_EMOJI: Record<MediaType, string> = {
  Film: "🎬",
  TV: "📺",
  Anime: "🎌",
  Game: "🎮",
  Book: "📚",
  YouTube: "🎥",
};

type MediaDashboardProps = {
  workspaceId: string | null;
};

const AUTO_REBUILD_WORKSPACES = new Set<string>();

export function MediaDashboard({ workspaceId }: MediaDashboardProps) {
  const { library, summary, filters, updateFilters, loading, error, refresh } =
    useMediaLibrary(workspaceId);
  const [refetchingIds, setRefetchingIds] = useState<Set<string>>(new Set());
  const [rebuildingCovers, setRebuildingCovers] = useState(false);
  const [rebuildToast, setRebuildToast] = useState<string | null>(null);
  const [selectedItem, setSelectedItem] = useState<MediaItem | null>(null);
  const [coverEpoch, setCoverEpoch] = useState(0);
  const toastTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (toastTimeoutRef.current) {
        clearTimeout(toastTimeoutRef.current);
      }
    };
  }, []);

  const handleViewDetails = useCallback((item: MediaItem) => {
    setSelectedItem(item);
  }, []);

  const handleCloseDetails = useCallback(() => {
    setSelectedItem(null);
  }, []);

  const handleRefetchCover = useCallback(
    async (mediaId: string) => {
      if (!workspaceId) return;
      setRefetchingIds((prev) => new Set(prev).add(mediaId));
      try {
        await refetchMediaCover(workspaceId, mediaId);
        await refresh();
      } catch (err) {
        console.error("Failed to refetch cover:", err);
      } finally {
        setRefetchingIds((prev) => {
          const next = new Set(prev);
          next.delete(mediaId);
          return next;
        });
      }
    },
    [workspaceId, refresh],
  );

  const handleRebuildCovers = useCallback(async () => {
    if (!workspaceId) return;
    setRebuildingCovers(true);
    try {
      const result = await rebuildMediaCovers(workspaceId);
      const failedCount = result.failed.length;
      const message = `Rebuilt ${result.rebuilt} covers, ${failedCount} failed`;
      setRebuildToast(message);
      if (toastTimeoutRef.current) {
        clearTimeout(toastTimeoutRef.current);
      }
      toastTimeoutRef.current = setTimeout(() => {
        setRebuildToast(null);
      }, 4000);
      await refresh();
      setCoverEpoch((prev) => prev + 1);
    } catch (err) {
      console.error("Failed to rebuild covers:", err);
      setRebuildToast("Cover rebuild failed.");
      if (toastTimeoutRef.current) {
        clearTimeout(toastTimeoutRef.current);
      }
      toastTimeoutRef.current = setTimeout(() => {
        setRebuildToast(null);
      }, 4000);
    } finally {
      setRebuildingCovers(false);
    }
  }, [workspaceId, refresh]);

  useEffect(() => {
    if (!workspaceId || !library) return;
    if (AUTO_REBUILD_WORKSPACES.has(workspaceId)) return;
    AUTO_REBUILD_WORKSPACES.add(workspaceId);
    void handleRebuildCovers();
  }, [workspaceId, library, handleRebuildCovers]);

  const filteredItems = useMemo(() => {
    if (!library) return [];
    return applyFilters(library.items, filters);
  }, [library, filters]);

  const grouped = useMemo(() => {
    const groups = new Map<MediaType, MediaItem[]>();
    for (const type of MEDIA_TYPES) {
      groups.set(type, []);
    }
    for (const item of filteredItems) {
      const list = groups.get(item.type) ?? [];
      list.push(item);
      groups.set(item.type, list);
    }
    for (const [key, list] of groups) {
      groups.set(key, sortItems(list, filters.sort));
    }
    return groups;
  }, [filteredItems, filters.sort]);

  const headerLine = summary
    ? `${summary.totalCount} items • ${summary.completedCount} completed • ⭐ ${summary.avgRating.toFixed(
        1,
      )} avg`
    : "--";

  return (
    <div className="life-dashboard life-media-dashboard">
      <div className="life-dashboard-header">
        <div>
          <div className="life-dashboard-title">🎬 Media Library</div>
          <div className="life-dashboard-subtitle">{headerLine}</div>
        </div>
        <div className="life-dashboard-actions">
          <button
            type="button"
            className="ghost life-refresh-button"
            onClick={() => void refresh()}
            disabled={loading}
          >
            Refresh
          </button>
        </div>
      </div>

      <MediaFilterBar filters={filters} onChange={updateFilters} />

      {rebuildingCovers && (
        <div className="life-dashboard-status">
          <span className="media-card__refetching-spinner">↻</span> Rebuilding covers…
        </div>
      )}
      {rebuildToast && <div className="life-dashboard-status">{rebuildToast}</div>}

      {error && <div className="life-dashboard-error">{error}</div>}
      {loading && !library && (
        <div className="life-dashboard-status">Loading media library…</div>
      )}

      {library ? (
        <div className="media-sections">
          {MEDIA_TYPES.map((type) => {
            const items = grouped.get(type) ?? [];
            if (items.length === 0) return null;
            return (
              <MediaSection
                key={type}
                title={`${TYPE_EMOJI[type]} ${type}`}
                count={items.length}
                items={items}
                viewMode={filters.viewMode}
                onRefetchCover={handleRefetchCover}
                refetchingIds={refetchingIds}
                onViewDetails={handleViewDetails}
                coverEpoch={coverEpoch}
              />
            );
          })}
        </div>
      ) : null}

      {selectedItem && (
        <MediaDetailPanel item={selectedItem} onClose={handleCloseDetails} />
      )}
    </div>
  );

}

function applyFilters(items: MediaItem[], filters: MediaFilterState) {
  return items.filter((item) => {
    if (filters.type !== "all" && item.type !== filters.type) {
      return false;
    }
    if (filters.status !== "all" && item.status !== filters.status) {
      return false;
    }
    if (filters.search.trim()) {
      const query = filters.search.toLowerCase();
      if (!item.title.toLowerCase().includes(query)) {
        return false;
      }
    }
    return true;
  });
}

function sortItems(items: MediaItem[], sort: MediaFilterState["sort"]) {
  const list = [...items];
  switch (sort) {
    case "rating":
      return list.sort((a, b) => (b.rating ?? 0) - (a.rating ?? 0));
    case "title":
      return list.sort((a, b) => a.title.localeCompare(b.title));
    case "updated":
      return list.sort(
        (a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime(),
      );
    case "type":
      return list.sort((a, b) => a.type.localeCompare(b.type));
    default:
      return list;
  }
}
