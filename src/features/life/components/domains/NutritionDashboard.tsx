import { useMemo } from "react";
import type { LifeTimeRange } from "../../types";
import { NUTRITION_GOALS } from "../../types";
import { useNutritionDashboard } from "../../hooks/useNutritionDashboard";
import { TimeRangeSelector } from "../shared/TimeRangeSelector";
import { FoodCard } from "../shared/FoodCard";

type NutritionDashboardProps = {
  workspaceId: string | null;
  range: LifeTimeRange;
  onRangeChange: (range: LifeTimeRange) => void;
};

export function NutritionDashboard({
  workspaceId,
  range,
  onRangeChange,
}: NutritionDashboardProps) {
  const { dashboard, loading, error, refresh } = useNutritionDashboard(
    workspaceId,
    range,
  );

  const stats = dashboard?.stats;
  const meals = dashboard?.meals ?? [];
  const trend = dashboard?.weeklyTrend;
  const mealTitle = range === "today" ? "Today's Meals" : "Meals";

  const caloriesValue = stats?.calories ?? 0;
  const calories = stats ? stats.calories.toFixed(0) : "--";
  const remaining = stats
    ? Math.max(0, NUTRITION_GOALS.calories - stats.calories)
    : 0;
  const macroRows = useMemo(
    () => [
      {
        label: "Protein",
        value: stats?.protein ?? 0,
        goal: NUTRITION_GOALS.protein,
        className: "nutrition-macro-fill--protein",
      },
      {
        label: "Carbs",
        value: stats?.carbs ?? 0,
        goal: NUTRITION_GOALS.carbs,
        className: "nutrition-macro-fill--carbs",
      },
      {
        label: "Fat",
        value: stats?.fat ?? 0,
        goal: NUTRITION_GOALS.fat,
        className: "nutrition-macro-fill--fat",
      },
      {
        label: "Fiber",
        value: stats?.fiber ?? 0,
        goal: NUTRITION_GOALS.fiber ?? 35,
        className: "nutrition-macro-fill--fiber",
      },
    ],
    [stats],
  );

  const trendRows = useMemo(() => {
    if (!trend) return [];
    const rows = Object.entries(trend).sort(([a], [b]) => a.localeCompare(b));
    const maxValue = Math.max(
      NUTRITION_GOALS.calories,
      ...rows.map(([, value]) => value ?? 0),
    );
    return rows.map(([date, value]) => ({
      date,
      value,
      percent: maxValue > 0 ? (value / maxValue) * 100 : 0,
    }));
  }, [trend]);

  return (
    <div className="life-dashboard life-nutrition-dashboard">
      <div className="life-dashboard-header">
        <div>
          <div className="life-dashboard-title">🍽️ Nutrition Dashboard</div>
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
        <div className="life-dashboard-status">Loading nutrition data…</div>
      )}

      {dashboard ? (
        <>
          <section className="life-card">
            <div className="nutrition-hero">
              <div>
                <div className="nutrition-hero-value">{calories}</div>
                <div className="nutrition-hero-label">calories</div>
                <div className="nutrition-hero-remaining">
                  {remaining.toFixed(0)} remaining
                </div>
              </div>
              <div className="life-stat-sub">
                Target: {NUTRITION_GOALS.calories} cal
                {caloriesValue > NUTRITION_GOALS.calories ? " · over goal" : ""}
              </div>
            </div>
          </section>

          <section className="life-card">
            <div className="life-section-title">Macros</div>
            {macroRows.map((macro) => {
              const percent =
                macro.goal > 0 ? Math.min(100, (macro.value / macro.goal) * 100) : 0;
              return (
                <div key={macro.label} className="nutrition-macro-bar">
                  <div className="nutrition-macro-label">{macro.label}</div>
                  <div className="nutrition-macro-track">
                    <div
                      className={`nutrition-macro-fill ${macro.className}`}
                      style={{ width: `${percent}%` }}
                    />
                  </div>
                  <div className="nutrition-macro-value">
                    {macro.value.toFixed(0)}g / {macro.goal}g
                  </div>
                </div>
              );
            })}
          </section>

          <section className="life-section">
            <div className="life-section-title">{mealTitle}</div>
            {meals.length ? (
              <div className="visual-card-grid">
                {meals.map((meal) => (
                  <FoodCard key={meal.id} {...meal} />
                ))}
              </div>
            ) : (
              <div className="life-dashboard-status">No meals logged in this range.</div>
            )}
          </section>

          <section className="life-section">
            <div className="life-section-title">Weekly Trends</div>
            {trendRows.length ? (
              <div className="life-bar-chart">
                {trendRows.map((row) => (
                  <div key={row.date} className="life-bar-row">
                    <div className="life-bar-label">{formatShortDate(row.date)}</div>
                    <div className="life-bar-track">
                      <div
                        className="life-bar-fill"
                        style={{ width: `${row.percent}%` }}
                      />
                    </div>
                    <div className="life-bar-value">{row.value.toFixed(0)} cal</div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="life-dashboard-status">No trends yet.</div>
            )}
          </section>
        </>
      ) : null}
    </div>
  );
}

function formatShortDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}
