use super::handlers::nutrition::NutritionHandler;
use super::obsidian::{normalize_time, parse_table_entry};
use super::service::{detect_intent, truncate};
use super::{CardType, DomainId};
use chrono::NaiveDate;

#[test]
fn detect_intent_meal_delivery_media() {
    let (card_type, domain, emoji) = detect_intent("Had eggs and bacon for breakfast");
    assert_eq!(card_type, CardType::Meal);
    assert_eq!(domain, DomainId::Nutrition);
    assert_eq!(emoji, "🍽️");

    let (card_type, domain, emoji) = detect_intent("Started delivery shift with DoorDash order");
    assert_eq!(card_type, CardType::DeliveryOrder);
    assert_eq!(domain, DomainId::Delivery);
    assert_eq!(emoji, "🚗");

    let (card_type, domain, emoji) = detect_intent("Watched a movie last night");
    assert_eq!(card_type, CardType::MediaAdd);
    assert_eq!(domain, DomainId::Media);
    assert_eq!(emoji, "🎬");
}

#[test]
fn normalize_time_handles_am_pm() {
    assert_eq!(normalize_time("5:58pm").as_deref(), Some("17:58"));
    assert_eq!(normalize_time("12:00am").as_deref(), Some("00:00"));
    assert_eq!(normalize_time("12:01pm").as_deref(), Some("12:01"));
    assert_eq!(normalize_time("07:05").as_deref(), Some("07:05"));
}

#[test]
fn truncate_respects_utf8_boundaries() {
    let input = "café latte";
    assert_eq!(truncate(input, 4), "café");
    assert_eq!(truncate("short", 10), "short");
}

#[test]
fn parse_food_entity_reads_frontmatter() {
    let handler = NutritionHandler::new("/tmp");
    let content = r#"---
name: Sardines
calories: 180
protein: 20
carbs: 0
fat: 10
fiber: 1
---
Body text
"#;

    let food = handler
        .parse_food_entity(content)
        .expect("should parse food frontmatter");
    assert_eq!(food.name, "Sardines");
    assert_eq!(food.calories, 180.0);
    assert_eq!(food.protein, 20.0);
    assert_eq!(food.carbs, 0.0);
    assert_eq!(food.fat, 10.0);
    assert_eq!(food.fiber, Some(1.0));
}

#[test]
fn parse_table_entry_extracts_task() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 21).expect("valid date");
    let line = "| -- | 5:58pm 🚗 Started dinner shift | + | <!--task:2026-01-21-1758-delivery-->";
    let parsed = parse_table_entry(line, date).expect("should parse entry");
    assert_eq!(parsed.task_id, "2026-01-21-1758-delivery");
    assert_eq!(parsed.occurred_at, "2026-01-21T17:58:00");
    assert_eq!(parsed.title, "🚗 Started dinner shift");
}
