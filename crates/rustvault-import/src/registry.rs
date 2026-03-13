//! Parser registry — provides format detection and parser lookup.

use crate::detect::{FileFormat, detect_format};
use crate::raw::ImportParser;

use crate::parsers::camt053::Camt053Parser;
use crate::parsers::csv::CsvParser;
use crate::parsers::json::JsonParser;
use crate::parsers::mt940::Mt940Parser;
use crate::parsers::ofx::OfxParser;
use crate::parsers::pdf::PdfParser;
use crate::parsers::qif::QifParser;
use crate::parsers::spreadsheet::SpreadsheetParser;

/// Central registry of all available import parsers.
///
/// Use [`ParserRegistry::new`] to create a registry pre-populated with all
/// built-in parsers. Parser selection can be done explicitly via
/// [`for_format`](Self::for_format) or automatically via
/// [`detect_and_select`](Self::detect_and_select).
pub struct ParserRegistry {
    parsers: Vec<Box<dyn ImportParser>>,
}

impl ParserRegistry {
    /// Create a registry with all built-in parsers.
    pub fn new() -> Self {
        Self {
            parsers: vec![
                Box::new(CsvParser),
                Box::new(Mt940Parser),
                Box::new(OfxParser),
                Box::new(QifParser),
                Box::new(Camt053Parser),
                Box::new(SpreadsheetParser),
                Box::new(JsonParser),
                Box::new(PdfParser),
            ],
        }
    }

    /// Return the parser for the given format.
    pub fn for_format(&self, format: FileFormat) -> &dyn ImportParser {
        let name = match format {
            FileFormat::Csv => "CSV",
            FileFormat::Mt940 => "MT940",
            FileFormat::Ofx => "OFX",
            FileFormat::Qif => "QIF",
            FileFormat::Camt053 => "CAMT.053",
            FileFormat::Spreadsheet => "Spreadsheet",
            FileFormat::Json => "JSON",
            FileFormat::Pdf => "PDF",
        };
        self.parsers
            .iter()
            .find(|p| p.name() == name)
            .expect("all built-in formats are registered")
            .as_ref()
    }

    /// Detect the format from raw data and optional extension, then return the
    /// matching parser.
    pub fn detect_and_select(
        &self,
        data: &[u8],
        extension: Option<&str>,
    ) -> Option<(&dyn ImportParser, FileFormat)> {
        let format = detect_format(data, extension)?;
        Some((self.for_format(format), format))
    }

    /// Return the parser with the highest confidence score for the given data.
    ///
    /// Unlike [`detect_and_select`](Self::detect_and_select), this queries
    /// every parser's `detect` method and picks the one with the highest
    /// confidence above the threshold.
    pub fn best_match(
        &self,
        data: &[u8],
        extension: Option<&str>,
        threshold: f32,
    ) -> Option<&dyn ImportParser> {
        self.parsers
            .iter()
            .map(|p| (p.as_ref(), p.detect(data, extension)))
            .filter(|(_, score)| *score >= threshold)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(p, _)| p)
    }

    /// List all registered parsers.
    pub fn all(&self) -> &[Box<dyn ImportParser>] {
        &self.parsers
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}
