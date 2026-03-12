//! Domain models.
//!
//! These are the core domain structs used throughout the application.
//! They are distinct from database row types and are mapped in the repository layer.

/// Account model — checking, savings, credit card, etc.
pub mod account;
/// Audit log entry model.
pub mod audit;
/// Bank / financial institution model.
pub mod bank;
/// Hierarchical category model (income / expense).
pub mod category;
/// Import session model.
pub mod import;
/// Pagination helpers for list endpoints.
pub mod pagination;
/// Auto-categorization rule model.
pub mod rule;
/// Refresh-token session model.
pub mod session;
/// User preferences / settings model.
pub mod settings;
/// Tag model for transaction labelling.
pub mod tag;
/// Transaction model — income, expense, transfer entries.
pub mod transaction;
/// Transfer model — links debit/credit transaction pairs.
pub mod transfer;
/// User identity model.
pub mod user;
