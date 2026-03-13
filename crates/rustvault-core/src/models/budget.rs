//! Budget domain models.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use uuid::Uuid;
use validator::Validate;

// ── Budget ────────────────────────────────────────────────────────────────────

/// A planned spending envelope covering a date range.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Budget {
    /// Unique identifier.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Human-readable name, e.g. "March 2026".
    pub name: String,
    /// Inclusive start of the planning period.
    pub period_start: Date,
    /// Inclusive end of the planning period.
    pub period_end: Date,
    /// Reporting currency (ISO 4217).
    pub currency: String,
    /// Whether this budget recurs automatically.
    pub is_recurring: bool,
    /// iCal RRULE string if recurring, e.g. `"FREQ=MONTHLY"`.
    pub recurrence_rule: Option<String>,
    /// Whether archived.
    pub is_archived: bool,
    /// Optional notes.
    pub notes: Option<String>,
    /// Extensible metadata.
    pub metadata: serde_json::Value,
    /// Creation timestamp.
    pub created_at: OffsetDateTime,
    /// Last update timestamp.
    pub updated_at: OffsetDateTime,
}

/// Data required to create a new budget.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct NewBudget {
    /// Display name.
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    /// Period start (inclusive).
    pub period_start: Date,
    /// Period end (inclusive).
    pub period_end: Date,
    /// Reporting currency.
    #[validate(length(min = 3, max = 3))]
    pub currency: String,
    /// Whether budget auto-recurs.
    #[serde(default)]
    pub is_recurring: bool,
    /// iCal RRULE — required when `is_recurring` is true.
    pub recurrence_rule: Option<String>,
    /// Optional notes.
    pub notes: Option<String>,
}

/// Data for updating an existing budget's metadata.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdateBudget {
    /// New display name.
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,
    /// New period start.
    pub period_start: Option<Date>,
    /// New period end.
    pub period_end: Option<Date>,
    /// New reporting currency.
    #[validate(length(min = 3, max = 3))]
    pub currency: Option<String>,
    /// Toggle recurring.
    pub is_recurring: Option<bool>,
    /// New recurrence rule.
    pub recurrence_rule: Option<String>,
    /// New notes.
    pub notes: Option<String>,
}

// ── BudgetLine ────────────────────────────────────────────────────────────────

/// A single per-category planned amount within a budget.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct BudgetLine {
    /// Unique identifier.
    pub id: Uuid,
    /// Parent budget ID.
    pub budget_id: Uuid,
    /// Associated category (nullable — `None` means "unallocated").
    pub category_id: Option<Uuid>,
    /// Planned spending limit in the budget's currency.
    pub planned_amount: Decimal,
    /// Cached actual spending (refreshed on demand).
    pub actual_amount_cache: Decimal,
    /// Optional notes for this line.
    pub notes: Option<String>,
    /// Display sort order.
    pub sort_order: i32,
    /// Creation timestamp.
    pub created_at: OffsetDateTime,
    /// Last update timestamp.
    pub updated_at: OffsetDateTime,
}

/// Data for creating or upserting a single budget line.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct NewBudgetLine {
    /// Category to budget (null = unallocated).
    pub category_id: Option<Uuid>,
    /// Planned amount (must be ≥ 0).
    pub planned_amount: Decimal,
    /// Optional notes.
    pub notes: Option<String>,
    /// Sort order.
    pub sort_order: Option<i32>,
}

/// Data for updating an existing budget line.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdateBudgetLine {
    /// New planned amount.
    pub planned_amount: Option<Decimal>,
    /// New notes.
    pub notes: Option<String>,
    /// New sort order.
    pub sort_order: Option<i32>,
}

/// Bulk set (upsert) of multiple budget lines.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BulkBudgetLines {
    /// Lines to upsert into the budget.
    pub lines: Vec<NewBudgetLine>,
}

// ── BudgetSummary ─────────────────────────────────────────────────────────────

/// Aggregate view of a budget's planned vs. actual state.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct BudgetSummary {
    /// Budget identifier.
    pub budget_id: Uuid,
    /// Total planned income.
    pub total_planned_income: Decimal,
    /// Total actual income in the period.
    pub total_actual_income: Decimal,
    /// Total planned expenses.
    pub total_planned_expenses: Decimal,
    /// Total actual expenses in the period.
    pub total_actual_expenses: Decimal,
    /// Planned net (income − expenses).
    pub net_planned: Decimal,
    /// Actual net (income − expenses).
    pub net_actual: Decimal,
    /// Per-category breakdown.
    pub lines: Vec<BudgetLineSummary>,
    /// Categories whose actual expenses exceed their planned limit.
    pub over_budget_categories: Vec<Uuid>,
}

/// Summary for a single budget line within a [`BudgetSummary`].
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct BudgetLineSummary {
    /// Budget line ID.
    pub id: Uuid,
    /// Category ID (nullable).
    pub category_id: Option<Uuid>,
    /// Planned amount.
    pub planned_amount: Decimal,
    /// Actual amount.
    pub actual_amount: Decimal,
    /// Remaining budget (planned − actual, may be negative if over budget).
    pub remaining: Decimal,
    /// Percentage of budget consumed (0–100+).
    pub percent_used: Decimal,
}

// ── ExchangeRate ──────────────────────────────────────────────────────────────

/// A single daily currency exchange rate.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ExchangeRate {
    /// DB row ID.
    pub id: i64,
    /// Base currency (e.g. "EUR" for ECB rates).
    pub base_currency: String,
    /// Target currency (e.g. "USD", "PLN").
    pub target_currency: String,
    /// Conversion rate (1 base = `rate` target).
    pub rate: Decimal,
    /// Date for which this rate is valid.
    pub date: Date,
    /// Source of the rate data.
    pub source: String,
    /// When this row was fetched.
    pub fetched_at: OffsetDateTime,
}
