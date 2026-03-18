//! Report service — aggregation logic for visualisation and analysis.
//!
//! Queries range from simple sums (monthly totals) to multi-step reconstructions
//! (historical account balances). All queries enforce `user_id` row-level security.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sqlx::PgPool;
use std::collections::HashMap;
use time::{Date, Month};
use uuid::Uuid;

use crate::error::CoreError;
use crate::models::report::{
    AccountBalance, AccountMeta, BalanceHistoryReport, BalanceSnapshot, CashFlowPeriod,
    CashFlowReport, CategorySpend, CategoryTrendReport, DashboardSummary, IncomeExpenseMonth,
    IncomeExpenseReport, MonthlyPoint, TrendPoint,
};

// ── LTTB downsampling ─────────────────────────────────────────────────────────

/// Largest-Triangle-Three-Buckets (LTTB) algorithm.
///
/// Reduces a sorted series of `(x, y)` pairs to at most `threshold` points
/// while preserving the visual shape of the curve.
fn lttb(data: &[(f64, f64)], threshold: usize) -> Vec<(f64, f64)> {
    let len = data.len();
    if threshold >= len || len <= 2 {
        return data.to_vec();
    }

    let mut sampled = Vec::with_capacity(threshold);
    sampled.push(data[0]);

    let bucket_size = (len - 2) as f64 / (threshold - 2) as f64;
    let mut a = 0usize; // point selected in previous bucket

    for i in 0..(threshold - 2) {
        // Calculate bucket boundaries
        let avg_range_start = ((i + 1) as f64 * bucket_size + 1.0) as usize;
        let avg_range_end = (((i + 2) as f64 * bucket_size) as usize).min(len);

        // Average point of next bucket
        let avg_x: f64 = data[avg_range_start..avg_range_end]
            .iter()
            .map(|p| p.0)
            .sum::<f64>()
            / (avg_range_end - avg_range_start) as f64;
        let avg_y: f64 = data[avg_range_start..avg_range_end]
            .iter()
            .map(|p| p.1)
            .sum::<f64>()
            / (avg_range_end - avg_range_start) as f64;

        // Current bucket
        let range_start = ((i as f64 * bucket_size) as usize + 1).min(len - 1);
        let range_end = (((i + 1) as f64 * bucket_size) as usize + 1).min(len);

        let (ax, ay) = data[a];
        let mut max_area = -1.0f64;
        let mut max_index = range_start;

        for j in range_start..range_end {
            let area =
                ((ax - avg_x) * (data[j].1 - ay) - (ax - data[j].0) * (avg_y - ay)).abs() * 0.5;
            if area > max_area {
                max_area = area;
                max_index = j;
            }
        }

        sampled.push(data[max_index]);
        a = max_index;
    }

    sampled.push(*data.last().unwrap());
    sampled
}

// ── Dashboard summary ─────────────────────────────────────────────────────────

/// Return the dashboard summary for a user:
/// net worth, current-month totals, monthly I/E trend, and category breakdown.
pub async fn summary(pool: &PgPool, user_id: Uuid) -> Result<DashboardSummary, CoreError> {
    let totals = rustvault_db::repos::report::summary_totals(pool, user_id).await?;
    let monthly_rows =
        rustvault_db::repos::report::monthly_income_expense(pool, user_id, 12).await?;
    let category_rows =
        rustvault_db::repos::report::spending_by_category(pool, user_id, 10).await?;

    let net_worth = totals.net_worth.unwrap_or(Decimal::ZERO);
    let month_income = totals.month_income.unwrap_or(Decimal::ZERO);
    let month_expenses = totals.month_expenses.unwrap_or(Decimal::ZERO);

    let savings_rate = if month_income > Decimal::ZERO {
        let rate = ((month_income - month_expenses) / month_income * Decimal::from(100)).to_f64();
        rate
    } else {
        None
    };

    let monthly_trend: Vec<MonthlyPoint> = monthly_rows
        .into_iter()
        .map(|r| MonthlyPoint {
            month: r.month,
            income: r.income.unwrap_or(Decimal::ZERO),
            expenses: r.expenses.unwrap_or(Decimal::ZERO),
        })
        .collect();

    let spending_by_category: Vec<CategorySpend> = category_rows
        .into_iter()
        .map(|r| CategorySpend {
            category_id: r.category_id,
            category_name: r.category_name,
            total: r.total.unwrap_or(Decimal::ZERO),
        })
        .collect();

    Ok(DashboardSummary {
        net_worth,
        month_income,
        month_expenses,
        savings_rate,
        unreviewed_count: totals.unreviewed_count,
        monthly_trend,
        spending_by_category,
    })
}

// ── Income vs Expense ─────────────────────────────────────────────────────────

