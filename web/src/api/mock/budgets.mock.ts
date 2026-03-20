/**
 * Demo mode — Budgets mock API.
 *
 * Handles budgets, budget lines, and the generate-next-period action.
 */

import { simulate } from "./latency";
import { demoStore, setDemoStore } from "./store";
import type {
  Budget,
  BudgetLine,
  BudgetSummary,
  BudgetLineSummary,
  NewBudget,
  UpdateBudget,
  NewBudgetLine,
  UpdateBudgetLine,
  BulkBudgetLines,
  CopyBudgetRequest,
} from "~/api/types";


// ── Budgets ───────────────────────────────────────────────────

export async function listBudgets(
  includeArchived = false,
): Promise<Budget[]> {
  const data = includeArchived
    ? [...demoStore.budgets]
    : demoStore.budgets.filter((b) => !b.is_archived);
  return simulate(data);
}

export async function getBudget(id: string): Promise<Budget> {
  const b = demoStore.budgets.find((b) => b.id === id);
  if (!b) throw new Error("Budget not found");
  return simulate(b);
}

export async function createBudget(body: NewBudget): Promise<Budget> {
  const budget: Budget = {
    id: `budget-${crypto.randomUUID()}`,
    user_id: "demo-user",
    name: body.name,
    period_start: body.period_start,
    period_end: body.period_end,
    currency: body.currency,
    is_recurring: body.is_recurring ?? false,
    recurrence_rule: body.recurrence_rule ?? null,
    is_archived: false,
    notes: body.notes ?? null,
    metadata: {},
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  setDemoStore("budgets", (prev) => [...prev, budget]);
  return simulate(budget);
}

export async function updateBudget(
  id: string,
  body: UpdateBudget,
): Promise<Budget> {
  setDemoStore("budgets", (prev) =>
    prev.map((b) =>
      b.id === id ? { ...b, ...body, updated_at: new Date().toISOString() } : b,
    ),
  );
  return simulate(demoStore.budgets.find((b) => b.id === id)!);
}

export async function deleteBudget(id: string): Promise<void> {
  setDemoStore("budgets", (prev) => prev.filter((b) => b.id !== id));
  return simulate(undefined);
}

export async function getBudgetSummary(id: string): Promise<BudgetSummary> {
  const lines = demoStore.budgetLines.filter((l) => l.budget_id === id);
  const lineSummaries: BudgetLineSummary[] = lines.map((l) => {
    const planned = parseFloat(l.planned_amount);
    const actual = parseFloat(l.actual_amount_cache);
    const remaining = planned - actual;
    const pct = planned > 0 ? ((actual / planned) * 100).toFixed(1) : "0.0";
    return {
      id: l.id,
      category_id: l.category_id,
      planned_amount: l.planned_amount,
      actual_amount: l.actual_amount_cache,
      remaining: remaining.toFixed(2),
      percent_used: pct,
    };
  });
  const totalPlannedExpenses = lineSummaries
    .reduce((s, l) => s + parseFloat(l.planned_amount), 0)
    .toFixed(2);
  const totalActualExpenses = lineSummaries
    .reduce((s, l) => s + parseFloat(l.actual_amount), 0)
    .toFixed(2);
  const overBudget = lineSummaries
    .filter((l) => parseFloat(l.actual_amount) > parseFloat(l.planned_amount))
    .map((l) => l.category_id ?? "");

  const summary: BudgetSummary = {
    budget_id: id,
    total_planned_income: "4500.00",
    total_actual_income: "4500.00",
    total_planned_expenses: totalPlannedExpenses,
    total_actual_expenses: totalActualExpenses,
    net_planned: (4500 - parseFloat(totalPlannedExpenses)).toFixed(2),
    net_actual: (4500 - parseFloat(totalActualExpenses)).toFixed(2),
    lines: lineSummaries,
    over_budget_categories: overBudget,
  };
  return simulate(summary);
}

export async function copyBudget(
  id: string,
  body: CopyBudgetRequest,
): Promise<Budget> {
  const src = demoStore.budgets.find((b) => b.id === id);
  if (!src) throw new Error("Budget not found");
  const newId = `budget-${crypto.randomUUID()}`;
  const newBudget: Budget = {
    ...src,
    id: newId,
    name: body.name,
    period_start: body.period_start,
    period_end: body.period_end,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  const newLines: BudgetLine[] = demoStore.budgetLines
    .filter((l) => l.budget_id === id)
    .map((l) => ({
      ...l,
      id: `bl-${crypto.randomUUID()}`,
      budget_id: newId,
      actual_amount_cache: "0.00",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }));
  setDemoStore("budgets", (prev) => [...prev, newBudget]);
  setDemoStore("budgetLines", (prev) => [...prev, ...newLines]);
  return simulate(newBudget);
}

export async function generateNextPeriod(id: string): Promise<Budget> {
  return copyBudget(id, {
    name: `Auto-generated from ${id}`,
    period_start: new Date().toISOString().slice(0, 10),
    period_end: new Date().toISOString().slice(0, 10),
  });
}

// ── Budget Lines ──────────────────────────────────────────────

export async function listBudgetLines(
  budgetId: string,
): Promise<BudgetLine[]> {
  const data = demoStore.budgetLines.filter((l) => l.budget_id === budgetId);
  return simulate(data);
}

export async function addBudgetLine(
  budgetId: string,
  body: NewBudgetLine,
): Promise<BudgetLine> {
  const line: BudgetLine = {
    id: `bl-${crypto.randomUUID()}`,
    budget_id: budgetId,
    category_id: body.category_id ?? null,
    planned_amount: body.planned_amount,
    actual_amount_cache: "0.00",
    notes: body.notes ?? null,
    sort_order: body.sort_order ?? demoStore.budgetLines.length,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  setDemoStore("budgetLines", (prev) => [...prev, line]);
  return simulate(line);
}

export async function bulkSetBudgetLines(
  budgetId: string,
  body: BulkBudgetLines,
): Promise<BudgetLine[]> {
  // Replace all lines for this budget
  setDemoStore("budgetLines", (prev) => prev.filter((l) => l.budget_id !== budgetId));
  const newLines: BudgetLine[] = body.lines.map((l, i) => ({
    id: `bl-${crypto.randomUUID()}`,
    budget_id: budgetId,
    category_id: l.category_id ?? null,
    planned_amount: l.planned_amount,
    actual_amount_cache: "0.00",
    notes: l.notes ?? null,
    sort_order: l.sort_order ?? i,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }));
  setDemoStore("budgetLines", (prev) => [...prev, ...newLines]);
  return simulate(newLines);
}

export async function updateBudgetLine(
  _budgetId: string,
  lineId: string,
  body: UpdateBudgetLine,
): Promise<BudgetLine> {
  setDemoStore("budgetLines", (prev) =>
    prev.map((l) =>
      l.id === lineId ? { ...l, ...body, updated_at: new Date().toISOString() } : l,
    ),
  );
  return simulate(demoStore.budgetLines.find((l) => l.id === lineId)!);
}

export async function deleteBudgetLine(
  _budgetId: string,
  lineId: string,
): Promise<void> {
  setDemoStore("budgetLines", (prev) => prev.filter((l) => l.id !== lineId));
  return simulate(undefined);
}
