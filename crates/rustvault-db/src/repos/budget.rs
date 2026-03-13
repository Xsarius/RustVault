//! Budget repository — SQL operations on `budgets` and `budget_lines` tables.

use rust_decimal::Decimal;
use sqlx::PgPool;
use time::Date;
use uuid::Uuid;

use crate::error::DbError;

// ── Row types ─────────────────────────────────────────────────────────────────

/// Row type for the `budgets` table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BudgetRow {
    /// Budget ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Display name.
    pub name: String,
    /// Period start.
    pub period_start: Date,
    /// Period end.
    pub period_end: Date,
    /// Reporting currency.
    pub currency: String,
    /// Recurring flag.
    pub is_recurring: bool,
    /// iCal RRULE string.
    pub recurrence_rule: Option<String>,
    /// Archived flag.
    pub is_archived: bool,
    /// Optional notes.
    pub notes: Option<String>,
    /// Metadata (JSONB).
    pub metadata: serde_json::Value,
    /// Creation timestamp.
    pub created_at: time::OffsetDateTime,
    /// Last update timestamp.
    pub updated_at: time::OffsetDateTime,
}

/// Row type for the `budget_lines` table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BudgetLineRow {
    /// Budget line ID.
    pub id: Uuid,
    /// Parent budget ID.
    pub budget_id: Uuid,
    /// Category ID (nullable).
    pub category_id: Option<Uuid>,
    /// Planned amount.
    pub planned_amount: Decimal,
    /// Cached actual amount.
    pub actual_amount_cache: Decimal,
    /// Optional notes.
    pub notes: Option<String>,
    /// Sort order.
    pub sort_order: i32,
    /// Creation timestamp.
    pub created_at: time::OffsetDateTime,
    /// Last update timestamp.
    pub updated_at: time::OffsetDateTime,
}

// ── Budget queries ────────────────────────────────────────────────────────────

