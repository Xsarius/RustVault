/**
 * Demo mode seed data — Tags, Settings, Reports.
 */

import type { Tag, UserSettings, DashboardSummary } from "~/api/types";

export const DEMO_TAGS: Tag[] = [
  { id: "tag-vacation",  user_id: "demo-user", name: "vacation",  color: "#2563eb", created_at: "2026-01-01T00:00:00Z" },
  { id: "tag-business",  user_id: "demo-user", name: "business",  color: "#7c3aed", created_at: "2026-01-01T00:00:00Z" },
  { id: "tag-recurring", user_id: "demo-user", name: "recurring", color: "#16a34a", created_at: "2026-01-01T00:00:00Z" },
  { id: "tag-online",    user_id: "demo-user", name: "online",    color: "#0891b2", created_at: "2026-01-01T00:00:00Z" },
  { id: "tag-cash",      user_id: "demo-user", name: "cash",      color: "#d97706", created_at: "2026-01-01T00:00:00Z" },
  { id: "tag-refund",    user_id: "demo-user", name: "refund",    color: "#86efac", created_at: "2026-01-01T00:00:00Z" },
  { id: "tag-shared",    user_id: "demo-user", name: "shared",    color: "#db2777", created_at: "2026-01-01T00:00:00Z" },
  { id: "tag-tax",       user_id: "demo-user", name: "tax",       color: "#64748b", created_at: "2026-01-01T00:00:00Z" },
];

export const DEMO_SETTINGS: UserSettings = {
  default_currency: "EUR",
  locale: "en-US",
  date_format: "YYYY-MM-DD",
  ai_enabled: false,
};

export const DEMO_DASHBOARD: DashboardSummary = {
  net_worth: "25045.67",
  month_income: "4500.00",
  month_expenses: "1987.45",
  savings_rate: 55.8,
  unreviewed_count: 14,
  monthly_trend: [
    { month: "2025-10-01", income: "4200.00", expenses: "2100.00" },
    { month: "2025-11-01", income: "4200.00", expenses: "1950.00" },
    { month: "2025-12-01", income: "5400.00", expenses: "3100.00" },
    { month: "2026-01-01", income: "4500.00", expenses: "1870.00" },
    { month: "2026-02-01", income: "4500.00", expenses: "2010.00" },
    { month: "2026-03-01", income: "4500.00", expenses: "1987.45" },
  ],
  spending_by_category: [
    { category_id: "cat-rent",      category_name: "Rent",         total: "1200.00" },
    { category_id: "cat-groceries", category_name: "Groceries",    total: "285.20"  },
    { category_id: "cat-dining",    category_name: "Dining Out",   total: "180.50"  },
    { category_id: "cat-streaming", category_name: "Streaming",    total: "52.00"   },
    { category_id: "cat-gym",       category_name: "Gym & Sport",  total: "39.99"   },
    { category_id: null,            category_name: "Other",        total: "229.76"  },
  ],
};
