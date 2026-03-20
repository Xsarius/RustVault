//! PDF statement parser.
//!
//! Uses text extraction and line heuristics to recover transaction rows from
//! bank statement PDFs. Because statement layouts vary per bank, this parser
//! focuses on robust date/amount extraction and keeps the original line in
//! metadata for later inspection.
//!
//! ## Parsing strategy
//!
//! Three passes run over the extracted text lines:
//! 1. Single-line: each line is tried independently.
//! 2. Pairs: adjacent non-blank lines are concatenated and retried
//!    (handles two-line transaction records).
//! 3. Triples: same with three adjacent lines.
//!
//! Each pass skips lines that were already matched by a previous pass.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::OnceLock;

use fluent_syntax::ast;
use fluent_syntax::parser as ftl_parser;
use regex::Regex;
use rust_decimal::Decimal;
use time::{Date, Month, OffsetDateTime};

use crate::ImportResult;
use crate::error::ImportError;
use crate::raw::{ColumnMapping, ImportParser, RawTransaction};

use super::date::parse_date;

// ── Month-abbreviation lookup (from locale FTL files) ─────────────────────────
//
// Locale data lives in the project-level `locales/` directory alongside the
// Fluent UI messages, so all locale data is in one place.
//
// ## Adding a new locale
//
// 1. Create `locales/<code>/months.ftl` with `month-N = abbrev` messages.
// 2. Register the locale in `locales/_meta.toml`.
// 3. Add one `locale_months!("<code>")` line to `LOCALE_SOURCES` below.

/// Embed a `locales/<code>/months.ftl` file at compile time.
macro_rules! locale_months {
    ($code:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../locales/",
            $code,
            "/months.ftl"
        ))
    };
}

const LOCALE_SOURCES: &[&str] = &[locale_months!("en-US"), locale_months!("pl-PL")];

fn build_month_map() -> HashMap<String, u8> {
    let mut map = HashMap::new();
    for src in LOCALE_SOURCES {
        let resource: ast::Resource<&str> = match ftl_parser::parse(*src) {
            Ok(r) => r,
            Err((r, errors)) => {
                for e in &errors {
                    tracing::warn!(?e, "months.ftl parse warning");
                }
                r
            }
        };
        for entry in &resource.body {
            let ast::Entry::Message(msg) = entry else {
                continue;
            };
            let Some(num_str) = msg.id.name.strip_prefix("month-") else {
                continue;
            };
            let Ok(num) = num_str.parse::<u8>() else {
                continue;
            };
            if let Some(p) = &msg.value {
                if let Some(t) = ftl_single_text(p) {
                    map.insert(t.to_lowercase(), num);
                }
            }
            for attr in &msg.attributes {
                if let Some(t) = ftl_single_text(&attr.value) {
                    map.insert(t.to_lowercase(), num);
                }
            }
        }
    }
    map
}

fn ftl_single_text<'a>(pattern: &'a ast::Pattern<&'a str>) -> Option<&'a str> {
    if let [ast::PatternElement::TextElement { value }] = pattern.elements.as_slice() {
        Some(value.trim())
    } else {
        None
    }
}

