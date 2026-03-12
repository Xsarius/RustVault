//! JSON transaction parser with flexible field mapping.

use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

use crate::error::ImportError;
use crate::raw::{ColumnMapping, ImportParser, RawTransaction};
use crate::ImportResult;

use super::date::parse_date;

/// Parser for JSON transaction files.
///
/// Expects either a top-level JSON array of objects, or an object with a single
/// array field containing the transactions.
pub struct JsonParser;

impl ImportParser for JsonParser {
    fn name(&self) -> &str {
        "JSON"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["json"]
    }

    fn parse(
        &self,
        data: &[u8],
        mapping: Option<&ColumnMapping>,
    ) -> ImportResult<Vec<RawTransaction>> {
        let text = std::str::from_utf8(data)
            .map_err(|e| ImportError::ParseFailed(format!("invalid UTF-8: {e}")))?;

        let root: Value = serde_json::from_str(text)
            .map_err(|e| ImportError::ParseFailed(format!("invalid JSON: {e}")))?;

        let items = extract_array(&root)?;

        let field_map = mapping.map(|m| &m.fields);
        let date_format = mapping.and_then(|m| m.date_format.as_deref());

        // If no mapping provided, try auto-detection from the first object.
        let auto_map = if field_map.map(|f| f.is_empty()).unwrap_or(true) {
            items
                .first()
                .and_then(|v| v.as_object())
                .map(auto_detect_fields)
        } else {
            None
        };

        let effective_map = match (&auto_map, field_map) {
            (Some(am), _) => am,
            (_, Some(fm)) if !fm.is_empty() => fm,
            _ => {
                return Err(ImportError::MappingRequired(
                    "field mapping required for JSON import".into(),
                ));
            }
        };

        let mut transactions = Vec::new();

        for item in items {
            let obj = item
                .as_object()
                .ok_or_else(|| ImportError::ParseFailed("array item is not an object".into()))?;

            if let Some(txn) = parse_object(obj, effective_map, date_format)? {
                transactions.push(txn);
            }
        }

        if transactions.is_empty() {
            return Err(ImportError::ParseFailed(
                "no transactions found in JSON file".into(),
            ));
        }

        Ok(transactions)
    }

    fn detect(&self, data: &[u8], extension: Option<&str>) -> f32 {
        let ext_match = extension
            .map(|e| e.eq_ignore_ascii_case("json"))
            .unwrap_or(false);

        let content_match = std::str::from_utf8(data)
            .ok()
            .map(|text| {
                let trimmed = text.trim_start();
                trimmed.starts_with('[') || trimmed.starts_with('{')
            })
            .unwrap_or(false);

        match (ext_match, content_match) {
            (true, true) => 0.85,
            (true, false) => 0.5,
            (false, true) => 0.1, // JSON is too generic to claim high confidence on content alone.
            (false, false) => 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the transaction array from the root value.
fn extract_array(root: &Value) -> ImportResult<&Vec<Value>> {
    match root {
        Value::Array(arr) => Ok(arr),
        Value::Object(obj) => {
            // Find the first field that is an array.
            for (_key, value) in obj {
                if let Value::Array(arr) = value {
                    return Ok(arr);
                }
            }
            Err(ImportError::ParseFailed(
                "JSON object contains no array field".into(),
            ))
        }
        _ => Err(ImportError::ParseFailed(
            "expected JSON array or object at root".into(),
        )),
    }
}

/// Field synonyms for auto-detection.
const FIELD_SYNONYMS: &[(&str, &[&str])] = &[
    ("date", &["date", "datum", "data", "booking_date", "bookingDate", "transaction_date", "transactionDate"]),
    ("amount", &["amount", "betrag", "kwota", "value", "sum", "total"]),
    ("description", &["description", "details", "narrative", "text", "memo", "note", "verwendungszweck"]),
    ("payee", &["payee", "recipient", "merchant", "name", "counterparty"]),
    ("currency", &["currency", "ccy"]),
    ("reference", &["reference", "ref", "id", "transaction_id", "transactionId", "fitid"]),
];

fn auto_detect_fields(obj: &serde_json::Map<String, Value>) -> HashMap<String, String> {
    let keys: Vec<String> = obj.keys().cloned().collect();
    let mut mapping = HashMap::new();

    for (logical, synonyms) in FIELD_SYNONYMS {
        for key in &keys {
            let lower = key.to_ascii_lowercase();
            if synonyms.iter().any(|s| lower == *s || lower.contains(s)) {
                mapping.insert((*logical).to_owned(), key.clone());
                break;
            }
        }
    }

    mapping
}

fn parse_object(
    obj: &serde_json::Map<String, Value>,
    fields: &HashMap<String, String>,
    date_format: Option<&str>,
) -> ImportResult<Option<RawTransaction>> {
    let date_key = match fields.get("date") {
        Some(k) => k,
        None => return Ok(None),
    };
    let amount_key = match fields.get("amount") {
        Some(k) => k,
        None => return Ok(None),
    };

    let date_val = match obj.get(date_key) {
        Some(v) => value_to_string(v),
        None => return Ok(None),
    };
    let amount_val = match obj.get(amount_key) {
        Some(v) => v,
        None => return Ok(None),
    };

    if date_val.is_empty() {
        return Ok(None);
    }

    let date = parse_date(&date_val, date_format)?;
    let amount = value_to_decimal(amount_val)?;

    let description = fields
        .get("description")
        .and_then(|k| obj.get(k))
        .map(value_to_string)
        .unwrap_or_default();

    let payee = fields
        .get("payee")
        .and_then(|k| obj.get(k))
        .map(value_to_string);

    let currency = fields
        .get("currency")
        .and_then(|k| obj.get(k))
        .map(value_to_string);

    let reference = fields
        .get("reference")
        .and_then(|k| obj.get(k))
        .map(value_to_string);

    // Preserve all extra fields as metadata.
    let mapped_keys: std::collections::HashSet<&str> = fields.values().map(|s| s.as_str()).collect();
    let mut metadata = HashMap::new();
    for (k, v) in obj {
        if !mapped_keys.contains(k.as_str()) {
            metadata.insert(k.clone(), v.clone());
        }
    }

    Ok(Some(RawTransaction {
        date,
        amount,
        currency,
        description,
        payee,
        reference,
        metadata,
    }))
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn value_to_decimal(v: &Value) -> ImportResult<Decimal> {
    match v {
        Value::Number(n) => {
            let s = n.to_string();
            Decimal::from_str(&s)
                .map_err(|e| ImportError::ParseFailed(format!("invalid amount {s}: {e}")))
        }
        Value::String(s) => {
            let cleaned: String = s
                .replace(',', ".")
                .chars()
                .filter(|c| *c == '-' || *c == '.' || c.is_ascii_digit())
                .collect();
            Decimal::from_str(&cleaned)
                .map_err(|e| ImportError::ParseFailed(format!("invalid amount '{s}': {e}")))
        }
        _ => Err(ImportError::ParseFailed(format!(
            "expected number or string for amount, got: {v}"
        ))),
    }
}
