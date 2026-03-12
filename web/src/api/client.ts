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
