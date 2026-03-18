/**
 * API client — typed fetch wrapper with auto-refresh on 401.
 *
 * All backend calls go through this module. It handles:
 * - Base URL prefixing
 * - JSON content type
 * - Bearer token injection
 * - Automatic token refresh on 401 (then retry original request)
 * - Structured error extraction
 */

import type { ApiErrorBody, ApiResponse, PaginatedResponse } from "./types";

// ── Token storage (in-memory only — never localStorage) ──────

let accessToken: string | null = null;
let refreshToken: string | null = null;
let refreshPromise: Promise<boolean> | null = null;

/** Store tokens after login/refresh. */
export function setTokens(access: string, refresh: string) {
  accessToken = access;
  refreshToken = refresh;
}

/** Clear tokens on logout. */
export function clearTokens() {
  accessToken = null;
  refreshToken = null;
}

/** Check if user has tokens (may be expired). */
export function hasTokens(): boolean {
  return accessToken !== null;
}

// ── API Error class ──────────────────────────────────────────

export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
    public details?: ApiErrorBody["error"]["details"],
  ) {
    super(message);
    this.name = "ApiError";
  }
}

// ── Core fetch wrapper ──────────────────────────────────────

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
  retry = true,
): Promise<T> {
  const headers: Record<string, string> = {};

  if (body !== undefined) {
    headers["Content-Type"] = "application/json";
  }

  if (accessToken) {
    headers["Authorization"] = `Bearer ${accessToken}`;
  }

  const res = await fetch(path, {
    method,
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });

  // Handle 401 — try refresh once
  if (res.status === 401 && retry && refreshToken) {
    const refreshed = await doRefresh();
    if (refreshed) {
      return request<T>(method, path, body, false);
    }
    // Refresh failed — clear tokens and throw
    clearTokens();
    throw new ApiError(401, "AUTH_EXPIRED", "Session expired. Please log in again.");
  }

  // Handle error responses
  if (!res.ok) {
    let errorBody: ApiErrorBody | undefined;
    try {
      errorBody = await res.json();
    } catch {
      // Response is not JSON
    }

    throw new ApiError(
      res.status,
      errorBody?.error?.code ?? "UNKNOWN",
      errorBody?.error?.message ?? `Request failed with status ${res.status}`,
      errorBody?.error?.details,
    );
  }

  // 204 No Content
  if (res.status === 204) {
    return undefined as T;
  }

  return res.json();
}

/** Attempt token refresh. Returns true on success. */
async function doRefresh(): Promise<boolean> {
  // Deduplicate concurrent refresh attempts
  if (refreshPromise) return refreshPromise;

  refreshPromise = (async () => {
    try {
      const res = await fetch("/api/auth/refresh", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ refresh_token: refreshToken }),
      });

      if (!res.ok) return false;

      const data = await res.json();
      accessToken = data.data.access_token;
      refreshToken = data.data.refresh_token;
      return true;
    } catch {
      return false;
    } finally {
      refreshPromise = null;
    }
  })();

  return refreshPromise;
}

// ── Public API methods ───────────────────────────────────────

export function get<T>(path: string): Promise<T> {
  return request<T>("GET", path);
}

export function post<T>(path: string, body?: unknown): Promise<T> {
  return request<T>("POST", path, body);
}

export function put<T>(path: string, body?: unknown): Promise<T> {
  return request<T>("PUT", path, body);
}

export function patch<T>(path: string, body?: unknown): Promise<T> {
  return request<T>("PATCH", path, body);
}

export function del<T>(path: string): Promise<T> {
  return request<T>("DELETE", path);
}

// ── Typed convenience helpers ────────────────────────────────

/** Fetch a single resource. */
export async function fetchOne<T>(path: string): Promise<T> {
  const res = await get<ApiResponse<T>>(path);
  return res.data;
}

/** Fetch a paginated collection. */
export async function fetchList<T>(path: string): Promise<PaginatedResponse<T>> {
  return get<PaginatedResponse<T>>(path);
}

/** Create a resource and return it. */
export async function createOne<T>(path: string, body: unknown): Promise<T> {
  const res = await post<ApiResponse<T>>(path, body);
  return res.data;
}

/** Update a resource and return it. */
export async function updateOne<T>(path: string, body: unknown): Promise<T> {
  const res = await put<ApiResponse<T>>(path, body);
  return res.data;
}

/** Upload a file via multipart/form-data. */
export async function postFormData<T>(path: string, formData: FormData): Promise<T> {
  const headers: Record<string, string> = {};
  if (accessToken) {
    headers["Authorization"] = `Bearer ${accessToken}`;
  }

  const res = await fetch(path, {
    method: "POST",
    headers,
    body: formData,
  });

  if (res.status === 401 && refreshToken) {
    const refreshed = await doRefresh();
    if (refreshed) {
      headers["Authorization"] = `Bearer ${accessToken}`;
      const retry = await fetch(path, { method: "POST", headers, body: formData });
      if (!retry.ok) {
        let errorBody: ApiErrorBody | undefined;
        try { errorBody = await retry.json(); } catch { /* not JSON */ }
        throw new ApiError(
          retry.status,
          errorBody?.error?.code ?? "UNKNOWN",
          errorBody?.error?.message ?? `Request failed with status ${retry.status}`,
          errorBody?.error?.details,
        );
      }
      return retry.json();
    }
    clearTokens();
    throw new ApiError(401, "AUTH_EXPIRED", "Session expired. Please log in again.");
  }

  if (!res.ok) {
    let errorBody: ApiErrorBody | undefined;
    try { errorBody = await res.json(); } catch { /* not JSON */ }
    throw new ApiError(
      res.status,
      errorBody?.error?.code ?? "UNKNOWN",
      errorBody?.error?.message ?? `Request failed with status ${res.status}`,
      errorBody?.error?.details,
    );
  }

  return res.json();
}