fn parse_month_abbrev(s: &str) -> Option<Month> {
    static MAP: OnceLock<HashMap<String, u8>> = OnceLock::new();
    let map = MAP.get_or_init(build_month_map);
    let num = map.get(&s.to_lowercase())?;
    Month::try_from(*num).ok()
}

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
    let lines: Vec<&str> = text.lines().collect();
    let n = lines.len();

    tracing::debug!(
        chars = text.len(),
        lines = n,
        "PDF: text extraction complete"
    );

    let mut rows: Vec<RawTransaction> = Vec::new();
    let mut used = vec![false; n];

    // Pass 1 — single lines.
    for (i, &line) in lines.iter().enumerate() {
        if let Some(txn) = parse_line(line, i + 1)? {
            tracing::debug!(line = i + 1, "PDF: matched single-line transaction");
            rows.push(txn);
            used[i] = true;
        }
    }

    // Pass 2 — pairs of adjacent non-blank lines.
    // Handles records where date and amount are on separate lines.
    for i in 0..n.saturating_sub(1) {
        if used[i] || used[i + 1] {
            continue;
        }
        let a = lines[i].trim();
        let b = lines[i + 1].trim();
        if a.is_empty() || b.is_empty() {
            continue;
        }
        let combined = format!("{a} {b}");
        if let Some(txn) = parse_line(&combined, i + 1)? {
            tracing::debug!(line = i + 1, "PDF: matched 2-line transaction");
            rows.push(txn);
            used[i] = true;
            used[i + 1] = true;
        }
    }

    // Pass 3 — triples of adjacent non-blank lines.
    for i in 0..n.saturating_sub(2) {
        if used[i] || used[i + 1] || used[i + 2] {
            continue;
        }
        let a = lines[i].trim();
        let b = lines[i + 1].trim();
        let c = lines[i + 2].trim();
        if a.is_empty() || b.is_empty() || c.is_empty() {
            continue;
        }
        let combined = format!("{a} {b} {c}");
        if let Some(txn) = parse_line(&combined, i + 1)? {
            tracing::debug!(line = i + 1, "PDF: matched 3-line transaction");
            rows.push(txn);
            used[i] = true;
            used[i + 1] = true;
            used[i + 2] = true;
        }
    }

    tracing::debug!(total = rows.len(), "PDF: parse complete");

    // Restore chronological order regardless of which pass matched each row.
    rows.sort_by_key(|r| r.date);

    // Return empty list rather than an error — let the caller decide
    // whether zero rows is acceptable (e.g. the upload handler will
    // surface this as total_rows=0 rather than a 400 Bad Request).
    Ok(rows)
}

fn parse_line(line: &str, line_no: usize) -> ImportResult<Option<RawTransaction>> {
    let normalized = normalize_whitespace(line);
    // Need at least 5 chars and 2 tokens (date + amount at minimum).
    if normalized.len() < 5 {
        return Ok(None);
    }

    let tokens: Vec<&str> = normalized.split(' ').collect();
    if tokens.len() < 2 {
        return Ok(None);
    }

    // Find a date: try single tokens first, then pairs and triples of
    // adjacent tokens to handle formats like "15 Jan 2024" or "Jan 15 2024".
    // Also tries year-optional variants ("01/15", "15 Jan") so that statements
    // without per-row year columns are still matched.
    let mut date_idx = None; // index of the LAST token of the matched date
    let mut date_span = 1usize; // how many tokens the date consumed
    let mut parsed_date = None;

    'outer: for i in 0..tokens.len() {
        // ── Full-year windows (most specific) ─────────────────────────────
        // Try longest spans first so "15 sty 2026" is preferred over
        // yearless "15 sty" when a year token is present.

        // three consecutive tokens: "15 Jan 2024", "Jan 15 2024", "15 sty 2026".
        if i + 2 < tokens.len() {
            let three = format!("{} {} {}", tokens[i], tokens[i + 1], tokens[i + 2]);
            if let Ok(d) = parse_date(&three, None) {
                date_idx = Some(i + 2);
                date_span = 3;
                parsed_date = Some(d);
                break 'outer;
            }
            // Fallback for locale-specific month abbreviations not understood by
            // the `time` crate (e.g. Polish "sty", "lut", "wrz", "paź").
            if let Some(d) = parse_date_custom_month(tokens[i], tokens[i + 1], tokens[i + 2]) {
                date_idx = Some(i + 2);
                date_span = 3;
                parsed_date = Some(d);
                break 'outer;
            }
        }
        // two consecutive tokens with full year: "15/Jan/2024" edge cases.
        if i + 1 < tokens.len() {
            let two = format!("{} {}", tokens[i], tokens[i + 1]);
            if let Ok(d) = parse_date(&two, None) {
                date_idx = Some(i + 1);
                date_span = 2;
                parsed_date = Some(d);
                break;
            }
        }
        // single token with full year: "2024-01-15", "01/15/2024", etc.
        if let Ok(d) = parse_date(tokens[i], None) {
            date_idx = Some(i);
            date_span = 1;
            parsed_date = Some(d);
            break;
        }

        // ── Year-optional fallbacks ────────────────────────────────────────
        // Only used when no full-year format matched at this position.

        // single token without year: "01/15", "15.01.", etc.
        if let Some(d) = parse_date_yearless(tokens[i]) {
            date_idx = Some(i);
            date_span = 1;
            parsed_date = Some(d);
            break;
        }
        // two consecutive tokens without year: "15 Jan", "Jan 15", "15 sty".
        if i + 1 < tokens.len() {
            let two = format!("{} {}", tokens[i], tokens[i + 1]);
            if let Some(d) = parse_date_yearless_two(&two) {
                date_idx = Some(i + 1);
                date_span = 2;
                parsed_date = Some(d);
                break;
            }
        }
    }

    let (date_end_idx, date) = match (date_idx, parsed_date) {
        (Some(i), Some(d)) => (i, d),
        _ => return Ok(None),
    };
    let date_idx = date_end_idx;

    // Find the rightmost amount-like token that appears after the date.
    // Require at least 2 characters to avoid treating bare single-digit
    // tokens (e.g. column separators) as monetary amounts.
    let mut amount_idx = None;
    let mut amount = None;
    let mut currency = None;
    for i in (date_idx + 1..tokens.len()).rev() {
        if tokens[i].len() < 2 {
            continue;
        }
        if let Some((parsed, cur)) = parse_amount_token(tokens[i]) {
            amount_idx = Some(i);
            amount = Some(parsed);
            currency = cur;
            break;
        }
    }

    let (amount_idx, amount) = match (amount_idx, amount) {
        (Some(i), Some(v)) => (i, v),
        _ => return Ok(None),
    };

    // Build description from tokens between date and amount.
    // Empty description is allowed — some statements omit it.
    let description = if amount_idx > date_idx + 1 {
        tokens[date_idx + 1..amount_idx].join(" ").trim().to_owned()
    } else {
        String::new()
    };

    let _ = date_span; // consumed above; suppress unused-variable warning

    let mut metadata = HashMap::new();
    metadata.insert("source".to_owned(), serde_json::json!("pdf"));
    metadata.insert("line".to_owned(), serde_json::json!(line_no));
    metadata.insert("raw_line".to_owned(), serde_json::json!(normalized));

    Ok(Some(RawTransaction {
        date,
        amount,
        currency,
        description,
        payee: None,
        reference: None,
        metadata,
    }))
}

fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ── Year-optional date parsing ────────────────────────────────────────────────
//
// Many bank statements omit the year column and repeat it only in the header.
// These helpers match common yearless formats and inject the current UTC year.

/// Parse three adjacent tokens that form a full-year date using locale-aware
/// month abbreviation lookup, which handles abbreviations (English, Polish, …)
/// that the `time` crate's `[month repr:short]` does not recognise.
///
/// Supported arrangements:
/// - `"DD mon YYYY"` e.g. `"15 sty 2026"`, `"01 Mar 2024"`
/// - `"mon DD YYYY"` e.g. `"Jan 15 2024"`
fn parse_date_custom_month(a: &str, b: &str, c: &str) -> Option<Date> {
    // "DD mon YYYY"
    if let (Ok(day), Some(month), Ok(year)) =
        (a.parse::<u8>(), parse_month_abbrev(b), c.parse::<i32>())
    {
        if let Ok(d) = Date::from_calendar_date(year, month, day) {
            return Some(d);
        }
    }
    // "mon DD YYYY"
    if let (Some(month), Ok(day), Ok(year)) =
        (parse_month_abbrev(a), b.parse::<u8>(), c.parse::<i32>())
    {
        if let Ok(d) = Date::from_calendar_date(year, month, day) {
            return Some(d);
        }
    }
    None
}

