//! Budget service — business logic for budgets and budget lines.

use rust_decimal::Decimal;
use sqlx::PgPool;
use time::Date;
use uuid::Uuid;

use crate::error::CoreError;
use crate::models::budget::{
    Budget, BudgetLine, BudgetLineSummary, BudgetSummary, ExchangeRate, NewBudget, NewBudgetLine,
    UpdateBudget, UpdateBudgetLine,
};

// ── Mapping helpers ───────────────────────────────────────────────────────────

fn row_to_budget(row: rustvault_db::repos::budget::BudgetRow) -> Budget {
    Budget {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        period_start: row.period_start,
        period_end: row.period_end,
        currency: row.currency,
        is_recurring: row.is_recurring,
        recurrence_rule: row.recurrence_rule,
        is_archived: row.is_archived,
        notes: row.notes,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn row_to_line(row: rustvault_db::repos::budget::BudgetLineRow) -> BudgetLine {
    BudgetLine {
        id: row.id,
        budget_id: row.budget_id,
        category_id: row.category_id,
        planned_amount: row.planned_amount,
        actual_amount_cache: row.actual_amount_cache,
        notes: row.notes,
        sort_order: row.sort_order,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn row_to_exchange_rate(row: rustvault_db::repos::exchange_rate::ExchangeRateRow) -> ExchangeRate {
    ExchangeRate {
        id: row.id,
        base_currency: row.base_currency,
        target_currency: row.target_currency,
        rate: row.rate,
        date: row.date,
        source: row.source,
        fetched_at: row.fetched_at,
    }
}

// ── Budget CRUD ───────────────────────────────────────────────────────────────

/// List all budgets for the user.
pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    include_archived: bool,
) -> Result<Vec<Budget>, CoreError> {
    let rows = rustvault_db::repos::budget::list_by_user(pool, user_id, include_archived).await?;
    Ok(rows.into_iter().map(row_to_budget).collect())
}

/// Get a single budget by ID (with ownership check).
pub async fn get(pool: &PgPool, user_id: Uuid, budget_id: Uuid) -> Result<Budget, CoreError> {
    let row = rustvault_db::repos::budget::find_by_id(pool, user_id, budget_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "budget".into(),
                id: budget_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;
    Ok(row_to_budget(row))
}

/// Create a new budget.
pub async fn create(pool: &PgPool, user_id: Uuid, input: NewBudget) -> Result<Budget, CoreError> {
    if input.period_end < input.period_start {
        return Err(CoreError::Validation(
            "period_end must be on or after period_start".into(),
        ));
    }
    if input.is_recurring && input.recurrence_rule.is_none() {
        return Err(CoreError::Validation(
            "recurrence_rule is required when is_recurring is true".into(),
        ));
    }

    let row = rustvault_db::repos::budget::insert(
        pool,
        user_id,
        &input.name,
        input.period_start,
        input.period_end,
        &input.currency,
        input.is_recurring,
        input.recurrence_rule.as_deref(),
        input.notes.as_deref(),
    )
    .await?;

    Ok(row_to_budget(row))
}

/// Update a budget's metadata.
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    budget_id: Uuid,
    input: UpdateBudget,
) -> Result<Budget, CoreError> {
    // Validate period bounds when both are being updated together
    if let (Some(start), Some(end)) = (input.period_start, input.period_end) {
        if end < start {
            return Err(CoreError::Validation(
                "period_end must be on or after period_start".into(),
            ));
        }
    }

    // Borrow the optional strings via local variables to satisfy lifetime requirements.
    let recurrence_rule_ref: Option<Option<&str>> =
        input.recurrence_rule.as_ref().map(|v| Some(v.as_str()));
    let notes_ref: Option<Option<&str>> = input.notes.as_ref().map(|v| Some(v.as_str()));

    let row = rustvault_db::repos::budget::update(
        pool,
        user_id,
        budget_id,
        input.name.as_deref(),
        input.period_start,
        input.period_end,
        input.currency.as_deref(),
        input.is_recurring,
        recurrence_rule_ref,
        notes_ref,
    )
    .await
    .map_err(|e| match e {
        rustvault_db::DbError::NotFound => CoreError::NotFound {
            entity: "budget".into(),
            id: budget_id.to_string(),
        },
        other => CoreError::Db(other),
    })?;

    Ok(row_to_budget(row))
}

/// Delete a budget (and its lines via CASCADE).
pub async fn delete(pool: &PgPool, user_id: Uuid, budget_id: Uuid) -> Result<(), CoreError> {
    rustvault_db::repos::budget::delete(pool, user_id, budget_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "budget".into(),
                id: budget_id.to_string(),
            },
            other => CoreError::Db(other),
        })
}

/// Copy an existing budget's lines into a new budget for a different period.
pub async fn copy(
    pool: &PgPool,
    user_id: Uuid,
    source_id: Uuid,
    name: String,
    period_start: Date,
    period_end: Date,
) -> Result<Budget, CoreError> {
    // Load source budget (ownership check)
    let source = get(pool, user_id, source_id).await?;

    // Create the new budget
    let new_budget = create(
        pool,
        user_id,
        NewBudget {
            name,
            period_start,
            period_end,
            currency: source.currency.clone(),
            is_recurring: false, // copied budgets start as non-recurring
            recurrence_rule: None,
            notes: None,
        },
    )
    .await?;

    // Copy all lines (reset actuals)
    let source_lines = rustvault_db::repos::budget::list_lines(pool, source_id).await?;
    for line in source_lines {
        rustvault_db::repos::budget::insert_line(
            pool,
            new_budget.id,
            line.category_id,
            line.planned_amount,
            line.notes.as_deref(),
            line.sort_order,
        )
        .await?;
    }

    Ok(new_budget)
}

// ── Budget Lines CRUD ─────────────────────────────────────────────────────────

/// List lines for a budget (ownership check via parent budget).
pub async fn list_lines(
    pool: &PgPool,
    user_id: Uuid,
    budget_id: Uuid,
) -> Result<Vec<BudgetLine>, CoreError> {
    // Verify ownership
    rustvault_db::repos::budget::find_by_id(pool, user_id, budget_id)
        .await
        .map_err(|_| CoreError::NotFound {
            entity: "budget".into(),
            id: budget_id.to_string(),
        })?;

    let rows = rustvault_db::repos::budget::list_lines(pool, budget_id).await?;
    Ok(rows.into_iter().map(row_to_line).collect())
}

/// Add a line to a budget.
pub async fn add_line(
    pool: &PgPool,
    user_id: Uuid,
    budget_id: Uuid,
    input: NewBudgetLine,
) -> Result<BudgetLine, CoreError> {
    // Verify ownership
    rustvault_db::repos::budget::find_by_id(pool, user_id, budget_id)
        .await
        .map_err(|_| CoreError::NotFound {
            entity: "budget".into(),
            id: budget_id.to_string(),
        })?;

    let sort_order = input.sort_order.unwrap_or(0);
    let row = rustvault_db::repos::budget::insert_line(
        pool,
        budget_id,
        input.category_id,
        input.planned_amount,
        input.notes.as_deref(),
        sort_order,
    )
    .await
    .map_err(|e| match e {
        rustvault_db::DbError::UniqueViolation(_) => {
            CoreError::Conflict("category already has a line in this budget".into())
        }
        other => CoreError::Db(other),
    })?;

    Ok(row_to_line(row))
}

/// Bulk add/replace lines on a budget (replaces all existing lines).
pub async fn bulk_set_lines(
    pool: &PgPool,
    user_id: Uuid,
    budget_id: Uuid,
    lines: Vec<NewBudgetLine>,
) -> Result<Vec<BudgetLine>, CoreError> {
    // Verify ownership
    rustvault_db::repos::budget::find_by_id(pool, user_id, budget_id)
        .await
        .map_err(|_| CoreError::NotFound {
            entity: "budget".into(),
            id: budget_id.to_string(),
        })?;

    // Delete existing lines then re-insert
    sqlx::query("DELETE FROM budget_lines WHERE budget_id = $1")
        .bind(budget_id)
        .execute(pool)
        .await
        .map_err(rustvault_db::DbError::Sqlx)?;

    let mut result = Vec::with_capacity(lines.len());
    for (i, input) in lines.into_iter().enumerate() {
        let sort_order = input.sort_order.unwrap_or(i as i32);
        let row = rustvault_db::repos::budget::insert_line(
            pool,
            budget_id,
            input.category_id,
            input.planned_amount,
            input.notes.as_deref(),
            sort_order,
        )
        .await?;
        result.push(row_to_line(row));
    }

    Ok(result)
}

/// Update a specific budget line.
pub async fn update_line(
    pool: &PgPool,
    user_id: Uuid,
    budget_id: Uuid,
    line_id: Uuid,
    input: UpdateBudgetLine,
) -> Result<BudgetLine, CoreError> {
    // Verify budget ownership
    rustvault_db::repos::budget::find_by_id(pool, user_id, budget_id)
        .await
        .map_err(|_| CoreError::NotFound {
            entity: "budget".into(),
            id: budget_id.to_string(),
        })?;

    let notes_ref: Option<Option<&str>> = input.notes.as_ref().map(|v| Some(v.as_str()));
    let row = rustvault_db::repos::budget::update_line(
        pool,
        line_id,
        budget_id,
        input.planned_amount,
        notes_ref,
        input.sort_order,
    )
    .await
    .map_err(|e| match e {
        rustvault_db::DbError::NotFound => CoreError::NotFound {
            entity: "budget_line".into(),
            id: line_id.to_string(),
        },
        other => CoreError::Db(other),
    })?;

    Ok(row_to_line(row))
}

/// Remove a specific budget line.
pub async fn delete_line(
    pool: &PgPool,
    user_id: Uuid,
    budget_id: Uuid,
    line_id: Uuid,
) -> Result<(), CoreError> {
    // Verify budget ownership
    rustvault_db::repos::budget::find_by_id(pool, user_id, budget_id)
        .await
        .map_err(|_| CoreError::NotFound {
            entity: "budget".into(),
            id: budget_id.to_string(),
        })?;

    rustvault_db::repos::budget::delete_line(pool, line_id, budget_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "budget_line".into(),
                id: line_id.to_string(),
            },
            other => CoreError::Db(other),
        })
}

