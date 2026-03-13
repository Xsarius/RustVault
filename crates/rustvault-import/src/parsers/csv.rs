//! CSV parser with auto-detection of delimiter, date format, and decimal separator.

use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::raw::{ColumnMapping, ImportParser, RawTransaction};
use crate::{ImportError, ImportResult};

use super::date::parse_date;

/// CSV / TSV / semicolon-separated file parser.
pub struct CsvParser;

impl ImportParser for CsvParser {
    fn name(&self) -> &str {
        "CSV"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["csv", "tsv", "txt"]
    }

    fn parse(
        &self,
        data: &[u8],
        mapping: Option<&ColumnMapping>,
    ) -> ImportResult<Vec<RawTransaction>> {
        let text = decode_text(data);
        let delimiter = detect_delimiter(&text);
        let has_header = mapping.and_then(|m| m.has_header).unwrap_or(true);

        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(has_header)
            .flexible(true)
            .from_reader(text.as_bytes());

        // Build field index map from column mapping or auto-detect from headers.
        let field_map = if let Some(m) = mapping {
            build_field_map_from_mapping(m)?
        } else {
            build_field_map_from_headers(&mut rdr)?
        };

        let date_format = mapping.and_then(|m| m.date_format.as_deref());
        let decimal_sep = mapping.and_then(|m| m.decimal_separator).unwrap_or('.');

        let mut transactions = Vec::new();
        for (row_idx, result) in rdr.records().enumerate() {
            let record = result
                .map_err(|e| ImportError::ParseFailed(format!("row {}: {e}", row_idx + 1)))?;

            match parse_row(&record, &field_map, date_format, decimal_sep) {
                Ok(tx) => transactions.push(tx),
                Err(e) => {
                    tracing::warn!(row = row_idx + 1, error = %e, "skipping CSV row");
                }
            }
        }

        if transactions.is_empty() {
            return Err(ImportError::ParseFailed(
                "no valid transactions found in CSV".into(),
            ));
        }

        Ok(transactions)
    }

    fn detect(&self, data: &[u8], extension: Option<&str>) -> f32 {
        if let Some(ext) = extension {
            let ext = ext.to_ascii_lowercase();
            if ext == "csv" || ext == "tsv" || ext == "txt" {
                return 0.7;
            }
        }

        // Content-based: is it valid UTF-8 with consistent delimiter counts?
        if let Ok(text) = std::str::from_utf8(data) {
            let trimmed = text.trim_start();
            if !trimmed.is_empty()
                && !trimmed.starts_with('{')
                && !trimmed.starts_with('[')
                && !trimmed.starts_with('<')
            {
                let delim = detect_delimiter(trimmed);
                let mut lines = trimmed.lines().take(5);
                if let Some(first) = lines.next() {
                    let expected = first.as_bytes().iter().filter(|&&b| b == delim).count();
                    if expected > 0 {
                        let consistent = lines.all(|l| {
                            l.as_bytes().iter().filter(|&&b| b == delim).count() == expected
                        });
                        if consistent {
                            return 0.4;
                        }
                    }
                }
            }
        }
        0.0
    }
}

// --- Field indices ---

/// Standard field names we recognise for column mapping.
const FIELD_DATE: &str = "date";
const FIELD_AMOUNT: &str = "amount";
const FIELD_DESCRIPTION: &str = "description";
const FIELD_PAYEE: &str = "payee";
const FIELD_REFERENCE: &str = "reference";
const FIELD_CURRENCY: &str = "currency";

/// Map of logical field name → column index.
type FieldMap = HashMap<String, usize>;

fn build_field_map_from_mapping(mapping: &ColumnMapping) -> ImportResult<FieldMap> {
    let mut map = HashMap::new();
    for (field, col) in &mapping.fields {
        let idx: usize = col.parse().map_err(|_| {
            ImportError::MappingRequired(format!("column index '{col}' is not a number"))
        })?;
        map.insert(field.clone(), idx);
    }

    // Require at minimum date and amount.
    if !map.contains_key(FIELD_DATE) || !map.contains_key(FIELD_AMOUNT) {
        return Err(ImportError::MappingRequired(
            "column mapping must include 'date' and 'amount'".into(),
        ));
    }
    Ok(map)
}

