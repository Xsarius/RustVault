//! Auto-categorization rule engine — evaluates JSONB conditions against
//! transactions and applies matching actions.
//!
//! # Condition format (JSONB)
//!
//! ```json
//! [
//!   { "field": "description_contains", "value": "spotify", "logic": "and" },
//!   { "field": "amount_range", "value": { "min": -50, "max": -1 }, "logic": "and" },
//!   { "field": "payee_equals", "value": "Spotify AB" }
//! ]
//! ```
//!
//! Supported condition fields:
//! - `description_contains` — case-insensitive substring match on description
//! - `description_regex` — regex match on description
//! - `payee_equals` — exact case-insensitive match on payee
//! - `payee_contains` — case-insensitive substring match on payee
//! - `amount_range` — `{ "min": number, "max": number }` (either optional)
//! - `account_id` — UUID match on account
//!
//! # Action format (JSONB)
//!
//! ```json
//! [
//!   { "type": "set_category", "value": "uuid-here" },
//!   { "type": "add_tags", "value": ["uuid1", "uuid2"] },
//!   { "type": "set_payee", "value": "Spotify" },
//!   { "type": "set_metadata", "value": { "key": "value" } }
//! ]
//! ```

use regex::Regex;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::rule::AutoRule;

/// A lightweight representation of a transaction for rule matching,
/// avoiding circular dependencies with the full `Transaction` model.
#[derive(Debug, Clone)]
pub struct MatchCandidate {
    /// Transaction description.
    pub description: String,
    /// Original bank description (if available).
    pub original_desc: Option<String>,
    /// Payee / merchant name.
    pub payee: Option<String>,
    /// Transaction amount.
    pub amount: Decimal,
    /// Account ID.
    pub account_id: Uuid,
}

/// A single condition within a rule.
#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct RuleCondition {
    /// Condition field type.
    pub field: String,
    /// Condition value (type depends on field).
    pub value: serde_json::Value,
    /// Logic connector: "and" (default) or "or".
    #[serde(default = "default_logic")]
    pub logic: String,
}

fn default_logic() -> String {
    "and".into()
}

/// A single action to apply when a rule matches.
#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct RuleAction {
    /// Action type.
    #[serde(rename = "type")]
    pub action_type: String,
    /// Action value.
    pub value: serde_json::Value,
}

/// Result of applying rules to a transaction.
#[derive(Debug, Clone, Default, Serialize, utoipa::ToSchema)]
pub struct RuleApplicationResult {
    /// Category ID to set (from first matching rule with set_category).
    pub category_id: Option<Uuid>,
    /// Tag IDs to add.
    pub tag_ids: Vec<Uuid>,
    /// Payee to set / normalize.
    pub payee: Option<String>,
    /// Metadata fields to merge.
    pub metadata: serde_json::Map<String, serde_json::Value>,
    /// The rules that matched (by ID and name).
    pub matched_rules: Vec<(Uuid, String)>,
}

/// Check whether a single rule matches the given candidate.
pub fn evaluate_rule(rule: &AutoRule, candidate: &MatchCandidate) -> bool {
    if !rule.is_enabled {
        return false;
    }

    let conditions: Vec<RuleCondition> = match serde_json::from_value(rule.conditions.clone()) {
        Ok(c) => c,
        Err(_) => return false,
    };

    if conditions.is_empty() {
        return false;
    }

    // Split conditions into AND and OR groups.
    // All AND conditions must pass. At least one OR condition must pass (if any exist).
    let mut and_conditions = Vec::new();
    let mut or_conditions = Vec::new();

    for cond in &conditions {
        if cond.logic.eq_ignore_ascii_case("or") {
            or_conditions.push(cond);
        } else {
            and_conditions.push(cond);
        }
    }

    let and_pass = and_conditions
        .iter()
        .all(|c| evaluate_condition(c, candidate));
    let or_pass = or_conditions.is_empty()
        || or_conditions
            .iter()
            .any(|c| evaluate_condition(c, candidate));

    and_pass && or_pass
}

