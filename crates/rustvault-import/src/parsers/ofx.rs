//! OFX / QFX (Open Financial Exchange) parser.
//!
//! Handles both OFX 1.x (SGML) and OFX 2.x (XML).
//!
//! Key elements:
//! - `STMTTRN` — individual transaction entries
//! - `DTPOSTED` — posting date (YYYYMMDD or YYYYMMDDHHMMSS)
//! - `TRNAMT` — transaction amount
//! - `NAME` / `MEMO` — description fields
//! - `FITID` — financial institution transaction ID (used for dedup)
//! - `BANKMSGSRSV1` — banking statements
//! - `CREDITCARDMSGSRSV1` — credit card statements

use std::collections::HashMap;

use rust_decimal::Decimal;
use time::Date;

use crate::raw::{ColumnMapping, ImportParser, RawTransaction};
use crate::{ImportError, ImportResult};

/// OFX/QFX parser.
pub struct OfxParser;

impl ImportParser for OfxParser {
    fn name(&self) -> &str {
        "OFX"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["ofx", "qfx"]
    }

    fn parse(
        &self,
        data: &[u8],
        _mapping: Option<&ColumnMapping>,
    ) -> ImportResult<Vec<RawTransaction>> {
        let text = String::from_utf8_lossy(data);
        let xml = normalise_to_xml(&text);
        parse_ofx_xml(&xml)
    }

    fn detect(&self, data: &[u8], extension: Option<&str>) -> f32 {
        if let Some(ext) = extension {
            let ext = ext.to_ascii_lowercase();
            if ext == "ofx" || ext == "qfx" {
                return 0.8;
            }
        }
        if let Ok(text) = std::str::from_utf8(data) {
            let trimmed = text.trim_start();
            if trimmed.starts_with("OFXHEADER") || trimmed.starts_with("<?OFX") {
                return 0.95;
            }
            if trimmed.contains("<OFX>") || trimmed.contains("<ofx>") {
                return 0.9;
            }
        }
        0.0
    }
}

/// Normalise OFX 1.x SGML to well-formed XML.
///
/// OFX 1.x uses SGML-like syntax where closing tags are optional.
/// This function closes unclosed tags and strips the SGML header so that
/// the result can be parsed with a standard XML parser.
fn normalise_to_xml(text: &str) -> String {
    // Find the start of the OFX body (after the SGML header block).
    let body_start = text.find("<OFX>").or_else(|| text.find("<ofx>"));
    let body = match body_start {
        Some(pos) => &text[pos..],
        None => text,
    };

    // If it already looks like well-formed XML (has closing tags), return as-is.
    if body.contains("</STMTTRN>") || body.contains("</stmttrn>") {
        return body.to_owned();
    }

    // Close unclosed SGML tags.
    let mut result = String::with_capacity(body.len() * 2);
    let mut tag_stack: Vec<String> = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("</") {
            // Closing tag — pop stack.
            let tag_name = extract_tag_name(trimmed).to_ascii_uppercase();
            // Pop tags until we find the matching one.
            while let Some(top) = tag_stack.last() {
                if top.to_ascii_uppercase() == tag_name {
                    tag_stack.pop();
                    break;
                }
                // Close intermediate unclosed tags.
                let popped = tag_stack.pop().unwrap();
                result.push_str(&format!("</{popped}>\n"));
            }
            result.push_str(trimmed);
            result.push('\n');
        } else if trimmed.starts_with('<') {
            let tag_name = extract_tag_name(trimmed);
            if tag_name.is_empty() {
                result.push_str(trimmed);
                result.push('\n');
                continue;
            }

            // Check if this line has content after the opening tag (e.g. `<NAME>ACME Corp`).
            let after_tag = &trimmed[tag_name.len() + 2..]; // skip `<TAG>`
            let after_tag = after_tag.trim_end();

            if !after_tag.is_empty() && !after_tag.starts_with('<') {
                // Self-contained value tag — close it.
                result.push_str(&format!("<{tag_name}>{after_tag}</{tag_name}>\n"));
            } else {
                // Container tag — push onto stack.
                result.push_str(trimmed);
                result.push('\n');
                tag_stack.push(tag_name.to_owned());
            }
        } else {
            result.push_str(trimmed);
            result.push('\n');
        }
    }

    // Close any remaining open tags.
    while let Some(tag) = tag_stack.pop() {
        result.push_str(&format!("</{tag}>\n"));
    }

    result
}