fn build_field_map_from_headers(rdr: &mut csv::Reader<&[u8]>) -> ImportResult<FieldMap> {
    let headers = rdr
        .headers()
        .map_err(|e| ImportError::ParseFailed(format!("failed to read CSV headers: {e}")))?;

    let mut map = HashMap::new();
    for (i, header) in headers.iter().enumerate() {
        let h = header.trim().to_ascii_lowercase();
        // Map common header synonyms.
        match h.as_str() {
            "date" | "datum" | "booking date" | "bookingdate" | "value date" | "valuedate"
            | "transaction date" | "transactiondate" | "posting date" => {
                map.entry(FIELD_DATE.to_owned()).or_insert(i);
            }
            "amount" | "betrag" | "value" | "sum" | "kwota" | "transaction amount" => {
                map.entry(FIELD_AMOUNT.to_owned()).or_insert(i);
            }
            "description"
            | "beschreibung"
            | "narrative"
            | "details"
            | "memo"
            | "opis"
            | "text"
            | "transaction description"
            | "remark"
            | "remarks" => {
                map.entry(FIELD_DESCRIPTION.to_owned()).or_insert(i);
            }
            "payee" | "merchant" | "recipient" | "beneficiary" | "name" | "counterparty"
            | "odbiorca" | "nadawca" => {
                map.entry(FIELD_PAYEE.to_owned()).or_insert(i);
            }
            "reference" | "ref" | "check" | "check number" | "cheque" | "numer" => {
                map.entry(FIELD_REFERENCE.to_owned()).or_insert(i);
            }
            "currency" | "ccy" | "waluta" | "curr" => {
                map.entry(FIELD_CURRENCY.to_owned()).or_insert(i);
            }
            _ => {}
        }
    }

    if !map.contains_key(FIELD_DATE) || !map.contains_key(FIELD_AMOUNT) {
        return Err(ImportError::MappingRequired(
            "could not auto-detect 'date' and 'amount' columns — provide a column mapping".into(),
        ));
    }
    Ok(map)
}

// --- Row parsing ---

fn parse_row(
    record: &csv::StringRecord,
    fields: &FieldMap,
    date_format: Option<&str>,
    decimal_sep: char,
) -> ImportResult<RawTransaction> {
    let date_str = get_field(record, fields, FIELD_DATE)?;
    let amount_str = get_field(record, fields, FIELD_AMOUNT)?;
    let description = get_field_opt(record, fields, FIELD_DESCRIPTION).unwrap_or_default();

    let date = parse_date(&date_str, date_format)?;
    let amount = parse_amount(&amount_str, decimal_sep)?;

    Ok(RawTransaction {
        date,
        amount,
        currency: get_field_opt(record, fields, FIELD_CURRENCY),
        description,
        payee: get_field_opt(record, fields, FIELD_PAYEE),
        reference: get_field_opt(record, fields, FIELD_REFERENCE),
        metadata: HashMap::new(),
    })
}

fn get_field(record: &csv::StringRecord, fields: &FieldMap, name: &str) -> ImportResult<String> {
    let idx = fields
        .get(name)
        .ok_or_else(|| ImportError::MappingRequired(format!("missing field '{name}'")))?;
    record
        .get(*idx)
        .map(|s| s.trim().to_owned())
        .ok_or_else(|| ImportError::ParseFailed(format!("column {idx} out of bounds")))
}

fn get_field_opt(record: &csv::StringRecord, fields: &FieldMap, name: &str) -> Option<String> {
    let idx = fields.get(name)?;
    let val = record.get(*idx)?.trim();
    if val.is_empty() {
        None
    } else {
        Some(val.to_owned())
    }
}

// --- Helpers ---

/// Decode bytes to text, handling common encodings.
fn decode_text(data: &[u8]) -> String {
    // Try UTF-8 first.
    if let Ok(s) = std::str::from_utf8(data) {
        return s.to_owned();
    }

    // UTF-8 BOM check.
    if data.len() >= 3 && data[..3] == [0xEF, 0xBB, 0xBF] {
        if let Ok(s) = std::str::from_utf8(&data[3..]) {
            return s.to_owned();
        }
    }

    // Try Windows-1252 (common for European bank CSVs).
    let (text, _, _) = encoding_rs::WINDOWS_1252.decode(data);
    text.into_owned()
}

/// Auto-detect CSV delimiter by counting occurrences in the first line.
fn detect_delimiter(text: &str) -> u8 {
    let first_line = text.lines().next().unwrap_or("");
    let candidates: &[(u8, char)] = &[(b',', ','), (b';', ';'), (b'\t', '\t'), (b'|', '|')];

    candidates
        .iter()
        .max_by_key(|(_, ch)| first_line.matches(*ch).count())
        .map(|(b, _)| *b)
        .unwrap_or(b',')
}

/// Parse an amount string, handling different decimal separators and
/// thousands separators.
fn parse_amount(s: &str, decimal_sep: char) -> ImportResult<Decimal> {
    // Strip currency symbols, whitespace, and non-breaking spaces.
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-' || *c == '+' || *c == '.' || *c == ',')
        .collect();

    if cleaned.is_empty() {
        return Err(ImportError::ParseFailed(format!("empty amount: '{s}'")));
    }

    // Normalise to dot-decimal.
    let normalised = if decimal_sep == ',' {
        // Comma is decimal: dots are thousands separators.
        cleaned.replace('.', "").replace(',', ".")
    } else {
        // Dot is decimal: commas are thousands separators.
        cleaned.replace(',', "")
    };

    normalised
        .parse::<Decimal>()
        .map_err(|e| ImportError::ParseFailed(format!("invalid amount '{s}': {e}")))
}
