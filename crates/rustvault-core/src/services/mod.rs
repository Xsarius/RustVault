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
/// Category service — CRUD, bulk creation, hierarchy.
pub mod category;
/// Settings service — read and update user preferences.
pub mod settings;
/// Tag service — CRUD and bulk creation.
pub mod tag;