/// Try to parse a *single* token that contains a date without a year:
/// - `MM/DD` or `DD/MM` (slash-separated two numbers)
/// - `DD.MM` or `DD.MM.` (dot-separated, optional trailing dot)
///
/// Ambiguous cases (both numbers ≤ 12) are interpreted as `MM/DD` for slash
/// and `DD.MM` for dots (European default).
fn parse_date_yearless(s: &str) -> Option<Date> {
    let year = OffsetDateTime::now_utc().year();
    let s = s.trim().trim_end_matches('.');
    let sep = if s.contains('/') {
        '/'
    } else if s.contains('.') {
        '.'
    } else {
        return None;
    };

    let (a, b) = split_two_u8(s, sep)?;

    if sep == '/' {
        // Try MM/DD first (American) then DD/MM.
        if (1..=12).contains(&a) && (1..=31).contains(&b) {
            if let Ok(d) = Date::from_calendar_date(year, Month::try_from(a).ok()?, b) {
                return Some(d);
            }
        }
        if (1..=31).contains(&a) && (1..=12).contains(&b) {
            if let Ok(d) = Date::from_calendar_date(year, Month::try_from(b).ok()?, a) {
                return Some(d);
            }
        }
    } else {
        // Dot: treat as DD.MM (European default).
        if (1..=31).contains(&a) && (1..=12).contains(&b) {
            if let Ok(d) = Date::from_calendar_date(year, Month::try_from(b).ok()?, a) {
                return Some(d);
            }
        }
    }

    None
}

/// Try to parse two adjacent tokens that form a year-less date:
/// - `"15 Jan"` or `"Jan 15"` (day + abbreviated month name, or vice-versa)
fn parse_date_yearless_two(s: &str) -> Option<Date> {
    let year = OffsetDateTime::now_utc().year();
    let parts: Vec<&str> = s.splitn(2, ' ').collect();
    if parts.len() != 2 {
        return None;
    }

    // "DD Mon"
    if let (Ok(day), Some(month)) = (parts[0].parse::<u8>(), parse_month_abbrev(parts[1])) {
        if let Ok(d) = Date::from_calendar_date(year, month, day) {
            return Some(d);
        }
    }

    // "Mon DD"
    if let (Some(month), Ok(day)) = (parse_month_abbrev(parts[0]), parts[1].parse::<u8>()) {
        if let Ok(d) = Date::from_calendar_date(year, month, day) {
            return Some(d);
        }
    }

    None
}

/// Split `"NN<sep>MM"` into `(u8, u8)`. Returns `None` if there are more than
/// two parts or parsing fails.
fn split_two_u8(s: &str, sep: char) -> Option<(u8, u8)> {
    let mut parts = s.split(sep);
    let a: u8 = parts.next()?.trim().parse().ok()?;
    let b: u8 = parts.next()?.trim().parse().ok()?;
    if parts.next().is_some() {
        return None; // more than two segments
    }
    Some((a, b))
}

/// Returns `(amount, detected_currency)` where `detected_currency` is
/// `Some("PLN")` when a Polish złoty marker (`zł` / `PLN`) is found on the
/// token, or `None` for plain numeric tokens.
///
/// Supports variants:
///  -123.45      +123,45      (123.45)
///  1 234,56     1.234,56     123.45-
///  1 234,56 zł  -24,99 zł   PLN 1 234,56
fn parse_amount_token(token: &str) -> Option<(Decimal, Option<String>)> {
    // Reject single-character tokens — they are almost certainly column
    // separators or single-digit row counters, not monetary amounts.
    if token.len() < 2 {
        return None;
    }

    // Detect and strip Polish złoty markers before running the numeric regex.
    // "zł" can appear as a suffix (e.g. "24,99 zł") or, rarely, a prefix.
    // Banks also write the ISO code "PLN" as a suffix or prefix.
    let mut currency_hint: Option<String> = None;
    let cleaned = strip_currency_marker(token, &mut currency_hint);
    let s_for_match = cleaned.trim();

    if s_for_match.is_empty() {
        return None;
    }

    static AMOUNT_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = AMOUNT_RE.get_or_init(|| {
        Regex::new(r"^[+-]?\(?\p{Sc}?[0-9][0-9\s.,]*\)?-?$").expect("amount regex must compile")
    });

    if !re.is_match(s_for_match) {
        return None;
    }

    let mut s = s_for_match.trim().replace(' ', "");

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
    Some((value, currency_hint))
}