/// List all budgets for a user, ordered by period_start descending.
pub async fn list_by_user(
    pool: &PgPool,
    user_id: Uuid,
    include_archived: bool,
) -> Result<Vec<BudgetRow>, DbError> {
    let rows = sqlx::query_as::<_, BudgetRow>(
        "SELECT id, user_id, name, period_start, period_end, currency,
                is_recurring, recurrence_rule, is_archived, notes, metadata,
                created_at, updated_at
         FROM budgets
         WHERE user_id = $1
           AND ($2 OR NOT is_archived)
         ORDER BY period_start DESC",
    )
    .bind(user_id)
    .bind(include_archived)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Fetch a single budget by ID, asserting ownership.
pub async fn find_by_id(pool: &PgPool, user_id: Uuid, budget_id: Uuid) -> Result<BudgetRow, DbError> {
    let row = sqlx::query_as::<_, BudgetRow>(
        "SELECT id, user_id, name, period_start, period_end, currency,
                is_recurring, recurrence_rule, is_archived, notes, metadata,
                created_at, updated_at
         FROM budgets
         WHERE id = $1 AND user_id = $2",
    )
    .bind(budget_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)?;

    Ok(row)
}

/// Insert a new budget.
#[allow(clippy::too_many_arguments)]
pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    period_start: Date,
    period_end: Date,
    currency: &str,
    is_recurring: bool,
    recurrence_rule: Option<&str>,
    notes: Option<&str>,
) -> Result<BudgetRow, DbError> {
    let row = sqlx::query_as::<_, BudgetRow>(
        "INSERT INTO budgets
                (user_id, name, period_start, period_end, currency,
                 is_recurring, recurrence_rule, notes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, user_id, name, period_start, period_end, currency,
                   is_recurring, recurrence_rule, is_archived, notes, metadata,
                   created_at, updated_at",
    )
    .bind(user_id)
    .bind(name)
    .bind(period_start)
    .bind(period_end)
    .bind(currency)
    .bind(is_recurring)
    .bind(recurrence_rule)
    .bind(notes)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Update mutable fields on an existing budget.
#[allow(clippy::too_many_arguments)]
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    budget_id: Uuid,
    name: Option<&str>,
    period_start: Option<Date>,
    period_end: Option<Date>,
    currency: Option<&str>,
    is_recurring: Option<bool>,
    recurrence_rule: Option<Option<&str>>,
    notes: Option<Option<&str>>,
) -> Result<BudgetRow, DbError> {
    let row = sqlx::query_as::<_, BudgetRow>(
        "UPDATE budgets SET
            name             = COALESCE($3, name),
            period_start     = COALESCE($4, period_start),
            period_end       = COALESCE($5, period_end),
            currency         = COALESCE($6, currency),
            is_recurring     = COALESCE($7, is_recurring),
            recurrence_rule  = CASE WHEN $8 THEN $9 ELSE recurrence_rule END,
            notes            = CASE WHEN $10 THEN $11 ELSE notes END,
            updated_at       = now()
         WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, name, period_start, period_end, currency,
                   is_recurring, recurrence_rule, is_archived, notes, metadata,
                   created_at, updated_at",
    )
    .bind(budget_id)
    .bind(user_id)
    .bind(name)
    .bind(period_start)
    .bind(period_end)
    .bind(currency)
    .bind(is_recurring)
    .bind(recurrence_rule.is_some())        // $8 — update sentinel
    .bind(recurrence_rule.flatten())        // $9 — new value (may be NULL)
    .bind(notes.is_some())                  // $10 — update sentinel
    .bind(notes.flatten())                  // $11 — new value (may be NULL)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)?;

    Ok(row)
}

/// Soft-delete a budget by marking it archived.
pub async fn archive(pool: &PgPool, user_id: Uuid, budget_id: Uuid) -> Result<(), DbError> {
    let affected = sqlx::query(
        "UPDATE budgets SET is_archived = true, updated_at = now()
         WHERE id = $1 AND user_id = $2",
    )
    .bind(budget_id)
    .bind(user_id)
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Hard-delete a budget (and its lines via CASCADE).
pub async fn delete(pool: &PgPool, user_id: Uuid, budget_id: Uuid) -> Result<(), DbError> {
    let affected = sqlx::query("DELETE FROM budgets WHERE id = $1 AND user_id = $2")
        .bind(budget_id)
        .bind(user_id)
        .execute(pool)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

// ── Budget Line queries ───────────────────────────────────────────────────────

/// Fetch all lines for a budget in sort_order.
pub async fn list_lines(pool: &PgPool, budget_id: Uuid) -> Result<Vec<BudgetLineRow>, DbError> {
    let rows = sqlx::query_as::<_, BudgetLineRow>(
        "SELECT id, budget_id, category_id, planned_amount, actual_amount_cache,
                notes, sort_order, created_at, updated_at
         FROM budget_lines
         WHERE budget_id = $1
         ORDER BY sort_order, created_at",
    )
    .bind(budget_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Fetch a single budget line by ID.
pub async fn find_line_by_id(pool: &PgPool, line_id: Uuid) -> Result<BudgetLineRow, DbError> {
    let row = sqlx::query_as::<_, BudgetLineRow>(
        "SELECT id, budget_id, category_id, planned_amount, actual_amount_cache,
                notes, sort_order, created_at, updated_at
         FROM budget_lines WHERE id = $1",
    )
    .bind(line_id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)?;

    Ok(row)
}

/// Insert a new budget line.
pub async fn insert_line(
    pool: &PgPool,
    budget_id: Uuid,
    category_id: Option<Uuid>,
    planned_amount: Decimal,
    notes: Option<&str>,
    sort_order: i32,
) -> Result<BudgetLineRow, DbError> {
    let row = sqlx::query_as::<_, BudgetLineRow>(
        "INSERT INTO budget_lines (budget_id, category_id, planned_amount, notes, sort_order)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, budget_id, category_id, planned_amount, actual_amount_cache,
                   notes, sort_order, created_at, updated_at",
    )
    .bind(budget_id)
    .bind(category_id)
    .bind(planned_amount)
    .bind(notes)
    .bind(sort_order)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err)
            if db_err.constraint() == Some("uq_budget_line_category") =>
        {
            DbError::UniqueViolation("category already has a line in this budget".into())
        }
        _ => DbError::Sqlx(e),
    })?;

    Ok(row)
}

/// Update a single budget line.
pub async fn update_line(
    pool: &PgPool,
    line_id: Uuid,
    budget_id: Uuid,
    planned_amount: Option<Decimal>,
    notes: Option<Option<&str>>,
    sort_order: Option<i32>,
) -> Result<BudgetLineRow, DbError> {
    let row = sqlx::query_as::<_, BudgetLineRow>(
        "UPDATE budget_lines SET
            planned_amount = COALESCE($3, planned_amount),
            notes          = CASE WHEN $4 THEN $5 ELSE notes END,
            sort_order     = COALESCE($6, sort_order),
            updated_at     = now()
         WHERE id = $1 AND budget_id = $2
         RETURNING id, budget_id, category_id, planned_amount, actual_amount_cache,
                   notes, sort_order, created_at, updated_at",
    )
    .bind(line_id)
    .bind(budget_id)
    .bind(planned_amount)
    .bind(notes.is_some())
    .bind(notes.flatten())
    .bind(sort_order)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)?;

    Ok(row)
}

/// Delete a budget line.
pub async fn delete_line(pool: &PgPool, line_id: Uuid, budget_id: Uuid) -> Result<(), DbError> {
    let affected =
        sqlx::query("DELETE FROM budget_lines WHERE id = $1 AND budget_id = $2")
            .bind(line_id)
            .bind(budget_id)
            .execute(pool)
            .await?
            .rows_affected();

    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Refresh the `actual_amount_cache` on all lines of a budget.
///
/// Computes the sum of non-deleted, non-transfer transactions for each
/// category within the budget period, in the budget's reporting currency.
/// Currency conversion is approximate (assumes amounts are already in the
/// budget currency — full multi-currency conversion is handled by the service
/// layer using stored `exchange_rates`).
pub async fn refresh_actuals(pool: &PgPool, budget_id: Uuid) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE budget_lines bl
         SET actual_amount_cache = COALESCE((
             SELECT SUM(ABS(t.amount))
             FROM transactions t
             JOIN budgets b ON b.id = $1
             WHERE t.category_id = bl.category_id
               AND t.user_id = b.user_id
               AND t.date BETWEEN b.period_start AND b.period_end
               AND t.transaction_type != 'transfer'
               AND NOT t.is_deleted
         ), 0),
         updated_at = now()
         WHERE bl.budget_id = $1",
    )
    .bind(budget_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ── Actuals for summary ───────────────────────────────────────────────────────

/// Row returned by the actual income/expense aggregation query.
#[derive(Debug, sqlx::FromRow)]
pub struct ActualTotalsRow {
    /// Total income in the period.
    pub total_income: Option<Decimal>,
    /// Total expenses in the period.
    pub total_expenses: Option<Decimal>,
}

/// Compute total actual income and expenses for a budget period.
pub async fn actual_totals(
    pool: &PgPool,
    user_id: Uuid,
    period_start: Date,
    period_end: Date,
) -> Result<ActualTotalsRow, DbError> {
    let row = sqlx::query_as::<_, ActualTotalsRow>(
        "SELECT
            SUM(CASE WHEN transaction_type = 'income' THEN ABS(amount) ELSE 0 END) AS total_income,
            SUM(CASE WHEN transaction_type = 'expense' THEN ABS(amount) ELSE 0 END) AS total_expenses
         FROM transactions
         WHERE user_id = $1
           AND date BETWEEN $2 AND $3
           AND transaction_type != 'transfer'
           AND NOT is_deleted",
    )
    .bind(user_id)
    .bind(period_start)
    .bind(period_end)
    .fetch_one(pool)
    .await?;

    Ok(row)
}