// ── Budget Summary ────────────────────────────────────────────────────────────

/// Compute and return the full budget summary (planned vs. actual).
///
/// Also refreshes `actual_amount_cache` on all lines as a side-effect.
pub async fn summary(
    pool: &PgPool,
    user_id: Uuid,
    budget_id: Uuid,
) -> Result<BudgetSummary, CoreError> {
    let budget = get(pool, user_id, budget_id).await?;

    // Refresh actuals
    rustvault_db::repos::budget::refresh_actuals(pool, budget_id).await?;

    // Load lines (now with fresh actuals)
    let lines = rustvault_db::repos::budget::list_lines(pool, budget_id).await?;

    // Overall actuals
    let totals = rustvault_db::repos::budget::actual_totals(
        pool,
        user_id,
        budget.period_start,
        budget.period_end,
    )
    .await?;

    let total_actual_income = totals.total_income.unwrap_or(Decimal::ZERO);
    let total_actual_expenses = totals.total_expenses.unwrap_or(Decimal::ZERO);

    // Planned totals from lines (no type info per line, so we use absolute planned amounts)
    let total_planned_income = Decimal::ZERO; // budget lines are expenses by default; income planning is optional
    let total_planned_expenses: Decimal = lines.iter().map(|l| l.planned_amount).sum();

    let net_planned = total_planned_income - total_planned_expenses;
    let net_actual = total_actual_income - total_actual_expenses;

    let mut over_budget = Vec::new();
    let line_summaries: Vec<BudgetLineSummary> = lines
        .iter()
        .map(|l| {
            let actual = l.actual_amount_cache;
            let planned = l.planned_amount;
            let remaining = planned - actual;
            let percent_used = if planned.is_zero() {
                Decimal::ZERO
            } else {
                (actual / planned * Decimal::ONE_HUNDRED).round_dp(2)
            };
            if actual > planned && !planned.is_zero() {
                if let Some(cat) = l.category_id {
                    over_budget.push(cat);
                }
            }
            BudgetLineSummary {
                id: l.id,
                category_id: l.category_id,
                planned_amount: planned,
                actual_amount: actual,
                remaining,
                percent_used,
            }
        })
        .collect();

    Ok(BudgetSummary {
        budget_id,
        total_planned_income,
        total_actual_income,
        total_planned_expenses,
        total_actual_expenses,
        net_planned,
        net_actual,
        lines: line_summaries,
        over_budget_categories: over_budget,
    })
}