/// Extract a tag name from `<TAG>...` or `</TAG>`.
fn extract_tag_name(s: &str) -> &str {
    let s = s.trim_start_matches("</").trim_start_matches('<');
    let end = s
        .find(|c: char| c == '>' || c.is_whitespace())
        .unwrap_or(s.len());
    &s[..end]
}

/// Parse well-formed OFX XML and extract STMTTRN entries.
fn parse_ofx_xml(xml: &str) -> ImportResult<Vec<RawTransaction>> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);

    let mut transactions = Vec::new();
    let mut in_stmttrn = false;
    let mut current_tag = String::new();
    let mut buf = Vec::new();

    // Current transaction fields.
    let mut dt_posted = String::new();
    let mut trn_amt = String::new();
    let mut name = String::new();
    let mut memo = String::new();
    let mut fit_id = String::new();
    let mut trn_type = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_ascii_uppercase();
                if tag == "STMTTRN" {
                    in_stmttrn = true;
                    dt_posted.clear();
                    trn_amt.clear();
                    name.clear();
                    memo.clear();
                    fit_id.clear();
                    trn_type.clear();
                }
                if in_stmttrn {
                    current_tag = tag;
                }
            }
            Ok(Event::Text(e)) => {
                if in_stmttrn {
                    let text = e.decode().unwrap_or_default().trim().to_owned();
                    match current_tag.as_str() {
                        "DTPOSTED" => dt_posted = text,
                        "TRNAMT" => trn_amt = text,
                        "NAME" => name = text,
                        "MEMO" => {
                            if memo.is_empty() {
                                memo = text;
                            }
                        }
                        "FITID" => fit_id = text,
                        "TRNTYPE" => trn_type = text,
                        _ => {}
                    }
                }
            }
            Ok(Event::End(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_ascii_uppercase();
                if tag == "STMTTRN" && in_stmttrn {
                    in_stmttrn = false;
                    // Build transaction.
                    if let Ok(tx) = build_ofx_transaction(
                        &dt_posted, &trn_amt, &name, &memo, &fit_id, &trn_type,
                    ) {
                        transactions.push(tx);
                    }
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(ImportError::ParseFailed(format!(
                    "OFX XML parse error: {e}"
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    if transactions.is_empty() {
        return Err(ImportError::ParseFailed(
            "no transactions found in OFX data".into(),
        ));
    }

    Ok(transactions)
}

fn build_ofx_transaction(
    dt_posted: &str,
    trn_amt: &str,
    name: &str,
    memo: &str,
    fit_id: &str,
    trn_type: &str,
) -> ImportResult<RawTransaction> {
    let date = parse_ofx_date(dt_posted)?;
    let amount: Decimal = trn_amt
        .parse()
        .map_err(|e| ImportError::ParseFailed(format!("OFX amount '{trn_amt}': {e}")))?;

    let description = if !name.is_empty() {
        if !memo.is_empty() && memo != name {
            format!("{name} — {memo}")
        } else {
            name.to_owned()
        }
    } else {
        memo.to_owned()
    };

    let mut metadata = HashMap::new();
    if !trn_type.is_empty() {
        metadata.insert(
            "ofx_type".to_owned(),
            serde_json::Value::String(trn_type.to_owned()),
        );
    }

    let reference = if fit_id.is_empty() {
        None
    } else {
        Some(fit_id.to_owned())
    };

    Ok(RawTransaction {
        date,
        amount,
        currency: None,
        description,
        payee: if name.is_empty() {
            None
        } else {
            Some(name.to_owned())
        },
        reference,
        metadata,
    })
}

/// Parse OFX date: `YYYYMMDD` or `YYYYMMDDHHMMSS[.XXX]`.
fn parse_ofx_date(s: &str) -> ImportResult<Date> {
    let s = s.trim();
    if s.len() < 8 {
        return Err(ImportError::ParseFailed(format!(
            "OFX date too short: '{s}'"
        )));
    }

    let date_part = &s[..8];
    if !date_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(ImportError::ParseFailed(format!(
            "OFX date contains non-digits: '{s}'"
        )));
    }

    let year: i32 = date_part[..4].parse().unwrap();
    let month: u8 = date_part[4..6].parse().unwrap();
    let day: u8 = date_part[6..8].parse().unwrap();

    let month = time::Month::try_from(month)
        .map_err(|_| ImportError::ParseFailed(format!("OFX date invalid month: '{s}'")))?;

    Date::from_calendar_date(year, month, day)
        .map_err(|e| ImportError::ParseFailed(format!("OFX date invalid: '{s}': {e}")))
}
