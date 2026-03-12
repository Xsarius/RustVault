/**
 * Authentication store — manages user session state.
 *
 * Provides reactive signals for the current user and auth status.
 * Handles login, register, logout, and token refresh lifecycle.
 */

import { createSignal, createRoot } from "solid-js";
import { api, type UserInfo } from "~/api";

export interface AuthState {
  /** The currently authenticated user, or null. */
  user: UserInfo | null;
  /** True while an auth operation (login/register/refresh) is in progress. */
  loading: boolean;
  /** True if the user is authenticated. */
  isAuthenticated: boolean;
}

function createAuthStore() {
  const [user, setUser] = createSignal<UserInfo | null>(null);
  const [loading, setLoading] = createSignal(false);

  const isAuthenticated = () => user() !== null;

  /** Register a new user. Does NOT auto-login. */
  async function register(username: string, email: string, password: string) {
    setLoading(true);
    try {
      await api.post("/api/auth/register", { username, email, password });
    } finally {
      setLoading(false);
    }
  }

  /** Log in with email + password. Stores tokens and fetches user profile. */
  async function login(email: string, password: string) {
    setLoading(true);
    try {
      const res = await api.post<{ data: { access_token: string; refresh_token: string } }>(
        "/api/auth/login",
        { email, password },
      );
      api.setTokens(res.data.access_token, res.data.refresh_token);
      await fetchMe();
    } finally {
      setLoading(false);
    }
  }

  /** Fetch the current user profile. */
  async function fetchMe() {
    try {
      const info = await api.fetchOne<UserInfo>("/api/auth/me");
      setUser(info);
    } catch {
      setUser(null);
      api.clearTokens();
    }
  }

  /** Log out — clear tokens and user state. */
  function logout() {
    api.clearTokens();
    setUser(null);
  }

  /** Try to restore session from existing tokens (called on app startup). */
  async function restoreSession() {
    if (!api.hasTokens()) return;
    setLoading(true);
    try {
      await fetchMe();
    } finally {
      setLoading(false);
    }
  }

  return {
    user,
    loading,
    isAuthenticated,
    register,
    login,
    logout,
    fetchMe,
    restoreSession,
  };
}

/** Singleton auth store — created once at module level. */
export const authStore = createRoot(createAuthStore);
