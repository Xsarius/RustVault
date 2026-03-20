//! Data export service (P7.10).
//!
//! Serialises a user's transactions into portable formats:
//! - **CSV**  — RFC 4180, suitable for Excel / Google Sheets.
//! - **JSON** — Array of transaction objects; full fidelity.
//! - **QIF**  — Quicken Interchange Format for import into budgeting apps.

use rust_decimal::Decimal;
use sqlx::PgPool;
use time::Date;
use uuid::Uuid;

use crate::error::CoreError;
use crate::models::transaction::Transaction;

// ── Public API ────────────────────────────────────────────────────────────────

/// Supported export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Csv,
    Json,
    Qif,
}

impl std::str::FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(ExportFormat::Csv),
            "json" => Ok(ExportFormat::Json),
            "qif" => Ok(ExportFormat::Qif),
            other => Err(format!("unsupported export format: {other}")),
        }
    }
}

/// Export all non-archived transactions for a user as the requested format.
///
/// Optional `date_from` / `date_to` narrow the export window.
/// Returns `(mime_type, filename, bytes)`.
pub async fn export_transactions(
    pool: &PgPool,
    user_id: Uuid,
    format: ExportFormat,
    date_from: Option<Date>,
    date_to: Option<Date>,
    account_id: Option<Uuid>,
) -> Result<(String, String, Vec<u8>), CoreError> {
    // Fetch all matching transactions (up to 10 000 — practical hard cap).
    let transactions = fetch_all(pool, user_id, date_from, date_to, account_id).await?;

    let (mime, filename, body) = match format {
        ExportFormat::Csv => {
            let csv = build_csv(&transactions);
            (
                "text/csv; charset=utf-8".to_owned(),
                "transactions.csv".to_owned(),
                csv.into_bytes(),
            )
        }
        ExportFormat::Json => {
            let json = serde_json::to_vec(&transactions)
                .map_err(|e| CoreError::Internal(format!("JSON serialisation failed: {e}")))?;
            (
                "application/json; charset=utf-8".to_owned(),
                "transactions.json".to_owned(),
                json,
            )
        }
        ExportFormat::Qif => {
            let qif = build_qif(&transactions);
            (
                "application/x-qif; charset=utf-8".to_owned(),
                "transactions.qif".to_owned(),
                qif.into_bytes(),
            )
        }
    };

    Ok((mime, filename, body))
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Fetch up to 10 000 transactions, applying optional filters.
async fn fetch_all(
    pool: &PgPool,
    user_id: Uuid,
    date_from: Option<Date>,
    date_to: Option<Date>,
    account_id: Option<Uuid>,
) -> Result<Vec<Transaction>, CoreError> {
    let filter = rustvault_db::repos::transaction::TransactionFilter {
        account_id,
        category_id: None,
        transaction_type: None,
        date_from,
        date_to,
        search: None,
        is_reviewed: None,
        include_deleted: false,
        tag_id: None,
        import_id: None,
    };

    let rows = rustvault_db::repos::transaction::list_by_user(
        pool,
        user_id,
        &filter,
        10_000, // hard cap for exports
        None,
        None,
    )
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            // Re-use the same mapping logic the transaction service uses.
            row_to_transaction(row)
        })
        .collect())
}

fn row_to_transaction(
    row: rustvault_db::repos::transaction::TransactionRow,
) -> Transaction {
    use crate::models::transaction::TransactionType;

    Transaction {
        id: row.id,
        user_id: row.user_id,
        account_id: row.account_id,
        category_id: row.category_id,
        import_id: row.import_id,
        transaction_type: match row.transaction_type.as_str() {
            "income" => TransactionType::Income,
            "transfer" => TransactionType::Transfer,
            _ => TransactionType::Expense,
        },
        amount: row.amount,
        currency: row.currency,
        date: row.date,
        description: row.description,
        original_desc: row.original_desc,
        payee: row.payee,
        reference: row.reference,
        notes: row.notes,
        is_reviewed: row.is_reviewed,
        is_deleted: row.is_deleted,
        is_duplicate: row.is_duplicate,
        metadata: row.metadata,
        tag_ids: None,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

// ── CSV builder ───────────────────────────────────────────────────────────────

fn build_csv(transactions: &[Transaction]) -> String {
    let mut out = String::with_capacity(transactions.len() * 80 + 200);

    // Header row
    out.push_str("date,amount,currency,type,description,payee,reference,notes,is_reviewed\r\n");

    for tx in transactions {
        out.push_str(&csv_field(&tx.date.to_string()));
        out.push(',');
        out.push_str(&csv_field(&tx.amount.to_string()));
        out.push(',');
        out.push_str(&csv_field(&tx.currency));
        out.push(',');
        out.push_str(&csv_field(&format!("{:?}", tx.transaction_type).to_lowercase()));
        out.push(',');
        out.push_str(&csv_field(&tx.description));
        out.push(',');
        out.push_str(&csv_field(tx.payee.as_deref().unwrap_or("")));
        out.push(',');
        out.push_str(&csv_field(tx.reference.as_deref().unwrap_or("")));
        out.push(',');
        out.push_str(&csv_field(tx.notes.as_deref().unwrap_or("")));
        out.push(',');
        out.push_str(if tx.is_reviewed { "true" } else { "false" });
        out.push_str("\r\n");
    }

    out
}

/// RFC 4180 CSV field: wrap in quotes and double-escape internal quotes.
fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}

// ── QIF builder ───────────────────────────────────────────────────────────────

fn build_qif(transactions: &[Transaction]) -> String {
    let mut out = String::with_capacity(transactions.len() * 80 + 64);
    out.push_str("!Type:Bank\n");

    for tx in transactions {
        // D = Date  (MM/DD'YYYY for maximum compatibility)
        let d = tx.date;
        out.push_str(&format!("D{:02}/{:02}'{:04}\n", d.month() as u8, d.day(), d.year()));

        // T = Amount (positive = credit, negative = debit)
        let sign = if tx.amount >= Decimal::ZERO { "" } else { "" };
        let _ = sign;
        out.push_str(&format!("T{}\n", tx.amount));

        // P = Payee
        if let Some(payee) = &tx.payee {
            if !payee.is_empty() {
                out.push_str(&format!("P{payee}\n"));
            }
        }

        // M = Memo / description
        if !tx.description.is_empty() {
            out.push_str(&format!("M{}\n", tx.description));
        }

        // N = Reference
        if let Some(reference) = &tx.reference {
            if !reference.is_empty() {
                out.push_str(&format!("N{reference}\n"));
            }
        }

        // ^ = End of entry
        out.push_str("^\n");
    }

    out
}
