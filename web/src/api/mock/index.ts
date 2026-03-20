/**
 * Demo mode — Mock API barrel.
 *
 * Exports the full mock API surface, matching the shape of client.ts.
 * Consumed by src/api/index.ts when __DEMO_MODE__ is true.
 */

// Low-level helpers (same signatures as client.ts)
export {
  setTokens,
  clearTokens,
  hasTokens,
  setBaseUrl,
  getBaseUrl,
  ApiError,
  get,
  post,
  put,
  patch,
  del,
  fetchOne,
  fetchList,
  createOne,
  updateOne,
  postFormData,
} from "./client.mock";

// Auth
export { login, register, getMe, refreshTokens, logout } from "./auth.mock";

// Banks
export {
  listBanks,
  getBank,
  createBank,
  updateBank,
  deleteBank,
} from "./banks.mock";

// Accounts
export {
  listAccounts,
  getAccount,
  createAccount,
  updateAccount,
  deleteAccount,
} from "./accounts.mock";

// Categories
export {
  listCategories,
  getCategory,
  createCategory,
  updateCategory,
  deleteCategory,
  bulkCreateCategories,
} from "./categories.mock";

// Tags
export {
  listTags,
  getTag,
  createTag,
  updateTag,
  deleteTag,
  bulkCreateTags,
} from "./tags.mock";

// Transactions
export {
  listTransactions,
  getTransaction,
  createTransaction,
  updateTransaction,
  deleteTransaction,
  bulkUpdateTransactions,
} from "./transactions.mock";

// Imports
export {
  uploadFile,
  executeImport,
  listImports,
  getImport,
  rollbackImport,
} from "./imports.mock";

// Rules
export {
  listRules,
  getRule,
  createRule,
  updateRule,
  deleteRule,
  testRule,
} from "./rules.mock";

// Budgets + budget lines
export {
  listBudgets,
  getBudget,
  createBudget,
  updateBudget,
  deleteBudget,
  getBudgetSummary,
  copyBudget,
  generateNextPeriod,
  listBudgetLines,
  addBudgetLine,
  bulkSetBudgetLines,
  updateBudgetLine,
  deleteBudgetLine,
} from "./budgets.mock";

// Reports + exchange rates
export {
  fetchDashboardSummary,
  fetchIncomeExpenseReport,
  fetchCategoryTrend,
  fetchBalanceHistory,
  fetchCashFlowReport,
  listExchangeRates,
  refreshExchangeRates,
} from "./reports.mock";

// Settings
export { getSettings, updateSettings } from "./settings.mock";
