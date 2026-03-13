//! PDF statement parser.
//!
//! Uses text extraction and line heuristics to recover transaction rows from
//! bank statement PDFs. Because statement layouts vary per bank, this parser
//! focuses on robust date/amount extraction and keeps the original line in
//! metadata for later inspection.

use std::collections::HashMap;
use std::str::FromStr;

use regex::Regex;
use rust_decimal::Decimal;

use crate::ImportResult;
use crate::error::ImportError;
use crate::raw::{ColumnMapping, ImportParser, RawTransaction};

use super::date::parse_date;

/// Parser for PDF statements.
pub struct PdfParser;

impl ImportParser for PdfParser {
    fn name(&self) -> &str {
        "PDF"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["pdf"]
    }

    fn parse(
        &self,
        data: &[u8],
        _mapping: Option<&ColumnMapping>,
    ) -> ImportResult<Vec<RawTransaction>> {
        let text = pdf_extract::extract_text_from_mem(data).map_err(|e| {
            ImportError::ParseFailed(format!("failed to extract text from PDF: {e}"))
        })?;

        parse_transactions_from_text(&text)
    }

    fn detect(&self, data: &[u8], extension: Option<&str>) -> f32 {
        let ext_match = extension
            .map(|e| e.eq_ignore_ascii_case("pdf"))
            .unwrap_or(false);
        let magic_match = data.len() >= 5 && data[..5] == [0x25, 0x50, 0x44, 0x46, 0x2D];

        match (ext_match, magic_match) {
            (true, true) => 0.95,
            (true, false) => 0.6,
            (false, true) => 0.8,
            (false, false) => 0.0,
        }
    }
}

fn parse_transactions_from_text(text: &str) -> ImportResult<Vec<RawTransaction>> {
    let mut rows = Vec::new();

    for (line_no, line) in text.lines().enumerate() {
        if let Some(txn) = parse_line(line, line_no + 1)? {
            rows.push(txn);
        }
    }

    if rows.is_empty() {
        return Err(ImportError::ParseFailed(
            "no transaction-like rows found in PDF text".into(),
        ));
    }

    Ok(rows)
}

fn parse_line(line: &str, line_no: usize) -> ImportResult<Option<RawTransaction>> {
    let normalized = normalize_whitespace(line);
    if normalized.len() < 8 {
        return Ok(None);
    }

    let tokens: Vec<&str> = normalized.split(' ').collect();
    if tokens.len() < 3 {
        return Ok(None);
    }

    // Find first token that parses as a date.
    let mut date_idx = None;
    let mut parsed_date = None;
    for (i, token) in tokens.iter().enumerate() {
        if let Ok(d) = parse_date(token, None) {
            date_idx = Some(i);
            parsed_date = Some(d);
            break;
        }
    }

    let (date_idx, date) = match (date_idx, parsed_date) {
        (Some(i), Some(d)) => (i, d),
        _ => return Ok(None),
    };

    // Find amount-like token from the end of line.
    let mut amount_idx = None;
    let mut amount = None;
    for i in (date_idx + 1..tokens.len()).rev() {
        if let Some(parsed) = parse_amount_token(tokens[i]) {
            amount_idx = Some(i);
            amount = Some(parsed);
            break;
        }
    }

    let (amount_idx, amount) = match (amount_idx, amount) {
        (Some(i), Some(v)) => (i, v),
        _ => return Ok(None),
    };

    if amount_idx <= date_idx + 1 {
        return Ok(None);
    }

    let description = tokens[date_idx + 1..amount_idx].join(" ").trim().to_owned();
    if description.is_empty() {
        return Ok(None);
    }

    let mut metadata = HashMap::new();
    metadata.insert("source".to_owned(), serde_json::json!("pdf"));
    metadata.insert("line".to_owned(), serde_json::json!(line_no));
    metadata.insert("raw_line".to_owned(), serde_json::json!(normalized));

    Ok(Some(RawTransaction {
        date,
        amount,
        currency: None,
        description,
        payee: None,
        reference: None,
        metadata,
    }))
}

fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_amount_token(token: &str) -> Option<Decimal> {
    // Supports variants like:
    //  -123.45
    //  +123,45
    //  (123.45)
    //  1,234.56
    //  1 234,56
    //  123.45-
    static AMOUNT_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = AMOUNT_RE.get_or_init(|| {
        Regex::new(r"^[+-]?\(?\p{Sc}?[0-9][0-9\s.,]*\)?-?$").expect("amount regex must compile")
    });

    if !re.is_match(token) {
        return None;
    }

    let mut s = token.trim().replace(' ', "");

    let trailing_minus = s.ends_with('-');
    if trailing_minus {
        s.pop();
    }

    let paren_negative = s.starts_with('(') && s.ends_with(')');
    if paren_negative {
        s = s.trim_start_matches('(').trim_end_matches(')').to_owned();
    }

    // Remove leading currency symbols and explicit '+' sign.
    s = s
        .trim_start_matches(|c: char| {
            c.is_ascii_whitespace() || c == '+' || c == '$' || c == '€' || c == '£'
        })
        .to_owned();

    // Handle decimal/thousands separators.
    let comma_count = s.matches(',').count();
    let dot_count = s.matches('.').count();

    if comma_count > 0 && dot_count > 0 {
        // Assume the last separator is decimal; remove the other as thousands.
        let last_comma = s.rfind(',').unwrap_or(0);
        let last_dot = s.rfind('.').unwrap_or(0);
        if last_comma > last_dot {
            s = s.replace('.', "").replace(',', ".");
        } else {
            s = s.replace(',', "");
        }
    } else if comma_count > 0 && dot_count == 0 {
        s = s.replace(',', ".");
    }

    let mut value = Decimal::from_str(&s).ok()?;
    if trailing_minus || paren_negative {
        value = -value;
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::{parse_amount_token, parse_transactions_from_text};

    #[test]
    fn parses_common_amount_variants() {
        assert_eq!(
            parse_amount_token("-123.45")
                .map(|d| d.to_string())
                .as_deref(),
            Some("-123.45")
        );
        assert_eq!(
            parse_amount_token("123,45")
                .map(|d| d.to_string())
                .as_deref(),
            Some("123.45")
        );
        assert_eq!(
            parse_amount_token("(12.30)")
                .map(|d| d.to_string())
                .as_deref(),
            Some("-12.30")
        );
        assert_eq!(
            parse_amount_token("123.45-")
                .map(|d| d.to_string())
                .as_deref(),
            Some("-123.45")
        );
    }

    #[test]
    fn parses_transaction_lines_from_text() {
        let input = r#"
            Statement header
            2026-03-01 Coffee Shop -12.45
            2026-03-02 Salary 2500.00
        "#;

        let rows = parse_transactions_from_text(input).expect("parse should succeed");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].description, "Coffee Shop");
        assert_eq!(rows[1].description, "Salary");
    }
}
