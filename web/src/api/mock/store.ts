/**
 * Demo mode — in-memory reactive store.
 *
 * Holds mutable copies of all seed data. All mock API modules
 * read from and write to this store so that mutations (create,
 * update, delete) are reflected immediately in the UI while
 * the user explores the demo — without touching any real server.
 *
 * State is reset on page refresh (intentional: demo mode is
 * session-scoped only).
 */

import { createStore } from "solid-js/store";

import { DEMO_BANKS } from "./data/banks";
import { DEMO_ACCOUNTS } from "./data/accounts";
import { DEMO_CATEGORIES } from "./data/categories";
import { DEMO_TAGS, DEMO_SETTINGS, DEMO_DASHBOARD } from "./data/misc";
import { DEMO_TRANSACTIONS } from "./data/transactions";
import { DEMO_BUDGETS, DEMO_BUDGET_LINES, DEMO_RULES } from "./data/budgets";

import type {
  Bank,
  Account,
  Category,
  Tag,
  Transaction,
  Budget,
  BudgetLine,
  AutoRule,
  UserSettings,
  DashboardSummary,
} from "~/api/types";

export interface DemoStore {
  banks: Bank[];
  accounts: Account[];
  categories: Category[];
  tags: Tag[];
  transactions: Transaction[];
  budgets: Budget[];
  budgetLines: BudgetLine[];
  rules: AutoRule[];
  settings: UserSettings;
  dashboard: DashboardSummary;
}

export const [demoStore, setDemoStore] = createStore<DemoStore>({
  banks: structuredClone(DEMO_BANKS),
  accounts: structuredClone(DEMO_ACCOUNTS),
  categories: structuredClone(DEMO_CATEGORIES),
  tags: structuredClone(DEMO_TAGS),
  transactions: structuredClone(DEMO_TRANSACTIONS),
  budgets: structuredClone(DEMO_BUDGETS),
  budgetLines: structuredClone(DEMO_BUDGET_LINES),
  rules: structuredClone(DEMO_RULES),
  settings: structuredClone(DEMO_SETTINGS),
  dashboard: structuredClone(DEMO_DASHBOARD),
});

/** Restore all store slices to seed data. Useful in tests to ensure isolation. */
export function resetDemoStore(): void {
  setDemoStore({
    banks: structuredClone(DEMO_BANKS),
    accounts: structuredClone(DEMO_ACCOUNTS),
    categories: structuredClone(DEMO_CATEGORIES),
    tags: structuredClone(DEMO_TAGS),
    transactions: structuredClone(DEMO_TRANSACTIONS),
    budgets: structuredClone(DEMO_BUDGETS),
    budgetLines: structuredClone(DEMO_BUDGET_LINES),
    rules: structuredClone(DEMO_RULES),
    settings: structuredClone(DEMO_SETTINGS),
    dashboard: structuredClone(DEMO_DASHBOARD),
  });
}