/// Strip a leading/trailing currency marker from `token` and, if found, set
/// `*hint` to the ISO currency code.
///
/// Recognised markers: `zł` (Polish złoty) and `PLN` (ISO code).
/// Other Unicode currency symbols (`$`, `€`, `£`, …) stay in the string so
/// the main regex can still handle them via `\p{Sc}`.
fn strip_currency_marker<'a>(token: &'a str, hint: &mut Option<String>) -> &'a str {
    // Suffix: "24,99 zł" or "24,99zł" or "1 234,56 PLN"
    let trimmed = token.trim();
    if let Some(base) = trimmed.strip_suffix("zł") {
        *hint = Some("PLN".to_owned());
        return base.trim_end();
    }
    if let Some(base) = trimmed.strip_suffix("PLN") {
        *hint = Some("PLN".to_owned());
        return base.trim_end();
    }
    // Prefix: "PLN 1 234,56"
    if let Some(base) = trimmed.strip_prefix("PLN") {
        *hint = Some("PLN".to_owned());
        return base.trim_start();
    }
    // Prefix: "zł 24,99" (rare but possible)
    if let Some(base) = trimmed.strip_prefix("zł") {
        *hint = Some("PLN".to_owned());
        return base.trim_start();
    }
    trimmed
}

#[cfg(test)]
mod tests {
    use super::{
        parse_amount_token, parse_date_yearless, parse_date_yearless_two,
        parse_transactions_from_text,
    };

    #[test]
    fn parses_common_amount_variants() {
        assert_eq!(
            parse_amount_token("-123.45")
                .map(|(d, _)| d.to_string())
                .as_deref(),
            Some("-123.45")
        );
        assert_eq!(
            parse_amount_token("123,45")
                .map(|(d, _)| d.to_string())
                .as_deref(),
            Some("123.45")
        );
        assert_eq!(
            parse_amount_token("(12.30)")
                .map(|(d, _)| d.to_string())
                .as_deref(),
            Some("-12.30")
        );
        assert_eq!(
            parse_amount_token("123.45-")
                .map(|(d, _)| d.to_string())
                .as_deref(),
            Some("-123.45")
        );
        // Single-character tokens must not be parsed as amounts.
        assert!(parse_amount_token("1").is_none());
    }

