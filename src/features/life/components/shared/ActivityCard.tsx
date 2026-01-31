import type { ExerciseEntry } from "../../types";

export type ActivityCardProps = ExerciseEntry;

export function ActivityCard({
  type,
  description,
  timestamp,
  miles,
  duration,
}: ActivityCardProps) {
  return (
    <div className="activity-card">
      <div className="activity-card__icon">
        <span>{TYPE_EMOJI[type] ?? "✨"}</span>
      </div>
      <div className="activity-card__body">
        <div className="activity-card__title">{description}</div>
        <div className="activity-card__meta">
          {duration ? `${duration.toFixed(0)} min` : ""}
          {miles ? `${duration ? " · " : ""}${miles.toFixed(1)} mi` : ""}
        </div>
        <div className="activity-card__time">{formatEntryTime(timestamp)}</div>
      </div>
    </div>
  );
}

const TYPE_EMOJI: Record<string, string> = {
  walk: "🚶",
  strength: "🏋️",
  cardio: "🏃",
  other: "✨",
};

function formatEntryTime(value: string) {
  if (value.length >= 16) {
    return value.slice(11, 16);
  }
  return value;
}