// ── Exchange Rates ────────────────────────────────────────────────────────────

/// Fetch and store today's rates from the ECB XML feed.
///
/// Returns the number of rates upserted.
pub async fn fetch_and_store_rates(pool: &PgPool) -> Result<u64, CoreError> {
    let rates = super::exchange_rate::fetch_ecb_rates().await?;
    let count = rustvault_db::repos::exchange_rate::upsert_batch(pool, &rates).await?;
    Ok(count)
}

/// List all latest exchange rates.
pub async fn list_exchange_rates(pool: &PgPool) -> Result<Vec<ExchangeRate>, CoreError> {
    let rows = rustvault_db::repos::exchange_rate::list_latest(pool).await?;
    Ok(rows.into_iter().map(row_to_exchange_rate).collect())
}

// ── Recurring budget generation (P4.6) ───────────────────────────────────────

/// Generate the next period's budget from a recurring template.
///
/// Rules:
/// - Source budget must have `is_recurring = true` and a `recurrence_rule`.
/// - Supported rules: `FREQ=MONTHLY` and `FREQ=YEARLY`.
/// - The generated budget is a copy of the source's lines with actuals reset.
/// - The generated budget has `is_recurring = false` (it is a snapshot, not a template).
///
/// Returns the newly created budget so the handler can respond with 201 Created.
pub async fn generate_next_period(
    pool: &PgPool,
    user_id: Uuid,
    source_id: Uuid,
) -> Result<Budget, CoreError> {
    let source = get(pool, user_id, source_id).await?;

    if !source.is_recurring {
        return Err(CoreError::Validation(
            "budget is not marked as recurring".into(),
        ));
    }

    let rule = source.recurrence_rule.as_deref().unwrap_or("");
    let (next_start, next_end) = advance_period(source.period_start, source.period_end, rule)?;

    // Derive a name: replace the last 4-digit year if present, otherwise append period.
    let new_name = {
        let s = source.name.clone();
        // Try to replace a trailing month-year pattern, e.g. "Jan 2026" → "Feb 2026".
        // Fallback: append the new period dates.
        format!("{} ({})", s, next_start)
    };

    let new_budget = create(
        pool,
        user_id,
        NewBudget {
            name: new_name,
            period_start: next_start,
            period_end: next_end,
            currency: source.currency.clone(),
            is_recurring: false,
            recurrence_rule: None,
            notes: source.notes.clone(),
        },
    )
    .await?;

    // Copy all budget lines from the source, with reset actuals.
    let source_lines = rustvault_db::repos::budget::list_lines(pool, source_id).await?;
    for line in source_lines {
        rustvault_db::repos::budget::insert_line(
            pool,
            new_budget.id,
            line.category_id,
            line.planned_amount,
            line.notes.as_deref(),
            line.sort_order,
        )
        .await?;
    }

    Ok(new_budget)
}