/// Evaluate a single condition against a candidate.
fn evaluate_condition(condition: &RuleCondition, candidate: &MatchCandidate) -> bool {
    match condition.field.as_str() {
        "description_contains" => {
            let needle = condition.value.as_str().unwrap_or_default().to_lowercase();
            let haystack = candidate.description.to_lowercase();
            if haystack.contains(&needle) {
                return true;
            }
            // Also check original_desc
            if let Some(orig) = &candidate.original_desc {
                return orig.to_lowercase().contains(&needle);
            }
            false
        }
        "description_regex" => {
            let pattern = condition.value.as_str().unwrap_or_default();
            let re = match Regex::new(pattern) {
                Ok(r) => r,
                Err(_) => return false,
            };
            if re.is_match(&candidate.description) {
                return true;
            }
            if let Some(orig) = &candidate.original_desc {
                return re.is_match(orig);
            }
            false
        }
        "payee_equals" => {
            let expected = condition.value.as_str().unwrap_or_default().to_lowercase();
            candidate
                .payee
                .as_ref()
                .is_some_and(|p| p.to_lowercase() == expected)
        }
        "payee_contains" => {
            let needle = condition.value.as_str().unwrap_or_default().to_lowercase();
            candidate
                .payee
                .as_ref()
                .is_some_and(|p| p.to_lowercase().contains(&needle))
        }
        "amount_range" => {
            let obj = match condition.value.as_object() {
                Some(o) => o,
                None => return false,
            };
            let min = obj
                .get("min")
                .and_then(|v| v.as_f64())
                .map(|f| Decimal::try_from(f).unwrap_or(Decimal::MIN));
            let max = obj
                .get("max")
                .and_then(|v| v.as_f64())
                .map(|f| Decimal::try_from(f).unwrap_or(Decimal::MAX));

            if let Some(min_val) = min {
                if candidate.amount < min_val {
                    return false;
                }
            }
            if let Some(max_val) = max {
                if candidate.amount > max_val {
                    return false;
                }
            }
            true
        }
        "account_id" => {
            let id_str = condition.value.as_str().unwrap_or_default();
            match Uuid::parse_str(id_str) {
                Ok(id) => candidate.account_id == id,
                Err(_) => false,
            }
        }
        _ => false, // Unknown condition type — skip
    }
}

/// Apply all matching rules to a candidate, returning the combined result.
/// Rules are evaluated in priority order (lower priority number = higher priority).
pub fn apply_rules(rules: &[AutoRule], candidate: &MatchCandidate) -> RuleApplicationResult {
    let mut sorted: Vec<&AutoRule> = rules.iter().collect();
    sorted.sort_by_key(|r| r.priority);

    let mut result = RuleApplicationResult::default();

    for rule in sorted {
        if !evaluate_rule(rule, candidate) {
            continue;
        }

        result.matched_rules.push((rule.id, rule.name.clone()));

        let actions: Vec<RuleAction> = match serde_json::from_value(rule.actions.clone()) {
            Ok(a) => a,
            Err(_) => continue,
        };

        for action in &actions {
            match action.action_type.as_str() {
                "set_category" => {
                    // First matching rule wins for category
                    if result.category_id.is_none() {
                        if let Some(id_str) = action.value.as_str() {
                            if let Ok(id) = Uuid::parse_str(id_str) {
                                result.category_id = Some(id);
                            }
                        }
                    }
                }
                "add_tags" => {
                    if let Some(arr) = action.value.as_array() {
                        for v in arr {
                            if let Some(id_str) = v.as_str() {
                                if let Ok(id) = Uuid::parse_str(id_str) {
                                    if !result.tag_ids.contains(&id) {
                                        result.tag_ids.push(id);
                                    }
                                }
                            }
                        }
                    }
                }
                "set_payee" => {
                    if result.payee.is_none() {
                        if let Some(payee) = action.value.as_str() {
                            result.payee = Some(payee.to_string());
                        }
                    }
                }
                "set_metadata" => {
                    if let Some(obj) = action.value.as_object() {
                        for (k, v) in obj {
                            result
                                .metadata
                                .entry(k.clone())
                                .or_insert_with(|| v.clone());
                        }
                    }
                }
                _ => {} // Unknown action — skip
            }
        }
    }

    result
}

/// Suggest a rule based on a transaction's description/payee.
/// Returns pre-filled conditions and a suggested name.
pub fn suggest_rule(
    description: &str,
    payee: Option<&str>,
    amount: Decimal,
) -> (String, Vec<RuleCondition>) {
    let mut conditions = Vec::new();

    // Use payee if available, otherwise fall back to description
    if let Some(p) = payee {
        let name = format!("Auto: {p}");
        conditions.push(RuleCondition {
            field: "payee_contains".into(),
            value: serde_json::Value::String(p.to_string()),
            logic: "and".into(),
        });
        // Also add an amount range around the typical amount (±20%)
        let abs_amount = amount.abs();
        if abs_amount > Decimal::ZERO {
            let margin = abs_amount * Decimal::new(20, 2); // 20%
            let (min, max) = (amount - margin, amount + margin);
            conditions.push(RuleCondition {
                field: "amount_range".into(),
                value: serde_json::json!({
                    "min": min.to_string().parse::<f64>().unwrap_or(0.0),
                    "max": max.to_string().parse::<f64>().unwrap_or(0.0),
                }),
                logic: "and".into(),
            });
        }
        (name, conditions)
    } else {
        // Extract a keyword from the description
        let keyword = extract_keyword(description);
        let name = format!("Auto: {keyword}");
        conditions.push(RuleCondition {
            field: "description_contains".into(),
            value: serde_json::Value::String(keyword.clone()),
            logic: "and".into(),
        });
        (name, conditions)
    }
}