// ── Budget API ───────────────────────────────────────────────

import type {
  Budget,
  BudgetLine,
  BudgetSummary,
  CopyBudgetRequest,
  ExchangeRate,
  NewBudget,
  NewBudgetLine,
  BulkBudgetLines,
  UpdateBudget,
  UpdateBudgetLine,
} from "./types";

/** List all budgets. */
export async function listBudgets(includeArchived = false): Promise<Budget[]> {
  const qs = includeArchived ? "?include_archived=true" : "";
  const res = await get<PaginatedResponse<Budget>>(`/api/budgets${qs}`);
  return res.data;
}

/** Get a single budget. */
export function getBudget(id: string): Promise<Budget> {
  return fetchOne<Budget>(`/api/budgets/${id}`);
}

/** Create a budget. */
export function createBudget(body: NewBudget): Promise<Budget> {
  return createOne<Budget>("/api/budgets", body);
}

/** Update a budget. */
export function updateBudget(id: string, body: UpdateBudget): Promise<Budget> {
  return updateOne<Budget>(`/api/budgets/${id}`, body);
}

/** Delete a budget. */
export function deleteBudget(id: string): Promise<void> {
  return del<void>(`/api/budgets/${id}`);
}

/** Get planned vs. actual summary for a budget. */
export function getBudgetSummary(id: string): Promise<BudgetSummary> {
  return fetchOne<BudgetSummary>(`/api/budgets/${id}/summary`);
}

/** Copy a budget's lines into a new period. */
export function copyBudget(id: string, body: CopyBudgetRequest): Promise<Budget> {
  return createOne<Budget>(`/api/budgets/${id}/copy`, body);
}

// ── Budget Line API ──────────────────────────────────────────

/** List all lines for a budget. */
export async function listBudgetLines(budgetId: string): Promise<BudgetLine[]> {
  const res = await get<PaginatedResponse<BudgetLine>>(`/api/budgets/${budgetId}/lines`);
  return res.data;
}

/** Add a line to a budget. */
export function addBudgetLine(budgetId: string, body: NewBudgetLine): Promise<BudgetLine> {
  return createOne<BudgetLine>(`/api/budgets/${budgetId}/lines`, body);
}

/** Replace all lines on a budget. */
export async function bulkSetBudgetLines(
  budgetId: string,
  body: BulkBudgetLines,
): Promise<BudgetLine[]> {
  const res = await post<PaginatedResponse<BudgetLine>>(
    `/api/budgets/${budgetId}/lines/bulk`,
    body,
  );
  return res.data;
}

/** Update a budget line. */
export function updateBudgetLine(
  budgetId: string,
  lineId: string,
  body: UpdateBudgetLine,
): Promise<BudgetLine> {
  return updateOne<BudgetLine>(`/api/budgets/${budgetId}/lines/${lineId}`, body);
}

/** Delete a budget line. */
export function deleteBudgetLine(budgetId: string, lineId: string): Promise<void> {
  return del<void>(`/api/budgets/${budgetId}/lines/${lineId}`);
}

// ── Exchange Rate API ────────────────────────────────────────

/** List the latest exchange rates. */
export async function listExchangeRates(): Promise<ExchangeRate[]> {
  const res = await get<PaginatedResponse<ExchangeRate>>("/api/exchange-rates");
  return res.data;
}

/** Trigger a fresh rate fetch from the ECB feed. */
export async function refreshExchangeRates(): Promise<number> {
  const res = await post<ApiResponse<number>>("/api/exchange-rates/refresh");
  return res.data;
}

// ── Reports API ──────────────────────────────────────────────

import type {
  DashboardSummary,
  IncomeExpenseReport,
  CategoryTrendReport,
  BalanceHistoryReport,
  CashFlowReport,
} from "./types";

/** Fetch dashboard summary (net worth, month totals, trend, top categories). */
export async function fetchDashboardSummary(): Promise<DashboardSummary> {
  const res = await get<ApiResponse<DashboardSummary>>("/api/reports/summary");
  return res.data;
}

/** Fetch monthly income vs. expense with category breakdown. */
export async function fetchIncomeExpenseReport(
  from: string,
  to: string,
): Promise<IncomeExpenseReport> {
  const res = await get<ApiResponse<IncomeExpenseReport>>(
    `/api/reports/income-expense?from=${from}&to=${to}`,
  );
  return res.data;
}

/** Fetch monthly spending trend for a single category. */
export async function fetchCategoryTrend(
  categoryId: string,
  from: string,
  to: string,
): Promise<CategoryTrendReport> {
  const res = await get<ApiResponse<CategoryTrendReport>>(
    `/api/reports/categories/${categoryId}/trend?from=${from}&to=${to}`,
  );
  return res.data;
}

/** Fetch historical account balance snapshots. */
export async function fetchBalanceHistory(
  from: string,
  to: string,
  accountIds?: string[],
): Promise<BalanceHistoryReport> {
  const base = `/api/reports/balance-history?from=${from}&to=${to}`;
  const url =
    accountIds && accountIds.length > 0
      ? `${base}&account_ids=${accountIds.join(",")}`
      : base;
  const res = await get<ApiResponse<BalanceHistoryReport>>(url);
  return res.data;
}

/** Fetch cash flow report with 3-month forecast. */
export async function fetchCashFlowReport(
  from: string,
  to: string,
): Promise<CashFlowReport> {
  const res = await get<ApiResponse<CashFlowReport>>(
    `/api/reports/cash-flow?from=${from}&to=${to}`,
  );
  return res.data;
}
