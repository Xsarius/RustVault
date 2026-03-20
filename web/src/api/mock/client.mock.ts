/**
 * Demo mode — generic low-level request helpers.
 *
 * Acts as a path-based router so calls like:
 *   api.fetchList<Bank>("/api/banks")
 *   api.fetchOne<Transaction>("/api/transactions/abc")
 *   api.createOne<Tag>("/api/tags", body)
 * all resolve against the in-memory demo store without hitting the network.
 */

import { simulate } from "./latency";
import { demoStore, setDemoStore } from "./store";
import type { ApiResponse, PaginatedResponse, Transaction, NewTransaction, UpdateTransaction, AccountType, CategoryType } from "~/api/types";

export { setTokens, clearTokens, hasTokens, setBaseUrl, getBaseUrl, ApiError } from "./auth.mock";

// ── Demo user ─────────────────────────────────────────────────────────────────
const DEMO_USER = {
  id: "demo-user",
  username: "Demo User",
  email: "demo@rustvault.app",
  role: "member" as const,
  auth_provider: "local" as const,
  locale: "en-US",
  timezone: "Europe/Berlin",
  settings: {},
  created_at: "2025-01-01T00:00:00Z",
};

const DEMO_TOKENS = {
  access_token: "demo-access-token",
  token_type: "Bearer",
  expires_in: 86400,
  refresh_token: "demo-refresh-token",
};

function fakeMeta(len: number) {
  return { page_size: len, has_more: false } as const;
}

// Strip query-string to get the base path, then match against known prefixes.
function basePath(url: string): string {
  return url.split("?")[0];
}

function idFrom(url: string, prefix: string): string {
  return basePath(url).replace(prefix + "/", "");
}

// ── GET ───────────────────────────────────────────────────────────────────────

export function get<T>(path: string): Promise<T> {
  const base = basePath(path);

  if (base === "/api/auth/me") {
    return simulate({ data: DEMO_USER } as unknown as T);
  }
  if (base === "/api/transactions") {
    const params = new URLSearchParams(path.includes("?") ? path.split("?")[1] : "");
    const accountId = params.get("account_id") ?? undefined;
    const categoryId = params.get("category_id") ?? undefined;
    const type = params.get("transaction_type") ?? undefined;
    const dateFrom = params.get("date_from") ?? undefined;
    const dateTo = params.get("date_to") ?? undefined;
    const q = params.get("q") ?? undefined;
    const tagId = params.get("tag_id") ?? undefined;
    const limit = params.has("limit") ? parseInt(params.get("limit")!) : undefined;
    let data = [...demoStore.transactions].filter((t) => !t.is_deleted);
    if (accountId) data = data.filter((t) => t.account_id === accountId);
    if (categoryId) data = data.filter((t) => t.category_id === categoryId);
    if (type) data = data.filter((t) => t.transaction_type === type);
    if (dateFrom) data = data.filter((t) => t.date >= dateFrom);
    if (dateTo) data = data.filter((t) => t.date <= dateTo);
    if (tagId) data = data.filter((t) => t.tag_ids?.includes(tagId));
    if (q) { const n = q.toLowerCase(); data = data.filter((t) => t.description.toLowerCase().includes(n) || (t.payee ?? "").toLowerCase().includes(n)); }
    data = data.sort((a, b) => b.date.localeCompare(a.date));
    if (limit) data = data.slice(0, limit);
    return simulate({ data, meta: fakeMeta(data.length) } as unknown as T);
  }
  if (base === "/api/banks") {
    return simulate({ data: [...demoStore.banks], meta: fakeMeta(demoStore.banks.length) } as unknown as T);
  }
  if (base === "/api/accounts") {
    const params = new URLSearchParams(path.includes("?") ? path.split("?")[1] : "");
    const bankId = params.get("bank_id") ?? undefined;
    const data = bankId ? demoStore.accounts.filter((a) => a.bank_id === bankId) : [...demoStore.accounts];
    return simulate({ data, meta: fakeMeta(data.length) } as unknown as T);
  }
  if (base === "/api/categories") {
    return simulate({ data: [...demoStore.categories], meta: fakeMeta(demoStore.categories.length) } as unknown as T);
  }
  if (base === "/api/tags") {
    return simulate({ data: [...demoStore.tags], meta: fakeMeta(demoStore.tags.length) } as unknown as T);
  }
  if (base === "/api/rules") {
    return simulate({ data: [...demoStore.rules], meta: fakeMeta(demoStore.rules.length) } as unknown as T);
  }
  if (base === "/api/settings") {
    return simulate({ data: { ...demoStore.settings } } as unknown as T);
  }
  return simulate({} as T);
}

