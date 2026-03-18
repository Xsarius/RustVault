//! OpenAPI specification and Scalar UI setup.

use utoipa::OpenApi;

/// Auto-generated OpenAPI specification for all P1 endpoints.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "RustVault API",
        description = "Self-hosted personal finance management platform",
        version = "0.1.0",
        license(name = "AGPL-3.0-or-later", url = "https://www.gnu.org/licenses/agpl-3.0.html"),
    ),
    tags(
        (name = "Health", description = "Liveness and readiness probes"),
        (name = "Auth", description = "Registration, login, token refresh, and user profile"),
        (name = "Banks", description = "Bank / financial institution management"),
        (name = "Accounts", description = "Account management within banks"),
        (name = "Categories", description = "Hierarchical transaction categories"),
        (name = "Tags", description = "Transaction tags / labels"),
        (name = "Settings", description = "User preferences and configuration"),
        (name = "i18n", description = "Internationalisation — available locales"),
        (name = "Budgets", description = "Budget planning — per-period spending envelopes with per-category lines"),
        (name = "Reports", description = "Visualisation and analysis — dashboard summary, income/expense trends, category analysis, balance history, cash flow"),
    ),
    paths(
        // Health
        crate::routes::health::health,
        // Auth
        crate::routes::auth::register,
        crate::routes::auth::login,
        crate::routes::auth::refresh,
        crate::routes::auth::me,
        // Banks
        crate::routes::banks::list,
        crate::routes::banks::create,
        crate::routes::banks::get,
        crate::routes::banks::update,
        crate::routes::banks::archive,
        // Accounts
        crate::routes::accounts::list,
        crate::routes::accounts::create,
        crate::routes::accounts::get,
        crate::routes::accounts::update,
        crate::routes::accounts::archive,
        // Categories
        crate::routes::categories::list,
        crate::routes::categories::create,
        crate::routes::categories::bulk_create,
        crate::routes::categories::get,
        crate::routes::categories::update,
        crate::routes::categories::delete,
        // Tags
        crate::routes::tags::list,
        crate::routes::tags::create,
        crate::routes::tags::bulk_create,
        crate::routes::tags::get,
        crate::routes::tags::update,
        crate::routes::tags::delete,
        // Settings
        crate::routes::settings::get,
        crate::routes::settings::update,
        // i18n
        crate::routes::i18n::list_locales,
        // Budgets
        crate::routes::budgets::list,
        crate::routes::budgets::create,
        crate::routes::budgets::get,
        crate::routes::budgets::update,
        crate::routes::budgets::delete,
        crate::routes::budgets::summary,
        crate::routes::budgets::copy,
        crate::routes::budgets::add_line,
        crate::routes::budgets::bulk_set_lines,
        crate::routes::budgets::update_line,
        crate::routes::budgets::delete_line,
        // Reports
        crate::routes::reports::summary,
        crate::routes::reports::income_expense,
        crate::routes::reports::category_trend,
        crate::routes::reports::balance_history,
        crate::routes::reports::cash_flow,
    ),
    components(
        schemas(
            // Response wrappers
            crate::response::ErrorBody,
            crate::response::ErrorData,
            crate::response::FieldError,
            crate::response::PaginationMeta,
            // Domain models
            rustvault_core::models::bank::Bank,
            rustvault_core::models::bank::NewBank,
            rustvault_core::models::bank::UpdateBank,
            rustvault_core::models::account::Account,
            rustvault_core::models::account::NewAccount,
            rustvault_core::models::account::UpdateAccount,
            rustvault_core::models::account::AccountType,
            rustvault_core::models::category::Category,
            rustvault_core::models::category::NewCategory,
            rustvault_core::models::category::UpdateCategory,
            rustvault_core::models::category::BulkCreateCategories,
            rustvault_core::models::category::CategoryType,
            rustvault_core::models::tag::Tag,
            rustvault_core::models::tag::NewTag,
            rustvault_core::models::tag::UpdateTag,
            rustvault_core::models::tag::BulkCreateTags,
            rustvault_core::models::user::NewUser,
            rustvault_core::models::user::LoginRequest,
            rustvault_core::models::user::UserInfo,
            rustvault_core::models::user::UserRole,
            rustvault_core::models::user::AuthProvider,
            rustvault_core::models::settings::UserSettings,
            rustvault_core::models::settings::UpdateSettings,
            rustvault_core::i18n::LocaleInfo,
            // Budget models
            rustvault_core::models::budget::Budget,
            rustvault_core::models::budget::NewBudget,
            rustvault_core::models::budget::UpdateBudget,
            rustvault_core::models::budget::BudgetLine,
            rustvault_core::models::budget::NewBudgetLine,
            rustvault_core::models::budget::UpdateBudgetLine,
            rustvault_core::models::budget::BulkBudgetLines,
            rustvault_core::models::budget::BudgetSummary,
            rustvault_core::models::budget::BudgetLineSummary,
            // Server-level request types
            crate::routes::auth::RefreshRequest,
            crate::routes::budgets::CopyBudgetRequest,
            // Report models
            rustvault_core::models::report::DashboardSummary,
            rustvault_core::models::report::MonthlyPoint,
            rustvault_core::models::report::CategorySpend,
            rustvault_core::models::report::IncomeExpenseReport,
            rustvault_core::models::report::IncomeExpenseMonth,
            rustvault_core::models::report::CategoryTrendReport,
            rustvault_core::models::report::TrendPoint,
            rustvault_core::models::report::BalanceHistoryReport,
            rustvault_core::models::report::AccountMeta,
            rustvault_core::models::report::BalanceSnapshot,
            rustvault_core::models::report::AccountBalance,
            rustvault_core::models::report::CashFlowReport,
            rustvault_core::models::report::CashFlowPeriod,
        ),
    ),
    security(
        ("bearer" = []),
    ),
)]
pub struct ApiDoc;
