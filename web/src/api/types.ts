/**
 * Shared API type definitions.
 *
 * Mirrors the Rust backend response shapes so the frontend gets
 * full type-safety without code generation.
 */

// ── Response wrappers ─────────────────────────────────────────

/** Standard single-resource API response. */
export interface ApiResponse<T> {
  data: T;
}

/** Paginated collection API response. */
export interface PaginatedResponse<T> {
  data: T[];
  meta: PaginationMeta;
}

/** Pagination metadata. */
export interface PaginationMeta {
  page_size: number;
  next_cursor?: string;
  has_more: boolean;
}

/** Standard API error response. */
export interface ApiErrorBody {
  error: {
    code: string;
    message: string;
    details?: FieldError[];
  };
}

/** A single field-level validation error. */
export interface FieldError {
  field: string;
  message: string;
  code: string;
}

// ── Auth ──────────────────────────────────────────────────────

export interface AuthTokens {
  access_token: string;
  token_type: string;
  expires_in: number;
  refresh_token: string;
}

export interface UserInfo {
  id: string;
  username: string;
  email: string;
  role: UserRole;
  auth_provider: AuthProvider;
  locale: string;
  timezone: string;
  settings: Record<string, unknown>;
  created_at: string;
}

export type UserRole = "admin" | "member" | "viewer";
export type AuthProvider = "local" | "oidc" | "both";

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
}

export interface RefreshRequest {
  refresh_token: string;
}

export interface OidcConfig {
  enabled: boolean;
  display_name: string;
  authorize_url: string;
}

// ── Banks ─────────────────────────────────────────────────────

export interface Bank {
  id: string;
  user_id: string;
  name: string;
  is_archived: boolean;
  sort_order: number;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface NewBank {
  name: string;
}

export interface UpdateBank {
  name?: string;
}

// ── Accounts ──────────────────────────────────────────────────

export type AccountType =
  | "checking"
  | "savings"
  | "credit"
  | "investment"
  | "loan";

export interface Account {
  id: string;
  user_id: string;
  bank_id: string;
  name: string;
  currency: string;
  type: AccountType;
  balance_cache: string; // Decimal as string
  supports_nonstandard_topup: boolean;
  is_archived: boolean;
  sort_order: number;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface NewAccount {
  bank_id: string;
  name: string;
  currency: string;
  type: AccountType;
  supports_nonstandard_topup?: boolean;
}

export interface UpdateAccount {
  name?: string;
  currency?: string;
  type?: AccountType;
  supports_nonstandard_topup?: boolean;
}

// ── Categories ────────────────────────────────────────────────

export type CategoryType = "expense" | "income";

export interface Category {
  id: string;
  user_id: string;
  name: string;
  parent_id: string | null;
  icon: string | null;
  color: string | null;
  category_type: CategoryType;
  sort_order: number;
  metadata: Record<string, unknown>;
  created_at: string;
}

export interface NewCategory {
  name: string;
  parent_id?: string;
  icon?: string;
  color?: string;
  category_type: CategoryType;
}

export interface UpdateCategory {
  name?: string;
  parent_id?: string | null;
  icon?: string | null;
  color?: string | null;
  category_type?: CategoryType;
}

export interface BulkCreateCategories {
  categories: NewCategory[];
}

// ── Tags ──────────────────────────────────────────────────────

export interface Tag {
  id: string;
  user_id: string;
  name: string;
  color: string | null;
  created_at: string;
}

export interface NewTag {
  name: string;
  color?: string;
}

export interface UpdateTag {
  name?: string;
  color?: string | null;
}

export interface BulkCreateTags {
  tags: NewTag[];
}

// ── Settings ──────────────────────────────────────────────────

export interface UserSettings {
  default_currency: string;
  locale: string;
  date_format: string;
  ai_enabled: boolean;
}

export interface UpdateSettings {
  default_currency?: string;
  locale?: string;
  date_format?: string;
  ai_enabled?: boolean;
}

// ── i18n ──────────────────────────────────────────────────────

export interface LocaleInfo {
  code: string;
  name: string;
  native_name: string;
  completeness: number;
  is_default: boolean;
}
