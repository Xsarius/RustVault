//! File format detection.
//!
//! Inspects raw file bytes and optional file extension to determine the most
//! likely bank statement format.

/// Detected file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// Comma/tab/semicolon-separated values.
    Csv,
    /// SWIFT MT940 bank statement.
    Mt940,
    /// OFX / QFX (Open Financial Exchange).
    Ofx,
    /// QIF (Quicken Interchange Format).
    Qif,
    /// CAMT.053 (ISO 20022 bank-to-customer statement).
    Camt053,
    /// Spreadsheet (XLSX / XLS / ODS).
    Spreadsheet,
    /// JSON.
    Json,
}

impl FileFormat {
    /// Return the canonical format string used in the database.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Mt940 => "mt940",
            Self::Ofx => "ofx",
            Self::Qif => "qif",
            Self::Camt053 => "camt053",
            Self::Spreadsheet => "xlsx",
            Self::Json => "json",
        }
    }
}

/// Try to detect the file format from raw bytes and an optional extension.
///
/// Returns `None` when the format cannot be determined.
pub fn detect_format(data: &[u8], extension: Option<&str>) -> Option<FileFormat> {
    // Check minimum size.
    if data.is_empty() {
        return None;
    }

    // --- Magic-byte / content-based checks (strongest signals) ---

    // XLSX / ODS / XLS are binary containers — check ZIP magic bytes first.
    if data.len() >= 4 && data[..4] == [0x50, 0x4B, 0x03, 0x04] {
        // ZIP archive — likely XLSX or ODS.
        return Some(FileFormat::Spreadsheet);
    }
    // Legacy XLS (BIFF / OLE2 Compound Document).
    if data.len() >= 8 && data[..8] == [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1] {
        return Some(FileFormat::Spreadsheet);
    }

    // Work with text from here on — try UTF-8 first, then lossy.
    let text = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => {
            // Could be a non-UTF8 encoding; try extension-based fallback.
            return detect_by_extension(extension);
        }
    };

    let trimmed = text.trim_start();

    // OFX: starts with OFXHEADER or <?OFX
    if trimmed.starts_with("OFXHEADER") || trimmed.starts_with("<?OFX") {
        return Some(FileFormat::Ofx);
    }
    // OFX 2.x XML variant — root element <OFX>
    if trimmed.starts_with("<?xml") && contains_ci(trimmed, "<ofx") {
        return Some(FileFormat::Ofx);
    }

    // CAMT.053 — ISO 20022 XML with BkToCstmrStmt.
    if trimmed.starts_with("<?xml") && contains_ci(trimmed, "BkToCstmrStmt") {
        return Some(FileFormat::Camt053);
    }

    // MT940 — starts with `:20:` (transaction reference) or the `{1:` SWIFT header.
    if trimmed.starts_with(":20:") || trimmed.starts_with("{1:") {
        return Some(FileFormat::Mt940);
    }

    // QIF — first non-empty line starts with `!Type:`.
    if trimmed.starts_with("!Type:") || trimmed.starts_with("!type:") {
        return Some(FileFormat::Qif);
    }

    // JSON — starts with `[` or `{`.
    if trimmed.starts_with('[') || trimmed.starts_with('{') {
        // Might also be a JSON-based OFX or something else,
        // but for bank data it's almost certainly a JSON export.
        return Some(FileFormat::Json);
    }

    // Fall back to extension.
    if let Some(fmt) = detect_by_extension(extension) {
        return Some(fmt);
    }

    // Last resort: if it looks like delimited text assume CSV.
    if looks_like_csv(trimmed) {
        return Some(FileFormat::Csv);
    }

    None
}

/// Extension-based detection.
fn detect_by_extension(extension: Option<&str>) -> Option<FileFormat> {
    let ext = extension?.to_ascii_lowercase();
    match ext.as_str() {
        "csv" | "tsv" | "txt" => Some(FileFormat::Csv),
        "mt940" | "sta" | "940" => Some(FileFormat::Mt940),
        "ofx" | "qfx" => Some(FileFormat::Ofx),
        "qif" => Some(FileFormat::Qif),
        "xml" | "camt053" | "camt" => Some(FileFormat::Camt053),
        "xlsx" | "xls" | "ods" => Some(FileFormat::Spreadsheet),
        "json" => Some(FileFormat::Json),
        _ => None,
    }
}

/// Case-insensitive substring check.
fn contains_ci(haystack: &str, needle: &str) -> bool {
    let h = haystack.to_ascii_lowercase();
    let n = needle.to_ascii_lowercase();
    h.contains(&n)
}

/// Heuristic: does the text look like delimited data?
fn looks_like_csv(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().take(5).collect();
    if let Some(first) = lines.first() {
        // Must have at least one typical delimiter.
        let delimiters = [',', ';', '\t'];
        for d in &delimiters {
            if first.contains(*d) {
                // Check at least 2 lines have the same delimiter count.
                let expected = first.matches(*d).count();
                if expected == 0 {
                    continue;
                }
                let consistent = lines[1..]
                    .iter()
                    .take(3)
                    .all(|l| l.matches(*d).count() == expected);
                if consistent {
                    return true;
                }
            }
        }
    }
    false
}
