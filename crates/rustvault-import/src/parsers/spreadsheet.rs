//! Spreadsheet parser for XLSX, XLS and ODS files using calamine.

use calamine::{Data, Reader, open_workbook_auto_from_rs};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::io::Cursor;
use std::str::FromStr;

use crate::ImportResult;
use crate::error::ImportError;
use crate::raw::{ColumnMapping, ImportParser, RawTransaction};

use super::date::parse_date;

/// Parser for Excel (XLSX/XLS) and OpenDocument (ODS) spreadsheet files.
pub struct SpreadsheetParser;

impl ImportParser for SpreadsheetParser {
    fn name(&self) -> &str {
        "Spreadsheet"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["xlsx", "xls", "ods"]
    }

    fn parse(
        &self,
        data: &[u8],
        mapping: Option<&ColumnMapping>,
    ) -> ImportResult<Vec<RawTransaction>> {
        let cursor = Cursor::new(data);
        let mut workbook = open_workbook_auto_from_rs(cursor)
            .map_err(|e| ImportError::ParseFailed(format!("failed to open spreadsheet: {e}")))?;

        let sheet_name = pick_sheet(&workbook, mapping)?;
        let range = workbook.worksheet_range(&sheet_name).map_err(|e| {
            ImportError::ParseFailed(format!("cannot read sheet '{sheet_name}': {e}"))
        })?;

        let rows: Vec<Vec<Data>> = range.rows().map(|r| r.to_vec()).collect();
        if rows.is_empty() {
            return Err(ImportError::ParseFailed("spreadsheet is empty".into()));
        }

        let has_header = mapping.and_then(|m| m.has_header).unwrap_or(true);
        let date_format = mapping.and_then(|m| m.date_format.as_deref());

        let (col_map, data_start) = if let Some(m) = mapping {
            if m.fields.is_empty() {
                auto_detect_columns(&rows, has_header)?
            } else {
                let cm = explicit_columns(&m.fields)?;
                let start = if has_header { 1 } else { 0 };
                (cm, start)
            }
        } else {
            auto_detect_columns(&rows, has_header)?
        };

        let mut transactions = Vec::new();

        for row in rows.iter().skip(data_start) {
            if row.iter().all(|c| matches!(c, Data::Empty)) {
                continue;
            }
            if let Some(txn) = parse_row(row, &col_map, date_format)? {
                transactions.push(txn);
            }
        }

        if transactions.is_empty() {
            return Err(ImportError::ParseFailed(
                "no transactions found in spreadsheet".into(),
            ));
        }

        Ok(transactions)
    }