export function post<T>(path: string, _body?: unknown): Promise<T> {
  const base = basePath(path);
  if (base === "/api/auth/login" || base === "/api/auth/register" || base === "/api/auth/refresh") {
    return simulate({ data: DEMO_TOKENS } as unknown as T);
  }
  return simulate({} as T);
}

export function put<T>(_path: string, _body?: unknown): Promise<T> {
  return simulate({} as T);
}

export function patch<T>(_path: string, _body?: unknown): Promise<T> {
  return simulate({} as T);
}

export function del<T>(path: string): Promise<T> {
  const base = basePath(path);
  if (base.startsWith("/api/transactions/")) {
    const id = idFrom(path, "/api/transactions");
    setDemoStore("transactions", (prev) => prev.filter((t) => t.id !== id));
  } else if (base.startsWith("/api/banks/")) {
    const id = idFrom(path, "/api/banks");
    setDemoStore("banks", (prev) => prev.filter((b) => b.id !== id));
  } else if (base.startsWith("/api/accounts/")) {
    const id = idFrom(path, "/api/accounts");
    setDemoStore("accounts", (prev) => prev.filter((a) => a.id !== id));
  } else if (base.startsWith("/api/categories/")) {
    const id = idFrom(path, "/api/categories");
    setDemoStore("categories", (prev) => prev.filter((c) => c.id !== id));
  } else if (base.startsWith("/api/tags/")) {
    const id = idFrom(path, "/api/tags");
    setDemoStore("tags", (prev) => prev.filter((t) => t.id !== id));
  } else if (base.startsWith("/api/rules/")) {
    const id = idFrom(path, "/api/rules");
    setDemoStore("rules", (prev) => prev.filter((r) => r.id !== id));
  }
  return simulate(undefined as T);
}

// ── Typed convenience helpers ─────────────────────────────────────────────────

export async function fetchOne<T>(path: string): Promise<T> {
  const base = basePath(path);
  if (base === "/api/auth/me") return simulate(DEMO_USER as unknown as T);
  if (base.startsWith("/api/transactions/")) {
    const id = idFrom(path, "/api/transactions");
    return simulate(demoStore.transactions.find((t) => t.id === id) as T);
  }
  if (base.startsWith("/api/banks/")) {
    const id = idFrom(path, "/api/banks");
    return simulate(demoStore.banks.find((b) => b.id === id) as T);
  }
  if (base.startsWith("/api/accounts/")) {
    const id = idFrom(path, "/api/accounts");
    return simulate(demoStore.accounts.find((a) => a.id === id) as T);
  }
  if (base.startsWith("/api/categories/")) {
    const id = idFrom(path, "/api/categories");
    return simulate(demoStore.categories.find((c) => c.id === id) as T);
  }
  if (base.startsWith("/api/tags/")) {
    const id = idFrom(path, "/api/tags");
    return simulate(demoStore.tags.find((t) => t.id === id) as T);
  }
  if (base.startsWith("/api/rules/")) {
    const id = idFrom(path, "/api/rules");
    return simulate(demoStore.rules.find((r) => r.id === id) as T);
  }
  if (base === "/api/settings") {
    return simulate({ ...demoStore.settings } as T);
  }
  const wrapper = await get<ApiResponse<T>>(path);
  return (wrapper as ApiResponse<T>).data;
}

export async function fetchList<T>(path: string): Promise<PaginatedResponse<T>> {
  return get<PaginatedResponse<T>>(path);
}