/// Monthly income/expenses with category breakdown for a date range.
pub async fn income_expense(
    pool: &PgPool,
    user_id: Uuid,
    from: Date,
    to: Date,
) -> Result<IncomeExpenseReport, CoreError> {
    let rows =
        rustvault_db::repos::report::income_expense_by_category(pool, user_id, from, to).await?;

    // Group rows by month
    let mut months_map: HashMap<Date, IncomeExpenseMonth> = HashMap::new();

    for row in rows {
        let entry = months_map.entry(row.month).or_insert(IncomeExpenseMonth {
            month: row.month,
            income: Decimal::ZERO,
            expenses: Decimal::ZERO,
            breakdown: Vec::new(),
        });

        let income = row.income.unwrap_or(Decimal::ZERO);
        let expenses = row.expenses.unwrap_or(Decimal::ZERO);
        entry.income += income;
        entry.expenses += expenses;

        if income > Decimal::ZERO || expenses > Decimal::ZERO {
            entry.breakdown.push(CategorySpend {
                category_id: row.category_id,
                category_name: row.category_name,
                total: if expenses > Decimal::ZERO {
                    expenses
                } else {
                    income
                },
            });
        }
    }

    let mut months: Vec<IncomeExpenseMonth> = months_map.into_values().collect();
    months.sort_by_key(|m| m.month);

    Ok(IncomeExpenseReport { months })
}

// ── Category trend ────────────────────────────────────────────────────────────

/// Spending trend for a single category over a date range.
pub async fn category_trend(
    pool: &PgPool,
    user_id: Uuid,
    category_id: Uuid,
    from: Date,
    to: Date,
) -> Result<CategoryTrendReport, CoreError> {
    let rows =
        rustvault_db::repos::report::category_trend(pool, user_id, category_id, from, to).await?;

    let periods: Vec<TrendPoint> = rows
        .iter()
        .map(|r| TrendPoint {
            period: r.period,
            total: r.total.unwrap_or(Decimal::ZERO),
        })
        .collect();

    let average = if periods.is_empty() {
        Decimal::ZERO
    } else {
        let sum: Decimal = periods.iter().map(|p| p.total).sum();
        sum / Decimal::from(periods.len() as i64)
    };

    Ok(CategoryTrendReport {
        category_id,
        periods,
        average,
    })
}

// ── Balance history ───────────────────────────────────────────────────────────

const LTTB_THRESHOLD: usize = 500;

