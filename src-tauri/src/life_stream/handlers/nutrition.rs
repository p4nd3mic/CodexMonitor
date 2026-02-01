use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use tokio::fs;

use crate::life_stream::types::{CardStatValue, EntityRef};

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct FoodEntity {
    pub(crate) name: String,
    pub(crate) calories: f64,
    pub(crate) protein: f64,
    pub(crate) carbs: f64,
    pub(crate) fat: f64,
    pub(crate) fiber: Option<f64>,
}

pub struct NutritionHandler {
    obsidian_root: String,
}

impl NutritionHandler {
    pub fn new(obsidian_root: &str) -> Self {
        Self {
            obsidian_root: obsidian_root.to_string(),
        }
    }

    pub async fn process(&self, input: &str, occurred_at: &str) -> Result<ProcessedMeal, String> {
        let foods = self.extract_foods(input).await?;

        let mut total_calories = 0.0;
        let mut total_protein = 0.0;
        let mut total_carbs = 0.0;
        let mut total_fat = 0.0;
        let mut total_fiber = 0.0;

        for food in &foods {
            total_calories += food.calories;
            total_protein += food.protein;
            total_carbs += food.carbs;
            total_fat += food.fat;
            total_fiber += food.fiber.unwrap_or(0.0);
        }

        let time = occurred_at.get(11..16).unwrap_or("??:??");
        let foods_empty = foods.is_empty();
        let title = if foods.len() == 1 {
            foods[0].name.clone()
        } else {
            format!("Meal at {}", time)
        };

        let mut stats = HashMap::new();
        stats.insert(
            "calories".to_string(),
            CardStatValue::Integer(total_calories.round() as i64),
        );
        stats.insert(
            "protein".to_string(),
            CardStatValue::String(format!("{}g", total_protein.round() as i64)),
        );
        stats.insert(
            "carbs".to_string(),
            CardStatValue::String(format!("{}g", total_carbs.round() as i64)),
        );
        stats.insert(
            "fat".to_string(),
            CardStatValue::String(format!("{}g", total_fat.round() as i64)),
        );
        if total_fiber > 0.0 {
            stats.insert(
                "fiber".to_string(),
                CardStatValue::String(format!("{}g", total_fiber.round() as i64)),
            );
        }

        let entities: Vec<EntityRef> = foods
            .iter()
            .map(|food| EntityRef {
                entity_type: "food".to_string(),
                id: None,
                name: food.name.clone(),
                link: Some(format!("[[Entities/Food/{}]]", food.name)),
            })
            .collect();

        Ok(ProcessedMeal {
            foods,
            title,
            subtitle: if foods_empty {
                Some(input.to_string())
            } else {
                Some(format!(
                    "{}cal, {}g protein",
                    total_calories.round() as i64,
                    total_protein.round() as i64
                ))
            },
            stats: Some(stats),
            entities: if entities.is_empty() {
                None
            } else {
                Some(entities)
            },
        })
    }

    async fn extract_foods(&self, input: &str) -> Result<Vec<FoodEntity>, String> {
        let food_dir = Path::new(&self.obsidian_root).join("Entities/Food");

        if !food_dir.exists() {
            return Ok(Vec::new());
        }

        let mut found = Vec::new();
        let lower_input = input.to_lowercase();

        let mut entries = fs::read_dir(&food_dir)
            .await
            .map_err(|e| format!("Failed to read food directory: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read entry: {}", e))?
        {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path).await {
                    if let Some(food) = self.parse_food_entity(&content) {
                        if lower_input.contains(&food.name.to_lowercase()) {
                            found.push(food);
                        }
                    }
                }
            }
        }

        Ok(found)
    }

    pub(crate) fn parse_food_entity(&self, content: &str) -> Option<FoodEntity> {
        if !content.starts_with("---") {
            return None;
        }

        let end = content[3..].find("---")?;
        let frontmatter = &content[3..3 + end];

        let mut name = None;
        let mut calories = None;
        let mut protein = None;
        let mut carbs = None;
        let mut fat = None;
        let mut fiber = None;

        for line in frontmatter.lines() {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "name" => name = Some(value.to_string()),
                "calories" => calories = value.parse().ok(),
                "protein" => protein = value.parse().ok(),
                "carbs" => carbs = value.parse().ok(),
                "fat" => fat = value.parse().ok(),
                "fiber" => fiber = value.parse().ok(),
                _ => {}
            }
        }

        Some(FoodEntity {
            name: name?,
            calories: calories?,
            protein: protein?,
            carbs: carbs?,
            fat: fat?,
            fiber,
        })
    }
}

pub struct ProcessedMeal {
    pub foods: Vec<FoodEntity>,
    pub title: String,
    pub subtitle: Option<String>,
    pub stats: Option<HashMap<String, CardStatValue>>,
    pub entities: Option<Vec<EntityRef>>,
}
