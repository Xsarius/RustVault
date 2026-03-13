//! SWIFT MT940 bank statement parser.
//!
//! MT940 is a SWIFT standard for bank statement messages.  Key fields:
//!
//! - `:20:`  Transaction reference
//! - `:25:`  Account identification
//! - `:60F:` / `:60M:` Opening balance
//! - `:61:`  Statement line (date, amount, ref)
//! - `:86:`  Information to account owner (description)
//! - `:62F:` / `:62M:` Closing balance

use std::collections::HashMap;

use rust_decimal::Decimal;
use time::Date;

use crate::raw::{ColumnMapping, ImportParser, RawTransaction};
use crate::{ImportError, ImportResult};

/// MT940 / MT942 parser.
pub struct Mt940Parser;

impl ImportParser for Mt940Parser {
    fn name(&self) -> &str {
        "MT940"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["mt940", "940", "sta"]
    }

    fn parse(
        &self,
        data: &[u8],
        _mapping: Option<&ColumnMapping>,
    ) -> ImportResult<Vec<RawTransaction>> {
        let text = String::from_utf8_lossy(data);
        parse_mt940(&text)
    }

    fn detect(&self, data: &[u8], extension: Option<&str>) -> f32 {
        if let Some(ext) = extension {
            let ext = ext.to_ascii_lowercase();
            if ext == "mt940" || ext == "940" || ext == "sta" {
                return 0.8;
            }
        }
        if let Ok(text) = std::str::from_utf8(data) {
            let trimmed = text.trim_start();
            if trimmed.starts_with(":20:") || trimmed.starts_with("{1:") {
                return 0.9;
            }
        }
        0.0
    }
}

/// Parse an MT940 file into raw transactions.
fn parse_mt940(text: &str) -> ImportResult<Vec<RawTransaction>> {
    let mut transactions = Vec::new();
    let mut currency = None::<String>;

    // Split the text into tag blocks.  Each tag starts with `:NN:` or `:NNA:`.
    let mut current_tag = String::new();
    let mut current_content = String::new();
    let mut pending_61: Option<(Date, Decimal, Option<String>)> = None;

    for line in text.lines() {
        let line = line.trim_end();

        // Detect a new tag.
        if let Some((tag, content)) = parse_tag_line(line) {
            // Flush previous tag.
            flush_tag(
                &current_tag,
                &current_content,
                &mut currency,
                &mut pending_61,
                &mut transactions,
            )?;
            current_tag = tag.to_owned();
            current_content = content.to_owned();
        } else if !current_tag.is_empty() {
            // Continuation line.
            current_content.push(' ');
            current_content.push_str(line);
        }
    }

    // Flush last tag.
    flush_tag(
        &current_tag,
        &current_content,
        &mut currency,
        &mut pending_61,
        &mut transactions,
    )?;

    // If a :61: is pending without a :86: description, emit it now.
    if let Some((date, amount, reference)) = pending_61.take() {
        transactions.push(RawTransaction {
            date,
            amount,
            currency: currency.clone(),
            description: String::new(),
            payee: None,
            reference,
            metadata: HashMap::new(),
        });
    }

    if transactions.is_empty() {
        return Err(ImportError::ParseFailed(
            "no transactions found in MT940 data".into(),
        ));
    }

    Ok(transactions)
}

/// Try to split a line into `(tag, content)` e.g. `:61:` → `("61", rest)`.
fn parse_tag_line(line: &str) -> Option<(&str, &str)> {
    if !line.starts_with(':') {
        return None;
    }
    // Find closing colon (`:XX:` or `:XXX:`).
    let rest = &line[1..];
    let end = rest.find(':')?;
    if !(2..=3).contains(&end) {
        return None;
    }
    let tag = &rest[..end];
    let content = &rest[end + 1..];
    Some((tag, content))
}

fn flush_tag(
    tag: &str,
    content: &str,
    currency: &mut Option<String>,
    pending_61: &mut Option<(Date, Decimal, Option<String>)>,
    transactions: &mut Vec<RawTransaction>,
) -> ImportResult<()> {
    match tag {
        // Opening balance — extract currency.
        "60F" | "60M" => {
            // Format: C/D + YYMMDD + CUR + amount
            if content.len() >= 10 {
                let cur = &content[7..10];
                *currency = Some(cur.to_owned());
            }
        }
        // Statement line.
        "61" => {
            // Flush any pending 61 without a following 86.
            if let Some((date, amount, reference)) = pending_61.take() {
                transactions.push(RawTransaction {
                    date,
                    amount,
                    currency: currency.clone(),
                    description: String::new(),
                    payee: None,
                    reference,
                    metadata: HashMap::new(),
                });
            }
            let parsed = parse_field_61(content)?;
            *pending_61 = Some(parsed);
        }
        // Information to account owner — transaction description.
        "86" => {
            if let Some((date, amount, reference)) = pending_61.take() {
                let description = clean_description(content);
                transactions.push(RawTransaction {
                    date,
                    amount,
                    currency: currency.clone(),
                    description,
                    payee: None,
                    reference,
                    metadata: HashMap::new(),
                });
            }
            // Else: :86: without preceding :61: — informational, skip.
        }
        _ => {}
    }
    Ok(())
}

