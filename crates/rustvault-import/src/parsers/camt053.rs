//! CAMT.053 (ISO 20022) bank-to-customer statement parser.

use quick_xml::Reader;
use quick_xml::events::Event;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

use crate::ImportResult;
use crate::error::ImportError;
use crate::raw::{ColumnMapping, ImportParser, RawTransaction};

use super::date::parse_date;

/// Parser for ISO 20022 camt.053 XML bank statements.
pub struct Camt053Parser;

impl ImportParser for Camt053Parser {
    fn name(&self) -> &str {
        "CAMT.053"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["xml", "camt", "camt053"]
    }

    fn parse(
        &self,
        data: &[u8],
        _mapping: Option<&ColumnMapping>,
    ) -> ImportResult<Vec<RawTransaction>> {
        let text = std::str::from_utf8(data)
            .map_err(|e| ImportError::ParseFailed(format!("invalid UTF-8: {e}")))?;

        parse_camt053(text)
    }

    fn detect(&self, data: &[u8], extension: Option<&str>) -> f32 {
        let ext_match = extension
            .map(|e| {
                let e = e.to_ascii_lowercase();
                e == "camt" || e == "camt053"
            })
            .unwrap_or(false);

        let content_match = std::str::from_utf8(data)
            .ok()
            .map(|text| {
                text.contains("urn:iso:std:iso:20022:tech:xsd:camt.053")
                    || (text.contains("<BkToCstmrStmt>") || text.contains("<Ntry>"))
            })
            .unwrap_or(false);

        match (ext_match, content_match) {
            (true, true) => 0.95,
            (false, true) => 0.90,
            (true, false) => 0.4,
            (false, false) => 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// XML parsing
// ---------------------------------------------------------------------------

/// Parse a CAMT.053 XML document into raw transactions.
fn parse_camt053(xml: &str) -> ImportResult<Vec<RawTransaction>> {
    let mut reader = Reader::from_str(xml);

    let mut transactions = Vec::new();
    let mut path: Vec<String> = Vec::new();
    let mut currency = None;
    let mut buf = Vec::new();

    // Per-entry state
    let mut in_entry = false;
    let mut entry = EntryBuilder::default();

    // Per-TxDtls state (batch entries may contain multiple TxDtls)
    let mut in_tx_dtls = false;
    let mut tx_dtls = TxDtlsBuilder::default();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = local_name(e.name().as_ref());
                path.push(local.clone());

                if local == "Ntry" {
                    in_entry = true;
                    entry = EntryBuilder::default();
                } else if local == "TxDtls" && in_entry {
                    in_tx_dtls = true;
                    tx_dtls = TxDtlsBuilder::default();
                }
            }
            Ok(Event::End(ref e)) => {
                let local = local_name(e.name().as_ref());

                if local == "TxDtls" && in_tx_dtls {
                    // Emit a transaction for this TxDtls.
                    if let Some(txn) = entry.build_with_tx_dtls(&tx_dtls, &currency)? {
                        transactions.push(txn);
                    }
                    in_tx_dtls = false;
                } else if local == "Ntry" && in_entry {
                    // If there were no TxDtls children, emit from entry directly.
                    if !entry.had_tx_dtls {
                        if let Some(txn) = entry.build_simple(&currency)? {
                            transactions.push(txn);
                        }
                    }
                    in_entry = false;
                }

                path.pop();
            }
            Ok(Event::Text(ref e)) => {
                let text = e.decode().unwrap_or_default().trim().to_owned();
                if text.is_empty() {
                    continue;
                }

                let current = path.last().map(String::as_str).unwrap_or("");
                let parent = path.iter().rev().nth(1).map(String::as_str).unwrap_or("");

                // Account currency (from Bal or Stmt level).
                if current == "Ccy" && !in_entry {
                    currency = Some(text.clone());
                }

                if in_tx_dtls {
                    match current {
                        "Amt" => tx_dtls.amount = Some(text),
                        "Ccy" => tx_dtls.currency = Some(text),
                        "Dt" if parent == "BookgDt" || parent == "ValDt" => {
                            if tx_dtls.date.is_none() {
                                tx_dtls.date = Some(text);
                            }
                        }
                        "Ustrd" => tx_dtls.remittance.push(text),
                        "Nm" if parent == "Cdtr" => tx_dtls.creditor = Some(text),
                        "Nm" if parent == "Dbtr" => tx_dtls.debtor = Some(text),
                        "IBAN" => {
                            if tx_dtls.iban.is_none() {
                                tx_dtls.iban = Some(text);
                            }
                        }
                        "BIC" | "BICFI" => {
                            if tx_dtls.bic.is_none() {
                                tx_dtls.bic = Some(text);
                            }
                        }
                        "EndToEndId" => tx_dtls.end_to_end_id = Some(text),
                        "InstrId" => tx_dtls.instr_id = Some(text),
                        "AcctSvcrRef" => tx_dtls.acct_svcr_ref = Some(text),
                        _ => {}
                    }
                } else if in_entry {
                    match current {
                        "Amt" => {
                            entry.amount = Some(text.clone());
                            // Currency may be an attribute — checked below.
                        }
                        "Ccy" => entry.currency = Some(text),
                        "CdtDbtInd" => entry.credit_debit = Some(text),
                        "Dt" if parent == "BookgDt" || parent == "ValDt" => {
                            if entry.date.is_none() {
                                entry.date = Some(text);
                            }
                        }
                        "Ustrd" => entry.remittance.push(text),
                        "AcctSvcrRef" => {
                            if entry.reference.is_none() {
                                entry.reference = Some(text);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(ImportError::ParseFailed(format!("XML parse error: {e}")));
            }
            _ => {}
        }
        buf.clear();
    }

    if transactions.is_empty() {
        return Err(ImportError::ParseFailed(
            "no entries found in CAMT.053 document".into(),
        ));
    }

    Ok(transactions)
}

/// Extract local name from a potentially namespace-prefixed QName.
fn local_name(qname: &[u8]) -> String {
    let s = std::str::from_utf8(qname).unwrap_or("");
    match s.rfind(':') {
        Some(pos) => s[pos + 1..].to_owned(),
        None => s.to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Builders
// ---------------------------------------------------------------------------

#[derive(Default)]
struct EntryBuilder {
    amount: Option<String>,
    currency: Option<String>,
    credit_debit: Option<String>,
    date: Option<String>,
    remittance: Vec<String>,
    reference: Option<String>,
    had_tx_dtls: bool,
}

impl EntryBuilder {
    /// Build a simple transaction from the entry when there are no `TxDtls`.
    fn build_simple(&self, stmt_currency: &Option<String>) -> ImportResult<Option<RawTransaction>> {
        let date_str = match &self.date {
            Some(d) => d,
            None => return Ok(None),
        };
        let amount_str = match &self.amount {
            Some(a) => a,
            None => return Ok(None),
        };

        let date = parse_date(date_str, None)?;
        let mut amount = Decimal::from_str(amount_str)
            .map_err(|e| ImportError::ParseFailed(format!("invalid amount '{amount_str}': {e}")))?;

        // Apply debit sign.
        if self
            .credit_debit
            .as_deref()
            .map(|s| s.eq_ignore_ascii_case("DBIT"))
            .unwrap_or(false)
        {
            amount = -amount;
        }

        let currency = self.currency.clone().or_else(|| stmt_currency.clone());
        let description = self.remittance.join(" / ");

        Ok(Some(RawTransaction {
            date,
            amount,
            currency,
            description,
            payee: None,
            reference: self.reference.clone(),
            metadata: HashMap::new(),
        }))
    }

    /// Build a transaction using entry-level defaults plus `TxDtls` overrides.
    fn build_with_tx_dtls(
        &mut self,
        dtls: &TxDtlsBuilder,
        stmt_currency: &Option<String>,
    ) -> ImportResult<Option<RawTransaction>> {
        self.had_tx_dtls = true;

        let date_str = dtls
            .date
            .as_ref()
            .or(self.date.as_ref())
            .ok_or_else(|| ImportError::ParseFailed("entry missing date".into()))?;
        let amount_str = dtls
            .amount
            .as_ref()
            .or(self.amount.as_ref())
            .ok_or_else(|| ImportError::ParseFailed("entry missing amount".into()))?;

        let date = parse_date(date_str, None)?;
        let mut amount = Decimal::from_str(amount_str)
            .map_err(|e| ImportError::ParseFailed(format!("invalid amount '{amount_str}': {e}")))?;

        if self
            .credit_debit
            .as_deref()
            .map(|s| s.eq_ignore_ascii_case("DBIT"))
            .unwrap_or(false)
        {
            amount = -amount;
        }

        let currency = dtls
            .currency
            .clone()
            .or_else(|| self.currency.clone())
            .or_else(|| stmt_currency.clone());

        let remittance = if dtls.remittance.is_empty() {
            self.remittance.join(" / ")
        } else {
            dtls.remittance.join(" / ")
        };

        let payee = dtls.creditor.clone().or_else(|| dtls.debtor.clone());
        let reference = dtls
            .acct_svcr_ref
            .clone()
            .or_else(|| self.reference.clone());

        let mut metadata = HashMap::new();
        if let Some(iban) = &dtls.iban {
            metadata.insert("iban".into(), serde_json::Value::String(iban.clone()));
        }
        if let Some(bic) = &dtls.bic {
            metadata.insert("bic".into(), serde_json::Value::String(bic.clone()));
        }
        if let Some(e2e) = &dtls.end_to_end_id {
            metadata.insert(
                "end_to_end_id".into(),
                serde_json::Value::String(e2e.clone()),
            );
        }
        if let Some(iid) = &dtls.instr_id {
            metadata.insert("instr_id".into(), serde_json::Value::String(iid.clone()));
        }

        Ok(Some(RawTransaction {
            date,
            amount,
            currency,
            description: remittance,
            payee,
            reference,
            metadata,
        }))
    }
}

#[derive(Default)]
struct TxDtlsBuilder {
    amount: Option<String>,
    currency: Option<String>,
    date: Option<String>,
    remittance: Vec<String>,
    creditor: Option<String>,
    debtor: Option<String>,
    iban: Option<String>,
    bic: Option<String>,
    end_to_end_id: Option<String>,
    instr_id: Option<String>,
    acct_svcr_ref: Option<String>,
}
