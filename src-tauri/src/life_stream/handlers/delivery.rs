use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use tokio::fs;

use crate::life_stream::types::EntityRef;

#[derive(Debug, Clone, Deserialize)]
struct MerchantIndex {
    merchants: HashMap<String, MerchantInfo>,
}

#[derive(Debug, Clone, Deserialize)]
struct MerchantInfo {
    tier: Option<String>,
    #[serde(rename = "avgWaitMins")]
    avg_wait_mins: Option<f64>,
    #[serde(rename = "avgPerMile")]
    avg_per_mile: Option<f64>,
    #[serde(rename = "lastSeen")]
    last_seen: Option<String>,
    notes: Option<Vec<String>>,
}

pub struct DeliveryHandler {
    obsidian_root: String,
}

impl DeliveryHandler {
    pub fn new(obsidian_root: &str) -> Self {
        Self {
            obsidian_root: obsidian_root.to_string(),
        }
    }

    pub async fn process(
        &self,
        input: &str,
        occurred_at: &str,
    ) -> Result<ProcessedDelivery, String> {
        let merchants = self.load_merchants().await?;
        let merchant_match = Self::find_best_merchant(input, &merchants);

        let payout = Self::parse_money(input);
        let miles = Self::find_number_before_keyword(input, &["mile", "miles", "mi"]);
        let acceptance_rate = Self::find_number_after_keyword(input, &["ar", "acceptance"]);
        let per_mile = match (payout, miles) {
            (Some(p), Some(m)) if m > 0.0 => Some(p / m),
            _ => None,
        };

        let time_label = occurred_at.get(11..16).unwrap_or("??:??");
        let lower = input.to_lowercase();

        let title = if let Some((name, _)) = &merchant_match {
            format!("{} order", name)
        } else if lower.contains("shift") {
            format!("Delivery shift {}", time_label)
        } else if lower.contains("order") {
            "Delivery order".to_string()
        } else {
            "Delivery log".to_string()
        };

        let subtitle = build_delivery_subtitle(payout, miles, per_mile);
        let summary = if subtitle.is_some() {
            None
        } else {
            Some(input.to_string())
        };

        let mut stats = HashMap::new();
        if let Some(value) = payout {
            stats.insert(
                "payout".to_string(),
                serde_json::json!(format!("${:.2}", value)),
            );
        }
        if let Some(value) = miles {
            stats.insert(
                "miles".to_string(),
                serde_json::json!(format!("{:.1} mi", value)),
            );
        }
        if let Some(value) = per_mile {
            stats.insert(
                "per_mile".to_string(),
                serde_json::json!(format!("${:.2}/mi", value)),
            );
        }
        if let Some(value) = acceptance_rate {
            stats.insert(
                "ar".to_string(),
                serde_json::json!(format!("{:.0}%", value)),
            );
        }

        if let Some((_name, info)) = &merchant_match {
            if let Some(tier) = &info.tier {
                stats.insert("merchant_tier".to_string(), serde_json::json!(tier));
            }
            if let Some(wait) = info.avg_wait_mins {
                stats.insert(
                    "avg_wait".to_string(),
                    serde_json::json!(format!("{:.0} min", wait)),
                );
            }
            if let Some(avg_per_mile) = info.avg_per_mile {
                stats.insert(
                    "merchant_avg".to_string(),
                    serde_json::json!(format!("${:.2}/mi", avg_per_mile)),
                );
            }
            if let Some(last_seen) = &info.last_seen {
                stats.insert("last_seen".to_string(), serde_json::json!(last_seen));
            }
        }

        if stats.is_empty() {
            stats.insert("entry".to_string(), serde_json::json!("delivery"));
        }

        let entities = merchant_match.map(|(name, _)| {
            vec![EntityRef {
                entity_type: "delivery_merchant".to_string(),
                id: None,
                name: name.clone(),
                link: Some(format!("[[Entities/Delivery/{}]]", name)),
            }]
        });

        Ok(ProcessedDelivery {
            title,
            subtitle,
            summary,
            stats: Some(stats),
            entities,
        })
    }