/// Extract a meaningful keyword from a bank transaction description.
fn extract_keyword(description: &str) -> String {
    // Take the longest word that isn't a common stop-word or number
    let stop_words = [
        "the",
        "a",
        "an",
        "of",
        "for",
        "to",
        "in",
        "on",
        "at",
        "by",
        "from",
        "with",
        "and",
        "or",
        "is",
        "was",
        "are",
        "has",
        "had",
        "have",
        "do",
        "does",
        "did",
        "will",
        "would",
        "can",
        "could",
        "may",
        "might",
        "shall",
        "should",
        "no",
        "not",
        "nor",
        "but",
        "yet",
        "so",
        "as",
        "if",
        "than",
        "that",
        "this",
        "it",
        "be",
        "been",
        "am",
        "its",
        "card",
        "payment",
        "direct",
        "debit",
        "credit",
        "transfer",
        "ref",
        "reference",
    ];

    description
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| {
            w.len() >= 3
                && !stop_words.contains(&w.to_lowercase().as_str())
                && !w.chars().all(|c| c.is_ascii_digit())
        })
        .max_by_key(|w| w.len())
        .unwrap_or(description)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rule(conditions: serde_json::Value, actions: serde_json::Value) -> AutoRule {
        AutoRule {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "test".into(),
            priority: 0,
            is_enabled: true,
            conditions,
            actions,
            metadata: serde_json::json!({}),
            created_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
        }
    }

    fn make_candidate() -> MatchCandidate {
        MatchCandidate {
            description: "SPOTIFY AB Payment".into(),
            original_desc: None,
            payee: Some("Spotify AB".into()),
            amount: Decimal::new(-999, 2), // -9.99
            account_id: Uuid::new_v4(),
        }
    }

    #[test]
    fn description_contains_matches() {
        let rule = make_rule(
            serde_json::json!([{ "field": "description_contains", "value": "spotify" }]),
            serde_json::json!([]),
        );
        assert!(evaluate_rule(&rule, &make_candidate()));
    }

    #[test]
    fn payee_equals_matches() {
        let rule = make_rule(
            serde_json::json!([{ "field": "payee_equals", "value": "spotify ab" }]),
            serde_json::json!([]),
        );
        assert!(evaluate_rule(&rule, &make_candidate()));
    }

    #[test]
    fn amount_range_matches() {
        let rule = make_rule(
            serde_json::json!([{ "field": "amount_range", "value": { "min": -20.0, "max": 0.0 } }]),
            serde_json::json!([]),
        );
        assert!(evaluate_rule(&rule, &make_candidate()));
    }

    #[test]
    fn amount_range_no_match() {
        let rule = make_rule(
            serde_json::json!([{ "field": "amount_range", "value": { "min": -5.0, "max": -1.0 } }]),
            serde_json::json!([]),
        );
        assert!(!evaluate_rule(&rule, &make_candidate()));
    }

    #[test]
    fn and_logic_all_must_pass() {
        let rule = make_rule(
            serde_json::json!([
                { "field": "payee_equals", "value": "spotify ab", "logic": "and" },
                { "field": "amount_range", "value": { "min": -100.0, "max": 0.0 }, "logic": "and" },
            ]),
            serde_json::json!([]),
        );
        assert!(evaluate_rule(&rule, &make_candidate()));
    }

    #[test]
    fn or_logic_any_passes() {
        let rule = make_rule(
            serde_json::json!([
                { "field": "payee_equals", "value": "WRONG", "logic": "or" },
                { "field": "description_contains", "value": "spotify", "logic": "or" },
            ]),
            serde_json::json!([]),
        );
        assert!(evaluate_rule(&rule, &make_candidate()));
    }

    #[test]
    fn apply_rules_merges_actions() {
        let cat_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let rules = vec![make_rule(
            serde_json::json!([{ "field": "payee_contains", "value": "spotify" }]),
            serde_json::json!([
                { "type": "set_category", "value": cat_id.to_string() },
                { "type": "add_tags", "value": [tag_id.to_string()] },
                { "type": "set_payee", "value": "Spotify" },
            ]),
        )];

        let result = apply_rules(&rules, &make_candidate());
        assert_eq!(result.category_id, Some(cat_id));
        assert_eq!(result.tag_ids, vec![tag_id]);
        assert_eq!(result.payee, Some("Spotify".into()));
        assert_eq!(result.matched_rules.len(), 1);
    }

    #[test]
    fn disabled_rule_does_not_match() {
        let mut rule = make_rule(
            serde_json::json!([{ "field": "payee_equals", "value": "spotify ab" }]),
            serde_json::json!([]),
        );
        rule.is_enabled = false;
        assert!(!evaluate_rule(&rule, &make_candidate()));
    }
}
