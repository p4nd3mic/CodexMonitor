import type { MealEntry } from "../../types";

export type FoodCardProps = MealEntry;

export function FoodCard({
  description,
  mealType,
  timestamp,
  imageUrl,
  estimatedCalories,
  protein,
  carbs,
  fat,
  fiber,
}: FoodCardProps) {
  return (
    <div className="food-card">
      <div className="food-card__image">
        {imageUrl ? (
          <img src={imageUrl} alt={description} loading="lazy" />
        ) : (
          <div className="food-card__placeholder">
            <span>{MEAL_EMOJI[mealType] ?? "🍽️"}</span>
          </div>
        )}
      </div>
      <div className="food-card__body">
        <div className="food-card__title">{description}</div>
        <div className="food-card__meta">
          {estimatedCalories !== undefined
            ? `${estimatedCalories.toFixed(0)} cal`
            : "Calories --"}
          {protein ? ` · P ${protein.toFixed(0)}g` : ""}
          {carbs ? ` · C ${carbs.toFixed(0)}g` : ""}
          {fat ? ` · F ${fat.toFixed(0)}g` : ""}
          {fiber ? ` · Fi ${fiber.toFixed(0)}g` : ""}
        </div>
        <div className="food-card__time">{formatMealTime(timestamp)}</div>
      </div>
    </div>
  );
}

const MEAL_EMOJI: Record<string, string> = {
  breakfast: "🌅",
  lunch: "🌞",
  dinner: "🌙",
  snack: "🍪",
};

function formatMealTime(value: string) {
  if (value.length >= 16) {
    return value.slice(11, 16);
  }
  return value;
}