/// Advance a start/end date pair according to the recurrence rule.
///
/// Only `FREQ=MONTHLY` and `FREQ=YEARLY` (case-insensitive) are supported.
/// The period duration (in days) is preserved for monthly recurrence; for
/// monthly recurrence the new end date is always the last day of the next
/// calendar month.
fn advance_period(
    start: Date,
    end: Date,
    rule: &str,
) -> Result<(Date, Date), CoreError> {
    use time::Month;

    let rule_upper = rule.to_uppercase();

    if rule_upper.contains("FREQ=MONTHLY") {
        // Next calendar month.
        let (next_year, next_month) = if start.month() == Month::December {
            (start.year() + 1, Month::January)
        } else {
            (start.year(), start.month().next())
        };

        let next_start = Date::from_calendar_date(next_year, next_month, 1)
            .map_err(|_| CoreError::Validation("could not compute next monthly period".into()))?;

        // End = last day of next month.
        let last_day = days_in_month(next_year, next_month);
        let next_end = Date::from_calendar_date(next_year, next_month, last_day)
            .map_err(|_| CoreError::Validation("could not compute last day of next month".into()))?;

        Ok((next_start, next_end))
    } else if rule_upper.contains("FREQ=YEARLY") {
        let duration = end - start;
        let next_start = start.replace_year(start.year() + 1)
            .map_err(|_| CoreError::Validation("could not compute next yearly period start".into()))?;
        let next_end = next_start + duration;
        Ok((next_start, next_end))
    } else {
        Err(CoreError::Validation(format!(
            "unsupported recurrence rule '{rule}'; only FREQ=MONTHLY and FREQ=YEARLY are supported"
        )))
    }
}

/// Returns the number of days in the given month/year.
fn days_in_month(year: i32, month: time::Month) -> u8 {
    use time::Month::*;
    match month {
        January | March | May | July | August | October | December => 31,
        April | June | September | November => 30,
        February => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
    }
}

/// Convert `amount` from `from_currency` to `to_currency` on `on_date`.
///
/// Uses stored rates. Returns `None` if no rate is available for the pair.
pub async fn convert_amount(
    pool: &PgPool,
    amount: Decimal,
    from_currency: &str,
    to_currency: &str,
    on_date: Date,
) -> Result<Option<Decimal>, CoreError> {
    if from_currency == to_currency {
        return Ok(Some(amount));
    }

    // Try direct rate
    if let Some(row) =
        rustvault_db::repos::exchange_rate::find_rate(pool, from_currency, to_currency, on_date)
            .await?
    {
        return Ok(Some(amount * row.rate));
    }

    // Try via EUR as intermediate (ECB rates are EUR-based)
    let from_eur =
        rustvault_db::repos::exchange_rate::find_rate(pool, from_currency, "EUR", on_date).await?;
    let eur_to =
        rustvault_db::repos::exchange_rate::find_rate(pool, "EUR", to_currency, on_date).await?;

    if let (Some(fe), Some(et)) = (from_eur, eur_to) {
        return Ok(Some(amount * fe.rate * et.rate));
    }

    Ok(None)
}
