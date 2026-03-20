/**
 * API module barrel export.
 *
 * Re-exports all API types and the client for convenient imports:
 *   import { api, type UserInfo } from "~/api";
 *
 * In demo mode (VITE_DEMO_MODE=true) the Vite build alias redirects
 * `~/api/client` → `~/api/mock/index` so the real fetch client is
 * replaced by an in-memory mock without changing any call sites.
 */

export * from "./types";
export * as api from "./client";
