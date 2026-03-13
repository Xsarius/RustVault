//! QIF (Quicken Interchange Format) parser.

use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

use crate::ImportResult;
use crate::error::ImportError;
use crate::raw::{ColumnMapping, ImportParser, RawTransaction};

use super::date::parse_date;

/// Parser for QIF files.
pub struct QifParser;

impl ImportParser for QifParser {
    fn name(&self) -> &str {
        "QIF"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["qif"]
    }

    fn parse(
        &self,
        data: &[u8],
        mapping: Option<&ColumnMapping>,
    ) -> ImportResult<Vec<RawTransaction>> {
        let text = std::str::from_utf8(data)
            .map_err(|e| ImportError::ParseFailed(format!("invalid UTF-8: {e}")))?;

        let date_format = mapping.and_then(|m| m.date_format.as_deref());
        let mut transactions = Vec::new();
        let mut current = RecordBuilder::default();

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // The `!Type:` header indicates account type — skip it.
            if line.starts_with('!') {
                continue;
            }

            // `^` marks the end of a record.
            if line == "^" {
                if let Some(txn) = current.build(date_format)? {
                    transactions.push(txn);
                }
                current = RecordBuilder::default();
                continue;
            }

            let code = line.as_bytes()[0];
            let value = &line[1..];

            match code {
                b'D' => current.date = Some(value.to_owned()),
                b'T' | b'U' => current.amount = Some(value.to_owned()),
                b'P' => current.payee = Some(value.to_owned()),
                b'M' => current.memo = Some(value.to_owned()),
                b'L' => current.category = Some(value.to_owned()),
                b'N' => current.number = Some(value.to_owned()),
                b'C' => current.cleared = Some(value.to_owned()),
                b'A' => current.address.push(value.to_owned()),
                _ => {} // Ignore unknown fields.
            }
        }

        // Handle final record if file doesn't end with `^`.
        if current.has_data() {
            if let Some(txn) = current.build(date_format)? {
                transactions.push(txn);
            }
        }

        if transactions.is_empty() {
            return Err(ImportError::ParseFailed(
                "no transactions found in QIF file".into(),
            ));
        }

        Ok(transactions)
    }

    fn detect(&self, data: &[u8], extension: Option<&str>) -> f32 {
        let ext_match = extension
            .map(|e| e.eq_ignore_ascii_case("qif"))
            .unwrap_or(false);

        let content_match = std::str::from_utf8(data)
            .ok()
            .map(|text| {
                let trimmed = text.trim_start();
                trimmed.starts_with("!Type:") || trimmed.starts_with("!Account")
            })
            .unwrap_or(false);

        match (ext_match, content_match) {
            (true, true) => 0.95,
            (false, true) => 0.85,
            (true, false) => 0.5,
            (false, false) => 0.0,
        }
    }
}

/// Builder that accumulates QIF record fields.
#[derive(Default)]
struct RecordBuilder {
    date: Option<String>,
    amount: Option<String>,
    payee: Option<String>,
    memo: Option<String>,
    category: Option<String>,
    number: Option<String>,
    cleared: Option<String>,
    address: Vec<String>,
}

impl RecordBuilder {
    fn has_data(&self) -> bool {
        self.date.is_some() || self.amount.is_some()
    }

    fn build(&self, date_format: Option<&str>) -> ImportResult<Option<RawTransaction>> {
        let date_str = match &self.date {
            Some(d) => d,
            None => return Ok(None),
        };
        let amount_str = match &self.amount {
            Some(a) => a,
            None => return Ok(None),
        };

        let date = parse_date(date_str, date_format)?;
        let amount = parse_amount(amount_str)?;

        let description = match (&self.payee, &self.memo) {
            (Some(p), Some(m)) => format!("{p} — {m}"),
            (Some(p), None) => p.clone(),
            (None, Some(m)) => m.clone(),
            (None, None) => String::new(),
        };

        let mut metadata = HashMap::new();
        if let Some(cat) = &self.category {
            metadata.insert(
                "qif_category".into(),
                serde_json::Value::String(cat.clone()),
            );
        }
        if let Some(clr) = &self.cleared {
            metadata.insert("qif_cleared".into(), serde_json::Value::String(clr.clone()));
        }
        if !self.address.is_empty() {
            metadata.insert(
                "qif_address".into(),
                serde_json::Value::String(self.address.join(", ")),
            );
        }

        Ok(Some(RawTransaction {
            date,
            amount,
            currency: None,
            description,
            payee: self.payee.clone(),
            reference: self.number.clone(),
            metadata,
        }))
    }
}

/// Parse a QIF amount string (may use commas for thousands or decimals).
fn parse_amount(s: &str) -> ImportResult<Decimal> {
    let cleaned: String = s
        .chars()
        .filter(|c| *c == '-' || *c == '.' || c.is_ascii_digit())
        .collect();

    Decimal::from_str(&cleaned)
        .map_err(|e| ImportError::ParseFailed(format!("invalid amount '{s}': {e}")))
}

#[cfg(test)]
mod tests {
    use super::QifParser;
    use crate::raw::ImportParser;

    #[test]
    fn parses_qif_record() {
        let parser = QifParser;
        let data = b"!Type:Bank\nD2026-03-01\nT-12.34\nPCoffee Shop\nMMorning coffee\nNREF1\n^\n";

        let rows = parser.parse(data, None).expect("qif parse should succeed");
        assert_eq!(rows.len(), 1);
        assert!(rows[0].description.contains("Coffee Shop"));
        assert_eq!(rows[0].amount.to_string(), "-12.34");
        assert_eq!(rows[0].reference.as_deref(), Some("REF1"));
    }
}
