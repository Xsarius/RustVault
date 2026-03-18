//! Report repository — SQL aggregation queries for visualisation and analysis.
//!
//! All queries filter by `user_id` for row-level security.

use rust_decimal::Decimal;
use sqlx::PgPool;
use time::Date;
use uuid::Uuid;

use crate::error::DbError;

// ── Result rows ───────────────────────────────────────────────────────────────

/// Net worth and current-month totals for the dashboard summary.
#[derive(Debug, sqlx::FromRow)]
pub struct SummaryTotalsRow {
    /// Sum of all non-archived account balances.
    pub net_worth: Option<Decimal>,
    /// Income transactions this month.
    pub month_income: Option<Decimal>,
    /// Expense transactions this month (absolute value).
    pub month_expenses: Option<Decimal>,
    /// Number of unreviewed, non-deleted transactions.
    pub unreviewed_count: i64,
}

/// Monthly income/expense aggregation row.
#[derive(Debug, sqlx::FromRow)]
pub struct MonthlyIeRow {
    /// First day of the month.
    pub month: Date,
    /// Total income for the month.
    pub income: Option<Decimal>,
    /// Total expenses for the month (absolute value).
    pub expenses: Option<Decimal>,
}

/// Spending per category for a given period.
#[derive(Debug, sqlx::FromRow)]
pub struct CategorySpendRow {
    /// Category ID.
    pub category_id: Option<Uuid>,
    /// Category name (NULL when uncategorised).
    pub category_name: Option<String>,
    /// Total absolute spend.
    pub total: Option<Decimal>,
}

/// Monthly income/expense row with a category/type breakdown.
#[derive(Debug, sqlx::FromRow)]
pub struct MonthlyIeCategoryRow {
    /// First day of the month.
    pub month: Date,
    /// Category ID (NULL = uncategorised).
    pub category_id: Option<Uuid>,
    /// Category name (NULL = uncategorised for expenses; "Income" for income).
    pub category_name: Option<String>,
    /// Total income for (month, category).
    pub income: Option<Decimal>,
    /// Total expenses for (month, category) (absolute value).
    pub expenses: Option<Decimal>,
}

/// Running balance snapshot per account per day.
#[derive(Debug, sqlx::FromRow)]
pub struct DailyBalanceRow {
    /// Transaction date.
    pub date: Date,
    /// Account ID.
    pub account_id: Uuid,
    /// Net change on this day for this account.
    pub daily_net: Option<Decimal>,
}

/// Account balance used when reconstructing history.
#[derive(Debug, sqlx::FromRow)]
pub struct AccountBalanceSeedRow {
    /// Account ID.
    pub account_id: Uuid,
    /// Current cached balance.
    pub balance_cache: Decimal,
    /// ISO 4217 currency.
    pub currency: String,
    /// Account display name.
    pub name: String,
}

/// Amount to adjust current balance for transactions after `to_date`.
#[derive(Debug, sqlx::FromRow)]
pub struct FutureAmountRow {
    /// Account ID.
    pub account_id: Uuid,
    /// Sum of transactions after `to_date` (may be NULL if none).
    pub future_net: Option<Decimal>,
}

/// Category trend row — spending per period for a single category.
#[derive(Debug, sqlx::FromRow)]
pub struct CategoryTrendRow {
    /// Period start (day-truncated to period granularity).
    pub period: Date,
    /// Total absolute spend in this period.
    pub total: Option<Decimal>,
}

/// Cash flow row — income and expenses per period.
#[derive(Debug, sqlx::FromRow)]
pub struct CashFlowRow {
    /// Period start.
    pub period: Date,
    /// Total income.
    pub income: Option<Decimal>,
    /// Total expenses (absolute value).
    pub expenses: Option<Decimal>,
}

// ── Query functions ───────────────────────────────────────────────────────────