    fn detect(&self, data: &[u8], extension: Option<&str>) -> f32 {
        let ext_match = extension
            .map(|e| {
                let e = e.to_ascii_lowercase();
                e == "xlsx" || e == "xls" || e == "ods"
            })
            .unwrap_or(false);

        // Check magic bytes: ZIP (XLSX/ODS) or OLE2 (XLS).
        let magic_match = data.starts_with(&[0x50, 0x4B, 0x03, 0x04])
            || data.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]);

        match (ext_match, magic_match) {
            (true, true) => 0.90,
            (true, false) => 0.5,
            (false, true) => 0.3,
            (false, false) => 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Column mapping
// ---------------------------------------------------------------------------

/// Logical column indices.
struct ColumnIndices {
    date: usize,
    amount: usize,
    description: Option<usize>,
    payee: Option<usize>,
    currency: Option<usize>,
    reference: Option<usize>,
}

/// Synonyms for auto-detecting header columns.
const HEADER_SYNONYMS: &[(&str, &[&str])] = &[
    (
        "date",
        &["date", "datum", "data", "booking", "valuta", "buchungstag"],
    ),
    (
        "amount",
        &["amount", "betrag", "kwota", "value", "sum", "total"],
    ),
    (
        "description",
        &[
            "description",
            "details",
            "narrative",
            "text",
            "verwendungszweck",
            "opis",
        ],
    ),
    (
        "payee",
        &["payee", "recipient", "empfänger", "beneficiary", "odbiorca"],
    ),
    ("currency", &["currency", "ccy", "währung", "waluta"]),
    ("reference", &["reference", "ref", "referenz", "numer"]),
];

fn auto_detect_columns(
    rows: &[Vec<Data>],
    has_header: bool,
) -> ImportResult<(ColumnIndices, usize)> {
    if !has_header || rows.is_empty() {
        return Err(ImportError::MappingRequired(
            "column mapping required for spreadsheet without headers".into(),
        ));
    }

    let headers: Vec<String> = rows[0]
        .iter()
        .map(|c| cell_to_string(c).to_ascii_lowercase())
        .collect();

    let find = |synonyms: &[&str]| -> Option<usize> {
        headers
            .iter()
            .position(|h| synonyms.iter().any(|s| h.contains(s)))
    };

    let date = find(HEADER_SYNONYMS[0].1)
        .ok_or_else(|| ImportError::MappingRequired("cannot find date column".into()))?;
    let amount = find(HEADER_SYNONYMS[1].1)
        .ok_or_else(|| ImportError::MappingRequired("cannot find amount column".into()))?;

    Ok((
        ColumnIndices {
            date,
            amount,
            description: find(HEADER_SYNONYMS[2].1),
            payee: find(HEADER_SYNONYMS[3].1),
            currency: find(HEADER_SYNONYMS[4].1),
            reference: find(HEADER_SYNONYMS[5].1),
        },
        1,
    ))
}

fn explicit_columns(fields: &HashMap<String, String>) -> ImportResult<ColumnIndices> {
    let get = |key: &str| -> ImportResult<usize> {
        fields
            .get(key)
            .ok_or_else(|| ImportError::MappingRequired(format!("missing field mapping: {key}")))?
            .parse::<usize>()
            .map_err(|e| ImportError::MappingRequired(format!("bad column index for {key}: {e}")))
    };
    let opt =
        |key: &str| -> Option<usize> { fields.get(key).and_then(|v| v.parse::<usize>().ok()) };

    Ok(ColumnIndices {
        date: get("date")?,
        amount: get("amount")?,
        description: opt("description"),
        payee: opt("payee"),
        currency: opt("currency"),
        reference: opt("reference"),
    })
}

// ---------------------------------------------------------------------------
// Row parsing
// ---------------------------------------------------------------------------

fn parse_row(
    row: &[Data],
    cols: &ColumnIndices,
    date_format: Option<&str>,
) -> ImportResult<Option<RawTransaction>> {
    let date_cell = row.get(cols.date).unwrap_or(&Data::Empty);
    let amount_cell = row.get(cols.amount).unwrap_or(&Data::Empty);

    let date_str = cell_to_string(date_cell);
    let amount_str = cell_to_string(amount_cell);

    if date_str.is_empty() || amount_str.is_empty() {
        return Ok(None);
    }

    let date = if let Data::DateTime(ref dt) = *date_cell {
        // calamine ExcelDateTime — extract as Excel serial number.
        excel_date_to_time(*dt)?
    } else {
        parse_date(&date_str, date_format)?
    };

    let amount = if let Data::Float(f) = amount_cell {
        Decimal::from_str(&format!("{f}"))
            .map_err(|e| ImportError::ParseFailed(format!("invalid amount: {e}")))?
    } else {
        parse_amount(&amount_str)?
    };

    let description = cols
        .description
        .and_then(|i| row.get(i))
        .map(cell_to_string)
        .unwrap_or_default();
    let payee = cols.payee.and_then(|i| row.get(i)).map(cell_to_string);
    let currency = cols.currency.and_then(|i| row.get(i)).map(cell_to_string);
    let reference = cols.reference.and_then(|i| row.get(i)).map(cell_to_string);

    Ok(Some(RawTransaction {
        date,
        amount,
        currency,
        description,
        payee,
        reference,
        metadata: HashMap::new(),
    }))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pick_sheet<RS: std::io::Read + std::io::Seek, R: Reader<RS>>(
    workbook: &R,
    mapping: Option<&ColumnMapping>,
) -> ImportResult<String> {
    let sheets = workbook.sheet_names();
    if sheets.is_empty() {
        return Err(ImportError::ParseFailed("workbook has no sheets".into()));
    }

    if let Some(m) = mapping {
        if let Some(ref name) = m.sheet {
            if let Ok(idx) = name.parse::<usize>() {
                return sheets.get(idx).cloned().ok_or_else(|| {
                    ImportError::ParseFailed(format!("sheet index {idx} out of range"))
                });
            }
            if sheets.iter().any(|s| s == name) {
                return Ok(name.clone());
            }
            return Err(ImportError::ParseFailed(format!(
                "sheet '{name}' not found"
            )));
        }
    }

    Ok(sheets[0].clone())
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => format!("{f}"),
        Data::Int(i) => format!("{i}"),
        Data::Bool(b) => format!("{b}"),
        Data::DateTime(dt) => format!("{dt}"),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("{e:?}"),
    }
}

/// Convert calamine `ExcelDateTime` (Excel serial number) to `time::Date`.
fn excel_date_to_time(dt: calamine::ExcelDateTime) -> ImportResult<time::Date> {
    // ExcelDateTime exposes as_f64() returning Excel serial number.
    // Excel epoch: 1899-12-30 (accounting for the Lotus 1-2-3 bug).
    let serial = dt.as_f64() as i64;
    let epoch =
        time::Date::from_calendar_date(1899, time::Month::December, 30).expect("valid epoch date");
    epoch
        .checked_add(time::Duration::days(serial))
        .ok_or_else(|| ImportError::ParseFailed(format!("invalid Excel date serial: {serial}")))
}

fn parse_amount(s: &str) -> ImportResult<Decimal> {
    // Strip currency symbols / whitespace, normalise decimal separator.
    let cleaned: String = s
        .replace(',', ".")
        .chars()
        .filter(|c| *c == '-' || *c == '.' || c.is_ascii_digit())
        .collect();

    Decimal::from_str(&cleaned)
        .map_err(|e| ImportError::ParseFailed(format!("invalid amount '{s}': {e}")))
}

#[cfg(test)]
mod tests {
    use super::SpreadsheetParser;
    use crate::raw::ImportParser;

    #[test]
    fn detects_spreadsheet_from_magic_bytes() {
        let parser = SpreadsheetParser;
        let zip_magic = [0x50, 0x4B, 0x03, 0x04, 0x00];
        let xls_magic = [0xD0, 0xCF, 0x11, 0xE0, 0x00];

        assert!(parser.detect(&zip_magic, None) > 0.0);
        assert!(parser.detect(&xls_magic, None) > 0.0);
    }

    #[test]
    fn detects_spreadsheet_from_extension() {
        let parser = SpreadsheetParser;
        assert!(parser.detect(b"not-a-real-file", Some("xlsx")) > 0.0);
        assert!(parser.detect(b"not-a-real-file", Some("ods")) > 0.0);
    }
}
