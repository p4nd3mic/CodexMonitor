import { useMemo } from "react";
import type { ExerciseEntry, LifeTimeRange } from "../../types";
import { useExerciseDashboard } from "../../hooks/useExerciseDashboard";
import { TimeRangeSelector } from "../shared/TimeRangeSelector";
import { ActivityCard } from "../shared/ActivityCard";

const TYPE_EMOJI: Record<ExerciseEntry["type"], string> = {
  walk: "🚶",
  strength: "🏋️",
  cardio: "🏃",
  other: "✨",
};

type ExerciseDashboardProps = {
  workspaceId: string | null;
  range: LifeTimeRange;
  onRangeChange: (range: LifeTimeRange) => void;
};

export function ExerciseDashboard({
  workspaceId,
  range,
  onRangeChange,
}: ExerciseDashboardProps) {
  const { dashboard, loading, error, refresh } = useExerciseDashboard(
    workspaceId,
    range,
  );

  const stats = dashboard?.stats;
  const entries = dashboard?.entries ?? [];
  const byType = dashboard?.byType ?? {};
  const activityTitle = range === "today" ? "Today's Activity" : "Activity";

  const breakdown = useMemo(() => {
    const rows = Object.entries(byType).sort(
      (a, b) => (b[1] ?? 0) - (a[1] ?? 0),
    );
    const maxValue = Math.max(1, ...rows.map(([, value]) => value ?? 0));
    return rows.map(([type, count]) => ({
      type,
      count,
      percent: maxValue > 0 ? (count / maxValue) * 100 : 0,
    }));
  }, [byType]);

  const totals = useMemo(() => {
    const totalMinutes = entries.reduce(
      (sum, entry) => sum + (entry.duration ?? 0),
      0,
    );
    const moveCalories = entries.reduce(
      (sum, entry) => sum + (entry.miles ?? 0) * 100,
      0,
    );
    const derivedMove = moveCalories + (stats?.workoutCount ?? 0) * 150;
    const minutes =
      totalMinutes > 0 ? totalMinutes : (stats?.workoutCount ?? 0) * 30;
    return {
      move: derivedMove,
      minutes,
      miles: stats?.walkingMiles ?? 0,
    };
  }, [entries, stats]);

  const activityDots = useMemo(() => {
    const active = new Set(
      entries.map((entry) => entry.timestamp.slice(0, 10)),
    );
    const today = new Date();
    return Array.from({ length: 7 }, (_, index) => {
      const date = new Date(today);
      date.setDate(today.getDate() - (6 - index));
      const key = date.toISOString().slice(0, 10);
      return {
        key,
        isActive: active.has(key),
      };
    });
  }, [entries]);

  return (
    <div className="life-dashboard life-exercise-dashboard">
      <div className="life-dashboard-header">
        <div>
          <div className="life-dashboard-title">🏋️ Exercise Dashboard</div>
          <div className="life-dashboard-subtitle">
            {dashboard?.meta ? `${dashboard.meta.periodStart} → ${dashboard.meta.periodEnd}` : ""}
          </div>
        </div>
        <div className="life-dashboard-actions">
          <TimeRangeSelector value={range} onChange={onRangeChange} />
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

      {error && <div className="life-dashboard-error">{error}</div>}
      {loading && !dashboard && (
        <div className="life-dashboard-status">Loading exercise data…</div>
      )}

      {dashboard ? (
        <>
          <section className="life-card">
            <div className="life-section-title">
              Activity{" "}
              <span className="exercise-streak">
                🔥 {stats ? stats.currentStreak : 0} day streak
              </span>
            </div>
            <div className="exercise-activity-row">
              <div className="exercise-activity-label">Move</div>
              <div className="exercise-activity-track">
                <div
                  className="exercise-activity-fill exercise-activity-fill--move"
                  style={{ width: `${Math.min(100, (totals.move / 500) * 100)}%` }}
                />
              </div>
              <div className="exercise-activity-value">
                {totals.move.toFixed(0)} / 500 cal
              </div>
            </div>
            <div className="exercise-activity-row">
              <div className="exercise-activity-label">Exercise</div>
              <div className="exercise-activity-track">
                <div
                  className="exercise-activity-fill exercise-activity-fill--exercise"
                  style={{
                    width: `${Math.min(100, (totals.minutes / 30) * 100)}%`,
                  }}
                />
              </div>
              <div className="exercise-activity-value">
                {totals.minutes.toFixed(0)} / 30 min
              </div>
            </div>
            <div className="exercise-activity-row">
              <div className="exercise-activity-label">Miles</div>
              <div className="exercise-activity-track">
                <div
                  className="exercise-activity-fill exercise-activity-fill--miles"
                  style={{
                    width: `${Math.min(100, (totals.miles / 4) * 100)}%`,
                  }}
                />
              </div>
              <div className="exercise-activity-value">
                {totals.miles.toFixed(1)} / 4.0 mi
              </div>
            </div>
          </section>

          <section className="life-section">
            <div className="life-section-title">{activityTitle}</div>
            {entries.length ? (
              <div className="visual-card-grid">
                {entries.map((entry) => (
                  <ActivityCard key={entry.id} {...entry} />
                ))}
              </div>
            ) : (
              <div className="life-dashboard-status">No exercise logged yet.</div>
            )}
          </section>

          <section className="life-section">
            <div className="life-section-title">This Week</div>
            <div className="exercise-activity-dots">
              {activityDots.map((dot) => (
                <div
                  key={dot.key}
                  className={`exercise-activity-dot${dot.isActive ? " is-active" : ""}`}
                />
              ))}
            </div>
          </section>

          <section className="life-section">
            <div className="life-section-title">Activity Breakdown</div>
            {breakdown.length ? (
              <div className="life-bar-chart">
                {breakdown.map((row) => (
                  <div key={row.type} className="life-bar-row">
                    <div className="life-bar-label">
                      {TYPE_EMOJI[row.type as ExerciseEntry["type"]] ?? "✨"} {row.type}
                    </div>
                    <div className="life-bar-track">
                      <div
                        className="life-bar-fill"
                        style={{ width: `${row.percent}%` }}
                      />
                    </div>
                    <div className="life-bar-value">{row.count}</div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="life-dashboard-status">No activity yet.</div>
            )}
          </section>
        </>
      ) : null}
    </div>
  );
}
