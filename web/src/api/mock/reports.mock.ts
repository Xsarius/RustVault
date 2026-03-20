/**
 * Demo mode — Reports mock API.
 *
 * Returns pre-computed / derived report data from the demo transaction set.
 */

import { simulate } from "./latency";
import { demoStore } from "./store";
import type {
  DashboardSummary,
  IncomeExpenseReport,
  CategoryTrendReport,
  BalanceHistoryReport,
  CashFlowReport,
} from "~/api/types";

export async function fetchDashboardSummary(): Promise<DashboardSummary> {
  return simulate({ ...demoStore.dashboard });
}

export async function fetchIncomeExpenseReport(
  from: string,
  to: string,
): Promise<IncomeExpenseReport> {
  // Group transactions by month within [from, to]
  const txns = demoStore.transactions.filter(
    (t) => t.date >= from && t.date <= to && !t.is_deleted,
  );

  const monthMap = new Map<string, { income: number; expenses: number }>();
  for (const t of txns) {
    const month = t.date.slice(0, 7) + "-01";
    const entry = monthMap.get(month) ?? { income: 0, expenses: 0 };
    const amt = parseFloat(t.amount);
    if (t.transaction_type === "income") entry.income += amt;
    else if (t.transaction_type === "expense") entry.expenses += amt;
    monthMap.set(month, entry);
  }

  const months = [...monthMap.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([month, v]) => ({
      month,
      income: v.income.toFixed(2),
      expenses: v.expenses.toFixed(2),
      breakdown: [],
    }));

  return simulate({ months });
}

export async function fetchCategoryTrend(
  categoryId: string,
  from: string,
  to: string,
): Promise<CategoryTrendReport> {
  const txns = demoStore.transactions.filter(
    (t) => t.date >= from && t.date <= to && t.category_id === categoryId && !t.is_deleted,
  );
  const periodMap = new Map<string, number>();
  for (const t of txns) {
    const period = t.date.slice(0, 7) + "-01";
    periodMap.set(period, (periodMap.get(period) ?? 0) + parseFloat(t.amount));
  }
  const periods = [...periodMap.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([period, total]) => ({ period, total: total.toFixed(2) }));
  const average =
    periods.length > 0
      ? (periods.reduce((s, p) => s + parseFloat(p.total), 0) / periods.length).toFixed(2)
      : "0.00";
  return simulate({ category_id: categoryId, periods, average });
}

export async function fetchBalanceHistory(
  _from: string,
  _to: string,
  _accountIds?: string[],
): Promise<BalanceHistoryReport> {
  const accounts = demoStore.accounts.map((a) => ({
    id: a.id,
    name: a.name,
    currency: a.currency,
  }));
  // Return 6 monthly snapshots with synthetic values
  const snapshots = [
    { date: "2025-10-31", balances: [{ account_id: "acc-revolut-eur", balance: "3200.00" }], net_worth: "18200.00" },
    { date: "2025-11-30", balances: [{ account_id: "acc-revolut-eur", balance: "3450.00" }], net_worth: "20100.00" },
    { date: "2025-12-31", balances: [{ account_id: "acc-revolut-eur", balance: "2900.00" }], net_worth: "21500.00" },
    { date: "2026-01-31", balances: [{ account_id: "acc-revolut-eur", balance: "3200.00" }], net_worth: "22800.00" },
    { date: "2026-02-28", balances: [{ account_id: "acc-revolut-eur", balance: "3400.00" }], net_worth: "24100.00" },
    { date: "2026-03-31", balances: [{ account_id: "acc-revolut-eur", balance: "3600.00" }], net_worth: "25045.67" },
  ];
  return simulate({ accounts, snapshots });
}

export async function fetchCashFlowReport(
  from: string,
  to: string,
): Promise<CashFlowReport> {
  const txns = demoStore.transactions.filter(
    (t) => t.date >= from && t.date <= to && !t.is_deleted,
  );
  const periodMap = new Map<string, { income: number; expenses: number }>();
  for (const t of txns) {
    const period = t.date.slice(0, 7) + "-01";
    const entry = periodMap.get(period) ?? { income: 0, expenses: 0 };
    if (t.transaction_type === "income") entry.income += parseFloat(t.amount);
    else if (t.transaction_type === "expense") entry.expenses += parseFloat(t.amount);
    periodMap.set(period, entry);
  }
  const periods = [...periodMap.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([period, v]) => ({
      period,
      income: v.income.toFixed(2),
      expenses: v.expenses.toFixed(2),
      net: (v.income - v.expenses).toFixed(2),
      is_forecast: false,
    }));
  const avgIncome = periods.length > 0
    ? (periods.reduce((s, p) => s + parseFloat(p.income), 0) / periods.length).toFixed(2)
    : "0.00";
  const avgExpenses = periods.length > 0
    ? (periods.reduce((s, p) => s + parseFloat(p.expenses), 0) / periods.length).toFixed(2)
    : "0.00";
  return simulate({ periods, avg_income: avgIncome, avg_expenses: avgExpenses, forecast: [] });
}

export async function listExchangeRates() {
  return simulate([]);
}

export async function refreshExchangeRates() {
  return simulate(0);
}
