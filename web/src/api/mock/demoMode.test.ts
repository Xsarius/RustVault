/**
 * Demo mode — integration tests.
 *
 * These tests exercise the mock API layer end-to-end: from the generic
 * client helpers (get / fetchList / createOne / updateOne / del) through
 * the domain-specific mocks (reports, budgets, transactions, …) to make
 * sure that all returned values are correctly typed and carry real data
 * from the seed set.
 *
 * The `latency` module is mocked so tests run at full speed.
 * The store is reset before every test to guarantee isolation.
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { resetDemoStore, demoStore } from "./store";

// Speed up tests: resolve simulate() immediately.
vi.mock("./latency", () => ({
  simulate: <T>(x: T) => Promise.resolve(x),
}));

// Import mock modules AFTER the latency mock is registered.
const {
  get,
  post,
  del,
  fetchOne,
  fetchList,
  createOne,
  updateOne,
} = await import("./client.mock");

const { fetchDashboardSummary, fetchIncomeExpenseReport } = await import("./reports.mock");

const {
  listBudgets,
  getBudget,
  getBudgetSummary,
  listBudgetLines,
  createBudget,
} = await import("./budgets.mock");

const { listBanks } = await import("./banks.mock");
const { listTags } = await import("./tags.mock");
const { listTransactions } = await import("./transactions.mock");

// ── Setup ─────────────────────────────────────────────────────────────────────

beforeEach(() => {
  resetDemoStore();
});

// ── Generic client: GET ───────────────────────────────────────────────────────

describe("get()", () => {
  it("returns the demo user for /api/auth/me", async () => {
    // get() returns ApiResponse<User> for the auth endpoint
    const res = await get<{ data: { email: string } }>("/api/auth/me");
    expect(res.data.email).toBe("demo@rustvault.app");
  });

  it("returns all banks wrapped in a PaginatedResponse", async () => {
    const res = await get<{ data: unknown[]; meta: unknown }>("/api/banks");
    expect(Array.isArray(res.data)).toBe(true);
    expect((res.data as unknown[]).length).toBe(3);
  });

  it("returns accounts filtered by bank_id", async () => {
    const res = await get<{ data: { bank_id: string }[] }>("/api/accounts?bank_id=bank-revolut");
    expect(res.data.every((a) => a.bank_id === "bank-revolut")).toBe(true);
  });

  it("returns all transactions sorted newest-first", async () => {
    const res = await get<{ data: { date: string }[] }>("/api/transactions");
    const dates = res.data.map((t) => t.date);
    const sorted = [...dates].sort((a, b) => b.localeCompare(a));
    expect(dates).toEqual(sorted);
  });

  it("filters transactions by account_id", async () => {
    const res = await get<{ data: { account_id: string }[] }>(
      "/api/transactions?account_id=acc-revolut-eur",
    );
    expect(res.data.length).toBeGreaterThan(0);
    expect(res.data.every((t) => t.account_id === "acc-revolut-eur")).toBe(true);
  });

  it("filters transactions by date range", async () => {
    const res = await get<{ data: { date: string }[] }>(
      "/api/transactions?date_from=2025-10-01&date_to=2025-10-31",
    );
    expect(res.data.length).toBeGreaterThan(0);
    expect(res.data.every((t) => t.date >= "2025-10-01" && t.date <= "2025-10-31")).toBe(true);
  });

  it("filters transactions by tag_id", async () => {
    const res = await get<{ data: { tag_ids: string[] }[] }>(
      "/api/transactions?tag_id=tag-recurring",
    );
    expect(res.data.length).toBeGreaterThan(0);
    expect(res.data.every((t) => t.tag_ids.includes("tag-recurring"))).toBe(true);
  });

  it("respects the limit parameter", async () => {
    const res = await get<{ data: unknown[] }>("/api/transactions?limit=5");
    expect(res.data.length).toBe(5);
  });

  it("returns categories", async () => {
    const res = await get<{ data: unknown[] }>("/api/categories");
    expect((res.data as unknown[]).length).toBeGreaterThan(0);
  });

  it("returns tags", async () => {
    const res = await get<{ data: { id: string }[] }>("/api/tags");
    expect(res.data.some((t) => t.id === "tag-vacation")).toBe(true);
  });

  it("returns settings", async () => {
    const res = await get<{ data: { default_currency: string } }>("/api/settings");
    expect(res.data.default_currency).toBe("EUR");
  });
});

// ── Generic client: POST ──────────────────────────────────────────────────────

describe("post()", () => {
  it("returns demo tokens on /api/auth/login", async () => {
    const res = await post<{ data: { access_token: string } }>("/api/auth/login", {
      email: "demo@rustvault.app",
      password: "demo",
    });
    expect(res.data.access_token).toBe("demo-access-token");
  });

  it("returns demo tokens on /api/auth/refresh", async () => {
    const res = await post<{ data: { access_token: string } }>("/api/auth/refresh");
    expect(res.data.access_token).toBe("demo-access-token");
  });
});

// ── Generic client: fetchOne ──────────────────────────────────────────────────

describe("fetchOne()", () => {
  it("returns the demo user for /api/auth/me", async () => {
    const user = await fetchOne<{ email: string }>("/api/auth/me");
    expect(user.email).toBe("demo@rustvault.app");
  });

  it("returns a bank by ID", async () => {
    const bank = await fetchOne<{ id: string; name: string }>("/api/banks/bank-revolut");
    expect(bank.id).toBe("bank-revolut");
    expect(bank.name).toBe("Revolut");
  });

  it("returns a transaction by ID", async () => {
    const txn = await fetchOne<{ id: string; description: string }>("/api/transactions/txn-001");
    expect(txn.id).toBe("txn-001");
    expect(txn.description).toContain("Salary");
  });

  it("returns a tag by ID", async () => {
    const tag = await fetchOne<{ id: string; name: string }>("/api/tags/tag-vacation");
    expect(tag.id).toBe("tag-vacation");
  });
});

// ── Generic client: fetchList ─────────────────────────────────────────────────

describe("fetchList()", () => {
  it("returns a PaginatedResponse with banks", async () => {
    const res = await fetchList<{ id: string }>("/api/banks");
    expect(Array.isArray(res.data)).toBe(true);
    expect(res.data.length).toBe(3);
    expect(res.meta.has_more).toBe(false);
  });

  it("returns a PaginatedResponse with transactions", async () => {
    const res = await fetchList<{ id: string }>("/api/transactions");
    expect(res.data.length).toBeGreaterThan(0);
  });
});

// ── Generic client: createOne / updateOne / del ───────────────────────────────

describe("createOne()", () => {
  it("adds a new bank to the store and returns it", async () => {
    const initial = demoStore.banks.length;
    const bank = await createOne<{ id: string; name: string }>("/api/banks", {
      name: "Test Bank",
    });
    expect(bank.name).toBe("Test Bank");
    expect(bank.id).toMatch(/^bank-/);
    expect(demoStore.banks.length).toBe(initial + 1);
  });

  it("adds a new tag and returns it", async () => {
    const tag = await createOne<{ id: string; name: string }>("/api/tags", {
      name: "my-new-tag",
      color: "#ff0000",
    });
    expect(tag.name).toBe("my-new-tag");
    expect(demoStore.tags.some((t) => t.name === "my-new-tag")).toBe(true);
  });

  it("adds a new transaction and returns it", async () => {
    const initial = demoStore.transactions.length;
    const txn = await createOne<{ id: string; description: string }>("/api/transactions", {
      account_id: "acc-revolut-eur",
      transaction_type: "expense",
      amount: "25.00",
      date: "2026-03-15",
      description: "Test purchase",
    });
    expect(txn.description).toBe("Test purchase");
    expect(demoStore.transactions.length).toBe(initial + 1);
  });
});

describe("updateOne()", () => {
  it("patches a transaction and persists the change", async () => {
    const updated = await updateOne<{ id: string; description: string }>(
      "/api/transactions/txn-001",
      { description: "Modified Salary" },
    );
    expect(updated.description).toBe("Modified Salary");
    expect(demoStore.transactions.find((t) => t.id === "txn-001")?.description).toBe("Modified Salary");
  });

  it("patches a bank name", async () => {
    await updateOne("/api/banks/bank-revolut", { name: "Revolut Personal" });
    expect(demoStore.banks.find((b) => b.id === "bank-revolut")?.name).toBe("Revolut Personal");
  });
});

describe("del()", () => {
  it("removes a tag from the store", async () => {
    expect(demoStore.tags.some((t) => t.id === "tag-vacation")).toBe(true);
    await del("/api/tags/tag-vacation");
    expect(demoStore.tags.some((t) => t.id === "tag-vacation")).toBe(false);
  });

  it("removes a transaction from the store", async () => {
    await del("/api/transactions/txn-005");
    expect(demoStore.transactions.some((t) => t.id === "txn-005")).toBe(false);
  });

  it("removes a bank from the store", async () => {
    await del("/api/banks/bank-zen");
    expect(demoStore.banks.some((b) => b.id === "bank-zen")).toBe(false);
  });
});

// ── Domain mocks: reports ─────────────────────────────────────────────────────

describe("fetchDashboardSummary()", () => {
  it("returns a DashboardSummary with a numeric savings_rate", async () => {
    const summary = await fetchDashboardSummary();
    expect(typeof summary.savings_rate).toBe("number");
    expect(summary.savings_rate).toBeGreaterThan(0);
  });

  it("returns net_worth as a string", async () => {
    const summary = await fetchDashboardSummary();
    expect(typeof summary.net_worth).toBe("string");
    // e.g. "25045.67" — should parse as a positive number
    expect(parseFloat(summary.net_worth)).toBeGreaterThan(0);
  });

  it("is not wrapped in an ApiResponse", async () => {
    const summary = await fetchDashboardSummary();
    // if it were wrapped, summary.savings_rate would be undefined
    expect(summary.savings_rate).not.toBeUndefined();
    // the object should NOT have a 'data' key at the top level
    expect((summary as Record<string, unknown>).data).toBeUndefined();
  });
});

describe("fetchIncomeExpenseReport()", () => {
  it("returns months array with numeric-string income/expenses", async () => {
    const report = await fetchIncomeExpenseReport("2025-10-01", "2025-12-31");
    expect(Array.isArray(report.months)).toBe(true);
    expect(report.months.length).toBeGreaterThan(0);
    for (const m of report.months) {
      expect(typeof m.income).toBe("string");
      expect(typeof m.expenses).toBe("string");
      expect(parseFloat(m.income)).toBeGreaterThanOrEqual(0);
    }
  });

  it("excludes months outside the date range", async () => {
    const report = await fetchIncomeExpenseReport("2025-10-01", "2025-10-31");
    for (const m of report.months) {
      expect(m.month.startsWith("2025-10")).toBe(true);
    }
  });

  it("is not wrapped in an ApiResponse", async () => {
    const report = await fetchIncomeExpenseReport("2025-10-01", "2025-12-31");
    expect((report as Record<string, unknown>).data).toBeUndefined();
  });
});

// ── Domain mocks: budgets ─────────────────────────────────────────────────────

describe("listBudgets()", () => {
  it("returns a plain Budget[] (not wrapped in PaginatedResponse)", async () => {
    const budgets = await listBudgets();
    expect(Array.isArray(budgets)).toBe(true);
    // Make sure it is NOT a PaginatedResponse wrapper
    expect((budgets as unknown as Record<string, unknown>).data).toBeUndefined();
  });

  it("returns the seed budgets", async () => {
    const budgets = await listBudgets();
    expect(budgets.some((b) => b.id === "budget-feb-2026")).toBe(true);
    expect(budgets.some((b) => b.id === "budget-mar-2026")).toBe(true);
  });
});

describe("getBudget()", () => {
  it("returns a single Budget by id", async () => {
    const budget = await getBudget("budget-mar-2026");
    expect(budget.id).toBe("budget-mar-2026");
    expect(budget.name).toBe("March 2026");
  });

  it("throws when the budget does not exist", async () => {
    await expect(getBudget("nonexistent")).rejects.toThrow();
  });
});

describe("getBudgetSummary()", () => {
  it("returns a BudgetSummary with a lines array", async () => {
    const summary = await getBudgetSummary("budget-mar-2026");
    expect(Array.isArray(summary.lines)).toBe(true);
    expect((summary as Record<string, unknown>).data).toBeUndefined();
  });
});

describe("listBudgetLines()", () => {
  it("returns a plain BudgetLine[] (not wrapped)", async () => {
    const lines = await listBudgetLines("budget-feb-2026");
    expect(Array.isArray(lines)).toBe(true);
    expect((lines as unknown as Record<string, unknown>).data).toBeUndefined();
  });
});

describe("createBudget()", () => {
  it("adds a budget to the store and returns it", async () => {
    const initial = demoStore.budgets.length;
    const b = await createBudget({
      name: "April 2026",
      period_start: "2026-04-01",
      period_end: "2026-04-30",
      currency: "EUR",
      is_recurring: false,
    });
    expect(b.name).toBe("April 2026");
    expect(demoStore.budgets.length).toBe(initial + 1);
  });
});

// ── Domain mocks: banks / tags / transactions ─────────────────────────────────

describe("listBanks()", () => {
  it("returns PaginatedResponse with the 3 seed banks", async () => {
    const res = await listBanks();
    expect(res.data.length).toBe(3);
    const names = res.data.map((b) => b.name);
    expect(names).toContain("Revolut");
    expect(names).toContain("Lunar");
  });
});

describe("listTags()", () => {
  it("returns at least 8 seed tags", async () => {
    const res = await listTags();
    expect(res.data.length).toBeGreaterThanOrEqual(8);
  });
});

describe("listTransactions()", () => {
  it("returns all non-deleted transactions", async () => {
    const res = await listTransactions();
    expect(res.data.length).toBeGreaterThan(0);
    expect(res.data.every((t) => !t.is_deleted)).toBe(true);
  });
});

// ── Store isolation ───────────────────────────────────────────────────────────

describe("store isolation", () => {
  it("reset restores deleted tags", async () => {
    await del("/api/tags/tag-vacation");
    expect(demoStore.tags.some((t) => t.id === "tag-vacation")).toBe(false);
    resetDemoStore();
    expect(demoStore.tags.some((t) => t.id === "tag-vacation")).toBe(true);
  });

  it("reset removes items added in a previous test", async () => {
    await createOne("/api/banks", { name: "Ephemeral Bank" });
    expect(demoStore.banks.some((b) => b.name === "Ephemeral Bank")).toBe(true);
    resetDemoStore();
    expect(demoStore.banks.some((b) => b.name === "Ephemeral Bank")).toBe(false);
  });
});
