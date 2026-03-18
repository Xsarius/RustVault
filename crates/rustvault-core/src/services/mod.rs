//! Business logic services.
//!
//! Each service orchestrates database calls, rule evaluation,
//! and external integrations for a specific domain area.

/// Account service — CRUD, archiving, balance queries.
pub mod account;
/// Auth service — registration, login, token refresh, OIDC.
pub mod auth;
/// Bank service — CRUD and archiving.
pub mod bank;
/// Budget service — CRUD, summary, copy, recurring budget generation.
pub mod budget;
/// Category service — CRUD, bulk creation, hierarchy.
pub mod category;
/// Exchange rate service — ECB feed fetcher and currency conversion helpers.
pub mod exchange_rate;
/// Import service — list, get, rollback.
pub mod import;
/// Report service — dashboard summary, income/expense, category trend, balance history, cash flow.
pub mod report;
/// Auto-rule service — CRUD for auto-categorization rules.
pub mod rule;
/// Rule engine — condition evaluation, action application, rule suggestions.
pub mod rule_engine;
/// Settings service — read and update user preferences.
pub mod settings;
/// Tag service — CRUD and bulk creation.
pub mod tag;
/// Transaction service — CRUD, search, bulk update, duplicate detection.
pub mod transaction;
/// Transfer service — create, link, unlink, detect matches.
pub mod transfer;
