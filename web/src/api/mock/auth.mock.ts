/**
 * Demo mode — Auth mock.
 *
 * In demo mode the user is auto-authenticated as a fixed "Demo User".
 * Login always succeeds regardless of credentials. No real JWT is issued.
 */

import { simulate } from "./latency";
import type { AuthTokens, UserInfo } from "~/api/types";

const DEMO_USER: UserInfo = {
  id: "demo-user",
  username: "Demo User",
  email: "demo@rustvault.app",
  role: "member",
  auth_provider: "local",
  locale: "en-US",
  timezone: "Europe/Berlin",
  settings: {},
  created_at: "2025-01-01T00:00:00Z",
};

const DEMO_TOKENS: AuthTokens = {
  access_token: "demo-access-token",
  token_type: "Bearer",
  expires_in: 86400,
  refresh_token: "demo-refresh-token",
};

/** Always succeeds — returns demo tokens. */
export function login(_email: string, _password: string) {
  return simulate({ data: DEMO_TOKENS });
}

/** No-op. */
export function register(_username: string, _email: string, _password: string) {
  return simulate({ data: DEMO_USER });
}

/** Returns the demo user profile. */
export function getMe() {
  return simulate({ data: DEMO_USER });
}

/** Always refreshes successfully. */
export function refreshTokens(_refreshToken: string) {
  return simulate({ data: DEMO_TOKENS });
}

/** No-op. */
export function logout() {
  return simulate(undefined);
}

// ── These are forwarded through the generic helpers below ──────────────────

/** Stub — demo has no real token state. */
export function setTokens(_access: string, _refresh: string) { /* no-op */ }
export function clearTokens() { /* no-op */ }
export function hasTokens() { return true; }
export function setBaseUrl(_url: string) { /* no-op */ }
export function getBaseUrl() { return ""; }
/** ApiError stub — same shape as real client. */
export class ApiError extends Error {
  constructor(public status: number, public code: string, message: string) {
    super(message);
    this.name = "ApiError";
  }
}
