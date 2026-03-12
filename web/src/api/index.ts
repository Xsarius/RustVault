/**
 * API module barrel export.
 *
 * Re-exports all API types and the client for convenient imports:
 *   import { api, type UserInfo } from "~/api";
 */

export * from "./types";
export * as api from "./client";