/// Net worth, current-month totals, and unreviewed count in a single pass.
pub async fn summary_totals(pool: &PgPool, user_id: Uuid) -> Result<SummaryTotalsRow, DbError> {
    let row = sqlx::query_as::<_, SummaryTotalsRow>(
        "SELECT
           (SELECT COALESCE(SUM(balance_cache), 0)
            FROM accounts
            WHERE user_id = $1 AND NOT is_archived) AS net_worth,

           (SELECT COALESCE(SUM(amount), 0)
            FROM transactions
            WHERE user_id = $1
              AND NOT is_deleted
              AND amount > 0
              AND transaction_type::text != 'transfer'
              AND date >= date_trunc('month', CURRENT_DATE)::date
              AND date <  (date_trunc('month', CURRENT_DATE) + INTERVAL '1 month')::date
           ) AS month_income,

           (SELECT COALESCE(SUM(ABS(amount)), 0)
            FROM transactions
            WHERE user_id = $1
              AND NOT is_deleted
              AND amount < 0
              AND transaction_type::text != 'transfer'
              AND date >= date_trunc('month', CURRENT_DATE)::date
              AND date <  (date_trunc('month', CURRENT_DATE) + INTERVAL '1 month')::date
           ) AS month_expenses,

           (SELECT COUNT(*)
            FROM transactions
            WHERE user_id = $1
              AND NOT is_deleted
              AND NOT is_reviewed
           ) AS unreviewed_count",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Monthly income and expenses for the last `months` months.
pub async fn monthly_income_expense(
    pool: &PgPool,
    user_id: Uuid,
    months: i32,
) -> Result<Vec<MonthlyIeRow>, DbError> {
    let rows = sqlx::query_as::<_, MonthlyIeRow>(
        "SELECT
           date_trunc('month', date)::date AS month,
           COALESCE(SUM(CASE WHEN amount > 0 AND transaction_type::text != 'transfer'
                              THEN amount ELSE 0 END), 0) AS income,
           COALESCE(SUM(CASE WHEN amount < 0 AND transaction_type::text != 'transfer'
                              THEN ABS(amount) ELSE 0 END), 0) AS expenses
         FROM transactions
         WHERE user_id = $1
           AND NOT is_deleted
           AND date >= (date_trunc('month', CURRENT_DATE) - ($2 - 1) * INTERVAL '1 month')::date
         GROUP BY 1
         ORDER BY 1",
    )
    .bind(user_id)
    .bind(months)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Spending by category for the current calendar month (top N).
pub async fn spending_by_category(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<CategorySpendRow>, DbError> {
    let rows = sqlx::query_as::<_, CategorySpendRow>(
        "SELECT
           t.category_id,
           c.name AS category_name,
           COALESCE(SUM(ABS(t.amount)), 0) AS total
         FROM transactions t
         LEFT JOIN categories c ON c.id = t.category_id
         WHERE t.user_id = $1
           AND NOT t.is_deleted
           AND t.amount < 0
           AND t.transaction_type::text != 'transfer'
           AND t.date >= date_trunc('month', CURRENT_DATE)::date
           AND t.date <  (date_trunc('month', CURRENT_DATE) + INTERVAL '1 month')::date
         GROUP BY t.category_id, c.name
         ORDER BY total DESC
         LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Monthly income/expenses with category breakdown for a date range.
pub async fn income_expense_by_category(
    pool: &PgPool,
    user_id: Uuid,
    from: Date,
    to: Date,
) -> Result<Vec<MonthlyIeCategoryRow>, DbError> {
    let rows = sqlx::query_as::<_, MonthlyIeCategoryRow>(
        "SELECT
           date_trunc('month', date)::date AS month,
           category_id,
           (SELECT name FROM categories c WHERE c.id = t.category_id) AS category_name,
           COALESCE(SUM(CASE WHEN amount > 0 AND transaction_type::text != 'transfer'
                              THEN amount ELSE 0 END), 0) AS income,
           COALESCE(SUM(CASE WHEN amount < 0 AND transaction_type::text != 'transfer'
                              THEN ABS(amount) ELSE 0 END), 0) AS expenses
         FROM transactions t
         WHERE user_id = $1
           AND NOT is_deleted
           AND date BETWEEN $2 AND $3
         GROUP BY 1, 2
         ORDER BY 1, total DESC",
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Daily net changes per account within a date range (for balance history reconstruction).
pub async fn daily_account_changes(
    pool: &PgPool,
    user_id: Uuid,
    account_ids: &[Uuid],
    from: Date,
    to: Date,
) -> Result<Vec<DailyBalanceRow>, DbError> {
    let rows = sqlx::query_as::<_, DailyBalanceRow>(
        "SELECT
           date,
           account_id,
           SUM(amount) AS daily_net
         FROM transactions
         WHERE user_id = $1
           AND account_id = ANY($2)
           AND date BETWEEN $3 AND $4
           AND NOT is_deleted
         GROUP BY date, account_id
         ORDER BY date",
    )
    .bind(user_id)
    .bind(account_ids)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Current balance and metadata for the requested accounts.
pub async fn account_balance_seeds(
    pool: &PgPool,
    user_id: Uuid,
    account_ids: &[Uuid],
) -> Result<Vec<AccountBalanceSeedRow>, DbError> {
    let rows = if account_ids.is_empty() {
        sqlx::query_as::<_, AccountBalanceSeedRow>(
            "SELECT id AS account_id, balance_cache, currency, name
             FROM accounts
             WHERE user_id = $1 AND NOT is_archived
             ORDER BY sort_order, name",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, AccountBalanceSeedRow>(
            "SELECT id AS account_id, balance_cache, currency, name
             FROM accounts
             WHERE user_id = $1 AND id = ANY($2) AND NOT is_archived
             ORDER BY sort_order, name",
        )
        .bind(user_id)
        .bind(account_ids)
        .fetch_all(pool)
        .await?
    };

    Ok(rows)
}

/// Sum of transactions after `after_date` per account (used to compute historical balance).
pub async fn transactions_after_date(
    pool: &PgPool,
    user_id: Uuid,
    account_ids: &[Uuid],
    after_date: Date,
) -> Result<Vec<FutureAmountRow>, DbError> {
    let rows = sqlx::query_as::<_, FutureAmountRow>(
        "SELECT account_id, SUM(amount) AS future_net
         FROM transactions
         WHERE user_id = $1
           AND account_id = ANY($2)
           AND date > $3
           AND NOT is_deleted
         GROUP BY account_id",
    )
    .bind(user_id)
    .bind(account_ids)
    .bind(after_date)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Category spending trend: total per month for the given category.
pub async fn category_trend(
    pool: &PgPool,
    user_id: Uuid,
    category_id: Uuid,
    from: Date,
    to: Date,
) -> Result<Vec<CategoryTrendRow>, DbError> {
    let rows = sqlx::query_as::<_, CategoryTrendRow>(
        "SELECT
           date_trunc('month', date)::date AS period,
           COALESCE(SUM(ABS(amount)), 0) AS total
         FROM transactions
         WHERE user_id = $1
           AND category_id = $2
           AND NOT is_deleted
           AND date BETWEEN $3 AND $4
         GROUP BY 1
         ORDER BY 1",
    )
    .bind(user_id)
    .bind(category_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Cash flow rows: income and expenses per month.
pub async fn cash_flow(
    pool: &PgPool,
    user_id: Uuid,
    from: Date,
    to: Date,
) -> Result<Vec<CashFlowRow>, DbError> {
    let rows = sqlx::query_as::<_, CashFlowRow>(
        "SELECT
           date_trunc('month', date)::date AS period,
           COALESCE(SUM(CASE WHEN amount > 0 AND transaction_type::text != 'transfer'
                              THEN amount ELSE 0 END), 0) AS income,
           COALESCE(SUM(CASE WHEN amount < 0 AND transaction_type::text != 'transfer'
                              THEN ABS(amount) ELSE 0 END), 0) AS expenses
         FROM transactions
         WHERE user_id = $1
           AND NOT is_deleted
           AND date BETWEEN $2 AND $3
         GROUP BY 1
         ORDER BY 1",
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
