//! Raw transaction and parser trait definitions.

use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::Date;

use crate::ImportResult;

/// A single raw transaction parsed from a bank statement file.
///
/// This is the intermediate representation — format-agnostic — that every
/// parser produces.  The import pipeline converts `RawTransaction` values
/// into domain `Transaction` records, applying deduplication, categorisation,
/// and transfer detection along the way.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTransaction {
    /// Transaction date.
    pub date: Date,
    /// Signed amount (positive = credit, negative = debit).
    pub amount: Decimal,
    /// ISO 4217 currency code (if available in the file).
    pub currency: Option<String>,
    /// Primary description / narrative.
    pub description: String,
    /// Payee / merchant name (if separately available).
    pub payee: Option<String>,
    /// Bank reference / check number / FITID.
    pub reference: Option<String>,
    /// Extra key-value data the parser wants to preserve (IBAN, BIC, etc.).
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Column mapping entry used by formats that need user-specified field mapping
/// (CSV, JSON, XLSX).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    /// Maps logical field names to column indices (0-based) or JSON paths.
    pub fields: HashMap<String, String>,
    /// Date format string (e.g. `"%Y-%m-%d"`, `"%d/%m/%Y"`).
    pub date_format: Option<String>,
    /// Decimal separator override (`'.'` or `','`).
    pub decimal_separator: Option<char>,
    /// Whether the file has a header row (relevant for CSV/XLSX).
    pub has_header: Option<bool>,
    /// Sheet name or index for spreadsheet files.
    pub sheet: Option<String>,
}

/// Trait that every import file parser must implement.
///
/// Implementing this trait is all that is needed to add a new file format to
/// the import pipeline.  Register the parser with [`ParserRegistry`](crate::registry::ParserRegistry)
/// and it will automatically be available for auto-detection and manual
/// selection in the import wizard.
///
/// # Examples
///
/// A minimal parser for a fictitious `*.bank` line-based format:
///
/// ```rust,no_run
/// use rustvault_import::raw::{ColumnMapping, ImportParser, RawTransaction};
/// use rustvault_import::ImportResult;
/// use rust_decimal::Decimal;
/// use time::Date;
/// use time::macros::format_description;
/// use std::str;
///
/// pub struct MyBankParser;
///
/// impl ImportParser for MyBankParser {
///     fn name(&self) -> &str { "MyBank" }
///
///     fn supported_extensions(&self) -> &[&str] { &["bank"] }
///
///     fn parse(
///         &self,
///         data: &[u8],
///         _mapping: Option<&ColumnMapping>,
///     ) -> ImportResult<Vec<RawTransaction>> {
///         let text = str::from_utf8(data)?;
///         let fmt = format_description!("[year]-[month]-[day]");
///         text.lines()
///             .filter(|l| !l.starts_with('#'))
///             .map(|line| {
///                 let parts: Vec<&str> = line.splitn(3, '|').collect();
///                 Ok(RawTransaction {
///                     date: Date::parse(parts[0].trim(), fmt)?,
///                     amount: parts[1].trim().parse::<Decimal>()?,
///                     currency: None,
///                     description: parts[2].trim().to_owned(),
///                     payee: None,
///                     reference: None,
///                     metadata: Default::default(),
///                 })
///             })
///             .collect()
///     }
///
///     fn detect(&self, _data: &[u8], extension: Option<&str>) -> f32 {
///         if extension == Some("bank") { 1.0 } else { 0.0 }
///     }
/// }
/// ```
pub trait ImportParser: Send + Sync {
    /// Human-readable name of the parser (e.g. `"CSV"`, `"OFX"`, `"CAMT.053"`).
    fn name(&self) -> &str;

    /// File extensions this parser handles (lowercase, without dot).
    fn supported_extensions(&self) -> &[&str];

    /// Parse the given bytes into a list of raw transactions.
    ///
    /// The parser receives the full file contents as a byte slice so that it
    /// can handle encoding detection internally.
    fn parse(
        &self,
        data: &[u8],
        mapping: Option<&ColumnMapping>,
    ) -> ImportResult<Vec<RawTransaction>>;

    /// Return a preview of the first rows without full parsing.
    ///
    /// Default implementation delegates to [`parse`](Self::parse) and truncates.
    fn preview(
        &self,
        data: &[u8],
        mapping: Option<&ColumnMapping>,
        max_rows: usize,
    ) -> ImportResult<Vec<RawTransaction>> {
        let mut rows = self.parse(data, mapping)?;
        rows.truncate(max_rows);
        Ok(rows)
    }

    /// Detect whether this parser can handle the given data.
    ///
    /// Returns a confidence score between 0.0 (definitely not) and 1.0
    /// (definitely yes).  Used by the format auto-detection system.
    fn detect(&self, _data: &[u8], extension: Option<&str>) -> f32 {
        // Default: match by extension only.
        if let Some(ext) = extension {
            let ext_lower = ext.to_ascii_lowercase();
            if self.supported_extensions().contains(&ext_lower.as_str()) {
                return 0.5;
            }
        }
        0.0
    }
}