    async fn load_merchants(&self) -> Result<HashMap<String, MerchantInfo>, String> {
        let index_path = Path::new(&self.obsidian_root).join("Indexes/delivery.merchants.v1.json");
        if !index_path.exists() {
            return Ok(HashMap::new());
        }
        let content = fs::read_to_string(&index_path)
            .await
            .map_err(|err| format!("Failed to read merchant index: {err}"))?;
        let index: MerchantIndex = serde_json::from_str(&content)
            .map_err(|err| format!("Failed to parse merchant index: {err}"))?;
        Ok(index.merchants)
    }

    fn find_best_merchant<'a>(
        input: &str,
        merchants: &'a HashMap<String, MerchantInfo>,
    ) -> Option<(String, MerchantInfo)> {
        let lower = input.to_lowercase();
        let mut best: Option<(String, MerchantInfo)> = None;
        let mut best_len = 0;

        for (name, info) in merchants {
            let name_lower = name.to_lowercase();
            if lower.contains(&name_lower) && name.len() > best_len {
                best = Some((name.clone(), info.clone()));
                best_len = name.len();
            }
        }

        best
    }

    fn parse_money(input: &str) -> Option<f64> {
        let lower = input.to_lowercase();
        if let Some(idx) = lower.find('$') {
            let after = &lower[idx + 1..];
            if let Some(value) = extract_first_number(after) {
                return Some(value);
            }
        }
        if lower.contains("dollar") || lower.contains("bucks") {
            return Self::find_number_before_keyword(&lower, &["dollar", "bucks"]);
        }
        None
    }

    fn find_number_before_keyword(input: &str, keywords: &[&str]) -> Option<f64> {
        let lower = input.to_lowercase();
        for keyword in keywords {
            if let Some(idx) = lower.find(keyword) {
                let before = &lower[..idx];
                if let Some(value) = extract_last_number(before) {
                    return Some(value);
                }
            }
        }
        None
    }

    fn find_number_after_keyword(input: &str, keywords: &[&str]) -> Option<f64> {
        let lower = input.to_lowercase();
        for keyword in keywords {
            if let Some(idx) = lower.find(keyword) {
                let after = &lower[idx + keyword.len()..];
                if let Some(value) = extract_first_number(after) {
                    return Some(value);
                }
            }
        }
        None
    }
}

pub struct ProcessedDelivery {
    pub title: String,
    pub subtitle: Option<String>,
    pub summary: Option<String>,
    pub stats: Option<HashMap<String, serde_json::Value>>,
    pub entities: Option<Vec<EntityRef>>,
}

fn build_delivery_subtitle(
    payout: Option<f64>,
    miles: Option<f64>,
    per_mile: Option<f64>,
) -> Option<String> {
    match (payout, miles, per_mile) {
        (Some(p), Some(m), Some(pm)) => Some(format!("${:.2} • {:.1} mi • ${:.2}/mi", p, m, pm)),
        (Some(p), Some(m), None) => Some(format!("${:.2} • {:.1} mi", p, m)),
        (Some(p), None, None) => Some(format!("${:.2} payout", p)),
        (None, Some(m), None) => Some(format!("{:.1} mi", m)),
        _ => None,
    }
}

fn extract_numbers(input: &str) -> Vec<f64> {
    let mut values = Vec::new();
    let mut current = String::new();

    for ch in input.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            current.push(ch);
        } else if !current.is_empty() {
            if let Ok(value) = current.parse::<f64>() {
                values.push(value);
            }
            current.clear();
        }
    }

    if !current.is_empty() {
        if let Ok(value) = current.parse::<f64>() {
            values.push(value);
        }
    }

    values
}

fn extract_first_number(input: &str) -> Option<f64> {
    extract_numbers(input).into_iter().next()
}

fn extract_last_number(input: &str) -> Option<f64> {
    extract_numbers(input).into_iter().last()
}
