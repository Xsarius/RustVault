//! Shared date parsing helpers used across multiple parsers.

use time::Date;

use crate::ImportError;

/// Common date formats found in bank statements.
///
/// Tried in order — most specific / unambiguous first.
const DATE_FORMATS: &[&str] = &[
    // ISO
    "[year]-[month]-[day]",
    // Verbose with short month name (multi-word, join tokens before calling)
    "[day] [month repr:short] [year]",          // 15 Jan 2024
    "[month repr:short] [day] [year]",          // Jan 15 2024
    "[day] [month repr:short] [year repr:last_two]", // 15 Jan 24
    // European
    "[day]/[month]/[year]",
    "[day]-[month]-[year]",
    "[day].[month].[year]",
    // American
    "[month]/[day]/[year]",
    "[month]-[day]-[year]",
    // Compact
    "[year][month][[day]]",
];

/// Parse a date string using an explicit format or by trying common formats.
pub fn parse_date(s: &str, explicit_format: Option<&str>) -> Result<Date, ImportError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(ImportError::ParseFailed("empty date string".into()));
    }

    if let Some(fmt) = explicit_format {
        return parse_with_format(s, fmt);
    }

    // Try each common format.
    for fmt in DATE_FORMATS {
        if let Ok(d) = parse_with_format(s, fmt) {
            return Ok(d);
        }
    }

    Err(ImportError::ParseFailed(format!(
        "unable to parse date: {s}"
    )))
}

fn parse_with_format(s: &str, fmt: &str) -> Result<Date, ImportError> {
    let items = time::format_description::parse(fmt)
        .map_err(|e| ImportError::ParseFailed(format!("bad date format '{fmt}': {e}")))?;
    Date::parse(s, &items)
        .map_err(|e| ImportError::ParseFailed(format!("date '{s}' doesn't match '{fmt}': {e}")))
}
