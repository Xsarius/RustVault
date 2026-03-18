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

// ── Budgets ───────────────────────────────────────────────────

export interface Budget {
  id: string;
  user_id: string;
  name: string;
  period_start: string; // ISO date (YYYY-MM-DD)
  period_end: string;   // ISO date (YYYY-MM-DD)
  currency: string;
  is_recurring: boolean;
  recurrence_rule: string | null;
  is_archived: boolean;
  notes: string | null;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface NewBudget {
  name: string;
  period_start: string;
  period_end: string;
  currency: string;
  is_recurring?: boolean;
  recurrence_rule?: string;
  notes?: string;
}

export interface UpdateBudget {
  name?: string;
  period_start?: string;
  period_end?: string;
  currency?: string;
  is_recurring?: boolean;
  recurrence_rule?: string | null;
  notes?: string | null;
}

export interface BudgetLine {
  id: string;
  budget_id: string;
  category_id: string | null;
  planned_amount: string; // Decimal as string
  actual_amount_cache: string; // Decimal as string
  notes: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface NewBudgetLine {
  category_id?: string;
  planned_amount: string;
  notes?: string;
  sort_order?: number;
}

export interface UpdateBudgetLine {
  planned_amount?: string;
  notes?: string | null;
  sort_order?: number;
}

export interface BulkBudgetLines {
  lines: NewBudgetLine[];
}

export interface BudgetLineSummary {
  id: string;
  category_id: string | null;
  planned_amount: string;
  actual_amount: string;
  remaining: string;
  percent_used: string;
}

export interface BudgetSummary {
  budget_id: string;
  total_planned_income: string;
  total_actual_income: string;
  total_planned_expenses: string;
  total_actual_expenses: string;
  net_planned: string;
  net_actual: string;
  lines: BudgetLineSummary[];
  over_budget_categories: string[];
}

export interface CopyBudgetRequest {
  name: string;
  period_start: string;
  period_end: string;
}

// ── Exchange Rates ────────────────────────────────────────────

export interface ExchangeRate {
  id: number;
  base_currency: string;
  target_currency: string;
  rate: string; // Decimal as string
  date: string; // ISO date
  source: string;
  fetched_at: string;
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

// ── Reports ───────────────────────────────────────────────────

/** One month's income/expense totals (dashboard trend). */
export interface MonthlyPoint {
  /** ISO date string — first day of the month. */
  month: string;
  income: string;
  expenses: string;
}

/** Total spending for a category in a given period. */
export interface CategorySpend {
  category_id: string | null;
  category_name: string | null;
  total: string;
}

/** Dashboard summary response. */
export interface DashboardSummary {
  net_worth: string;
  month_income: string;
  month_expenses: string;
  /** Null when income is zero. */
  savings_rate: number | null;
  unreviewed_count: number;
  monthly_trend: MonthlyPoint[];
  spending_by_category: CategorySpend[];
}

/** One month in the income/expense report (with category breakdown). */
export interface IncomeExpenseMonth {
  month: string;
  income: string;
  expenses: string;
  breakdown: CategorySpend[];
}

/** Full income vs expense report. */
export interface IncomeExpenseReport {
  months: IncomeExpenseMonth[];
}

/** One period data point in a category trend. */
export interface TrendPoint {
  period: string;
  total: string;
}

/** Monthly spending trend for a single category. */
export interface CategoryTrendReport {
  category_id: string;
  periods: TrendPoint[];
  average: string;
}

/** Lightweight account metadata embedded in balance history. */
export interface AccountMeta {
  id: string;
  name: string;
  currency: string;
}

/** One account's balance on a given date. */
export interface AccountBalance {
  account_id: string;
  balance: string;
}

/** Balance for all requested accounts on a single date. */
export interface BalanceSnapshot {
  date: string;
  balances: AccountBalance[];
  net_worth: string;
}

/** Historical account balance report. */
export interface BalanceHistoryReport {
  accounts: AccountMeta[];
  snapshots: BalanceSnapshot[];
}

/** One period's cash flow (historical or forecast). */
export interface CashFlowPeriod {
  period: string;
  income: string;
  expenses: string;
  net: string;
  is_forecast: boolean;
}

/** Cash flow report with forecast. */
export interface CashFlowReport {
  periods: CashFlowPeriod[];
  avg_income: string;
  avg_expenses: string;
  forecast: CashFlowPeriod[];
}
