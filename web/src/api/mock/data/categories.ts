/**
 * Demo mode seed data — Categories.
 */

import type { Category } from "~/api/types";

export const DEMO_CATEGORIES: Category[] = [
  // ── Income ────────────────────────────────────────────────
  { id: "cat-salary",     user_id: "demo-user", name: "Salary",          parent_id: null, icon: "briefcase",    color: "#16a34a", category_type: "income",  sort_order: 0,  metadata: {}, created_at: "2026-01-01T00:00:00Z" },
  { id: "cat-freelance",  user_id: "demo-user", name: "Freelance",       parent_id: null, icon: "laptop",       color: "#22c55e", category_type: "income",  sort_order: 1,  metadata: {}, created_at: "2026-01-01T00:00:00Z" },
  { id: "cat-interest",   user_id: "demo-user", name: "Interest",        parent_id: null, icon: "trending-up",  color: "#86efac", category_type: "income",  sort_order: 2,  metadata: {}, created_at: "2026-01-01T00:00:00Z" },

  // ── Expense — top-level ────────────────────────────────────
  { id: "cat-housing",    user_id: "demo-user", name: "Housing",         parent_id: null, icon: "home",         color: "#2563eb", category_type: "expense", sort_order: 10, metadata: {}, created_at: "2026-01-01T00:00:00Z" },
  { id: "cat-food",       user_id: "demo-user", name: "Food & Drink",    parent_id: null, icon: "coffee",       color: "#d97706", category_type: "expense", sort_order: 20, metadata: {}, created_at: "2026-01-01T00:00:00Z" },
  { id: "cat-transport",  user_id: "demo-user", name: "Transport",       parent_id: null, icon: "car",          color: "#7c3aed", category_type: "expense", sort_order: 30, metadata: {}, created_at: "2026-01-01T00:00:00Z" },
  { id: "cat-health",     user_id: "demo-user", name: "Health",          parent_id: null, icon: "heart",        color: "#dc2626", category_type: "expense", sort_order: 40, metadata: {}, created_at: "2026-01-01T00:00:00Z" },
  { id: "cat-entertain",  user_id: "demo-user", name: "Entertainment",   parent_id: null, icon: "play",         color: "#0891b2", category_type: "expense", sort_order: 50, metadata: {}, created_at: "2026-01-01T00:00:00Z" },
  { id: "cat-shopping",   user_id: "demo-user", name: "Shopping",        parent_id: null, icon: "shopping-bag", color: "#db2777", category_type: "expense", sort_order: 60, metadata: {}, created_at: "2026-01-01T00:00:00Z" },
  { id: "cat-utilities",  user_id: "demo-user", name: "Utilities",       parent_id: null, icon: "zap",          color: "#65a30d", category_type: "expense", sort_order: 70, metadata: {}, created_at: "2026-01-01T00:00:00Z" },

  // ── Expense — sub-categories ───────────────────────────────
  { id: "cat-rent",       user_id: "demo-user", name: "Rent",            parent_id: "cat-housing",   icon: "key",          color: "#3b82f6", category_type: "expense", sort_order: 11, metadata: {}, created_at: "2026-01-01T00:00:00Z" },
  { id: "cat-groceries",  user_id: "demo-user", name: "Groceries",       parent_id: "cat-food",      icon: "shopping-cart", color: "#f59e0b", category_type: "expense", sort_order: 21, metadata: {}, created_at: "2026-01-01T00:00:00Z" },
  { id: "cat-dining",     user_id: "demo-user", name: "Dining Out",      parent_id: "cat-food",      icon: "utensils",     color: "#f97316", category_type: "expense", sort_order: 22, metadata: {}, created_at: "2026-01-01T00:00:00Z" },
  { id: "cat-streaming",  user_id: "demo-user", name: "Streaming",       parent_id: "cat-entertain", icon: "tv",           color: "#06b6d4", category_type: "expense", sort_order: 51, metadata: {}, created_at: "2026-01-01T00:00:00Z" },
  { id: "cat-gym",        user_id: "demo-user", name: "Gym & Sport",     parent_id: "cat-health",    icon: "activity",     color: "#ef4444", category_type: "expense", sort_order: 41, metadata: {}, created_at: "2026-01-01T00:00:00Z" },
  { id: "cat-travel",     user_id: "demo-user", name: "Travel",          parent_id: null,            icon: "plane",        color: "#0284c7", category_type: "expense", sort_order: 80, metadata: {}, created_at: "2026-01-01T00:00:00Z" },
];
