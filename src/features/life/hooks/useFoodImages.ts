import { useMemo } from "react";
import type { MealEntry } from "../types";

export function useFoodImages(meals: MealEntry[] = []) {
  return useMemo(() => {
    const map = new Map<string, string>();
    meals.forEach((meal) => {
      if (meal.imageUrl) {
        map.set(meal.id, meal.imageUrl);
      }
    });
    return map;
  }, [meals]);
}