export async function createOne<T>(path: string, body: unknown): Promise<T> {
  const base = basePath(path);
  const now = new Date().toISOString();
  if (base === "/api/banks") {
    const b = body as { name: string };
    const item = { id: `bank-${crypto.randomUUID()}`, user_id: "demo-user", name: b.name, is_archived: false, sort_order: demoStore.banks.length, metadata: {}, created_at: now, updated_at: now };
    setDemoStore("banks", (prev) => [...prev, item]);
    return simulate(item as T);
  }
  if (base === "/api/accounts") {
    const b = body as { bank_id: string; name: string; currency: string; type: string };
    const item = { id: `acc-${crypto.randomUUID()}`, user_id: "demo-user", bank_id: b.bank_id, name: b.name, currency: b.currency, type: b.type as AccountType, balance_cache: "0.00", supports_nonstandard_topup: false, is_archived: false, sort_order: demoStore.accounts.length, metadata: {}, created_at: now, updated_at: now };
    setDemoStore("accounts", (prev) => [...prev, item]);
    return simulate(item as T);
  }
  if (base === "/api/categories") {
    const b = body as { name: string; category_type: string; parent_id?: string; color?: string; icon?: string };
    const item = { id: `cat-${crypto.randomUUID()}`, user_id: "demo-user", name: b.name, parent_id: b.parent_id ?? null, icon: b.icon ?? null, color: b.color ?? null, category_type: b.category_type as CategoryType, sort_order: demoStore.categories.length, metadata: {}, created_at: now };
    setDemoStore("categories", (prev) => [...prev, item]);
    return simulate(item as T);
  }
  if (base === "/api/tags") {
    const b = body as { name: string; color?: string };
    const item = { id: `tag-${crypto.randomUUID()}`, user_id: "demo-user", name: b.name, color: b.color ?? null, created_at: now };
    setDemoStore("tags", (prev) => [...prev, item]);
    return simulate(item as T);
  }
  if (base === "/api/transactions") {
    const b = body as NewTransaction;
    const item: Transaction = { id: `txn-${crypto.randomUUID()}`, user_id: "demo-user", account_id: b.account_id, category_id: b.category_id ?? null, import_id: null, transaction_type: b.transaction_type, amount: b.amount, currency: "EUR", date: b.date, description: b.description, original_desc: null, payee: b.payee ?? null, reference: null, notes: b.notes ?? null, is_reviewed: true, is_deleted: false, is_duplicate: false, metadata: {}, tag_ids: b.tag_ids ?? [], created_at: now, updated_at: now };
    setDemoStore("transactions", (prev) => [item, ...prev]);
    return simulate(item as T);
  }
  if (base === "/api/rules") {
    const b = body as { name: string; conditions: unknown; actions: unknown; priority?: number };
    const item = { id: `rule-${crypto.randomUUID()}`, user_id: "demo-user", name: b.name, priority: b.priority ?? 50, is_enabled: true, conditions: b.conditions, actions: b.actions, metadata: {}, created_at: now, updated_at: now };
    setDemoStore("rules", (prev) => [...prev, item]);
    return simulate(item as T);
  }
  return simulate(body as T);
}

export async function updateOne<T>(path: string, body: unknown): Promise<T> {
  const base = basePath(path);
  const now = new Date().toISOString();
  const patch = body as Record<string, unknown>;
  if (base.startsWith("/api/transactions/")) {
    const id = idFrom(path, "/api/transactions");
    const b = body as UpdateTransaction;
    setDemoStore("transactions", (prev) => prev.map((t) => t.id === id ? { ...t, ...b, updated_at: now } : t));
    return simulate(demoStore.transactions.find((t) => t.id === id) as T);
  }
  if (base.startsWith("/api/banks/")) {
    const id = idFrom(path, "/api/banks");
    setDemoStore("banks", (prev) => prev.map((b) => b.id === id ? { ...b, ...patch, updated_at: now } : b));
    return simulate(demoStore.banks.find((b) => b.id === id) as T);
  }
  if (base.startsWith("/api/accounts/")) {
    const id = idFrom(path, "/api/accounts");
    setDemoStore("accounts", (prev) => prev.map((a) => a.id === id ? { ...a, ...patch, updated_at: now } : a));
    return simulate(demoStore.accounts.find((a) => a.id === id) as T);
  }
  if (base.startsWith("/api/categories/")) {
    const id = idFrom(path, "/api/categories");
    setDemoStore("categories", (prev) => prev.map((c) => c.id === id ? { ...c, ...patch } : c));
    return simulate(demoStore.categories.find((c) => c.id === id) as T);
  }
  if (base.startsWith("/api/tags/")) {
    const id = idFrom(path, "/api/tags");
    setDemoStore("tags", (prev) => prev.map((t) => t.id === id ? { ...t, ...patch } : t));
    return simulate(demoStore.tags.find((t) => t.id === id) as T);
  }
  if (base.startsWith("/api/rules/")) {
    const id = idFrom(path, "/api/rules");
    setDemoStore("rules", (prev) => prev.map((r) => r.id === id ? { ...r, ...patch, updated_at: now } : r));
    return simulate(demoStore.rules.find((r) => r.id === id) as T);
  }
  return simulate(body as T);
}

export async function postFormData<T>(_path: string, _formData: FormData): Promise<T> {
  return simulate({} as T);
}
