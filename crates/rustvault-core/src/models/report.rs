//! Report domain models — response types for visualisation and analysis endpoints.

use rust_decimal::Decimal;
use serde::Serialize;
use time::Date;
use uuid::Uuid;

// ── Dashboard Summary ─────────────────────────────────────────────────────────

/// Quick-stats summary returned by `GET /api/reports/summary`.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct DashboardSummary {
    /// Current net worth (sum of all non-archived account balances).
    pub net_worth: Decimal,
    /// Total income in the current calendar month.
    pub month_income: Decimal,
    /// Total expenses in the current calendar month (positive value).
    pub month_expenses: Decimal,
    /// Savings rate for the current month: `(income - expenses) / income * 100`.
    /// `None` if income is zero.
    pub savings_rate: Option<f64>,
    /// Number of transactions not yet reviewed by the user.
    pub unreviewed_count: i64,
    /// Monthly income and expenses for the last 12 months (oldest first).
    pub monthly_trend: Vec<MonthlyPoint>,
    /// Top spending categories this calendar month.
    pub spending_by_category: Vec<CategorySpend>,
}

/// One month's income/expense totals.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonthlyPoint {
    /// ISO 8601 date string (first day of the month, e.g. `"2025-11-01"`).
    #[schema(value_type = String, format = Date)]
    pub month: Date,
    /// Income that month.
    pub income: Decimal,
    /// Expenses that month (positive value).
    pub expenses: Decimal,
}

/// Total spending for a category in a given period.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CategorySpend {
    /// Category ID (`None` = uncategorised).
    pub category_id: Option<Uuid>,
    /// Category display name (`None` = uncategorised).
    pub category_name: Option<String>,
    /// Absolute total.
    pub total: Decimal,
}

// ── Income vs Expense ─────────────────────────────────────────────────────────

/// Response for `GET /api/reports/income-expense`.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct IncomeExpenseReport {
    /// Monthly breakdown (oldest first).
    pub months: Vec<IncomeExpenseMonth>,
}

/// One month in the income/expense report.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct IncomeExpenseMonth {
    /// First day of the month.
    #[schema(value_type = String, format = Date)]
    pub month: Date,
    /// Total income (all categories).
    pub income: Decimal,
    /// Total expenses (all categories, positive).
    pub expenses: Decimal,
    /// Per-category/income-source breakdown.
    pub breakdown: Vec<CategorySpend>,
}

// ── Category Trend ────────────────────────────────────────────────────────────

/// Response for `GET /api/reports/category/:id/trend`.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CategoryTrendReport {
    /// Category ID queried.
    pub category_id: Uuid,
    /// Monthly data points (oldest first).
    pub periods: Vec<TrendPoint>,
    /// Simple moving average over the whole range.
    pub average: Decimal,
}

/// One period's value in a trend report.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct TrendPoint {
    /// Period start date (first day of the month).
    #[schema(value_type = String, format = Date)]
    pub period: Date,
    /// Total for this period.
    pub total: Decimal,
}

// ── Balance History ───────────────────────────────────────────────────────────

/// Response for `GET /api/reports/balance-history`.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct BalanceHistoryReport {
    /// Account metadata keyed by account ID.
    pub accounts: Vec<AccountMeta>,
    /// Date-ordered balance snapshots (downsampled if > 1000 points).
    pub snapshots: Vec<BalanceSnapshot>,
}

/// Lightweight account info embedded in the balance history report.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AccountMeta {
    /// Account ID.
    pub id: Uuid,
    /// Account display name.
    pub name: String,
    /// ISO 4217 currency.
    pub currency: String,
}

/// Balance for all requested accounts on a single date.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct BalanceSnapshot {
    /// Date of this snapshot.
    #[schema(value_type = String, format = Date)]
    pub date: Date,
    /// Balance per account (account_id → amount).
    pub balances: Vec<AccountBalance>,
    /// Combined net worth across all accounts (converted to reporting currency).
    pub net_worth: Decimal,
}

/// One account's balance on a given date.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AccountBalance {
    /// Account ID.
    pub account_id: Uuid,
    /// Balance on this date.
    pub balance: Decimal,
}

// ── Cash Flow ─────────────────────────────────────────────────────────────────

/// Response for `GET /api/reports/cash-flow`.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CashFlowReport {
    /// Historical periods (oldest first).
    pub periods: Vec<CashFlowPeriod>,
    /// Average monthly income over the range.
    pub avg_income: Decimal,
    /// Average monthly expenses over the range.
    pub avg_expenses: Decimal,
    /// Projected next 3 months based on averages.
    pub forecast: Vec<CashFlowPeriod>,
}

/// One period's cash flow.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CashFlowPeriod {
    /// Period start date.
    #[schema(value_type = String, format = Date)]
    pub period: Date,
    /// Total income.
    pub income: Decimal,
    /// Total expenses (positive).
    pub expenses: Decimal,
    /// Net cash flow: `income - expenses`.
    pub net: Decimal,
    /// `true` for forecast periods, `false` for historical.
    pub is_forecast: bool,
}