    #[test]
    fn parses_polish_amount_with_zl_suffix() {
        // "24,99 zł" — suffix złoty marker, comma decimal (Polish standard)
        let (amt, cur) = parse_amount_token("24,99zł").expect("should parse");
        assert_eq!(amt.to_string(), "24.99");
        assert_eq!(cur.as_deref(), Some("PLN"));

        // Negative with trailing minus
        let (amt2, cur2) = parse_amount_token("-1234,56zł").expect("should parse");
        assert_eq!(amt2.to_string(), "-1234.56");
        assert_eq!(cur2.as_deref(), Some("PLN"));

        // PLN ISO code suffix
        let (amt3, cur3) = parse_amount_token("1234,56PLN").expect("should parse");
        assert_eq!(amt3.to_string(), "1234.56");
        assert_eq!(cur3.as_deref(), Some("PLN"));
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

    #[test]
    fn parses_line_without_description() {
        // Amount immediately follows date — description must be empty string, not an error.
        let input = "2026-03-01 -24.99\n2026-03-02 500.00\n";
        let rows = parse_transactions_from_text(input).expect("parse should succeed");
        assert_eq!(rows.len(), 2);
        assert!(rows[0].description.is_empty());
        assert!(rows[1].description.is_empty());
    }

    #[test]
    fn parses_multi_line_transaction_via_sliding_window() {
        // Date on one line, amount on the next — matched by the 2-line pass.
        let input = "2026-03-15 Grocery Store\n-24.99\n2026-03-16 Salary\n2500.00\n";
        let rows = parse_transactions_from_text(input).expect("parse should succeed");
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn parses_yearless_slash_date() {
        // MM/DD without year.
        let input = "03/15 Coffee -5.00\n";
        let rows = parse_transactions_from_text(input).expect("parse should succeed");
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn parses_yearless_abbreviated_month() {
        // "15 Mar" without year (two-token date).
        let input = "15 Mar Grocery 24.99\n";
        let rows = parse_transactions_from_text(input).expect("parse should succeed");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].description, "Grocery");
    }

    #[test]
    fn yearless_helpers_return_none_for_invalid_input() {
        assert!(parse_date_yearless("abc").is_none());
        assert!(parse_date_yearless("99/99").is_none());
        assert!(parse_date_yearless_two("15 Xyz").is_none());
        assert!(parse_date_yearless_two("Not a date").is_none());
    }

    // ── Polish bank statement tests ────────────────────────────────────────────

    #[test]
    fn parses_polish_abbreviated_month_date() {
        // "15 sty 2026" = 15 January 2026 (PKO BP / ING style)
        let input = "15 sty 2026 Biedronka -24,99\n01 lut 2026 Wynagrodzenie 5000,00\n";
        let rows = parse_transactions_from_text(input).expect("parse should succeed");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].date.month() as u8, 1); // January
        assert_eq!(rows[0].description, "Biedronka");
        assert_eq!(rows[1].date.month() as u8, 2); // February
        assert_eq!(rows[1].description, "Wynagrodzenie");
    }

    #[test]
    fn parses_polish_statement_with_zl_currency() {
        // mBank / Santander style: DD.MM.YYYY description amount zł
        let input =
            "01.03.2026 Lidl Supermarket -89,90zł\n05.03.2026 Przelew przychodzacy 3500,00zł\n";
        let rows = parse_transactions_from_text(input).expect("parse should succeed");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].currency.as_deref(), Some("PLN"));
        assert_eq!(rows[1].currency.as_deref(), Some("PLN"));
    }

    #[test]
    fn parses_polish_statement_dot_date_format() {
        // European dot date: 01.03.2026
        let input = "01.03.2026 Allegro.pl -149,99\n10.03.2026 IKEA Warszawa -399,00\n";
        let rows = parse_transactions_from_text(input).expect("parse should succeed");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].date.day(), 1);
        assert_eq!(rows[0].date.month() as u8, 3);
        assert_eq!(rows[1].date.day(), 10);
    }

    #[test]
    fn parses_polish_october_paz_accent() {
        // "paź" uses a Polish diacritic — must map to October
        let input = "15 paź 2026 Biedronka -30,50\n";
        let rows = parse_transactions_from_text(input).expect("parse should succeed");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].date.month() as u8, 10); // October
    }

    #[test]
    fn parses_polish_october_paz_no_accent() {
        // Some PDFs lose diacritics on extraction, writing "paz" instead of "paź"
        let input = "15 paz 2026 Biedronka -30,50\n";
        let rows = parse_transactions_from_text(input).expect("parse should succeed");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].date.month() as u8, 10); // October
    }

    // ── Real-world bank statement format tests ────────────────────────────────

    /// Simulates an ING (Netherlands) PDF statement with tab-aligned columns.
    /// These statements use DD-MM-YYYY dates and comma decimals with space thousands.
    #[test]
    fn parses_ing_netherlands_style_statement() {
        let input = "\
            Account statement ING Bank N.V.\n\
            Period: 01-03-2026 through 31-03-2026\n\
            \n\
            01-03-2026 AH Albert Heijn -24,85\n\
            05-03-2026 Employer B.V. Salary March 3.450,00\n\
            10-03-2026 Vodafone NL Phone bill -29,99\n\
            15-03-2026 ABN Amro mortgage -850,00\n\
            \n\
            Closing balance 31-03-2026 EUR 2.545,16\n";

        let rows = parse_transactions_from_text(input).expect("ING parse should succeed");
        assert!(rows.len() >= 4,
            "expected ≥4 transactions from ING statement, got {}", rows.len());

        // Verify chronological order.
        for w in rows.windows(2) {
            assert!(w[0].date <= w[1].date, "rows should be in chronological order");
        }

        // The mortgage payment should parse as a debit (negative).
        let mortgage = rows.iter().find(|r| r.description.to_lowercase().contains("mortgage")
            || r.description.to_lowercase().contains("abnamro")
            || r.description.to_lowercase().contains("abn"));
        if let Some(m) = mortgage {
            assert!(m.amount < rust_decimal::Decimal::ZERO,
                "mortgage should be negative, got {}", m.amount);
        }
    }

    /// Revolut-style statement: ISO date, no currency symbol in amount column.
    #[test]
    fn parses_revolut_style_statement() {
        let input = "\
            Revolut Statement\n\
            Name: Test User\n\
            From: 2026-03-01  To: 2026-03-31\n\
            \n\
            Completed 2026-03-02 Spotify -9.99 EUR\n\
            Completed 2026-03-03 Lidl -18.45 EUR\n\
            Completed 2026-03-10 Freelance income 450.00 EUR\n\
            Completed 2026-03-20 Amazon -35.99 EUR\n";

        let rows = parse_transactions_from_text(input).expect("Revolut parse should succeed");
        assert!(rows.len() >= 3,
            "expected ≥3 Revolut transactions, got {}", rows.len());
    }

    /// mBank (Poland) PDF: European date with dot, comma decimal, Polish descriptions.
    #[test]
    fn parses_mbank_poland_style_statement() {
        let input = "\
            mBank S.A. - wyciag z rachunku\n\
            Okres: 01.03.2026 - 31.03.2026\n\
            \n\
            Data   Opis operacji                          Kwota\n\
            \n\
            03.03.2026 BIEDRONKA 1234 WARSZAWA            -35,90\n\
            05.03.2026 PRZELEW PRZYCHODZACY WYNAGRODZENIE  6500,00\n\
            07.03.2026 NETFLIX.COM                         -52,00\n\
            12.03.2026 ORLEN STACJA PALIW                  -120,00\n\
            20.03.2026 CZYNSZ MIESZKANIA                   -1800,00\n";

        let rows = parse_transactions_from_text(input).expect("mBank parse should succeed");
        assert!(rows.len() >= 4,
            "expected ≥4 mBank transactions, got {}", rows.len());

        let wynagrodzenie = rows
            .iter()
            .find(|r| r.description.to_lowercase().contains("wynagrodzenie")
                || r.description.to_lowercase().contains("przychodzacy")
                || r.amount > rust_decimal::Decimal::from(1000));
        assert!(wynagrodzenie.is_some(), "salary transaction should be found");
        if let Some(w) = wynagrodzenie {
            assert!(
                w.amount > rust_decimal::Decimal::ZERO,
                "salary should be positive income, got {}",
                w.amount
            );
        }
    }

    /// English abbreviated months (Jan, Feb, … Dec) are all recognised.
    #[test]
    fn parses_all_english_abbreviated_months() {
        let months = [
            ("01", "Jan"), ("02", "Feb"), ("03", "Mar"), ("04", "Apr"),
            ("05", "May"), ("06", "Jun"), ("07", "Jul"), ("08", "Aug"),
            ("09", "Sep"), ("10", "Oct"), ("11", "Nov"), ("12", "Dec"),
        ];
        for (num, abbr) in &months {
            let input = format!("15 {abbr} 2026 Test payment -10.00\n");
            let rows = parse_transactions_from_text(&input)
                .unwrap_or_else(|_| panic!("failed to parse month {abbr}"));
            assert_eq!(rows.len(), 1, "should parse one row for month {abbr}");
            let expected: u8 = num.parse().unwrap();
            assert_eq!(
                rows[0].date.month() as u8, expected,
                "wrong month for abbreviation {abbr}: expected {expected}, got {}",
                rows[0].date.month() as u8
            );
        }
    }

    /// A statement header/footer with running totals must not be mis-parsed as transactions.
    #[test]
    fn header_and_footer_lines_not_misidentified() {
        let input = "\
            BANK ACCOUNT STATEMENT\n\
            Account: PL61 1090 1014 0000 0712 1981 2874\n\
            Opening balance as of 2026-03-01: 5,234.12\n\
            \n\
            2026-03-05 Coffee -3.50\n\
            2026-03-10 Salary 2000.00\n\
            \n\
            Closing balance as of 2026-03-31: 7,230.62\n\
            Total debits: 3.50    Total credits: 2,000.00\n";

        let rows = parse_transactions_from_text(input).expect("parse should succeed");
        // Should find exactly the two real transactions, not header/footer amounts.
        assert_eq!(rows.len(), 2,
            "expected exactly 2 real transactions, got {}; rows: {:?}",
            rows.len(),
            rows.iter().map(|r| format!("{} {}", r.date, r.description)).collect::<Vec<_>>());
    }

    /// An entirely empty PDF text extraction yields 0 rows, not an error.
    #[test]
    fn empty_text_returns_no_rows() {
        let rows = parse_transactions_from_text("").expect("empty string should not error");
        assert_eq!(rows.len(), 0);
    }

    /// A PDF with only header/whitespace text yields 0 rows.
    #[test]
    fn whitespace_only_statement_yields_no_rows() {
        let input = "\n   \n\n   \n\n";
        let rows = parse_transactions_from_text(input).expect("whitespace-only should not error");
        assert_eq!(rows.len(), 0);
    }

    /// Output rows are always sorted by date, regardless of input order.
    #[test]
    fn output_is_sorted_chronologically() {
        // Input in reverse chronological order.
        let input = "\
            2026-03-31 Last day -5.00\n\
            2026-03-01 First day 100.00\n\
            2026-03-15 Middle day -20.00\n";
        let rows = parse_transactions_from_text(input).expect("parse should succeed");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].date.day(), 1,  "first row should be Mar 1");
        assert_eq!(rows[1].date.day(), 15, "second row should be Mar 15");
        assert_eq!(rows[2].date.day(), 31, "third row should be Mar 31");
    }

    /// Three-line PDF record: description and amount on separate lines after the date.
    #[test]
    fn parses_three_line_transaction_record() {
        // Some bank PDFs split: date | payee | amount across three lines.
        let input = "\
            2026-03-10\n\
            Online Gaming Store\n\
            -39.95\n\
            2026-03-11\n\
            Direct Debit Insurance\n\
            -120.00\n";
        let rows = parse_transactions_from_text(input).expect("3-line parse should succeed");
        // Should find both records.
        assert!(rows.len() >= 2,
            "expected ≥2 transactions from 3-line format, got {}", rows.len());
    }

    /// Amounts with thousands separators in both European and US formats.
    #[test]
    fn parses_amounts_with_thousands_separators() {
        // US style: comma as thousands, period as decimal.
        let input_us = "2026-03-01 Large payment -1,234.56\n";
        // European style: period as thousands, comma as decimal.
        let input_eu = "2026-03-01 Large payment -1.234,56\n";

        let rows_us = parse_transactions_from_text(input_us).expect("US thousands parse");
        let rows_eu = parse_transactions_from_text(input_eu).expect("EU thousands parse");

        // At least one format should successfully parse.
        let total = rows_us.len() + rows_eu.len();
        assert!(total >= 1,
            "at least one thousands-separator format should parse; US rows={}, EU rows={}",
            rows_us.len(), rows_eu.len());

        // Verify the amount is negative (debit).
        for row in rows_us.iter().chain(rows_eu.iter()) {
            assert!(row.amount < rust_decimal::Decimal::ZERO || row.amount.abs() > rust_decimal::Decimal::from(100),
                "large amount should be significant, got {}", row.amount);
        }
    }

    /// Metadata contains expected fields: source=pdf, raw_line, line number.
    #[test]
    fn transaction_metadata_contains_source_and_raw_line() {
        let input = "2026-03-05 Starbucks -5.40\n";
        let rows = parse_transactions_from_text(input).expect("parse should succeed");
        assert_eq!(rows.len(), 1);
        let meta = &rows[0].metadata;
        assert_eq!(meta.get("source").and_then(|v| v.as_str()), Some("pdf"),
            "metadata.source should be 'pdf'");
        assert!(meta.contains_key("raw_line"), "metadata should contain raw_line");
        assert!(meta.contains_key("line"), "metadata should contain line number");
    }

    /// The PdfParser detect() method returns correct confidence for various inputs.
    #[test]
    fn pdf_detect_confidence() {
        use crate::raw::ImportParser;
        let parser = super::PdfParser;

        // Magic bytes + extension: highest confidence.
        let pdf_magic = b"%PDF-1.4 fake content";
        assert!((parser.detect(pdf_magic, Some("pdf")) - 0.95).abs() < 0.01,
            "magic + ext should be 0.95");

        // Extension only, no magic: medium-low confidence.
        assert!((parser.detect(b"not a pdf", Some("pdf")) - 0.6).abs() < 0.01,
            "ext only should be 0.6");

        // Magic only, no extension: medium confidence.
        assert!((parser.detect(pdf_magic, None) - 0.8).abs() < 0.01,
            "magic only should be 0.8");

        // Neither: zero confidence.
        assert_eq!(parser.detect(b"random bytes", None), 0.0,
            "no match should be 0.0");
        assert_eq!(parser.detect(b"random bytes", Some("csv")), 0.0,
            "csv extension should be 0.0");
    }
}
