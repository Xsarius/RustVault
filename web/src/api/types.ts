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

// ── Transactions ──────────────────────────────────────────────

export type TransactionType = "income" | "expense" | "transfer";

export interface Transaction {
  id: string;
  user_id: string;
  account_id: string;
  category_id: string | null;
  import_id: string | null;
  transaction_type: TransactionType;
  amount: string; // Decimal as string
  currency: string;
  date: string; // ISO date (YYYY-MM-DD)
  description: string;
  original_desc: string | null;
  payee: string | null;
  reference: string | null;
  notes: string | null;
  is_reviewed: boolean;
  is_deleted: boolean;
  is_duplicate: boolean;
  metadata: Record<string, unknown>;
  tag_ids?: string[];
  created_at: string;
  updated_at: string;
}

export interface NewTransaction {
  account_id: string;
  category_id?: string;
  transaction_type: TransactionType;
  amount: string;
  date: string;
  description: string;
  payee?: string;
  notes?: string;
  tag_ids?: string[];
}

export interface UpdateTransaction {
  category_id?: string | null;
  transaction_type?: TransactionType;
  amount?: string;
  date?: string;
  description?: string;
  payee?: string | null;
  notes?: string | null;
  is_reviewed?: boolean;
  tag_ids?: string[];
}

export interface BulkUpdateTransactions {
  transaction_ids: string[];
  category_id?: string | null;
  is_reviewed?: boolean;
  add_tag_ids?: string[];
}

export interface TransactionListQuery {
  account_id?: string;
  category_id?: string;
  transaction_type?: string;
  date_from?: string;
  date_to?: string;
  q?: string;
  is_reviewed?: boolean;
  tag_id?: string;
  import_id?: string;
  limit?: number;
  cursor?: string;
}

// ── Imports ───────────────────────────────────────────────────

export type ImportStatus =
  | "pending"
  | "processing"
  | "completed"
  | "failed"
  | "rolled_back";

export interface Import {
  id: string;
  user_id: string;
  file_name: string;
  file_format: string;
  account_id: string;
  status: ImportStatus;
  total_rows: number;
  imported_count: number;
  skipped_count: number;
  duplicate_count: number;
  error_count: number;
  error_details: unknown | null;
  column_mapping: unknown | null;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface ParsedRow {
  date: string;
  amount: string;
  currency: string | null;
  description: string;
  payee: string | null;
  reference: string | null;
  metadata: Record<string, unknown>;
}

export interface UploadResponse {
  import: Import;
  detected_format: string;
  preview: ParsedRow[];
  total_rows: number;
}

export interface ConfigureImportRequest {
  mapping: Record<string, unknown>;
}

export interface ExecuteImportRequest {
  mapping?: Record<string, unknown>;
  skip_duplicates?: boolean;
}

export interface ImportExecutionResult {
  import: Import;
  imported_count: number;
  duplicate_count: number;
  error_count: number;
  errors: ImportRowError[];
  rules_applied: Record<string, number>;
}

export interface ImportRowError {
  row: number;
  message: string;
}

// ── Auto-Rules ────────────────────────────────────────────────

export interface AutoRule {
  id: string;
  user_id: string;
  name: string;
  priority: number;
  is_enabled: boolean;
  conditions: unknown;
  actions: unknown;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface NewAutoRule {
  name: string;
  priority?: number;
  conditions: RuleCondition[];
  actions: RuleAction[];
}

export interface UpdateAutoRule {
  name?: string;
  priority?: number;
  is_enabled?: boolean;
  conditions?: RuleCondition[];
  actions?: RuleAction[];
}

export interface RuleCondition {
  field: string;
  value: unknown;
  logic?: "and" | "or";
}

export interface RuleAction {
  type: string;
  value: unknown;
}

export interface TestRuleRequest {
  conditions: unknown;
  description: string;
  payee?: string;
  amount: string;
  account_id: string;
}

export interface TestRuleResponse {
  matched: boolean;
}

export interface SuggestRuleRequest {
  description: string;
  payee?: string;
  amount: string;
}

export interface SuggestRuleResponse {
  name: string;
  conditions: RuleCondition[];
}

// ── i18n ──────────────────────────────────────────────────────

export interface LocaleInfo {
  code: string;
  name: string;
  native_name: string;
  completeness: number;
  is_default: boolean;
}