/// Historical account balances for a date range.
///
/// Reconstructs past balances by starting from `balance_cache` and
/// working backwards using daily transaction sums.
pub async fn balance_history(
    pool: &PgPool,
    user_id: Uuid,
    account_ids: Vec<Uuid>,
    from: Date,
    to: Date,
) -> Result<BalanceHistoryReport, CoreError> {
    // Fetch account metadata and current balances
    let seeds =
        rustvault_db::repos::report::account_balance_seeds(pool, user_id, &account_ids).await?;

    if seeds.is_empty() {
        return Ok(BalanceHistoryReport {
            accounts: Vec::new(),
            snapshots: Vec::new(),
        });
    }

    let all_account_ids: Vec<Uuid> = seeds.iter().map(|s| s.account_id).collect();

    // Sum of transactions that occur after `to` (so we can derive balance at `to`)
    let future_rows =
        rustvault_db::repos::report::transactions_after_date(pool, user_id, &all_account_ids, to)
            .await?;

    let future_map: HashMap<Uuid, Decimal> = future_rows
        .into_iter()
        .map(|r| (r.account_id, r.future_net.unwrap_or(Decimal::ZERO)))
        .collect();

    // Balance at end of `to` = current balance − transactions after `to`
    let balance_at_to: HashMap<Uuid, Decimal> = seeds
        .iter()
        .map(|s| {
            let adjustment = future_map
                .get(&s.account_id)
                .copied()
                .unwrap_or(Decimal::ZERO);
            (s.account_id, s.balance_cache - adjustment)
        })
        .collect();

    // Daily changes within range
    let daily_rows = rustvault_db::repos::report::daily_account_changes(
        pool,
        user_id,
        &all_account_ids,
        from,
        to,
    )
    .await?;

    // Build a map: account_id → (date → net_change)
    let mut changes: HashMap<Uuid, HashMap<Date, Decimal>> = HashMap::new();
    for row in daily_rows {
        changes
            .entry(row.account_id)
            .or_default()
            .insert(row.date, row.daily_net.unwrap_or(Decimal::ZERO));
    }

    // Walk forward from `from` to `to`, computing per-account balance at each day
    // We walk backward from `to` to find the balance at `from`, then forward to build snapshots.
    let mut current = balance_at_to.clone();
    // Walk backward to find balances at `from`
    let mut walk_date = to;
    while walk_date >= from {
        for (acc_id, balance) in current.iter_mut() {
            if let Some(day_map) = changes.get(acc_id) {
                if let Some(net) = day_map.get(&walk_date) {
                    *balance -= net; // remove this day's change to go back
                }
            }
        }
        if walk_date == from {
            break;
        }
        walk_date = walk_date.previous_day().unwrap_or(walk_date);
    }

    // Now `current` = balances at beginning of `from`. Walk forward building snapshots.
    let mut snapshots_raw: Vec<(Date, Vec<(Uuid, Decimal)>)> = Vec::new();
    let mut walk_date = from;
    loop {
        // Apply daily changes for this date
        for (acc_id, balance) in current.iter_mut() {
            if let Some(day_map) = changes.get(acc_id) {
                if let Some(net) = day_map.get(&walk_date) {
                    *balance += net;
                }
            }
        }

        let balances: Vec<(Uuid, Decimal)> = all_account_ids
            .iter()
            .map(|id| (*id, *current.get(id).unwrap_or(&Decimal::ZERO)))
            .collect();
        snapshots_raw.push((walk_date, balances));

        if walk_date >= to {
            break;
        }
        walk_date = walk_date.next_day().unwrap_or(walk_date);
    }

    // Downsample if needed
    let downsampled = if snapshots_raw.len() > LTTB_THRESHOLD {
        // Use net worth as the signal for downsampling
        let series: Vec<(f64, f64)> = snapshots_raw
            .iter()
            .enumerate()
            .map(|(i, (_, balances))| {
                let net: Decimal = balances.iter().map(|(_, b)| b).sum();
                (i as f64, net.to_f64().unwrap_or(0.0))
            })
            .collect();

        let reduced = lttb(&series, LTTB_THRESHOLD);
        let selected_indices: std::collections::HashSet<usize> =
            reduced.iter().map(|(x, _)| *x as usize).collect();

        snapshots_raw
            .into_iter()
            .enumerate()
            .filter(|(i, _)| selected_indices.contains(i))
            .map(|(_, v)| v)
            .collect()
    } else {
        snapshots_raw
    };

    let snapshots: Vec<BalanceSnapshot> = downsampled
        .into_iter()
        .map(|(date, balances)| {
            let net_worth: Decimal = balances.iter().map(|(_, b)| b).sum();
            BalanceSnapshot {
                date,
                balances: balances
                    .into_iter()
                    .map(|(account_id, balance)| AccountBalance {
                        account_id,
                        balance,
                    })
                    .collect(),
                net_worth,
            }
        })
        .collect();

    let accounts: Vec<AccountMeta> = seeds
        .into_iter()
        .map(|s| AccountMeta {
            id: s.account_id,
            name: s.name,
            currency: s.currency,
        })
        .collect();

    Ok(BalanceHistoryReport {
        accounts,
        snapshots,
    })
}

// ── Cash flow ─────────────────────────────────────────────────────────────────

/// Cash flow analysis with historical periods and a 3-month forecast.
pub async fn cash_flow(
    pool: &PgPool,
    user_id: Uuid,
    from: Date,
    to: Date,
) -> Result<CashFlowReport, CoreError> {
    let rows = rustvault_db::repos::report::cash_flow(pool, user_id, from, to).await?;

    let periods: Vec<CashFlowPeriod> = rows
        .iter()
        .map(|r| {
            let income = r.income.unwrap_or(Decimal::ZERO);
            let expenses = r.expenses.unwrap_or(Decimal::ZERO);
            CashFlowPeriod {
                period: r.period,
                income,
                expenses,
                net: income - expenses,
                is_forecast: false,
            }
        })
        .collect();

    let (avg_income, avg_expenses) = if periods.is_empty() {
        (Decimal::ZERO, Decimal::ZERO)
    } else {
        let n = Decimal::from(periods.len() as i64);
        let sum_i: Decimal = periods.iter().map(|p| p.income).sum();
        let sum_e: Decimal = periods.iter().map(|p| p.expenses).sum();
        (sum_i / n, sum_e / n)
    };

    // Generate 3-month forecast starting from the month after `to`
    let forecast = (1..=3)
        .map(|offset| {
            let base = to;
            let forecast_month = add_months(base, offset);
            CashFlowPeriod {
                period: forecast_month,
                income: avg_income,
                expenses: avg_expenses,
                net: avg_income - avg_expenses,
                is_forecast: true,
            }
        })
        .collect();

    Ok(CashFlowReport {
        periods,
        avg_income,
        avg_expenses,
        forecast,
    })
}

/// Advance a date by `months` calendar months, clamping to the last valid day.
fn add_months(date: Date, months: i32) -> Date {
    let mut year = date.year();
    let mut month_num = date.month() as i32 + months;
    while month_num > 12 {
        month_num -= 12;
        year += 1;
    }
    while month_num < 1 {
        month_num += 12;
        year -= 1;
    }
    let month = Month::try_from(month_num as u8).unwrap_or(Month::January);
    // Use day 1 for forecast periods
    Date::from_calendar_date(year, month, 1).unwrap_or(date)
}