/// Parse the `:61:` field.
///
/// Format: `YYMMDD[MMDD]CD[amount][S|N|F]ref[//ref][CRLF supplementary]`
///
/// - 6 digits: value date (YYMMDD)
/// - Optional 4 digits: booking date (MMDD)
/// - 1–2 chars: debit/credit indicator (D, C, RD, RC)
/// - Amount (digits + optional comma/dot as decimal separator)
/// - Transaction type (S/N/F) + reference
fn parse_field_61(content: &str) -> ImportResult<(Date, Decimal, Option<String>)> {
    if content.len() < 10 {
        return Err(ImportError::ParseFailed(format!(
            "MT940 :61: field too short: '{content}'"
        )));
    }

    // Value date: 6 digits YYMMDD.
    let date_str = &content[..6];
    let date = parse_mt940_date(date_str)?;

    // Skip optional booking date (4 digits).
    let rest = if content.len() > 10 && content[6..10].chars().all(|c| c.is_ascii_digit()) {
        &content[10..]
    } else {
        &content[6..]
    };

    // Credit/Debit indicator.
    let (is_credit, rest) = parse_cd_indicator(rest)?;

    // Amount — digits and decimal separator until next letter.
    let amount_end = rest
        .find(|c: char| c.is_ascii_alphabetic())
        .unwrap_or(rest.len());
    let amount_str = &rest[..amount_end];
    let amount_normalised = amount_str.replace(',', ".");
    let mut amount: Decimal = amount_normalised
        .parse()
        .map_err(|e| ImportError::ParseFailed(format!("MT940 amount '{amount_str}': {e}")))?;

    if !is_credit {
        amount = -amount;
    }

    // Reference — after amount, skip type letter (S/N/F) and take until // or end.
    let ref_part = &rest[amount_end..];
    let reference = if ref_part.len() > 1 {
        let ref_text = ref_part
            .get(1..)
            .unwrap_or("")
            .split("//")
            .next()
            .unwrap_or("")
            .trim();
        if ref_text.is_empty() {
            None
        } else {
            Some(ref_text.to_owned())
        }
    } else {
        None
    };

    Ok((date, amount, reference))
}

fn parse_cd_indicator(s: &str) -> ImportResult<(bool, &str)> {
    if let Some(rest) = s.strip_prefix("RC") {
        Ok((true, rest))
    } else if let Some(rest) = s.strip_prefix("RD") {
        Ok((false, rest))
    } else if let Some(rest) = s.strip_prefix('C') {
        Ok((true, rest))
    } else if let Some(rest) = s.strip_prefix('D') {
        Ok((false, rest))
    } else {
        Err(ImportError::ParseFailed(format!(
            "expected C/D indicator, got: '{s}'"
        )))
    }
}

/// Parse YYMMDD date.
fn parse_mt940_date(s: &str) -> ImportResult<Date> {
    if s.len() != 6 || !s.chars().all(|c| c.is_ascii_digit()) {
        return Err(ImportError::ParseFailed(format!(
            "invalid MT940 date: '{s}'"
        )));
    }

    let yy: i32 = s[..2].parse().unwrap();
    let mm: u8 = s[2..4].parse().unwrap();
    let dd: u8 = s[4..6].parse().unwrap();

    // 2-digit year: 00-79 → 2000-2079, 80-99 → 1980-1999.
    let year = if yy < 80 { 2000 + yy } else { 1900 + yy };
    let month = time::Month::try_from(mm)
        .map_err(|_| ImportError::ParseFailed(format!("invalid month {mm} in date '{s}'")))?;

    Date::from_calendar_date(year, month, dd)
        .map_err(|e| ImportError::ParseFailed(format!("invalid MT940 date '{s}': {e}")))
}

/// Clean up MT940 :86: description.
fn clean_description(raw: &str) -> String {
    // MT940 uses sub-fields like ?20, ?21, etc.
    // Concatenate all ?2x sub-fields as the description.
    let mut parts = Vec::new();
    let mut current = String::new();

    for segment in raw.split('?') {
        if segment.is_empty() {
            continue;
        }
        // First two chars are the sub-field number.
        if segment.len() >= 2 && segment[..2].chars().all(|c| c.is_ascii_digit()) {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
            current.push_str(&segment[2..]);
        } else {
            current.push_str(segment);
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }

    if parts.is_empty() {
        raw.to_owned()
    } else {
        parts.join(" ").trim().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::Mt940Parser;
    use crate::raw::ImportParser;

    #[test]
    fn parses_single_mt940_transaction() {
        let parser = Mt940Parser;
        let data = b":20:REF123\n:60F:C260301EUR0,00\n:61:260301D12,34NTRFNONREF//ABC123\n:86:Coffee Shop\n:62F:C260301EUR100,00\n";

        let rows = parser
            .parse(data, None)
            .expect("mt940 parse should succeed");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].description, "Coffee Shop");
        assert_eq!(rows[0].amount.to_string(), "-12.34");
    }
}
