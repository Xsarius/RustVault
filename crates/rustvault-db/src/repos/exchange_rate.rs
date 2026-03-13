//! Exchange rate repository — SQL operations on the `exchange_rates` table.

use rust_decimal::Decimal;
use sqlx::PgPool;
use time::Date;

use crate::error::DbError;

/// Row type for a single exchange rate.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ExchangeRateRow {
    /// Row ID.
    pub id: i64,
    /// Base currency code.
    pub base_currency: String,
    /// Target currency code.
    pub target_currency: String,
    /// Conversion rate.
    pub rate: Decimal,
    /// Effective date.
    pub date: Date,
    /// Data source.
    pub source: String,
    /// When fetched.
    pub fetched_at: time::OffsetDateTime,
}

/// Upsert a batch of rates (insert or update if already present for that date).
pub async fn upsert_batch(pool: &PgPool, rates: &[UpsertRate]) -> Result<u64, DbError> {
    let mut total: u64 = 0;
    for r in rates {
        let affected = sqlx::query(
            "INSERT INTO exchange_rates (base_currency, target_currency, rate, date, source)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (base_currency, target_currency, date)
             DO UPDATE SET rate = EXCLUDED.rate, source = EXCLUDED.source,
                           fetched_at = now()",
        )
        .bind(&r.base_currency)
        .bind(&r.target_currency)
        .bind(r.rate)
        .bind(r.date)
        .bind(&r.source)
        .execute(pool)
        .await?
        .rows_affected();
        total += affected;
    }
    Ok(total)
}

/// An individual rate record for batch upsert.
pub struct UpsertRate {
    /// Base currency.
    pub base_currency: String,
    /// Target currency.
    pub target_currency: String,
    /// Rate value.
    pub rate: Decimal,
    /// Effective date.
    pub date: Date,
    /// Source label.
    pub source: String,
}

/// Look up the most recent rate for a currency pair on or before `on_date`.
pub async fn find_rate(
    pool: &PgPool,
    base: &str,
    target: &str,
    on_date: Date,
) -> Result<Option<ExchangeRateRow>, DbError> {
    let row = sqlx::query_as::<_, ExchangeRateRow>(
        "SELECT id, base_currency, target_currency, rate, date, source, fetched_at
         FROM exchange_rates
         WHERE base_currency = $1 AND target_currency = $2 AND date <= $3
         ORDER BY date DESC
         LIMIT 1",
    )
    .bind(base)
    .bind(target)
    .bind(on_date)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// List the latest rates for all pairs (one row per pair, most recent date).
pub async fn list_latest(pool: &PgPool) -> Result<Vec<ExchangeRateRow>, DbError> {
    let rows = sqlx::query_as::<_, ExchangeRateRow>(
        "SELECT DISTINCT ON (base_currency, target_currency)
                id, base_currency, target_currency, rate, date, source, fetched_at
         FROM exchange_rates
         ORDER BY base_currency, target_currency, date DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
